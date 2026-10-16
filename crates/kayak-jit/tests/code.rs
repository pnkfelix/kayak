//! Code sealed into the code area and run, as Fixpt's
//! `programs/made-code/leaves.fx` does from FX-26.

use kayak_jit::{
    Trap, call, card_marked, closure, closure_code, cons, foreign_procedure, header_of, layout,
    native_code, pair, seal, slot_of, state, string, string_of, suffix_word,
};

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

/// Code that makes a frame, calls the routine at state field `field`, and
/// returns what it gives, after `before`.
fn calling(before: &[u32], field: usize) -> Vec<u32> {
    let ldr = 0xf940_0000 | ((field / 8) as u32) << 10 | 24 << 5 | 16; // ldr x16, [x24, #field]
    let mut code = vec![0xa9bf_7bfd, 0x9100_03fd]; // stp x29, x30, [sp, #-16]!; mov x29, sp
    code.extend_from_slice(before);
    code.extend_from_slice(&[ldr, 0xd63f_0200, 0xa8c1_7bfd, 0xd65f_03c0]); // blr x16; ldp ...; ret
    code
}

fn run(code: &[u32], args: &[u64]) -> Result<u64, Trap> {
    let c = seal(layout::KIND_BLOBLET, &[], false, &words(code));
    call(closure(c), c.entry(), args)
}

/// `(f x)`, `f` in x1 and `x` in x2, by `foreign`.
const CALL_X1_ON_X2: [u32; 3] = [0xaa01_03ea, 0xaa02_03e1, 0xd280_0029]; // mov x10, x1; mov x1, x2; movz x9, #1

#[test]
fn cons_makes_a_pair() {
    if !native_code() {
        return;
    }
    let p = run(&calling(&[], state::CONS), &[fixnum(1), layout::NULL]).unwrap();
    assert_eq!(pair(p), Some((fixnum(1), layout::NULL)));
    assert_eq!(pair(cons(fixnum(2), p)), Some((fixnum(2), p)));
}

#[test]
fn foreign_calls_rust() {
    if !native_code() {
        return;
    }
    let f = foreign_procedure(Box::new(|a: &[u64]| a[0] + fixnum(1)));
    assert_eq!(
        run(&calling(&CALL_X1_ON_X2, state::FOREIGN), &[f, fixnum(41)]),
        Ok(fixnum(42))
    );
}

#[test]
fn rust_called_back_calls_native_code_again() {
    if !native_code() {
        return;
    }
    let adder = seal(
        layout::KIND_BLOBLET,
        &[],
        false,
        &words(&[0x8b02_0020, 0xd65f_03c0]),
    );
    let f = foreign_procedure(Box::new(move |a: &[u64]| {
        call(closure(adder), adder.entry(), &[a[0], fixnum(2)]).unwrap()
    }));
    let outer = calling(&CALL_X1_ON_X2, state::FOREIGN);
    assert_eq!(run(&outer, &[f, fixnum(40)]), Ok(fixnum(42)));
    // And a foreign procedure that runs native code that calls a foreign one.
    let g = foreign_procedure(Box::new(|a: &[u64]| a[0] * 2));
    let c = seal(layout::KIND_BLOBLET, &[], false, &words(&outer));
    let h = foreign_procedure(Box::new(move |a: &[u64]| {
        call(closure(c), c.entry(), &[g, a[0]]).unwrap() + fixnum(1)
    }));
    assert_eq!(run(&outer, &[h, fixnum(5)]), Ok(fixnum(11)));
}

#[test]
fn a_panic_called_back_goes_on_past_native_code() {
    if !native_code() {
        return;
    }
    let f = foreign_procedure(Box::new(|_: &[u64]| panic!("boom")));
    let code = calling(&CALL_X1_ON_X2, state::FOREIGN);
    let e = std::panic::catch_unwind(|| run(&code, &[f, fixnum(0)])).unwrap_err();
    assert_eq!(e.downcast_ref::<&str>(), Some(&"boom"));
    // Calls go on as before, and through two trampolines too.
    assert_eq!(
        run(
            &code,
            &[foreign_procedure(Box::new(|a: &[u64]| a[0])), fixnum(3)]
        ),
        Ok(fixnum(3))
    );
    let inner = seal(layout::KIND_BLOBLET, &[], false, &words(&code));
    let g = foreign_procedure(Box::new(move |_: &[u64]| {
        call(closure(inner), inner.entry(), &[f, fixnum(0)]).unwrap()
    }));
    let e = std::panic::catch_unwind(|| run(&code, &[g, fixnum(0)])).unwrap_err();
    assert_eq!(e.downcast_ref::<&str>(), Some(&"boom"));
}

#[test]
fn poll_gives_fuel() {
    if !native_code() {
        return;
    }
    // mov x28, #0, then poll, then x28 returned.
    let mut code = calling(&[0xd280_001c], state::POLL);
    code.insert(code.len() - 2, 0xaa1c_03e0); // mov x0, x28, before the frame goes
    assert_eq!(run(&code, &[]), Ok(1 << 20));
}

#[test]
fn traps_and_routines_kayak_lacks_end_the_call() {
    if !native_code() {
        return;
    }
    assert_eq!(
        run(&calling(&[0xd280_00a9], state::TRAP_STUB), &[]),
        Err(Trap::Kind(5))
    ); // movz x9, #5
    assert_eq!(
        run(&calling(&[], state::OVERFLOW), &[]),
        Err(Trap::Kind(kayak_jit::STACK_OVERFLOW))
    );
    assert_eq!(
        run(&calling(&[], state::CLOSURE), &[]),
        Err(Trap::Routine("closure"))
    );
    assert_eq!(
        run(&[0x8b02_0020, 0xd65f_03c0], &[fixnum(1), fixnum(2)]),
        Ok(fixnum(3))
    );
}

#[test]
fn bloblet_makes_one_zeroed_or_refuses() {
    if !native_code() {
        return;
    }
    // x1 bytes, x2 fields, as given.
    let b = run(&calling(&[], state::BLOBLET), &[fixnum(12), fixnum(2)]).unwrap();
    assert_eq!(header_of(b), Some((layout::KIND_BLOBLET, 3, 12)));
    assert_eq!(slot_of(b, 1), layout::trailer(3));
    assert_eq!(
        (
            slot_of(b, 2),
            slot_of(b, 3),
            suffix_word(b, 0),
            suffix_word(b, 1)
        ),
        (0, 0, 0, 0)
    );
    assert_eq!(
        run(&calling(&[], state::BLOBLET), &[fixnum(-1), fixnum(2)]),
        Err(Trap::Failed(
            "bloblet: no bloblet of -1 bytes and 2 fields".into()
        ))
    );
}

#[test]
fn a_store_marks_its_card() {
    if !native_code() {
        return;
    }
    // str x2, [x1, #-16] (field 0 of a bloblet of no suffix, from its
    // reference less 4); then its card: lsr x16, x1, #9; ldr x17, [x24,
    // #CARDS]; add x17, x17, x16; movz x16, #1; strb w16, [x17].
    let b = kayak_jit::bloblet(&[0], &[], 0);
    assert!(!card_marked(b));
    let ldr = 0xf940_0000 | ((state::CARDS / 8) as u32) << 10 | 24 << 5 | 17;
    let code = [
        0xf81e_c022u32,
        0xd349_fc30,
        ldr,
        0x8b10_0231,
        0xd280_0030,
        0x3900_0230,
        0xd65f_03c0,
    ];
    let c = seal(layout::KIND_BLOBLET, &[], false, &words(&code));
    call(closure(c), c.entry(), &[b, fixnum(9)]).unwrap();
    assert_eq!(slot_of(b, 2), fixnum(9));
    assert!(card_marked(b));
}

#[test]
fn strings_cross() {
    if !native_code() {
        return;
    }
    for t in ["", "a", "made", "été 🍁"] {
        assert_eq!(string_of(string(t)).as_deref(), Some(t));
    }
    assert_eq!(string_of(fixnum(3)), None);
}

#[test]
fn code_is_16_byte_aligned_in_both_views() {
    // Tier 5 (kayak's C131) keeps return points of its code, 8-aligned
    // within it, in frames the collector reads as fixnums: so every
    // sealed suffix starts 16-byte aligned, whatever its fields and size.
    if !native_code() {
        return;
    }
    for fields in 0..5usize {
        for n in [1usize, 2, 3, 5, 8] {
            let mut code = vec![0xd503_201f_u32; n - 1];
            code.push(0xd65f_03c0);
            let slots: Vec<u64> = (0..fields).map(|i| fixnum(i as i64)).collect();
            let c = seal(layout::KIND_BLOBLET, &slots, false, &words(&code));
            assert_eq!(
                c.entry() % 16,
                0,
                "{fields} fields, {n} instructions: the execute view"
            );
            assert_eq!(
                (c.reference() & !7) % 16,
                0,
                "{fields} fields, {n} instructions: the write view"
            );
        }
    }
}
