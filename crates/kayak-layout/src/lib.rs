//! Fixpt's object layout and word format, specified once
//! (`docs/research/one-runtime.md`, step 1; TODO §56): its
//! `fixpt-heap/src/layout.rs` (every tag, header field and kind) and
//! `value.rs` (the word: fixnums, immediates, pairs, bloblets), taken
//! whole by `scripts/built-in.py` (checked at the pin), as `kayak-reps`
//! takes `fixpt-reps`, under Fixpt's own module names. kayak-rt,
//! kcrt-core and kayak-jit each name their constants from them, so that no
//! tag or kind is written in two places, and the two runtimes, which share
//! one heap in process, share one format by construction. Not formatted
//! here: they are Fixpt's text.
//!
//! A word's bits are Fixpt's; what they point at is each runtime's own:
//! Fixpt's [`value::Value`] holds an object's index from its heap's start,
//! kayak's runtimes its address. So kayak takes the constants and the
//! immediates' words, not `Value`'s constructors of pointers.

#[rustfmt::skip]
#[allow(clippy::all, dead_code)]
pub mod layout;
#[rustfmt::skip]
#[allow(clippy::all, dead_code)]
pub mod value;

pub use layout::*;

/// The immediates' words, as Fixpt's `Value` makes them.
pub mod imm {
    use crate::value::Value;
    pub const FALSE: u64 = Value::FALSE.raw();
    pub const TRUE: u64 = Value::TRUE.raw();
    pub const NULL: u64 = Value::NULL.raw();
    pub const UNIT: u64 = Value::UNIT.raw();
}

/// What Fixpt's crate root gives the table's tests (`crate::HashSet`).
#[cfg(test)]
type HashSet<T> = std::collections::HashSet<T>;
