//! The live runner (`PLAN.md` M8; `docs/research/live-io.md`, R4 §7):
//! the terminal, pipes and files, as completions at steps.
//!
//! - **Standard input by what it is** (R4 §2): a tty is opened again,
//!   non-blocking, and put in raw mode for keystrokes; a pipe or socket is
//!   read when the reactor says there is something, no more than there
//!   is, so the shared description stays blocking; a regular file, a
//!   device, or anything the reactor refuses is read by a worker thread,
//!   which hands its answers to this runner and wakes it.
//! - **Signals** are events with no handler, each the answer to a
//!   `NextSignal`. A second ^C while this runner has not seen the first
//!   (a step that never ends) ends the process, its terminal restored
//!   ([`kayak_sys::watch_int`]).
//! - **The clock** is wall time, nanoseconds since the start, moved by
//!   this runner (`ClockMode::Runner`) only when a timer is due, so that
//!   busy periods add nothing to the log.
//! - **Writes** to standard output and error (R4 §6) never block the
//!   runner: each fd's are queued, in order, and each is answered once
//!   all of it is written. A terminal is opened again, non-blocking, and
//!   written until it would block; an inherited pipe or socket stays
//!   blocking, and is written only as much as the reactor says it has
//!   room for; a regular file or a device is written at once. So output
//!   no one reads stalls only what waits for it: keys, signals and other
//!   tasks go on.
//! - **The log**, if one is kept ([`Live::log_to`]), goes to its file as
//!   the run goes: what was recorded is written out whenever the run is
//!   idle, before this runner waits, and at the end (R4 §7.4). A crash
//!   loses only what a run did since it last waited.
//! - **Sockets** (M14), TCP on 127.0.0.1, are non-blocking: each
//!   operation is tried when submitted and, if it would block, again when
//!   the reactor says its fd is ready, in the order they came. A write is
//!   answered once all of it is written. A connect blocks, as one to
//!   localhost is answered at once.
//!
//! Nothing here is seen by a task except through the inbox, and the
//! kernel is never given a pointer into the heap: reads land in this
//! runner's buffers and are copied in as an arrival, at a step.
// coopt-source: crates/coopt-runner/src/live.rs@9283bba

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs::File;
use std::io::{self, Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::os::fd::{AsFd, BorrowedFd, OwnedFd};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

use kayak_sched::Inbox;
use kayak_sys::{Event, Interest, Kind, Poller, RawMode, Sig, Waker, kind, reopen_tty};

use crate::io::{Answer, Fd, Op, STDIN};
use crate::pool::Pool;
use crate::runner::{Kill, Outcome, Runner, Wake};
use crate::values::Values;

const STDIN_KEY: u64 = 0;
/// The program's first socket fd: 0, 1 and 2 are the terminal's.
const FIRST_FD: u32 = 3;
/// What a pipe takes after a readiness that does not say how much
/// (epoll's): `PIPE_BUF`, which a pipe with any room takes whole.
const PIPE_BUF: usize = 4096;

/// How an output fd is written.
enum How {
    /// A terminal opened again, non-blocking: written until it would block.
    Tty(OwnedFd),
    /// An inherited pipe or socket, blocking: written no more than the
    /// reactor said there is room for.
    Bounded,
    /// A regular file, a device, or what the reactor cannot wait on:
    /// written at once, blocking.
    Direct,
}

/// Standard output or error: what is written to it, in order.
struct Out {
    fd: Fd,
    how: How,
    /// Writes not yet all written: token, bytes, how many are written.
    queue: VecDeque<(u64, Vec<u8>, usize)>,
    armed: bool,
}

impl Out {
    fn new(fd: Fd) -> Out {
        let how = {
            let raw = if fd.0 == 2 {
                io::stderr().as_fd().try_clone_to_owned()
            } else {
                io::stdout().as_fd().try_clone_to_owned()
            };
            match raw.as_ref().map(|f| (f, kind(f.as_fd()))) {
                Ok((f, Ok(Kind::Tty))) => reopen_tty(f.as_fd()).map_or(How::Direct, How::Tty),
                Ok((_, Ok(Kind::Pipe | Kind::Socket))) => How::Bounded,
                _ => How::Direct,
            }
        };
        Out {
            fd,
            how,
            queue: VecDeque::new(),
            armed: false,
        }
    }
}

enum Sock {
    Listener(TcpListener),
    Stream(TcpStream),
}

impl Sock {
    fn fd(&self) -> BorrowedFd<'_> {
        match self {
            Sock::Listener(l) => l.as_fd(),
            Sock::Stream(s) => s.as_fd(),
        }
    }
}

/// A socket operation not yet answered.
enum Pending {
    Accept(Fd),
    Read(Fd, u32),
    /// The bytes, and how many are written.
    Write(Fd, Vec<u8>, usize),
}

impl Pending {
    fn fd(&self) -> Fd {
        match self {
            Pending::Accept(fd) | Pending::Read(fd, _) | Pending::Write(fd, _, _) => *fd,
        }
    }
}

/// The reactor's key for a socket and what it waits for.
fn key(fd: Fd, interest: Interest) -> u64 {
    1 + 2 * u64::from(fd.0) + u64::from(interest == Interest::Write)
}

enum Input {
    /// The reactor says when to read: a tty opened again non-blocking, or
    /// the inherited pipe.
    Reactor { tty: Option<OwnedFd> },
    /// A thread reads, blocking; its answers come back through `done`.
    Worker(mpsc::Sender<(u64, u32)>),
}

type Done = Arc<Mutex<VecDeque<(u64, Answer)>>>;

/// A log as a recorder writes it, and the file it goes to, if any.
type LogTo = (Rc<RefCell<Vec<u8>>>, Option<File>);

pub struct Live {
    poller: Poller,
    start: Instant,
    input: Input,
    /// Reads waiting for input.
    reads: VecDeque<(u64, u32)>,
    armed: bool,
    eof: bool,
    ready: Vec<(u64, Answer)>,
    done: Done,
    /// Reads handed to the worker, not yet answered.
    with_worker: usize,
    sig_waiters: VecDeque<u64>,
    raised: VecDeque<Sig>,
    /// `TERM` or `HUP`, if one came: the run ends at the next step.
    ending: Option<Sig>,
    raw: Option<RawMode>,
    /// The program's sockets, by fd.
    socks: BTreeMap<u32, Sock>,
    next_fd: u32,
    /// Socket operations not yet answered, in the order they came.
    pending: VecDeque<(u64, Pending)>,
    /// Socket fds and interests the reactor is armed for.
    armed_socks: BTreeSet<u64>,
    /// The log as recorded, and the file it goes to, if it is kept.
    log: Option<LogTo>,
    /// Standard output and error.
    outs: [Out; 2],
    /// How many SIGINTs this runner has seen, for the watchdog.
    ints: Arc<AtomicU64>,
    /// The jobs' threads, started with the first job; each job done wakes
    /// the reactor.
    pool: Option<Pool>,
    /// Where job values are kept, if they are (`Live::record_values`).
    values: Option<Rc<RefCell<Values>>>,
}

impl Live {
    /// The live runner on this process's standard input and output.
    pub fn new() -> io::Result<Live> {
        Live::open(true)
    }

    /// The live runner for a program of its own (kayak's, C381): a
    /// terminal left as it is, its input read a line at a time, as the
    /// program has not asked for keys; and no signal watched, so that a
    /// ^C ends the program at once, as it ends one that does no I/O
    /// through a runner.
    pub fn cooked() -> io::Result<Live> {
        Live::open(false)
    }

    /// `keys`: the REPL's, a terminal put in raw mode, and signals watched.
    fn open(keys: bool) -> io::Result<Live> {
        let mut poller = Poller::new()?;
        // Before any thread exists (R4 §3).
        if keys {
            poller.watch(&[Sig::Int, Sig::Winch, Sig::Term, Sig::Hup])?;
        }
        let stdin = io::stdin();
        let done: Done = Arc::default();
        let (input, raw) = match kind(stdin.as_fd())? {
            Kind::Tty => {
                let tty = reopen_tty(stdin.as_fd())?;
                let raw = if keys {
                    Some(RawMode::enter(stdin.as_fd())?)
                } else {
                    None
                };
                (Input::Reactor { tty: Some(tty) }, raw)
            }
            Kind::Pipe | Kind::Socket => (Input::Reactor { tty: None }, None),
            Kind::Regular | Kind::Other => {
                (Input::Worker(worker(poller.waker(), done.clone())), None)
            }
        };
        let ints = Arc::new(AtomicU64::new(0));
        if keys {
            let restorer = raw.as_ref().map(RawMode::restorer);
            kayak_sys::watch_int(&mut poller, ints.clone(), move || {
                if let Some(r) = restorer {
                    r.restore();
                }
                let msg = "\r\n; ^C again, and the first not yet seen: the session ends\r\n";
                let _ = kayak_sys::write(io::stderr().as_fd(), msg.as_bytes());
                // 128 + SIGINT, as a shell reports a process SIGINT ended.
                kayak_sys::exit_now(130);
            })?;
        }
        Ok(Live {
            poller,
            start: Instant::now(),
            input,
            reads: VecDeque::new(),
            armed: false,
            eof: false,
            ready: Vec::new(),
            done,
            with_worker: 0,
            sig_waiters: VecDeque::new(),
            raised: VecDeque::new(),
            ending: None,
            raw,
            socks: BTreeMap::new(),
            next_fd: FIRST_FD,
            pending: VecDeque::new(),
            armed_socks: BTreeSet::new(),
            log: None,
            outs: [Out::new(Fd(1)), Out::new(Fd(2))],
            ints,
            pool: None,
            values: None,
        })
    }

    /// A live runner with no standard input (its reads are at the end) and
    /// no signals: for sockets and files alone, in a test.
    pub fn without_input() -> io::Result<Live> {
        Ok(Live {
            poller: Poller::new()?,
            start: Instant::now(),
            input: Input::Reactor { tty: None },
            reads: VecDeque::new(),
            armed: false,
            eof: true,
            ready: Vec::new(),
            done: Arc::default(),
            with_worker: 0,
            sig_waiters: VecDeque::new(),
            raised: VecDeque::new(),
            ending: None,
            raw: None,
            socks: BTreeMap::new(),
            next_fd: FIRST_FD,
            pending: VecDeque::new(),
            armed_socks: BTreeSet::new(),
            log: None,
            outs: [Out::new(Fd(1)), Out::new(Fd(2))],
            ints: Arc::default(),
            pool: None,
            values: None,
        })
    }

    /// What a recorder writes to `buf` is moved to `file` as the run goes:
    /// whenever it is idle, and when it ends ([`Live::flush_log`]). With no
    /// file it is dropped, so that a session not recorded does not keep
    /// its whole log in memory.
    pub fn log_to(&mut self, buf: Rc<RefCell<Vec<u8>>>, file: Option<File>) {
        self.log = Some((buf, file));
    }

    /// Each job's value kept in `values`, as it comes back.
    pub fn record_values(&mut self, values: Rc<RefCell<Values>>) {
        self.values = Some(values);
    }

    /// Writes out what was recorded since the last time. An error stops
    /// the logging, said once on stderr: the session goes on.
    pub fn flush_log(&mut self) {
        let Some((buf, file)) = &mut self.log else {
            return;
        };
        let bytes = std::mem::take(&mut *buf.borrow_mut());
        if let Some(f) = file
            && !bytes.is_empty()
            && let Err(e) = f.write_all(&bytes)
        {
            // Standard error may be gone too (a terminal hung up).
            let _ = writeln!(io::stderr(), "; the log: {e}; no more is written");
            *file = None;
        }
    }

    /// Writes what is queued for standard output or error (`i` 0 or 1), as
    /// far as it goes without blocking; `room` is what the reactor said a
    /// pipe has room for (0 if it said nothing). Arms the reactor for the
    /// rest.
    fn serve_out(&mut self, i: usize, mut room: usize) {
        let out = &mut self.outs[i];
        while let Some((token, bytes, done)) = out.queue.front_mut() {
            let rest = &bytes[*done..];
            let wrote = if rest.is_empty() {
                Ok(0)
            } else {
                match &out.how {
                    How::Tty(f) => kayak_sys::write(f.as_fd(), rest),
                    How::Bounded if room == 0 => break,
                    How::Bounded => {
                        let n = rest.len().min(room);
                        let w = std_fd(out.fd, |f| kayak_sys::write(f, &rest[..n]));
                        if let Ok(k) = w {
                            room -= k;
                        }
                        w
                    }
                    How::Direct => std_fd(out.fd, |f| write_all(f, rest)).map(|()| rest.len()),
                }
            };
            match wrote {
                Ok(n) => {
                    *done += n;
                    if *done == bytes.len() {
                        let n = bytes.len() as u32;
                        let token = *token;
                        out.queue.pop_front();
                        self.ready.push((token, Answer::Count(n)));
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => {
                    // A reader gone (EPIPE), say: this write and every one
                    // after it on this fd fail alike.
                    let msg = format!("fd {}: {e}", out.fd.0);
                    for (t, _, _) in out.queue.drain(..) {
                        self.ready.push((t, Answer::Err(msg.clone())));
                    }
                }
            }
        }
        if !out.queue.is_empty() && !out.armed {
            let k = key(out.fd, Interest::Write);
            let armed = match &out.how {
                How::Tty(f) => self.poller.arm(f.as_fd(), Interest::Write, k),
                _ => std_fd(out.fd, |f| self.poller.arm(f, Interest::Write, k)),
            };
            match armed {
                Ok(()) => out.armed = true,
                Err(e) => {
                    let msg = format!("fd {}: {e}", out.fd.0);
                    for (t, _, _) in out.queue.drain(..) {
                        self.ready.push((t, Answer::Err(msg.clone())));
                    }
                }
            }
        }
    }

    /// A socket, given the program's fd for it.
    fn add(&mut self, s: Sock) -> Answer {
        let fd = self.next_fd;
        self.next_fd += 1;
        self.socks.insert(fd, s);
        Answer::Count(fd)
    }

    /// Tries every socket operation waiting, in order; answers those
    /// done, and arms the reactor for those that would block.
    fn serve_socks(&mut self) {
        let mut still = VecDeque::new();
        for (token, p) in std::mem::take(&mut self.pending) {
            match self.try_sock(p) {
                Ok(a) => self.ready.push((token, a)),
                Err((p, interest)) => {
                    let fd = p.fd();
                    let k = key(fd, interest);
                    if !self.armed_socks.contains(&k) {
                        let armed = self.socks.get(&fd.0).map(|s| {
                            let raw = s.fd();
                            self.poller.arm(raw, interest, k)
                        });
                        match armed {
                            Some(Ok(())) => {
                                self.armed_socks.insert(k);
                            }
                            Some(Err(e)) => {
                                self.ready.push((token, Answer::Err(e.to_string())));
                                continue;
                            }
                            None => {}
                        }
                    }
                    still.push_back((token, p));
                }
            }
        }
        self.pending = still;
    }

    /// An operation done, or given back with what it waits for.
    fn try_sock(&mut self, p: Pending) -> Result<Answer, (Pending, Interest)> {
        let fd = p.fd();
        let Some(sock) = self.socks.get(&fd.0) else {
            return Ok(Answer::Err(format!("fd {}: closed", fd.0)));
        };
        let err = |e: io::Error| Answer::Err(format!("fd {}: {e}", fd.0));
        let blocked = |e: &io::Error| e.kind() == io::ErrorKind::WouldBlock;
        match (p, sock) {
            (Pending::Accept(_), Sock::Listener(l)) => match l.accept() {
                Ok((s, _)) => Ok(match s.set_nonblocking(true) {
                    Ok(()) => self.add(Sock::Stream(s)),
                    Err(e) => err(e),
                }),
                Err(e) if blocked(&e) => Err((Pending::Accept(fd), Interest::Read)),
                Err(e) => Ok(err(e)),
            },
            (Pending::Read(_, max), Sock::Stream(s)) => {
                let mut buf = vec![0u8; max.max(1) as usize];
                match (&*s).read(&mut buf) {
                    Ok(n) => {
                        buf.truncate(n);
                        Ok(Answer::Bytes(buf))
                    }
                    Err(e) if blocked(&e) => Err((Pending::Read(fd, max), Interest::Read)),
                    Err(e) => Ok(err(e)),
                }
            }
            (Pending::Write(_, bytes, mut done), Sock::Stream(s)) => loop {
                if done == bytes.len() {
                    break Ok(Answer::Count(done as u32));
                }
                match (&*s).write(&bytes[done..]) {
                    Ok(n) => done += n,
                    Err(e) if blocked(&e) => {
                        break Err((Pending::Write(fd, bytes, done), Interest::Write));
                    }
                    Err(e) => break Ok(err(e)),
                }
            },
            (Pending::Accept(_), _) => Ok(Answer::Err(format!("fd {}: not a listener", fd.0))),
            _ => Ok(Answer::Err(format!("fd {}: not a connection", fd.0))),
        }
    }

    /// A socket operation that is answered at once.
    fn sock_now(&mut self, op: Op) -> Answer {
        let local = Ipv4Addr::LOCALHOST;
        match op {
            Op::Listen { port } => match TcpListener::bind((local, port)) {
                Ok(l) => match l.set_nonblocking(true) {
                    Ok(()) => self.add(Sock::Listener(l)),
                    Err(e) => Answer::Err(e.to_string()),
                },
                Err(e) => Answer::Err(format!("port {port}: {e}")),
            },
            Op::Port { fd } => match self.socks.get(&fd.0) {
                Some(Sock::Listener(l)) => l.local_addr().map_or_else(
                    |e| Answer::Err(e.to_string()),
                    |a| Answer::Count(u32::from(a.port())),
                ),
                Some(_) => Answer::Err(format!("fd {}: not a listener", fd.0)),
                None => Answer::Err(format!("fd {}: not open", fd.0)),
            },
            Op::Connect { port } => match TcpStream::connect((local, port)) {
                Ok(s) => match s.set_nonblocking(true) {
                    Ok(()) => self.add(Sock::Stream(s)),
                    Err(e) => Answer::Err(e.to_string()),
                },
                Err(e) => Answer::Err(format!("port {port}: {e}")),
            },
            Op::Shutdown { fd } => match self.socks.get(&fd.0) {
                Some(Sock::Stream(s)) => s.shutdown(std::net::Shutdown::Write).map_or_else(
                    |e| Answer::Err(format!("fd {}: {e}", fd.0)),
                    |()| Answer::Count(0),
                ),
                Some(_) => Answer::Err(format!("fd {}: not a connection", fd.0)),
                None => Answer::Err(format!("fd {}: not open", fd.0)),
            },
            Op::Close { fd } => match self.socks.remove(&fd.0) {
                Some(_) => {
                    // Its interests went with it.
                    for i in [Interest::Read, Interest::Write] {
                        self.armed_socks.remove(&key(fd, i));
                    }
                    Answer::Count(0)
                }
                None => Answer::Err(format!("fd {}: not open", fd.0)),
            },
            op => Answer::Err(format!("not a socket operation: {op}")),
        }
    }

    fn ticks(&self) -> u64 {
        self.start.elapsed().as_nanos() as u64
    }

    /// Reads what is there for the reads waiting, as far as it goes;
    /// arms the reactor for the rest.
    fn serve_reads(&mut self, avail: Option<usize>) -> io::Result<()> {
        let Input::Reactor { tty } = &self.input else {
            return Ok(());
        };
        while let Some(&(token, max)) = self.reads.front() {
            if self.eof {
                self.reads.pop_front();
                self.ready.push((token, Answer::Bytes(Vec::new())));
                continue;
            }
            let stdin = io::stdin();
            let fd = match tty {
                Some(t) => t.as_fd(),
                None => stdin.as_fd(),
            };
            // A pipe is blocking: read only what is known to be there.
            let n = match (tty, avail) {
                (Some(_), _) => max as usize,
                (None, Some(a)) => a.min(max as usize),
                (None, None) => kayak_sys::available(fd)?.min(max as usize),
            };
            if n == 0 && tty.is_none() && avail.is_none() {
                break;
            }
            let mut buf = vec![0u8; n.max(1)];
            match kayak_sys::read(fd, &mut buf) {
                Ok(got) => {
                    buf.truncate(got);
                    if got == 0 {
                        self.eof = true;
                    }
                    self.reads.pop_front();
                    self.ready.push((token, Answer::Bytes(buf)));
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => {
                    self.reads.pop_front();
                    self.ready.push((token, Answer::Err(e.to_string())));
                }
            }
            // One answer a turn: the next read comes when it is asked for.
            break;
        }
        if !self.reads.is_empty() && !self.eof && !self.armed {
            let stdin = io::stdin();
            let fd = match tty {
                Some(t) => t.as_fd(),
                None => stdin.as_fd(),
            };
            self.poller.arm(fd, Interest::Read, STDIN_KEY)?;
            self.armed = true;
        }
        Ok(())
    }

    /// One look, or wait, at the system. An event that fails does not
    /// lose the ones after it (a hang-up's SIGHUP comes with the tty's
    /// failing read): the first failure is returned once all are taken.
    fn turn(&mut self, timeout: Option<Duration>) -> io::Result<()> {
        let mut events = Vec::new();
        self.poller.wait(timeout, &mut events)?;
        let mut failed = Ok(());
        for e in events {
            match e {
                Event::Io {
                    key: STDIN_KEY,
                    avail,
                    eof,
                    ..
                } => {
                    self.armed = false;
                    // At a pipe's end a read must be tried, to see it:
                    // kqueue says nothing is left, epoll says only a
                    // hang-up. A tty is read whatever the count.
                    let pipe = matches!(self.input, Input::Reactor { tty: None });
                    let avail = match (eof, avail) {
                        (true, Some(0)) => Some(1),
                        (true, None) if pipe => Some(
                            kayak_sys::available(io::stdin().as_fd())
                                .unwrap_or(0)
                                .max(1),
                        ),
                        _ => avail,
                    };
                    if let Err(e) = self.serve_reads(avail) {
                        failed = failed.and(Err(e));
                    }
                }
                Event::Io { key: k, avail, .. }
                    if let Some(i) =
                        (0..2).find(|&i| k == key(self.outs[i].fd, Interest::Write)) =>
                {
                    self.outs[i].armed = false;
                    self.serve_out(i, avail.unwrap_or(PIPE_BUF));
                }
                Event::Io { key, .. } => {
                    self.armed_socks.remove(&key);
                    self.serve_socks();
                }
                Event::Wake => {}
                Event::Signal { sig, .. } if matches!(sig, Sig::Term | Sig::Hup) => {
                    self.ending.get_or_insert(sig);
                }
                Event::Signal { sig, count } => {
                    if sig == Sig::Int {
                        self.ints.fetch_add(u64::from(count), Ordering::SeqCst);
                    }
                    for _ in 0..count {
                        self.raised.push_back(sig);
                    }
                }
            }
        }
        let mut done = self.done.lock().expect("the worker's queue");
        self.with_worker -= done.len();
        self.ready.extend(done.drain(..));
        drop(done);
        if let Some(p) = &self.pool {
            for (t, d) in p.take_done() {
                if let (Some(v), Ok((fp, Some(bytes)))) = (&self.values, &d) {
                    v.borrow_mut().insert(*fp, bytes.clone());
                }
                self.ready.push((t, Answer::of_job(&d)));
            }
        }
        failed
    }

    /// Posts what is ready. Whether anything was.
    fn post(&mut self, inbox: &Inbox) -> bool {
        let mut any = false;
        for (t, a) in self.ready.drain(..) {
            inbox.post(t, a.encode());
            any = true;
        }
        while !self.raised.is_empty() && !self.sig_waiters.is_empty() {
            let sig = self.raised.pop_front().expect("checked");
            let t = self.sig_waiters.pop_front().expect("checked");
            inbox.post(t, Answer::Signal(sig.name().into()).encode());
            any = true;
        }
        any
    }

    fn waiting(&self) -> bool {
        !self.reads.is_empty()
            || self.with_worker > 0
            || self.pool.as_ref().is_some_and(|p| p.open() > 0)
            || !self.ready.is_empty()
            || !self.pending.is_empty()
            || self.outs.iter().any(|o| !o.queue.is_empty())
    }
}

/// The worker for an input the reactor cannot wait on: blocking reads of
/// standard input, one a request.
fn worker(waker: Waker, done: Done) -> mpsc::Sender<(u64, u32)> {
    let (tx, rx) = mpsc::channel::<(u64, u32)>();
    std::thread::spawn(move || {
        let mut stdin = io::stdin().lock();
        for (token, max) in rx {
            let mut buf = vec![0u8; max as usize];
            let a = match stdin.read(&mut buf) {
                Ok(n) => {
                    buf.truncate(n);
                    Answer::Bytes(buf)
                }
                Err(e) => Answer::Err(e.to_string()),
            };
            done.lock()
                .expect("the worker's queue")
                .push_back((token, a));
            let _ = waker.wake();
        }
    });
    tx
}

impl Live {
    /// What wakes this runner's reactor, from any thread (kayak's, C381):
    /// news from outside it, a program run beside the program.
    pub fn waker(&self) -> kayak_sys::Waker {
        self.poller.waker()
    }

    /// Nanoseconds since this runner began: its clock (kayak's, C381).
    pub fn now(&self) -> u64 {
        self.ticks()
    }

    /// Turns the reactor until every write submitted is written (kayak's,
    /// C381): before a run's end, or an `exit`.
    pub fn drain_writes(&mut self) {
        while self.outs.iter().any(|o| !o.queue.is_empty()) {
            if self.turn(Some(Duration::from_millis(10))).is_err() {
                return;
            }
        }
    }

    /// Whether keys come from a terminal (so a REPL may draw on it), not a
    /// pipe or a file.
    pub fn is_terminal(&self) -> bool {
        matches!(self.input, Input::Reactor { tty: Some(_) })
    }
}

impl Runner for Live {
    fn submit(&mut self, token: u64, op: Op) {
        match op {
            Op::NextSignal => self.sig_waiters.push_back(token),
            Op::Read { fd, max } if fd == STDIN => match &self.input {
                Input::Worker(tx) => {
                    self.with_worker += 1;
                    tx.send((token, max)).expect("the worker");
                }
                Input::Reactor { .. } => {
                    self.reads.push_back((token, max));
                    if let Err(e) = self.serve_reads(None) {
                        self.ready.push((token, Answer::Err(e.to_string())));
                    }
                }
            },
            Op::Read { fd, max } if fd.0 >= FIRST_FD => {
                self.pending.push_back((token, Pending::Read(fd, max)));
                self.serve_socks();
            }
            Op::Write { fd, bytes } if fd.0 >= FIRST_FD => {
                self.pending
                    .push_back((token, Pending::Write(fd, bytes, 0)));
                self.serve_socks();
            }
            Op::Accept { fd } => {
                self.pending.push_back((token, Pending::Accept(fd)));
                self.serve_socks();
            }
            op @ (Op::Listen { .. }
            | Op::Port { .. }
            | Op::Connect { .. }
            | Op::Close { .. }
            | Op::Shutdown { .. }) => {
                let closing = match op {
                    Op::Close { fd } => Some(fd),
                    _ => None,
                };
                let a = self.sock_now(op);
                self.ready.push((token, a));
                // What waited on a closed fd is answered: it is closed.
                if closing.is_some() {
                    self.serve_socks();
                }
            }
            Op::Cancel { token: t } => {
                if let Some(i) = self.pending.iter().position(|(p, _)| *p == t) {
                    self.pending.remove(i);
                    self.ready.push((t, Answer::Err("cancelled".into())));
                }
            }
            Op::Write { fd, bytes } if fd.0 == 1 || fd.0 == 2 => {
                let i = fd.0 as usize - 1;
                self.outs[i].queue.push_back((token, bytes, 0));
                self.serve_out(i, 0);
            }
            Op::TerminalWidth => {
                let a = if self.is_terminal() {
                    kayak_sys::terminal_width(io::stdout().as_fd())
                        .map_or_else(|e| Answer::Err(e.to_string()), |w| Answer::Count(w.into()))
                } else {
                    Answer::Err("not a terminal".into())
                };
                self.ready.push((token, a));
            }
            Op::WallClock => {
                let a = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or_else(
                        |_| Answer::Err("the clock is before 1970".into()),
                        |d| Answer::Time(d.as_nanos() as u64),
                    );
                self.ready.push((token, a));
            }
            // Small files, read and written whole, at once (the history's).
            Op::ReadFile { path } => {
                let a = std::fs::read(&path)
                    .map_or_else(|e| Answer::Err(format!("{path}: {e}")), Answer::Bytes);
                self.ready.push((token, a));
            }
            Op::AppendFile { path, bytes } => {
                use std::io::Write;
                let a = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)
                    .and_then(|mut f| f.write_all(&bytes))
                    .map_or_else(
                        |e| Answer::Err(format!("{path}: {e}")),
                        |()| Answer::Count(bytes.len() as u32),
                    );
                self.ready.push((token, a));
            }
            Op::Job(job) => {
                let keep = self.values.is_some();
                let poller = &self.poller;
                self.pool
                    .get_or_insert_with(|| {
                        let waker = poller.waker();
                        Pool::per_core(Some(Box::new(move || {
                            let _ = waker.wake();
                        })))
                    })
                    .submit(token, job, keep);
            }
            op => self
                .ready
                .push((token, Answer::Err(format!("no such file: {op}")))),
        }
    }

    fn poll(&mut self, _next: u64, now: u64, deadline: Option<u64>, inbox: &Inbox) -> Option<u64> {
        let _ = self.turn(Some(Duration::ZERO));
        self.post(inbox);
        let wall = self.ticks();
        deadline.filter(|&d| d <= wall && now < wall).map(|_| wall)
    }

    fn idle(&mut self, _step: u64, now: u64, deadline: Option<u64>, inbox: &Inbox) -> Wake {
        if self.post(inbox) {
            return Wake::Posted;
        }
        if !self.waiting() && deadline.is_none() {
            // Signals alone keep nothing alive.
            return Wake::Done;
        }
        self.flush_log();
        let timeout = deadline.map(|d| Duration::from_nanos(d.saturating_sub(self.ticks())));
        if self.turn(timeout).is_err() {
            return Wake::Done;
        }
        if self.post(inbox) {
            return Wake::Posted;
        }
        let wall = self.ticks();
        match deadline {
            Some(d) if d <= wall => Wake::Clock(wall.max(now)),
            _ => Wake::Posted,
        }
    }

    fn kill(&mut self) -> Option<Kill> {
        self.ending.take().map(|s| Kill::Signal(s.name().into()))
    }

    fn finish(&mut self, _outcome: &Outcome) {
        // The terminal as it was.
        self.raw = None;
    }
}

/// `f` on standard output (fd 1) or error (fd 2).
fn std_fd<T>(fd: Fd, f: impl FnOnce(BorrowedFd<'_>) -> T) -> T {
    if fd.0 == 2 {
        f(io::stderr().as_fd())
    } else {
        f(io::stdout().as_fd())
    }
}

/// All of `bytes`, to a file or a device, which takes them at once.
fn write_all(f: BorrowedFd<'_>, mut bytes: &[u8]) -> io::Result<()> {
    while !bytes.is_empty() {
        match kayak_sys::write(f, bytes) {
            Ok(n) => bytes = &bytes[n..],
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(1));
            }
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
