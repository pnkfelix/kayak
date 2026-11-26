#![allow(unsafe_code)]
//! A collection's roots, as each runtime finds them (C537; the plan's
//! step 3): the one thing about a collection the two runtimes must do
//! differently, so a trait, where their objects' format is one (the
//! plan's principle). kayak-rt's are conservative: the Rust stacks'
//! words, each perhaps a pointer, pin what they reach, and its kept
//! blocks' slots are exact. kc's are exact: every word of kc's stack is a
//! value of Fixpt's format (or a return address, outside the heap), and
//! so is every global's cell.

/// Where a collection's roots are.
pub trait Roots {
    /// Each exact root: the address of a slot holding a value, which the
    /// collection reads and rewrites where what it reaches moved.
    fn each_exact(&mut self, f: &mut dyn FnMut(usize));

    /// Each ambiguous root: a word that may be a pointer, whose object, if
    /// it is one, is kept and not moved. None, by default.
    fn each_ambiguous(&mut self, f: &mut dyn FnMut(u64)) {
        let _ = f;
    }
}

/// The slots of runs of words, each `[lo, hi)`, all exact: a native
/// stack's frames (from its pointer to its base), a run of globals' cells.
pub struct Runs<'a>(pub &'a [(usize, usize)]);

impl Roots for Runs<'_> {
    fn each_exact(&mut self, f: &mut dyn FnMut(usize)) {
        for &(lo, hi) in self.0 {
            let mut a = lo;
            while a < hi {
                f(a);
                a += 8;
            }
        }
    }
}

// What kayak-rt's collector took its roots as (`obj::roots`, C511-C513),
// here with the collector since C540.

/// How a kept block is scanned: by its type (`T`'s words told), or, none,
/// word by word.
pub type Scan = Option<unsafe fn(usize, &mut dyn FnMut(usize))>;

/// What is kept that a collection reaches only by a word (`Kept::free`):
/// this thread's, and a live thread's, by first byte, to find the one a
/// word is in.
#[derive(Default)]
pub struct KeptIndex {
    at: Vec<KeptAt>,
    /// The least first byte, and past the greatest last.
    lo: usize,
    hi: usize,
}

#[derive(Clone, Copy)]
pub struct KeptAt {
    pub start: usize,
    pub len: usize,
    pub scan: Scan,
    pub shared: bool,
    /// This thread's: scanned when reached, and freed if not. Another
    /// thread's is scanned only if shared (it locks what it reads), and
    /// told in this thread's list (`Reach`) when reached from its own.
    pub mine: bool,
}

impl KeptIndex {
    #[inline]
    pub fn made(mut at: Vec<KeptAt>) -> KeptIndex {
        at.sort_unstable_by_key(|k| k.start);
        let lo = at.first().map_or(0, |k| k.start);
        let hi = at.iter().map(|k| k.start + k.len).max().unwrap_or(0);
        KeptIndex { at, lo, hi }
    }

    /// This thread's kept objects that are not shared: scanned when
    /// reached.
    #[inline]
    pub fn mine_unshared(&self) -> impl Iterator<Item = KeptAt> + '_ {
        self.at.iter().copied().filter(|k| k.mine && !k.shared)
    }

    /// The kept object `w` is a word of, if any.
    #[inline]
    pub fn find(&self, w: usize) -> Option<KeptAt> {
        if w < self.lo || w >= self.hi {
            return None;
        }
        let i = self.at.partition_point(|k| k.start <= w);
        (i > 0 && w < self.at[i - 1].start + self.at[i - 1].len).then(|| self.at[i - 1])
    }
}

impl KeptAt {
    /// Its words told.
    #[inline]
    pub fn words(&self, f: &mut dyn FnMut(usize)) {
        match self.scan {
            // SAFETY: an object kept, of its bytes, not freed while a
            // collection that began before it was retired runs.
            None => unsafe { scan_words(self.start, self.len, f) },
            // SAFETY: a `T` kept, as above.
            Some(scan) => unsafe { scan(self.start, f) },
        }
    }
}

/// The aligned words of the `len` bytes from `start`, told.
///
/// # Safety
/// The bytes are readable for the scan. A word of padding may be told: an
/// ambiguous root's, which a collector only pins by.
#[inline]
pub unsafe fn scan_words(start: usize, len: usize, f: &mut dyn FnMut(usize)) {
    let mut at = (start + 7) & !7;
    while at + 8 <= start + len {
        // SAFETY: the caller's.
        f(unsafe { std::ptr::read_volatile(at as *const usize) });
        at += 8;
    }
}

/// A collection's roots as a runtime gathers them (kayak-rt's `obj::roots`,
/// C511; here since C540), every word ambiguous:
/// the registers spilled in `saved` and the stack from `from`, this
/// thread's owned blocks and thread locals, and every kept object, each
/// whole, by its bytes or its type (kept for the run, and perhaps reached
/// where no root is seen: a thread local of its own, a scheduler's slot);
/// and apart, the constants' words, which every thread may read, and what
/// other threads may take as this one collects (C512): what shared kept
/// objects hold, and every other thread's list (`Reach`).
pub struct Gathered {
    pub words: Vec<usize>,
    pub constants: Vec<usize>,
    pub shared: Vec<usize>,
    /// What is kept that is followed only where a word reaches it.
    pub kept: KeptIndex,
    /// Where this thread's list ended as the collection began.
    pub reach_from: usize,
    /// Where in `words` each source's words end: the registers' and
    /// stack's, the owned blocks', the kept objects' (the thread locals'
    /// after), for a report.
    pub ends: [usize; 3],
}

// A native stack's roots (kayak-rt's `obj::roots`, here since C541, for a
// built kc program's runtime too): its base, the registers spilled, and
// the cells of the native code running on it.

/// The thread's stack's base, its highest address (as the OS gives it,
/// so that a frame's words before kayak-rt was first called are counted).
#[inline]
pub fn stack_base() -> usize {
    thread_local!(static BASE: std::cell::Cell<usize> = const { std::cell::Cell::new(0) });
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
pub fn spill(saved: &mut [usize; 12]) {
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

std::thread_local! {
    /// The globals' cells of the native code running on this thread with
    /// room in its heap (C538; kcrt's `heap`, C541), `[lo, hi)`: roots, as its
    /// frames on this stack are, while it runs.
    static NATIVE_CELLS: std::cell::Cell<(usize, usize)> = const { std::cell::Cell::new((0, 0)) };
}

/// The globals' cells of the native code now running, `[lo, hi)`; `(0, 0)`,
/// none.
#[inline]
pub fn native_cells(run: (usize, usize)) {
    NATIVE_CELLS.set(run);
}

/// The globals' cells of the native code now running on this thread.
#[inline]
pub fn native_cells_run() -> (usize, usize) {
    NATIVE_CELLS.get()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_each_slot_once() {
        let mut seen = Vec::new();
        Runs(&[(64, 88), (200, 208), (300, 300)]).each_exact(&mut |a| seen.push(a));
        assert_eq!(seen, vec![64, 72, 80, 200]);
        let mut none = 0;
        Runs(&[(0, 16)]).each_ambiguous(&mut |_| none += 1);
        assert_eq!(none, 0);
    }
}
