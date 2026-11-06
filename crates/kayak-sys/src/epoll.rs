//! epoll (Linux, Android): interest by `EPOLLONESHOT`, waking by an
//! eventfd, signals blocked and read from a signalfd (R4 §1, §3). Tested on
//! Linux (`DONE.md` §41).
// coopt-source: crates/coopt-sys/src/epoll.rs@9283bba

use std::io;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use crate::{Event, Interest, Sig, check};

const WAKE_KEY: u64 = u64::MAX;
const SIGNAL_KEY: u64 = u64::MAX - 1;

pub(crate) struct WakeFd {
    efd: OwnedFd,
}

impl WakeFd {
    pub(crate) fn wake(&self) -> io::Result<()> {
        let one: u64 = 1;
        // SAFETY: an eventfd write takes exactly eight bytes, ours.
        let n = unsafe { libc::write(self.efd.as_raw_fd(), (&one as *const u64).cast(), 8) };
        if n == 8 || io::Error::last_os_error().kind() == io::ErrorKind::WouldBlock {
            Ok(())
        } else {
            Err(io::Error::last_os_error())
        }
    }
}

pub(crate) struct Poller {
    ep: OwnedFd,
    wake: Arc<WakeFd>,
    sigfd: Option<OwnedFd>,
    /// The signals `sigfd` reads.
    sigs: Vec<Sig>,
    /// SIGINTs read by the watchdog and not yet reported, if it reads
    /// them (`hand_over_int`); reported when it wakes this poller.
    forwarded: Option<Arc<AtomicU64>>,
    /// Interest each fd has been registered with once, so later arms
    /// modify rather than add.
    known: std::collections::BTreeSet<(RawFd, bool)>,
}

fn ctl(ep: RawFd, op: libc::c_int, fd: RawFd, events: u32, key: u64) -> io::Result<()> {
    let mut ev = libc::epoll_event { events, u64: key };
    // SAFETY: the event is ours, read during the call.
    check(unsafe { libc::epoll_ctl(ep, op, fd, &mut ev) }).map(|_| ())
}

impl Poller {
    pub(crate) fn new() -> io::Result<Poller> {
        // SAFETY: fresh descriptors, owned at once.
        let ep = unsafe { OwnedFd::from_raw_fd(check(libc::epoll_create1(libc::EPOLL_CLOEXEC))?) };
        let efd = unsafe {
            OwnedFd::from_raw_fd(check(libc::eventfd(
                0,
                libc::EFD_NONBLOCK | libc::EFD_CLOEXEC,
            ))?)
        };
        ctl(
            ep.as_raw_fd(),
            libc::EPOLL_CTL_ADD,
            efd.as_raw_fd(),
            libc::EPOLLIN as u32,
            WAKE_KEY,
        )?;
        Ok(Poller {
            ep,
            wake: Arc::new(WakeFd { efd }),
            sigfd: None,
            sigs: Vec::new(),
            forwarded: None,
            known: Default::default(),
        })
    }

    pub(crate) fn arm(&mut self, fd: RawFd, interest: Interest, key: u64) -> io::Result<()> {
        let write = interest == Interest::Write;
        let events =
            (if write { libc::EPOLLOUT } else { libc::EPOLLIN }) as u32 | libc::EPOLLONESHOT as u32;
        // epoll keeps one registration per fd: both interests on one fd
        // would need merging, which the live runner never asks for.
        let op = if self.known.insert((fd, write)) {
            libc::EPOLL_CTL_ADD
        } else {
            libc::EPOLL_CTL_MOD
        };
        ctl(self.ep.as_raw_fd(), op, fd, events, key)
    }

    pub(crate) fn watch(&mut self, sigs: &[Sig]) -> io::Result<()> {
        let set = sigset(sigs);
        self.sigs = sigs.to_vec();
        // SAFETY: blocks the set for this thread, and threads made later.
        check(unsafe { libc::pthread_sigmask(libc::SIG_BLOCK, &set, std::ptr::null_mut()) })?;
        // SAFETY: a fresh descriptor, owned at once.
        let fd = unsafe {
            OwnedFd::from_raw_fd(check(libc::signalfd(
                -1,
                &set,
                libc::SFD_NONBLOCK | libc::SFD_CLOEXEC,
            ))?)
        };
        ctl(
            self.ep.as_raw_fd(),
            libc::EPOLL_CTL_ADD,
            fd.as_raw_fd(),
            libc::EPOLLIN as u32,
            SIGNAL_KEY,
        )?;
        self.sigfd = Some(fd);
        Ok(())
    }

    /// SIGINT is no longer read here: another signalfd (the watchdog's)
    /// reads it, and adds each to the count returned, then wakes this
    /// poller, whose next wait reports them. A signal is read from one
    /// signalfd only, so the watchdog cannot watch beside this one.
    pub(crate) fn hand_over_int(&mut self) -> io::Result<Arc<AtomicU64>> {
        self.sigs.retain(|&s| s != Sig::Int);
        if let Some(fd) = &self.sigfd {
            // SAFETY: an existing signalfd given a new mask.
            check(unsafe {
                libc::signalfd(
                    fd.as_raw_fd(),
                    &sigset(&self.sigs),
                    libc::SFD_NONBLOCK | libc::SFD_CLOEXEC,
                )
            })?;
        }
        let forwarded = Arc::new(AtomicU64::new(0));
        self.forwarded = Some(forwarded.clone());
        Ok(forwarded)
    }

    pub(crate) fn wait(
        &mut self,
        timeout: Option<Duration>,
        out: &mut Vec<Event>,
    ) -> io::Result<()> {
        let ms = match timeout {
            None => -1,
            // Rounded up: a wait never ends before its time.
            Some(d) => d
                .as_nanos()
                .div_ceil(1_000_000)
                .min(libc::c_int::MAX as u128) as libc::c_int,
        };
        let mut events = [libc::epoll_event { events: 0, u64: 0 }; 64];
        // SAFETY: `events` is ours and sized as passed.
        let n = unsafe {
            libc::epoll_wait(
                self.ep.as_raw_fd(),
                events.as_mut_ptr(),
                events.len() as libc::c_int,
                ms,
            )
        };
        if n < 0 {
            let e = io::Error::last_os_error();
            return if e.kind() == io::ErrorKind::Interrupted {
                Ok(())
            } else {
                Err(e)
            };
        }
        for e in &events[..n as usize] {
            let (bits, key) = (e.events, e.u64);
            match key {
                WAKE_KEY => {
                    let mut v: u64 = 0;
                    // SAFETY: an eventfd read gives eight bytes, into ours.
                    unsafe {
                        libc::read(self.wake.efd.as_raw_fd(), (&mut v as *mut u64).cast(), 8);
                    }
                    out.push(Event::Wake);
                    let n = self
                        .forwarded
                        .as_ref()
                        .map_or(0, |f| f.swap(0, Ordering::SeqCst));
                    if n > 0 {
                        out.push(Event::Signal {
                            sig: Sig::Int,
                            count: n as u32,
                        });
                    }
                }
                SIGNAL_KEY => {
                    let Some(fd) = &self.sigfd else { continue };
                    loop {
                        // SAFETY: signalfd reads whole siginfo structs into ours.
                        let mut info: libc::signalfd_siginfo = unsafe { std::mem::zeroed() };
                        let n = unsafe {
                            libc::read(
                                fd.as_raw_fd(),
                                (&mut info as *mut libc::signalfd_siginfo).cast(),
                                std::mem::size_of::<libc::signalfd_siginfo>(),
                            )
                        };
                        if n <= 0 {
                            break;
                        }
                        if let Some(sig) = Sig::from_number(info.ssi_signo as libc::c_int) {
                            out.push(Event::Signal { sig, count: 1 });
                        }
                    }
                }
                key => {
                    let write = bits & libc::EPOLLOUT as u32 != 0;
                    out.push(Event::Io {
                        key,
                        interest: if write {
                            Interest::Write
                        } else {
                            Interest::Read
                        },
                        avail: None,
                        eof: bits & (libc::EPOLLHUP | libc::EPOLLRDHUP) as u32 != 0,
                    });
                }
            }
        }
        Ok(())
    }

    pub(crate) fn waker(&self) -> Arc<WakeFd> {
        self.wake.clone()
    }
}

fn sigset(sigs: &[Sig]) -> libc::sigset_t {
    // SAFETY: sigset operations on a set we own.
    let mut set: libc::sigset_t = unsafe { std::mem::zeroed() };
    unsafe { libc::sigemptyset(&mut set) };
    for s in sigs {
        // SAFETY: as above.
        unsafe { libc::sigaddset(&mut set, s.number()) };
    }
    set
}
