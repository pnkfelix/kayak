//! `(arrayof T R)`: an array of values, each a cell, as FX-26's may be
//! aliased and written through any alias (`CLAUDES_CHOICES.md` C56).

use crate::{Int, List, Show, fail, heap_cons, i64_to_int, int_to_i64, nil};
use std::cell::Cell;

#[derive(Clone, Copy)]
pub struct Array<T: 'static>(pub &'static [Cell<T>]);

/// As Fixpt writes one: a bloblet of a header word and a field each.
impl<T> Show for Array<T> {
    fn show(&self, out: &mut String) {
        out.push_str(&format!("#<bloblet {} fields 0 bytes>", self.0.len() + 1));
    }
}

fn cell<T>(a: Array<T>, i: Int) -> &'static Cell<T> {
    match usize::try_from(int_to_i64(i)).ok().and_then(|i| a.0.get(i)) {
        Some(c) => c,
        None => fail("array: index out of range"),
    }
}

pub fn make_array<T: Copy>(n: Int, v: T) -> Array<T> {
    match usize::try_from(int_to_i64(n)) {
        Ok(n) => Array(Box::leak(
            (0..n)
                .map(|_| Cell::new(v))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )),
        Err(_) => fail("make-array: a negative length"),
    }
}

#[inline]
pub fn array_ref<T: Copy>(a: Array<T>, i: Int) -> T {
    cell(a, i).get()
}

#[inline]
pub fn array_set_x<T: Copy>(a: Array<T>, i: Int, v: T) {
    cell(a, i).set(v)
}

#[inline]
pub fn array_length<T>(a: Array<T>) -> Int {
    i64_to_int(a.0.len() as i64)
}

pub fn array_to_list<T: Copy + 'static>(a: Array<T>) -> List<T> {
    a.0.iter()
        .rev()
        .fold(nil(), |acc, c| heap_cons(c.get(), acc))
}

pub fn list_to_array<T: Copy>(xs: List<T>) -> Array<T> {
    let mut v = Vec::new();
    let mut at = xs;
    while let Some(c) = at {
        v.push(Cell::new(c.car));
        at = c.cdr;
    }
    Array(Box::leak(v.into_boxed_slice()))
}
