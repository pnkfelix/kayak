//! The heap's roots (`TODO.md` §54, a collector's third step): every word
//! outside the space that may be a word into it, as Bartlett's
//! mostly-copying collector has them, ambiguous, each a word that may be a
//! pointer, which pins what it may point to. They are:
//! - the thread's stack, from where it is now to its base (the OS's), and
//!   the registers a caller may keep a value in, spilled;
//! - the constants (`once.rs`), each slot read again;
//! - blocks a thread owns while they live, outside the space: a region's
//!   arena's (`arena.rs`), a future's box (`pinned`, C493's, which Rust
//!   lays out as it does a stack);
//! - what kayak-rt keeps for the run of its own (a mailbox, an I-cell, a
//!   continuation, a module's state, a mark's value): each scanned when a
//!   word reaches it, as an object is traced, by its bytes or by how its
//!   type says (a mailbox its messages);
//! - what a thread keeps in its thread locals (the marks), told by a
//!   function of its own.
//!
//! A thread's roots are its own: its stack, the blocks it registered, its
//! thread locals; a constant's slot read as it reads it. What kayak-rt
//! keeps of its own is the process's, found from any thread's word.
//! `reached` is a mark by them and by the space's bits and headers alone
//! (`laid`): a census's now, a collector's to come.

use std::cell::{Cell, UnsafeCell};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};

/// A thread's registry: the blocks it owns, what it keeps of kayak-rt's
/// own, its thread locals' roots. Locked by its thread, and by another
/// only to drop a block it moved there (a future polled on another
/// thread) or to find what is kept: uncontended, as no lock all threads
/// share would be.
pub(crate) struct Registry {
    inner: Mutex<Inner>,
    /// The blocks owned, its thread's alone: no lock to own a future's
    /// box, made and dropped at every boxed call (C510).
    slab: UnsafeCell<Slab>,
    /// Slots another thread gave back (`Inner::freed`) since its thread
    /// last looked.
    remote: AtomicBool,
}

// SAFETY: `slab` is touched by the registry's thread alone (`with_slab`,
// `Owned::drop`); the rest is behind the lock or atomic.
unsafe impl Sync for Registry {}

/// Blocks owned while they live (`Owned`): each its first byte and its
/// bytes. `life` counts the threads that have had the registry, so that a
/// block owned by one gone gives back nothing of the next's.
#[derive(Default)]
struct Slab {
    owned: Vec<Option<(usize, usize)>>,
    free: Vec<usize>,
    life: u64,
}

impl Slab {
    fn give_back(&mut self, slot: usize) {
        self.owned[slot] = None;
        self.free.push(slot);
    }
}

#[derive(Default)]
struct Inner {
    /// Slots of blocks dropped on another thread, with their lives: given
    /// back by the registry's own thread.
    freed: Vec<(usize, u64)>,
    /// What is kept for the run: its first byte, bytes, how it is scanned.
    kept: Vec<Kept>,
    /// Functions telling the thread's thread locals' words.
    locals: Vec<Tell>,
}

/// A function telling words.
type Tell = fn(&mut dyn FnMut(usize));

#[derive(Clone, Copy)]
struct Kept {
    start: usize,
    len: usize,
    scan: Scan,
}

/// How a kept object is scanned: its bytes, or by its type.
type Scan = Option<unsafe fn(usize, &mut dyn FnMut(usize))>;

impl Registry {
    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }
}

/// Every thread's registry, ever made; one given back at its thread's end
/// taken again by a new thread (what it keeps stays, as kept for the run).
static REGISTRIES: Mutex<Vec<&'static Registry>> = Mutex::new(Vec::new());
static FREE: Mutex<Vec<&'static Registry>> = Mutex::new(Vec::new());

/// The thread's registry, given back when the thread ends.
struct Mine(Cell<Option<&'static Registry>>);

impl Drop for Mine {
    fn drop(&mut self) {
        if let Some(r) = self.0.take() {
            {
                // SAFETY: this thread's registry, ending; its slab touched
                // by no one else until `FREE` gives it to another.
                let s = unsafe { &mut *r.slab.get() };
                s.owned.clear();
                s.free.clear();
                s.life += 1;
                let mut i = r.lock();
                i.freed.clear();
                i.locals.clear();
            }
            FREE.lock().unwrap_or_else(|e| e.into_inner()).push(r);
        }
    }
}

thread_local! {
    static MINE: Mine = const { Mine(Cell::new(None)) };
}

/// This thread's registry. `None` while the thread ends (its thread
/// locals' destructors running), when nothing is registered.
#[inline]
fn mine() -> Option<&'static Registry> {
    match MINE.try_with(|m| m.0.get()) {
        Ok(Some(r)) => Some(r),
        Ok(None) => first_mine(),
        Err(_) => None,
    }
}

/// This thread's registry, its first: one given back, or a new one.
#[cold]
#[inline(never)]
fn first_mine() -> Option<&'static Registry> {
    MINE.try_with(|m| {
        m.0.get().unwrap_or_else(|| {
            let r = FREE
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .pop()
                .unwrap_or_else(|| {
                    let r: &'static Registry = Box::leak(Box::new(Registry {
                        inner: Mutex::new(Inner::default()),
                        slab: UnsafeCell::new(Slab::default()),
                        remote: AtomicBool::new(false),
                    }));
                    REGISTRIES.lock().unwrap_or_else(|e| e.into_inner()).push(r);
                    r
                });
            m.0.set(Some(r));
            r
        })
    })
    .ok()
}

/// A block owned while this lives, outside the space, whose words are
/// roots: a region's arena's, a future's box.
pub(crate) struct Owned {
    registry: &'static Registry,
    slot: usize,
    life: u64,
}

impl Drop for Owned {
    #[inline]
    fn drop(&mut self) {
        if mine().is_some_and(|m| std::ptr::eq(m, self.registry)) {
            // SAFETY: this thread's registry, so its slab this thread's
            // alone; nothing here calls back into it.
            let s = unsafe { &mut *self.registry.slab.get() };
            // A block of a thread gone, the registry's since, is not in
            // its slab.
            if s.life == self.life {
                s.give_back(self.slot);
            }
            return;
        }
        self.dropped_elsewhere();
    }
}

impl Owned {
    /// Dropped on another thread (a task moved there), or while this
    /// thread ends: its slot given back by its own thread, at its next
    /// block or mark.
    #[cold]
    #[inline(never)]
    fn dropped_elsewhere(&self) {
        self.registry.lock().freed.push((self.slot, self.life));
        self.registry.remote.store(true, Ordering::Release);
    }
}

/// `f` of this thread's registry and its slab, with the slots another
/// thread gave back given back.
#[inline]
fn with_slab<R>(f: impl FnOnce(&'static Registry, &mut Slab) -> R) -> Option<R> {
    let r = mine()?;
    // SAFETY: `r` is this thread's (`MINE`'s), so its slab this thread's
    // alone, and `f` does not reach it again.
    let s = unsafe { &mut *r.slab.get() };
    if r.remote.load(Ordering::Acquire) {
        r.remote.store(false, Ordering::Relaxed);
        let freed = std::mem::take(&mut r.lock().freed);
        for (slot, life) in freed {
            if life == s.life {
                s.give_back(slot);
            }
        }
    }
    Some(f(r, s))
}

/// The `len` bytes from `start`, roots while the answer lives.
#[inline]
pub(crate) fn owned(start: usize, len: usize) -> Option<Owned> {
    with_slab(|registry, s| {
        let slot = match s.free.pop() {
            Some(slot) => {
                s.owned[slot] = Some((start, len));
                slot
            }
            None => {
                s.owned.push(Some((start, len)));
                s.owned.len() - 1
            }
        };
        Owned {
            registry,
            slot,
            life: s.life,
        }
    })
}

/// `v`'s bytes, roots while the answer lives (a closure's, boxed).
pub(crate) fn owned_val<T: ?Sized>(v: &T) -> Option<Owned> {
    owned(
        std::ptr::from_ref(v).cast::<()>().addr(),
        std::mem::size_of_val(v),
    )
}

/// What is kept of kayak-rt's own, scanned by its type: its words told.
pub(crate) trait Scanned {
    fn scanned(&self, f: &mut dyn FnMut(usize));
}

/// The `len` bytes from `start`, kept for the run: scanned, word by word,
/// when a word reaches them.
pub(crate) fn kept(start: usize, len: usize) {
    keep(start, len, None);
}

/// `r`, kept for the run: scanned as its type says when a word reaches it.
pub(crate) fn kept_scanned<T: Scanned + 'static>(r: &'static T) {
    /// # Safety
    /// `at` is a `T`'s address, kept for the run.
    unsafe fn scan<T: Scanned>(at: usize, f: &mut dyn FnMut(usize)) {
        // SAFETY: the caller's.
        unsafe { &*(at as *const T) }.scanned(f);
    }
    keep(
        std::ptr::from_ref(r).addr(),
        std::mem::size_of::<T>(),
        Some(scan::<T>),
    );
}

fn keep(start: usize, len: usize, scan: Scan) {
    if len == 0 {
        return;
    }
    if let Some(r) = mine() {
        r.lock().kept.push(Kept { start, len, scan });
    }
}

/// A function telling words the thread's thread locals hold (once for
/// each function, however often asked).
pub(crate) fn thread_locals(f: Tell) {
    if let Some(r) = mine() {
        let mut i = r.lock();
        if !i.locals.contains(&f) {
            i.locals.push(f);
        }
    }
}

/// The words of `v`, each that may be a word into the space told: every
/// aligned word of its bytes. Nothing of a value aligned to less than a
/// word, which holds none.
pub(crate) fn words_of<T: Copy>(v: &T, f: &mut dyn FnMut(usize)) {
    if std::mem::align_of::<T>() >= 8 {
        // SAFETY: `v`'s bytes, a whole number of aligned words.
        unsafe { scan_words(std::ptr::from_ref(v).addr(), std::mem::size_of::<T>(), f) };
    }
}

/// The words of `v`, of a size known only as it runs (a closure's), as
/// `words_of` tells them.
pub(crate) fn words_of_val<T: ?Sized>(v: &T, f: &mut dyn FnMut(usize)) {
    if std::mem::align_of_val(v) >= 8 {
        // SAFETY: `v`'s bytes, a whole number of aligned words.
        unsafe {
            scan_words(
                std::ptr::from_ref(v).cast::<()>().addr(),
                std::mem::size_of_val(v),
                f,
            )
        };
    }
}

/// The aligned words of the `len` bytes from `start`, told.
///
/// # Safety
/// The bytes are readable for the scan. A word of padding may be told: an
/// ambiguous root's, which a collector only pins by.
unsafe fn scan_words(start: usize, len: usize, f: &mut dyn FnMut(usize)) {
    let mut at = (start + 7) & !7;
    while at + 8 <= start + len {
        // SAFETY: the caller's.
        f(unsafe { std::ptr::read_volatile(at as *const usize) });
        at += 8;
    }
}

/// The constants made: each one's slot, and how it is read (as the
/// reading thread reads it: a thread's own, for a constant kept for each).
type ConstantRead = unsafe fn(usize, &mut dyn FnMut(usize));
static CONSTANTS: Mutex<Vec<(usize, ConstantRead)>> = Mutex::new(Vec::new());

/// A constant, made: its slot a root (once for each slot, however many
/// threads make it).
pub(crate) fn constant<T: Copy + 'static, S: crate::once::OnceSlot<T> + 'static>(slot: &'static S) {
    /// # Safety
    /// `at` is a slot's address, an `S` kept for the run (a static).
    unsafe fn read<T: Copy + 'static, S: crate::once::OnceSlot<T> + 'static>(
        at: usize,
        f: &mut dyn FnMut(usize),
    ) {
        // SAFETY: the caller's.
        let slot: &'static S = unsafe { &*(at as *const S) };
        if let Some(v) = slot.made() {
            words_of(&v, f);
        }
    }
    let at = std::ptr::from_ref(slot).addr();
    let mut cs = CONSTANTS.lock().unwrap_or_else(|e| e.into_inner());
    if !cs.iter().any(|&(a, _)| a == at) {
        cs.push((at, read::<T, S>));
    }
}

/// The thread's stack's base, its highest address (as the OS gives it,
/// so that a frame's words before kayak-rt was first called are counted).
fn stack_base() -> usize {
    thread_local!(static BASE: Cell<usize> = const { Cell::new(0) });
    BASE.with(|b| {
        if b.get() == 0 {
            b.set(os_stack_base());
        }
        b.get()
    })
}

#[cfg(target_os = "linux")]
fn os_stack_base() -> usize {
    // SAFETY: the calling thread's attributes, read into memory of their
    // size, and destroyed after.
    unsafe {
        let mut attr: libc::pthread_attr_t = std::mem::zeroed();
        assert_eq!(
            libc::pthread_getattr_np(libc::pthread_self(), &raw mut attr),
            0,
            "the thread's stack"
        );
        let (mut addr, mut size) = (std::ptr::null_mut(), 0usize);
        assert_eq!(
            libc::pthread_attr_getstack(&raw const attr, &raw mut addr, &raw mut size),
            0,
            "the thread's stack"
        );
        libc::pthread_attr_destroy(&raw mut attr);
        addr.addr() + size
    }
}

#[cfg(target_os = "macos")]
fn os_stack_base() -> usize {
    // SAFETY: the calling thread's own stack's address.
    unsafe { libc::pthread_get_stackaddr_np(libc::pthread_self()) }.addr()
}

/// The registers a caller may keep a value in across a call (its
/// callee-saved ones), spilled into `saved`.
#[inline(always)]
fn spill(saved: &mut [usize; 12]) {
    #[cfg(target_arch = "x86_64")]
    // SAFETY: stores of the callee-saved registers into `saved`, twelve
    // words, of which six are written.
    unsafe {
        std::arch::asm!(
            "mov [{0}], rbx",
            "mov [{0} + 8], rbp",
            "mov [{0} + 16], r12",
            "mov [{0} + 24], r13",
            "mov [{0} + 32], r14",
            "mov [{0} + 40], r15",
            in(reg) saved.as_mut_ptr(),
            options(nostack, preserves_flags)
        );
    }
    #[cfg(target_arch = "aarch64")]
    // SAFETY: stores of the callee-saved registers into `saved`, twelve
    // words, each written.
    unsafe {
        std::arch::asm!(
            "stp x19, x20, [{0}]",
            "stp x21, x22, [{0}, #16]",
            "stp x23, x24, [{0}, #32]",
            "stp x25, x26, [{0}, #48]",
            "stp x27, x28, [{0}, #64]",
            "stp x29, x30, [{0}, #80]",
            in(reg) saved.as_mut_ptr(),
            options(nostack, preserves_flags)
        );
    }
}

/// What a mark by the roots found (`reached`).
#[cfg_attr(not(any(test, feature = "heap-stats")), allow(dead_code))]
#[derive(Default, Debug)]
pub(crate) struct Reached {
    /// Words told, by root: the registers', the stack's, the constants',
    /// the owned blocks' (and how many blocks), the thread locals'.
    pub registers: u64,
    pub stack: u64,
    pub constants: u64,
    pub owned_blocks: u64,
    pub owned: u64,
    pub locals: u64,
    /// The roots' words that are in an object of the space.
    pub roots_in_space: u64,
    /// The kept objects a word reached, scanned, and their words.
    pub kept: u64,
    pub kept_words: u64,
    /// The allocations of the space reached: their first words.
    pub allocations: std::collections::BTreeSet<usize>,
}

/// The thread's roots, and what they reach by the space's bits and
/// headers alone (`laid`), and what is kept that a word reaches: a
/// collector's mark, nothing moved or freed.
///
/// Its frame a small one, a safepoint's: the registers spilled there and
/// the stack read from there, so that what the mark itself writes below
/// it (its own locals) is no root.
#[cfg_attr(not(any(test, feature = "heap-stats")), allow(dead_code))]
#[inline(never)]
pub(crate) fn reached() -> Reached {
    let mut saved = [0usize; 12];
    spill(&mut saved);
    let saved = std::hint::black_box(saved);
    let r = reached_from(&saved, std::ptr::from_ref(&saved).addr());
    std::hint::black_box(&saved);
    r
}

#[inline(never)]
fn reached_from(saved: &[usize; 12], from: usize) -> Reached {
    use super::{HeapWord, laid, space};
    let mut r = Reached::default();
    // The roots' words, ambiguous: the registers', the stack's from `from`
    // to its base.
    let mut words: Vec<usize> = saved.to_vec();
    let base = stack_base();
    // SAFETY: this thread's own stack, from its caller's frame to its
    // base, which the thread is running on.
    unsafe { scan_words(from, base.saturating_sub(from), &mut |w| words.push(w)) };
    r.registers = saved.len() as u64;
    r.stack = words.len() as u64 - r.registers;
    let constants: Vec<(usize, ConstantRead)> =
        CONSTANTS.lock().unwrap_or_else(|e| e.into_inner()).clone();
    for (at, read) in constants {
        // SAFETY: a slot `constant` noted, a static.
        unsafe {
            read(at, &mut |w| {
                r.constants += 1;
                words.push(w);
            })
        };
    }
    // Every thread's kept objects, by their first bytes; this thread's
    // owned blocks and thread locals.
    let mut kept: Vec<Kept> = Vec::new();
    let mut owned: Vec<(usize, usize)> = Vec::new();
    let mut locals = Vec::new();
    let me = mine();
    with_slab(|_, s| owned.extend(s.owned.iter().flatten()));
    for reg in REGISTRIES.lock().unwrap_or_else(|e| e.into_inner()).iter() {
        let i = reg.lock();
        kept.extend_from_slice(&i.kept);
        if me.is_some_and(|m| std::ptr::eq(m, *reg)) {
            locals.extend_from_slice(&i.locals);
        }
    }
    kept.sort_by_key(|k| k.start);
    for (start, len) in owned {
        r.owned_blocks += 1;
        // SAFETY: a block owned while registered, this thread's, alive.
        unsafe {
            scan_words(start, len, &mut |w| {
                r.owned += 1;
                words.push(w);
            })
        };
    }
    for f in locals {
        f(&mut |w| {
            r.locals += 1;
            words.push(w);
        });
    }
    let kept_at = |w: usize| -> Option<Kept> {
        let i = kept.partition_point(|k| k.start <= w);
        (i > 0 && w < kept[i - 1].start + kept[i - 1].len).then(|| kept[i - 1])
    };
    let mut kept_seen = std::collections::HashSet::new();
    // Ambiguous words (the roots', a kept object's), and the heap's
    // precise ones (an object's fields).
    let mut heap: Vec<HeapWord> = Vec::new();
    let mut ambiguous = words;
    loop {
        if let Some(w) = ambiguous.pop() {
            if let Some(a) = space::object_of(w) {
                r.roots_in_space += 1;
                if r.allocations.insert(a) {
                    // SAFETY: an allocation's first word in the space.
                    for o in unsafe { laid::allocation(a) } {
                        heap.extend_from_slice(o.fields);
                    }
                }
            } else if let Some(k) = kept_at(w)
                && kept_seen.insert(k.start)
            {
                r.kept += 1;
                let mut tell = |w| {
                    r.kept_words += 1;
                    ambiguous.push(w);
                };
                match k.scan {
                    // SAFETY: an object kept for the run, of its bytes.
                    None => unsafe { scan_words(k.start, k.len, &mut tell) },
                    // SAFETY: a `T` kept for the run, by `kept_scanned`.
                    Some(scan) => unsafe { scan(k.start, &mut tell) },
                }
            }
        } else if let Some(w) = heap.pop() {
            if let Some(a) = laid::target(w).and_then(space::object_of) {
                if r.allocations.insert(a) {
                    // SAFETY: an allocation's first word in the space.
                    for o in unsafe { laid::allocation(a) } {
                        heap.extend_from_slice(o.fields);
                    }
                }
            } else if w.raw() & 7 == 0 && w.raw() != 0 {
                // A plain reference (C508): what is kept, if it is.
                if kept_at(w.raw()).is_some() {
                    ambiguous.push(w.raw());
                }
            }
        } else {
            break;
        }
    }
    r
}

/// A future in a box of its own, its block a root while it lives (C493:
/// a future is frames Rust lays out, scanned as a stack is).
pub struct Rooted<F> {
    f: F,
    owned: Cell<Option<Owned>>,
}

/// `f` boxed, its box a root while it lives.
pub fn pinned<F: std::future::Future>(f: F) -> std::pin::Pin<Box<Rooted<F>>> {
    let b = Box::pin(Rooted {
        f,
        owned: Cell::new(None),
    });
    b.owned.set(owned(
        std::ptr::from_ref::<Rooted<F>>(&*b).addr(),
        std::mem::size_of::<Rooted<F>>(),
    ));
    b
}

impl<F: std::future::Future> std::future::Future for Rooted<F> {
    type Output = F::Output;
    #[inline]
    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<F::Output> {
        // SAFETY: `f` is never moved out of a pinned `Rooted`: a
        // structural pin.
        unsafe { self.map_unchecked_mut(|s| &mut s.f) }.poll(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Trace, heap_cons, nil};

    /// The allocation `v`'s word is in.
    fn allocation_of<T: Trace>(v: T) -> usize
    where
        T::Stored: Into<crate::obj::HeapWord>,
    {
        let w: crate::obj::HeapWord = v.store().into();
        crate::obj::laid::target(w)
            .and_then(crate::obj::space::object_of)
            .expect("an object of the space")
    }

    #[test]
    fn a_list_on_the_stack_reached() {
        let xs = heap_cons(1i64, heap_cons(2i64, nil()));
        let a = allocation_of(xs);
        let r = reached();
        assert!(r.allocations.contains(&a));
        assert!(r.stack > 0 && r.registers > 0);
        std::hint::black_box(xs);
    }

    /// A block `owned` registers: its words told while it lives.
    #[test]
    fn an_owned_block_told() {
        let (word, a) = made_apart();
        let block: Box<[usize; 4]> = Box::new([0, 0, word, 0]);
        let held = owned(std::ptr::from_ref(&*block).addr(), 32);
        let r = reached();
        assert!(r.allocations.contains(&a));
        assert!(r.owned_blocks >= 1 && r.owned >= 4);
        drop(held);
        let r = reached();
        assert_eq!((r.owned_blocks, r.owned), (0, 0));
        std::hint::black_box(&block);
    }

    #[inline(never)]
    fn made_apart() -> (usize, usize) {
        let xs = heap_cons(7i64, nil());
        (xs.store().raw(), allocation_of(xs))
    }

    #[test]
    fn a_future_box_is_owned() {
        let f = pinned(async { 1 });
        let at = std::ptr::from_ref::<Rooted<_>>(&*f).addr();
        assert!(owned_now().contains(&at));
        drop(f);
        assert!(!owned_now().contains(&at));
    }

    fn owned_now() -> Vec<usize> {
        with_slab(|_, s| s.owned.iter().flatten().map(|&(a, _)| a).collect()).unwrap()
    }

    #[test]
    fn a_future_box_dropped_on_another_thread() {
        let f = pinned(async { 1 });
        let at = std::ptr::from_ref::<Rooted<_>>(&*f).addr();
        let g = pinned(async { 2 });
        let at_g = std::ptr::from_ref::<Rooted<_>>(&*g).addr();
        // The box itself stays here; its `Owned` crosses, as a moved
        // task's would.
        let o = f.owned.take();
        std::thread::spawn(move || drop(o)).join().unwrap();
        assert!(!owned_now().contains(&at));
        assert!(owned_now().contains(&at_g));
        drop((f, g));
        assert!(!owned_now().contains(&at_g));
    }

    struct Two(usize, usize);
    impl Scanned for Two {
        fn scanned(&self, f: &mut dyn FnMut(usize)) {
            f(self.0 ^ self.1);
        }
    }

    /// A kept object scanned by its type: its word told only so.
    #[test]
    fn a_kept_object_scanned_by_its_type() {
        let (word, a) = made_apart();
        let k: &'static Two = Box::leak(Box::new(Two(word ^ 0x5555, 0x5555)));
        kept_scanned(k);
        let at = std::ptr::from_ref(k).addr();
        // Its address alone, on the stack, and in a kept block.
        let refs: &'static [usize; 1] = Box::leak(Box::new([at]));
        kept(std::ptr::from_ref(refs).addr(), 8);
        let r = reached();
        assert!(r.allocations.contains(&a));
        assert!(r.kept >= 1);
        std::hint::black_box(refs);
    }

    /// A region's pair: its arena's block a root while the region lives.
    #[test]
    fn a_region_arena_owned() {
        use crate::arena::{Arena, Holds};
        #[derive(Default)]
        struct Region<'r> {
            pairs: Arena<crate::Cons<'r, i64>>,
        }
        impl<'r> Holds<crate::Cons<'r, i64>> for Region<'r> {
            fn arena(&self) -> &Arena<crate::Cons<'r, i64>> {
                &self.pairs
            }
        }
        let before = reached().owned_blocks;
        {
            let region = Region::default();
            let xs = crate::arena::rcons(&region, 1, nil());
            std::hint::black_box(xs);
            assert!(reached().owned_blocks > before);
        }
        assert_eq!(reached().owned_blocks, before);
    }

    /// A constant's value reached from its slot alone.
    #[test]
    fn a_constant_reached() {
        static SLOT: std::sync::OnceLock<crate::List<i64>> = std::sync::OnceLock::new();
        let xs = crate::once::once(&SLOT, || heap_cons(3i64, nil()));
        let a = allocation_of(xs);
        let r = reached();
        assert!(r.constants >= 1);
        assert!(r.allocations.contains(&a));
    }

    /// A mailbox's message, told by its scan: from the mailbox's address.
    #[test]
    fn a_mailbox_scanned() {
        let (word, a) = made_apart();
        let mb: &'static crate::process::MailboxBox<usize> =
            crate::leak_scanned(crate::process::MailboxBox {
                queue: std::sync::Mutex::new(std::collections::VecDeque::from([word])),
            });
        let r = reached();
        assert!(r.allocations.contains(&a));
        assert!(r.kept >= 1);
        std::hint::black_box(mb);
    }
}
