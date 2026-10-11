//! `kfx FILE`: KFX-0 to Rust, on standard output.
//!
//! `kfx.rs` is generated from `kfx/kfx.kfx` (`PLAN.md` §1); this file is the
//! only hand-written part of the compiler.

#[allow(warnings, clippy::all)]
#[rustfmt::skip]
mod kfx;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, path] = &args[..] else {
        eprintln!("usage: kfx FILE.kfx");
        std::process::exit(2);
    };
    let src = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("kfx: cannot read {path}: {e}");
        std::process::exit(1);
    });
    let src = kayak_rt::leak_string(src);
    print!("{}", kayak_rt::with_big_stack(move || kfx::compile(src)));
}
