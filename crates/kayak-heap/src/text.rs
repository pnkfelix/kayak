//! Strings and symbols' names over Fixpt's words, once (C536; the plan's
//! step 4): a string's layout, its hash, a symbol's hash, search. Over a
//! string's code points as one `&[u32]`, which each runtime has of its
//! strings in its own way (kayak-rt's `Str::points`, kcrt's
//! `string_points`), both of Fixpt's layout: a header of kind 1, no
//! fields, no trailer, the suffix the count, then the points, two to a
//! word, the first in the low half.

use kayak_layout as layout;

// Ported from Fixpt (C121; scripts/fixpt-drift.py): its string and symbol
// hashes, `string_search`.
// fixpt-source: crates/fixpt-heap/src/heap.rs@c5b5991

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// The words a string of `n` code points takes: header, count, points.
pub const fn string_words(n: usize) -> usize {
    2 + n.div_ceil(2)
}

/// A string's header, of `n` code points: its suffix's bytes, the
/// count's word and the points'; frozen, fields and suffix.
pub const fn string_header(n: usize) -> u64 {
    layout::tag("header")
        | layout::H_KIND.put(0, layout::kind("string") as u64)
        | layout::H_FIELDS_FROZEN.put(0, 1)
        | layout::H_SUFFIX_FROZEN.put(0, 1)
        | layout::H_BYTES.put(0, 8 * (string_words(n) as u64 - 1))
}

/// The words of a string of `points`, written into `ws`
/// (`string_words(points.len())` of them): header, count, points.
pub fn fill_string(points: &[u32], ws: &mut [u64]) {
    let n = points.len();
    ws[0] = string_header(n);
    ws[1] = n as u64;
    for (k, pair) in points.chunks(2).enumerate() {
        ws[2 + k] = pair[0] as u64 | pair.get(1).map_or(0, |&c| (c as u64) << 32);
    }
}

/// FX-26's `string-hash`, Fixpt's `%string-hash` (`Heap::string_fnv`):
/// FNV-1a of the code points, each a `u64`, shifted right 4, a fixnum's
/// value. Of the contents alone, never an address (the heap may move).
pub fn string_hash(points: &[u32]) -> i64 {
    let h = points
        .iter()
        .fold(FNV_OFFSET, |h, &c| (h ^ c as u64).wrapping_mul(FNV_PRIME));
    (h >> 4) as i64
}

/// A symbol's hash, as Fixpt computes it (`symbol_hash_of`): FNV-1a of
/// its name's UTF-8 bytes, its sign bit cleared, kept to a fixnum's 61
/// bits, the fixnum's value. A `const fn`, so that a literal's is made as
/// rustc builds.
pub const fn symbol_hash(name: &str) -> i64 {
    let m = name.as_bytes();
    let mut h = FNV_OFFSET;
    let mut i = 0;
    while i < m.len() {
        h = (h ^ m[i] as u64).wrapping_mul(FNV_PRIME);
        i += 1;
    }
    let n = (h as i64) & i64::MAX;
    (n << 3) >> 3
}

/// Where `sub` is first found in `s` at or past code point `from`, or
/// none: `from` past the end of `s` finds nothing; an empty `sub` is found
/// at `from` (Fixpt's `Heap::string_search`).
pub fn search(s: &[u32], sub: &[u32], from: usize) -> Option<usize> {
    if from > s.len() {
        return None;
    }
    if sub.is_empty() {
        return Some(from);
    }
    s[from..]
        .windows(sub.len())
        .position(|w| w == sub)
        .map(|k| from + k)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn points(t: &str) -> Vec<u32> {
        t.chars().map(|c| c as u32).collect()
    }

    #[test]
    fn a_strings_words() {
        let mut ws = [0; 4];
        fill_string(&points("abc"), &mut ws);
        assert_eq!(layout::H_BYTES.get(ws[0]), 24);
        assert_eq!(layout::H_KIND.get(ws[0]), 1);
        assert_eq!(&ws[1..], &[3, 97 | 98 << 32, 99]);
    }

    /// Fixpt's answers: `(string-hash "abc")` and the hash a symbol `abc`
    /// keeps.
    #[test]
    fn the_hashes_fixpts() {
        assert_eq!(string_hash(&[]), (FNV_OFFSET >> 4) as i64);
        assert_eq!(symbol_hash(""), ((FNV_OFFSET as i64 & i64::MAX) << 3) >> 3);
        assert!(symbol_hash("circle") >= -(1 << 60) && symbol_hash("circle") < 1 << 60);
    }

    #[test]
    fn searched() {
        let s = points("a§b§c");
        assert_eq!(search(&s, &points("§"), 0), Some(1));
        assert_eq!(search(&s, &points("§"), 2), Some(3));
        assert_eq!(search(&s, &points("§"), 4), None);
        assert_eq!(search(&s, &[], 5), Some(5));
        assert_eq!(search(&s, &[], 6), None);
    }
}
