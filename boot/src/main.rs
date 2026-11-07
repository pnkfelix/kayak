//! kfx, the compiler: `kfx/main.kfx`'s `main`, run on a big stack to its
//! end. Its uses, and its I/O, are KFX's (`CLAUDES_CHOICES.md` C50, C52):
//! `main` may suspend, to the runner its reads and writes go to, so it is
//! an `async fn`, run as a task of a scheduler of its own (`run`, C378),
//! which its fuel preempts when it is built in the async mode.
//!
//! The compiler is generated from `kfx/` (`PLAN.md` §1); this file is the
//! only hand-written part of it.

#[allow(warnings, clippy::all)]
#[rustfmt::skip]
mod kfx;

use kayak_rt::{int_to_i64, suspend::run, unreachable_unwind, with_big_stack};

/// kayak's own allocator (C370): small blocks from per-thread size classes.
#[global_allocator]
static ALLOC: kayak_sys::Alloc = kayak_sys::Alloc;

fn main() {
    let status =
        with_big_stack(|| int_to_i64(run(kfx::main()).unwrap_or_else(|u| unreachable_unwind(u))));
    kayak_rt::heap_stats_report();
    std::process::exit((status & 0xff) as i32);
}
