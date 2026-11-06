// coopt-source: crates/coopt-base/tests/fingerprint.rs@9283bba
use std::collections::BTreeMap;

use kayak_base::fingerprint::Sip128;
use kayak_base::{Fingerprint, fingerprint};

const K0: u64 = 0x0706_0504_0302_0100;
const K1: u64 = 0x0f0e_0d0c_0b0a_0908;

/// (length of the input 0, 1, 2, …; first half; second half), for the
/// standard test key. Produced by `siphasher` 1.0.3's `sip128` (run
/// outside this repository, 2026-10-10). Its 2-4 output for the empty
/// input is also the first vector of SipHash's reference `vectors.h`
/// (`a3 81 7f 04 ba 25 a8 e6 …`), which fixes the byte order.
const SIP13: [(usize, u64, u64); 10] = [
    (0, 0xbea58827b2bc7ee7, 0x013030dd6adb62fd),
    (1, 0xa8edd36004376ffc, 0x63f02f2bcc73055e),
    (7, 0xc3e0aaf223b98410, 0x77ab4808c82e2fa6),
    (8, 0xb4dae3d5e1fe12aa, 0x99c7f935ab164f72),
    (9, 0x9439f32c04b8dd81, 0x427c1394000e72f4),
    (15, 0x6c52bdb205557ec1, 0x09017e1eeccd2129),
    (16, 0xeb8e511557d9a8d0, 0x93179e3df8b013b5),
    (17, 0x8d66bb3ff9264123, 0xecf7bdfee8125197),
    (31, 0x8b853c5b1c96ea42, 0x1090a7df50e5c317),
    (63, 0x6f42fe4ee300584c, 0xad6052a70a6b9f07),
];
const SIP24: [(usize, u64, u64); 10] = [
    (0, 0xe6a825ba047f81a3, 0x930255c71472f66d),
    (1, 0x44af996bd8c187da, 0x45fc229b11597634),
    (7, 0x53c1dbd8beebf1a1, 0x3982f01fa64ab8c0),
    (8, 0x61f55862baa9623b, 0xb49714f364e2830f),
    (9, 0xabbad90a06994426, 0xed716dbb028b7fc4),
    (15, 0x11a8b03399e99354, 0xd9c3cf970fec087e),
    (16, 0xbb54b067caa4e26e, 0x77052385bf1533fd),
    (17, 0x98b88d73e8063d47, 0x4077e47ac466c054),
    (31, 0xfcfa233218b03929, 0x353dc4524fde2317),
    (63, 0x4a83502f77d15051, 0x7cbd3f979a063e50),
];

fn input() -> Vec<u8> {
    (0..64u8).collect()
}

#[test]
fn sip128_matches_the_vectors() {
    let buf = input();
    for (n, a, b) in SIP13 {
        let mut s = Sip128::<1, 3>::new(K0, K1);
        s.write(&buf[..n]);
        assert_eq!(s.finish(), (a, b), "1-3, {n} bytes");
    }
    for (n, a, b) in SIP24 {
        let mut s = Sip128::<2, 4>::new(K0, K1);
        s.write(&buf[..n]);
        assert_eq!(s.finish(), (a, b), "2-4, {n} bytes");
    }
}

#[test]
fn writing_in_pieces_is_writing_at_once() {
    let buf = input();
    for n in [0, 1, 7, 8, 9, 15, 16, 17, 31, 63] {
        let mut whole = Sip128::<1, 3>::new(K0, K1);
        whole.write(&buf[..n]);
        for cut in 0..=n {
            let mut parts = Sip128::<1, 3>::new(K0, K1);
            parts.write(&buf[..cut]);
            parts.write(&buf[cut..n]);
            assert_eq!(parts.finish(), whole.finish(), "{n} cut at {cut}");
        }
    }
}

#[test]
fn encodings_keep_apart_what_differs() {
    // Lengths come first, so a split moved between two strings shows.
    assert_ne!(fingerprint(&("ab", "c")), fingerprint(&("a", "bc")));
    assert_ne!(fingerprint(&Some(0u64)), fingerprint(&None::<u64>));
    // The encoding does not describe types: `0u64` and an empty vector
    // both write a zero word. Fingerprints only compare values of one type.
    assert_eq!(fingerprint(&0u64), fingerprint(&Vec::<u64>::new()));
    // Widths are fixed: a u8 and a u64 of the same number encode alike.
    assert_eq!(fingerprint(&7u8), fingerprint(&7u64));
    // A map's encoding follows its keys, not the order of insertion.
    let a: BTreeMap<_, _> = [(2, "b"), (1, "a")].into_iter().collect();
    let b: BTreeMap<_, _> = [(1, "a"), (2, "b")].into_iter().collect();
    assert_eq!(fingerprint(&a), fingerprint(&b));
}

#[test]
fn fingerprints_are_pinned() {
    // Recorded from this implementation, 2026-10-10. A change here changes
    // every fingerprint, and anything persisted with them.
    assert_eq!(fingerprint(&()), PINNED_UNIT);
    assert_eq!(fingerprint("coopt"), PINNED_COOPT);
}

const PINNED_UNIT: Fingerprint = Fingerprint(13349417439097792157, 467468203384067810);
const PINNED_COOPT: Fingerprint = Fingerprint(5655536748887989433, 9555169417518657443);
