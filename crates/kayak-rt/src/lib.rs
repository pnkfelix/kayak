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

mod arena;
mod array;
mod blob;
mod eqtable;
mod escape;
mod fixed;
mod float;
mod heap;
mod int;
pub mod marks;
mod native;
mod once;
pub mod process;
mod show;
mod stats;
pub mod suspend;
mod text;
pub mod value;
mod world;

pub use arena::*;
pub use array::*;
pub use blob::*;
pub use eqtable::*;
pub use escape::*;
pub use fixed::*;
pub use float::*;
pub use heap::*;
pub use int::*;
#[cfg(feature = "unsafe-par")]
pub use kayak_sync::Global;
pub use marks::{MarkKey, first_mark, make_continuation_mark_key, with_mark, with_mark_async};
pub use native::*;
pub use once::*;
pub use process::{Addr, ICell, Inbox, icell_get, icell_put_x, make_icell};
pub use show::*;
pub use stats::heap_stats_report;
use std::cell::Cell;
pub use std::cell::OnceCell;
pub use text::*;
pub use value::{
    array_p, bool_p, bytevector_p, char_p, datum_list_to_vector, f32_p, f64_p, int_p, pair_p,
    procedure_p, product_p, ref_p, string_p, sum_p, symbol_p, vector_p,
};
pub use world::*;

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

/// `(ref T R)`: a cell, which any alias may read and write, as FX-26's may;
/// in a `letrena`'s region, `RefAt<'r, T>`, borrowed from its arena, and
/// in the heap, which is never freed, `Ref<T>`.
pub type RefAt<'r, T> = &'r Cell<T>;
pub type Ref<T> = RefAt<'static, T>;

/// `(flatarrayof T R)`: a flat array, raw elements in a row, each a cell.
/// An `i64` one has Fixpt's flat array's header, element code and trailer
/// in the three cells before its first, so that native code is given it
/// as itself, shared (C126; `native.rs`).
pub type FlatArray<T> = &'static [Cell<T>];

pub struct Cons<'r, T> {
    pub car: T,
    pub cdr: ListAt<'r, T>,
}

/// `(listof T R)`: `nil`, or a pair whose tail is the list again; in a
/// `letrena`'s region, `ListAt<'r, T>`, and in the heap, `List<T>`.
pub type ListAt<'r, T> = Option<&'r Cons<'r, T>>;
pub type List<T> = ListAt<'static, T>;

/// Allocation: the heap is never freed, so what it holds lives as long as
/// anything wants, `'static` or a region's `'r`.
#[inline]
pub fn alloc<'a, T: 'a>(v: T) -> &'a T {
    stats::note::<T>(std::mem::size_of::<T>());
    Box::leak(Box::new(v))
}

#[inline]
pub fn leak_string(s: String) -> Str {
    let ascii = s.is_ascii();
    stats::note::<Str>(s.len());
    Str {
        text: Box::leak(s.into_boxed_str()),
        ascii,
    }
}

/// FX-26's `error`: a run-time error, which ends the run.
/// FX-26's `void`, the type of no value, as Rust's stable empty type: an
/// enum of no variants. Where a `void` is made (`error`, which returns
/// `!`), `!` coerces to it; nothing else makes one.
#[derive(Clone, Copy)]
pub enum Void {}

#[inline]
pub fn error(message: Str) -> ! {
    fail(message.text)
}

/// A run-time error of the runtime's own, as `error` is a program's: a
/// panic, which the program's hook says; in a monitored process (C108),
/// one no hook sees, since its watcher is told instead.
pub(crate) fn fail(message: &str) -> ! {
    if process::monitored() {
        std::panic::resume_unwind(Box::new(message.to_string()))
    }
    panic!("{message}")
}

// Procedures: a `lambda` that is a value is a closure, made a
// `&'static dyn Fn` by the helper of its arity, so that its parameters'
// and result's types are found from where it is used.

#[inline]
pub fn lambda0<'a, R>(f: impl Fn() -> R + 'a) -> &'a dyn Fn() -> R {
    alloc(f)
}

#[inline]
pub fn lambda1<'a, A, R>(f: impl Fn(A) -> R + 'a) -> &'a dyn Fn(A) -> R {
    alloc(f)
}

#[inline]
pub fn lambda2<'a, A, B, R>(f: impl Fn(A, B) -> R + 'a) -> &'a dyn Fn(A, B) -> R {
    alloc(f)
}

#[inline]
pub fn lambda3<'a, A, B, C, R>(f: impl Fn(A, B, C) -> R + 'a) -> &'a dyn Fn(A, B, C) -> R {
    alloc(f)
}

#[inline]
pub fn lambda4<'a, A, B, C, D, R>(f: impl Fn(A, B, C, D) -> R + 'a) -> &'a dyn Fn(A, B, C, D) -> R {
    alloc(f)
}

/// A procedure of an effect a process may have (C102): its closure
/// `Send + Sync`, which rustc checks of what it holds, so that it may be
/// held on another thread.
#[inline]
pub fn lambda0s<'a, R>(f: impl Fn() -> R + Send + Sync + 'a) -> &'a (dyn Fn() -> R + Send + Sync) {
    alloc(f)
}

#[inline]
pub fn lambda1s<'a, A, R>(
    f: impl Fn(A) -> R + Send + Sync + 'a,
) -> &'a (dyn Fn(A) -> R + Send + Sync) {
    alloc(f)
}

#[inline]
pub fn lambda2s<'a, A, B, R>(
    f: impl Fn(A, B) -> R + Send + Sync + 'a,
) -> &'a (dyn Fn(A, B) -> R + Send + Sync) {
    alloc(f)
}

#[inline]
pub fn lambda3s<'a, A, B, C, R>(
    f: impl Fn(A, B, C) -> R + Send + Sync + 'a,
) -> &'a (dyn Fn(A, B, C) -> R + Send + Sync) {
    alloc(f)
}

#[inline]
pub fn lambda4s<'a, A, B, C, D, R>(
    f: impl Fn(A, B, C, D) -> R + Send + Sync + 'a,
) -> &'a (dyn Fn(A, B, C, D) -> R + Send + Sync) {
    alloc(f)
}

/// A `letrec`'s knot: a procedure's place, empty until the procedure is
/// made, so that the procedures of a group, closures that cannot name
/// themselves, can name each other through it. FX-26's `letrec` binds only
/// lambdas, so no knot is read before it is tied; tied once, it is `Sync`
/// when the procedure is (C102), so a closure holding it may cross.
pub struct Knot<T>(std::sync::OnceLock<T>);

impl<T: Copy> Knot<T> {
    pub fn get(&self) -> T {
        *self
            .0
            .get()
            .unwrap_or_else(|| fail("letrec: a procedure used before it is made"))
    }

    pub fn set(&self, v: T) {
        if self.0.set(v).is_err() {
            fail("letrec: a procedure made twice")
        }
    }
}

#[inline]
pub fn knot<'a, T: Copy + 'a>() -> &'a Knot<T> {
    alloc(Knot(std::sync::OnceLock::new()))
}

// Lists.

#[inline]
pub fn nil<'r, T>() -> ListAt<'r, T> {
    None
}

#[inline]
pub fn cons<'r, T>(car: T, cdr: ListAt<'r, T>) -> ListAt<'r, T> {
    Some(alloc(Cons { car, cdr }))
}

#[inline]
pub fn car<T: Copy>(xs: ListAt<'_, T>) -> T {
    match xs {
        Some(p) => p.car,
        None => fail("car: nil"),
    }
}

#[inline]
pub fn cdr<'r, T>(xs: ListAt<'r, T>) -> ListAt<'r, T> {
    match xs {
        Some(p) => p.cdr,
        None => fail("cdr: nil"),
    }
}

#[inline]
pub fn null_p<T>(xs: ListAt<'_, T>) -> bool {
    xs.is_none()
}

// Booleans.

#[inline]
pub fn not(b: bool) -> bool {
    !b
}

// `i64`: FX-26's fixed-width arithmetic wraps, as Rust's `wrapping_*`
// does: `MIN / -1` is `MIN`, and a shift counts modulo 64. Division by zero
// is an error in both.

#[inline]
pub fn i64_plus(a: i64, b: i64) -> i64 {
    a.wrapping_add(b)
}

#[inline]
pub fn i64_minus(a: i64, b: i64) -> i64 {
    a.wrapping_sub(b)
}

#[inline]
pub fn i64_star(a: i64, b: i64) -> i64 {
    a.wrapping_mul(b)
}

#[inline]
pub fn i64_quotient(a: i64, b: i64) -> i64 {
    if b == 0 {
        fail("i64-quotient: division by zero")
    } else {
        a.wrapping_div(b)
    }
}

#[inline]
pub fn i64_remainder(a: i64, b: i64) -> i64 {
    if b == 0 {
        fail("i64-remainder: division by zero")
    } else {
        a.wrapping_rem(b)
    }
}

#[inline]
pub fn i64_and(a: i64, b: i64) -> i64 {
    a & b
}

#[inline]
pub fn i64_or(a: i64, b: i64) -> i64 {
    a | b
}

#[inline]
pub fn i64_xor(a: i64, b: i64) -> i64 {
    a ^ b
}

#[inline]
pub fn i64_not(a: i64) -> i64 {
    !a
}

#[inline]
pub fn i64_eq(a: i64, b: i64) -> bool {
    a == b
}

#[inline]
pub fn i64_lt(a: i64, b: i64) -> bool {
    a < b
}

#[inline]
pub fn i64_gt(a: i64, b: i64) -> bool {
    a > b
}

#[inline]
pub fn i64_lt_eq(a: i64, b: i64) -> bool {
    a <= b
}

#[inline]
pub fn i64_gt_eq(a: i64, b: i64) -> bool {
    a >= b
}

// The prelude (`kfx/prelude.fx`): FX-26's `int` primitives, at `i64`.

/// Characters, as FX-26 counts: bytes, if the string is ASCII.
#[inline]
pub fn string_length_i64(s: Str) -> i64 {
    if s.ascii {
        s.text.len() as i64
    } else {
        s.text.chars().count() as i64
    }
}

/// The `i`th character: at once if the string is ASCII, else counted to.
#[inline]
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

#[inline]
pub fn char_to_i64(c: char) -> i64 {
    c as i64
}

/// FX-26's `integer->char`, at `i64`: a code that is no character fails.
#[inline]
pub fn i64_to_char(n: i64) -> char {
    match u32::try_from(n).ok().and_then(char::from_u32) {
        Some(c) => c,
        None => fail("integer->char: no character has that code"),
    }
}

#[inline]
pub fn i64_to_string(n: i64) -> Str {
    leak_string(n.to_string())
}

// Mutable data: references and flat arrays. Regions are erased (every
// one's place is the heap, so every reference `'static`); effects too.

#[inline]
pub fn new<'r, T: Copy>(v: T) -> RefAt<'r, T> {
    alloc(Cell::new(v))
}

#[inline]
pub fn get<T: Copy>(r: RefAt<'_, T>) -> T {
    r.get()
}

#[inline]
pub fn set<T: Copy>(r: RefAt<'_, T>, v: T) {
    r.set(v)
}

/// Elements from which a flat array of 0s is reserved rather than written.
const BIG_FLATARRAY: usize = 1 << 20;

#[inline]
pub fn make_flatarray_i64(n: i64, v: i64) -> FlatArray<i64> {
    use kayak_jit::layout as l;
    match usize::try_from(n) {
        Ok(n) => {
            let prefix = [
                l::header(l::KIND_FLAT_ARRAY, 2, 8 * n, false, false),
                l::FLAT_I64 << l::FIXNUM_SHIFT,
                2 << 3 | l::TAG_TRAILER,
            ];
            // A big one of 0s reserved, committed as it is touched (C139).
            if v == 0 && n >= BIG_FLATARRAY {
                let prefix = prefix.map(|w| w as i64);
                if let Some(all) = kayak_jit::zeroed_cells(&prefix, n) {
                    return &all[prefix.len()..];
                }
            }
            let all: &'static [Cell<i64>] = Box::leak(
                prefix
                    .iter()
                    .map(|&w| Cell::new(w as i64))
                    .chain((0..n).map(|_| Cell::new(v)))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            );
            &all[prefix.len()..]
        }
        Err(_) => fail("make-flatarray: a negative length"),
    }
}

fn element<T>(a: &'static [Cell<T>], i: i64) -> &'static Cell<T> {
    match usize::try_from(i).ok().and_then(|i| a.get(i)) {
        Some(c) => c,
        None => fail("flatarray: index out of range"),
    }
}

#[inline]
pub fn flatarray_ref_i64(a: FlatArray<i64>, i: i64) -> i64 {
    element(a, i).get()
}

#[inline]
pub fn flatarray_set_i64_x(a: FlatArray<i64>, i: i64, v: i64) {
    element(a, i).set(v)
}

#[inline]
pub fn flatarray_length_i64(a: FlatArray<i64>) -> i64 {
    a.len() as i64
}

// Characters.

#[inline]
pub fn char_eq_p(a: char, b: char) -> bool {
    a == b
}

#[inline]
pub fn char_whitespace_p(c: char) -> bool {
    c.is_whitespace()
}

/// Any numeric character, as Fixpt's (`²`, `½`, `٣` too).
#[inline]
pub fn char_numeric_p(c: char) -> bool {
    c.is_numeric()
}

#[inline]
pub fn char_alphabetic_p(c: char) -> bool {
    c.is_alphabetic()
}

/// Unicode's upper case, its first character where it is more than one
/// (`ß` is `S`), as Fixpt's.
#[inline]
pub fn char_upcase(c: char) -> char {
    c.to_uppercase().next().unwrap_or(c)
}

#[inline]
pub fn char_to_string(c: char) -> Str {
    leak_string(c.to_string())
}

// Strings.

#[inline]
pub fn string_eq_p(a: Str, b: Str) -> bool {
    a == b
}

#[inline]
pub fn string_append(a: Str, b: Str) -> Str {
    leak_string(format!("{a}{b}"))
}

/// `nat`s, which are `Int`s: a string's length in characters, a list's.
#[inline]
pub fn string_length(s: Str) -> Int {
    i64_to_int(string_length_i64(s))
}

/// `string-ref` at an `int`, as FX-26 has it.
#[inline]
pub fn string_ref(s: Str, i: Int) -> char {
    if _lt(i, int_lit(0)) || _gt(i, int_lit(i64::MAX)) {
        fail("string-ref: out of range")
    }
    string_ref_i64(s, int_to_i64(i))
}

#[inline]
pub fn length<T>(xs: ListAt<'_, T>) -> Int {
    list_length(xs)
}

#[inline]
pub fn list_length<T>(mut xs: ListAt<'_, T>) -> Int {
    let mut n = 0i64;
    while let Some(p) = xs {
        n += 1;
        xs = p.cdr;
    }
    i64_to_int(n)
}

#[inline]
pub fn reverse<'b, T: Copy + 'b>(mut xs: ListAt<'_, T>) -> ListAt<'b, T> {
    let mut out = nil();
    while let Some(p) = xs {
        out = cons(p.car, out);
        xs = p.cdr;
    }
    out
}

#[inline]
pub fn append<'b, T: Copy + 'b>(xs: ListAt<'_, T>, ys: ListAt<'b, T>) -> ListAt<'b, T> {
    let mut items = Vec::new();
    let mut p = xs;
    while let Some(c) = p {
        items.push(c.car);
        p = c.cdr;
    }
    items.into_iter().rev().fold(ys, |rest, x| cons(x, rest))
}

/// A string literal: `const`, so that a literal in a loop costs nothing.
#[inline]
pub const fn str_lit(text: &'static str) -> Str {
    Str {
        text,
        ascii: text.is_ascii(),
    }
}

#[inline]
pub fn string_to_list(s: Str) -> List<char> {
    s.text.chars().rev().fold(nil(), |rest, c| cons(c, rest))
}

#[inline]
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
#[inline]
pub fn with_big_stack<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(1 << 30)
        .spawn(f)
        .expect("a thread for the run")
        .join()
        .unwrap_or_else(|_| std::process::exit(1))
}
