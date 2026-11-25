//! kc's code's heap in process (C538; `docs/research/one-runtime.md`, step
//! 3): this thread's pages, kc's objects collected with kayak-rt's by its
//! collector, where they were kcrt's semispaces, collected by Cheney's
//! copy. A built program keeps those (it has no kayak-rt).
//!
//! kc's code bumps `%r12` to `%r13` in line. It is lent the rest of the
//! thread's current page (`space::lend`), or a page or a large object
//! made for it; each time it asks for more (`kc_collect`, at a page's end;
//! or a routine of kcrt's that makes something in its heap), what it made
//! since is given back (`space::lent_back`): each object's start noted,
//! read by its header, as an allocation of kayak-rt's notes its own, so
//! that a collection finds them; then more lent, a collection perhaps run
//! first (`refill`'s policy). Its roots: its frames, on this thread's
//! stack, below the Rust frame that called it, which a collection scans;
//! its globals' cells, scanned while it runs (`roots::native_cells`). Its
//! stores into the heap pass kayak-rt's barrier (`kc_remember`). A run's
//! room it never gave back (it ended, or failed) is dropped and zeroed
//! again (`done`, `kayak:kc-done/1`), its objects dead.

use std::cell::Cell;

use kayak_kcrt_core::{B_CELLS_HI, B_CELLS_LO, B_HI, B_NEXT, Heap, HeapHost};

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
fn given_back(block: *mut u64) {
    if let Some(l) = LENT.take() {
        if l.block != block as usize {
            crate::fail("kc: two runs of kc's code on one thread at once, which share no heap yet");
        }
        if !l.large {
            super::space::lent_back(l.from, block_word(block, B_NEXT) as usize);
        }
    }
}

/// Room for `w` words lent the run, its cells its roots from now.
fn lent(block: *mut u64, w: usize) -> (usize, usize) {
    super::roots::native_cells((
        block_word(block, B_CELLS_LO) as usize,
        block_word(block, B_CELLS_HI) as usize,
    ));
    let (from, end, large) = super::space::lend(w);
    LENT.set(Some(Lent {
        block: block as usize,
        from,
        large,
    }));
    (from, end)
}

/// `kc_collect`'s: room for `need` bytes, kc's code's heap after.
fn room(need: u64, block: *mut u64) -> Result<Heap, String> {
    given_back(block);
    let (from, end) = lent(block, need.div_ceil(8) as usize);
    Ok(Heap {
        next: from as u64,
        end: end as u64,
    })
}

/// A heap routine's (kcrt's `gc::words`): `n` words, made by the routine
/// at once; the heap after them its block's (`B_NEXT`, `B_HI`), which its
/// wrapper makes kc's code's.
fn words(block: *mut u64, n: u64) -> Result<u64, String> {
    given_back(block);
    let (from, end) = lent(block, n as usize);
    set_block_word(block, B_NEXT, from as u64 + 8 * n);
    set_block_word(block, B_HI, end as u64);
    Ok(from as u64)
}

fn remember(slot: u64) {
    super::space::remember_slot(slot as usize);
}

/// A run's start: what one before left lent dropped.
fn reset() {
    done();
}

/// kc's heap, kayak-rt's, from now on, for every run in process (once).
pub(crate) fn install() {
    kayak_kcrt_core::set_host(HeapHost {
        room,
        words,
        remember,
        reset,
    });
}

/// A run over (C538, `kayak:kc-done/1`): the room it was lent and did not
/// give back dropped, zeroed again; its cells no roots.
pub(crate) fn done() {
    if let Some(l) = LENT.take()
        && !l.large
    {
        super::space::lent_dropped(l.from);
    }
    super::roots::native_cells((0, 0));
}
