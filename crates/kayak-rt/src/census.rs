//! What the heap holds that the stack reaches (C480), with the
//! `heap-stats` feature: the mark of a collector (`TODO.md` §54), as a
//! measure, nothing freed. Each object of the heap's space is noted as it
//! is made, with how one of its type is traced (C507; before, each heap
//! arena's block); each block kayak-rt leaks for
//! itself (a continuation) noted opaque, its size alone; each
//! constant noted as it is first made. A mark, every `KAYAK_HEAP_MARK`th
//! allocation: the run's stack read word by word, from here to its top
//! (Bartlett's: conservative), each word in a noted block its cell's
//! root; an opaque block so reached read the same way; then each root and
//! constant traced, precisely, by its type's `Trace`.
//!
//! Reading the stack, and a cell by its address, is `unsafe`, which
//! kayak-rt has nowhere else but `sum.rs`: here only with the feature, a
//! measure's, never a run's.

#![allow(unsafe_code)]

use crate::trace::{Trace, Tracer, Visit};
use std::sync::Mutex;

/// A heap arena's block: its cells' addresses, a cell's size, how one is
/// traced, its type's name.
struct Block {
    start: usize,
    end: usize,
    cell: usize,
    trace: unsafe fn(usize, &mut Tracer<'static>),
}

static BLOCKS: Mutex<Vec<Block>> = Mutex::new(Vec::new());
static OPAQUE: Mutex<Vec<(usize, usize)>> = Mutex::new(Vec::new());
type Root = Box<dyn Fn(&mut Tracer<'static>)>;
std::thread_local! {
    /// The constants this thread made: each one's slot, read again and
    /// traced (the run's constants, its marks' thread's).
    static CONSTANTS: std::cell::RefCell<Vec<Root>> = const { std::cell::RefCell::new(Vec::new()) };
}
static TOP: Mutex<usize> = Mutex::new(0);

/// An object kept in a block of its own (a closure, C481): its bytes, and
/// it, visited as it is.
struct One {
    start: usize,
    end: usize,
    it: *const (dyn Visit + 'static),
}

// SAFETY: read only by a mark, under the lock, of objects never freed.
unsafe impl Send for One {}

static ONES: Mutex<Vec<One>> = Mutex::new(Vec::new());

/// An object kept in a block of its own, never freed.
pub(crate) fn one<'r>(v: &'r (dyn Visit + 'r)) {
    let start = std::ptr::from_ref(v).cast::<()>() as usize;
    let size = v.visit_size().max(1);
    // SAFETY: a lifetime only, erased: the object is never freed, and a
    // mark reads it as kept.
    let it: *const (dyn Visit + 'static) = unsafe { std::mem::transmute(std::ptr::from_ref(v)) };
    ONES.lock().unwrap_or_else(|e| e.into_inner()).push(One {
        start,
        end: start + size,
        it,
    });
}

/// An object of the heap's space (C507), at `addr`, a `T`, reached.
///
/// # Safety
/// `addr` is a `T` the space keeps.
unsafe fn trace_kept<T: Visit + 'static>(addr: usize, t: &mut Tracer<'static>) {
    // SAFETY: the caller's.
    t.reach(unsafe { &*(addr as *const T) });
}

/// An object of the heap's space, made: noted with how it is traced.
pub(crate) fn kept<T: Visit + 'static>(r: &'static T) {
    object(
        std::ptr::from_ref(r) as usize,
        std::mem::size_of::<T>(),
        trace_kept::<T>,
    );
}

/// A block kayak-rt leaked for itself, of `size` bytes from `p`: opaque.
pub(crate) fn opaque(p: usize, size: usize) {
    if size > 0 {
        OPAQUE
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .push((p, size));
    }
}

/// An object of Fixpt's layout kept for the run (an array, a bloblet,
/// C486), of `size` bytes from `p`, traced by `trace` from its address.
pub(crate) fn object(p: usize, size: usize, trace: unsafe fn(usize, &mut Tracer<'static>)) {
    BLOCKS
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push(Block {
            start: p,
            end: p + size,
            cell: size,
            trace,
        });
}

/// A constant, made: its slot, read again and traced at each mark.
pub(crate) fn constant_of<T: Trace + 'static, S: crate::once::OnceSlot<T> + ?Sized>(
    slot: &'static S,
) {
    let root: Root = Box::new(move |t: &mut Tracer<'static>| {
        if let Some(v) = slot.made() {
            v.trace(t);
        }
    });
    CONSTANTS.with(|c| c.borrow_mut().push(root));
}

/// The top of the run's stack (`with_big_stack`'s thread, at its start).
pub(crate) fn stack_top(addr: usize) {
    *TOP.lock().unwrap_or_else(|e| e.into_inner()) = addr;
}

/// The block holding `w`, by a sorted list's search.
fn found<T>(
    sorted: &[T],
    w: usize,
    start: impl Fn(&T) -> usize,
    end: impl Fn(&T) -> usize,
) -> Option<&T> {
    let i = sorted.partition_point(|b| start(b) <= w);
    if i == 0 {
        return None;
    }
    let b = &sorted[i - 1];
    if w < end(b) { Some(b) } else { None }
}

/// What the stack's words reach in the heap's space, traced twice (C508):
/// by each object's type (`Trace`, a census's way), and by its layout
/// alone (`obj::laid`, a collector's way), from the same roots, the
/// objects the stack's words are in that a census noted; each as the
/// allocations reached, by their first words. Said: how many each, and
/// how many one reaches and the other not.
fn by_layout(stack: &[usize], blocks: &[Block], ones: &[One]) {
    use crate::obj::{laid, space};
    use std::collections::{BTreeSet, HashSet};
    let mut roots = BTreeSet::new();
    for &w in stack {
        if let Some(a) = space::object_of(w)
            && (found(blocks, a, |b| b.start, |b| b.end).is_some_and(|b| b.start == a)
                || found(ones, a, |o| o.start, |o| o.end).is_some_and(|o| o.start == a))
        {
            roots.insert(a);
        }
    }
    // By type.
    let mut t = Tracer::default();
    let mut more: Vec<usize> = roots.iter().copied().collect();
    let mut opened = HashSet::new();
    while !more.is_empty() {
        for a in more.drain(..) {
            if !opened.insert(a) {
                continue;
            }
            if let Some(b) = found(blocks, a, |b| b.start, |b| b.end) {
                // SAFETY: a noted block's object, at its start.
                unsafe { (b.trace)(b.start + (a - b.start) / b.cell * b.cell, &mut t) };
            } else if let Some(o) = found(ones, a, |o| o.start, |o| o.end) {
                // SAFETY: an object `one` noted, never freed.
                t.reach(unsafe { &*o.it });
            }
        }
        t.finish();
        more = t
            .take_opaque()
            .into_iter()
            .filter_map(space::object_of)
            .collect();
    }
    let typed: BTreeSet<usize> = t
        .seen()
        .filter_map(space::object_of)
        .chain(roots.iter().copied())
        .collect();
    // By layout.
    let mut laid_out = BTreeSet::new();
    let mut work: Vec<usize> = roots.iter().copied().collect();
    while let Some(a) = work.pop() {
        if !laid_out.insert(a) {
            continue;
        }
        // SAFETY: an allocation's first word in the space.
        for o in unsafe { laid::allocation(a) } {
            for &w in o.fields {
                if let Some(to) = laid::target(w).and_then(space::object_of)
                    && !laid_out.contains(&to)
                {
                    work.push(to);
                }
            }
        }
    }
    let mut kinds: std::collections::BTreeMap<u64, u64> = Default::default();
    for &a in laid_out.difference(&typed) {
        // SAFETY: an allocation's first word in the space.
        let h = unsafe { *(a as *const u64) };
        let k = if h & 7 == 0b110 {
            (h >> 3) & 0xff
        } else {
            1000 + (h & 7)
        };
        *kinds.entry(k).or_default() += 1;
    }
    if !kinds.is_empty() {
        eprintln!("mark:   by layout only, kinds (1000+ a pair, by its head's tag): {kinds:?}");
    }
    eprintln!(
        "mark:   by layout, from {} roots: {} allocations, by type {}; by layout only {}, by type only {}",
        roots.len(),
        laid_out.len(),
        typed.len(),
        laid_out.difference(&typed).count(),
        typed.difference(&laid_out).count()
    );
}

/// The roots (`obj::roots`, `TODO.md` §54's step 3) and what they reach
/// by the space's bits and headers alone, against what the typed trace
/// reached of the space: each allocation the types reach should be one
/// the roots do.
fn by_roots(t: &Tracer<'static>) {
    use crate::obj::space;
    let typed: std::collections::BTreeSet<usize> = t.seen().filter_map(space::object_of).collect();
    let r = crate::obj::roots::reached();
    let typed_only: Vec<usize> = typed.difference(&r.allocations).copied().collect();
    eprintln!(
        "mark:   roots: {} register words, {} stack, {} constants', {} owned blocks' ({} words), {} thread locals', {} in the space; {} kept objects reached ({} words); by layout {} allocations, by type {}; by type only {}, by layout only {}",
        r.registers,
        r.stack,
        r.constants,
        r.owned_blocks,
        r.owned,
        r.locals,
        r.roots_in_space,
        r.kept,
        r.kept_words,
        r.allocations.len(),
        typed.len(),
        typed_only.len(),
        r.allocations.difference(&typed).count()
    );
    let kinds = |set: &mut dyn Iterator<Item = usize>| {
        let mut kinds: std::collections::BTreeMap<u64, u64> = Default::default();
        for a in set {
            // SAFETY: an allocation's first word in the space.
            let h = unsafe { *(a as *const u64) };
            let k = if h & 7 == 0b110 {
                (h >> 3) & 0xff
            } else {
                1000 + (h & 7)
            };
            *kinds.entry(k).or_default() += 1;
        }
        kinds
    };
    if !typed_only.is_empty() {
        eprintln!(
            "mark:   by type only, kinds (1000+ a pair, by its head's tag): {:?}",
            kinds(&mut typed_only.iter().copied())
        );
    }
    eprintln!(
        "mark:   by the roots' layout only, kinds: {:?}",
        kinds(&mut r.allocations.difference(&typed).copied())
    );
}

/// What a mark found.
#[derive(Default)]
struct Found {
    stack_words: u64,
    roots: u64,
    opaque_blocks: u64,
    opaque_bytes: u64,
    via_opaque: u64,
    /// Words in an object of the heap's space, by its bits of starts
    /// (C507), and those whose start is not a noted object's.
    in_space: u64,
    unnoted: u64,
    /// Of those, a string's, which a census counts by its trace alone.
    unnoted_strings: u64,
}

/// The stack and the constants traced: what they reach, said on standard
/// error, a line of numbers and the types reached the most.
#[inline(never)]
pub(crate) fn mark(allocated: u64) {
    // The stack read from this frame, a small one: the mark's own (its
    // locals, as they are first written) below it, not read as roots.
    let here = 0usize;
    mark_from(allocated, std::ptr::from_ref(&here) as usize);
    std::hint::black_box(&here);
}

#[inline(never)]
fn mark_from(allocated: u64, from: usize) {
    let top = *TOP.lock().unwrap_or_else(|e| e.into_inner());
    if top <= from {
        return;
    }
    let mut blocks = BLOCKS.lock().unwrap_or_else(|e| e.into_inner());
    blocks.sort_by_key(|b| b.start);
    let mut opaque = OPAQUE.lock().unwrap_or_else(|e| e.into_inner());
    opaque.sort_by_key(|b| b.0);
    let mut ones = ONES.lock().unwrap_or_else(|e| e.into_inner());
    ones.sort_by_key(|o| o.start);
    let mut t = Tracer::default();
    let mut f = Found::default();
    let mut seen_opaque = std::collections::HashSet::new();
    // Words to look at: the stack's; then each opaque block's reached, and
    // each the trace reports (a plain reference: a closure in a value).
    let mut words: Vec<(usize, bool)> = Vec::new();
    let mut at = (from + 7) & !7;
    while at + 8 <= top {
        // SAFETY: an aligned word of this thread's own stack, between this
        // frame and the stack's top, which the thread is running on.
        words.push((
            unsafe { std::ptr::read_volatile(at as *const usize) },
            false,
        ));
        at += 8;
        f.stack_words += 1;
    }
    by_layout(
        &words.iter().map(|&(w, _)| w).collect::<Vec<_>>(),
        &blocks,
        &ones,
    );
    loop {
        while let Some((w, inside)) = words.pop() {
            if let Some(start) = crate::obj::space::object_of(w) {
                f.in_space += 1;
                if found(&blocks, start, |b| b.start, |b| b.end).is_none()
                    && found(&ones, start, |o| o.start, |o| o.end).is_none()
                {
                    f.unnoted += 1;
                    // SAFETY: the first word of an object of the space,
                    // kept for the run.
                    let h = unsafe { *(start as *const u64) };
                    if h & 7 == 0b110 && (h >> 3) & 0xff == 1 {
                        f.unnoted_strings += 1;
                    }
                }
            }
            if let Some(b) = found(&blocks, w, |b| b.start, |b| b.end) {
                f.roots += 1;
                if inside {
                    f.via_opaque += 1;
                }
                // SAFETY: a cell of a noted block, at a cell's start.
                unsafe { (b.trace)(b.start + (w - b.start) / b.cell * b.cell, &mut t) };
            } else if let Some(o) = found(&ones, w, |o| o.start, |o| o.end) {
                f.roots += 1;
                if inside {
                    f.via_opaque += 1;
                }
                // SAFETY: an object `one` noted, never freed.
                t.reach(unsafe { &*o.it });
            } else if let Some(&(p, n)) = found(&opaque, w, |b| b.0, |b| b.0 + b.1)
                && seen_opaque.insert(p)
            {
                {
                    f.opaque_blocks += 1;
                    f.opaque_bytes += n as u64;
                    let mut q = (p + 7) & !7;
                    while q + 8 <= p + n {
                        // SAFETY: an aligned word inside a block kayak-rt
                        // leaked, never freed, of `n` bytes from `p`.
                        words.push((unsafe { std::ptr::read_unaligned(q as *const usize) }, true));
                        q += 8;
                    }
                }
            }
        }
        CONSTANTS.with(|cs| {
            for c in cs.borrow().iter() {
                c(&mut t);
            }
        });
        t.finish();
        let more = t.take_opaque();
        if more.is_empty() {
            break;
        }
        words.extend(more.into_iter().map(|p| (p, true)));
    }
    by_roots(&t);
    let (rows, strings) = t.census();
    let (objects, bytes) = rows.iter().fold((0, 0), |(n, b), r| (n + r.1, b + r.2));
    eprintln!(
        "mark: {allocated} allocated; stack {} words, {} roots ({} via opaque); reached {objects} values, {:.1} MB, strings {} ({:.1} MB); opaque {} blocks, {:.1} MB",
        f.stack_words,
        f.roots,
        f.via_opaque,
        bytes as f64 / 1e6,
        strings.0,
        strings.1 as f64 / 1e6,
        f.opaque_blocks,
        f.opaque_bytes as f64 / 1e6
    );
    let (chunks, pages, large) = crate::obj::space::extent();
    eprintln!(
        "mark:   space {:.1} MB in chunks, {pages} pages given, {:.1} MB large; {} words in its objects, {} not noted ({} of them strings)",
        chunks as f64 / 1e6,
        large as f64 / 1e6,
        f.in_space,
        f.unnoted,
        f.unnoted_strings
    );
    for r in rows.iter().take(8) {
        eprintln!("mark:   {:>9.2} MB {:>9} {}", r.2 as f64 / 1e6, r.1, r.0);
    }
}
