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
use crate::{Array, BigAt, Cons, Handle, Int, Str, Symbol, Void};
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
    /// It visited where it is a one-word value, kept by the tracer only
    /// while visited (an array): what it reaches lives as long as the
    /// trace, not as the value. Visited by reference unless it says so.
    fn visit_owned(&self, _: &mut Tracer<'_>) {}
}

/// What a trace has visited, and what it still must, each alive for `'a`.
#[derive(Default)]
pub struct Tracer<'a> {
    seen: HashSet<usize>,
    pending: Vec<&'a (dyn Visit + 'a)>,
    /// What is still to visit of one-word values (an array, C486): each
    /// the value itself, which a reference would not outlive.
    pending_values: Vec<Box<dyn Visit + 'a>>,
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

    /// An object a one-word value points to (an array), at `addr`: counted
    /// and kept to visit, the value with it, if not seen before.
    pub fn reach_value<V: Visit + 'a>(&mut self, addr: usize, v: V) {
        if self.seen.insert(addr) {
            let e = self.counts.entry(v.visit_name()).or_insert((0, 0));
            e.0 += 1;
            e.1 += v.visit_size() as u64;
            self.pending_values.push(Box::new(v));
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
        loop {
            if let Some(v) = self.pending.pop() {
                v.visit(self);
            } else if let Some(v) = self.pending_values.pop() {
                // Visited as owned: what it reaches the heap's, not its.
                v.visit_owned(self);
            } else {
                break;
            }
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

/// A product (a tuple struct, C490): held boxed, one word to its object
/// (`Prod`, kind 37), where a heap object holds it, as FX-26's heap grammar
/// has it; its object its parts' fields and bytes apart (`Product`). On
/// the stack, its parts traced each.
#[macro_export]
macro_rules! trace_product {
    ($name:ident, $($a:ident $i:tt),*) => {
        impl<$($a: $crate::Trace),*> $crate::Trace for $name<$($a),*> {
            type Field = $crate::Prod<Self>;
            type Byte = ();
            #[inline]
            fn split(self) -> (Self::Field, ()) {
                ($crate::Prod::new(self), ())
            }
            #[inline(always)]
            fn join(f: Self::Field, _: ()) -> Self {
                f.get()
            }
            #[inline]
            fn trace<'a>(self, t: &mut $crate::Tracer<'a>) where Self: 'a {
                $(self.$i.trace(t);)*
            }
        }
        impl<$($a: $crate::Trace),*> $crate::Product for $name<$($a),*> {
            type Fields = $name<$($a::Field),*>;
            type Bytes = $name<$($a::Byte),*>;
            #[inline(always)]
            fn apart(self) -> (Self::Fields, Self::Bytes) {
                $(#[allow(non_snake_case)] let $a = $crate::Trace::split(self.$i);)*
                ($name($($a.0),*), $name($($a.1),*))
            }
            #[inline(always)]
            fn together(f: Self::Fields, b: Self::Bytes) -> Self {
                $name($($crate::Trace::join(f.$i, b.$i)),*)
            }
        }
        /// A union's member (C491): its object, kind 37.
        impl<$($a: $crate::Trace + 'static),*> $crate::value::Member for $name<$($a),*> {
            #[inline]
            fn object(self) -> &'static dyn $crate::value::Obj {
                $crate::Prod::new(self).object()
            }
            #[inline]
            fn of_object(o: &'static dyn $crate::value::Obj) -> Option<Self> {
                (o as &dyn std::any::Any)
                    .downcast_ref::<$crate::ProdObj<Self>>()
                    .map(|p| p.get())
            }
        }
    };
}

/// A product's parts, apart (C490): what its object (`Prod`) keeps, its
/// traced fields and its plain bytes, each part's split.
pub trait Product: Trace {
    type Fields: Trace;
    type Bytes: Copy;
    fn apart(self) -> (Self::Fields, Self::Bytes);
    fn together(f: Self::Fields, b: Self::Bytes) -> Self;
}

/// A value as a heap object holds it (C490): its traced part and its plain
/// bytes, its type's `split` (a product boxed, one word). A pair's
/// element, a sum's payload.
pub struct Held<T: Trace>(T::Field, T::Byte);

impl<T: Trace> Clone for Held<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Trace> Copy for Held<T> {}

impl<T: Trace> Held<T> {
    #[inline(always)]
    pub fn new(v: T) -> Self {
        let (f, b) = v.split();
        Held(f, b)
    }
    #[inline(always)]
    pub fn get(self) -> T {
        T::join(self.0, self.1)
    }
    /// What it reaches: its traced part's.
    #[inline]
    pub fn reach<'a>(self, t: &mut Tracer<'a>)
    where
        Self: 'a,
    {
        self.0.trace(t);
    }
}

impl<T: Trace<Field = (), Byte = T>> Held<T> {
    /// Plain bytes, held as they are: for a `static`'s (a `const`).
    pub const fn bytes(v: T) -> Self {
        Held((), v)
    }
}

impl<T: Trace<Field = T, Byte = ()>> Held<T> {
    /// A value kept whole, held as it is: for a `static`'s (a `const`).
    pub const fn whole(v: T) -> Self {
        Held(v, ())
    }
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

/// A bloblet's fields, each a cell (C486): each visited.
macro_rules! visit_tuples {
    ($(($($a:ident $i:tt),+)),*) => {
        $(impl<$($a: Visit),+> Visit for ($($a,)+) {
            fn visit<'a>(&'a self, t: &mut Tracer<'a>) {
                $(self.$i.visit(t);)+
            }
        })*
    };
}
visit_tuples!(
    (A 0),
    (A 0, B 1),
    (A 0, B 1, C 2),
    (A 0, B 1, C 2, D 3),
    (A 0, B 1, C 2, D 3, E 4),
    (A 0, B 1, C 2, D 3, E 4, F 5),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13, O 14),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13, O 14, P 15)
);

/// No fields.
impl Visit for () {
    fn visit<'a>(&'a self, _: &mut Tracer<'a>) {}
}

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

/// An integer (C488): a fixnum, nothing; a bignum, its object reached,
/// nothing in it traced.
impl Trace for Int {
    trace_whole!();
    #[inline]
    fn trace<'a>(self, t: &mut Tracer<'a>)
    where
        Self: 'a,
    {
        if let crate::IntView::Big(b) = self.view() {
            t.reach_value(self.word() & !0b111, b);
        }
    }
}

/// A flat array (C489): its object reached once; its elements numbers,
/// nothing traced.
impl<T: 'static> Trace for crate::FlatArray<T> {
    trace_whole!();
    #[inline]
    fn trace<'a>(self, t: &mut Tracer<'a>)
    where
        Self: 'a,
    {
        t.reach_value(self.addr(), self);
    }
}

impl<T> Visit for crate::FlatArray<T> {
    fn visit<'a>(&'a self, _: &mut Tracer<'a>) {}
    fn visit_size(&self) -> usize {
        self.object_bytes()
    }
}

impl Visit for BigAt {
    fn visit<'a>(&'a self, _: &mut Tracer<'a>) {}
    fn visit_size(&self) -> usize {
        self.object_bytes()
    }
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
        self.car_held().reach(t);
        self.cdr.trace(t);
    }
}

impl<T: Trace> Visit for Cell<T> {
    fn visit<'a>(&'a self, t: &mut Tracer<'a>) {
        self.get().trace(t);
    }
}

/// A cell of KFX's (C490): a reference's, its value held (a product boxed,
/// one word, as FX-26's box holds one). Read and written as a `Cell` is.
pub struct Slot<T: Trace>(Cell<Held<T>>);

impl<T: Trace> Slot<T> {
    #[inline(always)]
    pub fn new(v: T) -> Self {
        Slot(Cell::new(Held::new(v)))
    }
    #[inline(always)]
    pub fn get(&self) -> T {
        self.0.get().get()
    }
    #[inline(always)]
    pub fn set(&self, v: T) {
        self.0.set(Held::new(v))
    }
}

impl<T: Trace> Visit for Slot<T> {
    fn visit<'a>(&'a self, t: &mut Tracer<'a>) {
        self.0.get().reach(t);
    }
}

/// An array (C486): its object reached once, its elements visited then.
impl<T: Trace> Trace for Array<T> {
    trace_whole!();
    #[inline]
    fn trace<'a>(self, t: &mut Tracer<'a>)
    where
        Self: 'a,
    {
        t.reach_value(self.addr(), self);
    }
}

impl<T: Trace> Visit for Array<T> {
    fn visit<'a>(&'a self, t: &mut Tracer<'a>) {
        self.visit_owned(t);
    }
    fn visit_owned(&self, t: &mut Tracer<'_>) {
        for c in self.cells() {
            c.get().trace(t);
        }
    }
    fn visit_size(&self) -> usize {
        self.object_bytes()
    }
}

/// A dynamic value (C491): what it holds reached, precisely: a pair, a
/// member's object, a string, a bignum.
impl Trace for Value {
    trace_whole!();
    fn trace<'a>(self, t: &mut Tracer<'a>)
    where
        Self: 'a,
    {
        match self {
            Value::Str(s) => s.trace(t),
            Value::Symbol(s) => s.trace(t),
            Value::Int(n) => n.trace(t),
            Value::Pair(p) => t.reach(p),
            Value::Other(o, _) => t.reach(o),
            _ => {}
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
    fn arrays_and_bloblets_reached_once_each() {
        // An array of a string twice and a pair; a bloblet holding the
        // array twice and another string.
        let p = crate::heap_cons(crate::str_lit("p"), None);
        let a = crate::list_to_array(crate::heap_cons(
            p,
            crate::heap_cons(None, crate::heap_cons(p, None)),
        ));
        let b = crate::make_bloblet(
            crate::int_lit(12),
            (Cell::new(a), Cell::new(a), Cell::new(crate::str_lit("q"))),
        );
        let t = traced((b, b, a));
        let (rows, strings) = t.census();
        let of = |name: &str| {
            rows.iter()
                .filter(|r| r.0.starts_with(name))
                .collect::<Vec<_>>()
        };
        assert_eq!(of("kayak_rt::obj::Array<")[0].1, 1);
        // Its header, three elements of a word, its trailer.
        assert_eq!(of("kayak_rt::obj::Array<")[0].2, 5 * 8);
        assert_eq!(of("kayak_rt::obj::Bloblet<")[0].1, 1);
        // Its header, three fields, its trailer, two words of suffix.
        assert_eq!(of("kayak_rt::obj::Bloblet<")[0].2, 7 * 8);
        assert_eq!(of("kayak_rt::Cons<")[0].1, 1);
        assert_eq!(strings.0, 2);
    }

    /// A chain of arrays, each holding the next: a type of its own.
    #[derive(Clone, Copy)]
    struct Link(Option<crate::Array<Link>>);

    impl Trace for Link {
        trace_whole!();
        fn trace<'a>(self, t: &mut Tracer<'a>)
        where
            Self: 'a,
        {
            self.0.trace(t);
        }
    }

    #[test]
    fn a_long_chain_of_arrays_in_a_loop() {
        let mut a = crate::list_to_array(crate::heap_cons(Link(None), None));
        for _ in 0..200_000 {
            a = crate::list_to_array(crate::heap_cons(Link(Some(a)), None));
        }
        let (rows, _) = traced(a).census();
        assert_eq!(rows.iter().map(|r| r.1).sum::<u64>(), 200_001);
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
