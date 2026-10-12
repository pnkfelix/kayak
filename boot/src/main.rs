//! kfx, the compiler: `kfx/main.kfx`'s `main`, run on a big stack. Its
//! uses, and its I/O, are KFX's (`CLAUDES_CHOICES.md` C50).
//!
//! The compiler is generated from `kfx/` (`PLAN.md` §1); this file is the
//! only hand-written part of it.

#[allow(warnings, clippy::all)]
#[rustfmt::skip]
mod kfx;

use kayak_rt::{int_to_i64, with_big_stack};

fn main() {
    let status = with_big_stack(|| int_to_i64(kfx::main()));
    kayak_rt::heap_stats_report();
    std::process::exit((status & 0xff) as i32);
}
