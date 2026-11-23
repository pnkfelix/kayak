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
//!   word, found by its header. Each is followed to its allocation; every
//!   allocation reached on a page of this thread's that no root pins is
//!   copied to a new page, a forwarding word left in its first (Fixpt's
//!   tag 7, as `kcrt-core`'s collector and Fixpt's leave one), and every
//!   reached object's fields are then made to point where their targets
//!   went, each word's offset into its allocation kept.
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
//! The mark is first and whole, the copy after it: a kept object reached
//! late may pin what is reached early, and nothing is moved before every
//! pin is known.

use super::space::{self, View, Whose};
use super::{HeapWord, laid};
use std::collections::HashSet;
use std::hash::{BuildHasherDefault, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};

/// Fixpt's forwarding word's tag.
const TAG_FORWARD: u64 = 0b111;
const TAG_MASK: u64 = 0b111;
const TAG_PAIR: u64 = 0b001;
const TAG_OBJECT: u64 = 0b100;
const TAG_HEADER: u64 = 0b110;

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
    let mut saved = [0usize; 12];
    super::roots::spill(&mut saved);
    let saved = std::hint::black_box(saved);
    collect_from(&saved, std::ptr::from_ref(&saved).addr());
    std::hint::black_box(&saved);
}

#[inline(never)]
fn collect_from(saved: &[usize; 12], from: usize) {
    let mut timer = Timer::start();
    space::begin_collection();
    // What threads ended with no heir left (a process's pages), this
    // thread's to collect from now.
    space::adopt_orphans();
    let roots = super::roots::roots_from(saved, from);
    timer.lap();
    let view = View::new();
    let mine = space::my_pages();
    let mut m = Mark {
        view: &view,
        kept: &roots.kept,
        kept_reached: Set::default(),
        kept_whole: Set::default(),
        kept_work: Vec::new(),
        order: Vec::new(),
        others: Set::default(),
        read: Set::default(),
        pinned: Set::default(),
        work: Vec::new(),
        whole: Set::default(),
        noting: false,
        report: Set::default(),
        found: Vec::new(),
        found_kept: Vec::new(),
    };
    // First what every thread may read, and what another may take as this
    // one runs (C512), each allocation it reaches pinned: the constants,
    // what shared kept objects hold, the other threads' lists.
    for &w in &roots.constants {
        m.ambiguous(w, Way::Whole);
    }
    m.drain();
    let constants = m.pinned.len();
    m.noting = true;
    for &w in &roots.shared {
        m.ambiguous(w, Way::Whole);
    }
    m.drain();
    m.noting = false;
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
    m.drain();
    if verbose() {
        eprintln!(
            "kayak: pinned pages: constants {}, shared {}, stack {}, owned {}, kept {}, locals {}; words: stack {}, owned {}, kept {}, shared {}; others' objects reached {}",
            constants,
            pins[0] - constants,
            pins[1] - pins[0],
            pins[2] - pins[1],
            pins[3] - pins[2],
            pins[4] - pins[3],
            roots.ends[0],
            roots.ends[1] - roots.ends[0],
            roots.ends[2] - roots.ends[1],
            roots.shared.len(),
            m.report.len(),
        );
    }
    let Mark {
        order,
        others,
        pinned,
        report,
        kept_reached,
        ..
    } = m;
    // What this thread reaches of others', for their collections; not
    // moved by this one, so told now.
    super::roots::reported(report.into_iter().collect(), roots.reach_from);

    // From-space: this thread's pages that nothing pins, but those adopted
    // that a collection on another thread may still write.
    timer.lap();
    let unsettled = space::unsettled();
    let from_pages: Vec<_> = mine
        .iter()
        .copied()
        .filter(|&(c, p)| {
            let k = space::page_key(c, p);
            !pinned.contains(&k) && !unsettled.contains(&k)
        })
        .collect();
    for &(c, p) in &from_pages {
        c.set_from(p, true);
    }
    // Each reached allocation of from-space copied, a forwarding word
    // left in its first.
    let mut moved = 0;
    for &a in &order {
        if !in_from(&view, a) {
            continue;
        }
        let mut end = a;
        each_object(&view, a, |o| end = o.end);
        let n = view.extent_of(a, end);
        let to = space::words(n);
        // SAFETY: `n` words of the allocation, read whole into `n` new
        // words of to-space, which no other object overlaps; then its
        // first word, its forwarding word.
        unsafe {
            let at = space::word_mut(&view, a);
            std::ptr::copy_nonoverlapping(at, to, n);
            at.write(to.addr() as u64 | TAG_FORWARD);
        }
        moved += 1;
    }
    timer.lap();

    // Each reached object this thread may write, where it is now, its
    // fields made to point where their targets went. The space seen again,
    // with to-space's chunks.
    let view = View::new();
    let forwarded = |s: usize| -> Option<usize> {
        if !in_from(&view, s) {
            return None;
        }
        // SAFETY: an allocation's first word in from-space, its words kept
        // till the collection's end.
        let w = unsafe { std::ptr::with_exposed_provenance::<u64>(s).read() };
        (w & TAG_MASK == TAG_FORWARD).then_some((w & !TAG_MASK) as usize)
    };
    for &a in order.iter().chain(others.iter()) {
        let at = forwarded(a).unwrap_or(a);
        each_object(&view, at, |o| {
            for f in o.fields {
                let w = f.raw() as u64;
                let s = match w & TAG_MASK {
                    TAG_PAIR => (w & !TAG_MASK) as usize,
                    TAG_OBJECT => {
                        match view.object_of(((w & !TAG_MASK) as usize).wrapping_sub(8)) {
                            Some(s) => s,
                            None => continue,
                        }
                    }
                    _ => continue,
                };
                let Some(to) = forwarded(s) else { continue };
                let new = w.wrapping_add(to as u64).wrapping_sub(s as u64);
                // SAFETY: a field of a reached object this thread may write.
                unsafe {
                    space::word_mut(&view, std::ptr::from_ref(f).addr()).write(new);
                }
            }
        });
    }
    timer.lap();
    // This thread's large objects not reached, freed (once no collection
    // running may read them).
    space::free_large(|a| others.contains(&a));
    for &(c, p) in &mine {
        c.clear_marks(p);
    }
    for &(c, p) in &from_pages {
        c.set_from(p, false);
    }
    crate::forget_cursor();
    LAST.set(Last {
        reached: order.len() + others.len(),
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
    space::end_collection(&from_pages);
    // What this thread keeps that no word reached (what was kept since
    // the roots were read, left), freed once no collection running may
    // scan it: after the collection, as a drop may allocate.
    super::roots::retire_kept(|k| {
        kept_reached.contains(&k) || roots.kept.find(k).is_none_or(|i| i.start != k || !i.mine)
    });
    COLLECTIONS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    timer.lap();
    let last = LAST.get();
    timer.done(&last, roots.ends[0]);
}

/// Whether `a` is on a page a collection copies from. Asked only of this
/// thread's reached allocations and of their fields' targets: another
/// thread copies from a page of its own nothing of this one's reaches
/// (what this one reaches of its, it pins), so the page is this thread's.
#[inline(always)]
fn in_from(view: &View, a: usize) -> bool {
    view.page_of(a).is_some_and(|(c, _)| c.is_from(a))
}

/// `KAYAK_GC_STATS=1`: each collection's phases timed, summed over the
/// run's threads, and said when the run ends (`report_stats`).
fn stats_on() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var("KAYAK_GC_STATS").is_ok_and(|v| v != "0"))
}

/// The phases a collection is timed by: its roots read (and orphans
/// adopted), its mark, its copy, its fields fixed, its end (large and kept
/// objects freed, pages given back).
const PHASES: [&str; 5] = ["roots", "mark", "copy", "fixup", "end"];

/// The run's sums, in nanoseconds and counts, over every thread's
/// collections.
struct Stats {
    phase_ns: [AtomicU64; 5],
    max_ns: AtomicU64,
    reached: AtomicU64,
    moved: AtomicU64,
    pinned_pages: AtomicU64,
    freed_pages: AtomicU64,
    stack_words: AtomicU64,
    // Pauses by length: below 1 ms, 10 ms, 100 ms, and above.
    buckets: [AtomicU64; 4],
}

static STATS: Stats = Stats {
    phase_ns: [const { AtomicU64::new(0) }; 5],
    max_ns: AtomicU64::new(0),
    reached: AtomicU64::new(0),
    moved: AtomicU64::new(0),
    pinned_pages: AtomicU64::new(0),
    freed_pages: AtomicU64::new(0),
    stack_words: AtomicU64::new(0),
    buckets: [const { AtomicU64::new(0) }; 4],
};

/// A collection's times, phase by phase.
struct Timer(Option<(std::time::Instant, [u64; 5], usize)>);

impl Timer {
    fn start() -> Self {
        Timer(stats_on().then(|| (std::time::Instant::now(), [0; 5], 0)))
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
    fn done(self, l: &Last, stack_words: usize) {
        let Some((_, ns, _)) = self.0 else { return };
        let r = Ordering::Relaxed;
        for (s, n) in STATS.phase_ns.iter().zip(ns) {
            s.fetch_add(n, r);
        }
        let total: u64 = ns.iter().sum();
        STATS.max_ns.fetch_max(total, r);
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
        "kayak-gc: per collection: {:.0} reached, {:.0} moved, {:.1} pages pinned, {:.1} freed, {:.0} stack words",
        per(STATS.reached.load(r)),
        per(STATS.moved.load(r)),
        per(STATS.pinned_pages.load(r)),
        per(STATS.freed_pages.load(r)),
        per(STATS.stack_words.load(r)),
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
fn each_object(view: &View, start: usize, mut f: impl FnMut(&laid::Laid)) {
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

/// The mark: this thread's allocations reached, in the order found (each
/// marked on its page); an ended thread's that may be followed and
/// written, and a large object of this thread's or an ended one's
/// (`others`); a live thread's, read but never written (`read`); the
/// pages pinned; and what of others' this thread reaches (`report`).
struct Mark<'v> {
    view: &'v View,
    /// What is kept that only a word to it keeps (C513): those reached,
    /// those of this thread's reached pinning whole and not yet followed
    /// as its own (as `whole`), and those to scan.
    kept: &'v super::roots::KeptIndex,
    kept_reached: Set<usize>,
    kept_whole: Set<usize>,
    kept_work: Vec<(super::roots::KeptAt, Way)>,
    order: Vec<usize>,
    others: Set<usize>,
    read: Set<usize>,
    pinned: Set<(usize, usize)>,
    work: Vec<(usize, Way, Whose)>,
    /// This thread's allocations marked while pinning whole what other
    /// threads may take (not a constant, which every thread reaches for
    /// ever), not yet followed as its own: what of others' they hold is
    /// not yet told.
    whole: Set<usize>,
    /// Whether what is pinned whole now is noted in `whole`.
    noting: bool,
    report: Set<usize>,
    /// `follow`'s buffers, kept between its calls.
    found: Vec<usize>,
    found_kept: Vec<super::roots::KeptAt>,
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

    /// The allocation at `a`, reached (`way`), its page pinned if `pin`.
    #[inline]
    fn visit(&mut self, a: usize, pin: bool, way: Way) {
        let (whose, large) = match self.view.page_of(a) {
            Some((c, p)) => {
                let whose = self.view.whose_page(c, p);
                if whose == Whose::Mine {
                    if pin {
                        self.pinned.insert(space::page_key(c, p));
                    }
                    if c.mark(a) {
                        self.order.push(a);
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
                Some((_, _, whose)) => (whose, true),
                None => return,
            },
        };
        if way == Way::Own && whose != Whose::Mine {
            self.report.insert(a);
        }
        // A live thread's object is read, never written: what it holds of
        // this thread's is pinned (C512). An ended thread's, or a large
        // object, is written where it points to what moves; one another
        // thread may read was reached first pinning whole, so nothing it
        // points to moves.
        let first = if whose == Whose::Live {
            self.read.insert(a)
        } else {
            self.others.insert(a)
        };
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

    /// Each allocation reached, its words followed, precisely (an object's
    /// fields): pinning whole, each allocation reached pinned, and every
    /// one a live thread's object holds; from this thread's own, what of
    /// others' it holds told.
    fn drain(&mut self) {
        loop {
            if let Some((a, way, whose)) = self.work.pop() {
                self.follow(a, way, whose);
            } else if let Some((k, way)) = self.kept_work.pop() {
                let mut words = Vec::new();
                k.words(&mut |w| words.push(w));
                for w in words {
                    self.ambiguous(w, way);
                }
            } else {
                return;
            }
        }
    }

    /// The allocation at `a`'s fields followed: what they point to in the
    /// space, and what is kept that a plain word of one is (C508).
    fn follow(&mut self, a: usize, way: Way, whose: Whose) {
        {
            let view = self.view;
            let kept = self.kept;
            let mut found = std::mem::take(&mut self.found);
            let mut found_kept = std::mem::take(&mut self.found_kept);
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
    }
}

/// The allocation a field's word points to, if it is a pair's or an
/// object's word of one: a pair's, at an allocation's first word that is
/// no header; an object's, at the suffix of an object of the allocation it
/// is in, its header found by its trailer and checked. Checked so, since a
/// dead object on a pinned page, reached by a stale word, may hold words
/// into pages taken again since.
#[inline]
fn target(view: &View, f: HeapWord) -> Option<usize> {
    let w = f.raw() as u64;
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
            crate::obj::space::my_large_bytes() <= 2 * (big.len() + 64),
            "{} bytes of large objects",
            crate::obj::space::my_large_bytes()
        );
        assert_eq!(std::hint::black_box(held).as_str(), big);
    }

    #[test]
    fn an_ended_threads_pages_adopted() {
        // A thread makes garbage and a list, which it leaves in an I-cell,
        // and ends: its pages are given to the next thread that collects,
        // which frees those holding nothing and keeps the list.
        let c = crate::process::make_icell::<crate::obj::HeapWord>();
        let (pages, number) = std::thread::spawn(move || {
            for _ in 0..20 {
                let _ = std::hint::black_box(list(garbage()));
            }
            crate::process::icell_put_x(c, list(500).0);
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
        assert_eq!(
            sum(of_word(crate::process::icell_get(c))),
            1000 * 499 * 500 / 2
        );
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
    fn sent(
        p: crate::Addr<(crate::obj::HeapWord, crate::ICell<crate::obj::HeapWord>)>,
    ) -> crate::ICell<crate::obj::HeapWord> {
        let c = crate::process::make_icell();
        crate::process::send(p, (list(300).0, c));
        c
    }

    #[test]
    fn a_message_wrapped_and_sent_back_kept() {
        // A process puts cells of its own before each list it is sent, and
        // sends it back: the list then this thread's only below another's
        // cells, each side collecting as it goes.
        const ROUNDS: usize = 40;
        let p = crate::process::spawn(
            |b: crate::Inbox<(crate::obj::HeapWord, crate::ICell<crate::obj::HeapWord>)>| {
                for _ in 0..ROUNDS {
                    let (w, c) = crate::process::receive(b);
                    let ys = before(100, of_word(w));
                    let _ = std::hint::black_box(list(garbage()));
                    super::collect();
                    crate::process::icell_put_x(c, ys.0);
                }
            },
        );
        for _ in 0..ROUNDS {
            let c = sent(p);
            let _ = std::hint::black_box(list(garbage()));
            super::collect();
            let w = crate::process::icell_get(c);
            let _ = std::hint::black_box(list(garbage()));
            super::collect();
            assert_eq!(sum(of_word(w)), 99 * 100 / 2 + 1000 * 299 * 300 / 2);
        }
    }
}
