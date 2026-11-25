//! kc's collector (C453; TODO §52, step 4): Cheney's, of two spaces,
//! over objects of Fixpt's layout, as Coopt's collector's baseline is
//! (`coopt-heap`'s `forward`, `cheney`) and tier 1's in KFX
//! (`cells/heap.kfx`): a forwarding word, tag 7, holds its copy's
//! address, in a pair's first word, in a bloblet's header, and in the word
//! just below its suffix when that is no trailer.
//!
//! One copy, a built program's and kc's code's in process alike (C452).
//! kc's code calls it where an allocation does not fit (x64.kfx's
//! `kc_gc_call`), given what it needs, its next free word, its stack
//! pointer and its block (below). The roots, as Larceny's are, by no map:
//! - kc's stack, every word from the stack pointer to the base its entry
//!   kept, each frame zeroed when made, so that each word is a value of
//!   Fixpt's format, a return address (in the code, outside the heap), or
//!   a value a register held, pushed for the call;
//! - the program's globals' cells, one run in the entry object (from
//!   `kc_cells_lo` to `kc_cells_hi`).
//!
//! What this runtime makes for kc's code, its strings and pairs, it makes
//! in kc's heap too (C455, [`words`]), collected as kc's own are: a
//! routine that makes one is called with its arguments in stack words the
//! collector moves (x64.kfx's `x64-heap-routine`), and reads them after it
//! has its room. Only its symbols are its own, interned for the run, as
//! Fixpt's symbol table keeps its heap's.
//!
//! Only a word that points into the space being collected is followed:
//! this runtime's symbols, the code's addresses and a literal's data are
//! none of it. No generations, so no barrier. An object's extent and
//! traced fields are read as kayak-rt's collector reads them
//! (`kayak_heap::object_at`, C535).

use std::cell::{Cell, RefCell};

use crate::{TAG_BLOBLET, TAG_HEADER, TAG_MASK, TAG_PAIR, TAG_TRAILER};

const TAG_FORWARD: u64 = kayak_layout::tag("forward");
/// A header's large bit (Fixpt's `header-large`), its field count past
/// what the header holds, in a word below it: kc makes none.
const LARGE: u64 = 1 << kayak_layout::H_LARGE.lo;
/// Words a space may grow to: 2^31, Fixpt's semispace's most.
const MOST_WORDS: u64 = 1 << 31;

/// Where things are, a block of the entry object's (`kc_gc`), its
/// writable view's address given: the space's first word and its end, the
/// stack's base, and the globals' cells' run (their writable addresses);
/// and, for a routine that makes something in the heap (C455), the heap's
/// next free word and the stack pointer where its caller's frames start.
const B_LO: usize = 0;
const B_HI: usize = 1;
const B_STACK: usize = 2;
const B_CELLS_LO: usize = 3;
const B_CELLS_HI: usize = 4;
const B_NEXT: usize = 5;
const B_SP: usize = 6;

/// The heap after a collection, in `%rax` and `%rdx`: its next free word
/// and its end; a next of 0, a failure (in process).
#[repr(C)]
pub struct Heap {
    pub next: u64,
    pub end: u64,
}

std::thread_local! {
    /// The spaces this collector made, the one in use last: the one before
    /// it freed when it is copied from. The first space is the caller's.
    static SPACES: RefCell<Vec<Vec<u64>>> = const { RefCell::new(Vec::new()) };
    /// Collections so far, and words copied.
    static COLLECTIONS: Cell<u64> = const { Cell::new(0) };
}

/// The spaces this collector made dropped (C454): a run in process starts
/// again from the space it is given, nothing of the run before reachable.
pub(crate) fn reset() {
    SPACES.with(|s| s.borrow_mut().clear());
}

/// Collections so far in this thread.
pub fn collections() -> u64 {
    COLLECTIONS.with(|c| c.get())
}

fn get(a: u64) -> u64 {
    // SAFETY: an address of a word of the space being collected, of kc's
    // stack between its pointer and its base, or of a cell or pair the run
    // keeps: each live for the collection.
    unsafe { *(a as *const u64) }
}

fn put(a: u64, w: u64) {
    // SAFETY: as `get`; each writable (a cell through its writable view).
    unsafe { *(a as *mut u64) = w }
}

struct Copy {
    lo: u64,
    hi: u64,
    to_base: u64,
    top: u64,
}

impl Copy {
    fn from(&self, a: u64) -> bool {
        a >= self.lo && a < self.hi
    }

    fn alloc(&mut self, n: u64) -> u64 {
        let at = self.top;
        self.top += 8 * n;
        at
    }

    fn copied(&mut self, from: u64, n: u64) -> u64 {
        let at = self.alloc(n);
        for k in 0..n {
            put(at + 8 * k, get(from + 8 * k));
        }
        at
    }

    /// The word for `w` after the collection: its object's copy, copied
    /// first if need be; itself if it points nowhere in the space.
    fn forward(&mut self, w: u64) -> Result<u64, String> {
        match w & TAG_MASK {
            TAG_PAIR => {
                let a = w - TAG_PAIR;
                if !self.from(a) {
                    return Ok(w);
                }
                let first = get(a);
                if first & TAG_MASK == TAG_FORWARD {
                    return Ok((first - TAG_FORWARD) + TAG_PAIR);
                }
                let at = self.copied(a, 2);
                put(a, at | TAG_FORWARD);
                Ok(at + TAG_PAIR)
            }
            TAG_BLOBLET => {
                let p = w - TAG_BLOBLET;
                if !self.from(p - 8) {
                    return Ok(w);
                }
                let before = get(p - 8);
                if before & TAG_MASK == TAG_FORWARD {
                    return Ok((before - TAG_FORWARD) + 8 + TAG_BLOBLET);
                }
                let h = match before & TAG_MASK {
                    TAG_TRAILER => p - 8 - 8 * (before >> 3),
                    TAG_HEADER => p - 8,
                    // A field, no trailer: the header the first word back
                    // that is one (fields are values, none a header).
                    _ => {
                        let mut h = p - 16;
                        while get(h) & TAG_MASK != TAG_HEADER && get(h) & TAG_MASK != TAG_FORWARD {
                            h -= 8;
                        }
                        h
                    }
                };
                let head = get(h);
                if head & TAG_MASK == TAG_FORWARD {
                    return Ok((head - TAG_FORWARD) + (p - h) + TAG_BLOBLET);
                }
                if head & LARGE != 0 {
                    return Err("kc: not yet: a large bloblet collected".to_string());
                }
                // Its extent as kayak-rt's collector reads one (C535).
                let o = kayak_heap::object_at(h as usize, |a| get(a as u64));
                let (f, n) = (o.fields as u64, (o.end - o.main) as u64 / 8);
                let at = self.copied(h, n);
                put(h, at | TAG_FORWARD);
                if f > 0 && before & TAG_MASK != TAG_TRAILER {
                    put(p - 8, (at + 8 * f) | TAG_FORWARD);
                }
                Ok(at + (p - h) + TAG_BLOBLET)
            }
            _ => Ok(w),
        }
    }

    fn forward_at(&mut self, a: u64) -> Result<(), String> {
        let w = self.forward(get(a))?;
        put(a, w);
        Ok(())
    }

    /// Cheney's scan: each copy's traced words forwarded, until the scan
    /// meets the copying.
    fn scan(&mut self) -> Result<(), String> {
        let mut i = self.to_base;
        while i < self.top {
            let w = get(i);
            if w & TAG_MASK == TAG_HEADER {
                // Its fields that hold values, Fixpt's table's (C535;
                // `kayak_heap`'s walk, kayak-rt's collector's too).
                let o = kayak_heap::object_at(i as usize, |a| get(a as u64));
                for k in 1..=o.traced as u64 {
                    self.forward_at(i + 8 * k)?;
                }
                i = o.end as u64;
            } else {
                self.forward_at(i)?;
                self.forward_at(i + 8)?;
                i += 16;
            }
        }
        Ok(())
    }
}

/// Everything the roots reach copied from the space `[lo, hi)` into a
/// new one of `words`; the new one's first word and its next free one.
fn copy_all(lo: u64, hi: u64, sp: u64, block: *mut u64, words: u64) -> Result<(u64, u64), String> {
    let b = |k: usize| get(block as u64 + 8 * k as u64);
    let mut space = vec![0u64; words as usize];
    let to_base = space.as_mut_ptr() as u64;
    let mut c = Copy {
        lo,
        hi,
        to_base,
        top: to_base,
    };
    let mut a = sp;
    while a < b(B_STACK) {
        c.forward_at(a)?;
        a += 8;
    }
    let mut a = b(B_CELLS_LO);
    while a < b(B_CELLS_HI) {
        c.forward_at(a)?;
        a += 8;
    }
    c.scan()?;
    let top = c.top;
    SPACES.with(|s| {
        let mut s = s.borrow_mut();
        // The space copied from, if it is one of these, freed.
        s.retain(|v| v.as_ptr() as u64 != lo);
        s.push(space);
    });
    put(block as u64 + 8 * B_LO as u64, to_base);
    put(block as u64 + 8 * B_HI as u64, to_base + 8 * words);
    Ok((to_base, top))
}

/// A collection, for `need` bytes more: the space copied into one as big,
/// or, if what lives and what is needed fill more than half of it, again
/// into one big enough that they fill half.
pub(crate) fn collect(need: u64, sp: u64, block: *mut u64) -> Result<Heap, String> {
    let b = |k: usize| get(block as u64 + 8 * k as u64);
    let (lo, hi) = (b(B_LO), b(B_HI));
    let words = (hi - lo) / 8;
    let need_words = need.div_ceil(8);
    let (base, top) = copy_all(lo, hi, sp, block, words)?;
    let live = (top - base) / 8;
    COLLECTIONS.with(|c| c.set(c.get() + 1));
    if 2 * (live + need_words) <= words {
        return Ok(Heap {
            next: top,
            end: base + 8 * words,
        });
    }
    let mut more = 2 * words;
    while more < 2 * (live + need_words) {
        more *= 2;
    }
    if more > MOST_WORDS {
        return Err(crate::fail_words(5).to_string());
    }
    let (base, top) = copy_all(base, base + 8 * words, sp, block, more)?;
    COLLECTIONS.with(|c| c.set(c.get() + 1));
    Ok(Heap {
        next: top,
        end: base + 8 * more,
    })
}

/// Room for `n` words in kc's heap, for a routine of this runtime's (C455):
/// the heap's next free word as its block has it, the heap collected first
/// if they do not fit, the caller's frames from the stack pointer its
/// block has; the block's next word moved past them, for the caller to
/// take back. What the routine was given it reads after, from the stack
/// words the collection moved.
pub(crate) fn words(block: *mut u64, n: u64) -> Result<u64, String> {
    let b = |k: usize| get(block as u64 + 8 * k as u64);
    let mut next = b(B_NEXT);
    if next + 8 * n > b(B_HI) {
        next = collect(8 * n, b(B_SP), block)?.next;
    }
    put(block as u64 + 8 * B_NEXT as u64, next + 8 * n);
    Ok(next)
}
