//! FX-26's standard operations on booleans, characters, strings and
//! symbols beside those in `lib.rs` (Fixpt 17ac0c5's `standard.rs`, each as
//! its runtime has it; `CLAUDES_CHOICES.md` C56).

use crate::{Int, Show, Str, fail, i64_to_int, int_to_i64, leak_string};

/// FX-26's `char->integer` and `integer->char`, at `int`.
#[inline]
pub fn char_to_integer(c: char) -> Int {
    i64_to_int(c as i64)
}

pub fn integer_to_char(n: Int) -> char {
    crate::i64_to_char(int_to_i64(n))
}

#[inline]
pub fn bool_eq_p(a: bool, b: bool) -> bool {
    a == b
}

#[inline]
pub fn char_lt_p(a: char, b: char) -> bool {
    a < b
}
#[inline]
pub fn char_lt_eq_p(a: char, b: char) -> bool {
    a <= b
}
#[inline]
pub fn char_gt_p(a: char, b: char) -> bool {
    a > b
}
#[inline]
pub fn char_gt_eq_p(a: char, b: char) -> bool {
    a >= b
}

/// The first character of its lower case, or itself.
pub fn char_downcase(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

// Strings compare character by character, which is UTF-8's byte order.

#[inline]
pub fn string_lt_p(a: Str, b: Str) -> bool {
    a.as_str() < b.as_str()
}
#[inline]
pub fn string_lt_eq_p(a: Str, b: Str) -> bool {
    a.as_str() <= b.as_str()
}
#[inline]
pub fn string_gt_p(a: Str, b: Str) -> bool {
    a.as_str() > b.as_str()
}
#[inline]
pub fn string_gt_eq_p(a: Str, b: Str) -> bool {
    a.as_str() >= b.as_str()
}

/// -1, 0 or 1, as `a` comes before, is, or comes after `b`.
pub fn string_compare(a: Str, b: Str) -> Int {
    i64_to_int(a.as_str().cmp(b.as_str()) as i64)
}

/// Where `sub` first occurs in `s` at or after character `from`, in
/// characters; or -1.
pub fn string_search(s: Str, sub: Str, from: Int) -> Int {
    let (s, sub, from) = (s.as_str(), sub.as_str(), int_to_i64(from).max(0));
    let Some((start, _)) = s
        .char_indices()
        .chain(std::iter::once((s.len(), ' ')))
        .nth(from as usize)
    else {
        return i64_to_int(-1);
    };
    i64_to_int(
        s[start..]
            .find(sub)
            .map_or(-1, |b| from + s[start..start + b].chars().count() as i64),
    )
}

pub fn string_ci_eq_p(a: Str, b: Str) -> bool {
    a.as_str().to_lowercase() == b.as_str().to_lowercase()
}

pub fn string_downcase(s: Str) -> Str {
    leak_string(s.as_str().to_lowercase())
}

/// The characters from `start` to `end`.
pub fn substring(s: Str, start: Int, end: Int) -> Str {
    let (start, end) = (int_to_i64(start), int_to_i64(end));
    let n = s.as_str().chars().count() as i64;
    if start < 0 || end < start || end > n {
        fail("substring range out of bounds")
    }
    leak_string(
        s.as_str()
            .chars()
            .skip(start as usize)
            .take((end - start) as usize)
            .collect(),
    )
}

/// `symbol`: a name, compared by its text, which is what FX-26's
/// interning makes `eq?` of two symbols.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Symbol(Str);

impl Show for Symbol {
    fn show(&self, out: &mut String) {
        out.push_str(self.0.as_str())
    }
}

/// A quoted symbol, `'name`.
pub const fn symbol_lit(s: Str) -> Symbol {
    Symbol(s)
}

#[inline]
pub fn string_to_symbol(s: Str) -> Symbol {
    Symbol(s)
}

#[inline]
pub fn symbol_to_string(s: Symbol) -> Str {
    s.0
}

// Ported from Fixpt (C121; scripts/fixpt-drift.py):
// fixpt-source: crates/fixpt-heap/src/heap.rs@f03cabe
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// FX-26's `symbol-name-hash`, as Fixpt computes it (`symbol_hash_of`):
/// FNV-1a of the name's bytes, its sign bit cleared, as a fixnum of 61
/// bits, so that a table's buckets are in the same order.
pub fn symbol_name_hash(s: Symbol) -> Int {
    let h =
        s.0.as_str()
            .bytes()
            .fold(FNV_OFFSET, |h, b| (h ^ b as u64).wrapping_mul(FNV_PRIME));
    let n = (h as i64) & i64::MAX;
    i64_to_int(((n as u64) << 3) as i64 >> 3)
}

/// FX-26's `string-hash`, as Fixpt computes it (`string_hash_of`): FNV-1a
/// of the code points, shifted into a fixnum.
pub fn string_hash(s: Str) -> Int {
    let h = s
        .as_str()
        .chars()
        .fold(FNV_OFFSET, |h, c| (h ^ c as u64).wrapping_mul(FNV_PRIME));
    i64_to_int((h >> 4) as i64)
}

#[inline]
pub fn symbol_eq_p(a: Symbol, b: Symbol) -> bool {
    a.0.as_str() == b.0.as_str()
}

pub fn symbol_compare(a: Symbol, b: Symbol) -> Int {
    string_compare(a.0, b.0)
}
