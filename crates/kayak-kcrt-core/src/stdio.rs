//! kc's standard input and output (C546): `read-byte`, `write-out`,
//! `write-err`, and whether a read of the standard input would wait (the
//! scheduler's operations 43-45, Fixpt's `STDIN_READY`, `STDIN_WAIT`,
//! `STDIN_UNWAIT`). As Fixpt's world has them (`world.rs`'s `buffered`,
//! `read_byte`, `stdin_ready`, `wait_stdin`; `tty.rs`'s `ready`): the
//! input read a block at a time into a buffer of this thread's, and ready
//! when the buffer has a byte or the descriptor would not wait.
//!
//! kayak-rt has its own (`world.rs`'s `Input`, a reader thread), which kc's
//! runtime, apart from it, cannot use; so in process a program of kc's
//! reads the standard input as kc's own buffer has it, not kayak-rt's (kc
//! reads none in `kc run`; `kc serve` does, a program's read there takes
//! from its requests). Sharing one is the plan's (`one-runtime.md`).
//!
//! A failure is answered as a fixnum, `-2 - KIND` (`KIND` an `io-error`'s
//! kind's place in [`KINDS`]), its words kept for [`detail`]; kc's l0 makes
//! the `io-error` of them (`kc/l0.kfx`'s `l0-io`).

use std::cell::RefCell;
use std::io::ErrorKind;

/// An `io-error`'s kinds, in the order l0 reads a failure's number by
/// (kayak-rt's `io_error`).
const KINDS: [&str; 8] = [
    "not-found",
    "permission-denied",
    "already-exists",
    "not-a-directory",
    "is-a-directory",
    "broken-pipe",
    "invalid-data",
    "other",
];

#[derive(Default)]
struct Input {
    buf: Vec<u8>,
    at: usize,
    /// The last failure's words.
    detail: String,
}

std::thread_local! {
    static INPUT: RefCell<Input> = RefCell::new(Input::default());
}

/// A failure's number, its words kept.
fn failed(e: &std::io::Error) -> i64 {
    let name = match e.kind() {
        ErrorKind::NotFound => "not-found",
        ErrorKind::PermissionDenied => "permission-denied",
        ErrorKind::AlreadyExists => "already-exists",
        ErrorKind::NotADirectory => "not-a-directory",
        ErrorKind::IsADirectory => "is-a-directory",
        ErrorKind::BrokenPipe => "broken-pipe",
        ErrorKind::InvalidData => "invalid-data",
        _ => "other",
    };
    let k = KINDS
        .iter()
        .position(|&n| n == name)
        .unwrap_or(KINDS.len() - 1) as i64;
    INPUT.with(|i| i.borrow_mut().detail = e.to_string());
    -2 - k
}

/// The last failure's words (`io-error`'s `detail`).
pub fn detail() -> String {
    INPUT.with(|i| i.borrow().detail.clone())
}

/// Whether a read of descriptor 0 would not block (it has a byte, is at
/// its end, or failed), waiting for that at most `ms` milliseconds, or for
/// ever if negative (Fixpt's `tty::ready`).
fn ready_within(ms: i32) -> bool {
    let mut p = libc::pollfd {
        fd: 0,
        events: libc::POLLIN,
        revents: 0,
    };
    loop {
        // SAFETY: one `pollfd`, valid for the call.
        let n = unsafe { libc::poll(&mut p, 1, ms) };
        if n >= 0 {
            return n > 0;
        }
        if std::io::Error::last_os_error().kind() != ErrorKind::Interrupted {
            return true;
        }
    }
}

/// Whether a read of the standard input would not wait: a byte has come,
/// or it is at its end.
pub fn ready() -> bool {
    INPUT.with(|i| {
        let i = i.borrow();
        i.at < i.buf.len()
    }) || ready_within(0)
}

/// Waits until a read of the standard input would not, or until the clock
/// reaches `until` (`clock_ns`), if given; whether it would not.
pub fn wait(until: Option<i64>) -> bool {
    if ready() {
        return true;
    }
    let ms = until.map_or(-1, |t| {
        ((t - crate::sched::clock_ns()).max(0) as u64)
            .div_ceil(1_000_000)
            .min(i32::MAX as u64) as i32
    });
    ready_within(ms)
}

/// The next byte, waiting for one: `0..=255`, `-1` at the end, a
/// failure's number.
pub fn read_byte() -> i64 {
    INPUT.with(|i| {
        let mut i = i.borrow_mut();
        if i.at == i.buf.len() {
            let mut block = [0u8; 4096];
            let n = loop {
                // SAFETY: `block` is writable for its length.
                let n = unsafe { libc::read(0, block.as_mut_ptr().cast(), block.len()) };
                if n >= 0 {
                    break n as usize;
                }
                let e = std::io::Error::last_os_error();
                if e.kind() != ErrorKind::Interrupted {
                    drop(i);
                    return failed(&e);
                }
            };
            i.buf.clear();
            i.at = 0;
            i.buf.extend_from_slice(&block[..n]);
        }
        match i.buf.get(i.at).copied() {
            Some(b) => {
                i.at += 1;
                b as i64
            }
            None => -1,
        }
    })
}

/// `s` written to the standard output (`err` false) or error: 0, or a
/// failure's number. In process (`here`), the output kc's code's values
/// go to (`OUTPUT`), so that they keep their order.
pub fn write(s: &str, err: bool, here: bool) -> i64 {
    use std::io::Write;
    if here && !err {
        crate::output_push(s);
        return 0;
    }
    let r = if err {
        let mut e = std::io::stderr().lock();
        e.write_all(s.as_bytes()).and_then(|()| e.flush())
    } else {
        let mut o = std::io::stdout().lock();
        o.write_all(s.as_bytes()).and_then(|()| o.flush())
    };
    match r {
        Ok(()) => 0,
        Err(e) => failed(&e),
    }
}
