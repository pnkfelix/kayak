//! The frames' census (`KAYAK_GC_FRAMES=PATH`; `TODO.md` §54 step 5): at
//! each collection, the thread's frames found by their frame pointers (the
//! workspace builds with `force-frame-pointers`), and of each call site
//! (a return address) met in 32 frames or more, how many, how many of the
//! stack's words in them reach an object of the collection's, and how many
//! pages those pin. A deep recursion's frames each hold a few live words,
//! of objects made at different times, on different pages: what pins most
//! of a major collection's pages (kc's `located-nodes`, C524). Appended to
//! `PATH`, with the process's executable mappings, for
//! `scripts/deep-frames.py`, which names each site's procedure.

use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::sync::{Mutex, OnceLock};

/// A call site's frames, the stack words in them that pin, and the pages
/// those pin.
type Site = (usize, usize, HashSet<(usize, usize)>);

/// A call site met in fewer frames is left out.
const DEEP: usize = 32;

/// Where the census goes, if asked for.
pub(super) fn on() -> bool {
    out().is_some()
}

fn out() -> Option<&'static Mutex<std::fs::File>> {
    static OUT: OnceLock<Option<Mutex<std::fs::File>>> = OnceLock::new();
    OUT.get_or_init(|| {
        let path = std::env::var_os("KAYAK_GC_FRAMES")?;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .ok()?;
        let maps = std::fs::read_to_string("/proc/self/maps").unwrap_or_default();
        let mut s = format!("process {}\n", std::process::id());
        for l in maps.lines().filter(|l| l.contains("r-xp")) {
            s.push_str("map ");
            s.push_str(l);
            s.push('\n');
        }
        f.write_all(s.as_bytes()).ok()?;
        Some(Mutex::new(f))
    })
    .as_ref()
}

/// This frame's frame pointer.
#[inline(always)]
fn frame_pointer() -> usize {
    let fp: usize;
    #[cfg(target_arch = "x86_64")]
    // SAFETY: a register read.
    unsafe {
        std::arch::asm!("mov {}, rbp", out(reg) fp, options(nomem, nostack, preserves_flags));
    }
    #[cfg(target_arch = "aarch64")]
    // SAFETY: a register read.
    unsafe {
        std::arch::asm!("mov {}, x29", out(reg) fp, options(nomem, nostack, preserves_flags));
    }
    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    {
        fp = 0;
    }
    fp
}

/// The census of one collection: `words` the stack's as `roots_from` read
/// them (the 12 registers spilled, then each word from `from` up), and
/// `pages` the pages a word pins, as the collection pins them.
#[inline(never)]
pub(super) fn census(
    from: usize,
    words: &[usize],
    minor: bool,
    pinned: usize,
    mut pages: impl FnMut(usize, &mut Vec<(usize, usize)>),
) {
    let Some(out) = out() else { return };
    let start = (from + 7) & !7;
    let end = start + 8 * words.len().saturating_sub(12);
    // The frames from `from` up: each frame pointer, and the return
    // address beside it, into the procedure whose frame is above it.
    let mut chain: Vec<(usize, usize)> = Vec::new();
    let mut f = frame_pointer();
    while f != 0 && f & 7 == 0 && f + 16 <= end {
        if f >= start {
            // SAFETY: a frame record of this thread's stack, between its
            // collection's frame and its base: two words, read.
            let ret = unsafe { std::ptr::with_exposed_provenance::<usize>(f + 8).read() };
            chain.push((f, ret));
        }
        // SAFETY: as above.
        let next = unsafe { std::ptr::with_exposed_provenance::<usize>(f).read() };
        if next <= f {
            break;
        }
        f = next;
    }
    // Each site's frames: the words from just above one frame's record
    // to just above the next's are the frame of the procedure the first's
    // return address is in.
    let mut sites: HashMap<usize, Site> = HashMap::new();
    let mut found = Vec::new();
    for (k, &(fp, ret)) in chain.iter().enumerate() {
        let hi = chain.get(k + 1).map_or(end, |&(n, _)| (n + 16).min(end));
        let site = sites.entry(ret).or_default();
        site.0 += 1;
        let mut at = fp + 16;
        while at < hi {
            found.clear();
            pages(words[12 + (at - start) / 8], &mut found);
            if !found.is_empty() {
                site.1 += 1;
                site.2.extend(found.iter().copied());
            }
            at += 8;
        }
    }
    let mut deep: Vec<_> = sites.into_iter().filter(|(_, s)| s.0 >= DEEP).collect();
    deep.sort_by_key(|(_, s)| std::cmp::Reverse(s.2.len()));
    let mut s = format!(
        "collection {} {} pinned {} words {} frames {}\n",
        std::process::id(),
        if minor { "minor" } else { "major" },
        pinned,
        words.len(),
        chain.len()
    );
    for (ret, (n, w, p)) in deep {
        s.push_str(&format!(
            "site {ret:#x} frames {n} words {w} pages {}\n",
            p.len()
        ));
    }
    let _ = out
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .write_all(s.as_bytes());
}
