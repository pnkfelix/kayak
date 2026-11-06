//! A seeded generator: xoshiro256** (Blackman and Vigna), its state filled
//! from the seed by SplitMix64, as its authors recommend.
//!
//! A run has one seed. Each subsystem that makes choices takes its own
//! stream with [`Rng::split`], so that a new draw in one subsystem (a
//! stress policy turned on, say) does not shift every other subsystem's
//! choices.
// coopt-source: crates/coopt-base/src/rng.rs@9283bba

/// SplitMix64: one 64-bit word of state, advanced by a constant.
#[derive(Clone, Debug)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> SplitMix64 {
        SplitMix64 { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
}

/// xoshiro256**.
#[derive(Clone, Debug)]
pub struct Rng {
    s: [u64; 4],
}

impl Rng {
    /// The generator for `seed`. Every seed gives a usable state: SplitMix64
    /// never yields four zero words in a row.
    pub fn new(seed: u64) -> Rng {
        let mut sm = SplitMix64::new(seed);
        Rng {
            s: [sm.next_u64(), sm.next_u64(), sm.next_u64(), sm.next_u64()],
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        let result = self.s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }

    /// A number in `0..n`, every one equally likely. `n` must not be zero.
    ///
    /// Lemire's multiply-and-reject: the high word of `x * n` is in range,
    /// and the few `x` whose low word falls in the biased zone are drawn
    /// again.
    pub fn below(&mut self, n: u64) -> u64 {
        assert!(n > 0, "Rng::below(0)");
        let threshold = n.wrapping_neg() % n;
        loop {
            let m = u128::from(self.next_u64()) * u128::from(n);
            if (m as u64) >= threshold {
                return (m >> 64) as u64;
            }
        }
    }

    /// True with probability `num / den`.
    pub fn chance(&mut self, num: u64, den: u64) -> bool {
        self.below(den) < num
    }

    /// A new, independent stream, named by `label`. The same generator split
    /// by the same label gives the same stream; splitting does not advance
    /// this generator, so the order in which subsystems take their streams
    /// does not matter.
    pub fn split(&self, label: &str) -> Rng {
        let mut sm = SplitMix64::new(
            self.s[0]
                ^ self.s[1].rotate_left(16)
                ^ self.s[2].rotate_left(32)
                ^ self.s[3].rotate_left(48)
                ^ crate::hash::hash_one(label),
        );
        Rng {
            s: [sm.next_u64(), sm.next_u64(), sm.next_u64(), sm.next_u64()],
        }
    }
}
