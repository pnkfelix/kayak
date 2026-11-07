//! I/O through kayak-runner in the emitter's async mode (C381): the run's
//! I/O is the runner's (`Live`, Coopt's, C368), its answers reaching
//! tasks through the scheduler's inbox, so that a task waiting for input
//! or for its output to be taken waits as a task, and the rest run.
//!
//! - **Writes** to the standard output and error go to the runner at
//!   once, in the order made, the reactor turned so that what can be
//!   written is; a task awaits its answer, as a pipe no one reads holds
//!   up only its writer. A write where nothing may wait (a `block`) is
//!   not awaited. All of them are written before the run ends, or exits.
//! - **The standard input** is read by a task of its own, through the
//!   runner, into the buffer the synchronous mode's reader thread fills
//!   (`world`), when a read wants more; so a line, a byte and `pselect`'s
//!   key arm take from it as they do there, and answer alike.
//! - **`sleep-until`** waits as a task, for its deadline.
//!
//! Files, programs run beside this one and the terminal's modes are as
//! the synchronous mode's, for now (`TODO.md`).

use crate::world::{
    BytesAt, IoError, Sum_4byte_3eof_6failed, Sum_4done_6failed, Sum_4line_3eof_6failed,
    Sum_5count_3eof_6failed, input,
};
use crate::{Int, Str, heap_alloc};
use kayak_runner::live::Live;
use kayak_runner::{Answer, Fd, Op, Runner, STDIN};
use kayak_sched::Inbox;
use std::cell::{Cell, RefCell};

thread_local! {
    /// The run's runner and its scheduler's inbox, while a run is.
    static LIVE: RefCell<Option<(Live, Inbox)>> = const { RefCell::new(None) };
    /// Tokens for what is submitted.
    static NEXT: Cell<u64> = const { Cell::new(0) };
    /// Whether a read wants more of the standard input.
    static WANTED: Cell<bool> = const { Cell::new(false) };
    /// Whether the standard input's task is started.
    static PUMPING: Cell<bool> = const { Cell::new(false) };
}

/// Whether a run's I/O is a runner's.
pub(crate) fn installed() -> bool {
    LIVE.with(|l| l.borrow().is_some())
}

/// The run's runner, `live`, answering through `inbox`.
pub(crate) fn install(live: Live, inbox: Inbox) {
    LIVE.with(|l| *l.borrow_mut() = Some((live, inbox)));
    PUMPING.with(|p| p.set(false));
    WANTED.with(|w| w.set(false));
}

/// The run's end: what is written all written, and the runner gone.
pub(crate) fn uninstall() {
    drain();
    if let Some((mut live, _)) = LIVE.with(|l| l.borrow_mut().take()) {
        live.finish(&kayak_runner::Outcome::Returned);
    }
}

/// Every write submitted, written.
pub(crate) fn drain() {
    LIVE.with(|l| {
        if let Some((live, _)) = l.borrow_mut().as_mut() {
            live.drain_writes();
        }
    })
}

/// `op`, submitted, and the reactor turned once: its token.
fn submit(op: Op) -> u64 {
    let token = NEXT.with(|n| {
        n.set(n.get() + 1);
        n.get()
    });
    LIVE.with(|l| {
        let mut l = l.borrow_mut();
        let Some((live, inbox)) = l.as_mut() else {
            crate::fail("I/O through a runner, with none")
        };
        live.submit(token, op);
        live.poll(0, 0, None, inbox);
    });
    token
}

/// Before a step: what has come, posted.
pub(crate) fn poll() {
    LIVE.with(|l| {
        if let Some((live, inbox)) = l.borrow_mut().as_mut() {
            live.poll(0, 0, None, inbox);
        }
    })
}

/// No task can run: the runner waits until something comes, or the clock
/// (`world::clock`'s) reaches `deadline`. Whether anything may come.
pub(crate) fn idle(deadline: Option<i64>) -> bool {
    LIVE.with(|l| {
        let mut l = l.borrow_mut();
        let Some((live, inbox)) = l.as_mut() else {
            return false;
        };
        let at =
            deadline.map(|d| live.now() + d.saturating_sub(crate::world::clock()).max(0) as u64);
        !matches!(live.idle(0, 0, at, inbox), kayak_runner::runner::Wake::Done)
            || deadline.is_some()
    })
}

fn failed_with(m: String) -> IoError {
    let kind = if m.contains("Broken pipe") {
        std::io::ErrorKind::BrokenPipe
    } else {
        std::io::ErrorKind::Other
    };
    crate::world::io_error(&std::io::Error::new(kind, m))
}

fn written(a: Answer) -> &'static Sum_4done_6failed<(), IoError> {
    match a {
        Answer::Err(m) => heap_alloc(Sum_4done_6failed::Failed(failed_with(m))),
        _ => &Sum_4done_6failed::Done(()),
    }
}

async fn write(fd: u32, s: Str) -> &'static Sum_4done_6failed<(), IoError> {
    let Some(cx) = crate::suspend::task_cx() else {
        return write_now(fd, s);
    };
    let token = submit(Op::Write {
        fd: Fd(fd),
        bytes: s.as_str().as_bytes().to_vec(),
    });
    written(Answer::decode(&cx.completion(token).await))
}

/// `(write-out s)`, the task waiting until it is written.
pub async fn write_out(s: Str) -> &'static Sum_4done_6failed<(), IoError> {
    write(1, s).await
}

/// `(write-err s)`.
pub async fn write_err(s: Str) -> &'static Sum_4done_6failed<(), IoError> {
    write(2, s).await
}

/// A write where nothing may wait: submitted, its answer not awaited.
pub(crate) fn write_now(fd: u32, s: Str) -> &'static Sum_4done_6failed<(), IoError> {
    let token = submit(Op::Write {
        fd: Fd(fd),
        bytes: s.as_str().as_bytes().to_vec(),
    });
    if let Some(cx) = crate::suspend::task_cx() {
        cx.abandon(token);
    }
    &Sum_4done_6failed::Done(())
}

/// A read wants more of the standard input: its task, started at the
/// first, reads it.
pub(crate) fn want_input() {
    WANTED.with(|w| w.set(true));
    if PUMPING.with(|p| p.replace(true)) {
        return;
    }
    let Some(cx) = crate::suspend::task_cx() else {
        PUMPING.with(|p| p.set(false));
        return;
    };
    let spawn = kayak_sched::Spawn::new("stdin").effect(kayak_sched::Effect::pure());
    cx.spawn(spawn, |cx| async move {
        loop {
            crate::async_process::wait_for(|| {
                let full = input().buf.len() >= crate::world::READ_AHEAD;
                (WANTED.with(Cell::get) && !full).then_some(())
            })
            .await;
            WANTED.with(|w| w.set(false));
            let token = submit(Op::Read {
                fd: STDIN,
                max: 4096,
            });
            let answer = Answer::decode(&cx.completion(token).await);
            let ended = {
                let mut i = input();
                match answer {
                    Answer::Bytes(b) if b.is_empty() => i.ended = true,
                    Answer::Bytes(b) => i.buf.extend(b),
                    Answer::Err(m) => {
                        i.error = Some(std::io::Error::other(m));
                        i.ended = true;
                    }
                    _ => {}
                }
                i.ended
            };
            crate::async_process::changed();
            if ended {
                return;
            }
        }
    });
}

/// Waits until a read of the standard input would not wait: a byte there,
/// or its end; or, `line`, a whole line there, or its end, or as much as
/// is read ahead.
async fn input_ready(line: bool) {
    crate::async_process::wait_for_input(move || {
        let i = input();
        let ready = i.ended
            || if line {
                i.buf.contains(&b'\n') || i.buf.len() >= crate::world::READ_AHEAD
            } else {
                !i.buf.is_empty()
            };
        drop(i);
        if !ready {
            want_input();
        }
        ready.then_some(())
    })
    .await
}

/// `(read-line)`, the task waiting for its line.
pub async fn read_line() -> &'static Sum_4line_3eof_6failed<Str, (), IoError> {
    let mut line = Vec::new();
    loop {
        input_ready(true).await;
        let mut i = input();
        match i.buf.iter().position(|&b| b == b'\n') {
            Some(k) => {
                line.extend(i.buf.drain(..=k));
                break;
            }
            None => {
                line.extend(i.buf.drain(..));
                if i.ended {
                    break;
                }
            }
        }
    }
    let r = if line.is_empty() {
        match input().error.take() {
            Some(e) => Err(e),
            None => Ok(None),
        }
    } else {
        if line.last() == Some(&b'\n') {
            line.pop();
            if line.last() == Some(&b'\r') {
                line.pop();
            }
        }
        Ok(Some(line))
    };
    crate::world::line_answer(r)
}

/// `(read-byte)`.
pub async fn read_byte() -> &'static Sum_4byte_3eof_6failed<u8, (), IoError> {
    input_ready(false).await;
    crate::world::read_byte()
}

/// `(read-bytes bv start end)`: none asked for, 0 at once.
pub async fn read_bytes(
    bv: BytesAt<'_>,
    start: Int,
    end: Int,
) -> &'static Sum_5count_3eof_6failed<Int, (), IoError> {
    let (s, e) = crate::world::span(bv, start, end, "read-bytes: a span that does not fit");
    if s == e {
        return crate::world::counted(0);
    }
    input_ready(false).await;
    crate::world::read_bytes(bv, start, end)
}

/// `(sleep-until t)`: the task waiting for the clock to reach `t`.
pub async fn sleep_until(t: i64) {
    crate::async_process::wait_until_clock(t).await
}
