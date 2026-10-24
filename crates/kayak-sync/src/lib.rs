//! What `kayak-rt` needs of threads that Rust cannot be shown is safe, for
//! the `unsafe-par` mode alone (`CLAUDES_CHOICES.md` C102): `par`'s branches
//! that share data some branch writes, which KFX's checker proves no other
//! branch touches (Lucassen and Gifford's interference table), run at once;
//! and the globals they may see, made once for the whole program. The one
//! crate of kayak's with `unsafe` (PLAN.md Q7). Without the mode, those
//! branches run in turn, and globals are each a thread's, in safe Rust.

/// A value Rust cannot see may cross threads, which the checker says may.
struct AssertSend<T>(T);

// SAFETY: made only in `par2_unchecked`, of a `par` branch and its value:
// what the branch holds that another branch can see, it only reads if the
// other only reads it, and writes only what no other branch reads or
// writes, which KFX's checker proved (`par-clash`, check-exprs.kfx).
unsafe impl<T> Send for AssertSend<T> {}

impl<T> AssertSend<T> {
    fn into_inner(self) -> T {
        self.0
    }
}

/// A thread's stack, as big as the main thread's is made.
const STACK: usize = 1 << 30;

/// `f` here and `g` on a scoped thread, at once; `finish` given `f`'s value
/// and the join of `g`'s thread, which gives `g`'s.
pub fn par2_unchecked<A, B>(
    f: impl FnOnce() -> A,
    g: impl FnOnce() -> std::thread::Result<B>,
    finish: impl FnOnce(A, &mut dyn FnMut() -> std::thread::Result<std::thread::Result<B>>) -> (A, B),
) -> (A, B) {
    std::thread::scope(|scope| {
        let g = AssertSend(g);
        let other = std::thread::Builder::new()
            .stack_size(STACK)
            .spawn_scoped(scope, move || AssertSend(g.into_inner()()))
            .unwrap_or_else(|e| panic!("par: no thread: {e}"));
        let a = f();
        let mut other = Some(other);
        let mut join = move || match other.take() {
            Some(h) => h.join().map(AssertSend::into_inner),
            None => panic!("par: joined twice"),
        };
        finish(a, &mut join)
    })
}

/// A global made once for the whole program, not once per thread: a `par`
/// branch on another thread sees the same value as the program, a
/// reference the same cell.
pub struct Global<T>(std::sync::OnceLock<T>);

// SAFETY: a global is shared between threads only by `par2_unchecked`'s
// branches, which never interfere (above); a process holds only globals of
// transmissible types, which are `OnceLock`s of their own, checked by rustc.
unsafe impl<T> Sync for Global<T> {}

impl<T> Global<T> {
    pub const fn new() -> Self {
        Global(std::sync::OnceLock::new())
    }

    /// The value, made by `make` the first time, on whichever thread first
    /// asks; another asking meanwhile waits for it.
    pub fn get_or_init(&self, make: impl FnOnce() -> T) -> &T {
        self.0.get_or_init(make)
    }

    /// The value, if made yet.
    #[inline]
    pub fn get(&self) -> Option<&T> {
        self.0.get()
    }
}

impl<T> Default for Global<T> {
    fn default() -> Self {
        Self::new()
    }
}
