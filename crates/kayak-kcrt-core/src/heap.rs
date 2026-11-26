//! kc's heap (C541; `docs/research/one-runtime.md`, step 3): the shared
//! heap's (`kayak_heap`), a thread's pages, collected by its collector, a
//! built program's and kc's code's in process alike. What was kcrt's own,
//! Cheney's copy of two spaces (C453, `gc.rs`), is gone; in process its
//! objects were kayak-rt's already (C538, kayak-rt's `obj::kc_heap`, which
//! this was). The collector's runtime (`kayak_heap::collect::Host`, its
//! roots) is the one thing that differs: kayak-rt's in process, the
//! built program's own in `kayak-kcrt` (its stack and its cells).
//!
//! kc's code bumps `%r12` to `%r13` in line. It is lent the rest of the
//! thread's current page (`space::lend`), or a page or a large object
//! made for it; each time it asks for more (`kc_collect`, at a page's end;
//! or a routine of this runtime's that makes something in its heap,
//! [`words`]), what it made since is given back (`space::lent_back`): each
//! object's start noted, read by its header, as an allocation of
//! kayak-rt's notes its own, so that a collection finds them; then more
//! lent, a collection perhaps run first (the space's policy). Its roots:
//! its frames, on this thread's stack, exact while it asks for room
//! (C542, `roots::with_native_frames`), so that what only they reach
//! moves; its globals' cells, exact too, while it runs
//! (`roots::native_cells`). Its stores into the heap pass the heap's
//! barrier (`kc_remember`). What a run made since it was last lent room is
//! given back as it ends ([`done`]), as any other, its objects dead.
//!
//! A routine that makes something in kc's heap is called with its
//! arguments in stack words (x64.kfx's `x64-heap-routine`), and reads them
//! after it has its room: a collection moves what they reach.

use std::cell::Cell;

use kayak_heap::space;

/// Where things are, a block of the entry object's (`kc_gc`, the state
/// block's words from 688), its writable view's address given: the heap's
/// first word and its end, the stack's base, and the globals' cells' run
/// (their writable addresses); and, for a routine that makes something in
/// the heap (C455), the heap's next free word and the stack pointer where
/// its caller's frames start.
pub const B_HI: usize = 1;
const B_STACK: usize = 2;
pub const B_CELLS_LO: usize = 3;
pub const B_CELLS_HI: usize = 4;
pub const B_NEXT: usize = 5;
const B_SP: usize = 6;

/// The heap after a collection, in `%rax` and `%rdx`: its next free word
/// and its end; a next of 0, a failure (in process).
#[repr(C)]
pub struct Heap {
    pub next: u64,
    pub end: u64,
}

#[derive(Clone, Copy)]
struct Lent {
    /// The block of the run lent it (`kc_gc`): one run at a time.
    block: usize,
    from: usize,
    large: bool,
}

std::thread_local! {
    static LENT: Cell<Option<Lent>> = const { Cell::new(None) };
}

/// Word `k` of the block.
fn block_word(block: *mut u64, k: usize) -> u64 {
    // SAFETY: the run's block, the state block's words from 688, which
    // kc's code gave the routine, alive while it runs.
    unsafe { *block.add(k) }
}

fn set_block_word(block: *mut u64, k: usize, w: u64) {
    // SAFETY: as `block_word`; the block is the state block's, writable.
    unsafe { *block.add(k) = w }
}

/// What the run made since its room was lent, given back: to where its
/// code's heap reached (`B_NEXT`, which `kc_gc_call` and a heap routine's
/// wrapper write).
fn given_back(block: *mut u64) -> Result<(), String> {
    if let Some(l) = LENT.take() {
        if l.block != block as usize {
            return Err(
                "kc: two runs of kc's code on one thread at once, which share no heap yet"
                    .to_string(),
            );
        }
        if !l.large {
            space::lent_back(l.from, block_word(block, B_NEXT) as usize);
        }
    }
    Ok(())
}

/// Room for `w` words lent the run, its cells its roots from now; its
/// frames, from `sp` to its run's base (`B_STACK`, which `kc_gc_init` and
/// `kc_call2` write), exact roots while the room is made (C542).
fn lent(block: *mut u64, sp: u64, w: usize) -> (usize, usize) {
    kayak_heap::roots::native_cells((
        block_word(block, B_CELLS_LO) as usize,
        block_word(block, B_CELLS_HI) as usize,
    ));
    let frames = (sp as usize, block_word(block, B_STACK) as usize);
    let (from, end, large) = kayak_heap::roots::with_native_frames(frames, || space::lend(w));
    LENT.set(Some(Lent {
        block: block as usize,
        from,
        large,
    }));
    (from, end)
}

/// `kc_collect`'s: room for `need` bytes, kc's code's heap after; its
/// frames from its stack pointer `sp`.
pub(crate) fn collect(need: u64, sp: u64, block: *mut u64) -> Result<Heap, String> {
    given_back(block)?;
    let (from, end) = lent(block, sp, need.div_ceil(8) as usize);
    Ok(Heap {
        next: from as u64,
        end: end as u64,
    })
}

/// Room for `n` words in kc's heap, for a routine of this runtime's
/// (C455): made by the routine at once; the heap after them its block's
/// (`B_NEXT`, `B_HI`), which its wrapper makes kc's code's.
pub(crate) fn words(block: *mut u64, n: u64) -> Result<u64, String> {
    given_back(block)?;
    // Its caller's frames from the stack pointer its wrapper kept, its
    // arguments' words first.
    let (from, end) = lent(block, block_word(block, B_SP), n as usize);
    set_block_word(block, B_NEXT, from as u64 + 8 * n);
    set_block_word(block, B_HI, end as u64);
    Ok(from as u64)
}

/// kc's code's barrier: a slot of a heap object written.
#[inline]
pub(crate) fn remember(slot: u64) {
    space::remember_slot(slot as usize);
}

/// A run's start: what one before left lent given back.
pub(crate) fn reset() {
    done();
    crate::sched::reset();
}

/// Collections so far.
pub fn collections() -> u64 {
    kayak_heap::collect::collections()
}

/// A run over (C538, kayak-rt's `kayak:kc-done/1`): what it made since its
/// room was lent given back, to where its heap reached as it ended (the
/// state block's `kc_heap_next`, word 680, the word before the block,
/// which `kc_entry`, `kc_call2` and `kc_failed` write), each object's
/// start noted; its cells no roots. Its objects are then the heap's as any
/// dead one is: not dropped and its words made the heap's at once, which
/// left a dead noted object of the run pointing into words made again with
/// no collection between (C540).
pub fn done() {
    if let Some(l) = LENT.take()
        && !l.large
    {
        // SAFETY: the run's state block's word 680, before its block.
        let next = unsafe { *(l.block as *const u64).sub(1) } as usize;
        space::lent_back(l.from, next);
    }
    kayak_heap::roots::native_cells((0, 0));
}
