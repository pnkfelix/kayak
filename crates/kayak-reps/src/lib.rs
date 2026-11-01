//! Types as reps (`docs/research/type-reps.md`, C299, C300): Fixpt's
//! `fixpt-reps`, taken whole, its `lib.rs`, `binary.rs`, `fill.rs` and
//! `text.rs` copied as they are into `fixpt/` by `scripts/built-in.py`
//! (checked at the pin), so that a rep kayak makes is Fixpt's byte for
//! byte. Not formatted here: they are Fixpt's text. Its `heap.rs`, of
//! Fixpt's heap, is kayak's own (`fixpt/heap.rs`).

#[rustfmt::skip]
#[allow(clippy::all)]
mod fixpt;

pub use fixpt::*;
