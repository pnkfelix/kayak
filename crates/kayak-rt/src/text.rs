//! FX-26's standard operations on booleans, characters, strings and
//! symbols beside those in `lib.rs` (Fixpt 17ac0c5's `standard.rs`, each as
//! its runtime has it; `CLAUDES_CHOICES.md` C56).

use crate::{Int, Show, Str, char_byte, cursor_at, fail, i64_to_int, int_to_i64, leak_string};

/// FX-26's `char->integer` and `integer->char`, at `int`.
#[inline]
pub fn char_to_integer(c: char) -> Int {
    i64_to_int(c as i64)
}

/// A code past a Unicode scalar value fails in Fixpt's words, a big one
/// too (its low bits were taken for it before C402).
pub fn integer_to_char(n: Int) -> char {
    match n {
        Int::Small(v) => match u32::try_from(v).ok().and_then(char::from_u32) {
            Some(c) => c,
            None => fail("not a Unicode scalar value"),
        },
        Int::Big(_) => fail("not a Unicode scalar value"),
    }
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
    let from = int_to_i64(from).max(0);
    let Some(start) = char_byte(s, from as usize) else {
        return i64_to_int(-1);
    };
    let t = s.as_str();
    let Some(b) = t[start..].find(sub.as_str()) else {
        return i64_to_int(-1);
    };
    let at = from as usize + t[start..start + b].chars().count();
    cursor_at(s, at, start + b);
    i64_to_int(at as i64)
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
    let bytes = (start >= 0 && end >= start)
        .then(|| Some((char_byte(s, start as usize)?, char_byte(s, end as usize)?)))
        .flatten();
    let Some((b0, b1)) = bytes else {
        fail("substring range out of bounds")
    };
    crate::str_of_parts(&[&s.as_str()[b0..b1]])
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
// fixpt-source: crates/fixpt-heap/src/heap.rs@6f7bd73
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{int_lit, string_length_i64, string_ref_i64};

    /// Numbers, characters and lists of characters made into strings, as
    /// Rust writes them.
    #[test]
    fn made_in_place() {
        for n in [0, 7, -7, 10, 1234567890, i64::MAX, i64::MIN] {
            assert_eq!(crate::i64_to_string(n).as_str(), n.to_string());
        }
        for c in ['a', '…', '§', '\u{1F600}'] {
            assert_eq!(crate::char_to_string(c).as_str(), c.to_string());
        }
        let t = "a…b§\u{1F600}c";
        assert_eq!(
            crate::list_to_string(crate::string_to_list(crate::str_lit(t))).as_str(),
            t
        );
        assert_eq!(crate::list_to_string(crate::nil()).as_str(), "");
    }

    /// Each character, search and substring of a non-ASCII string, in any
    /// order, as counting from the start gives.
    #[test]
    fn counted_from_the_cursor() {
        let t = "a…b§c\n§§x…\nend";
        let s = leak_string(t.to_string());
        let cs: Vec<char> = t.chars().collect();
        let n = cs.len();
        assert_eq!(string_length_i64(s), n as i64);
        for &i in &[3, 0, 9, 10, 2, 13, 13, 1, 7, 4, 12, 5] {
            assert_eq!(string_ref_i64(s, i as i64), cs[i]);
        }
        for a in 0..=n {
            for b in a..=n {
                let want: String = cs[a..b].iter().collect();
                let got = substring(s, int_lit(a as i64), int_lit(b as i64));
                assert_eq!(got.as_str(), want);
            }
        }
        for from in 0..=n + 1 {
            let want = (from..n).find(|&i| cs[i] == '\n').map_or(-1, |i| i as i64);
            let got = string_search(s, crate::str_lit("\n"), int_lit(from as i64));
            assert_eq!(int_to_i64(got), want);
            if want >= 0 {
                assert_eq!(string_ref_i64(s, want), '\n');
            }
        }
    }
}
