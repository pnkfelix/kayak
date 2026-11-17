//! The code area's room given back (C456), as Fixpt's collected code
//! area's test (`crates/fixpt-native/tests/code_gc.rs`) has it: code
//! sealed, run and dropped, over and over, past what the area holds at
//! once, never full; and code freed never run again.

use kayak_jit::{area_use, call, closure, free, last_sealed, layout, live, native_code, seal};

/// The first two arguments' sum, then `pad` bytes never reached.
fn add2(pad: usize) -> Vec<u8> {
    #[cfg(target_arch = "aarch64")]
    let mut code: Vec<u8> = [0x8b02_0020u32, 0xd65f_03c0]
        .iter()
        .flat_map(|w| w.to_le_bytes())
        .collect();
    #[cfg(not(target_arch = "aarch64"))]
    let mut code = vec![0x48, 0x8d, 0x04, 0x37, 0xc3];
    code.resize(code.len() + pad, 0);
    code
}

fn fixnum(n: i64) -> u64 {
    (n << layout::FIXNUM_SHIFT) as u64
}

#[test]
fn code_freed_makes_room_past_the_area() {
    if !native_code() {
        return;
    }
    // 1 MiB a round, 600 rounds: more than twice the area's 256 MiB.
    let (top0, _) = area_use();
    for k in 0..600 {
        let c = seal(layout::KIND_BLOBLET, &[], false, &add2(1 << 20));
        assert_eq!(last_sealed(), c.entry());
        assert_eq!(
            call(closure(c), c.entry(), &[fixnum(k), fixnum(1)]).unwrap(),
            fixnum(k + 1)
        );
        assert!(live(c));
        assert!(free(c.entry()));
        assert!(!live(c));
        assert!(!free(c.entry()), "freed twice");
    }
    // Each round's block the one the round before gave back.
    assert_eq!(area_use().0, top0);
    blocks_freed_are_joined_and_used_again();
}

/// In the same test as the one above: the area is the process's, and its
/// tests' threads would take each other's blocks.
fn blocks_freed_are_joined_and_used_again() {
    let cs: Vec<_> = (0..4)
        .map(|_| seal(layout::KIND_BLOBLET, &[], false, &add2(4096)))
        .collect();
    let (top, _) = area_use();
    // The middle two freed: one block of both, below the top.
    assert!(free(cs[1].entry()));
    assert!(free(cs[2].entry()));
    let (_, given_back) = area_use();
    assert!(given_back >= 2 * 4096);
    // A seal of both's size fits there, first fit: the top as it was.
    let big = seal(layout::KIND_BLOBLET, &[], false, &add2(2 * 4096));
    assert_eq!(big.entry(), cs[1].entry());
    assert_eq!(area_use().0, top);
    assert!(live(cs[0]) && live(cs[3]) && live(big) && !live(cs[1]) && !live(cs[2]));
    // An address of no code: nothing freed.
    assert!(!free(cs[0].entry() + 8));
    assert!(!free(0));
    for c in [cs[0], cs[3], big] {
        assert!(free(c.entry()));
    }
}
