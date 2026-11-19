//! What the heap holds that the stack reaches (C480), with the
//! `heap-stats` feature: the mark of a collector (`TODO.md` §54), as a
//! measure, nothing freed. Each heap arena's block is noted as it is made,
//! with how a cell of its type is traced; each block kayak-rt leaks for
//! itself (a closure, `lambdaN`) noted opaque, its size alone; each
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
use std::cell::OnceCell;
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

/// A cell at `addr`, traced if it holds a value.
///
/// # Safety
/// `addr` is a cell of a block `block` noted for `T`, which the heap keeps.
unsafe fn trace_cell<T: Visit + 'static>(addr: usize, t: &mut Tracer<'static>) {
    // SAFETY: the caller's: a `OnceCell<T>` of a heap arena, never freed.
    let c: &'static OnceCell<T> = unsafe { &*(addr as *const OnceCell<T>) };
    if let Some(v) = c.get() {
        t.reach(v);
    }
}

/// A heap arena's new block, of `len` cells of `T` from `p`.
pub(crate) fn block<T: Visit + 'static>(p: *const OnceCell<T>, len: usize) {
    let cell = std::mem::size_of::<OnceCell<T>>();
    let start = p as usize;
    let mut b = BLOCKS.lock().unwrap_or_else(|e| e.into_inner());
    b.push(Block {
        start,
        end: start + len * cell,
        cell,
        trace: trace_cell::<T>,
    });
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

/// What a mark found.
#[derive(Default)]
struct Found {
    stack_words: u64,
    roots: u64,
    opaque_blocks: u64,
    opaque_bytes: u64,
    via_opaque: u64,
}

/// The stack and the constants traced: what they reach, said on standard
/// error, a line of numbers and the types reached the most.
#[inline(never)]
pub(crate) fn mark(allocated: u64) {
    let here = 0usize;
    let from = std::ptr::from_ref(&here) as usize;
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
    loop {
        while let Some((w, inside)) = words.pop() {
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
    for r in rows.iter().take(8) {
        eprintln!("mark:   {:>9.2} MB {:>9} {}", r.2 as f64 / 1e6, r.1, r.0);
    }
}
