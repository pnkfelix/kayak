//! What every other crate needs in order to be deterministic.
//!
//! - [`hash`]: a fixed hasher, and the map and set types built on it. The
//!   standard library's default hasher is seeded randomly per process, so a
//!   `HashMap`'s iteration order differs from run to run; `coopt-tidy`
//!   refuses `HashMap` and `HashSet` everywhere but here.
//! - [`fingerprint`]: 128-bit fingerprints of values, over an encoding we
//!   define, for the query engine's early cutoff.
//! - [`rng`]: a seeded generator. Every choice the runtime makes "at random"
//!   (the scheduler's tie-breaks, stress policies, the simulated runner's
//!   delays) draws from one of these, split from the run's seed.
// coopt-source: crates/coopt-base/src/lib.rs@9283bba

pub mod fingerprint;
pub mod hash;
pub mod rng;
pub mod stored;

pub use fingerprint::{Fingerprint, StableEncode, fingerprint};
pub use hash::{FixedMap, FixedSet, FixedState, FxHasher};
pub use rng::Rng;
pub use stored::{Bad, Decoder, Load};
