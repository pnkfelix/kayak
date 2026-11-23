//! The heap's space (C507; `TODO.md` §54, a collector's first step): every
//! object of the heap, of Fixpt's layout or a pair, made by bumping in a
//! page of the thread's, as Fixpt's heap is one space of words. Pages are
//! `PAGE_BYTES`, taken a page at a time from chunks the process shares, so
//! that a page is one thread's (a thread's collection is of its pages,
//! `obj::collect`, `TODO.md` §55 item 2's heap per thread). Each chunk
//! keeps a bit per word, set at an object's first word: where an object
//! starts, found from any word inside it (Bartlett's ambiguous roots, an
//! interior pointer), without reading what the heap holds. An object past
//! `LARGE_BYTES` is a block of its own, noted apart.
//!
//! Allocation is the safepoint (C511): where a page is full, or at every
//! allocation under `KAYAK_GC_STRESS`, the thread's pages may be collected
//! (`obj::collect`), and the pages it frees are taken again before a chunk
//! is. A large object is never moved, nor freed yet.

use std::cell::{Cell, RefCell};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// A page's bytes: what a thread takes at once, and what a collector
/// pins whole (Bartlett's).
pub(crate) const PAGE_BYTES: usize = 1 << 15;

/// A chunk's pages: what the process asks of the allocator at once.
const CHUNK_PAGES: usize = 64;
const CHUNK_BYTES: usize = PAGE_BYTES * CHUNK_PAGES;
/// A chunk is aligned to its size, so that a word's chunk is its address
/// shifted (C511).
const CHUNK_SHIFT: u32 = CHUNK_BYTES.trailing_zeros();
const CHUNK_WORDS: usize = CHUNK_BYTES / 8;
const PAGE_WORDS: usize = PAGE_BYTES / 8;

/// Past this, an object is a block of its own: a page holds four at least.
pub(crate) const LARGE_BYTES: usize = PAGE_BYTES / 4;

/// A chunk: its words, and a bit for each, set where an object starts.
pub(crate) struct Chunk {
    base: *mut u64,
    starts: &'static [AtomicU64],
    /// A collection's marks, a bit per word as `starts`: each word of them
    /// within one page, written by its page's thread alone.
    marks: &'static [AtomicU64],
    /// The pages a collection copies from, a bit each, while it runs.
    from: AtomicU64,
    /// For each page, the thread it is given to (its `ThreadId`'s
    /// number), 0 while no thread has it.
    owners: [AtomicU64; CHUNK_PAGES],
}

// SAFETY: a chunk's words are only bumped into, and collected, by the
// thread its page is given to; its base, the address of memory kept for
// the run.
unsafe impl Send for Chunk {}
unsafe impl Sync for Chunk {}

/// The process's chunks, by their bases, and the next page not yet given
/// of the last made; the pages a collection freed, to be given again.
struct Pool {
    chunks: Vec<&'static Chunk>,
    last: Option<&'static Chunk>,
    next_page: usize,
    free: Vec<(&'static Chunk, usize)>,
}

static POOL: Mutex<Pool> = Mutex::new(Pool {
    chunks: Vec::new(),
    last: None,
    next_page: CHUNK_PAGES,
    free: Vec::new(),
});

/// Objects each a block of their own: their first word, words, and the
/// thread that made each.
static LARGE: Mutex<Vec<(usize, usize, u64)>> = Mutex::new(Vec::new());

/// The thread's page: where the next object goes, where an allocation
/// goes to `refill` (the page's end; under stress, where the next goes),
/// the page's end, and its chunk; the pages it has, and how many it took
/// since its last collection.
struct Bump {
    at: Cell<*mut u64>,
    end: Cell<*mut u64>,
    limit: Cell<*mut u64>,
    chunk: Cell<Option<&'static Chunk>>,
    pages: RefCell<Vec<(&'static Chunk, usize)>>,
    since: Cell<usize>,
    /// Pages to take before a collection is due.
    due: Cell<usize>,
    /// While a collection runs, or a caller holds what no root sees
    /// (`NoCollect`), none starts.
    held: Cell<usize>,
    /// Allocations under stress, counted.
    ticks: Cell<u64>,
}

thread_local! {
    static BUMP: Bump = const {
        Bump {
            at: Cell::new(std::ptr::null_mut()),
            end: Cell::new(std::ptr::null_mut()),
            limit: Cell::new(std::ptr::null_mut()),
            chunk: Cell::new(None),
            pages: RefCell::new(Vec::new()),
            since: Cell::new(0),
            due: Cell::new(0),
            held: Cell::new(0),
            ticks: Cell::new(0),
        }
    };
}

/// The words of a new object of `w` words, zeroed, its first noted an
/// object's start. A safepoint: the thread's pages may be collected first.
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
    // SAFETY: `w` zeroed words of the space, new, given to the caller
    // alone, kept while reached (a collection moves them only where no
    // ambiguous root holds them, as this slice is held).
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
    // one, written once here.
    unsafe {
        p.write(v);
        &*p
    }
}

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
    pub(crate) fn start_of(&self, addr: usize) -> Option<usize> {
        let i = (addr - self.base.addr()) / 8;
        let page = i * 8 / PAGE_BYTES;
        if self.owners[page].load(Ordering::Relaxed) == 0 {
            return None;
        }
        let first = page * PAGE_WORDS;
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

    /// The first word of the next object after the one at `addr`, if one
    /// is in its page.
    fn next_start(&self, addr: usize) -> Option<usize> {
        let i = (addr - self.base.addr()) / 8 + 1;
        let end = ((i - 1) / PAGE_WORDS + 1) * PAGE_WORDS;
        let mut k = i;
        while k < end {
            let bits = self.starts[k / 64].load(Ordering::Relaxed) & (u64::MAX << (k % 64));
            if bits != 0 {
                let j = k / 64 * 64 + bits.trailing_zeros() as usize;
                return Some(self.base.addr() + 8 * j);
            }
            k = k / 64 * 64 + 64;
        }
        None
    }

    /// Page `page` given back: its words zeroed or poisoned, its starts
    /// cleared, no thread's.
    fn clear_page(&self, page: usize, poison: bool) {
        let first = page * PAGE_WORDS;
        // SAFETY: the page's words, of this chunk, given to no one else
        // while it is cleared.
        unsafe {
            let p = self.base.add(first);
            if poison {
                // A header of kind 0x7f and absurd counts: what reads it
                // is wrong at once.
                for k in 0..PAGE_WORDS {
                    p.add(k).write(0xdead_dead_dead_3ff6);
                }
            } else {
                p.write_bytes(0, PAGE_WORDS);
            }
        }
        for s in &self.starts[first / 64..(first + PAGE_WORDS) / 64] {
            s.store(0, Ordering::Relaxed);
        }
        self.owners[page].store(0, Ordering::Relaxed);
    }

    /// Whether an object starts at `addr`, a word of this chunk.
    #[inline]
    pub(crate) fn is_start(&self, addr: usize) -> bool {
        let i = (addr - self.base.addr()) / 8;
        self.starts[i / 64].load(Ordering::Relaxed) & 1 << (i % 64) != 0
    }

    /// The object at `addr` marked; whether it was not before.
    #[inline]
    pub(crate) fn mark(&self, addr: usize) -> bool {
        let i = (addr - self.base.addr()) / 8;
        let m = &self.marks[i / 64];
        let was = m.load(Ordering::Relaxed);
        m.store(was | 1 << (i % 64), Ordering::Relaxed);
        was & 1 << (i % 64) == 0
    }

    /// Page `page`'s marks cleared.
    pub(crate) fn clear_marks(&self, page: usize) {
        let first = page * PAGE_WORDS / 64;
        for m in &self.marks[first..first + PAGE_WORDS / 64] {
            m.store(0, Ordering::Relaxed);
        }
    }

    /// Page `page` one a collection copies from, or no more.
    pub(crate) fn set_from(&self, page: usize, from: bool) {
        let bit = 1u64 << page;
        if from {
            self.from.fetch_or(bit, Ordering::Relaxed);
        } else {
            self.from.fetch_and(!bit, Ordering::Relaxed);
        }
    }

    /// Whether `addr` is on a page a collection copies from.
    #[inline]
    pub(crate) fn is_from(&self, addr: usize) -> bool {
        self.from.load(Ordering::Relaxed) & 1 << ((addr - self.base.addr()) / PAGE_BYTES) != 0
    }

    /// Page `page`'s first word.
    fn page_start(&self, page: usize) -> *mut u64 {
        // SAFETY: a page of this chunk.
        unsafe { self.base.add(page * PAGE_WORDS) }
    }
}

/// The thread's page full, or none yet, or under stress: perhaps a
/// collection first; then an object of its own if large, else room in
/// the page, else a new page.
#[cold]
fn refill(b: &Bump, w: usize) -> *mut u64 {
    let stress = stress();
    if b.held.get() == 0 && enabled() {
        let due = if stress > 0 {
            b.ticks.set(b.ticks.get() + 1);
            b.ticks.get().is_multiple_of(stress)
        } else {
            b.since.get() >= b.due.get().max(min_due())
        };
        if due && concurrent() == 0 {
            super::collect::collect();
        }
    }
    if w * 8 > LARGE_BYTES {
        return large(w);
    }
    let at = b.at.get();
    let room = (b.limit.get().addr() - at.addr()) / 8;
    if room < w {
        take_page(b);
    }
    let at = b.at.get();
    // SAFETY: `w` words fit before the page's end.
    b.at.set(unsafe { at.add(w) });
    b.end.set(if stress > 0 {
        b.at.get()
    } else {
        b.limit.get()
    });
    b.chunk.get().expect("a page").note_start(at);
    at
}

/// A page for the thread, freed by a collection or new, now its page.
fn take_page(b: &Bump) {
    let (chunk, page, fresh) = {
        let mut pool = POOL.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((c, p)) = pool.free.pop() {
            (c, p, false)
        } else {
            if pool.next_page == CHUNK_PAGES {
                let c = new_chunk();
                let i = pool
                    .chunks
                    .partition_point(|k| k.base.addr() < c.base.addr());
                pool.chunks.insert(i, c);
                pool.last = Some(c);
                pool.next_page = 0;
            }
            let page = pool.next_page;
            pool.next_page += 1;
            (pool.last.expect("a chunk"), page, true)
        }
    };
    let start = chunk.page_start(page);
    if !fresh {
        // SAFETY: the page's words, now this thread's alone.
        unsafe { start.write_bytes(0, PAGE_WORDS) };
    }
    chunk.owners[page].store(thread_number(), Ordering::Relaxed);
    b.pages.borrow_mut().push((chunk, page));
    b.since.set(b.since.get() + 1);
    b.chunk.set(Some(chunk));
    b.at.set(start);
    // SAFETY: as above; its end, one past its last word.
    b.limit.set(unsafe { start.add(PAGE_WORDS) });
    b.end.set(b.limit.get());
}

/// A chunk, zeroed, kept for the run.
fn new_chunk() -> &'static Chunk {
    let layout = std::alloc::Layout::from_size_align(CHUNK_BYTES, CHUNK_BYTES).expect("a chunk");
    // SAFETY: a layout of nonzero size.
    let base = unsafe { std::alloc::alloc_zeroed(layout) }.cast::<u64>();
    if base.is_null() {
        std::alloc::handle_alloc_error(layout)
    }
    // A collection's copies written by their addresses (`word_mut`).
    let _ = base.expose_provenance();
    let starts: &'static [AtomicU64] =
        Box::leak((0..CHUNK_WORDS / 64).map(|_| AtomicU64::new(0)).collect());
    let marks: &'static [AtomicU64] =
        Box::leak((0..CHUNK_WORDS / 64).map(|_| AtomicU64::new(0)).collect());
    Box::leak(Box::new(Chunk {
        base,
        starts,
        marks,
        from: AtomicU64::new(0),
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
    // Written again by a collection, by its address (`word_mut`).
    let _ = p.expose_provenance();
    let mut large = LARGE.lock().unwrap_or_else(|e| e.into_inner());
    let i = large.partition_point(|&(q, _, _)| q < p.addr());
    large.insert(i, (p.addr(), w, thread_number()));
    p
}

/// This thread's number, never 0.
pub(crate) fn thread_number() -> u64 {
    thread_local!(static N: Number = const { Number(Cell::new(0)) });
    static NEXT: AtomicU64 = AtomicU64::new(1);
    N.try_with(|n| {
        if n.0.get() == 0 {
            n.0.set(NEXT.fetch_add(1, Ordering::Relaxed));
        }
        n.0.get()
    })
    .unwrap_or(u64::MAX)
}

/// The numbers of threads ended: their objects no thread writes, so a
/// collection may follow and write them (C511).
static DEAD: Mutex<Vec<u64>> = Mutex::new(Vec::new());

/// A thread's number, noted dead when its thread ends.
struct Number(Cell<u64>);

impl Drop for Number {
    fn drop(&mut self) {
        if self.0.get() != 0 {
            DEAD.lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(self.0.get());
        }
    }
}

/// The first word of the space's object that `addr` is a word of, if any
/// (an ambiguous root's object, Bartlett's: a word of the stack, perhaps
/// a tagged or interior pointer); past a page's last object, that object,
/// as the page is pinned whole.
pub(crate) fn object_of(addr: usize) -> Option<usize> {
    {
        let pool = POOL.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(c) = chunk_of(&pool.chunks, addr) {
            return c.start_of(addr);
        }
    }
    let large = LARGE.lock().unwrap_or_else(|e| e.into_inner());
    large_of(&large, addr).map(|(p, _, _)| p)
}

fn chunk_of(chunks: &[&'static Chunk], addr: usize) -> Option<&'static Chunk> {
    let i = chunks.partition_point(|c| c.base.addr() <= addr);
    (i > 0 && chunks[i - 1].holds(addr)).then(|| chunks[i - 1])
}

fn large_of(large: &[(usize, usize, u64)], addr: usize) -> Option<(usize, usize, u64)> {
    let i = large.partition_point(|&(p, _, _)| p <= addr);
    (i > 0 && addr < large[i - 1].0 + 8 * large[i - 1].1).then(|| large[i - 1])
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
        }
        - pool.free.len();
    let large: usize = LARGE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .map(|&(_, w, _)| 8 * w)
        .sum();
    (pool.chunks.len() * CHUNK_BYTES, pages, large)
}

// What a collection needs of the space (`obj::collect`).

/// The space as a collection sees it: the chunks and large objects, read
/// once, and the thread's number.
pub(crate) struct View {
    chunks:
        std::collections::HashMap<usize, &'static Chunk, std::hash::BuildHasherDefault<Shifted>>,
    large: Vec<(usize, usize, u64)>,
    me: u64,
    dead: std::collections::HashSet<u64>,
}

/// Whose an object is, as a collection sees it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Whose {
    /// This thread's.
    Mine,
    /// A thread's that has ended: read and written, never moved.
    Dead,
    /// A live thread's: never read (it may be writing it).
    Live,
}

/// A chunk's number (its address shifted), hashed by a multiply.
#[derive(Default)]
pub(crate) struct Shifted(u64);

impl std::hash::Hasher for Shifted {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0.rotate_left(8) ^ u64::from(b)).wrapping_mul(0x9e37_79b9_7f4a_7c15);
        }
    }
    fn write_usize(&mut self, n: usize) {
        self.0 = (n as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
    }
}

impl View {
    pub(crate) fn new() -> View {
        View {
            chunks: POOL
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .chunks
                .iter()
                .map(|&c| (c.base.addr() >> CHUNK_SHIFT, c))
                .collect(),
            large: LARGE.lock().unwrap_or_else(|e| e.into_inner()).clone(),
            me: thread_number(),
            dead: DEAD
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .iter()
                .copied()
                .collect(),
        }
    }

    fn whose(&self, owner: u64) -> Whose {
        if owner == self.me {
            Whose::Mine
        } else if self.dead.contains(&owner) {
            Whose::Dead
        } else {
            Whose::Live
        }
    }

    #[inline]
    fn chunk(&self, addr: usize) -> Option<&'static Chunk> {
        self.chunks.get(&(addr >> CHUNK_SHIFT)).copied()
    }

    /// The chunk `addr` is a word of, and its page.
    #[inline]
    pub(crate) fn page_of(&self, addr: usize) -> Option<(&'static Chunk, usize)> {
        self.chunk(addr)
            .map(|c| (c, (addr - c.base.addr()) / PAGE_BYTES))
    }

    /// Whose the page is.
    #[inline]
    pub(crate) fn whose_page(&self, c: &Chunk, page: usize) -> Whose {
        self.whose(c.owners[page].load(Ordering::Relaxed))
    }

    /// The large object `addr` is a word of: its first word, its words,
    /// whose.
    #[inline]
    pub(crate) fn large_owner(&self, addr: usize) -> Option<(usize, usize, Whose)> {
        large_of(&self.large, addr).map(|(p, w, o)| (p, w, self.whose(o)))
    }

    /// `object_of`, against this view.
    #[inline]
    pub(crate) fn object_of(&self, addr: usize) -> Option<usize> {
        if let Some(c) = self.chunk(addr) {
            return c.start_of(addr);
        }
        large_of(&self.large, addr).map(|(p, _, _)| p)
    }

    /// Whose the object at `start` (an object's first word) is.
    pub(crate) fn whose_of(&self, start: usize) -> Whose {
        if let Some((c, p)) = self.page_of(start) {
            return self.whose_page(c, p);
        }
        let (_, _, owner) = large_of(&self.large, start).expect("an object of the space");
        self.whose(owner)
    }

    /// The words of the allocation at `start` (its first word, in a
    /// page): to the next allocation's first, so that nothing between is
    /// left behind; the last of its page, to its last object's `end`.
    pub(crate) fn extent_of(&self, start: usize, end: usize) -> usize {
        let c = self.chunk(start).expect("an object in a page");
        let to = c.next_start(start).unwrap_or(end);
        debug_assert!(to >= end, "an allocation past the next");
        (to - start) / 8
    }
}

/// A page's identity: its chunk's base and its number.
pub(crate) fn page_key(c: &'static Chunk, page: usize) -> (usize, usize) {
    (c.base.addr(), page)
}

/// The thread's pages, from its first.
pub(crate) fn my_pages() -> Vec<(&'static Chunk, usize)> {
    BUMP.with(|b| b.pages.borrow().clone())
}

/// A collection's start: no page bumped into until the next allocation
/// takes one, and none started while it runs.
pub(crate) fn begin_collection() {
    BUMP.with(|b| {
        b.held.set(b.held.get() + 1);
        b.at.set(std::ptr::null_mut());
        b.end.set(std::ptr::null_mut());
        b.limit.set(std::ptr::null_mut());
        b.chunk.set(None);
    });
}

/// A collection's end: the pages it freed given back (each zeroed when
/// taken again, poisoned now under stress), the thread's the rest, and the
/// next due when as many pages again are taken.
pub(crate) fn end_collection(freed: &[(&'static Chunk, usize)]) {
    let poison = stress() > 0 || poison();
    let keys: std::collections::HashSet<(usize, usize)> =
        freed.iter().map(|&(c, p)| page_key(c, p)).collect();
    for &(c, p) in freed {
        c.clear_page(p, poison);
    }
    POOL.lock()
        .unwrap_or_else(|e| e.into_inner())
        .free
        .extend_from_slice(freed);
    BUMP.with(|b| {
        let mut pages = b.pages.borrow_mut();
        pages.retain(|&(c, p)| !keys.contains(&page_key(c, p)));
        // The heap may grow to three times what was kept before the next.
        b.due.set(2 * pages.len());
        b.since.set(0);
        b.held.set(b.held.get() - 1);
    });
}

/// The word at `addr` of the space, to write (a collection's).
///
/// # Safety
/// `addr` is a word of an object of the space, reached, of this thread's
/// or of a thread that runs no Rust while this one collects.
pub(crate) unsafe fn word_mut(view: &View, addr: usize) -> *mut u64 {
    if let Some(c) = view.chunk(addr) {
        return c.base.with_addr(addr);
    }
    std::ptr::with_exposed_provenance_mut(addr)
}

/// While this lives, no collection starts on this thread: kayak-rt's code
/// that holds what no root sees (values in a `Vec` of its own) across an
/// allocation.
pub(crate) struct NoCollect(());

impl NoCollect {
    pub(crate) fn new() -> NoCollect {
        BUMP.with(|b| b.held.set(b.held.get() + 1));
        NoCollect(())
    }
}

impl Drop for NoCollect {
    fn drop(&mut self) {
        // While the thread ends, nothing collects anyway.
        let _ = BUMP.try_with(|b| b.held.set(b.held.get() - 1));
    }
}

/// Threads kayak-rt runs beside the program's (a process's, a `par`
/// branch's) while they live: one may hold another's words where no
/// thread's collection sees, so none collects (`TODO.md` §54, step 4's
/// cross-thread work).
static CONCURRENT: AtomicUsize = AtomicUsize::new(0);

fn concurrent() -> usize {
    CONCURRENT.load(Ordering::Acquire)
}

/// A thread run beside the program's, while this lives.
pub(crate) struct Concurrent(());

impl Concurrent {
    pub(crate) fn new() -> Concurrent {
        CONCURRENT.fetch_add(1, Ordering::AcqRel);
        Concurrent(())
    }
}

impl Drop for Concurrent {
    fn drop(&mut self) {
        CONCURRENT.fetch_sub(1, Ordering::AcqRel);
    }
}

fn env_number(name: &str) -> Option<u64> {
    std::env::var(name).ok().and_then(|s| s.parse().ok())
}

/// Whether collections run: `KAYAK_GC=0` says not. Not in a census's
/// build (`heap-stats`), whose census keeps objects' addresses, unless
/// `KAYAK_GC=1` says so.
fn enabled() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| match env_number("KAYAK_GC") {
        Some(n) => n != 0,
        None => !cfg!(feature = "heap-stats"),
    })
}

/// `KAYAK_GC_STRESS=n`: a collection at every `n`th allocation; 0, none.
fn stress() -> u64 {
    static N: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
    *N.get_or_init(|| env_number("KAYAK_GC_STRESS").unwrap_or(0))
}

/// Whether collections run under stress.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn stressed() -> bool {
    stress() > 0
}

/// `KAYAK_GC_POISON=1`: freed pages poisoned, as under stress.
fn poison() -> bool {
    static P: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *P.get_or_init(|| env_number("KAYAK_GC_POISON").is_some_and(|n| n != 0))
}

/// Pages taken before a first collection, and at least between two:
/// `KAYAK_GC_PAGES`, else 64 MB's.
fn min_due() -> usize {
    static N: std::sync::OnceLock<usize> = std::sync::OnceLock::new();
    *N.get_or_init(|| env_number("KAYAK_GC_PAGES").map_or((64 << 20) / PAGE_BYTES, |n| n as usize))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_object_found_from_any_of_its_words() {
        let _held = NoCollect::new();
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
        let _held = NoCollect::new();
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
