//! Each corpus program's Rust gives the answer Fixpt gives running it as
//! FX-26 (`tests/kfx/NAME.out`, `scripts/corpus.py`): `PLAN.md`'s K1,
//! program by program.

#[test]
fn every_program_gives_fixpts_answer() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/kfx");
    let mut wrong = Vec::new();
    // This machine's own answer where Fixpt's differs here (C134).
    let platform = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);
    // A collector's stress run (C511, `scripts/gc-stress.sh`) takes some
    // programs alone, at another interval: `KAYAK_CORPUS_ONLY` and
    // `KAYAK_CORPUS_SKIP`, names separated by commas.
    let names = |var: &str| -> Option<Vec<String>> {
        std::env::var(var)
            .ok()
            .map(|v| v.split(',').map(str::to_string).collect())
    };
    let (only, skip) = (names("KAYAK_CORPUS_ONLY"), names("KAYAK_CORPUS_SKIP"));
    for &(name, main) in kfx_corpus::PROGRAMS {
        if only.as_ref().is_some_and(|o| !o.iter().any(|n| n == name))
            || skip.as_ref().is_some_and(|s| s.iter().any(|n| n == name))
        {
            continue;
        }
        let mine = format!("{dir}/{name}.{platform}.out");
        let path = if std::path::Path::new(&mine).exists() {
            mine
        } else {
            format!("{dir}/{name}.out")
        };
        let want = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
        let started = std::time::Instant::now();
        let got = kayak_rt::with_big_stack(main);
        // Each program's time, seen with `--nocapture` (a collector's
        // stress run, C511).
        eprintln!("corpus: {name} {:.2} s", started.elapsed().as_secs_f64());
        if got.as_str() != want {
            wrong.push(format!("{name}:\n  Fixpt: {want:?}\n  Rust:  {got:?}"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// `tails.kfx`'s procedures loop: run on a stack of 256 KiB, as no
/// recursion three million deep could be.
#[test]
fn self_tail_calls_run_in_constant_stack() {
    let (_, main) = kfx_corpus::PROGRAMS
        .iter()
        .find(|(name, _)| *name == "tails")
        .expect("tails.kfx is in the corpus");
    let main = *main;
    let got = std::thread::Builder::new()
        .stack_size(256 << 10)
        .spawn(main)
        .unwrap()
        .join()
        .expect("tails.kfx overflows a small stack: its self tail calls do not loop");
    assert!(got.as_str().starts_with("4499998500000 "), "{got}");
}
