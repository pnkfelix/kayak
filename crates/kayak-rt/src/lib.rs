//! What the Rust that KFX-0 generates calls (`PLAN.md` §2).
//!
//! Every KFX value is a `Copy` Rust value: a scalar, or a `&'static`
//! reference to memory that is never freed. Each primitive is named by the
//! mangling of its FX-26 name: `string->list` is `string_to_list`, `char=?`
//! is `char_eq_p`, `i64+` is `i64_plus`.
//!
//! Strings are of characters, as FX-26's are: a `Str` knows whether it is
//! ASCII, so that indexing one that is costs nothing, and indexing one that
//! is not counts characters (`CLAUDES_CHOICES.md`, C14).

mod float;
mod int;
mod show;

pub use float::*;
pub use int::*;
pub use show::*;
use std::cell::Cell;

/// `string`: text, and whether it is ASCII, found when the string is made.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Str {
    text: &'static str,
    ascii: bool,
}

impl Str {
    pub fn as_str(self) -> &'static str {
        self.text
    }
}

impl std::fmt::Display for Str {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.text)
    }
}

impl std::fmt::Debug for Str {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.text)
    }
}

/// `(ref T R)`: a cell, which any alias may read and write, as FX-26's may.
pub type Ref<T> = &'static Cell<T>;

/// `(flatarrayof T R)`: a flat array, raw elements in a row, each a cell.
pub type FlatArray<T> = &'static [Cell<T>];

pub struct Cons<T: 'static> {
    pub car: T,
    pub cdr: List<T>,
}

/// `(listof T R)`: `nil`, or a pair whose tail is the list again.
pub type List<T> = Option<&'static Cons<T>>;

/// Allocation: the heap is never freed.
pub fn alloc<T>(v: T) -> &'static T {
    Box::leak(Box::new(v))
}

pub fn leak_string(s: String) -> Str {
    let ascii = s.is_ascii();
    Str {
        text: Box::leak(s.into_boxed_str()),
        ascii,
    }
}

/// FX-26's `error`: a run-time error, which ends the run.
pub fn error(message: Str) -> ! {
    panic!("{}", message.text)
}

/// A run-time error of the runtime's own, as `error` is a program's.
pub(crate) fn fail(message: &str) -> ! {
    panic!("{message}")
}

// Procedures: a `lambda` that is a value is a closure, made a
// `&'static dyn Fn` by the helper of its arity, so that its parameters'
// and result's types are found from where it is used.

pub fn lambda0<R>(f: impl Fn() -> R + 'static) -> &'static dyn Fn() -> R {
    alloc(f)
}

pub fn lambda1<A, R>(f: impl Fn(A) -> R + 'static) -> &'static dyn Fn(A) -> R {
    alloc(f)
}

pub fn lambda2<A, B, R>(f: impl Fn(A, B) -> R + 'static) -> &'static dyn Fn(A, B) -> R {
    alloc(f)
}

pub fn lambda3<A, B, C, R>(f: impl Fn(A, B, C) -> R + 'static) -> &'static dyn Fn(A, B, C) -> R {
    alloc(f)
}

pub fn lambda4<A, B, C, D, R>(
    f: impl Fn(A, B, C, D) -> R + 'static,
) -> &'static dyn Fn(A, B, C, D) -> R {
    alloc(f)
}

/// A `letrec`'s knot: a procedure's place, empty until the procedure is
/// made, so that the procedures of a group, closures that cannot name
/// themselves, can name each other through it. FX-26's `letrec` binds only
/// lambdas, so no knot is read before it is tied.
pub struct Knot<T>(Cell<Option<T>>);

impl<T: Copy> Knot<T> {
    pub fn get(&self) -> T {
        self.0
            .get()
            .unwrap_or_else(|| fail("letrec: a procedure used before it is made"))
    }

    pub fn set(&self, v: T) {
        self.0.set(Some(v))
    }
}

pub fn knot<T: Copy + 'static>() -> &'static Knot<T> {
    alloc(Knot(Cell::new(None)))
}

// Lists.

pub fn nil<T>() -> List<T> {
    None
}

pub fn cons<T>(car: T, cdr: List<T>) -> List<T> {
    Some(alloc(Cons { car, cdr }))
}

pub fn car<T: Copy>(xs: List<T>) -> T {
    match xs {
        Some(p) => p.car,
        None => fail("car: nil"),
    }
}

pub fn cdr<T>(xs: List<T>) -> List<T> {
    match xs {
        Some(p) => p.cdr,
        None => fail("cdr: nil"),
    }
}

pub fn null_p<T>(xs: List<T>) -> bool {
    xs.is_none()
}

// Booleans.

pub fn not(b: bool) -> bool {
    !b
}

// `i64`: FX-26's fixed-width arithmetic wraps, as Rust's `wrapping_*`
// does: `MIN / -1` is `MIN`, and a shift counts modulo 64. Division by zero
// is an error in both.

pub fn i64_plus(a: i64, b: i64) -> i64 {
    a.wrapping_add(b)
}

pub fn i64_minus(a: i64, b: i64) -> i64 {
    a.wrapping_sub(b)
}

pub fn i64_star(a: i64, b: i64) -> i64 {
    a.wrapping_mul(b)
}

pub fn i64_quotient(a: i64, b: i64) -> i64 {
    if b == 0 {
        fail("i64-quotient: division by zero")
    } else {
        a.wrapping_div(b)
    }
}

pub fn i64_remainder(a: i64, b: i64) -> i64 {
    if b == 0 {
        fail("i64-remainder: division by zero")
    } else {
        a.wrapping_rem(b)
    }
}

pub fn i64_and(a: i64, b: i64) -> i64 {
    a & b
}

pub fn i64_or(a: i64, b: i64) -> i64 {
    a | b
}

pub fn i64_xor(a: i64, b: i64) -> i64 {
    a ^ b
}

pub fn i64_not(a: i64) -> i64 {
    !a
}

pub fn i64_eq(a: i64, b: i64) -> bool {
    a == b
}

pub fn i64_lt(a: i64, b: i64) -> bool {
    a < b
}

pub fn i64_gt(a: i64, b: i64) -> bool {
    a > b
}

pub fn i64_lt_eq(a: i64, b: i64) -> bool {
    a <= b
}

pub fn i64_gt_eq(a: i64, b: i64) -> bool {
    a >= b
}

// The prelude (`kfx/prelude.fx`): FX-26's `int` primitives, at `i64`.

/// Characters, as FX-26 counts: bytes, if the string is ASCII.
pub fn string_length_i64(s: Str) -> i64 {
    if s.ascii {
        s.text.len() as i64
    } else {
        s.text.chars().count() as i64
    }
}

/// The `i`th character: at once if the string is ASCII, else counted to.
pub fn string_ref_i64(s: Str, i: i64) -> char {
    let found = usize::try_from(i).ok().and_then(|i| {
        if s.ascii {
            s.text.as_bytes().get(i).map(|&b| b as char)
        } else {
            s.text.chars().nth(i)
        }
    });
    found.unwrap_or_else(|| fail("string-ref: out of range"))
}

pub fn char_to_i64(c: char) -> i64 {
    c as i64
}

pub fn i64_to_string(n: i64) -> Str {
    leak_string(n.to_string())
}

// Mutable data: references and flat arrays. Regions are erased (every
// one's place is the heap, so every reference `'static`); effects too.

pub fn new<T: Copy>(v: T) -> Ref<T> {
    alloc(Cell::new(v))
}

pub fn get<T: Copy>(r: Ref<T>) -> T {
    r.get()
}

pub fn set<T: Copy>(r: Ref<T>, v: T) {
    r.set(v)
}

pub fn make_flatarray_i64(n: i64, v: i64) -> FlatArray<i64> {
    match usize::try_from(n) {
        Ok(n) => Box::leak(
            (0..n)
                .map(|_| Cell::new(v))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        ),
        Err(_) => fail("make-flatarray: a negative length"),
    }
}

fn element<T>(a: &'static [Cell<T>], i: i64) -> &'static Cell<T> {
    match usize::try_from(i).ok().and_then(|i| a.get(i)) {
        Some(c) => c,
        None => fail("flatarray: index out of range"),
    }
}

pub fn flatarray_ref_i64(a: FlatArray<i64>, i: i64) -> i64 {
    element(a, i).get()
}

pub fn flatarray_set_i64_x(a: FlatArray<i64>, i: i64, v: i64) {
    element(a, i).set(v)
}

pub fn flatarray_length_i64(a: FlatArray<i64>) -> i64 {
    a.len() as i64
}

// Characters.

pub fn char_eq_p(a: char, b: char) -> bool {
    a == b
}

pub fn char_whitespace_p(c: char) -> bool {
    c.is_whitespace()
}

pub fn char_numeric_p(c: char) -> bool {
    c.is_ascii_digit()
}

pub fn char_alphabetic_p(c: char) -> bool {
    c.is_alphabetic()
}

pub fn char_upcase(c: char) -> char {
    c.to_ascii_uppercase()
}

pub fn char_to_string(c: char) -> Str {
    leak_string(c.to_string())
}

// Strings.

pub fn string_eq_p(a: Str, b: Str) -> bool {
    a == b
}

pub fn string_append(a: Str, b: Str) -> Str {
    leak_string(format!("{a}{b}"))
}

/// `nat`s, which are `Int`s: a string's length in characters, a list's.
pub fn string_length(s: Str) -> Int {
    i64_to_int(string_length_i64(s))
}

pub fn length<T>(xs: List<T>) -> Int {
    list_length(xs)
}

pub fn list_length<T>(mut xs: List<T>) -> Int {
    let mut n = 0i64;
    while let Some(p) = xs {
        n += 1;
        xs = p.cdr;
    }
    i64_to_int(n)
}

pub fn reverse<T: Copy>(mut xs: List<T>) -> List<T> {
    let mut out = nil();
    while let Some(p) = xs {
        out = cons(p.car, out);
        xs = p.cdr;
    }
    out
}

pub fn append<T: Copy>(xs: List<T>, ys: List<T>) -> List<T> {
    let mut items = Vec::new();
    let mut p = xs;
    while let Some(c) = p {
        items.push(c.car);
        p = c.cdr;
    }
    items.into_iter().rev().fold(ys, |rest, x| cons(x, rest))
}

/// A string literal: `const`, so that a literal in a loop costs nothing.
pub const fn str_lit(text: &'static str) -> Str {
    Str {
        text,
        ascii: text.is_ascii(),
    }
}

pub fn string_to_list(s: Str) -> List<char> {
    s.text.chars().rev().fold(nil(), |rest, c| cons(c, rest))
}

pub fn list_to_string(mut cs: List<char>) -> Str {
    let mut s = String::new();
    while let Some(p) = cs {
        s.push(p.car);
        cs = p.cdr;
    }
    leak_string(s)
}

/// Runs `f` on a thread with a stack of 1 GiB: KFX-0 has no loops, and
/// Rust no tail calls, so a walk of a long list is as deep as the list.
pub fn with_big_stack<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(1 << 30)
        .spawn(f)
        .expect("a thread for the run")
        .join()
        .unwrap_or_else(|_| std::process::exit(1))
}
