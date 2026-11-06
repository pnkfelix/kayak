//! The operating system, for the live runner (`PLAN.md` M8;
//! `docs/research/live-io.md`, R4): the second of the two crates where
//! `unsafe` is allowed (`PLAN.md` §1, I2), the first being `coopt-mem`.
//!
//! Small, safe wrappers, and nothing more:
//! - [`Poller`]: one-shot interest in reading or writing an fd, a wait with
//!   a timeout, a [`Waker`] another thread may use, and signals without
//!   handlers. kqueue on macOS; epoll, eventfd and signalfd on Linux.
//! - [`kind`] of an fd, [`reopen_tty`], [`read`], [`write`],
//!   [`available`];
//! - [`RawMode`] for keystroke input, restored when dropped;
//! - [`raise_fd_limit`];
//! - [`watch_int`], a second ^C's way out of a run that never yields, and
//!   [`exit_now`].
//!
//! The Linux paths are compiled for Linux and Android targets, and tested
//! only where a test runs: on macOS here (`TODO.md` §41).
//!
//! Every `unsafe` block says what it relies on. No pointer given to the
//! kernel points into the moving heap: only into buffers of this crate's
//! callers, Rust slices that outlive the call (`PLAN.md` §3.3).
// coopt-source: crates/coopt-sys/src/lib.rs@9283bba
#![allow(unsafe_code)]

use std::io;
use std::os::fd::{AsRawFd, BorrowedFd, FromRawFd, OwnedFd, RawFd};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

#[cfg(any(target_os = "linux", target_os = "android"))]
mod epoll;
#[cfg(target_os = "macos")]
mod kqueue;

#[cfg(any(target_os = "linux", target_os = "android"))]
use epoll as imp;
#[cfg(target_os = "macos")]
use kqueue as imp;

/// What to wait for on an fd.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Interest {
    Read,
    Write,
}

/// A signal the live runner listens for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Sig {
    Int,
    Term,
    Hup,
    Winch,
}

impl Sig {
    fn number(self) -> libc::c_int {
        match self {
            Sig::Int => libc::SIGINT,
            Sig::Term => libc::SIGTERM,
            Sig::Hup => libc::SIGHUP,
            Sig::Winch => libc::SIGWINCH,
        }
    }

    fn from_number(n: libc::c_int) -> Option<Sig> {
        [Sig::Int, Sig::Term, Sig::Hup, Sig::Winch]
            .into_iter()
            .find(|s| s.number() == n)
    }

    /// Its name without `SIG`: `INT`.
    pub fn name(self) -> &'static str {
        match self {
            Sig::Int => "INT",
            Sig::Term => "TERM",
            Sig::Hup => "HUP",
            Sig::Winch => "WINCH",
        }
    }
}

/// What a wait found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    /// The fd armed under `key` is ready; `avail` is how much there is to
    /// read, or room to write, if the system says.
    Io {
        key: u64,
        interest: Interest,
        avail: Option<usize>,
        eof: bool,
    },
    /// A [`Waker`] woke the poller.
    Wake,
    /// A signal watched for came, `count` times since the last wait.
    Signal { sig: Sig, count: u32 },
}

fn check(r: libc::c_int) -> io::Result<libc::c_int> {
    if r < 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(r)
    }
}

/// Waits for fds and signals; woken from other threads by its [`Waker`].
pub struct Poller {
    inner: imp::Poller,
}

/// Wakes a [`Poller`] from any thread.
#[derive(Clone)]
pub struct Waker {
    inner: Arc<imp::WakeFd>,
}

impl Waker {
    pub fn wake(&self) -> io::Result<()> {
        self.inner.wake()
    }
}

impl Poller {
    pub fn new() -> io::Result<Poller> {
        Ok(Poller {
            inner: imp::Poller::new()?,
        })
    }

    /// One-shot interest: the next wait that finds `fd` ready reports it
    /// once, under `key`, and the interest is gone.
    pub fn arm(&mut self, fd: BorrowedFd<'_>, interest: Interest, key: u64) -> io::Result<()> {
        self.inner.arm(fd.as_raw_fd(), interest, key)
    }

    /// Signals become events, with no handler: on macOS ignored and
    /// watched, on Linux blocked and read. Call before any thread starts.
    pub fn watch(&mut self, sigs: &[Sig]) -> io::Result<()> {
        self.inner.watch(sigs)
    }

    /// Waits until something in `out`, or `timeout` (`None`: no limit;
    /// `Some(0)`: a look). An interrupted wait is an empty one.
    pub fn wait(&mut self, timeout: Option<Duration>, out: &mut Vec<Event>) -> io::Result<()> {
        self.inner.wait(timeout, out)
    }

    pub fn waker(&self) -> Waker {
        Waker {
            inner: self.inner.waker(),
        }
    }
}

/// What an fd is, which decides how it is read (R4 §2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Tty,
    Pipe,
    Socket,
    Regular,
    Other,
}

pub fn kind(fd: BorrowedFd<'_>) -> io::Result<Kind> {
    // SAFETY: isatty only reads the descriptor number.
    if unsafe { libc::isatty(fd.as_raw_fd()) } == 1 {
        return Ok(Kind::Tty);
    }
    // SAFETY: fstat writes into the zeroed stat we own.
    let mut st: libc::stat = unsafe { std::mem::zeroed() };
    check(unsafe { libc::fstat(fd.as_raw_fd(), &mut st) })?;
    Ok(match st.st_mode & libc::S_IFMT {
        libc::S_IFIFO => Kind::Pipe,
        libc::S_IFSOCK => Kind::Socket,
        libc::S_IFREG => Kind::Regular,
        _ => Kind::Other,
    })
}

/// The tty behind `fd`, opened again: a description of its own, which may
/// be made non-blocking without changing the shell's (R4 §2.2).
pub fn reopen_tty(fd: BorrowedFd<'_>) -> io::Result<OwnedFd> {
    let mut name = [0 as libc::c_char; 256];
    // SAFETY: ttyname_r writes at most `name.len()` bytes into our buffer.
    let r = unsafe { libc::ttyname_r(fd.as_raw_fd(), name.as_mut_ptr(), name.len()) };
    if r != 0 {
        return Err(io::Error::from_raw_os_error(r));
    }
    // SAFETY: `name` is now a NUL-terminated path; open copies it.
    let new = check(unsafe {
        libc::open(
            name.as_ptr(),
            libc::O_RDWR | libc::O_NONBLOCK | libc::O_NOCTTY | libc::O_CLOEXEC,
        )
    })?;
    // SAFETY: open returned a fresh descriptor that nothing else owns.
    Ok(unsafe { OwnedFd::from_raw_fd(new) })
}

/// Reads into `buf`; `WouldBlock` if nothing is there on a non-blocking
/// fd; 0 at the end.
pub fn read(fd: BorrowedFd<'_>, buf: &mut [u8]) -> io::Result<usize> {
    loop {
        // SAFETY: the kernel writes at most `buf.len()` bytes into `buf`.
        let n = unsafe { libc::read(fd.as_raw_fd(), buf.as_mut_ptr().cast(), buf.len()) };
        if n >= 0 {
            return Ok(n as usize);
        }
        let e = io::Error::last_os_error();
        if e.kind() != io::ErrorKind::Interrupted {
            return Err(e);
        }
    }
}

/// Writes from `buf`, as much as the fd takes.
pub fn write(fd: BorrowedFd<'_>, buf: &[u8]) -> io::Result<usize> {
    loop {
        // SAFETY: the kernel reads at most `buf.len()` bytes from `buf`.
        let n = unsafe { libc::write(fd.as_raw_fd(), buf.as_ptr().cast(), buf.len()) };
        if n >= 0 {
            return Ok(n as usize);
        }
        let e = io::Error::last_os_error();
        if e.kind() != io::ErrorKind::Interrupted {
            return Err(e);
        }
    }
}

/// A terminal's width, in columns (TIOCGWINSZ).
pub fn terminal_width(fd: BorrowedFd<'_>) -> io::Result<u16> {
    // SAFETY: a winsize is plain integers; TIOCGWINSZ fills the one we own.
    let mut w: libc::winsize = unsafe { std::mem::zeroed() };
    // SAFETY: TIOCGWINSZ writes one winsize into `w`.
    check(unsafe { libc::ioctl(fd.as_raw_fd(), libc::TIOCGWINSZ, &mut w) })?;
    Ok(w.ws_col)
}

/// How many bytes can be read now (FIONREAD).
pub fn available(fd: BorrowedFd<'_>) -> io::Result<usize> {
    let mut n: libc::c_int = 0;
    // SAFETY: FIONREAD writes one int into `n`.
    check(unsafe { libc::ioctl(fd.as_raw_fd(), libc::FIONREAD, &mut n) })?;
    Ok(n.max(0) as usize)
}

/// A terminal in raw mode, for keystroke input: no line buffering, no
/// echo; `ISIG` kept, so ^C is SIGINT (R4, decision 2). Restored when
/// dropped.
pub struct RawMode {
    fd: RawFd,
    saved: libc::termios,
}

impl RawMode {
    pub fn enter(fd: BorrowedFd<'_>) -> io::Result<RawMode> {
        // SAFETY: tcgetattr fills the termios we own.
        let mut t: libc::termios = unsafe { std::mem::zeroed() };
        check(unsafe { libc::tcgetattr(fd.as_raw_fd(), &mut t) })?;
        let saved = t;
        t.c_lflag &= !(libc::ICANON | libc::ECHO | libc::IEXTEN);
        t.c_iflag &= !(libc::IXON | libc::ICRNL);
        t.c_cc[libc::VMIN] = 1;
        t.c_cc[libc::VTIME] = 0;
        // SAFETY: tcsetattr reads the termios we own.
        check(unsafe { libc::tcsetattr(fd.as_raw_fd(), libc::TCSANOW, &t) })?;
        restore_on_panic(fd.as_raw_fd(), saved);
        Ok(RawMode {
            fd: fd.as_raw_fd(),
            saved,
        })
    }

    pub fn restore(&self) {
        self.restorer().restore();
    }

    /// What restores the terminal, for another thread to use.
    pub fn restorer(&self) -> Restorer {
        Restorer {
            fd: self.fd,
            saved: self.saved,
        }
    }
}

/// A terminal's settings as they were before raw mode, and the fd to put
/// them back on.
#[derive(Clone, Copy)]
pub struct Restorer {
    fd: RawFd,
    saved: libc::termios,
}

impl Restorer {
    pub fn restore(&self) {
        // SAFETY: tcsetattr reads the termios we own; on an fd closed
        // since, it fails, and there is nothing to undo.
        unsafe {
            libc::tcsetattr(self.fd, libc::TCSANOW, &self.saved);
        }
    }
}

/// Ends the process at once, with `code`: no destructors, no handlers,
/// nothing that could wait on a lock a stuck thread holds.
pub fn exit_now(code: i32) -> ! {
    // SAFETY: _exit takes no pointers and does not return.
    unsafe { libc::_exit(code) }
}

/// A watchdog for ^C (R4, question 3): a thread with a poller of its own,
/// watching SIGINT for `poller`, the caller's. `taken` is how many SIGINTs
/// the caller has seen; once two have come that it has not, the caller is
/// taken to be stuck (a step that never ends), and `force` is called, on
/// the watchdog's thread. Two in a row, quicker than the caller looks,
/// count as well: that is a second ^C too.
///
/// On macOS every kqueue watching a signal is told of it, so the
/// watchdog watches beside `poller`. On Linux a signal is read from one
/// signalfd only, so the watchdog takes SIGINT from `poller` and hands
/// each on, waking it: the caller sees SIGINT as before. Call after
/// `poller` watches SIGINT, and before any other thread starts.
pub fn watch_int(
    poller: &mut Poller,
    taken: Arc<AtomicU64>,
    force: impl FnOnce() + Send + 'static,
) -> io::Result<()> {
    // Watched here before `poller` lets go of it, so that none is lost.
    let mut p = Poller::new()?;
    p.watch(&[Sig::Int])?;
    #[cfg(any(target_os = "linux", target_os = "android"))]
    let forward = Some((poller.inner.hand_over_int()?, poller.waker()));
    #[cfg(target_os = "macos")]
    let forward: Option<(Arc<AtomicU64>, Waker)> = {
        let _ = poller;
        None
    };
    std::thread::Builder::new()
        .name("^C watchdog".into())
        .spawn(move || {
            let mut seen = 0u64;
            let mut out = Vec::new();
            loop {
                out.clear();
                if p.wait(None, &mut out).is_err() {
                    return;
                }
                for e in &out {
                    if let Event::Signal {
                        sig: Sig::Int,
                        count,
                    } = e
                    {
                        seen += u64::from(*count);
                        if let Some((forwarded, waker)) = &forward {
                            forwarded.fetch_add(u64::from(*count), Ordering::SeqCst);
                            let _ = waker.wake();
                        }
                    }
                }
                if seen.saturating_sub(taken.load(Ordering::SeqCst)) >= 2 {
                    force();
                    return;
                }
            }
        })?;
    Ok(())
}

/// A pseudo-terminal: its leader and its follower (for tests of raw mode).
pub fn openpty() -> io::Result<(OwnedFd, OwnedFd)> {
    let (mut leader, mut follower) = (0, 0);
    // SAFETY: openpty fills the two fds; the rest may be null.
    check(unsafe {
        libc::openpty(
            &mut leader,
            &mut follower,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    })?;
    // SAFETY: openpty gave us these fds, owned from here.
    Ok(unsafe { (OwnedFd::from_raw_fd(leader), OwnedFd::from_raw_fd(follower)) })
}

/// Whether the terminal at `fd` is in canonical (cooked) mode.
pub fn canonical(fd: BorrowedFd<'_>) -> io::Result<bool> {
    // SAFETY: tcgetattr fills the termios we own.
    let mut t: libc::termios = unsafe { std::mem::zeroed() };
    check(unsafe { libc::tcgetattr(fd.as_raw_fd(), &mut t) })?;
    Ok(t.c_lflag & libc::ICANON != 0)
}

/// From now on, a panic restores the terminal before it is reported (R4,
/// recommendation 5): a panic is no reason to leave the terminal raw, and
/// its message is read on a terminal that echoes again. The hook stays:
/// once the guard has restored the terminal, restoring it again is
/// harmless.
fn restore_on_panic(fd: RawFd, saved: libc::termios) {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // SAFETY: tcsetattr reads the termios the hook owns; on an fd
        // closed since, it fails, and there is nothing to undo.
        unsafe {
            libc::tcsetattr(fd, libc::TCSANOW, &saved);
        }
        previous(info);
    }));
}

impl Drop for RawMode {
    fn drop(&mut self) {
        self.restore();
    }
}

/// Raises the soft limit on open fds as far as the system allows: the
/// hard limit, capped by `kern.maxfilesperproc` on macOS, falling back to
/// `OPEN_MAX` (R4 §4). Gives the limit now in force.
pub fn raise_fd_limit() -> io::Result<u64> {
    // SAFETY: getrlimit fills the rlimit we own.
    let mut r: libc::rlimit = unsafe { std::mem::zeroed() };
    check(unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut r) })?;
    let want = max_files(r.rlim_max);
    if want <= r.rlim_cur {
        return Ok(r.rlim_cur as u64);
    }
    let mut new = r;
    new.rlim_cur = want;
    // SAFETY: setrlimit reads the rlimit we own.
    if unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &new) } != 0 {
        new.rlim_cur = (OPEN_MAX as libc::rlim_t).max(r.rlim_cur);
        // SAFETY: as above.
        check(unsafe { libc::setrlimit(libc::RLIMIT_NOFILE, &new) })?;
    }
    Ok(new.rlim_cur as u64)
}

/// The most files a process may have: the hard limit, and on macOS no
/// more than `kern.maxfilesperproc`.
#[cfg(target_os = "macos")]
fn max_files(hard: libc::rlim_t) -> libc::rlim_t {
    let mut per: libc::c_int = 0;
    let mut len = std::mem::size_of::<libc::c_int>();
    // SAFETY: sysctlbyname writes one int into `per`, and its length into
    // `len`; the name is a NUL-terminated literal.
    let ok = unsafe {
        libc::sysctlbyname(
            c"kern.maxfilesperproc".as_ptr(),
            (&mut per as *mut libc::c_int).cast(),
            &mut len,
            std::ptr::null_mut(),
            0,
        )
    } == 0;
    if ok && per > 0 {
        hard.min(per as libc::rlim_t)
    } else {
        hard
    }
}

#[cfg(not(target_os = "macos"))]
fn max_files(hard: libc::rlim_t) -> libc::rlim_t {
    hard
}

/// `OPEN_MAX` of `<sys/syslimits.h>` on macOS, which `libc` does not
/// export: the fallback when the system refuses a larger limit.
const OPEN_MAX: u64 = 10240;

/// A pipe, both ends, for tests and for workers' wakes.
pub fn pipe() -> io::Result<(OwnedFd, OwnedFd)> {
    let mut fds = [0 as libc::c_int; 2];
    // SAFETY: pipe writes two descriptors into our array.
    check(unsafe { libc::pipe(fds.as_mut_ptr()) })?;
    // SAFETY: both are fresh descriptors that nothing else owns.
    Ok(unsafe { (OwnedFd::from_raw_fd(fds[0]), OwnedFd::from_raw_fd(fds[1])) })
}
