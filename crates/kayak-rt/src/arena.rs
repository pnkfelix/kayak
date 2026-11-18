//! FX-26's `letrena`: a region whose data lives while its body runs and is
//! freed, all at once, when the body ends. In Rust, a value of the region's
//! type (one the emitter writes for each `letrena`, `Region0<'r>`) holding
//! an [`Arena`] for each type the region holds, borrowed for `'r`: the
//! borrow checker keeps anything made in it from outliving it, as FX-26's
//! checker keeps the region out of the body's type.
//!
//! In safe Rust: an arena holds one type, in chunks of `OnceCell`s, each
//! chunk twice the last, hung on a `OnceCell` of the one before; a chunk
//! past [`BLOCK`] cells in blocks of that many, each made when first
//! reached (C364), so that the chunk's unused part is not made, as a chunk
//! made whole was: a third of a compile's memory, written and never used.
//! Nothing
//! is ever taken out of a `OnceCell`, so a value's reference is good for
//! as long as the arena is borrowed. KFX's values are `Copy`, so no
//! destructor is ever wanted.

use crate::{Cons, ListAt, RefAt};
use std::cell::{Cell, OnceCell};

/// The first chunk's cells; each chunk after has twice the last's.
const FIRST: usize = 16;

/// The most cells a chunk makes at once: a block of a larger chunk.
const BLOCK: usize = FIRST << 8;

/// Chunks enough for any arena memory could hold: the last alone is
/// `FIRST << 47` cells.
const CHUNKS: usize = 48;

/// A block's cells, each filled once.
type Block<T> = Box<[OnceCell<T>]>;

/// A chunk's blocks, each made when first reached.
type Chunk<T> = Box<[OnceCell<Block<T>>]>;

/// Values of one type, freed together when the arena is.
pub struct Arena<T> {
    chunks: [OnceCell<Chunk<T>>; CHUNKS],
    used: Cell<usize>,
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Arena {
            chunks: std::array::from_fn(|_| OnceCell::new()),
            used: Cell::new(0),
        }
    }
}

/// The chunk the `i`th value is in, and where in it: chunk `k` holds
/// `FIRST << k` cells, after `FIRST * (2^k - 1)` in the chunks before it,
/// so no walk finds it (the chunk a current one, without a reference into
/// the arena that safe Rust would not let it keep).
#[inline]
fn place(i: usize) -> (usize, usize) {
    let q = i / FIRST + 1;
    let k = (usize::BITS - 1 - q.leading_zeros()) as usize;
    (k, i - FIRST * ((1 << k) - 1))
}

impl<T> Arena<T> {
    /// `v`, kept in the arena, in the next free cell.
    #[inline]
    pub fn alloc(&self, v: T) -> crate::Handle<'_, T> {
        self.alloc_noting(v, |_, _| {})
    }

    /// `alloc`, and each block it makes told to `noted`, its first cell and
    /// how many (C480: the heap's, for a census).
    #[inline]
    pub(crate) fn alloc_noting(
        &self,
        v: T,
        noted: impl FnOnce(*const OnceCell<T>, usize),
    ) -> crate::Handle<'_, T> {
        let i = self.used.get();
        self.used.set(i + 1);
        let (k, at) = place(i);
        let size = FIRST << k;
        let len = size.min(BLOCK);
        let blocks =
            self.chunks[k].get_or_init(|| (0..size / len).map(|_| OnceCell::new()).collect());
        let mut made = false;
        let block = blocks[at / len].get_or_init(|| {
            made = true;
            (0..len).map(|_| OnceCell::new()).collect()
        });
        if made {
            noted(block.as_ptr(), block.len());
        }
        crate::Handle::made(block[at % len].get_or_init(|| v))
    }
}

/// A region that holds values of type `T`: its arena for them.
pub trait Holds<T> {
    fn arena(&self) -> &Arena<T>;
}

/// `(rcons r x xs)`: a pair made in the region `r`.
pub fn rcons<'r, T, R: Holds<Cons<'r, T>>>(r: &'r R, car: T, cdr: ListAt<'r, T>) -> ListAt<'r, T> {
    Some(r.arena().alloc(Cons { car, cdr }))
}

/// `(rnew r v)`: a reference made in the region `r`.
pub fn rnew<'r, T: Copy, R: Holds<Cell<T>>>(r: &'r R, v: T) -> RefAt<'r, T> {
    r.arena().alloc(Cell::new(v))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{car, cdr, nil, null_p};

    #[derive(Default)]
    struct Region<'r> {
        pairs: Arena<Cons<'r, i64>>,
        cells: Arena<Cell<i64>>,
    }

    impl<'r> Holds<Cons<'r, i64>> for Region<'r> {
        fn arena(&self) -> &Arena<Cons<'r, i64>> {
            &self.pairs
        }
    }

    impl Holds<Cell<i64>> for Region<'_> {
        fn arena(&self) -> &Arena<Cell<i64>> {
            &self.cells
        }
    }

    fn sum(mut xs: ListAt<'_, i64>) -> i64 {
        let mut n = 0;
        while !null_p(xs) {
            n += car(xs);
            xs = cdr(xs);
        }
        n
    }

    #[test]
    fn places_follow_on() {
        let mut want = (0, 0);
        for i in 0..100_000 {
            assert_eq!(place(i), want, "at {i}");
            want.1 += 1;
            if want.1 == FIRST << want.0 {
                want = (want.0 + 1, 0);
            }
        }
    }

    #[test]
    fn many_pairs_and_cells() {
        let region = Region::default();
        let r = &region;
        let mut xs = nil();
        for i in 0..100_000 {
            xs = rcons(r, i, xs);
        }
        let c = rnew(r, 0i64);
        c.set(sum(xs));
        assert_eq!(c.get(), 4_999_950_000);
    }
}
