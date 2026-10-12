//! kayak-fx26, FX-26's engine (`docs/research/engine.md`, C58, C64):
//! `fx26/main.kfx`'s `main`, run on a big stack; or, given `repl` or
//! nothing, the REPL (`fx26/repl.kfx`, C65), whose loop is here: the
//! prompts, and a refusal or a failure caught, so that it goes on (the
//! checker refuses by failing, and KFX catches nothing). Generated from
//! `fx26/` (`scripts/engine-programs.py --bless`); this file is the only
//! hand-written part.

#[allow(warnings, clippy::all)]
#[rustfmt::skip]
mod fx26;

use kayak_rt::{int_to_i64, leak_string, with_big_stack};
use std::io::{BufRead, Write};

fn repl() -> i64 {
    // A failure's message alone, as Fixpt's REPL writes it.
    std::panic::set_hook(Box::new(|info| {
        let message = match info.payload().downcast_ref::<String>() {
            Some(s) => s.clone(),
            None => info
                .payload()
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .unwrap_or_default(),
        };
        println!("{message}");
    }));
    println!("{}", fx26::repl_banner().as_str());
    fx26::repl_start_x();
    let stdin = std::io::stdin();
    let mut text = String::new();
    loop {
        print!("{}", if text.is_empty() { "fx26> " } else { "     | " });
        let _ = std::io::stdout().flush();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap_or(0) == 0 {
            println!();
            return 0;
        }
        text.push_str(&line);
        if text.trim().is_empty() {
            text.clear();
            continue;
        }
        if !fx26::repl_complete_p(leak_string(text.clone())) {
            continue;
        }
        let input = leak_string(std::mem::take(&mut text));
        if let Ok(out) =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| fx26::repl_eval_x(input)))
        {
            print!("{}", out.as_str());
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let status = if args.is_empty() || args == ["repl"] {
        with_big_stack(repl)
    } else {
        with_big_stack(|| int_to_i64(fx26::main()))
    };
    std::process::exit((status & 0xff) as i32);
}
