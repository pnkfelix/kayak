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
    Other(&'static dyn Any, Shape),
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
pub trait PairLike {
    fn car(&self) -> Value;
    fn cdr(&self) -> Value;
    fn as_any(&self) -> &dyn Any;
}

/// A pair of a union's, `cons`ed where one is wanted.
struct ValuePair(Value, Value);

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
impl<T: ToValue + Copy + 'static> PairLike for Cons<'static, T> {
    fn car(&self) -> Value {
        self.car.to_value()
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

impl<T: ToValue + Copy + 'static> ToValue for List<T> {
    fn to_value(self) -> Value {
        list(self)
    }
}

impl ToValue for Vector {
    fn to_value(self) -> Value {
        other(self, Shape::Vector)
    }
}

macro_rules! procedure_value {
    ($($a:ident),*) => {
        impl<$($a: 'static,)* R: 'static> ToValue for &'static dyn Fn($($a),*) -> R {
            fn to_value(self) -> Value {
                other(self, Shape::Procedure)
            }
        }
        impl<$($a,)* R> Shaped for &dyn Fn($($a),*) -> R {
            fn shape(&self) -> Shape {
                Shape::Procedure
            }
        }
    };
}
procedure_value!();
procedure_value!(A);
procedure_value!(A, B);
procedure_value!(A, B, C);
procedure_value!(A, B, C, D);

impl<T: Copy + 'static> ToValue for &'static std::cell::Cell<T> {
    fn to_value(self) -> Value {
        other(self, Shape::Box)
    }
}

/// A member of another shape (a datatype's value, a sum, a product, a
/// reference, a procedure, an array) given for its union: behind `Any`,
/// its shape kept beside it.
pub fn other<T: Any + Copy>(x: T, shape: Shape) -> Value {
    Value::Other(crate::alloc(x), shape)
}

/// A list given for its union: `nil`, or its pair, seen as a union's.
pub fn list<T: ToValue + Copy + 'static>(xs: List<T>) -> Value {
    match xs {
        None => Value::Nil,
        Some(p) => Value::Pair(p),
    }
}

/// A list at `acyclic` given as a list of a union: copied, its elements
/// boxed (C78). Frozen, it is the same list as far as a program can tell.
pub fn boxed_list<T: ToValue + Copy>(xs: List<T>) -> List<Value> {
    let mut items = Vec::new();
    let mut p = xs;
    while let Some(c) = p {
        items.push(c.car.to_value());
        p = c.cdr;
    }
    items
        .into_iter()
        .rev()
        .fold(None, |tail, x| crate::heap_cons(x, tail))
}

/// `cons` where a pair of a union is wanted, its parts boxed.
pub fn cons<A: ToValue, B: ToValue>(a: A, b: B) -> Value {
    Value::Pair(crate::alloc(ValuePair(a.to_value(), b.to_value())))
}

/// A quasiquote's `,@xs`: the list's elements, boxed, before `rest`.
pub fn spliced<T: ToValue + Copy>(xs: List<T>, rest: Value) -> Value {
    let mut items = Vec::new();
    let mut p = xs;
    while let Some(c) = p {
        items.push(c.car.to_value());
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
pub fn unbox_other<T: Any + Copy>(v: Value) -> T {
    match v {
        Value::Other(x, _) => match x.downcast_ref::<T>() {
            Some(t) => *t,
            None => not_held(),
        },
        _ => not_held(),
    }
}

/// A union narrowed to a list member: `nil`, or the list whose pair it
/// holds.
pub fn unbox_list<T: 'static>(v: Value) -> List<T> {
    match v {
        Value::Nil => None,
        Value::Pair(p) => match p.as_any().downcast_ref::<Cons<'static, T>>() {
            Some(c) => Some(c),
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

impl<T> Shaped for ListAt<'_, T> {
    fn shape(&self) -> Shape {
        if self.is_none() {
            Shape::Nil
        } else {
            Shape::Pair
        }
    }
}

impl<T> Shaped for &std::cell::Cell<T> {
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
                if let Some(v) = x.downcast_ref::<Vector>() {
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
        let f: &'static dyn Fn(i64) -> i64 = &|x| x + 1;
        let p = f.to_value();
        assert!(procedure_p(p));
        assert_eq!(unbox_other::<&'static dyn Fn(i64) -> i64>(p)(4), 5);
        let d = cons(1i64, cons(str_lit("a"), Value::Nil));
        let mut out = String::new();
        d.show(&mut out);
        assert_eq!(out, "(1 \"a\")");
        let mut out = String::new();
        cons(1i64, 2i64).show(&mut out);
        assert_eq!(out, "(1 . 2)");
    }
}
