//! kc, kayak's compiler from scratch (`docs/research/kc.md`): `kc/main.kfx`'s
//! `main`, run on a big stack. Generated from `kc/` (`scripts/regen.sh
//! --all`); this file is the only hand-written part.

#[allow(warnings, clippy::all)]
#[rustfmt::skip]
mod kc;

use kayak_rt::{int_to_i64, with_big_stack};

/// kayak's own allocator (C370).
#[global_allocator]
static ALLOC: kayak_sys::Alloc = kayak_sys::Alloc;

fn main() {
    kayak_rt::set_args(std::env::args().skip(1).collect());
    let status = with_big_stack(|| int_to_i64(kc::main()));
    kayak_rt::finish();
    // What the heap was given, by type, with the `heap-stats` feature
    // (as `boot`'s and tier 1's); nothing without it.
    kayak_rt::heap_stats_report();
    std::process::exit(status as i32);
}
