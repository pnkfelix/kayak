//! A thread's collection of its pages (C511; `TODO.md` §54, a collector's
//! fourth step): Bartlett's mostly-copying collector, over objects of
//! Fixpt's layout, read by their headers alone (`laid`).
//!
//! - **The roots** (`roots::roots_from`) are ambiguous, each a word that
//!   may be a pointer: the registers and stack, the thread's owned blocks
//!   and thread locals, and its kept objects and an ended thread's, whole. An allocation one
//!   reaches is pinned, its page with it: nothing on a pinned page moves.
//!   So is the allocation before a word that is an allocation's first, as
//!   an object's word with no suffix points one past it.
//! - **A constant's** every allocation it reaches is pinned, not only the
//!   first: any thread may read it, as it runs.
//! - **The heap's words are precise**: an object's fields, each a Fixpt
//!   word, found by its header. Cheney's copy (C523): each field of what
//!   is reached is taken in turn; its target, if on a page of this
//!   thread's that nothing pins, copied to a new page (once: a forwarding
//!   word left in its first, Fixpt's tag 7, as `kcrt-core`'s collector
//!   and Fixpt's leave one), the field made to point at the copy, its
//!   offset into its allocation kept; else marked and scanned in place.
//!   The copies are scanned in the order made, Cheney's queue.
//! - **Other threads run as one collects** (C512): a live thread's object
//!   is read, never written, and what it holds of this thread's pinned;
//!   an ended thread's is written where it points to what moved, never
//!   moved. What another thread may reach of this one's is pinned whole,
//!   as a constant's is: what crosses is transmissible, never written, so
//!   it is what the other threads' lists (`roots::Reach`) reach, and what
//!   shared kept objects hold (a mailbox's messages, an I-cell's value,
//!   a group's failure closure). And each collection tells what it found
//!   the thread reaching of others', for theirs: each object of
//!   another's, the first on each path from the thread's own.
//! - **The pages copied from** are given back (`space::end_collection`);
//!   the string cursor, keyed by address, forgotten. A large object is
//!   never moved, and freed once no collection of its thread's reaches it
//!   (C513); so is what kayak-rt keeps (a continuation, a mailbox, an
//!   eq table: `roots::kept_collected`), found where a word reaches it, a
//!   root's ambiguous one or a plain word of an object (C508), as each
//!   thread's list tells what it reaches of another's. An ended thread's
//!   pages are the next collecting thread's (`space::adopt_orphans`).
//!
//! Every pin is known before anything moves (Bartlett's): the ambiguous
//! roots first, and what every kept object of this thread's holds (one
//! may be found late, by a field the copy reaches, and what it holds must
//! not have moved by then; a dead one's float a collection longer).

use super::space::{self, View, Whose};
use super::{HeapWord, laid};
use std::collections::HashSet;
use std::hash::{BuildHasherDefault, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};

/// Fixpt's forwarding word's tag.
const TAG_FORWARD: u64 = kayak_layout::tag("forward");
const TAG_MASK: u64 = kayak_layout::TAG_MASK;
const TAG_PAIR: u64 = kayak_layout::tag("pair");
const TAG_OBJECT: u64 = kayak_layout::tag("bloblet");
const TAG_HEADER: u64 = kayak_layout::tag("header");

/// Addresses, hashed by a multiply: they are spread already.
#[derive(Default)]
struct Addresses(u64);

impl Hasher for Addresses {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0.rotate_left(8) ^ u64::from(b)).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        }
    }
    fn write_usize(&mut self, n: usize) {
        self.0 = (self.0 ^ n as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    }
}

type Set<T> = HashSet<T, BuildHasherDefault<Addresses>>;

/// A collection's count, for a test or a report.
static COLLECTIONS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn collections() -> u64 {
    COLLECTIONS.load(std::sync::atomic::Ordering::Relaxed)
}

/// What the thread's last collection did.
#[derive(Clone, Copy, Default, Debug)]
pub(crate) struct Last {
    pub reached: usize,
    pub moved: usize,
    pub pinned_pages: usize,
    pub freed_pages: usize,
}

thread_local!(static LAST: std::cell::Cell<Last> = const { std::cell::Cell::new(Last { reached: 0, moved: 0, pinned_pages: 0, freed_pages: 0 }) });

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn last() -> Last {
    LAST.get()
}

/// A collection of this thread's pages, now. Its frame a small one, a
/// safepoint's, as `roots::reached`'s: the registers spilled there, the
/// stack read from there.
#[inline(never)]
pub(crate) fn collect() {
    collect_as(false);
}

/// A minor collection of this thread's pages, now (TODO §54 step 4b's
/// (d)): its young pages alone, its old objects neither followed nor
/// moved; the slots the barrier noted, and the fields of what it adopted
/// since its last collection, roots that are made to point where what
/// they hold went. Its thread's alone: no other thread reads its objects.
#[inline(never)]
pub(crate) fn collect_minor() {
    collect_as(true);
}

#[inline(never)]
fn collect_as(minor: bool) {
    let mut saved = [0usize; 12];
    super::roots::spill(&mut saved);
    let saved = std::hint::black_box(saved);
    collect_from(&saved, std::ptr::from_ref(&saved).addr(), minor);
    std::hint::black_box(&saved);
}

#[inline(never)]
fn collect_from(saved: &[usize; 12], from: usize, minor: bool) {
    let mut timer = Timer::start();
    let remembered = space::remembered_len();
    space::begin_collection();
    // What threads ended with no heir left (a process's pages), this
    // thread's to collect from now.
    space::adopt_orphans();
    // A minor collection's: the young large objects; the slots the barrier
    // noted (the store buffer emptied at the collection's end), and each
    // field of what was adopted since the last collection (old, and what
    // they hold of this thread's young objects no barrier saw).
    let young_large: Set<usize> = if minor {
        space::young_large().into_iter().collect()
    } else {
        Set::default()
    };
    let mut noted = if minor {
        space::remembered()
    } else {
        Vec::new()
    };
    if minor {
        let view = View::new();
        let (pages, large) = space::received();
        let mut field = |a: usize| {
            each_object(&view, a, |o| {
                for f in o.fields {
                    noted.push(std::ptr::from_ref(f).addr());
                }
            });
        };
        for (c, p) in pages {
            c.each_start(p, &mut field);
        }
        for a in large {
            field(a);
        }
    }
    // Each slot once: one noted twice would be read, the second time,
    // pointing into to-space.
    noted.sort_unstable();
    noted.dedup();
    let roots = super::roots::roots_from(saved, from);
    let mine = space::my_pages();
    let unsettled = space::unsettled();
    if space::verifying() {
        verify_barrier(&View::new(), &mine, &unsettled);
    }
    timer.lap();
    let young_at_start = if minor {
        mine.iter().filter(|&&(c, p)| c.is_young(p)).count()
    } else {
        0
    };
    let mut m = Mark {
        view: View::new(),
        kept: &roots.kept,
        kept_reached: Set::default(),
        kept_whole: Set::default(),
        kept_work: Vec::new(),
        marked: 0,
        others: Set::default(),
        pinned: Set::default(),
        work: Vec::new(),
        whole: Set::default(),
        noting: false,
        report: Set::default(),
        found: Vec::new(),
        found_kept: Vec::new(),
        fields: Vec::new(),
        minor,
        young_large: &young_large,
        moving: false,
        checked: space::verifying(),
        copies: Vec::new(),
        moved_words: 0,
    };
    // The pins, every one before anything moves (Bartlett's): first what
    // every thread may read, and what another may take as this one runs
    // (C512), each allocation it reaches pinned: the constants, what
    // shared kept objects hold, the other threads' lists.
    for &w in &roots.constants {
        m.ambiguous(w, Way::Whole);
    }
    m.drain_whole();
    let constants = m.pinned.len();
    m.noting = true;
    for &w in &roots.shared {
        m.ambiguous(w, Way::Whole);
    }
    m.drain_whole();
    m.noting = false;
    // Then the ambiguous roots, each allocation one reaches pinned and
    // queued to be scanned in place, not yet followed.
    let mut pins = [m.pinned.len(); 5];
    for (i, &w) in roots.words.iter().enumerate() {
        m.ambiguous(w, Way::Own);
        for (k, &e) in roots.ends.iter().enumerate() {
            if i + 1 == e {
                pins[k + 1] = m.pinned.len();
            }
        }
    }
    pins[4] = m.pinned.len();
    // And what this thread's kept objects hold, by ambiguous words too:
    // one may be found only by a field the copy reaches, late, and what
    // it holds must not have moved by then. A minor collection, which
    // follows no old object that may reach one, takes each as a root; a
    // major one only pins what each holds, and scans one only once a word
    // reaches it, which is what frees one no word does (C513).
    let kept_mine: Vec<_> = roots.kept.mine_unshared().collect();
    for k in kept_mine {
        if minor {
            m.kept_visit(k, Way::Own);
        } else {
            k.words(&mut |w| m.pin_word(w));
        }
    }
    m.drain_kept();
    if super::frames::on() {
        let pinned = m.pinned.len();
        super::frames::census(
            from,
            &roots.words[..roots.ends[0]],
            minor,
            pinned,
            |w, out| {
                m.word_pages(w, |_, k| out.push(k));
            },
        );
    }
    if verbose() {
        eprintln!(
            "kayak: pinned pages: constants {}, shared {}, stack {}, owned {}, kept {}, locals {}, kept objects {}; words: stack {}, owned {}, kept {}, shared {}; others' objects reached {}",
            constants,
            pins[0] - constants,
            pins[1] - pins[0],
            pins[2] - pins[1],
            pins[3] - pins[2],
            pins[4] - pins[3],
            m.pinned.len() - pins[4],
            roots.ends[0],
            roots.ends[1] - roots.ends[0],
            roots.ends[2] - roots.ends[1],
            roots.shared.len(),
            m.report.len(),
        );
    }

    // From-space: this thread's pages that nothing pins, but those adopted
    // that a collection on another thread may still write.
    let from_pages: Vec<_> = mine
        .iter()
        .copied()
        .filter(|&(c, p)| {
            let k = space::page_key(c, p);
            !m.pinned.contains(&k) && !unsettled.contains(&k) && (!minor || c.is_young(p))
        })
        .collect();
    for &(c, p) in &from_pages {
        c.set_from(p, true);
    }
    timer.lap();

    // Cheney's copy: each field of what is reached, copied if its target
    // is in from-space (once, a forwarding word left in its first word),
    // made to point where its target went as it is scanned; each copy
    // scanned in its turn.
    m.moving = true;
    // A minor collection's noted slots first, each of an old object of
    // this thread's (a young one's is scanned as its object is), taken as
    // a field is. A slot no longer of the space (an arena's dropped, a
    // stack's) left.
    for &slot in &noted {
        let Some(o) = m.view.object_of(slot) else {
            continue;
        };
        if young(&m.view, &young_large, o) || m.view.whose_of(o) != Whose::Mine {
            continue;
        }
        // SAFETY: a word of an allocation of this thread's, read.
        let w = unsafe { std::ptr::with_exposed_provenance::<u64>(slot).read() };
        let new = m.field(w, Way::Own);
        if new != w {
            // SAFETY: as above, the slot written, by this thread alone.
            unsafe { space::word_mut(&m.view, slot).write(new) };
        }
    }
    m.cheney();
    timer.lap();

    let Mark {
        view,
        marked,
        others,
        pinned,
        report,
        kept_reached,
        copies,
        moved_words,
        ..
    } = m;
    let moved = copies.len();
    drop(copies);
    // What this thread reaches of others', for their collections; not
    // moved by this one, so told now.
    // A minor collection follows no old object, so what one of them
    // reaches of others' kept objects it does not find: added to the list.
    super::roots::reported(report.into_iter().collect(), roots.reach_from, minor);
    // This thread's large objects not reached, freed (once no collection
    // running may read them); by a minor collection, its young ones alone.
    space::free_large(|a| others.contains(&a) || (minor && !young_large.contains(&a)));
    // Each allocation left on a page it pinned that it did not reach,
    // dead: its fields cleared (fixnum 0), so that a stale word that
    // reaches it later (an ambiguous root's) keeps nothing more alive and
    // reads no word into a page freed and taken again since. Not on a
    // page adopted that another collection may still write.
    // Each mark cleared too, so that none is left for the next collection
    // to take as reached: made only on a page pinned, an unsettled one, or
    // one of to-space (an object reached in place there).
    for &(c, p) in &mine {
        let k = space::page_key(c, p);
        if unsettled.contains(&k) {
            c.clear_marks(p);
        }
        if unsettled.contains(&k) || !pinned.contains(&k) {
            continue;
        }
        c.each_start(p, |a| {
            if c.is_marked(a) {
                return;
            }
            each_object(&view, a, |o| {
                for f in o.fields {
                    // SAFETY: a field of a dead object of this thread's
                    // page, which no one reads but a stale word's
                    // collection.
                    unsafe {
                        space::word_mut(&view, std::ptr::from_ref(f).addr()).write(0);
                    }
                }
            });
        });
        c.clear_marks(p);
    }
    // To-space: the pages taken since the collection began, after those it
    // began with.
    let now = space::my_pages();
    debug_assert!(
        now.len() >= mine.len(),
        "a page given up as a collection runs"
    );
    for &(c, p) in &now[mine.len().min(now.len())..] {
        c.clear_marks(p);
    }
    for &(c, p) in &from_pages {
        c.set_from(p, false);
    }
    LAST.set(Last {
        reached: marked + others.len() + moved,
        moved,
        pinned_pages: pinned.len(),
        freed_pages: from_pages.len(),
    });
    if verbose() {
        let l = LAST.get();
        let (kn, kb) = super::roots::kept_extent();
        let (_, pages, large) = space::extent();
        eprintln!(
            "kayak: collected: {} reached, {} moved, {} pages pinned, {} freed; space {} pages, large {} B; kept {} objects, {} B",
            l.reached, l.moved, l.pinned_pages, l.freed_pages, pages, large, kn, kb
        );
    }
    space::end_collection(&from_pages, minor);
    if minor && stats_on() {
        let r = Ordering::Relaxed;
        STATS.minor_young_pages.fetch_add(young_at_start as u64, r);
        STATS.minor_moved_words.fetch_add(moved_words as u64, r);
        let pinned_young = mine
            .iter()
            .filter(|&&(c, p)| pinned.contains(&space::page_key(c, p)))
            .count();
        STATS.minor_pinned_pages.fetch_add(pinned_young as u64, r);
    }
    // What this thread keeps that no word reached (what was kept since
    // the roots were read, left), freed once no collection running may
    // scan it: after the collection, as a drop may allocate. Not after a
    // minor one, which takes every one as a root.
    if !minor {
        super::roots::retire_kept(|k| {
            kept_reached.contains(&k) || roots.kept.find(k).is_none_or(|i| i.start != k || !i.mine)
        });
    }
    COLLECTIONS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    timer.lap();
    let last = LAST.get();
    timer.done(&last, roots.ends[0], remembered, minor);
}

/// The verifier's (`space::verifying`), as a collection begins: every old
/// object of this thread's, each field of it pointing to a young object of
/// the thread's a slot the barrier noted; but what was adopted since the
/// last collection (`space::received`: what it holds of this thread's
/// young objects no barrier saw), and a page adopted still unsettled.
#[inline(never)]
#[cold]
fn verify_barrier(
    view: &View,
    mine: &[(&'static space::Chunk, usize)],
    unsettled: &std::collections::HashSet<(usize, usize)>,
) {
    let remembered: Set<usize> = space::remembered().into_iter().collect();
    let young_large: Set<usize> = space::young_large().into_iter().collect();
    let (got, got_large) = space::received();
    let got: Set<(usize, usize)> = got
        .into_iter()
        .map(|(c, p)| space::page_key(c, p))
        .collect();
    let got_large: Set<usize> = got_large.into_iter().collect();
    let check = |a: usize| {
        each_object(view, a, |o| {
            for f in o.fields {
                let slot = std::ptr::from_ref(f).addr();
                if let Some(t) = target(view, *f)
                    && young(view, &young_large, t)
                    && view.whose_of(t) == Whose::Mine
                    && !remembered.contains(&slot)
                {
                    missed(a, slot, t);
                }
            }
        });
    };
    for &(c, p) in mine {
        let k = space::page_key(c, p);
        if c.is_young(p) || got.contains(&k) || unsettled.contains(&k) {
            continue;
        }
        c.each_start(p, check);
    }
    for a in view.my_large() {
        if !young_large.contains(&a) && !got_large.contains(&a) {
            check(a);
        }
    }
}

/// `KAYAK_GC_STATS=1`: each collection's phases timed, summed over the
/// run's threads, and said when the run ends (`report_stats`).
fn stats_on() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var("KAYAK_GC_STATS").is_ok_and(|v| v != "0"))
}

/// The phases a collection is timed by: its roots read (and orphans
/// adopted), its pins, its copy (Cheney's, its fields fixed as it goes),
/// its end (large and kept objects freed, pages given back).
const PHASES: [&str; 4] = ["roots", "pin", "copy", "end"];

/// The run's sums, in nanoseconds and counts, over every thread's
/// collections.
struct Stats {
    phase_ns: [AtomicU64; PHASES.len()],
    max_ns: AtomicU64,
    reached: AtomicU64,
    moved: AtomicU64,
    pinned_pages: AtomicU64,
    freed_pages: AtomicU64,
    stack_words: AtomicU64,
    /// The store buffer's slots at each collection's start, summed.
    remembered: AtomicU64,
    /// The minor collections, and their pauses' sum; their young pages as
    /// each began, the words each copied, the young pages each pinned
    /// (what survived, by bytes).
    minors: AtomicU64,
    minor_ns: AtomicU64,
    minor_young_pages: AtomicU64,
    minor_moved_words: AtomicU64,
    minor_pinned_pages: AtomicU64,
    // Pauses by length: below 1 ms, 10 ms, 100 ms, and above.
    buckets: [AtomicU64; 4],
}

static STATS: Stats = Stats {
    phase_ns: [const { AtomicU64::new(0) }; PHASES.len()],
    max_ns: AtomicU64::new(0),
    reached: AtomicU64::new(0),
    moved: AtomicU64::new(0),
    pinned_pages: AtomicU64::new(0),
    freed_pages: AtomicU64::new(0),
    stack_words: AtomicU64::new(0),
    remembered: AtomicU64::new(0),
    minors: AtomicU64::new(0),
    minor_ns: AtomicU64::new(0),
    minor_young_pages: AtomicU64::new(0),
    minor_moved_words: AtomicU64::new(0),
    minor_pinned_pages: AtomicU64::new(0),
    buckets: [const { AtomicU64::new(0) }; 4],
};

/// A collection's times, phase by phase.
struct Timer(Option<(std::time::Instant, [u64; PHASES.len()], usize)>);

impl Timer {
    fn start() -> Self {
        Timer(stats_on().then(|| (std::time::Instant::now(), [0; PHASES.len()], 0)))
    }
    /// The phase now ended.
    fn lap(&mut self) {
        if let Some((t, ns, i)) = &mut self.0 {
            let now = std::time::Instant::now();
            ns[*i] = (now - *t).as_nanos() as u64;
            *t = now;
            *i += 1;
        }
    }
    fn done(self, l: &Last, stack_words: usize, remembered: usize, minor: bool) {
        let Some((_, ns, _)) = self.0 else { return };
        let r = Ordering::Relaxed;
        for (s, n) in STATS.phase_ns.iter().zip(ns) {
            s.fetch_add(n, r);
        }
        let total: u64 = ns.iter().sum();
        STATS.max_ns.fetch_max(total, r);
        if minor {
            STATS.minors.fetch_add(1, r);
            STATS.minor_ns.fetch_add(total, r);
        }
        let b = match total {
            0..1_000_000 => 0,
            1_000_000..10_000_000 => 1,
            10_000_000..100_000_000 => 2,
            _ => 3,
        };
        STATS.buckets[b].fetch_add(1, r);
        STATS.reached.fetch_add(l.reached as u64, r);
        STATS.moved.fetch_add(l.moved as u64, r);
        STATS.pinned_pages.fetch_add(l.pinned_pages as u64, r);
        STATS.freed_pages.fetch_add(l.freed_pages as u64, r);
        STATS.stack_words.fetch_add(stack_words as u64, r);
        STATS.remembered.fetch_add(remembered as u64, r);
        if verbose() {
            eprintln!(
                "kayak: pause {} us: {}",
                total / 1000,
                PHASES
                    .iter()
                    .zip(ns)
                    .map(|(p, n)| format!("{p} {}", n / 1000))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }
}

/// The run's collections, said on standard error under `KAYAK_GC_STATS=1`:
/// for a program's driver to call as it ends (`crate::heap_stats_report`).
pub(crate) fn report_stats() {
    if !stats_on() {
        return;
    }
    let r = Ordering::Relaxed;
    let n = COLLECTIONS.load(r);
    let ns: Vec<u64> = STATS.phase_ns.iter().map(|a| a.load(r)).collect();
    let total: u64 = ns.iter().sum();
    let ms = |x: u64| x as f64 / 1e6;
    let per = |x: u64| if n == 0 { 0.0 } else { x as f64 / n as f64 };
    eprintln!(
        "kayak-gc: {n} collections, {:.1} ms in all, mean {:.3} ms, max {:.3} ms; pauses <1 ms {}, <10 ms {}, <100 ms {}, more {}",
        ms(total),
        per(total) / 1e6,
        ms(STATS.max_ns.load(r)),
        STATS.buckets[0].load(r),
        STATS.buckets[1].load(r),
        STATS.buckets[2].load(r),
        STATS.buckets[3].load(r),
    );
    let minors = STATS.minors.load(r);
    eprintln!(
        "kayak-gc: {minors} minor, {:.1} ms; {} major, {:.1} ms",
        ms(STATS.minor_ns.load(r)),
        n - minors,
        ms(total - STATS.minor_ns.load(r)),
    );
    if minors > 0 {
        let young = STATS.minor_young_pages.load(r) as f64 * space::PAGE_BYTES as f64;
        let moved = STATS.minor_moved_words.load(r) as f64 * 8.0;
        let pinned = STATS.minor_pinned_pages.load(r) as f64 * space::PAGE_BYTES as f64;
        eprintln!(
            "kayak-gc: minor survival: {:.1}% of the young bytes copied, {:.1}% on young pages pinned whole ({:.1} pages a minor)",
            100.0 * moved / young.max(1.0),
            100.0 * pinned / young.max(1.0),
            STATS.minor_pinned_pages.load(r) as f64 / minors as f64,
        );
    }
    eprintln!(
        "kayak-gc: phases (ms): {}",
        PHASES
            .iter()
            .zip(&ns)
            .map(|(p, &x)| format!("{p} {:.1}", ms(x)))
            .collect::<Vec<_>>()
            .join(", ")
    );
    eprintln!(
        "kayak-gc: per collection: {:.0} reached, {:.0} moved, {:.1} pages pinned, {:.1} freed, {:.0} stack words, {:.0} slots remembered",
        per(STATS.reached.load(r)),
        per(STATS.moved.load(r)),
        per(STATS.pinned_pages.load(r)),
        per(STATS.freed_pages.load(r)),
        per(STATS.stack_words.load(r)),
        per(STATS.remembered.load(r)),
    );
}

/// `KAYAK_GC_VERBOSE=1`: each collection said.
fn verbose() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var("KAYAK_GC_VERBOSE").is_ok_and(|v| v != "0"))
}

/// Each object of the allocation at `start`, as `laid::allocation` finds
/// them, with no `Vec`: one, or a datatype's value's two (C500).
#[inline]
pub(super) fn each_object(view: &View, start: usize, mut f: impl FnMut(&laid::Laid)) {
    // Where the allocation may end at the most: its page's end, or its
    // large object's.
    let (chunk, limit) = match view.page_of(start) {
        Some((c, _)) => (
            Some(c),
            (start & !(space::PAGE_BYTES - 1)) + space::PAGE_BYTES,
        ),
        None => match view.large_owner(start) {
            Some((p, w, _)) => (None, p + 8 * w),
            None => return,
        },
    };
    let mut at = start;
    loop {
        // SAFETY: an allocation's first word, or the next object of the
        // same allocation, which a header begins.
        let o = unsafe { laid::at(std::ptr::with_exposed_provenance::<u64>(at)) };
        // Past where it may end: no object, but words read as another
        // thread writes them (C512), or freed and poisoned.
        if o.end > limit {
            return;
        }
        f(&o);
        if o.pair || o.end >= limit || chunk.is_some_and(|c| c.is_start(o.end)) {
            return;
        }
        // SAFETY: a word of the same allocation, past its last object.
        if unsafe { std::ptr::with_exposed_provenance::<u64>(o.end).read() } & TAG_MASK
            != TAG_HEADER
        {
            return;
        }
        at = o.end;
    }
}

/// A collection's work: the pins, then Cheney's copy. This thread's
/// allocations reached in place (on a pinned page, or one adopted and
/// unsettled), each marked on its page; an ended thread's, which may be
/// followed and written, and a large object of this thread's or an ended
/// one's (`others`); the pages pinned; the copies made, in order, each
/// scanned in its turn (Cheney's queue); and what of others' this thread
/// reaches (`report`).
struct Mark<'v> {
    /// The space, seen again when the copy takes a new chunk.
    view: View,
    /// What is kept that only a word to it keeps (C513): those reached,
    /// those of this thread's reached pinning whole and not yet followed
    /// as its own (as `whole`), and those to scan.
    kept: &'v super::roots::KeptIndex,
    kept_reached: Set<usize>,
    kept_whole: Set<usize>,
    kept_work: Vec<(super::roots::KeptAt, Way)>,
    /// This thread's allocations marked in place, counted.
    marked: usize,
    others: Set<usize>,
    pinned: Set<(usize, usize)>,
    /// What is reached in place, to be scanned: pinning whole, followed
    /// at once (`drain_whole`); else, in the copy.
    work: Vec<(usize, Way, Whose)>,
    /// This thread's allocations marked while pinning whole what other
    /// threads may take (not a constant, which every thread reaches for
    /// ever), not yet followed as its own: what of others' they hold is
    /// not yet told.
    whole: Set<usize>,
    /// Whether what is pinned whole now is noted in `whole`.
    noting: bool,
    report: Set<usize>,
    /// `follow`'s and `scan`'s buffers, kept between their calls.
    found: Vec<usize>,
    found_kept: Vec<super::roots::KeptAt>,
    fields: Vec<(usize, u64)>,
    /// A minor collection's: this thread's old objects neither marked nor
    /// followed; its young large objects.
    minor: bool,
    young_large: &'v Set<usize>,
    /// Whether the copy has begun: no page may be pinned now.
    moving: bool,
    /// Whether a field's word is checked as an ambiguous one is (under the
    /// verifier): a precise word of a live object needs no check.
    checked: bool,
    /// Each copy's first word, in the order made: Cheney's queue.
    copies: Vec<usize>,
    moved_words: usize,
}

/// How an allocation's words are followed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Way {
    /// From what every thread may read or another may take: every
    /// allocation reached pinned, nothing of others' told.
    Whole,
    /// From this thread's roots and objects: each object of another's
    /// reached first from them told (`report`).
    Own,
    /// Below an object of another's reached from this thread's own: told
    /// already by that object, which every other collection follows.
    Below,
}

impl Mark<'_> {
    /// An ambiguous word: the allocation it is a word of, and the one
    /// before when it is an allocation's first, reached and pinned.
    fn ambiguous(&mut self, w: usize, way: Way) {
        let a = self.view.object_of(w);
        if let Some(a) = a {
            self.visit(a, true, way);
        }
        if a.is_none_or(|a| a == w & !7)
            && let Some(b) = self.view.object_of((w & !7).wrapping_sub(8))
        {
            self.visit(b, true, way);
        }
        if a.is_none()
            && let Some(k) = self.kept.find(w)
        {
            self.kept_visit(k, way);
        }
    }

    /// The pages `ambiguous` would pin for the word `w`, pinned, and
    /// nothing reached: what a kept object holds, before the copy, that a
    /// word may reach later.
    fn pin_word(&mut self, w: usize) {
        self.word_pages(w, |m, k| {
            m.pinned.insert(k);
        });
    }

    /// The pages `ambiguous` would pin for the word `w`, each given to `f`.
    fn word_pages(&mut self, w: usize, mut f: impl FnMut(&mut Self, (usize, usize))) {
        let a = self.view.object_of(w);
        if let Some(a) = a
            && let Some(k) = self.pin_page(a)
        {
            f(self, k);
        }
        if a.is_none_or(|a| a == w & !7)
            && let Some(b) = self.view.object_of((w & !7).wrapping_sub(8))
            && let Some(k) = self.pin_page(b)
        {
            f(self, k);
        }
    }

    /// The page an allocation at `a` is on, if a pin of it is this
    /// collection's: a page of this thread's, and young in a minor one.
    fn pin_page(&self, a: usize) -> Option<(usize, usize)> {
        let (c, p) = self.view.page_of(a)?;
        (self.view.whose_page(c, p) == Whose::Mine && (!self.minor || c.is_young(p)))
            .then(|| space::page_key(c, p))
    }

    /// The kept object `k`, reached (`way`): kept, told if another's and
    /// reached from this thread's own, and scanned if this thread's and
    /// not shared (a shared one's words every collection follows whole).
    fn kept_visit(&mut self, k: super::roots::KeptAt, way: Way) {
        if way == Way::Own && !k.mine {
            self.report.insert(k.start);
        }
        if !k.mine || k.shared {
            self.kept_reached.insert(k.start);
            return;
        }
        if self.kept_reached.insert(k.start) {
            if way == Way::Whole && self.noting {
                self.kept_whole.insert(k.start);
            }
            self.kept_work.push((k, way));
        } else if way == Way::Own && !self.kept_whole.is_empty() && self.kept_whole.remove(&k.start)
        {
            self.kept_work.push((k, Way::Own));
        }
    }

    /// The allocation at `a`, reached in place (`way`), its page pinned
    /// if `pin`: not in from-space (once the copy has begun, a field's
    /// target there is copied, `field`).
    #[inline]
    fn visit(&mut self, a: usize, pin: bool, way: Way) {
        let (whose, large) = match self.view.page_of(a) {
            Some((c, p)) => {
                let whose = self.view.whose_page(c, p);
                if whose == Whose::Mine {
                    if self.minor && !c.is_young(p) {
                        return;
                    }
                    if pin {
                        if self.moving && c.is_from(a) {
                            late_pin(a);
                        }
                        self.pinned.insert(space::page_key(c, p));
                    }
                    if c.mark(a) {
                        self.marked += 1;
                        self.push(a, way, Whose::Mine);
                    } else if way == Way::Own && !self.whole.is_empty() && self.whole.remove(&a) {
                        // Pinned whole already; followed again as this
                        // thread's own, for what of others' it holds.
                        self.work.push((a, Way::Own, Whose::Mine));
                    }
                    return;
                }
                (whose, false)
            }
            None => match self.view.large_owner(a) {
                Some((_, _, whose)) => {
                    if self.minor && whose == Whose::Mine && !self.young_large.contains(&a) {
                        return;
                    }
                    (whose, true)
                }
                None => return,
            },
        };
        // A live thread's object, or one of no thread's (a fragment's), is
        // left, neither followed nor told: no thread's heap is reached from
        // another's (TODO §54 step 4b's (c), C521), but a `par` branch's
        // from its waiting maker's, which does not move as it waits, and a
        // fragment's, which holds no thread's. An ended thread's, or a large
        // object, is written where it points to what moves.
        if whose == Whose::Live {
            return;
        }
        let first = self.others.insert(a);
        if first {
            self.push(a, way, whose);
        } else if large
            && whose == Whose::Mine
            && way == Way::Own
            && !self.whole.is_empty()
            && self.whole.remove(&a)
        {
            self.work.push((a, Way::Own, whose));
        }
    }

    fn push(&mut self, a: usize, way: Way, whose: Whose) {
        if way == Way::Whole && self.noting {
            self.whole.insert(a);
        }
        self.work.push((a, way, whose));
    }

    /// Pinning whole: each allocation reached followed at once, its
    /// fields' targets pinned, and what kept objects hold.
    fn drain_whole(&mut self) {
        loop {
            if let Some((a, way, whose)) = self.work.pop() {
                self.follow(a, way, whose);
            } else if !self.drain_kept_one() {
                return;
            }
        }
    }

    /// Before the copy: each kept object reached scanned, its words
    /// ambiguous, what they reach pinned and queued, not yet followed.
    fn drain_kept(&mut self) {
        while self.drain_kept_one() {}
    }

    fn drain_kept_one(&mut self) -> bool {
        let Some((k, way)) = self.kept_work.pop() else {
            return false;
        };
        let mut words = Vec::new();
        k.words(&mut |w| words.push(w));
        for w in words {
            self.ambiguous(w, way);
        }
        true
    }

    /// The allocation at `a`'s fields followed, pinning whole: what they
    /// point to in the space, and what is kept that a plain word of one is
    /// (C508).
    fn follow(&mut self, a: usize, way: Way, whose: Whose) {
        let mut found = std::mem::take(&mut self.found);
        let mut found_kept = std::mem::take(&mut self.found_kept);
        let view = &self.view;
        let kept = self.kept;
        each_object(view, a, |o| {
            for &f in o.fields {
                if let Some(t) = target(view, f) {
                    found.push(t);
                } else if f.raw() & 7 == 0
                    && let Some(k) = kept.find(f.raw())
                {
                    found_kept.push(k);
                }
            }
        });
        let pin = way == Way::Whole || whose == Whose::Live;
        let next = match (way, whose) {
            (Way::Whole, _) => Way::Whole,
            (_, Whose::Mine) => way,
            _ => Way::Below,
        };
        for &t in &found {
            self.visit(t, pin, next);
        }
        for &k in &found_kept {
            self.kept_visit(k, next);
        }
        found.clear();
        found_kept.clear();
        self.found = found;
        self.found_kept = found_kept;
    }

    /// Cheney's loop: what is reached in place scanned, and each kept
    /// object reached, and each copy in the order made, till none is left.
    fn cheney(&mut self) {
        let mut next = 0;
        loop {
            if let Some((a, way, whose)) = self.work.pop() {
                debug_assert!(way != Way::Whole, "pinning whole as objects move");
                self.scan(a, way, whose);
            } else if self.drain_kept_one() {
            } else if let Some(&to) = self.copies.get(next) {
                next += 1;
                self.scan(to, Way::Own, Whose::Mine);
            } else {
                return;
            }
        }
    }

    /// The allocation at `a`'s fields, each taken (`field`) and written
    /// where its target moved: an allocation of this thread's (a copy, or
    /// one in place) or of an ended thread's, which runs no Rust.
    fn scan(&mut self, a: usize, way: Way, whose: Whose) {
        let next = if whose == Whose::Mine {
            way
        } else {
            Way::Below
        };
        let mut fields = std::mem::take(&mut self.fields);
        each_object(&self.view, a, |o| {
            for f in o.fields {
                fields.push((std::ptr::from_ref(f).addr(), f.raw() as u64));
            }
        });
        for &(slot, w) in &fields {
            let new = self.field(w, next);
            if new != w {
                // SAFETY: a field of a reached allocation this thread may
                // write.
                unsafe { space::word_mut(&self.view, slot).write(new) };
            }
        }
        fields.clear();
        self.fields = fields;
    }

    /// A precise word (a field's, a noted slot's), as the copy takes it:
    /// its target, if in from-space, copied (once) and the word made to
    /// point at the copy, its offset into the allocation kept; else
    /// reached in place; a plain word of a kept object, that reached.
    #[inline]
    fn field(&mut self, w: u64, way: Way) -> u64 {
        let s = match w & TAG_MASK {
            TAG_PAIR => (w & !TAG_MASK) as usize,
            TAG_OBJECT => match self
                .view
                .object_of(((w & !TAG_MASK) as usize).wrapping_sub(8))
            {
                Some(s) => s,
                None => return w,
            },
            _ => {
                if w & 7 == 0
                    && let Some(k) = self.kept.find(w as usize)
                {
                    self.kept_visit(k, way);
                }
                return w;
            }
        };
        let from = self.view.page_of(s).is_some_and(|(c, _)| c.is_from(s));
        if from {
            // SAFETY: a word of an allocation in from-space, its words
            // kept till the collection's end.
            let first = unsafe { std::ptr::with_exposed_provenance::<u64>(s).read() };
            if first & TAG_MASK == TAG_FORWARD {
                let to = (first & !TAG_MASK) as usize;
                let moved = w.wrapping_add(to as u64).wrapping_sub(s as u64);
                // Checked against the copy: the original's first word is
                // its forwarding word now.
                if self.checked && target_word(&self.view, moved) != Some(to) {
                    bad_field(w, s);
                }
                debug_assert!(self.view.whose_of(s) == Whose::Mine, "another's from-space");
                return moved;
            }
        }
        // Under the verifier, a word checked as an ambiguous one is (no
        // word of a live object fails it).
        if self.checked && target_word(&self.view, w) != Some(s) {
            if self.view.object_of(s).is_none() {
                return w;
            }
            bad_field(w, s);
        }
        if from {
            // Another thread copies from a page of its own that nothing of
            // this one's reaches (no thread's heap is reached from
            // another's: C521), so the page is this thread's.
            if self.checked && self.view.whose_of(s) != Whose::Mine {
                panic!("kayak: a field's word {w:#x} reaches another thread's from-space");
            }
            let to = self.copy(s);
            return w.wrapping_add(to as u64).wrapping_sub(s as u64);
        }
        self.visit(s, false, way);
        w
    }

    /// The allocation at `s` (its first word, in from-space) copied whole
    /// to a new page of the thread's, a forwarding word left in its first;
    /// the copy queued, to be scanned.
    fn copy(&mut self, s: usize) -> usize {
        let mut end = s;
        each_object(&self.view, s, |o| end = o.end);
        let n = self.view.extent_of(s, end);
        let to = space::words(n);
        // SAFETY: `n` words of the allocation, read whole into `n` new
        // words of to-space, which no other object overlaps; then its
        // first word, its forwarding word.
        unsafe {
            let at = space::word_mut(&self.view, s);
            // Most are a pair's two words, or a few: copied in line.
            if n <= 4 {
                for i in 0..n {
                    to.add(i).write(at.add(i).read());
                }
            } else {
                std::ptr::copy_nonoverlapping(at, to, n);
            }
            at.write(to.addr() as u64 | TAG_FORWARD);
        }
        let to = to.addr();
        // A new chunk taken for to-space: the space seen again.
        if self.view.page_of(to).is_none() {
            self.view = View::new();
        }
        self.copies.push(to);
        self.moved_words += n;
        to
    }
}

/// A field's word that points into the space at no object (the
/// verifier's): a word of a dead object not cleared, or a store of no
/// Fixpt word.
#[cold]
fn bad_field(w: u64, s: usize) -> ! {
    panic!("kayak: a field's word {w:#x} points into the allocation at {s:#x}, at no object")
}

/// An ambiguous word found as objects move, to an allocation in
/// from-space: what a kept object holds that the pins did not see.
#[cold]
fn late_pin(a: usize) -> ! {
    panic!(
        "kayak: an ambiguous word found as the copy runs reaches the allocation at {a:#x}, in from-space"
    )
}

/// Whether the object at `a` (an allocation's first word) is young: on a
/// young page, or a large object made since the last collection.
fn young(view: &View, young_large: &Set<usize>, a: usize) -> bool {
    match view.page_of(a) {
        Some((c, p)) => c.is_young(p),
        None => young_large.contains(&a),
    }
}

/// A store the barrier missed, found by the verifier: an old object's
/// slot pointing to a young object, not noted.
#[cold]
fn missed(obj: usize, slot: usize, target: usize) -> ! {
    // SAFETY: the first words of two allocations of the space, read.
    let (h, t) = unsafe {
        (
            std::ptr::with_exposed_provenance::<u64>(obj).read(),
            std::ptr::with_exposed_provenance::<u64>(target).read(),
        )
    };
    panic!(
        "kayak: the write barrier missed a store: the old object at {obj:#x} (first word {h:#x}), its slot at {slot:#x}, points to the young object at {target:#x} (first word {t:#x})"
    )
}

/// The allocation a field's word points to, if it is a pair's or an
/// object's word of one: a pair's, at an allocation's first word that is
/// no header; an object's, at the suffix of an object of the allocation it
/// is in, its header found by its trailer and checked. Checked so, since a
/// dead object on a pinned page, reached by a stale word, may hold words
/// into pages taken again since.
#[inline]
fn target(view: &View, f: HeapWord) -> Option<usize> {
    target_word(view, f.raw() as u64)
}

/// `target`, of a word's bits.
#[inline]
pub(super) fn target_word(view: &View, w: u64) -> Option<usize> {
    let read = |a: usize| -> u64 {
        // SAFETY: a word of an allocation of the space, read.
        unsafe { std::ptr::with_exposed_provenance::<u64>(a).read() }
    };
    match w & TAG_MASK {
        TAG_PAIR => {
            let t = (w & !TAG_MASK) as usize;
            (view.object_of(t) == Some(t) && read(t) & TAG_MASK != TAG_HEADER).then_some(t)
        }
        TAG_OBJECT => {
            let at = (w & !TAG_MASK) as usize;
            let s = view.object_of(at.wrapping_sub(8))?;
            let before = read(at - 8);
            let main = if before & TAG_MASK == TAG_HEADER {
                at - 8
            } else {
                let back = ((before >> 3) & ((1 << 59) - 1)) as usize;
                (at - 8).checked_sub(8 * back)?
            };
            if main < s || read(main) & TAG_MASK != TAG_HEADER {
                return None;
            }
            // An extension word before a main header (Fixpt's large
            // field count) is where its object starts.
            let first = if main > s
                && (read(main - 8) >> 3) & 0xff == 255
                && read(main - 8) & TAG_MASK == TAG_HEADER
            {
                main - 8
            } else {
                main
            };
            // SAFETY: an object's first word, its header checked.
            let o = unsafe { laid::at(std::ptr::with_exposed_provenance::<u64>(first)) };
            (!o.pair && o.suffix == at).then_some(s)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::{Int, List, heap_cons, heap_new, i64_to_int, int_to_i64, nil};

    fn list(n: i64) -> List<Int> {
        (0..n)
            .rev()
            .fold(nil(), |acc, i| heap_cons(i64_to_int(i * 1000), acc))
    }

    fn sum(xs: List<Int>) -> i64 {
        let mut s = 0;
        let mut at = xs;
        while let Some(c) = at.pair() {
            s += int_to_i64(c.car());
            at = c.cdr;
        }
        s
    }

    #[test]
    fn what_the_stack_holds_kept_the_rest_freed() {
        // Garbage first, then what is kept: a list held by the stack, its
        // cells' cdrs precise words of the heap.
        let _ = std::hint::black_box(list(5000));
        let xs = std::hint::black_box(list(3000));
        let before = super::collections();
        super::collect();
        assert!(super::collections() > before);
        let _ = std::hint::black_box(list(5000));
        super::collect();
        let l = super::last();
        assert!(
            crate::obj::space::stressed() || (l.moved > 0 && l.freed_pages > 0),
            "{l:?}"
        );
        assert_eq!(sum(std::hint::black_box(xs)), 1000 * 2999 * 3000 / 2);
    }

    #[test]
    fn a_list_only_a_box_holds_moved_and_followed() {
        // The box is held by the stack (pinned); its list, by its field
        // alone, a precise word: copied, and the field made to follow.
        let ys = list(4000);
        let r = heap_new(ys);
        let r = std::hint::black_box(r);
        super::collect();
        let l = super::last();
        // Under stress, moved already.
        assert!(
            crate::obj::space::stressed() || (l.moved >= 1000 && l.freed_pages >= 1),
            "{l:?}"
        );
        assert_eq!(sum(r.get()), 1000 * 3999 * 4000 / 2);
    }

    /// Garbage made between collections: less under stress, where each
    /// allocation may collect already.
    fn garbage() -> i64 {
        if crate::obj::space::stressed() {
            100
        } else {
            3000
        }
    }

    fn of_word(w: crate::obj::HeapWord) -> List<Int> {
        crate::obj::ListAt(w, std::marker::PhantomData)
    }

    /// `n` cells of this thread's before `tail`: 0, 1, …, n - 1.
    fn before(n: i64, tail: List<Int>) -> List<Int> {
        (0..n)
            .rev()
            .fold(tail, |acc, i| heap_cons(i64_to_int(i), acc))
    }

    #[test]
    fn a_par_branch_collects_beside_its_maker() {
        // The branch puts cells of its own before its maker's list, which
        // its closure holds, collecting as it goes; the maker collects all
        // the while. After the join, the branch's pages are the maker's.
        let xs = list(2000);
        let w = xs.0;
        let (mine, theirs) = crate::process::par2(
            || {
                for _ in 0..30 {
                    let _ = std::hint::black_box(list(garbage()));
                    super::collect();
                }
                sum(xs)
            },
            move || {
                let mut ys = of_word(w);
                for _ in 0..30 {
                    ys = before(100, ys);
                    let _ = std::hint::black_box(list(garbage()));
                    super::collect();
                }
                ys.0
            },
        );
        assert_eq!(mine, 1000 * 1999 * 2000 / 2);
        for _ in 0..3 {
            let _ = std::hint::black_box(list(garbage()));
            super::collect();
        }
        assert_eq!(
            sum(of_word(theirs)),
            30 * 99 * 100 / 2 + 1000 * 1999 * 2000 / 2
        );
    }

    #[test]
    fn a_minor_collection_reads_the_young_alone() {
        // An old list (a major collection made it so) is neither read nor
        // moved by a minor one: a major one right after reaches its 300
        // cells more than the minor did, and both keep both lists.
        let old = list(300);
        super::collect();
        let old_at = old.0.raw();
        let young = list(300);
        super::collect_minor();
        let minor = super::last().reached;
        assert_eq!(
            old.0.raw(),
            old_at,
            "an old object moved by a minor collection"
        );
        super::collect();
        let major = super::last().reached;
        assert!(
            minor + 300 <= major,
            "a minor collection reached {minor}, a major {major}"
        );
        assert_eq!(sum(old), 1000 * 299 * 300 / 2);
        assert_eq!(sum(young), 1000 * 299 * 300 / 2);
    }

    #[test]
    fn a_young_list_an_old_box_holds_kept_by_a_minor_collection() {
        // A box made old, then given a young list (the barrier notes its
        // slot): a minor collection keeps the list through the slot alone
        // and makes the slot point where the list went. Without the
        // barrier (`Slot::set`'s `remember`), the list's pages are freed.
        let r = heap_new(nil::<Int>());
        super::collect();
        crate::set(r, list(500));
        for _ in 0..3 {
            let _ = std::hint::black_box(list(garbage()));
            super::collect_minor();
        }
        assert_eq!(sum(crate::get(r)), 1000 * 499 * 500 / 2);
    }

    /// A box holding a kept cell (kayak-rt's own memory, whose stores no
    /// barrier sees), made old, the cell then reached by the box alone.
    #[inline(never)]
    fn an_old_box_of_a_kept_cell() -> crate::Ref<&'static std::cell::Cell<List<Int>>> {
        let r = heap_new(crate::leak_collected(std::cell::Cell::new(nil::<Int>())));
        super::collect();
        r
    }

    #[inline(never)]
    fn given_a_young_list(r: crate::Ref<&'static std::cell::Cell<List<Int>>>) {
        crate::get(r).set(list(500));
    }

    #[test]
    fn a_young_list_a_kept_cell_an_old_box_holds_kept_by_a_minor_collection() {
        // A minor collection follows no old object, so a kept object only
        // an old one reaches it takes as a root (C523): else the young
        // list the cell holds, which no barrier noted, freed.
        let r = an_old_box_of_a_kept_cell();
        given_a_young_list(r);
        for _ in 0..3 {
            let _ = std::hint::black_box(list(garbage()));
            super::collect_minor();
        }
        assert_eq!(sum(crate::get(r).get()), 1000 * 499 * 500 / 2);
    }

    fn text_of(n: usize) -> String {
        (0..n).map(|i| if i % 3 == 0 { 'é' } else { 'x' }).collect()
    }

    /// The strings of each length to 40 in a list, which a box alone holds,
    /// made after them, so that no word of the stack reaches the strings'
    /// pages.
    #[inline(never)]
    fn strings_boxed() -> crate::Ref<List<crate::Str>> {
        let xs: List<crate::Str> = (0..40).rev().fold(nil(), |acc, n| {
            heap_cons(crate::leak_string(text_of(n)), acc)
        });
        for _ in 0..4 {
            let _ = std::hint::black_box(list(garbage()));
        }
        heap_new(xs)
    }

    /// Strings of each length, moved by collections, as they were made
    /// (C533: a string's second word, its count, any bits at all, not read
    /// as a trailer).
    #[test]
    fn strings_of_each_length_moved() {
        let r = strings_boxed();
        for _ in 0..3 {
            let _ = std::hint::black_box(list(garbage()));
            super::collect();
        }
        let mut at = crate::get(r);
        for n in 0..40 {
            let c = at.pair().expect("40 strings");
            assert_eq!(c.car().text(), text_of(n));
            at = c.cdr;
        }
    }

    #[test]
    fn large_objects_not_reached_freed() {
        // A large string held by the stack, kept as large garbage is made
        // and freed around it.
        let big = "ab".repeat(3 * crate::obj::space::PAGE_BYTES);
        let held = std::hint::black_box(crate::str_of_parts(&[&big]));
        for i in 0..40 {
            let _ = std::hint::black_box(crate::str_of_parts(&[&big, &i.to_string()]));
            if i % 8 == 0 {
                super::collect();
            }
        }
        super::collect();
        // What is left: the string held, and perhaps one more a stale word
        // pins.
        assert!(
            // Its object's bytes: four a code point (C533).
            crate::obj::space::my_large_bytes() <= 2 * (8 * crate::string_words(big.len()) + 64),
            "{} bytes of large objects",
            crate::obj::space::my_large_bytes()
        );
        assert_eq!(std::hint::black_box(held).text(), big);
    }

    #[test]
    fn an_ended_threads_pages_adopted() {
        // A thread makes garbage and a list, which it leaves in an I-cell,
        // and ends: its pages are given to the next thread that collects,
        // which frees those holding nothing and keeps the list.
        let c = crate::process::make_icell::<List<Int>>();
        let (pages, number) = std::thread::spawn(move || {
            for _ in 0..20 {
                let _ = std::hint::black_box(list(garbage()));
            }
            crate::process::icell_put_x(c, list(500));
            (
                crate::obj::space::my_page_words(),
                crate::obj::space::my_number(),
            )
        })
        .join()
        .unwrap();
        assert!(!pages.is_empty());
        for _ in 0..3 {
            let _ = std::hint::black_box(list(garbage()));
            super::collect();
        }
        for p in pages {
            assert_ne!(crate::obj::space::page_owner(p), number);
        }
        assert_eq!(sum(crate::process::icell_get(c)), 1000 * 499 * 500 / 2);
    }

    #[inline(never)]
    fn marks_made(n: i64) -> i64 {
        let key = crate::marks::make_continuation_mark_key::<i64>();
        (0..n)
            .map(|i| crate::marks::with_mark(key, i, || crate::marks::first_mark(key, -1)))
            .sum()
    }

    /// `n` I-cells, each in a box, the boxes in a list: each I-cell held
    /// by a plain word of the heap alone (C508).
    #[inline(never)]
    fn icells_in_boxes(n: i64) -> List<crate::Ref<crate::ICell<i64>>> {
        (0..n).fold(nil(), |acc, i| {
            let c = crate::process::make_icell::<i64>();
            crate::process::icell_put_x(c, i);
            heap_cons(heap_new(c), acc)
        })
    }

    fn icells_sum(mut xs: List<crate::Ref<crate::ICell<i64>>>) -> i64 {
        let mut s = 0;
        while let Some(c) = xs.pair() {
            s += crate::process::icell_get(c.car().get());
            xs = c.cdr;
        }
        s
    }

    #[test]
    fn kept_objects_no_word_reaches_freed() {
        // Marks' values, each kept while its mark is pushed, then reached
        // by no word: freed. I-cells the heap alone holds: kept, as their
        // boxes move.
        let bs = std::hint::black_box(icells_in_boxes(50));
        let before = crate::obj::roots::my_kept();
        assert_eq!(marks_made(300), 299 * 300 / 2);
        for _ in 0..3 {
            let _ = std::hint::black_box(list(garbage()));
            super::collect();
        }
        let after = crate::obj::roots::my_kept();
        assert!(after < before + 10, "{after} kept, {before} before");
        assert!(after + 5 >= before, "{after} kept, {before} before");
        assert_eq!(icells_sum(std::hint::black_box(bs)), 49 * 50 / 2);
    }

    #[inline(never)]
    fn sent(p: crate::Addr<(List<Int>, crate::ICell<List<Int>>)>) -> crate::ICell<List<Int>> {
        let c = crate::process::make_icell();
        crate::process::send(p, (list(300), c));
        c
    }

    #[test]
    fn a_message_wrapped_and_sent_back_kept() {
        // A process puts cells of its own before each list it is sent, and
        // sends it back: the list then this thread's only below another's
        // cells, each side collecting as it goes.
        const ROUNDS: usize = 40;
        let p = crate::process::spawn(|b: crate::Inbox<(List<Int>, crate::ICell<List<Int>>)>| {
            for _ in 0..ROUNDS {
                let (w, c) = crate::process::receive(b);
                let ys = before(100, w);
                let _ = std::hint::black_box(list(garbage()));
                super::collect();
                crate::process::icell_put_x(c, ys);
            }
        });
        for _ in 0..ROUNDS {
            let c = sent(p);
            let _ = std::hint::black_box(list(garbage()));
            super::collect();
            let w = crate::process::icell_get(c);
            let _ = std::hint::black_box(list(garbage()));
            super::collect();
            assert_eq!(sum(w), 99 * 100 / 2 + 1000 * 299 * 300 / 2);
        }
    }
}
