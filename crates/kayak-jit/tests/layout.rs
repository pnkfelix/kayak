//! kayak-jit's object layout is Fixpt's at the pin (`src/layout.fx`, which
//! `scripts/built-in.py` copies from the file Fixpt generates from its
//! `layout.rs`; C121): a change upstream shows here when the pin moves.

use kayak_jit::layout;

const LAYOUT: &str = include_str!("../src/layout.fx");

/// `(define NAME int N)`'s `N`.
fn value(name: &str) -> i64 {
    let prefix = format!("(define {name} int ");
    let line = LAYOUT
        .lines()
        .find(|l| l.starts_with(&prefix))
        .unwrap_or_else(|| panic!("layout.fx defines no {name}"));
    let rest = &line[prefix.len()..];
    rest[..rest.find(')').unwrap()].parse().unwrap()
}

#[test]
fn the_tags_are_fixpts() {
    for (name, ours) in [
        ("tag-fixnum", layout::TAG_FIXNUM),
        ("tag-pair", layout::TAG_PAIR),
        ("tag-immediate", layout::TAG_IMMEDIATE),
        ("tag-bloblet", layout::TAG_BLOBLET),
        ("tag-trailer", layout::TAG_TRAILER),
        ("tag-header", layout::TAG_HEADER),
    ] {
        assert_eq!(value(name), ours as i64, "{name}");
    }
    assert_eq!(1 << value("tag-bits"), layout::TAG_MASK as i64 + 1);
    assert_eq!(value("tag-bits"), layout::FIXNUM_SHIFT as i64);
}

#[test]
fn the_kinds_and_closure_fields_are_fixpts() {
    for (name, ours) in [
        ("kind-string", layout::KIND_STRING),
        ("kind-bloblet", layout::KIND_BLOBLET),
        ("kind-cellular-closure", layout::KIND_CELLULAR_CLOSURE),
        ("kind-native-closure", layout::KIND_NATIVE_CLOSURE),
    ] {
        assert_eq!(value(name), ours as i64, "{name}");
    }
    assert_eq!(value("closure-code"), layout::CLOSURE_CODE as i64);
}

/// A field of the header `layout::header` makes: its bits, by Fixpt's
/// `NAME-lo` and `NAME-width`.
fn field(word: u64, name: &str) -> u64 {
    let lo = value(&format!("header-{name}-lo"));
    let width = value(&format!("header-{name}-width"));
    (word >> lo) & ((1u64 << width) - 1)
}

#[test]
fn a_header_is_laid_out_as_fixpts() {
    let h = layout::header(layout::KIND_BLOBLET, 5, 4096, true, false);
    assert_eq!(field(h, "tag"), layout::TAG_HEADER);
    assert_eq!(field(h, "kind"), layout::KIND_BLOBLET as u64);
    assert_eq!(field(h, "large"), 0);
    assert_eq!(field(h, "fields-frozen"), 1);
    assert_eq!(field(h, "suffix-frozen"), 0);
    assert_eq!(field(h, "fields"), 5);
    assert_eq!(field(h, "bytes"), 4096);
}
