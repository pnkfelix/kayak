//! kayak's in place of Fixpt's `fixpt-reps/src/heap.rs`, which is of
//! Fixpt's heap (C300): a rep crosses kayak's runtimes as its binary form
//! in a bytevector, as the standard procedures `rep-of-nodes` and
//! `rep-nodes` have it, so what is kept of Fixpt's is its tag and its
//! toggle.

/// The tag of a rep's product, as Fixpt's.
pub const TAG: &str = "type-rep";

/// Whether every rep read is validated, from the trusted pieces too: the
/// environment variable `FIXPT_VALIDATE_REPS` set, as Fixpt's.
pub fn validate_all() -> bool {
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("FIXPT_VALIDATE_REPS").is_some())
}
