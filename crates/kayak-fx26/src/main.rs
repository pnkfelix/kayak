//! kayak-fx26, FX-26's engine (`docs/research/engine.md`, C58):
//! `fx26/main.kfx`'s `main`, run on a big stack. Generated from `fx26/`
//! (`scripts/engine-programs.py --bless`); this file is the only
//! hand-written part.

#[allow(warnings, clippy::all)]
#[rustfmt::skip]
mod fx26;

use kayak_rt::{int_to_i64, with_big_stack};

fn main() {
    let status = with_big_stack(|| int_to_i64(fx26::main()));
    std::process::exit((status & 0xff) as i32);
}
