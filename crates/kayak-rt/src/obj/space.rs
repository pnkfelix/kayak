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
//! is. A large object is never moved; one no collection of its thread's
//! reached is freed (C513), once every collection running as it was found
//! so has ended (`Epochs`: one may hold it in its view, from a stale
//! word). A thread that ends gives its pages and large objects to its
//! heir (a `par` branch's maker, C512), or else to whichever thread
//! collects next (`ORPHANS`), which copies from them once every collection
//! that may have seen them an ended thread's, to write, has ended.

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
    /// The pages taken since their thread's last collection, a bit each
    /// (young: TODO §54 step 4's generations). Every other page given is
    /// old: what its last collection kept, copied or pinned.
    young: AtomicU64,
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
    /// The thread's number, noted as it takes its first page or large
    /// object (its number's thread local may be gone as the thread ends).
    me: Cell<u64>,
    /// Pages the thread's large objects would fill, kept at its last
    /// collection or made or adopted since: counted toward the next.
    large_pages: Cell<usize>,
    /// The number of the collection running on this thread
    /// (`Epochs`).
    epoch: Cell<u64>,
    /// Pages adopted (an ended thread's) that no collection copies from
    /// until every collection running when they were adopted has ended:
    /// one of those may have seen them an ended thread's, to write.
    probation: RefCell<Vec<((usize, usize), u64)>>,
    /// The write barrier's store buffer (`remember`): the address of each
    /// slot outside the thread's current page a heap pointer was stored
    /// into since its last collection, perhaps an old object's that now
    /// points to a young one. Kept by the thread alone, so no lock.
    remembered: RefCell<Vec<usize>>,
    /// The thread's large objects made since its last collection, young
    /// (their first words): what a store as one is made puts in them
    /// passes no barrier.
    young_large: RefCell<Vec<usize>>,
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
            me: Cell::new(0),
            large_pages: Cell::new(0),
            epoch: Cell::new(0),
            probation: RefCell::new(Vec::new()),
            remembered: RefCell::new(Vec::new()),
            young_large: RefCell::new(Vec::new()),
        }
    };
    /// Touched as the thread takes its first page or large object, so that
    /// as it ends what it has is given to whichever thread collects next.
    static ENDER: Ender = const { Ender };
}

/// As its thread ends: its pages and large objects orphans (`ORPHANS`).
struct Ender;

impl Drop for Ender {
    fn drop(&mut self) {
        let p = hand_over();
        if !p.pages.is_empty() || p.large > 0 {
            ORPHANS.lock().unwrap_or_else(|e| e.into_inner()).push(p);
        }
    }
}

/// The pages and large objects of threads ended with no heir (a
/// process's), until a collection adopts them (`adopt_orphans`).
static ORPHANS: Mutex<Vec<Parcel>> = Mutex::new(Vec::new());

/// The thread's number, noted in its `Bump`, and its end awaited.
fn note_me(b: &Bump) {
    if b.me.get() == 0 {
        b.me.set(thread_number());
        let _ = ENDER.try_with(|_| ());
    }
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
        self.set_young(page, false);
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

    /// Each allocation's first word on page `page`, in order.
    pub(crate) fn each_start(&self, page: usize, mut f: impl FnMut(usize)) {
        let first = page * PAGE_WORDS;
        for k in first / 64..(first + PAGE_WORDS) / 64 {
            let mut bits = self.starts[k].load(Ordering::Relaxed);
            while bits != 0 {
                let j = k * 64 + bits.trailing_zeros() as usize;
                f(self.base.addr() + 8 * j);
                bits &= bits - 1;
            }
        }
    }

    /// Whether the allocation at `addr` (its first word) is marked.
    #[inline]
    pub(crate) fn is_marked(&self, addr: usize) -> bool {
        let i = (addr - self.base.addr()) / 8;
        self.marks[i / 64].load(Ordering::Relaxed) & 1 << (i % 64) != 0
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

    /// Page `page` young (taken since its thread's last collection), or
    /// old.
    fn set_young(&self, page: usize, young: bool) {
        let bit = 1u64 << page;
        if young {
            self.young.fetch_or(bit, Ordering::Relaxed);
        } else {
            self.young.fetch_and(!bit, Ordering::Relaxed);
        }
    }

    /// Whether page `page` is young.
    #[inline]
    pub(crate) fn is_young(&self, page: usize) -> bool {
        self.young.load(Ordering::Relaxed) & 1 << page != 0
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
        return large(b, w);
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
    note_me(b);
    chunk.owners[page].store(b.me.get(), Ordering::Relaxed);
    chunk.set_young(page, true);
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
        young: AtomicU64::new(0),
        owners: std::array::from_fn(|_| AtomicU64::new(0)),
    }))
}

/// The pages `w` words would fill.
fn pages_of(w: usize) -> usize {
    (8 * w).div_ceil(PAGE_BYTES)
}

fn large_layout(w: usize) -> std::alloc::Layout {
    std::alloc::Layout::array::<u64>(w)
        .and_then(|l| l.align_to(PAGE_BYTES))
        .expect("a large object")
}

/// An object of `w` words, a block of its own, zeroed: counted as the
/// pages it would fill, toward the next collection.
fn large(b: &Bump, w: usize) -> *mut u64 {
    let layout = large_layout(w);
    // SAFETY: a layout of nonzero size (`w` past a page's quarter).
    let p = unsafe { std::alloc::alloc_zeroed(layout) }.cast::<u64>();
    if p.is_null() {
        std::alloc::handle_alloc_error(layout)
    }
    // Written again by a collection, by its address (`word_mut`).
    let _ = p.expose_provenance();
    note_me(b);
    b.since.set(b.since.get() + pages_of(w));
    b.large_pages.set(b.large_pages.get() + pages_of(w));
    let mut large = LARGE.lock().unwrap_or_else(|e| e.into_inner());
    let i = large.partition_point(|&(q, _, _)| q < p.addr());
    large.insert(i, (p.addr(), w, b.me.get()));
    b.young_large.borrow_mut().push(p.addr());
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
    /// A thread's that has ended: read and written, never moved, till a
    /// thread adopts it (`adopt`, `adopt_orphans`), its own then.
    Dead,
    /// A live thread's: never written, and read only where it crossed to
    /// this thread, which no thread writes (C512).
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

/// The write barrier (TODO §54 step 4's generations): the slot at
/// `slot`, a word just stored, noted if it now holds a heap pointer (a
/// pair's or an object's tag) and is not in the thread's current page,
/// where only young objects are. What is noted is checked when used, so a
/// slot of a stack, an arena or a kept block noted too costs a word.
#[inline]
pub(crate) fn remember<T: Copy>(cell: &Cell<T>) {
    const {
        assert!(
            std::mem::size_of::<T>() == 8,
            "a slot of one word (a Fixpt word)"
        )
    };
    let slot = cell.as_ptr().addr();
    // SAFETY: the cell's one word, a `T` of 8 bytes, read as its bits.
    let w = unsafe { cell.as_ptr().cast::<u64>().read() };
    let tag = w & 0b111;
    if tag != 0b001 && tag != 0b100 {
        return;
    }
    BUMP.with(|b| {
        let limit = b.limit.get().addr();
        if limit != 0 && slot < limit && slot >= limit - PAGE_BYTES {
            return;
        }
        b.remembered.borrow_mut().push(slot);
    });
}

/// The number of the thread's last collection to begin (`Epochs`): one
/// that differs from one read before says a collection ran since.
#[inline]
pub(crate) fn epoch() -> u64 {
    BUMP.with(|b| b.epoch.get())
}

/// How many slots the thread's store buffer holds.
pub(crate) fn remembered_len() -> usize {
    BUMP.with(|b| b.remembered.borrow().len())
}

/// The thread's store buffer, a copy (the verifier's).
pub(crate) fn remembered() -> Vec<usize> {
    BUMP.with(|b| b.remembered.borrow().clone())
}

/// The thread's young large objects (their first words).
pub(crate) fn young_large() -> Vec<usize> {
    BUMP.with(|b| b.young_large.borrow().clone())
}

/// `KAYAK_GC_VERIFY=1`, or under stress: each collection checks that
/// every old object of the thread's it reaches that points to a young one
/// does so from a slot the barrier noted (Fixpt's verifier,
/// `generational-gc.md`).
pub(crate) fn verifying() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| match env_number("KAYAK_GC_VERIFY") {
        Some(n) => n != 0,
        None => stress() > 0,
    })
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
    let n = Epochs::begin();
    BUMP.with(|b| {
        b.epoch.set(n);
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
        // What it kept, old now: no young object is left for an old one to
        // point to, so the store buffer is emptied; the page its copies
        // went to left, so that what is made next goes to a young one.
        for &(c, p) in pages.iter() {
            c.set_young(p, false);
        }
        b.remembered.borrow_mut().clear();
        b.young_large.borrow_mut().clear();
        b.at.set(std::ptr::null_mut());
        b.end.set(std::ptr::null_mut());
        b.limit.set(std::ptr::null_mut());
        b.chunk.set(None);
        // The heap may grow to three times what was kept before the next.
        b.due.set(2 * (pages.len() + b.large_pages.get()));
        b.since.set(0);
        b.held.set(b.held.get() - 1);
        Epochs::end(b.epoch.get());
    });
    free_retired();
}

/// A thread's pages and large objects, given to another as the thread
/// ends: a `par` branch's to its maker at the join (C512), what the
/// branch's value holds, the maker's to collect; any other's to whichever
/// thread collects next (`ORPHANS`).
pub(crate) struct Parcel {
    pages: Vec<(&'static Chunk, usize)>,
    from: u64,
    /// The pages its large objects would fill.
    large: usize,
    /// Its store buffer and young large objects, its pages' ages kept.
    remembered: Vec<usize>,
    young_large: Vec<usize>,
}

/// This thread's pages, given up as it ends: none bumped into again (a
/// page taken after is its own, an ended thread's). Still its, as other
/// threads see them, until `adopt`.
pub(crate) fn hand_over() -> Parcel {
    BUMP.with(|b| {
        b.at.set(std::ptr::null_mut());
        b.end.set(std::ptr::null_mut());
        b.limit.set(std::ptr::null_mut());
        b.chunk.set(None);
        b.since.set(0);
        b.probation.borrow_mut().clear();
        Parcel {
            pages: std::mem::take(&mut *b.pages.borrow_mut()),
            from: b.me.get(),
            large: b.large_pages.replace(0),
            remembered: std::mem::take(&mut *b.remembered.borrow_mut()),
            young_large: std::mem::take(&mut *b.young_large.borrow_mut()),
        }
    })
}

/// `p`'s pages and large objects, this thread's now, collected with its
/// own; counted as taken, so that a collection comes as soon as if it had
/// made them. The thread that gave them has ended, so no collection of
/// its runs; one running on another thread may have seen them an ended
/// thread's, to write where they point to what it moves, so none of this
/// thread's copies from them until every one running now has ended
/// (`unsettled`).
pub(crate) fn adopt(p: Parcel) {
    if p.from == 0 {
        return;
    }
    let me = BUMP.with(|b| {
        note_me(b);
        b.me.get()
    });
    for &(c, page) in &p.pages {
        c.owners[page].store(me, Ordering::Relaxed);
    }
    for l in LARGE.lock().unwrap_or_else(|e| e.into_inner()).iter_mut() {
        if l.2 == p.from {
            l.2 = me;
        }
    }
    let stamp = Epochs::lock().next;
    BUMP.with(|b| {
        b.since.set(b.since.get() + p.pages.len() + p.large);
        b.large_pages.set(b.large_pages.get() + p.large);
        b.probation
            .borrow_mut()
            .extend(p.pages.iter().map(|&(c, page)| (page_key(c, page), stamp)));
        b.pages.borrow_mut().extend(p.pages);
        b.remembered.borrow_mut().extend(p.remembered);
        b.young_large.borrow_mut().extend(p.young_large);
    });
}

/// The orphans whose threads have ended, this thread's (a collection's
/// first act).
pub(crate) fn adopt_orphans() {
    let ready: Vec<Parcel> = {
        let mut o = ORPHANS.lock().unwrap_or_else(|e| e.into_inner());
        if o.is_empty() {
            return;
        }
        let dead = DEAD.lock().unwrap_or_else(|e| e.into_inner());
        let (ready, wait) = std::mem::take(&mut *o)
            .into_iter()
            .partition(|p| dead.contains(&p.from));
        *o = wait;
        ready
    };
    for p in ready {
        adopt(p);
    }
}

/// The thread's adopted pages a collection may not copy from yet: some
/// collection running when they were adopted runs still.
pub(crate) fn unsettled() -> std::collections::HashSet<(usize, usize)> {
    BUMP.with(|b| {
        let mut p = b.probation.borrow_mut();
        if p.is_empty() {
            return std::collections::HashSet::new();
        }
        let e = Epochs::lock();
        p.retain(|&(_, stamp)| !e.quiesced(stamp, b.epoch.get()));
        p.iter().map(|&(k, _)| k).collect()
    })
}

/// The collections running, each by its number, given as it begins: what
/// another thread frees or adopts as they run, one of them may have seen
/// as it was (in its `View`), so it is freed, or copied from, once every
/// one that began before has ended.
struct Epochs {
    next: u64,
    running: Vec<u64>,
}

static EPOCHS: Mutex<Epochs> = Mutex::new(Epochs {
    next: 1,
    running: Vec::new(),
});

impl Epochs {
    fn lock() -> std::sync::MutexGuard<'static, Epochs> {
        EPOCHS.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn begin() -> u64 {
        let mut e = Epochs::lock();
        let n = e.next;
        e.next += 1;
        e.running.push(n);
        n
    }

    fn end(n: u64) {
        let mut e = Epochs::lock();
        if let Some(i) = e.running.iter().position(|&k| k == n) {
            e.running.swap_remove(i);
        }
    }

    /// Whether every collection running but `except` began at `stamp` or
    /// after (a stamp is `next` when taken).
    fn quiesced(&self, stamp: u64, except: u64) -> bool {
        self.running.iter().all(|&k| k == except || k >= stamp)
    }
}

/// A stamp: what is retired now is freed once `quiesced` says.
pub(crate) fn stamp() -> u64 {
    Epochs::lock().next
}

/// Whether every collection running but this thread's began at `stamp`
/// or after.
pub(crate) fn quiesced(stamp: u64) -> bool {
    let me = BUMP.with(|b| if b.held.get() > 0 { b.epoch.get() } else { 0 });
    Epochs::lock().quiesced(stamp, me)
}

/// Whether freed memory is poisoned (under stress, or
/// `KAYAK_GC_POISON`).
pub(crate) fn poisoning() -> bool {
    stress() > 0 || poison()
}

/// Large objects no collection reached, out of `LARGE`, each with the
/// stamp of when: freed once every collection that began before has ended.
static RETIRED: Mutex<Vec<(usize, usize, u64)>> = Mutex::new(Vec::new());

/// This thread's large objects a collection did not reach (`reached`
/// says), retired: no view taken from now has them. A collection running
/// on another thread may have one in its view, and read it from a stale
/// word, so it is freed once every such collection has ended
/// (`free_retired`).
pub(crate) fn free_large(reached: impl Fn(usize) -> bool) {
    BUMP.with(|b| {
        let me = b.me.get();
        if me == 0 {
            return;
        }
        let mut dead = Vec::new();
        let mut kept = 0;
        LARGE
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .retain(|&(p, w, o)| {
                if o != me {
                    true
                } else if reached(p) {
                    kept += pages_of(w);
                    true
                } else {
                    dead.push((p, w));
                    false
                }
            });
        b.large_pages.set(kept);
        if !dead.is_empty() {
            let stamp = Epochs::lock().next;
            RETIRED
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .extend(dead.into_iter().map(|(p, w)| (p, w, stamp)));
        }
    });
}

/// The large objects retired before every collection running now began,
/// freed (poisoned first, under stress or `KAYAK_GC_POISON`).
fn free_retired() {
    let ready: Vec<(usize, usize, u64)> = {
        let mut r = RETIRED.lock().unwrap_or_else(|e| e.into_inner());
        if r.is_empty() {
            return;
        }
        let e = Epochs::lock();
        let (ready, wait) = std::mem::take(&mut *r)
            .into_iter()
            .partition(|&(_, _, stamp)| e.quiesced(stamp, 0));
        *r = wait;
        ready
    };
    let poison = stress() > 0 || poison();
    for (p, w, _) in ready {
        let at = std::ptr::with_exposed_provenance_mut::<u64>(p);
        // SAFETY: a large object `large` made, of `w` words, which no
        // thread reaches (no collection did) and no collection's view
        // has; freed once, as it left `LARGE` once.
        unsafe {
            if poison {
                for k in 0..w {
                    at.add(k).write(0xdead_dead_dead_3ff6);
                }
            }
            std::alloc::dealloc(at.cast(), large_layout(w));
        }
    }
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

/// `par` branches that share what may be written (`par2_shared`, the
/// `unsafe-par` feature) while they run: one writes into the other's
/// objects, where no collection of the writer's sees, so none collects
/// (`TODO.md` §54, step 4). A process or a plain `par` branch shares only
/// what is never written, and collects beside the rest (C512).
#[cfg_attr(not(feature = "unsafe-par"), allow(dead_code))]
static CONCURRENT: AtomicUsize = AtomicUsize::new(0);

fn concurrent() -> usize {
    CONCURRENT.load(Ordering::Acquire)
}

/// A thread run beside the program's, while this lives.
#[cfg_attr(not(feature = "unsafe-par"), allow(dead_code))]
pub(crate) struct Concurrent(());

#[cfg_attr(not(feature = "unsafe-par"), allow(dead_code))]
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

/// The bytes of this thread's large objects (a test's).
#[cfg(test)]
pub(crate) fn my_large_bytes() -> usize {
    let me = BUMP.with(|b| b.me.get());
    LARGE
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .iter()
        .filter(|l| l.2 == me)
        .map(|l| 8 * l.1)
        .sum()
}

/// The thread a page is given to, by a word of it; 0, none (a test's).
#[cfg(test)]
pub(crate) fn page_owner(addr: usize) -> u64 {
    let pool = POOL.lock().unwrap_or_else(|e| e.into_inner());
    let c = chunk_of(&pool.chunks, addr).expect("a word of a page");
    c.owners[(addr - c.base.addr()) / PAGE_BYTES].load(Ordering::Relaxed)
}

/// The thread's pages, a word of each (a test's).
#[cfg(test)]
pub(crate) fn my_page_words() -> Vec<usize> {
    my_pages()
        .iter()
        .map(|&(c, p)| c.page_start(p).addr())
        .collect()
}

/// This thread's number, as its pages say (a test's).
#[cfg(test)]
pub(crate) fn my_number() -> u64 {
    BUMP.with(|b| b.me.get())
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
