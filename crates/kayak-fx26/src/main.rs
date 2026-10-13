//! kayak-fx26, FX-26's engine (`docs/research/engine.md`, C58, C64):
//! `fx26/main.kfx`'s `main`, run on a big stack; or, given `repl` or
//! nothing, the REPL (`fx26/live-repl.kfx`, `docs/research/repl.md`), a
//! function of what arrives, around which this file only shuttles bytes
//! and time: the terminal put in raw mode (by `stty`, as Fixpt's REPL
//! does), keys read by a thread of their own, the session's slices run
//! and its screen written; and a refusal or a failure caught, so that it
//! goes on (the checker refuses by failing, and KFX catches nothing).
//! `--keys FILE` runs a session from a script of keys at virtual times,
//! writing what a terminal of 24 rows and 80 columns would be sent: the
//! REPL's tests. Generated from `fx26/` (`scripts/engine-programs.py
//! --bless`); this file is the only hand-written part.

#[allow(warnings, clippy::all)]
#[rustfmt::skip]
mod fx26;

use fx26::live_repl::session as s;
use kayak_rt::{int_to_i64, leak_string, with_big_stack};
use std::cell::RefCell;
use std::io::{IsTerminal, Read, Write};
use std::process::Command;
use std::sync::mpsc;
use std::time::{Duration, Instant};

thread_local! {
    static FAILURE: RefCell<String> = const { RefCell::new(String::new()) };
}

/// `f` run; a failure's message if it fails.
fn guarded(f: impl FnOnce()) -> Result<(), String> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f))
        .map_err(|_| FAILURE.with(|m| std::mem::take(&mut *m.borrow_mut())))
}

/// The terminal in raw mode while this lives: no echo, keys as they
/// come, ^C a key; its settings put back after.
struct Raw(Option<String>);

impl Raw {
    fn new() -> Raw {
        let saved = stty(&["-g"]).map(|s| s.trim().to_string());
        let _ = stty(&["-icanon", "-echo", "-isig", "min", "1"]);
        Raw(saved)
    }
}

impl Drop for Raw {
    fn drop(&mut self) {
        if let Some(saved) = &self.0 {
            let _ = stty(&[saved.as_str()]);
        }
    }
}

fn stty(args: &[&str]) -> Option<String> {
    let tty = std::fs::File::open("/dev/tty").ok()?;
    let out = Command::new("stty").args(args).stdin(tty).output().ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

/// The terminal's rows and columns, or 24 by 80.
fn size() -> (i64, i64) {
    stty(&["size"])
        .and_then(|s| {
            let mut ws = s.split_whitespace().map(|w| w.parse::<i64>().ok());
            Some((ws.next()??, ws.next()??))
        })
        .filter(|&(r, c)| r > 0 && c > 0)
        .unwrap_or((24, 80))
}

/// One key given to the session, a refusal caught.
fn key(c: u8, now: i64) {
    if let Err(message) = guarded(|| s::key_x(c as i64, now)) {
        s::refused_x(leak_string(message), now);
    }
}

/// Slices run, the entries by turns, while `more` says so.
fn slices(now: &mut dyn FnMut() -> i64, more: &mut dyn FnMut(i64) -> bool) {
    loop {
        let k = s::next_task();
        let t = now();
        if k < 0 || !more(t) {
            return;
        }
        if let Err(message) = guarded(|| s::slice_x(k, t)) {
            s::failed_x(k, leak_string(message), t);
        }
    }
}

fn repl() -> i64 {
    // A failure's message alone, kept for the session to show.
    std::panic::set_hook(Box::new(|info| {
        let message = match info.payload().downcast_ref::<String>() {
            Some(s) => s.clone(),
            None => info
                .payload()
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .unwrap_or_default(),
        };
        FAILURE.with(|m| *m.borrow_mut() = message);
    }));
    let live = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    let _raw = live.then(Raw::new);
    let (rows, cols) = if live { size() } else { (24, 80) };
    let mut out = std::io::stdout();
    // The history's file, a live session's alone.
    let history = match std::env::var("HOME") {
        Ok(home) if live => format!("{home}/.kayak_fx26_history"),
        _ => String::new(),
    };
    let banner = s::session_start_x(live, rows, cols, leak_string(history));
    let _ = out.write_all(banner.as_str().as_bytes());
    let (tx, rx) = mpsc::channel::<Option<Vec<u8>>>();
    std::thread::spawn(move || {
        let mut stdin = std::io::stdin();
        let mut buf = [0u8; 1024];
        loop {
            match stdin.read(&mut buf) {
                Ok(0) | Err(_) => {
                    let _ = tx.send(None);
                    return;
                }
                Ok(n) => {
                    if tx.send(Some(buf[..n].to_vec())).is_err() {
                        return;
                    }
                }
            }
        }
    });
    let start = Instant::now();
    let now = move || start.elapsed().as_millis() as i64;
    let mut ended = false;
    loop {
        let wait = s::wait_ms();
        let got = if ended {
            None
        } else if wait < 0 {
            rx.recv().ok()
        } else if wait == 0 {
            rx.try_recv().ok()
        } else {
            rx.recv_timeout(Duration::from_millis(wait as u64)).ok()
        };
        match got {
            Some(Some(bytes)) => bytes.iter().for_each(|&b| key(b, now())),
            Some(None) => {
                ended = true;
                s::leave_x();
            }
            None => {}
        }
        // Slices for a sixtieth of a second at most, then the screen.
        let began = now();
        slices(&mut { now }, &mut |t| t - began < 16);
        let _ = out.write_all(s::screen_x(now()).as_str().as_bytes());
        let _ = out.flush();
        if s::over_p() {
            if live {
                let _ = out.write_all(b"\r\n");
            }
            return 0;
        }
    }
}

/// `--keys FILE`: a session from a script, each line `MS TEXT`, TEXT's
/// keys arriving at MS on a virtual clock (`\n`, `\r`, `\t`, `\e`, `\xHH`
/// escapes); a slice a millisecond. The input ends after the last line.
fn scripted(path: &str) -> i64 {
    std::panic::set_hook(Box::new(|info| {
        let message = match info.payload().downcast_ref::<String>() {
            Some(s) => s.clone(),
            None => info
                .payload()
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .unwrap_or_default(),
        };
        FAILURE.with(|m| *m.borrow_mut() = message);
    }));
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("kayak-fx26: {path}: {e}");
        std::process::exit(66)
    });
    let mut arrivals: Vec<(i64, Vec<u8>)> = text
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .map(|l| {
            let (ms, keys) = l.split_once(' ').unwrap_or((l, ""));
            (ms.parse().unwrap_or(0), unescape(keys))
        })
        .collect();
    arrivals.reverse();
    let mut out = std::io::stdout();
    let banner = s::session_start_x(true, 24, 80, leak_string(String::new()));
    let _ = out.write_all(banner.as_str().as_bytes());
    let mut clock = 0i64;
    let mut ended = false;
    loop {
        while arrivals.last().is_some_and(|(t, _)| *t <= clock) {
            let (_, keys) = arrivals.pop().unwrap();
            keys.iter().for_each(|&b| key(b, clock));
        }
        if arrivals.is_empty() && !ended {
            ended = true;
            s::leave_x();
        }
        let mut ran = 0;
        {
            let c = &mut clock;
            let mut tick = || {
                *c += 1;
                *c
            };
            slices(&mut tick, &mut |_| {
                ran += 1;
                ran <= 16
            });
        }
        let _ = out.write_all(s::screen_x(clock).as_str().as_bytes());
        if s::over_p() {
            let _ = out.write_all(b"\r\n");
            return 0;
        }
        if s::next_task() < 0 {
            // Nothing to run: the clock to the next key, or a frame on.
            clock = match arrivals.last() {
                Some((t, _)) => (*t).max(clock + 1),
                None => clock + 80,
            };
        }
    }
}

fn unescape(s: &str) -> Vec<u8> {
    let mut out = Vec::new();
    let b = s.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\\' && i + 1 < b.len() {
            match b[i + 1] {
                b'n' => out.push(b'\n'),
                b'r' => out.push(b'\r'),
                b't' => out.push(b'\t'),
                b'e' => out.push(27),
                b'\\' => out.push(b'\\'),
                b'x' if i + 3 < b.len() => {
                    out.push(u8::from_str_radix(&s[i + 2..i + 4], 16).unwrap_or(b'?'));
                    i += 2;
                }
                other => out.push(other),
            }
            i += 2;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let status = match args.as_slice() {
        [] => with_big_stack(repl),
        [r] if r == "repl" => with_big_stack(repl),
        [k, path] if k == "--keys" => {
            let path = path.clone();
            with_big_stack(move || scripted(&path))
        }
        _ => with_big_stack(|| int_to_i64(fx26::main())),
    };
    std::process::exit((status & 0xff) as i32);
}
