# kayak

KFX: a sublanguage of FX-26 in which every construct maps to something in
Rust, and a transpiler from KFX to Rust, written in KFX, that bootstraps.

- `kfx/kfx.kfx`: the transpiler, in KFX-0.
- `boot/`: the Rust it makes of itself (`boot/src/kfx.rs`, generated and
  committed: the seed), and a hand-written `main`.
- `crates/kayak-rt`: what the generated Rust calls.
- `kfx/prelude.fx`: what KFX has that FX-26 has only at `int`, in FX-26.
- [`PLAN.md`](PLAN.md): the design and the mapping.
- [`TODO.md`](TODO.md): every task not yet done, numbered.
- [`DONE.md`](DONE.md): what is done, and what was tried and abandoned,
  under the same numbers.
- [`CLAUDES_CHOICES.md`](CLAUDES_CHOICES.md): decisions made without
  asking.

## Checking the bootstrap

```
scripts/fixpoint.sh
```

builds `boot/`, runs it on `kfx/kfx.kfx`, and requires `boot/src/kfx.rs`
back, byte for byte. It also runs stage 0: Fixpt, at the commit in
`FIXPT_REV`, running `kfx/kfx.kfx` as FX-26, must make the same file.
`scripts/fixpt.sh` builds that Fixpt from the clone at
`~/Dev/Rust/kayak-refs/fixpt` (or `KAYAK_FIXPT_CLONE`), and refuses a
clone at any other commit. With no clone, stage 0 is skipped, and the
script says so.

## Changing the compiler

Edit `kfx/kfx.kfx`, then regenerate with the compiler you have:

```
cargo build --release -p kfx
./target/release/kfx kfx/kfx.kfx > boot/src/kfx.rs
```

and repeat until `scripts/fixpoint.sh` passes. A feature goes in two
steps: first teach the compiler to translate it, regenerate, and only then
use it in `kfx.kfx`.
