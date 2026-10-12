//! FX-26's unions of shapes in KFX's Rust (C75, `docs/research/uniform-values.md`):
//! a uniform [`Value`] where a value's type is a union, and Rust's own types,
//! unboxed, where it is known. A member is boxed where it is given for its
//! union, and unboxed where a test narrows the union to it. Neither can be
//! seen by a KFX program: it has no `eq?`, and nothing a coercion touches is
//! written through another alias (C57 superseded).

use crate::{Int, Show, Str, Symbol, Void};
use std::any::Any;

/// A value of a union's type: a scalar unboxed, or a reference to what is
/// in the heap, behind `Any`, found again by its type when the union is
/// narrowed to it.
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
    Bytevector,
    Box,
    Sum,
    Product,
}

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
                    _ => crate::fail("a union narrowed to a member it does not hold"),
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

/// A member of another shape (a datatype's value, a sum, a product, a
/// reference, a procedure, an array) given for its union: behind `Any`,
/// its shape kept beside it.
pub fn other<T: Any + Copy>(x: T, shape: Shape) -> Value {
    Value::Other(Box::leak(Box::new(x)), shape)
}

/// A list given for its union: `nil`, or a pair.
pub fn list<T: 'static>(xs: crate::List<T>) -> Value {
    match xs {
        None => Value::Nil,
        Some(_) => Value::Other(Box::leak(Box::new(xs)), Shape::Pair),
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
            None => crate::fail("a union narrowed to a member it does not hold"),
        },
        _ => crate::fail("a union narrowed to a member it does not hold"),
    }
}

/// A union narrowed to a list member: `nil`, or the list it holds.
pub fn unbox_list<L: Any + Copy + Default>(v: Value) -> L {
    match v {
        Value::Nil => L::default(),
        _ => unbox_other(v),
    }
}

/// A union's `null?`; `pair?`, of a union or a list.
pub fn null_p<T: Shaped>(x: T) -> bool {
    x.shape() == Shape::Nil
}
pub fn pair_p<T: Shaped>(x: T) -> bool {
    x.shape() == Shape::Pair
}

impl<T> Shaped for crate::ListAt<'_, T> {
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

macro_rules! shaped_procedure {
    ($($a:ident),*) => {
        impl<$($a,)* R> Shaped for &dyn Fn($($a),*) -> R {
            fn shape(&self) -> Shape {
                Shape::Procedure
            }
        }
    };
}
shaped_procedure!();
shaped_procedure!(A);
shaped_procedure!(A, B);
shaped_procedure!(A, B, C);
shaped_procedure!(A, B, C, D);

pub fn f32_p<T: Shaped>(x: T) -> bool {
    x.shape() == Shape::F32
}
pub fn bytevector_p<T: Shaped>(x: T) -> bool {
    x.shape() == Shape::Bytevector
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

/// The shape tests, FX-26's, of a value of any type: a union's by its tag,
/// any other's by its type.
pub fn int_p<T: Shaped>(x: T) -> bool {
    x.shape() == Shape::Int
}
pub fn f64_p<T: Shaped>(x: T) -> bool {
    x.shape() == Shape::F64
}
pub fn char_p<T: Shaped>(x: T) -> bool {
    x.shape() == Shape::Char
}
pub fn bool_p<T: Shaped>(x: T) -> bool {
    x.shape() == Shape::Bool
}
pub fn string_p<T: Shaped>(x: T) -> bool {
    x.shape() == Shape::String
}
pub fn symbol_p<T: Shaped>(x: T) -> bool {
    x.shape() == Shape::Symbol
}
pub fn procedure_p<T: Shaped>(x: T) -> bool {
    x.shape() == Shape::Procedure
}
pub fn array_p<T: Shaped>(x: T) -> bool {
    x.shape() == Shape::Bloblet
}
pub fn ref_p<T: Shaped>(x: T) -> bool {
    x.shape() == Shape::Box
}
pub fn sum_p<T: Shaped>(x: T) -> bool {
    x.shape() == Shape::Sum
}
pub fn product_p<T: Shaped>(x: T) -> bool {
    x.shape() == Shape::Product
}

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
        let xs: crate::List<i64> = crate::cons(1, None);
        let l = list(xs);
        assert!(pair_p(l) && !null_p(l) && pair_p(xs));
        assert_eq!(crate::car(unbox_list::<crate::List<i64>>(l)), 1);
        assert!(null_p(list::<i64>(None)));
        let f: &'static dyn Fn(i64) -> i64 = &|x| x + 1;
        let p = other(f, Shape::Procedure);
        assert!(procedure_p(p));
        assert_eq!(unbox_other::<&'static dyn Fn(i64) -> i64>(p)(4), 5);
    }
}
