//! kayak-fx26, FX-26's engine (`docs/research/engine.md`, C58, C64):
//! `fx26/main.kfx`'s `main`, run on a big stack; or, given `repl` or
//! nothing, the REPL (`fx26/live-repl.kfx`, `docs/research/repl.md`), a
//! function of what arrives, around which this file only shuttles bytes,
//! time and messages: the terminal put in raw mode (by `stty`, as Fixpt's
//! REPL does), keys read by a thread of their own; the front (keys,
//! editing, checking, the screen) on this thread, the machine (the
//! entries' runs) on another, each with a copy of the session of its own,
//! the two speaking by the session's requests and news (C103), so that no
//! run, however long a step of it, holds up a key; and a refusal or a
//! failure caught, on either, so that it goes on (the checker refuses by
//! failing, and KFX catches nothing). `--keys FILE` runs a session from a
//! script of keys at virtual times, the two sides in step, writing what a
//! terminal of 24 rows and 80 columns would be sent: the REPL's tests.
//! Generated from `fx26/` (`scripts/engine-programs.py --bless`); this
//! file is the only hand-written part.

#[allow(warnings, clippy::all)]
#[rustfmt::skip]
mod fx26;

use fx26::live_repl::session as s;
use fx26::live_repl::{News, Request};
use kayak_rt::{List, car, cdr, int_to_i64, leak_string, null_p, with_big_stack};
use std::cell::RefCell;
use std::io::{IsTerminal, Read, Write};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

// The front and the machine each have a copy of the session (C103), as
// each thread has of every constant of no transmissible type, unless
// kayak-rt's `unsafe-par` keeps one for the program.
const _: () = assert!(
    !kayak_rt::KEPT_FOR_PROGRAM,
    "kayak-fx26 needs kayak-rt without `unsafe-par`: its two threads would share one session"
);

type Req = &'static Request;
type New = &'static News;

thread_local! {
    static FAILURE: RefCell<String> = const { RefCell::new(String::new()) };
}

/// `f` run; a failure's message if it fails.
fn guarded<T>(f: impl FnOnce() -> T) -> Result<T, String> {
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

/// The draft checked as it is typed (and run early, if it may be), a try
/// at a time while the session has another to make, each refusal caught.
/// `KAYAK_SPEC_DEBUG` set, each refusal is written to stderr.
fn speculate(now: i64) {
    loop {
        match guarded(|| s::speculate_x(now)) {
            Ok(true) => {}
            Ok(false) => return,
            Err(message) => {
                if std::env::var_os("KAYAK_SPEC_DEBUG").is_some() {
                    eprintln!("speculation refused: {message}");
                }
                s::speculation_refused_x(leak_string(message), now)
            }
        }
    }
}

/// Slices run, the entries by turns, while `more` says so: the machine's
/// side.
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

/// Each of a KFX list's items, in order.
fn each<T: Copy>(mut xs: List<T>, mut f: impl FnMut(T)) {
    while !null_p(xs) {
        f(car(xs));
        xs = cdr(xs);
    }
}

/// A request from the front, on the machine's side, a refusal caught.
fn request(r: Req, now: i64) {
    if let Err(message) = guarded(|| s::request_x(r, now)) {
        s::machine_refused_x(leak_string(message), now);
    }
}

/// A failure's message alone, kept for the session to show: for every
/// thread, each catching its own.
fn keep_failures() {
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
}

/// A thread with a stack as big as the main one's.
fn thread(f: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(1 << 30)
        .spawn(f)
        .expect("a thread for the machine");
}

/// What comes to the front: keys, the input's end, or news from the machine.
enum Front {
    Keys(Vec<u8>),
    Ended,
    News(New),
}

/// The machine's side of a live or plain session (C103): requests taken,
/// waited for when nothing may run; slices for a sixtieth of a second at
/// most, or until a request comes; the news sent. Its own copy of the
/// session, as every thread has.
fn machine(
    live: bool,
    start: Instant,
    from: mpsc::Receiver<Req>,
    sent: Arc<AtomicUsize>,
    to: mpsc::Sender<Front>,
) {
    let now = move || start.elapsed().as_millis() as i64;
    s::machine_start_x(live);
    if live {
        // A slice at most some milliseconds, so that a request waits no longer.
        s::slices_timed_x(4);
    }
    let mut taken = 0usize;
    loop {
        if s::wait_ms() < 0 {
            match from.recv() {
                Ok(r) => {
                    taken += 1;
                    request(r, now())
                }
                Err(_) => return,
            }
        }
        while let Ok(r) = from.try_recv() {
            taken += 1;
            request(r, now());
        }
        let began = now();
        slices(&mut { now }, &mut |t| {
            t - began < 16 && sent.load(Ordering::SeqCst) <= taken
        });
        let mut gone = false;
        each(s::news_taken_x(now()), |n| {
            gone |= to.send(Front::News(n)).is_err()
        });
        if gone {
            return;
        }
    }
}

/// The front's requests, sent to the machine, counted.
fn send_requests(to: &mpsc::Sender<Req>, sent: &AtomicUsize) {
    each(s::requests_x(), |r| {
        if to.send(r).is_ok() {
            sent.fetch_add(1, Ordering::SeqCst);
        }
    });
}

fn repl() -> i64 {
    keep_failures();
    let live = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();
    let _raw = live.then(Raw::new);
    let (rows, cols) = if live { size() } else { (24, 80) };
    let mut out = std::io::stdout();
    // The history's file, a live session's alone.
    let history = match std::env::var("HOME") {
        Ok(home) if live => format!("{home}/.kayak_fx26_history"),
        _ => String::new(),
    };
    let start = Instant::now();
    let now = move || start.elapsed().as_millis() as i64;
    let (to_front, front) = mpsc::channel::<Front>();
    let (to_machine, machine_gets) = mpsc::channel::<Req>();
    let sent = Arc::new(AtomicUsize::new(0));
    {
        let (sent, to_front) = (sent.clone(), to_front.clone());
        thread(move || machine(live, start, machine_gets, sent, to_front));
    }
    let banner = s::session_start_x(live, rows, cols, leak_string(history));
    let _ = out.write_all(banner.as_str().as_bytes());
    // The prompt, before the first key.
    let _ = out.write_all(s::screen_x(0).as_str().as_bytes());
    let _ = out.flush();
    std::thread::spawn(move || {
        let mut stdin = std::io::stdin();
        let mut buf = [0u8; 1024];
        loop {
            match stdin.read(&mut buf) {
                Ok(0) | Err(_) => {
                    let _ = to_front.send(Front::Ended);
                    return;
                }
                Ok(n) => {
                    if to_front.send(Front::Keys(buf[..n].to_vec())).is_err() {
                        return;
                    }
                }
            }
        }
    });
    loop {
        let wait = s::wait_ms();
        let first = if wait < 0 {
            front.recv().ok()
        } else {
            front.recv_timeout(Duration::from_millis(wait as u64)).ok()
        };
        let t0 = now();
        let mut typed = false;
        for got in first
            .into_iter()
            .chain(std::iter::from_fn(|| front.try_recv().ok()))
        {
            match got {
                Front::Keys(bytes) => {
                    typed = true;
                    bytes.iter().for_each(|&b| key(b, now()));
                }
                Front::Ended => s::leave_x(),
                Front::News(n) => s::news_x(n, now()),
            }
        }
        send_requests(&to_machine, &sent);
        // What was typed, drawn before the draft is checked.
        if typed {
            let _ = out.write_all(s::screen_x(now()).as_str().as_bytes());
            let _ = out.flush();
        }
        let t1 = now();
        speculate(now());
        send_requests(&to_machine, &sent);
        let t2 = now();
        let _ = out.write_all(s::screen_x(now()).as_str().as_bytes());
        if std::env::var_os("KAYAK_TIMING").is_some() && now() - t0 > 5 {
            eprintln!(
                "keys {} speculate {} screen {}",
                t1 - t0,
                t2 - t1,
                now() - t2
            );
        }
        let _ = out.flush();
        if s::over_p() {
            if live {
                let _ = out.write_all(b"\r\n");
            }
            return 0;
        }
    }
}

/// What the front gives the machine in a scripted session: the requests,
/// and the virtual clock; what comes back: the news, the clock after the
/// slices, and whether no task may run.
struct Round {
    requests: Vec<Req>,
    clock: i64,
}

struct Ran {
    news: Vec<New>,
    clock: i64,
    idle: bool,
}

/// The machine's side of a scripted session, in step with the front, so
/// that it replays the same: a round's requests taken, then 16 slices at
/// most, a millisecond each.
fn machine_scripted(from: mpsc::Receiver<Round>, to: mpsc::Sender<Ran>) {
    s::machine_start_x(true);
    while let Ok(round) = from.recv() {
        let mut clock = round.clock;
        round.requests.into_iter().for_each(|r| request(r, clock));
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
        let mut news = Vec::new();
        each(s::news_taken_x(clock), |n| news.push(n));
        let idle = s::next_task() < 0;
        if to.send(Ran { news, clock, idle }).is_err() {
            return;
        }
    }
}

/// `--keys FILE`: a session from a script, each line `MS TEXT`, TEXT's
/// keys arriving at MS on a virtual clock (`\n`, `\r`, `\t`, `\e`, `\xHH`
/// escapes); a slice a millisecond. The input ends after the last line.
fn scripted(path: &str) -> i64 {
    keep_failures();
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
    let (to_machine, machine_gets) = mpsc::channel::<Round>();
    let (to_front, front) = mpsc::channel::<Ran>();
    thread(move || machine_scripted(machine_gets, to_front));
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
        speculate(clock);
        let mut requests = Vec::new();
        each(s::requests_x(), |r| requests.push(r));
        let ran = to_machine
            .send(Round { requests, clock })
            .ok()
            .and_then(|()| front.recv().ok())
            .unwrap_or_else(|| {
                eprintln!("kayak-fx26: the machine's thread ended");
                std::process::exit(70)
            });
        clock = ran.clock;
        ran.news.into_iter().for_each(|n| s::news_x(n, clock));
        let _ = out.write_all(s::screen_x(clock).as_str().as_bytes());
        if s::over_p() {
            let _ = out.write_all(b"\r\n");
            return 0;
        }
        if ran.idle {
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
