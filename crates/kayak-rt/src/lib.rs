//! What the Rust that KFX-0 generates calls (`PLAN.md` §2).
//!
//! Every KFX value is a `Copy` Rust value: a scalar, or a `&'static`
//! reference to memory that is never freed. Each primitive is named by the
//! mangling of its FX-26 name: `string->list` is `string_to_list`, `char=?`
//! is `char_eq_p`, `i64+` is `i64_plus`.
//!
//! Strings are ASCII, so that `string-ref` is an index; a byte past ASCII
//! is an error where it is read.

pub type Str = &'static str;

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
    Box::leak(s.into_boxed_str())
}

/// FX-26's `error`: a run-time error, which ends the run.
pub fn error(message: Str) -> ! {
    panic!("{message}")
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
        None => error("car: nil"),
    }
}

pub fn cdr<T>(xs: List<T>) -> List<T> {
    match xs {
        Some(p) => p.cdr,
        None => error("cdr: nil"),
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
    if b == 0 { error("i64-quotient: division by zero") } else { a.wrapping_div(b) }
}

pub fn i64_remainder(a: i64, b: i64) -> i64 {
    if b == 0 { error("i64-remainder: division by zero") } else { a.wrapping_rem(b) }
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

pub fn string_length_i64(s: Str) -> i64 {
    s.len() as i64
}

pub fn string_ref_i64(s: Str, i: i64) -> char {
    match usize::try_from(i).ok().and_then(|i| s.as_bytes().get(i)) {
        Some(&b) if b.is_ascii() => b as char,
        Some(_) => error("string-ref: not ASCII"),
        None => error("string-ref: out of range"),
    }
}

pub fn char_to_i64(c: char) -> i64 {
    c as i64
}

pub fn i64_to_string(n: i64) -> Str {
    leak_string(n.to_string())
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
