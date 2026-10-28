//! With the `heap-stats` feature, what the heap was given, by Rust type:
//! how many allocations and how many bytes, written to standard error by
//! [`heap_stats_report`]. Without it, nothing, and no cost.

#[cfg(feature = "heap-stats")]
mod on {
    use std::collections::HashMap;
    use std::sync::Mutex;

    static TABLE: Mutex<Option<HashMap<&'static str, (u64, u64)>>> = Mutex::new(None);

    static SAMPLES: Mutex<Option<HashMap<String, u64>>> = Mutex::new(None);

    pub fn note(name: &'static str, bytes: usize) {
        sample(name, bytes);
        let mut t = TABLE.lock().unwrap_or_else(|e| e.into_inner());
        let e = t
            .get_or_insert_with(HashMap::new)
            .entry(name)
            .or_insert((0, 0));
        e.0 += 1;
        e.1 += bytes as u64;
    }

    /// With `KAYAK_HEAP_SAMPLE=TYPE`, every millionth allocation of the
    /// type named so (`*`: of any) (every `KAYAK_HEAP_EVERY`th, if that is set): the
    /// program's frames that made it (the KFX compiler's or the engine's,
    /// `KAYAK_HEAP_DEPTH` of them, 4 if not set), a line each, with the
    /// bytes the samples there were given.
    fn sample(name: &'static str, bytes: usize) {
        use std::sync::atomic::{AtomicU64, Ordering};
        static WANTED: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
        static COUNT: AtomicU64 = AtomicU64::new(0);
        let Some(w) = WANTED.get_or_init(|| std::env::var("KAYAK_HEAP_SAMPLE").ok()) else {
            return;
        };
        static EVERY: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
        let every = *EVERY.get_or_init(|| {
            std::env::var("KAYAK_HEAP_EVERY")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1_000_000)
        });
        if (w != "*" && name != w) || COUNT.fetch_add(1, Ordering::Relaxed) % every != 0 {
            return;
        }
        let bt = std::backtrace::Backtrace::force_capture().to_string();
        let frames: Vec<&str> = bt
            .lines()
            .map(str::trim)
            .filter(|l| {
                (l.contains("kfx::kfx::") || l.contains("kayak_fx26::fx26::"))
                    && !l.starts_with("at ")
            })
            .map(|l| l.split_once(": ").map_or(l, |(_, f)| f))
            .take(depth())
            .collect();
        let key = frames.join(" < ");
        let mut t = SAMPLES.lock().unwrap_or_else(|e| e.into_inner());
        *t.get_or_insert_with(HashMap::new).entry(key).or_insert(0) += bytes as u64;
    }

    fn depth() -> usize {
        static DEPTH: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
        *DEPTH.get_or_init(|| {
            std::env::var("KAYAK_HEAP_DEPTH")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(4)
        })
    }

    pub fn report() {
        if let Some(s) = SAMPLES.lock().unwrap_or_else(|e| e.into_inner()).as_ref() {
            let mut rows: Vec<_> = s.iter().collect();
            rows.sort_by(|a, b| b.1.cmp(a.1));
            for (k, n) in rows.iter().take(25) {
                eprintln!("sample {:>8.1} MB  {k}", **n as f64 / 1e6);
            }
        }
        let t = TABLE.lock().unwrap_or_else(|e| e.into_inner());
        let Some(t) = t.as_ref() else { return };
        let mut rows: Vec<_> = t.iter().collect();
        rows.sort_by(|a, b| b.1.1.cmp(&a.1.1));
        let (n, b) = rows
            .iter()
            .fold((0, 0), |(n, b), (_, (m, c))| (n + m, b + c));
        eprintln!("heap: {n} allocations, {:.1} MB", b as f64 / 1e6);
        for (name, (m, c)) in rows.iter().take(60) {
            eprintln!("{:>10.1} MB {:>11} {name}", *c as f64 / 1e6, m);
        }
    }
}

/// That the heap was given a `T`, or `bytes` of one named `name`.
#[inline]
pub(crate) fn note<T>(_bytes: usize) {
    #[cfg(feature = "heap-stats")]
    on::note(std::any::type_name::<T>(), _bytes);
}

/// The table, if the feature is on.
pub fn heap_stats_report() {
    #[cfg(feature = "heap-stats")]
    on::report();
}
