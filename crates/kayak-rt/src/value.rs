//! FX-26's unions of shapes in KFX's Rust (C75, `docs/research/uniform-values.md`):
//! a uniform [`Value`] where a value's type is a union, `datum` or a pair
//! whose tail is no list, and Rust's own types, unboxed, where it is
//! known. A member is boxed where it is given for its union, and unboxed
//! where a test narrows the union to it. Neither can be seen by a KFX
//! program: it has no `eq?`, and nothing a coercion touches is written
//! through another alias (C57 superseded).
//!
//! A pair is a [`PairLike`]: a pair of `Value`s, made by `cons` where a
//! pair of a union is wanted, or a typed list's own pair, seen as one of
//! `Value`s (a view: each element boxed as it is read), so that a list is
//! given for a union, or as a datum, as it is (C77).

use crate::{Cons, Int, List, ListAt, Show, Str, Symbol, Void};
use std::any::Any;

/// A value of a union's type: a scalar unboxed, a pair, or a reference to
/// what is in the heap, behind `Any`, found again by its type when the
/// union is narrowed to it.
#[derive(Clone, Copy)]
pub enum Value {
    I64(i64),
    Int(Int),
    F64(f64),
    Char(char),
    Bool(bool),
    Str(Str),
    Symbol(Symbol),
    Unit,
    Nil,
    Pair(&'static dyn PairLike),
    Other(&'static dyn Obj, Shape),
}

/// What a union holds of a member of another shape (C491): the member's
/// own object, as FX-26's union holds it (a word to it): a datatype's
/// value's, a sum's cell, a product's (kind 37), a ref's cell; what has
/// none of its own yet (a procedure, an array) a box of it. Visited, so a
/// trace follows it; found again by its type.
pub trait Obj: Any + crate::Visit {}

impl<T: Any + crate::Visit> Obj for T {}

/// A member of another shape: its object, and the member again from it.
pub trait Member: Copy + 'static {
    fn object(self) -> &'static dyn Obj;
    fn of_object(o: &'static dyn Obj) -> Option<Self>;
}

/// A handle's value (a datatype's, a ref): its object.
impl<D: crate::Visit + 'static> Member for crate::Handle<'static, D> {
    #[inline]
    fn object(self) -> &'static dyn Obj {
        self.to_ref()
    }
    #[inline]
    fn of_object(o: &'static dyn Obj) -> Option<Self> {
        (o as &dyn Any).downcast_ref::<D>().map(crate::Handle::made)
    }
}

/// A sum: its cell, by its head.
impl Member for crate::Sum {
    #[inline]
    fn object(self) -> &'static dyn Obj {
        self.head()
    }
    #[inline]
    fn of_object(o: &'static dyn Obj) -> Option<Self> {
        (o as &dyn Any)
            .downcast_ref::<crate::SumHead>()
            .map(crate::Sum::of_head)
    }
}

/// A member with no object of its own yet, boxed: the box traces it.
pub struct Boxed<T>(T);

impl<T: crate::Trace> crate::Visit for Boxed<T> {
    fn visit<'a>(&'a self, t: &mut crate::Tracer<'a>) {
        self.0.trace(t);
    }
}

/// `x` boxed, in the heap, as a member.
pub fn boxed_object<T: crate::Trace + 'static>(x: T) -> &'static dyn Obj {
    crate::heap_alloc(Boxed(x)).to_ref()
}

/// The member a box holds.
pub fn of_boxed<T: Copy + 'static>(o: &'static dyn Obj) -> Option<T> {
    (o as &dyn Any).downcast_ref::<Boxed<T>>().map(|b| b.0)
}

/// Members of no object of their own, boxed.
#[macro_export]
macro_rules! boxed_member {
    ($(<$($p:ident $(: $b:path)?),*> $t:ty;)*) => {$(
        impl<$($p: 'static $(+ $b)?),*> $crate::value::Member for $t {
            #[inline]
            fn object(self) -> &'static dyn $crate::value::Obj {
                $crate::value::boxed_object(self)
            }
            #[inline]
            fn of_object(o: &'static dyn $crate::value::Obj) -> Option<Self> {
                $crate::value::of_boxed(o)
            }
        }
    )*};
}

boxed_member! {
    <T: crate::Trace> crate::Array<T>;
    <> Vector;
    <> f32;
    <> i32;
    <> u32;
    <> u64;
    <> u8;
}

/// A value's shape at run time, which a test tells apart (FX-26's).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shape {
    Int,
    F64,
    F32,
    Char,
    Bool,
    String,
    Symbol,
    Nil,
    Pair,
    Procedure,
    Bloblet,
    Vector,
    Bytevector,
    Box,
    Sum,
    Product,
}

/// A pair, as a union holds one: its head and tail as `Value`s.
pub trait PairLike: crate::Visit {
    fn car(&self) -> Value;
    fn cdr(&self) -> Value;
    fn as_any(&self) -> &dyn Any;
}

/// A pair of a union's, `cons`ed where one is wanted.
struct ValuePair(Value, Value);

impl crate::Visit for ValuePair {
    fn visit<'a>(&'a self, t: &mut crate::Tracer<'a>) {
        crate::Trace::trace(self.0, t);
        crate::Trace::trace(self.1, t);
    }
}

impl PairLike for ValuePair {
    fn car(&self) -> Value {
        self.0
    }
    fn cdr(&self) -> Value {
        self.1
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// A typed list's pair, seen as a union's: its elements boxed as read.
impl<T: ToValue + crate::Trace + 'static> PairLike for Cons<'static, T> {
    fn car(&self) -> Value {
        self.car().to_value()
    }
    fn cdr(&self) -> Value {
        list(self.cdr)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// `datum`'s vector.
pub type Vector = &'static [Value];

/// What a value's shape is, for every type a union may have as a member.
pub trait Shaped {
    fn shape(&self) -> Shape;
}

/// A member given for its union: boxed.
pub trait ToValue {
    fn to_value(self) -> Value;
}

/// A union narrowed to a member: unboxed, the member's type known.
pub trait FromValue: Sized {
    fn from_value(v: Value) -> Self;
}

fn not_held() -> ! {
    crate::fail("a union narrowed to a member it does not hold")
}

macro_rules! scalar {
    ($t:ty, $v:ident, $shape:ident) => {
        impl Shaped for $t {
            fn shape(&self) -> Shape {
                Shape::$shape
            }
        }
        impl ToValue for $t {
            fn to_value(self) -> Value {
                Value::$v(self)
            }
        }
        impl FromValue for $t {
            fn from_value(v: Value) -> Self {
                match v {
                    Value::$v(x) => x,
                    _ => not_held(),
                }
            }
        }
    };
}

scalar!(i64, I64, Int);
scalar!(Int, Int, Int);
scalar!(f64, F64, F64);
scalar!(char, Char, Char);
scalar!(bool, Bool, Bool);
scalar!(Str, Str, String);
scalar!(Symbol, Symbol, Symbol);

impl Shaped for () {
    fn shape(&self) -> Shape {
        Shape::Symbol
    }
}
impl ToValue for () {
    fn to_value(self) -> Value {
        Value::Unit
    }
}
impl FromValue for () {
    fn from_value(_: Value) -> Self {}
}

impl ToValue for Void {
    fn to_value(self) -> Value {
        match self {}
    }
}

impl<T: ToValue + crate::Trace + 'static> ToValue for List<T> {
    fn to_value(self) -> Value {
        list(self)
    }
}

impl ToValue for Vector {
    fn to_value(self) -> Value {
        other(self, Shape::Vector)
    }
}

impl<T: crate::Trace + 'static> ToValue for &'static crate::Slot<T> {
    fn to_value(self) -> Value {
        other(crate::Handle::made(self), Shape::Box)
    }
}

/// A ref, a handle of a cell (C478), as its cell's reference.
impl<T: crate::Trace + 'static> ToValue for crate::Handle<'static, crate::Slot<T>> {
    fn to_value(self) -> Value {
        other(self, Shape::Box)
    }
}

/// A member of another shape (a datatype's value, a sum, a product, a
/// reference, a procedure, an array) given for its union: behind `Any`,
/// its shape kept beside it.
pub fn other<T: Member>(x: T, shape: Shape) -> Value {
    Value::Other(x.object(), shape)
}

/// A list given for its union: `nil`, or its pair, seen as a union's.
pub fn list<T: ToValue + crate::Trace + 'static>(xs: List<T>) -> Value {
    match xs {
        None => Value::Nil,
        Some(p) => Value::Pair(p.to_ref()),
    }
}

/// A list at `acyclic` given as a list of a union: copied, its elements
/// boxed (C78). Frozen, it is the same list as far as a program can tell.
pub fn boxed_list<T: ToValue + crate::Trace>(xs: List<T>) -> List<Value> {
    let mut items = Vec::new();
    let mut p = xs;
    while let Some(c) = p {
        items.push(c.car().to_value());
        p = c.cdr;
    }
    items
        .into_iter()
        .rev()
        .fold(None, |tail, x| crate::heap_cons(x, tail))
}

/// `cons` where a pair of a union is wanted, its parts boxed.
pub fn cons<A: ToValue, B: ToValue>(a: A, b: B) -> Value {
    Value::Pair(crate::heap_alloc(ValuePair(a.to_value(), b.to_value())).to_ref())
}

/// A quasiquote's `,@xs`: the list's elements, boxed, before `rest`.
pub fn spliced<T: ToValue + crate::Trace>(xs: List<T>, rest: Value) -> Value {
    let mut items = Vec::new();
    let mut p = xs;
    while let Some(c) = p {
        items.push(c.car().to_value());
        p = c.cdr;
    }
    items.into_iter().rev().fold(rest, |tail, x| cons(x, tail))
}

/// `car` and `cdr` of a union with a pair: checked, as FX-26's every machine
/// checks.
pub fn car(v: Value) -> Value {
    match v {
        Value::Pair(p) => p.car(),
        _ => crate::fail("car of no pair"),
    }
}
pub fn cdr(v: Value) -> Value {
    match v {
        Value::Pair(p) => p.cdr(),
        _ => crate::fail("cdr of no pair"),
    }
}

/// A union narrowed to a scalar member.
pub fn unbox<T: FromValue>(v: Value) -> T {
    T::from_value(v)
}

/// A union narrowed to a member of another shape: found again by its type.
pub fn unbox_other<T: Member>(v: Value) -> T {
    match v {
        Value::Other(x, _) => match T::of_object(x) {
            Some(t) => t,
            None => not_held(),
        },
        _ => not_held(),
    }
}

/// A union narrowed to a list member: `nil`, or the list whose pair it
/// holds.
pub fn unbox_list<T: crate::Trace + 'static>(v: Value) -> List<T> {
    match v {
        Value::Nil => None,
        Value::Pair(p) => match p.as_any().downcast_ref::<Cons<'static, T>>() {
            Some(c) => Some(crate::Handle::made(c)),
            None => not_held(),
        },
        _ => not_held(),
    }
}

/// `datum-list->vector`: a datum's list, its elements in a vector.
pub fn datum_list_to_vector(d: Value) -> Value {
    let mut v = Vec::new();
    let mut p = d;
    while let Value::Pair(c) = p {
        v.push(c.car());
        p = c.cdr();
    }
    let v: Vector = Box::leak(v.into_boxed_slice());
    other(v, Shape::Vector)
}

impl Shaped for Value {
    fn shape(&self) -> Shape {
        match self {
            Value::I64(_) | Value::Int(_) => Shape::Int,
            Value::F64(_) => Shape::F64,
            Value::Char(_) => Shape::Char,
            Value::Bool(_) => Shape::Bool,
            Value::Str(_) => Shape::String,
            Value::Symbol(_) | Value::Unit => Shape::Symbol,
            Value::Nil => Shape::Nil,
            Value::Pair(_) => Shape::Pair,
            Value::Other(_, s) => *s,
        }
    }
}

impl ToValue for Value {
    fn to_value(self) -> Value {
        self
    }
}

impl FromValue for Value {
    fn from_value(v: Value) -> Self {
        v
    }
}

impl Shaped for Void {
    fn shape(&self) -> Shape {
        match *self {}
    }
}

impl<T: crate::Trace> Shaped for ListAt<'_, T> {
    fn shape(&self) -> Shape {
        if self.is_none() {
            Shape::Nil
        } else {
            Shape::Pair
        }
    }
}

impl<T: crate::Trace> Shaped for &crate::Slot<T> {
    fn shape(&self) -> Shape {
        Shape::Box
    }
}

impl<T: crate::Trace> Shaped for crate::Handle<'_, crate::Slot<T>> {
    fn shape(&self) -> Shape {
        Shape::Box
    }
}

impl Shaped for f32 {
    fn shape(&self) -> Shape {
        Shape::F32
    }
}

/// The shape tests, FX-26's, of a value of any type: a union's by its tag,
/// any other's by its type.
macro_rules! shape_test {
    ($name:ident, $shape:ident) => {
        pub fn $name<T: Shaped>(x: T) -> bool {
            x.shape() == Shape::$shape
        }
    };
}
shape_test!(int_p, Int);
shape_test!(f64_p, F64);
shape_test!(f32_p, F32);
shape_test!(char_p, Char);
shape_test!(bool_p, Bool);
shape_test!(string_p, String);
shape_test!(symbol_p, Symbol);
shape_test!(procedure_p, Procedure);
shape_test!(array_p, Bloblet);
shape_test!(vector_p, Vector);
shape_test!(bytevector_p, Bytevector);
shape_test!(ref_p, Box);
shape_test!(sum_p, Sum);
shape_test!(product_p, Product);
shape_test!(null_p, Nil);
shape_test!(pair_p, Pair);

/// Written as Fixpt writes the member it holds.
impl Show for Value {
    fn show(&self, out: &mut String) {
        match self {
            Value::I64(x) => x.show(out),
            Value::Int(x) => x.show(out),
            Value::F64(x) => x.show(out),
            Value::Char(x) => x.show(out),
            Value::Bool(x) => x.show(out),
            Value::Str(x) => x.show(out),
            Value::Symbol(x) => x.show(out),
            Value::Unit => ().show(out),
            Value::Nil => out.push_str("()"),
            Value::Pair(p) => {
                out.push('(');
                p.car().show(out);
                let mut rest = p.cdr();
                loop {
                    match rest {
                        Value::Nil => break,
                        Value::Pair(q) => {
                            out.push(' ');
                            q.car().show(out);
                            rest = q.cdr();
                        }
                        _ => {
                            out.push_str(" . ");
                            rest.show(out);
                            break;
                        }
                    }
                }
                out.push(')');
            }
            Value::Other(x, Shape::Vector) => {
                out.push_str("#(");
                if let Some(v) = of_boxed::<Vector>(*x) {
                    for (i, e) in v.iter().enumerate() {
                        if i > 0 {
                            out.push(' ');
                        }
                        e.show(out);
                    }
                }
                out.push(')');
            }
            Value::Other(_, Shape::Procedure) => out.push_str("#<procedure>"),
            Value::Other(..) => out.push_str("#<value>"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::str_lit;

    #[test]
    fn boxed_and_narrowed() {
        let v = 3i64.to_value();
        assert!(int_p(v) && !string_p(v));
        assert_eq!(i64::from_value(v), 3);
        let s = str_lit("x").to_value();
        assert!(string_p(s));
        assert!(int_p(7i64) && !int_p(str_lit("y")));
        let xs: List<i64> = crate::cons(1, None);
        let l = list(xs);
        assert!(pair_p(l) && !null_p(l) && pair_p(xs));
        assert_eq!(crate::car(unbox_list::<i64>(l)), 1);
        assert_eq!(i64::from_value(car(l)), 1);
        assert!(null_p(cdr(l)));
        assert!(null_p(list::<i64>(None)));
        let f: crate::Proc1<'static, i64, i64> = crate::closure1((), |(), x| x + 1);
        let p = other(f, Shape::Procedure);
        assert!(procedure_p(p));
        assert_eq!(unbox_other::<crate::Proc1<'static, i64, i64>>(p).call(4), 5);
        let d = cons(1i64, cons(str_lit("a"), Value::Nil));
        let mut out = String::new();
        d.show(&mut out);
        assert_eq!(out, "(1 \"a\")");
        let mut out = String::new();
        cons(1i64, 2i64).show(&mut out);
        assert_eq!(out, "(1 . 2)");
    }

    #[test]
    fn a_member_its_own_object_and_traced() {
        // A sum given for a union: the union holds its cell, no box.
        let s = crate::sum_of("pair", 7i64);
        let v = other(s, Shape::Sum);
        match v {
            Value::Other(o, Shape::Sum) => {
                assert_eq!(std::ptr::from_ref(o).cast::<()>() as usize, s.addr())
            }
            _ => panic!("not a sum's"),
        }
        assert_eq!(unbox_other::<crate::Sum>(v).addr(), s.addr());
        // A ref: its cell.
        let r: crate::Ref<i64> = crate::heap_new(5);
        assert!(unbox_other::<crate::Ref<i64>>(r.to_value()).same(r));
        // A datum of a string, the sum and the ref: each reached once,
        // precisely, nothing opaque.
        let d = cons(str_lit("a"), cons(v, cons(r.to_value(), Value::Nil)));
        let mut t = crate::traced(d);
        assert!(t.take_opaque().is_empty());
        let (rows, strings) = t.census();
        let of = |name: &str| {
            rows.iter()
                .filter(|r| r.0.contains(name))
                .map(|r| r.1)
                .sum::<u64>()
        };
        assert_eq!(of("ValuePair"), 3);
        assert_eq!(of("SumOf<i64>"), 1);
        assert_eq!(of("Slot<i64>"), 1);
        assert_eq!(strings.0, 1);
        // Three words, not Fixpt's one (`TODO.md` §54).
        assert_eq!(std::mem::size_of::<Value>(), 24);
    }
}
