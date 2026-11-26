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
//! A task's read or write never blocks the thread (C547): what would is
//! parked, its task waiting on an I-cell the scheduler fills once the
//! descriptor is ready (`sched.rs`), the thread's one wait a `poll` of all
//! that is waited for ([`wait_any`]).
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
pub(crate) fn failed(e: &std::io::Error) -> i64 {
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

/// The next byte, waiting for one: `0..=255`, `-1` at the end, a
/// failure's number. It waits only if nothing is buffered and the
/// descriptor is not [`ready`]: a task's read asks first, and parks if
/// not (C547, `sched.rs`'s `READ_BYTE`).
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

/// Whether a write to `fd` of at most [`CHUNK`] bytes would not block.
pub fn writable(fd: i32) -> bool {
    let mut p = libc::pollfd {
        fd,
        events: libc::POLLOUT,
        revents: 0,
    };
    loop {
        // SAFETY: one `pollfd`, valid for the call.
        let n = unsafe { libc::poll(&mut p, 1, 0) };
        if n >= 0 {
            return n > 0;
        }
        if std::io::Error::last_os_error().kind() != ErrorKind::Interrupted {
            return true;
        }
    }
}

/// The most written at once to a descriptor `poll` says is writable: a
/// pipe's atomic write (`PIPE_BUF`), which then does not block. No
/// descriptor is made non-blocking (`O_NONBLOCK`), which its other users
/// (a shell's terminal) would see.
pub const CHUNK: usize = 4096;

/// One write of `bytes` to `fd`, perhaps short; retried if interrupted.
fn write_once(fd: i32, bytes: &[u8]) -> std::io::Result<usize> {
    loop {
        // SAFETY: `bytes` is readable for its length.
        let n = unsafe { libc::write(fd, bytes.as_ptr().cast(), bytes.len()) };
        if n >= 0 {
            return Ok(n as usize);
        }
        let e = std::io::Error::last_os_error();
        if e.kind() != ErrorKind::Interrupted {
            return Err(e);
        }
    }
}

/// As much of `bytes` written to `fd` as it takes without blocking, a
/// chunk at a time while it is writable: how much.
pub fn write_some(fd: i32, bytes: &[u8]) -> std::io::Result<usize> {
    let mut at = 0;
    while at < bytes.len() && writable(fd) {
        let n = write_once(fd, &bytes[at..(at + CHUNK).min(bytes.len())])?;
        if n == 0 {
            break;
        }
        at += n;
    }
    Ok(at)
}

/// All of `bytes` written to `fd`, waiting as it must: 0, or a failure's
/// number.
pub fn write_all(fd: i32, bytes: &[u8]) -> i64 {
    let mut at = 0;
    while at < bytes.len() {
        match write_once(fd, &bytes[at..]) {
            Ok(n) => at += n,
            Err(e) => return failed(&e),
        }
    }
    0
}

/// Waits until the standard input is ready (if `input`), or one of `outs`
/// writable, or the clock reaches `until`, if given: one `poll` of them
/// all, the thread's one wait when no task can run (C547).
pub fn wait_any(until: Option<i64>, input: bool, outs: &[i32]) {
    if input && ready() {
        return;
    }
    let mut ps: Vec<libc::pollfd> = Vec::new();
    if input {
        ps.push(libc::pollfd {
            fd: 0,
            events: libc::POLLIN,
            revents: 0,
        });
    }
    for &fd in outs {
        ps.push(libc::pollfd {
            fd,
            events: libc::POLLOUT,
            revents: 0,
        });
    }
    let ms = until.map_or(-1, |t| {
        ((t - crate::sched::clock_ns()).max(0) as u64)
            .div_ceil(1_000_000)
            .min(i32::MAX as u64) as i32
    });
    loop {
        // SAFETY: `ps` holds `ps.len()` `pollfd`s, valid for the call.
        let n = unsafe { libc::poll(ps.as_mut_ptr(), ps.len() as libc::nfds_t, ms) };
        if n >= 0 || std::io::Error::last_os_error().kind() != ErrorKind::Interrupted {
            return;
        }
    }
}
