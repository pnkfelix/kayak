//! kayak-jit's constants are the native convention's it declares, Fixpt's
//! at the pin (`src/native-arm64.conv`, which `scripts/built-in.py` copies):
//! a change to either shows here, as Fixpt's own `tests/convention.rs`
//! compares its declaration with its tables.

use kayak_jit::{DECLARATION, layout, state};

/// The clause of role `role`, as written.
fn clause(role: &str) -> &'static str {
    DECLARATION
        .lines()
        .find(|l| l.starts_with(&format!("({role} ")))
        .unwrap_or_else(|| panic!("no clause {role}"))
}

fn has(role: &str, part: &str) {
    assert!(
        clause(role).contains(part),
        "({role} ...) has no {part}: {}",
        clause(role)
    );
}

#[test]
fn the_state_fields_are_at_the_offsets_declared() {
    for (name, at) in [
        ("fuel", state::FUEL),
        ("reserve", state::RESERVE),
        ("trap", state::TRAP),
        ("pc", state::PC),
        ("args", state::ARGS),
        ("nargs", state::NARGS),
        ("aux", state::AUX),
        ("callout", state::CALLOUT),
        ("rust-sp", state::RUST_SP),
        ("native-sp", state::NATIVE_SP),
        ("fp", state::FP),
        ("top", state::TOP),
        ("words", state::WORDS),
        ("alloc-limit", state::ALLOC_LIMIT),
        ("code", state::CODE),
        ("clo", state::CLO),
        ("delta", state::DELTA),
        ("code-lo", state::CODE_LO),
        ("trap-stub", state::TRAP_STUB),
        ("foreign", state::FOREIGN),
        ("closure", state::CLOSURE),
        ("cards", state::CARDS),
        ("stack-limit", state::STACK_LIMIT),
        ("overflow", state::OVERFLOW),
        ("poll", state::POLL),
        ("cons", state::CONS),
        ("bloblet", state::BLOBLET),
    ] {
        has("state", &format!("({name} {at})"));
    }
    // kayak's own field is past every one declared.
    assert!(
        clause("state")
            .split(['(', ')', ' '])
            .filter_map(|w| w.parse::<usize>().ok())
            .all(|at| at < state::LEAVE)
    );
}

#[test]
fn the_registers_are_the_trampolines() {
    has("registers", "(arguments x1 x2 x3 x4 x5 x6 x7 x8)");
    has("registers", "(count x9) (result x0) (closure x10)");
    has(
        "registers",
        "(pinned (state x24) (fuel x28) (stack-limit x27) (code-distance x26))",
    );
    has("code", "(runs-at reference plus x26)");
}

#[test]
fn the_values_are_laid_out_as_declared() {
    has(
        "values",
        &format!(
            "(fixnum {}) (pair {})",
            layout::TAG_FIXNUM,
            layout::TAG_PAIR
        ),
    );
    has(
        "values",
        &format!(
            "(immediate {}) (bloblet {}) (trailer {}) (header {})",
            layout::TAG_IMMEDIATE,
            layout::TAG_BLOBLET,
            layout::TAG_TRAILER,
            layout::TAG_HEADER
        ),
    );
    has(
        "values",
        &format!(
            "(fixnum (shift {}) (bits {}))",
            layout::FIXNUM_SHIFT,
            layout::FIXNUM_BITS
        ),
    );
    has(
        "values",
        &format!(
            "(false {}) (true {}) (null {}) (unit {})",
            layout::FALSE,
            layout::TRUE,
            layout::NULL,
            layout::UNIT
        ),
    );
    has(
        "values",
        &format!(
            "(character (subtag {}) (payload-shift {}))",
            layout::CHAR_SUBTAG,
            layout::CHAR_SHIFT
        ),
    );
    has("values", "(pair (car -1) (cdr 7))");
    has(
        "values",
        "(bloblet (reference suffix 4) (field-k-before-suffix 8))",
    );
    has(
        "values",
        "(header (tag 0 3) (kind 3 8) (large 11 1) (fields-frozen 12 1) (suffix-frozen 13 1) (fields 14 18) (bytes 32 32))",
    );
    has("values", &format!("(bloblet {})", layout::KIND_BLOBLET));
    has(
        "values",
        &format!("(cellular-closure {})", layout::KIND_CELLULAR_CLOSURE),
    );
    has(
        "values",
        &format!("(native-closure {})", layout::KIND_NATIVE_CLOSURE),
    );
    has(
        "closures",
        &format!(
            "(kind {}) (code-field {})",
            layout::KIND_NATIVE_CLOSURE,
            layout::CLOSURE_CODE
        ),
    );
    has("closures", "(native-when-code-at-least code-lo)");
}

#[test]
fn the_routines_are_the_ones_kayak_jit_answers() {
    for (name, field) in [
        ("foreign", "foreign"),
        ("poll", "poll"),
        ("cons", "cons"),
        ("bloblet", "bloblet"),
        ("trap", "trap-stub"),
    ] {
        has("routines", &format!("({name} (entry state {field})"));
    }
    has(
        "routines",
        "(overflow (when sp below x27 after-frame-made) (entry state overflow by x30)",
    );
    has("routines", "(2 stack-overflow)");
    assert_eq!(kayak_jit::STACK_OVERFLOW, 2);
    has("fuel", "(register x28)");
}
