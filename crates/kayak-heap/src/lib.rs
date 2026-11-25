//! The heap both of kayak's runtimes share, a piece at a time
//! (`docs/research/one-runtime.md`, step 3): what kayak-rt's collector
//! (`obj::collect`) and kcrt's (`gc.rs`, Cheney's) each did their own way,
//! over Fixpt's words alone, not either runtime's types.
//!
//! First (C535) the object walk: where an object starts, ends, and which
//! of its words hold values, read from its header as Fixpt's layout says
//! (`kayak-layout`), whose table says which kinds' fields are traced. No
//! memory is read here: each function is given how to read a word, so the
//! crate has no `unsafe`, and its caller's own rules for reading (a word
//! of kayak-rt's space, of kc's stack) stay the caller's.

use kayak_layout as layout;

const TAG_MASK: u64 = layout::TAG_MASK;
const TAG_HEADER: u64 = layout::tag("header");
const TAG_TRAILER: u64 = layout::tag("trailer");
const KIND_EXTENSION: u64 = layout::KIND_EXTENSION as u64;

/// Whether the fields of an object of kind `code` hold values a collector
/// follows: Fixpt's table's `traced` (a string's, a bytevector's, a
/// flonum's and a bignum's do not).
pub const fn traced(code: u8) -> bool {
    KIND_TRACED[code as usize]
}

const KIND_TRACED: [bool; 256] = {
    let mut t = [true; 256];
    let mut i = 0;
    while i < layout::KINDS.len() {
        t[layout::KINDS[i].code as usize] = layout::KINDS[i].traced;
        i += 1;
    }
    t
};

/// An object, by the addresses of its words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Object {
    /// A pair: two words, each a value, and no header.
    pub pair: bool,
    /// Its main header's address (a pair's first word's), its identity:
    /// past its extension word, if it has one.
    pub main: usize,
    /// Its kind (0 for a pair).
    pub kind: u8,
    /// Its count of fields, the words after the main header.
    pub fields: usize,
    /// How many of them, from the first, hold values: all, or none.
    pub traced: usize,
    /// Where its suffix starts, where an object's word points (0 for a
    /// pair).
    pub suffix: usize,
    /// The address past its last word.
    pub end: usize,
}

/// The object whose first word is at `at` (a pair's, a header, or an
/// extension word before the main header), `word` reading the word at an
/// address.
#[inline]
pub fn object_at(at: usize, word: impl Fn(usize) -> u64) -> Object {
    let w = word(at);
    if w & TAG_MASK != TAG_HEADER {
        return Object {
            pair: true,
            main: at,
            kind: 0,
            fields: 2,
            traced: 2,
            suffix: 0,
            end: at + 16,
        };
    }
    let (main, fields) = if layout::H_KIND.get(w) == KIND_EXTENSION {
        (at + 8, layout::X_FIELDS.get(w) as usize)
    } else {
        (at, layout::H_FIELDS.get(w) as usize)
    };
    let h = if main == at { w } else { word(main) };
    let kind = layout::H_KIND.get(h) as u8;
    let bytes = layout::H_BYTES.get(h) as usize;
    let suffix = main + 8 * (1 + fields);
    Object {
        pair: false,
        main,
        kind,
        fields,
        traced: if traced(kind) { fields } else { 0 },
        suffix,
        end: suffix + 8 * bytes.div_ceil(8),
    }
}

/// The main header of the object whose suffix starts at `p`, as Fixpt
/// finds it: the word before, if a header (an object of no fields); else
/// that word is its trailer, which says how far back the header is. `None`
/// if it is neither.
#[inline]
pub fn header_of_suffix(p: usize, word: impl Fn(usize) -> u64) -> Option<usize> {
    let before = word(p - 8);
    match before & TAG_MASK {
        TAG_HEADER => Some(p - 8),
        TAG_TRAILER => Some(p - 8 - 8 * layout::T_DISTANCE.get(before) as usize),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(kind: u8, fields: u64, bytes: u64) -> u64 {
        TAG_HEADER
            | layout::H_KIND.put(0, kind as u64)
            | layout::H_FIELDS.put(0, fields)
            | layout::H_BYTES.put(0, bytes)
    }

    fn reader(ws: &[u64]) -> impl Fn(usize) -> u64 + '_ {
        move |a| ws[a / 8]
    }

    #[test]
    fn the_kinds_traced_as_fixpt_says() {
        for k in ["string", "bytevector", "flonum", "bignum"] {
            assert!(!traced(layout::kind(k)), "{k}");
        }
        for k in [
            "symbol", "vector", "closure", "box", "bloblet", "sum", "product",
        ] {
            assert!(traced(layout::kind(k)), "{k}");
        }
    }

    #[test]
    fn a_pair() {
        let ws = [0b001, 0b001];
        let o = object_at(0, reader(&ws));
        assert!(o.pair);
        assert_eq!((o.traced, o.end), (2, 16));
    }

    /// A string of three code points, as kayak-rt and kcrt make it: no
    /// fields, its suffix the count and the points.
    #[test]
    fn a_string() {
        let ws = [header(layout::kind("string"), 0, 24), 3, 0, 0];
        let o = object_at(0, reader(&ws));
        assert_eq!(
            (o.kind, o.fields, o.traced, o.suffix, o.end),
            (1, 0, 0, 8, 32)
        );
        assert_eq!(header_of_suffix(8, reader(&ws)), Some(0));
    }

    /// A product of two values: its fields by negative offset from its
    /// (empty) suffix, field 1 its trailer, three words back to the header.
    #[test]
    fn a_product_and_its_trailer() {
        let ws = [
            header(layout::kind("product"), 3, 0),
            0b001,
            0b001,
            TAG_TRAILER | 3 << 3,
        ];
        let o = object_at(0, reader(&ws));
        assert_eq!((o.fields, o.traced, o.suffix, o.end), (3, 3, 32, 32));
        assert_eq!(header_of_suffix(32, reader(&ws)), Some(0));
    }

    /// Fields past the main header's width: an extension word first.
    #[test]
    fn an_extension_word() {
        let ext = TAG_HEADER | KIND_EXTENSION << 3 | 5 << 11;
        let ws = [ext, header(layout::kind("vector"), 0, 0), 0, 0, 0, 0, 0];
        let o = object_at(0, reader(&ws));
        assert_eq!(
            (o.main, o.fields, o.traced, o.suffix, o.end),
            (8, 5, 5, 56, 56)
        );
    }
}
