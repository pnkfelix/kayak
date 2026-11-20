//! `(arrayof T R)`: an array of values, each a cell, as FX-26's may be
//! aliased and written through any alias (`CLAUDES_CHOICES.md` C56).

use crate::{Int, List, Show, fail, heap_cons, i64_to_int, int_to_i64, nil};

/// One word to an object of Fixpt's layout (C485, `obj.rs`).
pub use crate::obj::Array;

/// As Fixpt writes one: a bloblet of a header word and a field each.
impl<T: crate::Trace> Show for Array<T> {
    fn show(&self, out: &mut String) {
        out.push_str(&format!("#<bloblet {} fields 0 bytes>", self.len() + 1));
    }
}

fn cell<T: crate::Trace>(a: Array<T>, i: Int) -> &'static crate::Slot<T> {
    match usize::try_from(int_to_i64(i))
        .ok()
        .and_then(|i| a.cells().get(i))
    {
        Some(c) => c,
        None => fail("array: index out of range"),
    }
}

pub fn make_array<T: crate::Trace + 'static>(n: Int, v: T) -> Array<T> {
    match usize::try_from(int_to_i64(n)) {
        Ok(n) => crate::obj::array_made(std::iter::repeat_n(v, n)),
        Err(_) => fail("make-array: a negative length"),
    }
}

#[inline]
pub fn array_ref<T: crate::Trace>(a: Array<T>, i: Int) -> T {
    cell(a, i).get()
}

#[inline]
pub fn array_set_x<T: crate::Trace>(a: Array<T>, i: Int, v: T) {
    cell(a, i).set(v)
}

#[inline]
pub fn array_length<T: crate::Trace>(a: Array<T>) -> Int {
    i64_to_int(a.len() as i64)
}

pub fn array_to_list<T: Copy + 'static + crate::Trace>(a: Array<T>) -> List<T> {
    a.cells()
        .iter()
        .rev()
        .fold(nil(), |acc, c| heap_cons(c.get(), acc))
}

pub fn list_to_array<T: crate::Trace + 'static>(xs: List<T>) -> Array<T> {
    let mut v = Vec::new();
    let mut at = xs;
    while let Some(c) = at.pair() {
        v.push(c.car());
        at = c.cdr;
    }
    crate::obj::array_made(v.into_iter())
}
