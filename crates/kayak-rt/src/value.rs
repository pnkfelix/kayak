//! FX-26's unions of shapes in KFX's Rust (C75, `docs/research/uniform-values.md`):
//! a uniform [`Value`] where a value's type is a union, `datum` or a pair
//! whose tail is no list, and Rust's own types, unboxed, where it is
//! known. A member is given for its union as its word, and narrowed again
//! where a test narrows the union to it. Neither can be seen by a KFX
//! program: it has no `eq?`, and nothing a coercion touches is written
//! through another alias (C57 superseded).
//!
//! A value is one Fixpt word (C506), as Fixpt's every value is: a fixnum,
//! an immediate, a pair's word, an object's; its shape its tag's, or its
//! object's kind's, as Fixpt's headers say. A member's word is the one a
//! heap object holds it as (`Trace::Stored`) where that is Fixpt's own
//! (`Trace::VALUE_WORD`), so a typed list is a list of values as it is
//! (C77), its pairs Fixpt's cells. Narrowed, a member is found again by its
//! kind (a datatype's value by its tag too), not its Rust type: its
//! element types the checker's to know, as Fixpt's native code has them.

use crate::{HeapWord, Int, List, ListAt, Show, Str, Symbol, Void};
use std::any::Any;

/// A value of a union's type (C506): one Fixpt word. Neither `Send` nor
/// `Sync`, as before: it may be a word to a box another alias writes.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Value(HeapWord, std::marker::PhantomData<*const ()>);

/// What a sum's object keeps to trace its payload (C491, C499): it, as
/// its type has it.
pub trait Obj: Any + crate::Visit {}

impl<T: Any + crate::Visit> Obj for T {}

/// Fixpt's kinds a value's object may be of.
mod kind {
    pub const STRING: u8 = 1;
    pub const SYMBOL: u8 = 2;
    pub const VECTOR: u8 = 3;
    pub const BYTEVECTOR: u8 = 4;
    pub const FLONUM: u8 = 5;
    pub const BIGNUM: u8 = 6;
    pub const BOX: u8 = 10;
    pub const BLOBLET: u8 = 32;
    pub const SUM: u8 = 36;
    pub const PRODUCT: u8 = 37;
    pub const NATIVE_CLOSURE: u8 = 41;
    pub const FLAT_ARRAY: u8 = 42;
}

impl Value {
    /// The empty list, Fixpt's immediate.
    #[allow(non_upper_case_globals)]
    pub const Nil: Value = Value(HeapWord::NULL, std::marker::PhantomData);

    /// The value a word is.
    #[inline(always)]
    pub fn of_word(w: HeapWord) -> Value {
        Value(w, std::marker::PhantomData)
    }

    /// Its word.
    #[inline(always)]
    pub fn word(self) -> HeapWord {
        self.0
    }

    /// Whether it is the empty list.
    #[inline(always)]
    pub fn is_nil(self) -> bool {
        self.0.raw() == HeapWord::NULL.raw()
    }

    /// Whether it is a pair.
    #[inline(always)]
    pub fn is_pair(self) -> bool {
        self.0.is_pair()
    }

    /// Its object's kind, if it is a word to an object.
    #[inline(always)]
    fn kind(self) -> Option<u8> {
        self.0.kind()
    }

    /// Whether it is an integer: a fixnum, or a bignum's word.
    #[inline]
    fn is_int(self) -> bool {
        self.0.tag() == 0 || self.kind() == Some(kind::BIGNUM)
    }

    /// Its head and tail, if a pair.
    #[inline]
    fn parts(self) -> Option<(Value, Value)> {
        self.is_pair().then(|| {
            let (a, d) = self.0.pair_words();
            (Value::of_word(a), Value::of_word(d))
        })
    }
}

/// A member of another shape (a datatype's value, a sum, a product, a
/// reference, a procedure, an array): given for its union as its word, and
/// found again by its kind.
pub trait Member: Copy + 'static {
    /// It given for its union, of shape `shape`: its word.
    fn into_value(self, shape: Shape) -> Value;
    /// It, if `v` holds it.
    fn of_value(v: Value) -> Option<Self>;
}

/// A member whose word is its object's, of kind `$kind` (C506).
#[macro_export]
macro_rules! word_member {
    ([$($p:tt)*] $t:ty; $($kind:expr),+) => {
        impl<$($p)*> $crate::value::Member for $t {
            #[inline]
            fn into_value(self, _: $crate::value::Shape) -> $crate::value::Value {
                $crate::value::Value::of_word($crate::Trace::store(self))
            }
            #[inline]
            fn of_value(v: $crate::value::Value) -> ::core::option::Option<Self> {
                let k = $crate::value::kind_of(v)?;
                ($(k == $kind)||+).then(|| <Self as $crate::Trace>::load(v.word()))
            }
        }
    };
}

/// A value's object's kind, if it is a word to an object (for
/// `word_member!`).
#[doc(hidden)]
#[inline(always)]
pub fn kind_of(v: Value) -> Option<u8> {
    v.kind()
}

/// A sum (C499): its word, to Fixpt's sum of four fields (its tag, its
/// payload, its payload's type, its trailer; a datatype's value's has
/// three).
impl Member for crate::Sum {
    #[inline]
    fn into_value(self, _: Shape) -> Value {
        Value::of_word(self.word())
    }
    #[inline]
    fn of_value(v: Value) -> Option<Self> {
        let h = v.0.object_header()?;
        ((h >> 3) & 0xff == kind::SUM as u64 && (h >> 14) & 0x3ffff == 4)
            .then(|| crate::Sum::of_word(v.0))
    }
}

// An array: its word, to Fixpt's plain bloblet (kind 32, as Fixpt's
// `make-array` makes one, C506); a datum's vector's, to Fixpt's vector
// (kind 3).
word_member!([T: crate::Trace + 'static] crate::Array<T>; kind::BLOBLET, kind::VECTOR);

/// An integer of a fixed width: its word, a fixnum (or, `u64`, a bignum's).
macro_rules! int_member {
    ($($t:ty),*) => {$(
        impl Member for $t {
            #[inline]
            fn into_value(self, _: Shape) -> Value {
                Value::of_word(crate::Trace::store(self))
            }
            #[inline]
            fn of_value(v: Value) -> Option<Self> {
                v.is_int().then(|| <$t as crate::Trace>::load(v.0))
            }
        }
    )*};
}

int_member!(i32, u32, u64, u8);

/// An `f32`: Fixpt's immediate of it.
impl Member for f32 {
    #[inline]
    fn into_value(self, _: Shape) -> Value {
        Value::of_word(HeapWord::of_f32(self))
    }
    #[inline]
    fn of_value(v: Value) -> Option<Self> {
        (v.0.immediate() == Some(crate::obj::IMM_F32)).then(|| v.0.to_f32())
    }
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

/// `datum`'s vector (C505): Fixpt's vector (kind 3) of its elements, an
/// array of them, where it was a slice of its own.
pub type Vector = crate::Array<Value>;

/// What a value's shape is, for every type a union may have as a member.
pub trait Shaped {
    fn shape(&self) -> Shape;
}

/// A member given for its union: its word.
pub trait ToValue {
    fn to_value(self) -> Value;
}

/// A union narrowed to a member: the member's type known.
pub trait FromValue: Sized {
    fn from_value(v: Value) -> Self;
}

fn not_held() -> ! {
    crate::fail("a union narrowed to a member it does not hold")
}

macro_rules! scalar {
    ($t:ty, $shape:ident, $is:expr) => {
        impl Shaped for $t {
            fn shape(&self) -> Shape {
                Shape::$shape
            }
        }
        impl ToValue for $t {
            #[inline]
            fn to_value(self) -> Value {
                Value::of_word(crate::Trace::store(self))
            }
        }
        impl FromValue for $t {
            #[inline]
            fn from_value(v: Value) -> Self {
                if ($is)(v) {
                    <$t as crate::Trace>::load(v.0)
                } else {
                    not_held()
                }
            }
        }
    };
}

scalar!(i64, Int, |v: Value| v.is_int());
scalar!(Int, Int, |v: Value| v.is_int());
scalar!(f64, F64, |v: Value| v.kind() == Some(kind::FLONUM));
scalar!(char, Char, |v: Value| v.0.immediate()
    == Some(crate::obj::IMM_CHAR));
scalar!(bool, Bool, |v: Value| v.0.raw()
    == HeapWord::boolean(true).raw()
    || v.0.raw() == HeapWord::boolean(false).raw());
scalar!(Str, String, |v: Value| v.kind() == Some(kind::STRING));
scalar!(Symbol, Symbol, |v: Value| v.kind() == Some(kind::SYMBOL));

impl Shaped for () {
    fn shape(&self) -> Shape {
        Shape::Symbol
    }
}
impl ToValue for () {
    fn to_value(self) -> Value {
        Value::of_word(HeapWord::UNIT)
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

/// A ref, as its box's word.
impl<T: crate::Trace + 'static> ToValue for crate::Ref<T> {
    fn to_value(self) -> Value {
        other(self, Shape::Box)
    }
}

/// A member of another shape (a datatype's value, a sum, a product, a
/// reference, a procedure, an array) given for its union: its word.
pub fn other<T: Member>(x: T, shape: Shape) -> Value {
    x.into_value(shape)
}

/// A list given for its union: its word, where its elements' are values'
/// (C506: a typed list a list of values as it is); else a copy, its
/// elements given each.
pub fn list<T: ToValue + crate::Trace + 'static>(xs: List<T>) -> Value {
    if T::VALUE_WORD {
        return Value::of_word(crate::Trace::store(xs));
    }
    // Values in a `Vec` of its own, which no root sees, across allocations.
    let _held = crate::obj::space::NoCollect::new();
    let mut items = Vec::new();
    let mut p = xs;
    while let Some(c) = p.pair() {
        items.push(c.car().to_value());
        p = c.cdr;
    }
    items
        .into_iter()
        .rev()
        .fold(Value::Nil, |tail, x| cons(x, tail))
}

/// A list at `acyclic` given as a list of a union (C78): itself, where its
/// elements' words are values' (C506); else copied, its elements given
/// each. Frozen, it is the same list as far as a program can tell.
pub fn boxed_list<T: ToValue + crate::Trace>(xs: List<T>) -> List<Value> {
    if T::VALUE_WORD {
        return <List<Value> as crate::Trace>::load(crate::Trace::store(xs));
    }
    // Values in a `Vec` of its own, which no root sees, across allocations.
    let _held = crate::obj::space::NoCollect::new();
    let mut items = Vec::new();
    let mut p = xs;
    while let Some(c) = p.pair() {
        items.push(c.car().to_value());
        p = c.cdr;
    }
    items
        .into_iter()
        .rev()
        .fold(crate::ListAt::NIL, |tail, x| crate::heap_cons(x, tail))
}

/// `cons` where a pair of a union is wanted: Fixpt's pair cell of its
/// parts' words.
pub fn cons<A: ToValue, B: ToValue>(a: A, b: B) -> Value {
    Value::of_word(crate::obj::value_cons(a.to_value(), b.to_value().0))
}

/// A quasiquote's `,@xs`: the list's elements, given, before `rest`.
pub fn spliced<T: ToValue + crate::Trace>(xs: List<T>, rest: Value) -> Value {
    // Values in a `Vec` of its own, which no root sees, across allocations.
    let _held = crate::obj::space::NoCollect::new();
    let mut items = Vec::new();
    let mut p = xs;
    while let Some(c) = p.pair() {
        items.push(c.car().to_value());
        p = c.cdr;
    }
    items.into_iter().rev().fold(rest, |tail, x| cons(x, tail))
}

/// `car` and `cdr` of a union with a pair: checked, as FX-26's every machine
/// checks.
pub fn car(v: Value) -> Value {
    match v.parts() {
        Some((a, _)) => a,
        None => crate::fail("car of no pair"),
    }
}
pub fn cdr(v: Value) -> Value {
    match v.parts() {
        Some((_, d)) => d,
        None => crate::fail("cdr of no pair"),
    }
}

/// A union narrowed to a scalar member.
pub fn unbox<T: FromValue>(v: Value) -> T {
    T::from_value(v)
}

/// A union narrowed to a member of another shape: found again by its kind.
pub fn unbox_other<T: Member>(v: Value) -> T {
    match T::of_value(v) {
        Some(t) => t,
        None => not_held(),
    }
}

/// A union narrowed to a list member: `nil`, or the list its pair begins,
/// its word as it is (its elements' words values', C506).
pub fn unbox_list<T: crate::Trace + 'static>(v: Value) -> List<T> {
    if v.is_nil() {
        crate::ListAt::NIL
    } else if v.is_pair() && T::VALUE_WORD {
        <List<T> as crate::Trace>::load(v.0)
    } else {
        not_held()
    }
}

/// `datum-list->vector`: a datum's list, its elements in a vector.
pub fn datum_list_to_vector(d: Value) -> Value {
    // Values in a `Vec` of its own, which no root sees, across allocations.
    let _held = crate::obj::space::NoCollect::new();
    let mut v = Vec::new();
    let mut p = d;
    while let Some((a, rest)) = p.parts() {
        v.push(a);
        p = rest;
    }
    let v: Vector = crate::obj::array_made_of(KIND_VECTOR, v.into_iter());
    other(v, Shape::Vector)
}

/// Fixpt's vector's kind, a datum's vector's.
const KIND_VECTOR: u8 = kind::VECTOR;

impl Shaped for Value {
    fn shape(&self) -> Shape {
        let w = self.0;
        if w.tag() == 0 {
            return Shape::Int;
        }
        if w.is_pair() {
            return Shape::Pair;
        }
        if let Some(sub) = w.immediate() {
            return match sub {
                crate::obj::IMM_CHAR => Shape::Char,
                crate::obj::IMM_F32 => Shape::F32,
                _ if w.raw() == HeapWord::NULL.raw() => Shape::Nil,
                _ if w.raw() == HeapWord::UNIT.raw() => Shape::Symbol,
                _ => Shape::Bool,
            };
        }
        match self.kind() {
            Some(kind::STRING) => Shape::String,
            Some(kind::SYMBOL) => Shape::Symbol,
            Some(kind::VECTOR) => Shape::Vector,
            Some(kind::BYTEVECTOR) => Shape::Bytevector,
            Some(kind::FLONUM) => Shape::F64,
            Some(kind::BIGNUM) => Shape::Int,
            Some(kind::BOX) => Shape::Box,
            Some(kind::SUM) => Shape::Sum,
            Some(kind::PRODUCT) => Shape::Product,
            Some(kind::NATIVE_CLOSURE) => Shape::Procedure,
            Some(kind::BLOBLET | kind::FLAT_ARRAY) => Shape::Bloblet,
            _ => crate::fail("a value of no shape kayak-rt knows"),
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
        if self.is_nil() {
            Shape::Nil
        } else {
            Shape::Pair
        }
    }
}

impl<T: crate::Trace> Shaped for crate::RefAt<'_, T> {
    fn shape(&self) -> Shape {
        Shape::Box
    }
}

impl Shaped for f32 {
    fn shape(&self) -> Shape {
        Shape::F32
    }
}

/// The shape tests, FX-26's, of a value of any type: a union's by its word,
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

/// A dynamic value's pair, reached from a value (C506): its head and tail
/// values traced, its cell counted once.
struct PairAt(HeapWord);

impl crate::Visit for PairAt {
    fn visit<'a>(&'a self, t: &mut crate::Tracer<'a>) {
        self.visit_owned(t);
    }
    fn visit_owned(&self, t: &mut crate::Tracer<'_>) {
        let (a, d) = self.0.pair_words();
        crate::Trace::trace(Value::of_word(a), t);
        crate::Trace::trace(Value::of_word(d), t);
    }
    fn visit_size(&self) -> usize {
        16
    }
    fn visit_name(&self) -> &'static str {
        "kayak_rt::value::Pair"
    }
}

/// A dynamic value (C491, C506): what it holds reached, precisely, by its
/// word: a pair's parts, as values; a string's text; a symbol, a bignum, a
/// `sumof`'s and a closure's objects, by what each says of its own type
/// (a sum's payload's type, a closure's code). An object whose fields'
/// types only its type knows (an array, a box, a product, a bloblet, a
/// datatype's value) told the tracer by its header's address, which a
/// census finds the block of, and traces as its type has it.
impl crate::Trace for Value {
    type Stored = HeapWord;
    const VALUE_WORD: bool = true;
    #[inline(always)]
    fn store(self) -> HeapWord {
        self.0
    }
    #[inline(always)]
    fn load(w: HeapWord) -> Self {
        Value::of_word(w)
    }
    fn trace<'a>(self, t: &mut crate::Tracer<'a>)
    where
        Self: 'a,
    {
        let w = self.0;
        if w.is_pair() {
            t.reach_value(w.raw() & !7, PairAt(w));
            return;
        }
        let Some(h) = w.object_header() else {
            return;
        };
        match ((h >> 3) & 0xff) as u8 {
            kind::STRING => w.to_str().trace(t),
            kind::SYMBOL => crate::Symbol::of_word(w).trace(t),
            kind::BIGNUM => w.to_int().trace(t),
            kind::FLONUM => {}
            kind::SUM if (h >> 14) & 0x3ffff == 4 => crate::Sum::of_word(w).trace(t),
            kind::NATIVE_CLOSURE => crate::obj::procs::reach_closure(w, t),
            _ => t.opaque(w.header_addr()),
        }
    }
}

/// Written as Fixpt writes the member it holds.
impl Show for Value {
    fn show(&self, out: &mut String) {
        let v = *self;
        match v.shape() {
            Shape::Int => Int::from_value(v).show(out),
            Shape::F64 => f64::from_value(v).show(out),
            Shape::Char => char::from_value(v).show(out),
            Shape::Bool => bool::from_value(v).show(out),
            Shape::String => Str::from_value(v).show(out),
            Shape::Symbol if v.0.raw() == HeapWord::UNIT.raw() => ().show(out),
            Shape::Symbol => Symbol::from_value(v).show(out),
            Shape::Nil => out.push_str("()"),
            Shape::Pair => {
                out.push('(');
                car(v).show(out);
                let mut rest = cdr(v);
                loop {
                    if rest.is_nil() {
                        break;
                    }
                    match rest.parts() {
                        Some((a, d)) => {
                            out.push(' ');
                            a.show(out);
                            rest = d;
                        }
                        None => {
                            out.push_str(" . ");
                            rest.show(out);
                            break;
                        }
                    }
                }
                out.push(')');
            }
            Shape::Vector => {
                out.push_str("#(");
                if let Some(a) = Vector::of_value(v) {
                    for (i, e) in a.cells().iter().enumerate() {
                        if i > 0 {
                            out.push(' ');
                        }
                        e.get().show(out);
                    }
                }
                out.push(')');
            }
            Shape::Sum => match crate::Sum::of_value(v) {
                Some(s) => s.show(out),
                None => out.push_str("#<value>"),
            },
            Shape::Procedure => out.push_str("#<procedure>"),
            _ => out.push_str("#<value>"),
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
        let xs: List<i64> = crate::cons(1, crate::nil());
        let l = list(xs);
        assert!(pair_p(l) && !null_p(l) && pair_p(xs));
        assert_eq!(crate::car(unbox_list::<i64>(l)), 1);
        assert_eq!(i64::from_value(car(l)), 1);
        assert!(null_p(cdr(l)));
        assert!(null_p(list::<i64>(crate::nil())));
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
        // A datum's vector Fixpt's (C505): an array of its elements.
        let v = datum_list_to_vector(d);
        assert!(vector_p(v));
        let a = unbox_other::<Vector>(v);
        assert_eq!(a.len(), 2);
        assert_eq!(i64::from_value(a.cells()[0].get()), 1);
        let mut out = String::new();
        v.show(&mut out);
        assert_eq!(out, "#(1 \"a\")");
    }

    /// One word (C506), as Fixpt's: each member's own, its shape its
    /// tag's or its object's kind's.
    #[test]
    fn one_word_its_shape_its_own() {
        assert_eq!(std::mem::size_of::<Value>(), 8);
        let sym = crate::sym_lit!("x").to_value();
        assert!(symbol_p(sym) && !string_p(sym));
        assert!(symbol_p(().to_value()));
        assert!(char_p('c'.to_value()) && char::from_value('c'.to_value()) == 'c');
        assert!(bool_p(false.to_value()) && !bool::from_value(false.to_value()));
        assert!(f64_p(2.5f64.to_value()) && f64::from_value(2.5f64.to_value()) == 2.5);
        let big = crate::int_text("123456789012345678901234567890");
        assert!(int_p(big.to_value()));
        // A typed list a list of values as it is: the same word.
        let xs: List<Str> = crate::cons(str_lit("a"), crate::nil());
        assert_eq!(list(xs).word().raw(), crate::Trace::store(xs).raw());
        // A value's list narrowed to a typed one: its elements read so.
        let ys = cons(1i64, cons(2i64, Value::Nil));
        let zs = unbox_list::<i64>(ys);
        assert_eq!(crate::car(crate::cdr(zs)), 2);
        // An array a bloblet's shape, a datum's vector a vector's.
        let arr = crate::make_array(crate::int_lit(2), 7i64);
        let av = other(arr, Shape::Bloblet);
        assert!(array_p(av) && !vector_p(av));
        assert_eq!(unbox_other::<crate::Array<i64>>(av).cells()[1].get(), 7);
        // A ref's box, kind 10.
        let r: crate::Ref<i64> = crate::heap_new(5);
        assert!(ref_p(r.to_value()));
        assert!(unbox_other::<crate::Ref<i64>>(r.to_value()).same(r));
    }

    #[test]
    fn a_member_its_own_object_and_traced() {
        // A sum given for a union: the union holds its word.
        let s = crate::sum_of(crate::tag_lit!("pair"), 7i64);
        let v = other(s, Shape::Sum);
        assert!(sum_p(v));
        assert_eq!(unbox_other::<crate::Sum>(v).addr(), s.addr());
        // A ref: its box.
        let r: crate::Ref<i64> = crate::heap_new(5);
        assert!(unbox_other::<crate::Ref<i64>>(r.to_value()).same(r));
        // A datum of a string, the sum and the ref: the pairs and the sum
        // reached precisely; the box, whose cell's type only it knows,
        // told by its header's address, which a census traces by its block.
        let d = cons(str_lit("a"), cons(v, cons(r.to_value(), Value::Nil)));
        let mut t = crate::traced(d);
        let opaque = t.take_opaque();
        assert_eq!(opaque.len(), 1);
        let (rows, strings) = t.census();
        let of = |name: &str| {
            rows.iter()
                .filter(|r| r.0.contains(name))
                .map(|r| r.1)
                .sum::<u64>()
        };
        assert_eq!(of("value::Pair"), 3);
        assert_eq!(of("SumOf<i64>"), 1);
        assert_eq!(strings.0, 1);
    }
}
