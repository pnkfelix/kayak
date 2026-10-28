//! kayak-fx26, FX-26's engine (`docs/research/engine.md`, C58, C64), and
//! its REPL (`docs/research/repl.md`): `fx26/start.kfx`'s `main`, run on a
//! big stack, which is the engine run on files, or the REPL, written in
//! KFX as FX-26 processes (C111, `fx26/live.kfx`), with nothing of it
//! here but its history file's path, given as an argument, since FX-26
//! reads no environment; and the terminal put back as it was, at the end,
//! as `fixpt` does. With the `rust-repl` feature, the REPL is C103's, its
//! threads and catching in Rust (`rust_repl.rs`), kept to compare the two.
//! Generated from `fx26/` (`scripts/engine-programs.py --bless`); this
//! file and `rust_repl.rs` are the only hand-written parts.

#[allow(warnings, clippy::all)]
#[rustfmt::skip]
mod fx26;

#[cfg(feature = "rust-repl")]
mod rust_repl;

use kayak_rt::{int_to_i64, with_big_stack};

#[cfg(feature = "rust-repl")]
fn run(args: Vec<String>) -> i64 {
    match args.as_slice() {
        [] => with_big_stack(rust_repl::repl),
        [r] if r == "repl" => with_big_stack(rust_repl::repl),
        [k, path] if k == "--keys" => {
            let path = path.clone();
            with_big_stack(move || rust_repl::scripted(&path))
        }
        _ => with_big_stack(|| int_to_i64(fx26::engine_main())),
    }
}

#[cfg(not(feature = "rust-repl"))]
fn run(args: Vec<String>) -> i64 {
    if (args.is_empty() || args == ["repl"])
        && let Ok(home) = std::env::var("HOME")
    {
        kayak_rt::set_args(vec![
            "repl".to_string(),
            "--history".to_string(),
            format!("{home}/.kayak_fx26_history"),
        ]);
    }
    let status = with_big_stack(|| int_to_i64(fx26::main()));
    kayak_rt::finish();
    status
}

/// A batch run's failure said as the engine's (C273): its message alone,
/// `kayak-fx26: …`, where the checker and reader it shares with kfx say
/// `kfx: …`, and Rust's own hook says the thread and where it panicked.
/// The REPL catches its own and says them in its entries.
fn say_failures() {
    std::panic::set_hook(Box::new(|info| {
        let p = info.payload();
        let m = p
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| p.downcast_ref::<&str>().copied())
            .unwrap_or("a failure");
        eprintln!("kayak-fx26: {}", m.strip_prefix("kfx: ").unwrap_or(m));
    }));
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !(args.is_empty() || args == ["repl"]) {
        say_failures();
    }
    let status = run(args);
    kayak_rt::heap_stats_report();
    std::process::exit((status & 0xff) as i32);
}
