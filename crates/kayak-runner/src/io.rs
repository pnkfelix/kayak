//! Operations as data, and how a task performs one (`PLAN.md` §3.3;
//! `docs/research/runners.md` §5.1).
//!
//! A task performs an [`Op`] through its [`Io`]: the op gets a token from a
//! counter, waits in the hub until the driver hands it to the runner, and
//! the task waits on `cx.completion(token)`. The answer comes back as
//! bytes, through the scheduler's inbox, at a step: the only way anything
//! from outside reaches a task (`PLAN.md` §1, I1).
// coopt-source: crates/coopt-runner/src/io.rs@9283bba

use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use kayak_base::{Fingerprint, Load, StableEncode};
use kayak_sched::{Cx, Job};

/// A file descriptor as the program sees it: an index into the runner's
/// table, never an OS descriptor. 0, 1 and 2 are as usual.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fd(pub u32);

pub const STDIN: Fd = Fd(0);
pub const STDOUT: Fd = Fd(1);

/// An operation. More come with the runners that answer them (M8, M14).
/// Sockets are TCP on 127.0.0.1 alone (`TODO.md` §23).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Op {
    /// Up to `max` bytes, of standard input or a connection; none at the
    /// end.
    Read { fd: Fd, max: u32 },
    /// Answered with how many were written: all of them.
    Write { fd: Fd, bytes: Vec<u8> },
    /// The next signal: SIGINT is `INT`. A handler only ever posts one of
    /// these answers (`PLAN.md` §1, I1).
    NextSignal,
    /// A whole file, by path: what the REPL's history needs, read once at
    /// the start (`TODO.md` §38). Answered with its bytes, or an error.
    ReadFile { path: String },
    /// Bytes added at a file's end, made if it is not there. Answered
    /// with how many were written.
    AppendFile { path: String, bytes: Vec<u8> },
    /// The terminal's width, in columns, for a line editor to wrap its
    /// lines as the terminal does; an error if output is no terminal.
    TerminalWidth,
    /// The date and time: nanoseconds since the Unix epoch, as `Time`.
    /// An input like any, so that a replay sees the time the run saw.
    WallClock,
    /// A listener on 127.0.0.1 at `port`, or at any free port if 0:
    /// answered with its fd.
    Listen { port: u16 },
    /// The port a listener is at.
    Port { fd: Fd },
    /// The next connection to a listener, once one comes: its fd.
    Accept { fd: Fd },
    /// A connection to 127.0.0.1 at `port`: its fd.
    Connect { port: u16 },
    /// Done with a listener or a connection: answered with 0. The other
    /// end of a connection reads its end.
    Close { fd: Fd },
    /// Done writing to a connection (a half-close, `shutdown`): the other
    /// end reads its end once it has read what was written, and may still
    /// write; this end may still read, but not write. Answered with 0.
    Shutdown { fd: Fd },
    /// The operation submitted under `token`, if it still waits (an accept
    /// or a read on a socket, a write not all written), answered with an
    /// error, "cancelled", rather than whatever it waited for. This one is
    /// answered with nothing.
    Cancel { token: u64 },
    /// Pure work, run where the runner likes (a worker thread): answered,
    /// once it has run, with its value's fingerprint (16 bytes), or an
    /// error saying what its panic said. The value is in the job's slot.
    Job(Job),
}

impl Op {
    /// What kind of operation it is, as the log keeps it.
    pub fn kind(&self) -> &'static str {
        match self {
            Op::Read { .. } => "read",
            Op::Write { .. } => "write",
            Op::NextSignal => "signal",
            Op::ReadFile { .. } => "read-file",
            Op::AppendFile { .. } => "append-file",
            Op::TerminalWidth => "terminal-width",
            Op::WallClock => "wall-clock",
            Op::Listen { .. } => "listen",
            Op::Port { .. } => "port",
            Op::Accept { .. } => "accept",
            Op::Connect { .. } => "connect",
            Op::Close { .. } => "close",
            Op::Shutdown { .. } => "shutdown",
            Op::Cancel { .. } => "cancel",
            Op::Job(_) => "job",
        }
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Op::Read { fd, max } => write!(f, "read fd{} max {max}", fd.0),
            Op::Write { fd, bytes } => {
                write!(f, "write fd{} {:?}", fd.0, String::from_utf8_lossy(bytes))
            }
            Op::NextSignal => f.write_str("signal"),
            Op::ReadFile { path } => write!(f, "read-file {path:?}"),
            Op::AppendFile { path, bytes } => {
                write!(f, "append-file {path:?} {} bytes", bytes.len())
            }
            Op::TerminalWidth => f.write_str("terminal-width"),
            Op::WallClock => f.write_str("wall-clock"),
            Op::Listen { port } => write!(f, "listen {port}"),
            Op::Port { fd } => write!(f, "port fd{}", fd.0),
            Op::Accept { fd } => write!(f, "accept fd{}", fd.0),
            Op::Connect { port } => write!(f, "connect {port}"),
            Op::Close { fd } => write!(f, "close fd{}", fd.0),
            Op::Shutdown { fd } => write!(f, "shutdown fd{}", fd.0),
            Op::Cancel { token } => write!(f, "cancel #{token}"),
            Op::Job(j) => write!(f, "job {}", j.name()),
        }
    }
}

/// A co-operation's answer, as bytes in the inbox and the log.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    Bytes(Vec<u8>),
    Count(u32),
    /// An error the program sees, and may handle: Ahman and Bauer's
    /// exceptions, apart from the signals that end a run.
    Err(String),
    /// A signal, by name: `INT`.
    Signal(String),
    /// Nanoseconds since the Unix epoch.
    Time(u64),
}

impl Answer {
    /// A job's answer, as it ran.
    pub fn of_job(done: &kayak_sched::job::Ran) -> Answer {
        match done {
            Ok((Fingerprint(a, b), _)) => {
                let mut v = a.to_le_bytes().to_vec();
                v.extend_from_slice(&b.to_le_bytes());
                Answer::Bytes(v)
            }
            Err(m) => Answer::Err(format!("the job panicked: {m}")),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        match self {
            Answer::Bytes(b) => {
                out.push(0);
                out.extend_from_slice(b);
            }
            Answer::Count(n) => {
                out.push(1);
                out.extend_from_slice(&n.to_le_bytes());
            }
            Answer::Err(m) => {
                out.push(2);
                out.extend_from_slice(m.as_bytes());
            }
            Answer::Signal(m) => {
                out.push(3);
                out.extend_from_slice(m.as_bytes());
            }
            Answer::Time(t) => {
                out.push(4);
                out.extend_from_slice(&t.to_le_bytes());
            }
        }
        out
    }

    pub fn decode(b: &[u8]) -> Answer {
        match b.split_first() {
            Some((0, rest)) => Answer::Bytes(rest.to_vec()),
            Some((1, rest)) if rest.len() == 4 => {
                Answer::Count(u32::from_le_bytes(rest.try_into().expect("four bytes")))
            }
            Some((2, rest)) => Answer::Err(String::from_utf8_lossy(rest).into()),
            Some((3, rest)) => Answer::Signal(String::from_utf8_lossy(rest).into()),
            Some((4, rest)) if rest.len() == 8 => {
                Answer::Time(u64::from_le_bytes(rest.try_into().expect("eight bytes")))
            }
            _ => Answer::Err(format!("a malformed answer: {b:?}")),
        }
    }
}

/// Operations submitted, waiting for the driver to hand them over.
#[derive(Default)]
pub struct Hub {
    next: u64,
    submitted: Vec<(u64, Op)>,
}

/// A handle on the hub, shared by every task that does I/O.
#[derive(Clone, Default)]
pub struct IoHub(Rc<RefCell<Hub>>);

impl IoHub {
    pub fn new() -> IoHub {
        IoHub::default()
    }

    /// Submits `op` and gives its token, without waiting for the answer.
    /// Tokens come from a counter, so they are the same in every run.
    pub fn submit(&self, op: Op) -> u64 {
        let mut h = self.0.borrow_mut();
        h.next += 1;
        let t = h.next;
        h.submitted.push((t, op));
        t
    }

    /// What was submitted since last asked, in order.
    pub fn take(&self) -> Vec<(u64, Op)> {
        std::mem::take(&mut self.0.borrow_mut().submitted)
    }

    /// An `Io` for a task.
    pub fn io(&self, cx: &Cx) -> Io {
        Io {
            cx: cx.clone(),
            hub: self.clone(),
        }
    }
}

/// A task's way to perform operations.
#[derive(Clone)]
pub struct Io {
    cx: Cx,
    hub: IoHub,
}

impl Io {
    /// Performs `op`, and waits for its answer: a suspension point.
    pub async fn perform(&self, op: Op) -> Answer {
        let token = self.hub.submit(op);
        let bytes = self.cx.completion(token).await;
        Answer::decode(&bytes)
    }

    /// `f()`, computed by a job (a suspension point): on a worker thread,
    /// in any runner of this crate. It panics as `f` did.
    pub async fn job<T, F>(&self, name: impl Into<String>, f: F) -> T
    where
        T: StableEncode + Load + Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        let (job, slot) = Job::stored(name, f);
        self.perform(Op::Job(job)).await;
        slot.take().expect("a job's value, once it is answered")
    }
}
