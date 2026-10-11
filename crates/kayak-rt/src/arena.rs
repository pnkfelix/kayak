//! FX-26's `letrena`: a region whose data lives while its body runs and is
//! freed, all at once, when the body ends. In Rust, a value of the region's
//! type (one the emitter writes for each `letrena`, `Region0<'r>`) holding
//! an [`Arena`] for each type the region holds, borrowed for `'r`: the
//! borrow checker keeps anything made in it from outliving it, as FX-26's
//! checker keeps the region out of the body's type.
//!
//! In safe Rust: an arena holds one type, in chunks of `OnceCell`s, each
//! chunk twice the last, hung on a `OnceCell` of the one before. Nothing
//! is ever taken out of a `OnceCell`, so a value's reference is good for
//! as long as the arena is borrowed. KFX's values are `Copy`, so no
//! destructor is ever wanted.

use crate::{Cons, ListAt, RefAt};
use std::cell::{Cell, OnceCell};

struct Chunk<T> {
    cells: Box<[OnceCell<T>]>,
    next: OnceCell<Box<Chunk<T>>>,
}

impl<T> Chunk<T> {
    fn new(n: usize) -> Self {
        Chunk {
            cells: (0..n).map(|_| OnceCell::new()).collect(),
            next: OnceCell::new(),
        }
    }
}

/// Values of one type, freed together when the arena is.
pub struct Arena<T> {
    head: Chunk<T>,
    used: Cell<usize>,
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Arena {
            head: Chunk::new(16),
            used: Cell::new(0),
        }
    }
}

impl<T> Arena<T> {
    /// `v`, kept in the arena: the next free cell, past the chunks that
    /// are full (a walk of as many chunks as the arena has, a few: each is
    /// twice the last).
    pub fn alloc(&self, v: T) -> &T {
        let mut i = self.used.get();
        self.used.set(i + 1);
        let mut chunk = &self.head;
        while i >= chunk.cells.len() {
            i -= chunk.cells.len();
            let n = chunk.cells.len() * 2;
            chunk = chunk.next.get_or_init(|| Box::new(Chunk::new(n)));
        }
        let cell = &chunk.cells[i];
        if cell.set(v).is_err() {
            crate::fail("arena: a cell filled twice");
        }
        match cell.get() {
            Some(v) => v,
            None => crate::fail("arena: a cell not filled"),
        }
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
    fn many_pairs_and_cells() {
        let region = Region::default();
        let r = &region;
        let mut xs = nil();
        for i in 0..10_000 {
            xs = rcons(r, i, xs);
        }
        let c = rnew(r, 0i64);
        c.set(sum(xs));
        assert_eq!(c.get(), 49_995_000);
    }
}
