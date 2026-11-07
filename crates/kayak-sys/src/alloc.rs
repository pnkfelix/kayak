//! kayak's own allocator (C370; docs/research/async-everywhere.md on the
//! branch experiment/async-everywhere): a
//! `GlobalAlloc` for small blocks, one free list per size class per
//! thread, refilled by bumping through chunks taken from the system; large
//! or over-aligned blocks, and the chunks themselves, the system's.
//!
//! What it is for: the boxed futures of KFX's Rust made `async` (each
//! awaited call a box, freed when the call completes, last in, first out),
//! whose `malloc` and `free` were 38% of the compiler's run. A block freed
//! on another thread than made it goes on that thread's list: blocks are
//! never returned to the system, only used again.
//!
//! The thread's state is a `const` thread local with no destructor, so
//! that allocating from it never allocates, nor registers anything.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::ptr;

/// Blocks are multiples of 16 bytes, up to `LARGEST`.
const GRAIN: usize = 16;
const LARGEST: usize = 1024;
const CLASSES: usize = LARGEST / GRAIN;
/// What a thread takes from the system at once, to bump through.
const CHUNK: usize = 1 << 20;

struct Local {
    free: [Cell<*mut u8>; CLASSES],
    bump: Cell<*mut u8>,
    end: Cell<*mut u8>,
}

thread_local! {
    static LOCAL: Local = const {
        Local {
            free: [const { Cell::new(ptr::null_mut()) }; CLASSES],
            bump: Cell::new(ptr::null_mut()),
            end: Cell::new(ptr::null_mut()),
        }
    };
}

/// The allocator, for a binary's `#[global_allocator]`.
pub struct Alloc;

fn class(layout: Layout) -> Option<usize> {
    if layout.size() == 0 || layout.size() > LARGEST || layout.align() > GRAIN {
        return None;
    }
    Some(layout.size().div_ceil(GRAIN) - 1)
}

unsafe impl GlobalAlloc for Alloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let Some(k) = class(layout) else {
            // SAFETY: the caller's layout, given to the system as is.
            return unsafe { System.alloc(layout) };
        };
        let got = LOCAL.try_with(|l| {
            let head = l.free[k].get();
            if !head.is_null() {
                // SAFETY: a free block of this class holds the next one's
                // address in its first word, written by `dealloc`.
                l.free[k].set(unsafe { *(head as *mut *mut u8) });
                return head;
            }
            let size = (k + 1) * GRAIN;
            let mut at = l.bump.get();
            if at.is_null() || (l.end.get() as usize) - (at as usize) < size {
                // SAFETY: a chunk of CHUNK bytes, aligned to GRAIN, never freed.
                at = unsafe { System.alloc(Layout::from_size_align_unchecked(CHUNK, GRAIN)) };
                if at.is_null() {
                    return at;
                }
                // SAFETY: `at` begins CHUNK bytes just allocated.
                l.end.set(unsafe { at.add(CHUNK) });
            }
            // SAFETY: `size` bytes from `at` are within the chunk.
            l.bump.set(unsafe { at.add(size) });
            at
        });
        match got {
            Ok(p) => p,
            // The thread's state gone (its end): the system's.
            // SAFETY: as above.
            Err(_) => unsafe { System.alloc(layout) },
        }
    }

    unsafe fn dealloc(&self, p: *mut u8, layout: Layout) {
        let Some(k) = class(layout) else {
            // SAFETY: a large block came from the system, with this layout.
            return unsafe { System.dealloc(p, layout) };
        };
        // A small block freed at a thread's end is left: the chunks are
        // never returned anyway.
        let _ = LOCAL.try_with(|l| {
            // SAFETY: `p` is a block of this class no one uses now: its
            // first word holds the list's head.
            unsafe { *(p as *mut *mut u8) = l.free[k].get() };
            l.free[k].set(p);
        });
    }
}
