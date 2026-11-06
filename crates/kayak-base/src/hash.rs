//! A fixed hasher: the same input hashes the same way in every run.
//!
//! The function is FxHash, the multiply-and-rotate hash rustc and Firefox
//! use: fast on short keys, and no defence against keys chosen to collide,
//! which nothing here needs. Written out rather than taken as a dependency
//! (`PLAN.md` §1, I4).
//!
//! What it hashes is what the key's `Hash` impl writes. For our own types
//! that is ours to keep stable; for the standard library's (`str`, tuples,
//! integers) it is stable in practice but not promised across Rust
//! releases, which is why `tests/golden.rs` pins values. Anything persisted
//! across builds (fingerprints in a heap image, later) must not rely on it.
// coopt-source: crates/coopt-base/src/hash.rs@9283bba

// The word mixes on whole 64-bit words. Both platforms coopt targets
// (arm64 macOS, x86_64 and arm64 Linux) have a 64-bit `usize`.
const _: () = assert!(usize::BITS == 64);

use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};

const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

/// FxHash over 64-bit words.
#[derive(Default, Clone, Copy, Debug)]
pub struct FxHasher {
    hash: u64,
}

impl FxHasher {
    #[inline]
    fn add(&mut self, word: u64) {
        self.hash = (self.hash.rotate_left(5) ^ word).wrapping_mul(SEED);
    }
}

impl Hasher for FxHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let (chunks, rest) = bytes.as_chunks::<8>();
        for &chunk in chunks {
            self.add(u64::from_le_bytes(chunk));
        }
        if !rest.is_empty() {
            let mut word = [0u8; 8];
            word[..rest.len()].copy_from_slice(rest);
            // The length goes in too, so that "a" and "a\0" differ.
            self.add(u64::from_le_bytes(word) ^ ((rest.len() as u64) << 56));
        }
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.add(u64::from(i));
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.add(u64::from(i));
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.add(u64::from(i));
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.add(i);
    }

    #[inline]
    fn write_usize(&mut self, i: usize) {
        self.add(i as u64);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.hash
    }
}

/// Builds [`FxHasher`]s, all alike.
pub type FixedState = BuildHasherDefault<FxHasher>;

/// A `HashMap` whose order depends only on what was put in it, and when.
pub type FixedMap<K, V> = HashMap<K, V, FixedState>;

/// A `HashSet` whose order depends only on what was put in it, and when.
pub type FixedSet<K> = HashSet<K, FixedState>;

/// The hash of one value, for fingerprints and tests.
pub fn hash_one<T: std::hash::Hash + ?Sized>(value: &T) -> u64 {
    let mut h = FxHasher::default();
    value.hash(&mut h);
    h.finish()
}
