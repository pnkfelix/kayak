//! The heap's space (C507; `TODO.md` §54, a collector's first step): every
//! object of the heap, of Fixpt's layout or a pair, made by bumping in a
//! page of the thread's, as Fixpt's heap is one space of words. Pages are
//! `PAGE_BYTES`, taken a page at a time from chunks the process shares, so
//! that a page is one thread's (the collector to come collects a thread's
//! pages, `TODO.md` §55 item 2's heap per thread). Each chunk keeps a bit
//! per word, set at an object's first word: where an object starts, found
//! from any word inside it (Bartlett's ambiguous roots, an interior
//! pointer), without reading what the heap holds. An object past
//! `LARGE_BYTES` is a block of its own, noted apart.
//!
//! Nothing is freed yet: the space's words are kept for the run, as the
//! arenas and leaked blocks it replaces were.

use std::cell::Cell;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

/// A page's bytes: what a thread takes at once, and what a collector
/// pins whole (Bartlett's).
pub(crate) const PAGE_BYTES: usize = 1 << 15;

/// A chunk's pages: what the process asks of the allocator at once.
const CHUNK_PAGES: usize = 64;
const CHUNK_BYTES: usize = PAGE_BYTES * CHUNK_PAGES;
const CHUNK_WORDS: usize = CHUNK_BYTES / 8;

/// Past this, an object is a block of its own: a page holds four at least.
pub(crate) const LARGE_BYTES: usize = PAGE_BYTES / 4;

/// A chunk: its words, and a bit for each, set where an object starts.
pub(crate) struct Chunk {
    base: *mut u64,
    starts: &'static [AtomicU64],
    /// For each page, the thread it is given to (its `ThreadId`'s
    /// number), 0 while no thread has it.
    owners: [AtomicU64; CHUNK_PAGES],
}

// SAFETY: a chunk's words are only bumped into by the thread its page is
// given to; its base, the address of memory kept for the run.
unsafe impl Send for Chunk {}
unsafe impl Sync for Chunk {}

/// The process's chunks, in the order made, and the next page not yet
/// given of the last.
struct Pool {
    chunks: Vec<&'static Chunk>,
    next_page: usize,
}

static POOL: Mutex<Pool> = Mutex::new(Pool {
    chunks: Vec::new(),
    next_page: CHUNK_PAGES,
});

/// Objects each a block of their own: their first word and words.
static LARGE: Mutex<Vec<(usize, usize)>> = Mutex::new(Vec::new());

/// The thread's page: where the next object goes, the page's end, and its
/// chunk.
struct Bump {
    at: Cell<*mut u64>,
    end: Cell<*mut u64>,
    chunk: Cell<Option<&'static Chunk>>,
}

thread_local! {
    static BUMP: Bump = const {
        Bump {
            at: Cell::new(std::ptr::null_mut()),
            end: Cell::new(std::ptr::null_mut()),
            chunk: Cell::new(None),
        }
    };
}

/// The words of a new object of `w` words, zeroed, its first noted an
/// object's start; kept for the run.
#[inline]
pub(crate) fn words(w: usize) -> *mut u64 {
    debug_assert!(w > 0, "an object of no words");
    BUMP.with(|b| {
        let at = b.at.get();
        // `at` and `end` of one page (or both null), `at` not past `end`.
        let room = (b.end.get().addr() - at.addr()) / 8;
        if room >= w && w * 8 <= LARGE_BYTES {
            // SAFETY: `w` words fit before the page's end.
            b.at.set(unsafe { at.add(w) });
            if let Some(c) = b.chunk.get() {
                c.note_start(at);
            }
            at
        } else {
            refill(b, w)
        }
    })
}

/// `words(w)`, as a slice.
#[inline]
pub(crate) fn words_slice(w: usize) -> &'static mut [u64] {
    // SAFETY: `w` zeroed words of the space, new, kept for the run, given
    // to the caller alone.
    unsafe { std::slice::from_raw_parts_mut(words(w), w) }
}

/// `v` in the space: an object of Fixpt's layout or a pair, of whole
/// words.
#[inline]
pub(crate) fn put<'a, T: 'a>(v: T) -> &'a T {
    const {
        assert!(
            std::mem::size_of::<T>() >= 8,
            "an object of less than a word"
        );
        assert!(
            std::mem::size_of::<T>().is_multiple_of(8),
            "an object not whole words"
        );
        assert!(
            std::mem::align_of::<T>() <= 8,
            "an object aligned past a word"
        );
    };
    let p = words(std::mem::size_of::<T>() / 8).cast::<T>();
    // SAFETY: new words of the space, enough for a `T` and aligned for
    // one, written once here, kept for the run.
    unsafe {
        p.write(v);
        &*p
    }
}

// What finds an object from a word: a census's now, the collector's to
// come (`TODO.md` §54).
#[cfg_attr(not(any(test, feature = "heap-stats")), allow(dead_code))]
impl Chunk {
    #[inline(always)]
    fn note_start(&self, p: *mut u64) {
        // SAFETY: `p` is a word of this chunk.
        let i = unsafe { p.offset_from(self.base) } as usize;
        let s = &self.starts[i / 64];
        s.store(s.load(Ordering::Relaxed) | 1 << (i % 64), Ordering::Relaxed);
    }

    /// Whether `addr` is a word of this chunk.
    fn holds(&self, addr: usize) -> bool {
        let b = self.base.addr();
        b <= addr && addr < b + CHUNK_BYTES
    }

    /// The first word of the object `addr` is inside, if it is in a page
    /// given to a thread: the nearest start at or before it.
    fn start_of(&self, addr: usize) -> Option<usize> {
        let i = (addr - self.base.addr()) / 8;
        let page = i * 8 / PAGE_BYTES;
        if self.owners[page].load(Ordering::Relaxed) == 0 {
            return None;
        }
        let first = page * PAGE_BYTES / 8;
        let mut k = i;
        loop {
            let bits = self.starts[k / 64].load(Ordering::Relaxed) & (u64::MAX >> (63 - k % 64));
            if bits != 0 {
                let j = k / 64 * 64 + 63 - bits.leading_zeros() as usize;
                return (j >= first).then(|| self.base.addr() + 8 * j);
            }
            if k / 64 * 64 <= first {
                return None;
            }
            k = k / 64 * 64 - 1;
        }
    }
}

/// The thread's page full, or none yet: an object of its own if large,
/// else a new page.
#[cold]
fn refill(b: &Bump, w: usize) -> *mut u64 {
    if w * 8 > LARGE_BYTES {
        return large(w);
    }
    let (chunk, page) = {
        let mut pool = POOL.lock().unwrap_or_else(|e| e.into_inner());
        if pool.next_page == CHUNK_PAGES {
            pool.chunks.push(new_chunk());
            pool.next_page = 0;
        }
        let page = pool.next_page;
        pool.next_page += 1;
        (*pool.chunks.last().expect("a chunk"), page)
    };
    chunk.owners[page].store(thread_number(), Ordering::Relaxed);
    // SAFETY: page `page` of the chunk, given to this thread alone.
    let start = unsafe { chunk.base.add(page * PAGE_BYTES / 8) };
    // SAFETY: as above; its end, one past its last word.
    let end = unsafe { start.add(PAGE_BYTES / 8) };
    b.chunk.set(Some(chunk));
    // SAFETY: `w` words fit a new page (`w` not large).
    b.at.set(unsafe { start.add(w) });
    b.end.set(end);
    chunk.note_start(start);
    start
}

/// A chunk, zeroed, kept for the run.
fn new_chunk() -> &'static Chunk {
    let layout = std::alloc::Layout::from_size_align(CHUNK_BYTES, PAGE_BYTES).expect("a chunk");
    // SAFETY: a layout of nonzero size.
    let base = unsafe { std::alloc::alloc_zeroed(layout) }.cast::<u64>();
    if base.is_null() {
        std::alloc::handle_alloc_error(layout)
    }
    let starts: &'static [AtomicU64] =
        Box::leak((0..CHUNK_WORDS / 64).map(|_| AtomicU64::new(0)).collect());
    Box::leak(Box::new(Chunk {
        base,
        starts,
        owners: std::array::from_fn(|_| AtomicU64::new(0)),
    }))
}

/// An object of `w` words, a block of its own, zeroed.
fn large(w: usize) -> *mut u64 {
    let layout = std::alloc::Layout::array::<u64>(w)
        .and_then(|l| l.align_to(PAGE_BYTES))
        .expect("a large object");
    // SAFETY: a layout of nonzero size (`w` past a page's quarter).
    let p = unsafe { std::alloc::alloc_zeroed(layout) }.cast::<u64>();
    if p.is_null() {
        std::alloc::handle_alloc_error(layout)
    }
    LARGE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .push((p.addr(), w));
    p
}

/// This thread's number, never 0.
fn thread_number() -> u64 {
    thread_local!(static N: Cell<u64> = const { Cell::new(0) });
    static NEXT: AtomicU64 = AtomicU64::new(1);
    N.with(|n| {
        if n.get() == 0 {
            n.set(NEXT.fetch_add(1, Ordering::Relaxed));
        }
        n.get()
    })
}

/// The first word of the space's object that `addr` is a word of, if any
/// (an ambiguous root's object, Bartlett's: a word of the stack, perhaps
/// a tagged or interior pointer); past a page's last object, that object,
/// as the page is pinned whole.
#[cfg_attr(not(any(test, feature = "heap-stats")), allow(dead_code))]
pub(crate) fn object_of(addr: usize) -> Option<usize> {
    {
        let pool = POOL.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(c) = pool.chunks.iter().find(|c| c.holds(addr)) {
            return c.start_of(addr);
        }
    }
    let large = LARGE.lock().unwrap_or_else(|e| e.into_inner());
    large
        .iter()
        .find(|&&(p, w)| p <= addr && addr < p + 8 * w)
        .map(|&(p, _)| p)
}

/// What the space holds: its chunks' bytes, the pages given, the large
/// objects' bytes.
#[cfg_attr(not(feature = "heap-stats"), allow(dead_code))]
pub(crate) fn extent() -> (usize, usize, usize) {
    let pool = POOL.lock().unwrap_or_else(|e| e.into_inner());
    let pages = pool.chunks.len().saturating_sub(1) * CHUNK_PAGES
        + if pool.chunks.is_empty() {
            0
        } else {
            pool.next_page
        };
    let large: usize = LARGE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .map(|&(_, w)| 8 * w)
        .sum();
    (pool.chunks.len() * CHUNK_BYTES, pages, large)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_object_found_from_any_of_its_words() {
        let a = words(3);
        let b = words(5);
        let big = words(LARGE_BYTES / 8 + 1);
        for (p, n) in [(a, 3), (b, 5), (big, LARGE_BYTES / 8 + 1)] {
            for k in 0..n {
                // SAFETY: a word of the object.
                let q = unsafe { p.add(k) }.addr();
                assert_eq!(object_of(q), Some(p.addr()));
                // A tagged pointer to it, as a pair's or an object's word.
                assert_eq!(object_of(q | 0b100), Some(p.addr()));
            }
        }
        assert!(object_of(std::ptr::from_ref(&a).addr()).is_none());
    }

    #[test]
    fn pages_one_threads_each() {
        let here = words(1).addr();
        let there = std::thread::spawn(|| words(1).addr()).join().unwrap();
        assert_ne!(here / PAGE_BYTES, there / PAGE_BYTES);
        let mut seen = Vec::new();
        for _ in 0..3 * PAGE_BYTES / 64 {
            seen.push(words(8));
        }
        for p in seen {
            assert_eq!(object_of(p.addr() + 9), Some(p.addr()));
        }
    }
}
