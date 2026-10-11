//! Each corpus program's Rust gives the answer Fixpt gives running it as
//! FX-26 (`tests/kfx/NAME.out`, `scripts/corpus.py`): `PLAN.md`'s K1,
//! program by program.

#[test]
fn every_program_gives_fixpts_answer() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/kfx");
    let mut wrong = Vec::new();
    for &(name, main) in kfx_corpus::PROGRAMS {
        let want = std::fs::read_to_string(format!("{dir}/{name}.out"))
            .unwrap_or_else(|e| panic!("{name}.out: {e}"));
        let got = kayak_rt::with_big_stack(main);
        if got.as_str() != want {
            wrong.push(format!("{name}:\n  Fixpt: {want:?}\n  Rust:  {got:?}"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// `tails.kfx`'s procedures loop: run on a stack of 256 KiB, as no
/// recursion three million deep could be.
#[test]
fn self_tail_calls_run_in_constant_stack() {
    let (_, main) = kfx_corpus::PROGRAMS
        .iter()
        .find(|(name, _)| *name == "tails")
        .expect("tails.kfx is in the corpus");
    let main = *main;
    let got = std::thread::Builder::new()
        .stack_size(256 << 10)
        .spawn(main)
        .unwrap()
        .join()
        .expect("tails.kfx overflows a small stack: its self tail calls do not loop");
    assert!(got.as_str().starts_with("4499998500000 "), "{got}");
}
