//! kayak-jit's constants are the native convention's it declares, Fixpt's
//! at the pin (`src/native-arm64.conv`, or on x86_64 `src/native-x86_64.conv`,
//! which `scripts/built-in.py` copies):
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
    // x86_64's arguments past the third.
    if cfg!(target_arch = "x86_64") {
        for k in 0..5 {
            has("state", &format!("(reg{} {})", k + 4, state::REG4 + 8 * k));
        }
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
#[cfg(not(target_arch = "x86_64"))]
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
#[cfg(target_arch = "x86_64")]
fn the_registers_are_the_trampolines() {
    has(
        "registers",
        "(arguments rdi rsi r8 (state reg4) (state reg5) (state reg6) (state reg7) (state reg8))",
    );
    has("registers", "(count r10) (result rax) (closure rbx)");
    has("registers", "(link pushed)");
    has(
        "registers",
        "(pinned (state r15) (fuel (state fuel)) (stack-limit (state stack-limit)) (code-distance (state delta)))",
    );
    has("code", "(runs-at reference plus (state delta))");
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
    // Every procedure's code runs (Fixpt 8c7111d, C234): tier 1's words
    // carry the stub that enters `foreign`; code written to the older
    // clause still tests against the state's `code-lo`, its native part.
    has("closures", "(code-always-runs)");
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
    if cfg!(target_arch = "x86_64") {
        has(
            "routines",
            "(overflow (when rsp below (state stack-limit) after-frame-made) (entry state overflow by call)",
        );
        has(
            "routines",
            "(foreign (entry state foreign) (in (closure rbx) (count r10)",
        );
        has(
            "routines",
            "(cons (entry state cons) (in (car rdi) (cdr rsi)) (out rax)",
        );
        has(
            "routines",
            "(bloblet (entry state bloblet) (in (bytes rdi) (fields rsi)) (out rax)",
        );
        has("routines", "(trap (entry state trap-stub) (in (kind r10))");
        has("fuel", "(fuel (state fuel)");
    } else {
        has(
            "routines",
            "(overflow (when sp below x27 after-frame-made) (entry state overflow by x30)",
        );
        has("fuel", "(register x28)");
    }
    has("routines", "(2 stack-overflow)");
    assert_eq!(kayak_jit::STACK_OVERFLOW, 2);
}
