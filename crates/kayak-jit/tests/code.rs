//! Code sealed into the code area and run, as Fixpt's
//! `programs/made-code/leaves.fx` does from FX-26: each machine's code
//! made by [`machine`], the same tests on both.

use kayak_jit::{
    Trap, call, card_marked, closure, closure_code, cons, foreign_procedure, header_of, layout,
    native_code, pair, seal, slot_of, state, string, string_of, suffix_word,
};

/// The code the tests run, in each machine's instructions.
#[cfg(target_arch = "aarch64")]
mod machine {
    use kayak_jit::state;

    fn words(code: &[u32]) -> Vec<u8> {
        code.iter().flat_map(|w| w.to_le_bytes()).collect()
    }

    /// The first two arguments' sum: add x0, x1, x2; ret.
    pub fn add2() -> Vec<u8> {
        words(&[0x8b02_0020, 0xd65f_03c0])
    }

    /// Field 0, after the trailer (slot 2): ldr x0, <16 bytes back>; ret.
    pub fn field0() -> Vec<u8> {
        words(&[0x58ff_ff80, 0xd65f_03c0])
    }

    /// x0 = x1 + ... + x8, and x9, the count.
    pub fn sum_all() -> Vec<u8> {
        words(&[
            0x8b02_0020, // add x0, x1, x2
            0x8b03_0000, // add x0, x0, x3
            0x8b04_0000, // add x0, x0, x4
            0x8b05_0000, // add x0, x0, x5
            0x8b06_0000, // add x0, x0, x6
            0x8b07_0000, // add x0, x0, x7
            0x8b08_0000, // add x0, x0, x8
            0x8b09_0000, // add x0, x0, x9
            0xd65f_03c0, // ret
        ])
    }

    /// Code that makes a frame, calls the routine at state field `field`
    /// after `before`, and returns what it gives after `after`.
    pub fn calling(before: &[u8], field: usize, after: &[u8]) -> Vec<u8> {
        let ldr = 0xf940_0000 | ((field / 8) as u32) << 10 | 24 << 5 | 16; // ldr x16, [x24, #field]
        let mut code = words(&[0xa9bf_7bfd, 0x9100_03fd]); // stp x29, x30, [sp, #-16]!; mov x29, sp
        code.extend_from_slice(before);
        code.extend(words(&[ldr, 0xd63f_0200])); // blr x16
        code.extend_from_slice(after);
        code.extend(words(&[0xa8c1_7bfd, 0xd65f_03c0])); // ldp x29, x30, [sp], #16; ret
        code
    }

    /// `(f x)`, `f` the first argument and `x` the second, by `foreign`:
    /// mov x10, x1; mov x1, x2; movz x9, #1.
    pub fn call_first_on_second() -> Vec<u8> {
        words(&[0xaa01_03ea, 0xaa02_03e1, 0xd280_0029])
    }

    /// The fuel spent: movz x28, #0.
    pub fn spend_fuel() -> Vec<u8> {
        words(&[0xd280_001c])
    }

    /// The fuel answered: mov x0, x28.
    pub fn answer_fuel() -> Vec<u8> {
        words(&[0xaa1c_03e0])
    }

    /// A trap's kind set: movz x9, #kind.
    pub fn trap_kind(kind: u32) -> Vec<u8> {
        words(&[0xd280_0009 | kind << 5])
    }

    /// The second argument stored into field 0 of the first, a bloblet of
    /// no suffix, and its card marked: str x2, [x1, #-16]; lsr x16, x1,
    /// #9; ldr x17, [x24, #CARDS]; add x17, x17, x16; movz x16, #1; strb
    /// w16, [x17]; ret.
    pub fn store_marking() -> Vec<u8> {
        let ldr = 0xf940_0000 | ((state::CARDS / 8) as u32) << 10 | 24 << 5 | 17;
        words(&[
            0xf81e_c022,
            0xd349_fc30,
            ldr,
            0x8b10_0231,
            0xd280_0030,
            0x3900_0230,
            0xd65f_03c0,
        ])
    }

    /// `n` instructions, the last a return.
    pub fn nops(n: usize) -> Vec<u8> {
        let mut code = vec![0xd503_201f_u32; n - 1];
        code.push(0xd65f_03c0);
        words(&code)
    }
}

/// The code the tests run, in each machine's instructions.
#[cfg(target_arch = "x86_64")]
mod machine {
    use kayak_jit::state;

    /// `op [r15 + field]`, `rex` and `op` its prefix and opcode, `reg` the
    /// ModRM's register (or opcode extension): r15 by REX.B, disp32.
    fn r15(rex: u8, op: u8, reg: u8, field: usize) -> Vec<u8> {
        let mut code = vec![rex | 1, op, 0x87 | (reg & 7) << 3];
        code.extend((field as u32).to_le_bytes());
        code
    }

    /// The first two arguments' sum: lea rax, [rdi + rsi]; ret.
    pub fn add2() -> Vec<u8> {
        vec![0x48, 0x8d, 0x04, 0x37, 0xc3]
    }

    /// Field 0, after the trailer (slot 2), 16 bytes before the code:
    /// mov rax, [rip - 23]; ret.
    pub fn field0() -> Vec<u8> {
        let mut code = vec![0x48, 0x8b, 0x05];
        code.extend((-23i32).to_le_bytes());
        code.push(0xc3);
        code
    }

    /// rax = rdi + rsi + r8 + the state's reg4 ... reg8, and r10, the count.
    pub fn sum_all() -> Vec<u8> {
        let mut code = vec![
            0x48, 0x89, 0xf8, // mov rax, rdi
            0x48, 0x01, 0xf0, // add rax, rsi
            0x4c, 0x01, 0xc0, // add rax, r8
        ];
        for k in 0..5 {
            code.extend(r15(0x48, 0x03, 0, state::REG4 + 8 * k)); // add rax, [r15 + regK]
        }
        code.extend([0x4c, 0x01, 0xd0, 0xc3]); // add rax, r10; ret
        code
    }

    /// Code that makes a frame, calls the routine at state field `field`
    /// after `before`, and returns what it gives after `after`: push rbp;
    /// mov rbp, rsp; call [r15 + field]; pop rbp; ret.
    pub fn calling(before: &[u8], field: usize, after: &[u8]) -> Vec<u8> {
        let mut code = vec![0x55, 0x48, 0x89, 0xe5];
        code.extend_from_slice(before);
        code.extend(r15(0x40, 0xff, 2, field));
        code.extend_from_slice(after);
        code.extend([0x5d, 0xc3]);
        code
    }

    /// `(f x)`, `f` the first argument and `x` the second, by `foreign`:
    /// mov rbx, rdi; mov rdi, rsi; mov r10d, 1.
    pub fn call_first_on_second() -> Vec<u8> {
        vec![0x48, 0x89, 0xfb, 0x48, 0x89, 0xf7, 0x41, 0xba, 1, 0, 0, 0]
    }

    /// The fuel spent: mov qword [r15 + fuel], 0.
    pub fn spend_fuel() -> Vec<u8> {
        let mut code = r15(0x48, 0xc7, 0, state::FUEL);
        code.extend([0, 0, 0, 0]);
        code
    }

    /// The fuel answered: mov rax, [r15 + fuel].
    pub fn answer_fuel() -> Vec<u8> {
        r15(0x48, 0x8b, 0, state::FUEL)
    }

    /// A trap's kind set: mov r10d, kind.
    pub fn trap_kind(kind: u32) -> Vec<u8> {
        let mut code = vec![0x41, 0xba];
        code.extend(kind.to_le_bytes());
        code
    }

    /// The second argument stored into field 0 of the first, a bloblet of
    /// no suffix, and its card marked: mov [rdi - 20], rsi; mov rax, rdi;
    /// shr rax, 9; add rax, [r15 + cards]; mov byte [rax], 1; ret.
    pub fn store_marking() -> Vec<u8> {
        let mut code = vec![
            0x48, 0x89, 0x77, 0xec, 0x48, 0x89, 0xf8, 0x48, 0xc1, 0xe8, 9,
        ];
        code.extend(r15(0x48, 0x03, 0, state::CARDS));
        code.extend([0xc6, 0x00, 0x01, 0xc3]);
        code
    }

    /// `n` instructions, the last a return.
    pub fn nops(n: usize) -> Vec<u8> {
        let mut code = vec![0x90; n - 1];
        code.push(0xc3);
        code
    }
}

use machine::*;

fn fixnum(n: i64) -> u64 {
    (n << layout::FIXNUM_SHIFT) as u64
}

#[test]
fn a_leaf_adds_two_fixnums() {
    if !native_code() {
        return;
    }
    let c = seal(layout::KIND_BLOBLET, &[], false, &add2());
    let f = closure(c);
    assert_eq!(closure_code(f), c.reference());
    assert_eq!(
        call(f, c.entry(), &[fixnum(20), fixnum(22)]).unwrap(),
        fixnum(42)
    );
    assert_eq!(c.byte(0), add2()[0]);
}

#[test]
fn a_leaf_reads_its_own_field() {
    if !native_code() {
        return;
    }
    let c = seal(
        layout::KIND_BLOBLET,
        &[layout::trailer(2), fixnum(41)],
        false,
        &field0(),
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
    let c = seal(layout::KIND_BLOBLET, &[], false, &sum_all());
    let args: Vec<u64> = (1..=8).map(fixnum).collect();
    assert_eq!(call(closure(c), c.entry(), &args).unwrap(), fixnum(36) + 8);
}

#[test]
#[cfg(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(target_os = "linux", target_arch = "x86_64")
))]
fn this_machine_runs_native_code() {
    assert!(native_code());
}

fn run(code: &[u8], args: &[u64]) -> Result<u64, Trap> {
    let c = seal(layout::KIND_BLOBLET, &[], false, code);
    call(closure(c), c.entry(), args)
}

#[test]
fn cons_makes_a_pair() {
    if !native_code() {
        return;
    }
    let p = run(&calling(&[], state::CONS, &[]), &[fixnum(1), layout::NULL]).unwrap();
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
        run(
            &calling(&call_first_on_second(), state::FOREIGN, &[]),
            &[f, fixnum(41)]
        ),
        Ok(fixnum(42))
    );
}

#[test]
fn rust_called_back_calls_native_code_again() {
    if !native_code() {
        return;
    }
    let adder = seal(layout::KIND_BLOBLET, &[], false, &add2());
    let f = foreign_procedure(Box::new(move |a: &[u64]| {
        call(closure(adder), adder.entry(), &[a[0], fixnum(2)]).unwrap()
    }));
    let outer = calling(&call_first_on_second(), state::FOREIGN, &[]);
    assert_eq!(run(&outer, &[f, fixnum(40)]), Ok(fixnum(42)));
    // And a foreign procedure that runs native code that calls a foreign one.
    let g = foreign_procedure(Box::new(|a: &[u64]| a[0] * 2));
    let c = seal(layout::KIND_BLOBLET, &[], false, &outer);
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
    let code = calling(&call_first_on_second(), state::FOREIGN, &[]);
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
    let inner = seal(layout::KIND_BLOBLET, &[], false, &code);
    let g = foreign_procedure(Box::new(move |_: &[u64]| {
        call(closure(inner), inner.entry(), &[f, fixnum(0)]).unwrap()
    }));
    let e = std::panic::catch_unwind(|| run(&code, &[g, fixnum(0)])).unwrap_err();
    assert_eq!(e.downcast_ref::<&str>(), Some(&"boom"));
}

/// `call` is re-entrant (C228): native code calls a procedure by `foreign`,
/// whose Rust calls native code again, which calls another; each returns to
/// its own caller, the trampolines nested, and native code goes on after.
#[test]
fn calls_nest_through_foreign() {
    if !native_code() {
        return;
    }
    let code = calling(&call_first_on_second(), state::FOREIGN, &[]);
    let inner = seal(layout::KIND_BLOBLET, &[], false, &code);
    let add1 = foreign_procedure(Box::new(|a: &[u64]| a[0] + fixnum(1)));
    let twice = foreign_procedure(Box::new(move |a: &[u64]| {
        call(closure(inner), inner.entry(), &[add1, a[0]]).unwrap() + fixnum(1)
    }));
    assert_eq!(run(&code, &[twice, fixnum(5)]), Ok(fixnum(7)));
    let thrice = foreign_procedure(Box::new(move |a: &[u64]| {
        call(closure(inner), inner.entry(), &[twice, a[0]]).unwrap() + fixnum(1)
    }));
    assert_eq!(run(&code, &[thrice, fixnum(5)]), Ok(fixnum(8)));
}

#[test]
fn poll_gives_fuel() {
    if !native_code() {
        return;
    }
    // The fuel spent, then poll, then the fuel returned.
    let code = calling(&spend_fuel(), state::POLL, &answer_fuel());
    assert_eq!(run(&code, &[]), Ok(1 << 20));
}

#[test]
fn traps_and_routines_kayak_lacks_end_the_call() {
    if !native_code() {
        return;
    }
    assert_eq!(
        run(&calling(&trap_kind(5), state::TRAP_STUB, &[]), &[]),
        Err(Trap::Kind(5))
    );
    assert_eq!(
        run(&calling(&[], state::OVERFLOW, &[]), &[]),
        Err(Trap::Kind(kayak_jit::STACK_OVERFLOW))
    );
    assert_eq!(
        run(&calling(&[], state::CLOSURE, &[]), &[]),
        Err(Trap::Routine("closure"))
    );
    assert_eq!(run(&add2(), &[fixnum(1), fixnum(2)]), Ok(fixnum(3)));
}

#[test]
fn bloblet_makes_one_zeroed_or_refuses() {
    if !native_code() {
        return;
    }
    // The bytes and fields, as given.
    let b = run(&calling(&[], state::BLOBLET, &[]), &[fixnum(12), fixnum(2)]).unwrap();
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
        run(&calling(&[], state::BLOBLET, &[]), &[fixnum(-1), fixnum(2)]),
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
    let b = kayak_jit::bloblet(&[0], &[], 0);
    assert!(!card_marked(b));
    let c = seal(layout::KIND_BLOBLET, &[], false, &store_marking());
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
            let slots: Vec<u64> = (0..fields).map(|i| fixnum(i as i64)).collect();
            let c = seal(layout::KIND_BLOBLET, &slots, false, &nops(n));
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
