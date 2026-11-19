//! What of the heap a value reaches (C479, C481), precisely, as a collector
//! will need it (`TODO.md` §54; Bartlett's mostly-copying: the Rust stack's
//! words conservative, the heap's values precise).
//!
//! Two traits, as FX-26's heap grammar has two things (C481):
//! - **`Trace`**, of a value (every KFX value is `Copy`): traced by value,
//!   so a cell's copy traces what it reaches; and split, by its type, into
//!   its traced part (`Field`: handles, strings, what a collector follows)
//!   and its plain bytes (`Byte`: an `i64`, an `f64`, a character), as a
//!   bloblet keeps its traced fields apart from its untraced suffix. A
//!   closure's captures are laid out by it (`closure.rs`).
//! - **`Visit`**, of what a handle points to (a datatype's value, a pair, a
//!   cell, a sum's cell, a closure): visited by reference, once, and
//!   object-safe, so a closure behind `dyn` is visited too.
//!
//! A lifetime is the method's, not the trait's: a handle's `'r` outlives
//! the trace's `'a`, as the value it is in does.
//!
//! A tracer keeps its own list of what is still to visit, so that a long
//! list is traced in a loop, not a recursion as deep as the list.

use crate::value::Value;
use crate::{Array, Big, Cons, Handle, Int, Str, Symbol, Void};
use std::cell::Cell;
use std::collections::{HashMap, HashSet};

/// A value: what of the heap it reaches, told to the tracer; and its
/// traced part and its plain bytes, apart (a bloblet's fields and suffix).
pub trait Trace: Copy {
    /// Its part a collector follows.
    type Field: Trace;
    /// Its plain bytes.
    type Byte: Copy;
    fn split(self) -> (Self::Field, Self::Byte);
    fn join(f: Self::Field, b: Self::Byte) -> Self;
    fn trace<'a>(self, t: &mut Tracer<'a>)
    where
        Self: 'a;
}

/// What a handle points to: visited by reference, once.
pub trait Visit {
    fn visit<'a>(&'a self, t: &mut Tracer<'a>);
    /// Its type's name, for a census.
    fn visit_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    /// Its bytes, for a census.
    fn visit_size(&self) -> usize {
        std::mem::size_of_val(self)
    }
}

/// What a trace has visited, and what it still must, each alive for `'a`.
#[derive(Default)]
pub struct Tracer<'a> {
    seen: HashSet<usize>,
    pending: Vec<&'a (dyn Visit + 'a)>,
    /// Of each type a handle reached: how many values, and their bytes.
    counts: HashMap<&'static str, (u64, u64)>,
    strings: (u64, u64),
    /// The plain references met (a bloblet, a flat array): what a census
    /// reads as opaque blocks (C480).
    opaque: Vec<usize>,
}

impl<'a> Tracer<'a> {
    /// A handle's value, `v`: counted and kept to visit, if not seen
    /// before.
    pub fn reach(&mut self, v: &'a (dyn Visit + 'a)) {
        if self
            .seen
            .insert(std::ptr::from_ref(v).cast::<()>() as usize)
        {
            let e = self.counts.entry(v.visit_name()).or_insert((0, 0));
            e.0 += 1;
            e.1 += v.visit_size() as u64;
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

    /// A plain reference met, to what no `Trace` says the parts of.
    pub fn opaque(&mut self, p: usize) {
        self.opaque.push(p);
    }

    /// The plain references met since asked last.
    pub fn take_opaque(&mut self) -> Vec<usize> {
        std::mem::take(&mut self.opaque)
    }

    /// Everything kept to visit, visited, and what that reaches.
    pub fn finish(&mut self) {
        while let Some(v) = self.pending.pop() {
            v.visit(self);
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
pub fn traced<'a, T: Trace + 'a>(root: T) -> Tracer<'a> {
    let mut t = Tracer::default();
    root.trace(&mut t);
    t.finish();
    t
}

/// A value all plain bytes: none of it traced.
#[macro_export]
macro_rules! trace_bytes {
    ($($t:ty),*) => {
        $(impl $crate::Trace for $t {
            type Field = ();
            type Byte = Self;
            #[inline(always)]
            fn split(self) -> ((), Self) { ((), self) }
            #[inline(always)]
            fn join(_: (), b: Self) -> Self { b }
            #[inline(always)]
            fn trace<'a>(self, _: &mut $crate::Tracer<'a>) where Self: 'a {}
        })*
    };
}

/// A value kept whole among the traced fields (`Field` itself, no bytes):
/// in an impl of `Trace`, before its `trace`.
#[macro_export]
macro_rules! trace_whole {
    () => {
        type Field = Self;
        type Byte = ();
        #[inline(always)]
        fn split(self) -> (Self, ()) {
            (self, ())
        }
        #[inline(always)]
        fn join(f: Self, _: ()) -> Self {
            f
        }
    };
}

/// A product (a tuple struct): each field split apart, as a tuple's.
#[macro_export]
macro_rules! trace_product {
    ($name:ident, $($a:ident $i:tt),*) => {
        impl<$($a: $crate::Trace),*> $crate::Trace for $name<$($a),*> {
            type Field = $name<$($a::Field),*>;
            type Byte = $name<$($a::Byte),*>;
            #[inline(always)]
            fn split(self) -> (Self::Field, Self::Byte) {
                $(#[allow(non_snake_case)] let $a = self.$i.split();)*
                ($name($($a.0),*), $name($($a.1),*))
            }
            #[inline(always)]
            fn join(f: Self::Field, b: Self::Byte) -> Self {
                $name($($a::join(f.$i, b.$i)),*)
            }
            #[inline]
            fn trace<'a>(self, t: &mut $crate::Tracer<'a>) where Self: 'a {
                $(self.$i.trace(t);)*
            }
        }
    };
}

trace_bytes!(
    i8, i16, i32, i64, u8, u16, u32, u64, usize, f32, f64, bool, char, Void
);

/// Nothing: neither traced nor bytes.
impl Trace for () {
    type Field = ();
    type Byte = ();
    #[inline(always)]
    fn split(self) -> ((), ()) {
        ((), ())
    }
    #[inline(always)]
    fn join(_: (), _: ()) {}
    #[inline(always)]
    fn trace<'a>(self, _: &mut Tracer<'a>)
    where
        Self: 'a,
    {
    }
}

/// Tuples: each part's split apart, the fields together and the bytes
/// together (a closure's captures, `closure.rs`).
macro_rules! trace_tuples {
    ($(($($a:ident $i:tt),+)),*) => {
        $(impl<$($a: Trace),+> Trace for ($($a,)+) {
            type Field = ($($a::Field,)+);
            type Byte = ($($a::Byte,)+);
            #[inline(always)]
            fn split(self) -> (Self::Field, Self::Byte) {
                $(#[allow(non_snake_case)] let $a = self.$i.split();)+
                (($($a.0,)+), ($($a.1,)+))
            }
            #[inline(always)]
            fn join(f: Self::Field, b: Self::Byte) -> Self {
                ($($a::join(f.$i, b.$i),)+)
            }
            #[inline]
            fn trace<'a>(self, t: &mut Tracer<'a>) where Self: 'a {
                $(self.$i.trace(t);)+
            }
        })*
    };
}
trace_tuples!(
    (A 0),
    (A 0, B 1),
    (A 0, B 1, C 2),
    (A 0, B 1, C 2, D 3),
    (A 0, B 1, C 2, D 3, E 4),
    (A 0, B 1, C 2, D 3, E 4, F 5),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7)
);

/// A bytevector's bytes, the heap's but holding none of it.
impl Trace for crate::BytesAt<'_> {
    trace_whole!();
    fn trace<'a>(self, _: &mut Tracer<'a>)
    where
        Self: 'a,
    {
    }
}

impl Trace for Str {
    trace_whole!();
    fn trace<'a>(self, t: &mut Tracer<'a>)
    where
        Self: 'a,
    {
        t.string(self.as_str());
    }
}

/// A symbol: interned, the process's (C467's table), not the heap's.
impl Trace for Symbol {
    trace_whole!();
    fn trace<'a>(self, _: &mut Tracer<'a>)
    where
        Self: 'a,
    {
    }
}

/// An integer: a bignum's limbs are a `Vec` of its own, opaque.
impl Trace for Int {
    trace_whole!();
    fn trace<'a>(self, _: &mut Tracer<'a>)
    where
        Self: 'a,
    {
    }
}

impl Visit for Big {
    fn visit<'a>(&'a self, _: &mut Tracer<'a>) {}
}

impl<'r, T: Visit + 'r> Trace for Handle<'r, T> {
    trace_whole!();
    #[inline]
    fn trace<'a>(self, t: &mut Tracer<'a>)
    where
        Self: 'a,
    {
        t.reach(self.to_ref());
    }
}

impl<T: Trace> Trace for Option<T> {
    trace_whole!();
    #[inline]
    fn trace<'a>(self, t: &mut Tracer<'a>)
    where
        Self: 'a,
    {
        if let Some(x) = self {
            x.trace(t);
        }
    }
}

impl<T: Trace> Visit for Cons<'_, T> {
    fn visit<'a>(&'a self, t: &mut Tracer<'a>) {
        self.car.trace(t);
        self.cdr.trace(t);
    }
}

impl<T: Trace> Visit for Cell<T> {
    fn visit<'a>(&'a self, t: &mut Tracer<'a>) {
        self.get().trace(t);
    }
}

impl<T: Trace> Trace for Array<T> {
    trace_whole!();
    fn trace<'a>(self, t: &mut Tracer<'a>)
    where
        Self: 'a,
    {
        for c in self.0 {
            c.get().trace(t);
        }
    }
}

/// A dynamic value: its pair or its other member is kayak-rt's own,
/// leaked, opaque for now.
impl Trace for Value {
    trace_whole!();
    fn trace<'a>(self, t: &mut Tracer<'a>)
    where
        Self: 'a,
    {
        if let Value::Str(s) = self {
            s.trace(t);
        }
    }
}

/// A plain reference, what KFX's Rust has that is no handle yet: a dynamic
/// sum, a bloblet, a flat array; opaque for now (a collector scans such a
/// block as a whole).
impl<T: ?Sized> Trace for &'_ T {
    trace_whole!();
    fn trace<'a>(self, t: &mut Tracer<'a>)
    where
        Self: 'a,
    {
        t.opaque(std::ptr::from_ref::<T>(self).cast::<()>() as usize);
    }
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
        let t = traced(both);
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
        let (rows, _) = traced(xs).census();
        assert_eq!(rows.iter().map(|r| r.1).sum::<u64>(), 1_000_000);
    }

    #[test]
    fn a_tuple_split_into_fields_and_bytes() {
        let s = crate::str_lit("s");
        let ((f0, f1, f2), (b0, b1, b2)) = (3i64, s, 2.5f64).split();
        let _: ((), Str, ()) = (f0, f1, f2);
        let _: (i64, (), f64) = (b0, b1, b2);
        let back = <(i64, Str, f64)>::join((f0, f1, f2), (b0, b1, b2));
        assert_eq!((back.0, back.2), (3, 2.5));
    }
}
