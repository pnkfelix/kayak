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
        if got != want {
            wrong.push(format!("{name}:\n  Fixpt: {want:?}\n  Rust:  {got:?}"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
