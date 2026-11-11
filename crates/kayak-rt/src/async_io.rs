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
//! - **Files and the terminal's modes** (C382): what the system is asked
//!   (a read, a write, a listing, `stty`) is a job the runner runs on a
//!   worker of its own, the task waiting for its answer; the answer, a
//!   KFX value, is made back on the program's thread, whose heap it is in.
//! - **Programs run beside this one** (C382): their readers' news wakes
//!   the runner's reactor, so that a task waiting for one (`pselect`'s
//!   arms, `process-exit`) looks again when there is news, not by the
//!   millisecond; a write to one's input, which waits for it to read, is
//!   a worker's.

use crate::world::{
    BytesAt, IoError, Sum_4byte_3eof_6failed, Sum_4done_6failed, Sum_4line_3eof_6failed,
    Sum_5count_3eof_6failed, input,
};
use crate::{Int, Str, heap_alloc};
use kayak_runner::live::Live;
use kayak_runner::{Answer, Fd, Op, Runner, STDIN};
use kayak_sched::Inbox;
use std::cell::{Cell, RefCell};
use std::sync::{Arc, Mutex};

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

/// A runner's waker, kept as what calls it.
type Wake = Box<dyn Fn() + Send + Sync>;

/// What wakes the runner's reactor, from any thread, while a run is.
static WAKE: Mutex<Option<Wake>> = Mutex::new(None);

/// News from outside the runner's sight (a program run beside this one):
/// its reactor woken, if a run is.
pub(crate) fn wake() {
    if let Some(w) = WAKE.lock().unwrap_or_else(|p| p.into_inner()).as_ref() {
        w();
    }
}

/// The run's runner, `live`, answering through `inbox`.
pub(crate) fn install(live: Live, inbox: Inbox) {
    let waker = live.waker();
    *WAKE.lock().unwrap_or_else(|p| p.into_inner()) = Some(Box::new(move || {
        let _ = waker.wake();
    }));
    LIVE.with(|l| *l.borrow_mut() = Some((live, inbox)));
    PUMPING.with(|p| p.set(false));
    WANTED.with(|w| w.set(false));
}

/// The run's end: what is written all written, and the runner gone.
pub(crate) fn uninstall() {
    drain();
    *WAKE.lock().unwrap_or_else(|p| p.into_inner()) = None;
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

/// `f()`, run by a worker of the runner's (a job), the task waiting for
/// it; here, where nothing may wait or no run is.
async fn on_worker<R: Send + 'static>(
    name: &'static str,
    f: impl FnOnce() -> R + Send + 'static,
) -> R {
    let Some(cx) = crate::suspend::task_cx().filter(|_| installed()) else {
        return f();
    };
    let out: Arc<Mutex<Option<R>>> = Arc::default();
    let into = out.clone();
    let (job, _) = kayak_sched::Job::new(name, move || {
        *into.lock().unwrap_or_else(|p| p.into_inner()) = Some(f());
    });
    let token = submit(Op::Job(job));
    if let Answer::Err(m) = Answer::decode(&cx.completion(token).await) {
        crate::fail(&m)
    }
    let r = out.lock().unwrap_or_else(|p| p.into_inner()).take();
    r.unwrap_or_else(|| crate::fail("a worker's job gave nothing"))
}

/// `(read-file path)`.
pub async fn read_file(
    path: Str,
) -> &'static crate::world::Sum_4data_6failed<crate::world::Bytes, IoError> {
    let path = path.as_str().to_string();
    crate::world::read_file_answer(
        on_worker("read-file", move || crate::world::read_regular_file(&path)).await,
    )
}

/// `(write-file path data)`: the bytes taken now, written by a worker.
pub async fn write_file(path: Str, data: BytesAt<'_>) -> &'static Sum_4done_6failed<(), IoError> {
    let (path, bytes) = (path.as_str().to_string(), crate::world::to_vec(data));
    crate::world::written(
        on_worker("write-file", move || {
            crate::world::write_file_bytes(&path, &bytes)
        })
        .await,
    )
}

/// `(list-dir path)`.
pub async fn list_dir(
    path: Str,
) -> &'static crate::world::Sum_5names_6failed<crate::List<Str>, IoError> {
    let path = path.as_str().to_string();
    crate::world::list_dir_answer(
        on_worker("list-dir", move || crate::world::list_dir_names(&path)).await,
    )
}

/// `(file-info path)`.
pub async fn file_info(
    path: Str,
) -> &'static crate::world::Sum_4info_6failed<crate::world::FileInfo, IoError> {
    let path = path.as_str().to_string();
    crate::world::file_info_answer(
        on_worker("file-info", move || std::fs::symlink_metadata(path)).await,
    )
}

/// `(read-symlink path)`.
pub async fn read_symlink(path: Str) -> &'static crate::world::Sum_6target_6failed<Str, IoError> {
    let path = path.as_str().to_string();
    crate::world::read_symlink_answer(
        on_worker("read-symlink", move || std::fs::read_link(path)).await,
    )
}

/// `(terminal-raw)`.
pub async fn terminal_raw()
-> &'static crate::world::Sum_4done_14not_a_terminal_6failed<(), (), IoError> {
    crate::world::moded(on_worker("terminal-raw", crate::world::terminal_raw_stty).await)
}

/// `(terminal-restore)`.
pub async fn terminal_restore()
-> &'static crate::world::Sum_4done_14not_a_terminal_6failed<(), (), IoError> {
    crate::world::moded(on_worker("terminal-restore", crate::world::terminal_restore_stty).await)
}

/// `(terminal-size)`.
pub async fn terminal_size() -> &'static crate::world::Sum_4size_14not_a_terminal_6failed<
    crate::world::Product_4rows_7columns<Int, Int>,
    (),
    IoError,
> {
    crate::world::terminal_size_answer(
        on_worker("terminal-size", || crate::world::stty(&["size"])).await,
    )
}

/// `(process-write p text)`: written by a worker, which waits for the
/// process to read it.
pub async fn process_write(p: Int, text: Str) -> &'static Sum_4done_6failed<(), IoError> {
    let (input, bytes) = (
        crate::world::process_input(p),
        text.as_str().as_bytes().to_vec(),
    );
    crate::world::written(
        on_worker("process-write", move || {
            crate::world::process_write_bytes(input, &bytes)
        })
        .await,
    )
}

/// `(process-exit p)`: the task waiting until process `p` has ended and
/// its output is all read.
pub async fn process_exit(
    p: Int,
) -> &'static crate::world::Sum_6exited_9signalled_7unknown<Int, Int, ()> {
    crate::async_process::wait_for_outside(move || crate::world::process_exited(p)).await
}
