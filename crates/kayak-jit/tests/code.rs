//! Code sealed into the code area and run, as Fixpt's
//! `programs/made-code/leaves.fx` does from FX-26.

use kayak_jit::{Trap, call, closure, closure_code, layout, native_code, seal, state};

fn words(code: &[u32]) -> Vec<u8> {
    code.iter().flat_map(|w| w.to_le_bytes()).collect()
}

fn fixnum(n: i64) -> u64 {
    (n << layout::FIXNUM_SHIFT) as u64
}

#[test]
fn a_leaf_adds_two_fixnums() {
    if !native_code() {
        return;
    }
    // add x0, x1, x2; ret
    let c = seal(
        layout::KIND_BLOBLET,
        &[],
        false,
        &words(&[0x8b02_0020, 0xd65f_03c0]),
    );
    let f = closure(c);
    assert_eq!(closure_code(f), c.reference());
    assert_eq!(
        call(f, c.entry(), &[fixnum(20), fixnum(22)]).unwrap(),
        fixnum(42)
    );
    assert_eq!(c.byte(0), 0x20);
}

#[test]
fn a_leaf_reads_its_own_field() {
    if !native_code() {
        return;
    }
    // ldr x0, <16 bytes back>; ret: field 0, after the trailer (slot 1).
    let c = seal(
        layout::KIND_BLOBLET,
        &[layout::trailer(2), fixnum(41)],
        false,
        &words(&[0x58ff_ff80, 0xd65f_03c0]),
    );
    let f = closure(c);
    assert_eq!(call(f, c.entry(), &[]).unwrap(), fixnum(41));
    c.set_slot(2, fixnum(7));
    assert_eq!(call(f, c.entry(), &[]).unwrap(), fixnum(7));
}

#[test]
fn every_argument_arrives() {
    if !native_code() {
        return;
    }
    // x0 = x1 + ... + x8, and x9, the count.
    let code = [
        0x8b02_0020u32, // add x0, x1, x2
        0x8b03_0000,    // add x0, x0, x3
        0x8b04_0000,    // add x0, x0, x4
        0x8b05_0000,    // add x0, x0, x5
        0x8b06_0000,    // add x0, x0, x6
        0x8b07_0000,    // add x0, x0, x7
        0x8b08_0000,    // add x0, x0, x8
        0x8b09_0000,    // add x0, x0, x9
        0xd65f_03c0,    // ret
    ];
    let c = seal(layout::KIND_BLOBLET, &[], false, &words(&code));
    let args: Vec<u64> = (1..=8).map(fixnum).collect();
    assert_eq!(call(closure(c), c.entry(), &args).unwrap(), fixnum(36) + 8);
}

#[test]
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn this_machine_runs_native_code() {
    assert!(native_code());
}

#[test]
fn a_routine_kayak_lacks_comes_back_as_a_trap() {
    if !native_code() {
        return;
    }
    // ldr x16, [x24, #CONS]; blr x16: the `cons` routine.
    let ldr = 0xf940_0000 | ((state::CONS / 8) as u32) << 10 | 24 << 5 | 16;
    let code = [
        0xa9bf_7bfd,
        0x9100_03fd,
        ldr,
        0xd63f_0200,
        0xa8c1_7bfd,
        0xd65f_03c0,
    ];
    let c = seal(layout::KIND_BLOBLET, &[], false, &words(&code));
    assert_eq!(call(closure(c), c.entry(), &[]), Err(Trap::Routine("cons")));
    // And the trap routine, kind 5 in x9.
    let ldr = 0xf940_0000 | ((state::TRAP_STUB / 8) as u32) << 10 | 24 << 5 | 16;
    let code = [0xd280_00a9, ldr, 0xd63f_0200, 0xd65f_03c0]; // movz x9, #5
    let c = seal(layout::KIND_BLOBLET, &[], false, &words(&code));
    assert_eq!(call(closure(c), c.entry(), &[]), Err(Trap::Kind(5)));
    // After which calls go on as before.
    let c = seal(
        layout::KIND_BLOBLET,
        &[],
        false,
        &words(&[0x8b02_0020, 0xd65f_03c0]),
    );
    assert_eq!(
        call(closure(c), c.entry(), &[fixnum(1), fixnum(2)]),
        Ok(fixnum(3))
    );
}
