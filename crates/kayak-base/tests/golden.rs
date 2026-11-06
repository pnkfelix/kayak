//! Pinned outputs. A change here changes every seeded run's choices and
//! every fingerprint, so it must be deliberate.
// coopt-source: crates/coopt-base/tests/golden.rs@9283bba

use kayak_base::hash::hash_one;
use kayak_base::rng::SplitMix64;
use kayak_base::{FixedMap, Rng};

#[test]
fn splitmix64_matches_the_reference() {
    // The reference implementation's first outputs for seed 0
    // (Vigna, splitmix64.c), as commonly published.
    let mut sm = SplitMix64::new(0);
    assert_eq!(sm.next_u64(), 0xe220_a839_7b1d_cdaf);
    assert_eq!(sm.next_u64(), 0x6e78_9e6a_a1b9_65f4);
    assert_eq!(sm.next_u64(), 0x06c4_5d18_8009_454f);
}

#[test]
fn xoshiro_outputs_are_pinned() {
    let mut r = Rng::new(42);
    let got: Vec<u64> = (0..4).map(|_| r.next_u64()).collect();
    assert_eq!(got, PINNED_XOSHIRO_42);
}

#[test]
fn hashes_are_pinned() {
    let got = [
        hash_one(&0u64),
        hash_one(&1u64),
        hash_one("coopt"),
        hash_one(&("a", 7u32)),
    ];
    assert_eq!(got, PINNED_HASHES);
}

#[test]
fn short_byte_strings_that_differ_only_in_length_hash_apart() {
    let mut a = kayak_base::FxHasher::default();
    let mut b = kayak_base::FxHasher::default();
    std::hash::Hasher::write(&mut a, b"a");
    std::hash::Hasher::write(&mut b, b"a\0");
    assert_ne!(std::hash::Hasher::finish(&a), std::hash::Hasher::finish(&b));
}

#[test]
fn a_fixed_map_iterates_in_the_same_order_every_time() {
    let order = || {
        let mut m = FixedMap::default();
        for i in 0..100u32 {
            m.insert(format!("key{i}"), i);
        }
        m.into_values().collect::<Vec<_>>()
    };
    let first = order();
    assert_eq!(first, order());
    // And the order is pinned, not merely stable within this process.
    assert_eq!(&first[..8], PINNED_MAP_ORDER);
}

#[test]
fn below_stays_in_range_and_reaches_every_value() {
    let mut r = Rng::new(7);
    let mut seen = [0u32; 6];
    for _ in 0..6000 {
        let x = r.below(6);
        seen[x as usize] += 1;
    }
    assert!(seen.iter().all(|&n| n > 800), "{seen:?}");
}

#[test]
fn split_streams_are_independent_of_the_order_they_are_taken() {
    let root = Rng::new(1);
    let mut a1 = root.split("sched");
    let mut b1 = root.split("stress");
    let mut b2 = root.split("stress");
    let mut a2 = root.split("sched");
    assert_eq!(a1.next_u64(), a2.next_u64());
    assert_eq!(b1.next_u64(), b2.next_u64());
    assert_ne!(
        root.split("sched").next_u64(),
        root.split("stress").next_u64()
    );
}

// Recorded from this implementation on 2026-10-10. They pin the outputs;
// they were not checked against an outside reference.
const PINNED_XOSHIRO_42: [u64; 4] = [
    1546998764402558742,
    6990951692964543102,
    12544586762248559009,
    17057574109182124193,
];
// FxHash takes the zero word to zero: it is a fast table hash, not a
// fingerprint (`TODO.md` §26).
const PINNED_HASHES: [u64; 4] = [
    0,
    5871781006564002453,
    508709074503017124,
    2307813877579403552,
];
const PINNED_MAP_ORDER: &[u32] = &[13, 6, 58, 67, 70, 74, 83, 87];
