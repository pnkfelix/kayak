//! FX-26's standard operations on booleans, characters, strings and
//! symbols beside those in `lib.rs` (Fixpt 17ac0c5's `standard.rs`, each as
//! its runtime has it; `CLAUDES_CHOICES.md` C56).

use crate::{Int, Show, Str, Symbol, fail, i64_to_int, int_to_i64, leak_string};

/// FX-26's `char->integer` and `integer->char`, at `int`.
#[inline]
pub fn char_to_integer(c: char) -> Int {
    i64_to_int(c as i64)
}

/// A code past a Unicode scalar value fails in Fixpt's words, a big one
/// too (its low bits were taken for it before C402).
pub fn integer_to_char(n: Int) -> char {
    match n.view() {
        crate::IntView::Small(v) => match u32::try_from(v).ok().and_then(char::from_u32) {
            Some(c) => c,
            None => fail("not a Unicode scalar value"),
        },
        crate::IntView::Big(_) => fail("not a Unicode scalar value"),
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

// Strings compare character by character: their code points'.

#[inline]
pub fn string_lt_p(a: Str, b: Str) -> bool {
    a.points() < b.points()
}
#[inline]
pub fn string_lt_eq_p(a: Str, b: Str) -> bool {
    a.points() <= b.points()
}
#[inline]
pub fn string_gt_p(a: Str, b: Str) -> bool {
    a.points() > b.points()
}
#[inline]
pub fn string_gt_eq_p(a: Str, b: Str) -> bool {
    a.points() >= b.points()
}

/// -1, 0 or 1, as `a` comes before, is, or comes after `b`.
pub fn string_compare(a: Str, b: Str) -> Int {
    i64_to_int(a.points().cmp(b.points()) as i64)
}

/// Where `sub` first occurs in `s` at or after character `from`, in
/// characters; or -1.
pub fn string_search(s: Str, sub: Str, from: Int) -> Int {
    let from = int_to_i64(from).max(0) as usize;
    let (t, m) = (s.points(), sub.points());
    if from > t.len() {
        return i64_to_int(-1);
    }
    let found = if m.is_empty() {
        Some(from)
    } else {
        t[from..]
            .windows(m.len())
            .position(|w| w == m)
            .map(|k| from + k)
    };
    i64_to_int(found.map_or(-1, |k| k as i64))
}

pub fn string_ci_eq_p(a: Str, b: Str) -> bool {
    a.text().to_lowercase() == b.text().to_lowercase()
}

pub fn string_downcase(s: Str) -> Str {
    leak_string(s.text().to_lowercase())
}

/// The characters from `start` to `end`.
pub fn substring(s: Str, start: Int, end: Int) -> Str {
    let (start, end) = (int_to_i64(start), int_to_i64(end));
    let ps = s.points();
    if start < 0 || end < start || end as usize > ps.len() {
        fail("substring range out of bounds")
    }
    crate::str_of_points(&[&ps[start as usize..end as usize]])
}

impl Show for Symbol {
    fn show(&self, out: &mut String) {
        out.extend(self.name().chars())
    }
}

/// `string->symbol` (C504): a symbol's object made of it.
#[inline]
pub fn string_to_symbol(s: Str) -> Symbol {
    Symbol::new(s)
}

#[inline]
pub fn symbol_to_string(s: Symbol) -> Str {
    s.name()
}

// Ported from Fixpt (C121; scripts/fixpt-drift.py):
// fixpt-source: crates/fixpt-heap/src/heap.rs@c5b5991
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// FX-26's `symbol-name-hash`, as Fixpt computes it (`symbol_hash_of`):
/// FNV-1a of the name's bytes, its sign bit cleared, as a fixnum of 61
/// bits, so that a table's buckets are in the same order.
pub fn symbol_name_hash(s: Symbol) -> Int {
    // Its object's (C504), Fixpt's `symbol_hash_of` as rustc or the heap
    // made it.
    i64_to_int(s.hash() as i64)
}

/// FX-26's `string-hash`, as Fixpt computes it (`string_hash_of`): FNV-1a
/// of the code points, shifted into a fixnum.
pub fn string_hash(s: Str) -> Int {
    let h = s
        .points()
        .iter()
        .fold(FNV_OFFSET, |h, &c| (h ^ c as u64).wrapping_mul(FNV_PRIME));
    i64_to_int((h >> 4) as i64)
}

#[inline]
pub fn symbol_eq_p(a: Symbol, b: Symbol) -> bool {
    a == b
}

pub fn symbol_compare(a: Symbol, b: Symbol) -> Int {
    string_compare(a.name(), b.name())
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
            assert_eq!(crate::i64_to_string(n).text(), n.to_string());
        }
        for c in ['a', '…', '§', '\u{1F600}'] {
            assert_eq!(crate::char_to_string(c).text(), c.to_string());
        }
        let t = "a…b§\u{1F600}c";
        assert_eq!(
            crate::list_to_string(crate::string_to_list(crate::str_lit(t))).text(),
            t
        );
        assert_eq!(crate::list_to_string(crate::nil()).text(), "");
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
                assert_eq!(got.text(), want);
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
