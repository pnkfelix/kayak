//! `Trace` (C479): what of the heap a value reaches, precisely, as a
//! collector will need it (`TODO.md` §54; Bartlett's mostly-copying: the
//! Rust stack's words conservative, the heap's values precise). Each type
//! KFX's Rust holds says which of its parts are the heap's: a handle's
//! value, visited once; a string's bytes; and, opaque for now, what
//! kayak-rt leaks for itself (a closure, a dynamic sum, a bignum).
//!
//! A tracer keeps its own list of what is still to visit, so that a long
//! list is traced in a loop, not a recursion as deep as the list.

use crate::value::Value;
use crate::{Array, Big, Cons, Handle, Int, Str, Symbol, Void};
use std::cell::Cell;
use std::collections::{HashMap, HashSet};

/// What a value reaches of the heap, alive for `'a`: each of its handles'
/// values, told to the tracer. A value itself may be a copy (a cell's,
/// read): what it reaches is the heap's, not its own.
pub trait Trace<'a> {
    fn trace(&self, t: &mut Tracer<'a>);
}

/// What a trace has visited, and what it still must, each a value alive
/// for `'a`.
#[derive(Default)]
pub struct Tracer<'a> {
    seen: HashSet<usize>,
    pending: Vec<&'a dyn Trace<'a>>,
    /// Of each type a handle reached: how many values, and their bytes.
    counts: HashMap<&'static str, (u64, u64)>,
    strings: (u64, u64),
}

impl<'a> Tracer<'a> {
    /// A handle's value, `v`: counted and kept to visit, if not seen
    /// before.
    pub fn reach<T: Trace<'a> + 'a>(&mut self, v: &'a T) {
        if self
            .seen
            .insert(std::ptr::from_ref(v) as *const () as usize)
        {
            let e = self
                .counts
                .entry(std::any::type_name::<T>())
                .or_insert((0, 0));
            e.0 += 1;
            e.1 += std::mem::size_of::<T>() as u64;
            self.pending.push(v);
        }
    }

    /// A string's bytes, counted once.
    pub fn string(&mut self, s: &str) {
        if !s.is_empty() && self.seen.insert(s.as_ptr() as usize) {
            self.strings.0 += 1;
            self.strings.1 += s.len() as u64;
        }
    }

    /// Everything kept to visit, visited, and what that reaches.
    pub fn finish(&mut self) {
        while let Some(v) = self.pending.pop() {
            v.trace(self);
        }
    }

    /// What was reached: by type, how many and how many bytes, the most
    /// bytes first; and the strings'.
    pub fn census(&self) -> (Vec<CensusRow>, (u64, u64)) {
        let mut rows: Vec<_> = self.counts.iter().map(|(k, (n, b))| (*k, *n, *b)).collect();
        rows.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(b.0)));
        (rows, self.strings)
    }
}

/// A census's row: a type's name, how many of its values, their bytes.
pub type CensusRow = (&'static str, u64, u64);

/// What `root` reaches: traced to the end.
pub fn traced<'a, T: Trace<'a>>(root: &T) -> Tracer<'a> {
    let mut t = Tracer::default();
    root.trace(&mut t);
    t.finish();
    t
}

macro_rules! leaves {
    ($($t:ty),*) => { $(impl<'a> Trace<'a> for $t { #[inline] fn trace(&self, _: &mut Tracer<'a>) {} })* };
}
leaves!(
    i8,
    i16,
    i32,
    i64,
    u8,
    u16,
    u32,
    u64,
    f32,
    f64,
    bool,
    char,
    (),
    Void
);

/// A bytevector's bytes, the heap's but holding none of it.
impl<'a> Trace<'a> for crate::BytesAt<'_> {
    fn trace(&self, _: &mut Tracer<'a>) {}
}

impl<'a> Trace<'a> for Str {
    fn trace(&self, t: &mut Tracer<'a>) {
        t.string(self.as_str());
    }
}

/// A symbol: interned, the process's (C467's table), not the heap's.
impl<'a> Trace<'a> for Symbol {
    fn trace(&self, _: &mut Tracer<'a>) {}
}

/// An integer: a bignum's limbs are a `Vec` of its own, opaque.
impl<'a> Trace<'a> for Int {
    fn trace(&self, _: &mut Tracer<'a>) {}
}

impl<'a> Trace<'a> for Big {
    fn trace(&self, _: &mut Tracer<'a>) {}
}

impl<'a, 'r: 'a, T: Trace<'a> + 'r> Trace<'a> for Handle<'r, T> {
    #[inline]
    fn trace(&self, t: &mut Tracer<'a>) {
        t.reach(self.to_ref());
    }
}

impl<'a, T: Trace<'a>> Trace<'a> for Option<T> {
    #[inline]
    fn trace(&self, t: &mut Tracer<'a>) {
        if let Some(x) = self {
            x.trace(t);
        }
    }
}

impl<'a, 'r: 'a, T: Trace<'a>> Trace<'a> for Cons<'r, T> {
    fn trace(&self, t: &mut Tracer<'a>) {
        self.car.trace(t);
        self.cdr.trace(t);
    }
}

impl<'a, T: Trace<'a> + Copy> Trace<'a> for Cell<T> {
    fn trace(&self, t: &mut Tracer<'a>) {
        self.get().trace(t);
    }
}

impl<'a, T: Trace<'a> + Copy> Trace<'a> for Array<T> {
    fn trace(&self, t: &mut Tracer<'a>) {
        for c in self.0 {
            c.get().trace(t);
        }
    }
}

/// A dynamic value: its pair or its other member is kayak-rt's own,
/// leaked, opaque for now.
impl<'a> Trace<'a> for Value {
    fn trace(&self, t: &mut Tracer<'a>) {
        if let Value::Str(s) = self {
            s.trace(t);
        }
    }
}

/// A plain reference, what KFX's Rust has that is no handle yet: a
/// procedure (a closure kayak-rt leaked, `lambdaN`, whose captures Rust
/// does not show), a dynamic sum, a bloblet, a flat array; opaque for now
/// (a collector scans such a block as a whole).
impl<'a, T: ?Sized> Trace<'a> for &'_ T {
    fn trace(&self, _: &mut Tracer<'a>) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_list_reached_once_each() {
        let shared = crate::heap_cons(crate::str_lit("x"), None);
        let xs = crate::heap_cons(crate::str_lit("y"), shared);
        let ys = crate::heap_cons(crate::str_lit("z"), shared);
        let both = crate::heap_cons(xs, crate::heap_cons(ys, None));
        let t = traced(&both);
        let (rows, strings) = t.census();
        let pairs = |name: &str| {
            rows.iter()
                .filter(|r| r.0.contains(name))
                .map(|r| r.1)
                .sum::<u64>()
        };
        // Two pairs of the outer list, three of strings (the shared tail once).
        assert_eq!(pairs("Cons"), 5);
        assert_eq!(strings.0, 3);
    }

    #[test]
    fn a_long_list_in_a_loop() {
        let mut xs: crate::List<i64> = None;
        for k in 0..1_000_000 {
            xs = crate::heap_cons(k, xs);
        }
        let (rows, _) = traced(&xs).census();
        assert_eq!(rows.iter().map(|r| r.1).sum::<u64>(), 1_000_000);
    }
}
