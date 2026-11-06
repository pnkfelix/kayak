//! kqueue (macOS): interest, waking and signals in one call.
//! `EVFILT_USER` wakes; `EVFILT_SIGNAL` counts a signal even when it is
//! ignored, which it is, so there is no handler (R4 §3.1, verified there).
// coopt-source: crates/coopt-sys/src/kqueue.rs@9283bba

use std::io;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::sync::Arc;
use std::time::Duration;

use crate::{Event, Interest, Sig, check};

/// The `EVFILT_USER` identifier of the waker.
const WAKE_IDENT: usize = 0;

pub(crate) struct WakeFd {
    kq: OwnedFd,
}

fn kevent_struct(ident: usize, filter: i16, flags: u16, fflags: u32, udata: u64) -> libc::kevent {
    libc::kevent {
        ident,
        filter,
        flags,
        fflags,
        data: 0,
        udata: udata as *mut libc::c_void,
    }
}

/// Submits `changes`, each with `EV_RECEIPT`, and checks every receipt.
fn submit(kq: RawFd, changes: &mut [libc::kevent]) -> io::Result<()> {
    for c in changes.iter_mut() {
        c.flags |= libc::EV_RECEIPT;
    }
    let mut receipts = changes.to_vec();
    // SAFETY: both arrays are ours and sized as passed; with EV_RECEIPT
    // the kernel writes one receipt for each change into `receipts`.
    let n = check(unsafe {
        libc::kevent(
            kq,
            changes.as_ptr(),
            changes.len() as libc::c_int,
            receipts.as_mut_ptr(),
            receipts.len() as libc::c_int,
            std::ptr::null(),
        )
    })?;
    for r in &receipts[..n as usize] {
        if r.flags & libc::EV_ERROR != 0 && r.data != 0 {
            return Err(io::Error::from_raw_os_error(r.data as i32));
        }
    }
    Ok(())
}

impl WakeFd {
    pub(crate) fn wake(&self) -> io::Result<()> {
        let mut c = [kevent_struct(
            WAKE_IDENT,
            libc::EVFILT_USER,
            0,
            libc::NOTE_TRIGGER,
            0,
        )];
        submit(self.kq.as_raw_fd(), &mut c)
    }
}

pub(crate) struct Poller {
    /// Shared with the wakers: kevent on one kqueue from several threads
    /// is allowed.
    wake: Arc<WakeFd>,
}

impl Poller {
    pub(crate) fn new() -> io::Result<Poller> {
        // SAFETY: kqueue takes nothing, and gives a fresh descriptor.
        let fd = check(unsafe { libc::kqueue() })?;
        // SAFETY: nothing else owns it.
        let kq = unsafe { OwnedFd::from_raw_fd(fd) };
        // SAFETY: setting close-on-exec on our own descriptor.
        check(unsafe { libc::fcntl(fd, libc::F_SETFD, libc::FD_CLOEXEC) })?;
        let mut c = [kevent_struct(
            WAKE_IDENT,
            libc::EVFILT_USER,
            libc::EV_ADD | libc::EV_CLEAR,
            0,
            0,
        )];
        submit(fd, &mut c)?;
        Ok(Poller {
            wake: Arc::new(WakeFd { kq }),
        })
    }

    fn kq(&self) -> RawFd {
        self.wake.kq.as_raw_fd()
    }

    pub(crate) fn arm(&mut self, fd: RawFd, interest: Interest, key: u64) -> io::Result<()> {
        let filter = match interest {
            Interest::Read => libc::EVFILT_READ,
            Interest::Write => libc::EVFILT_WRITE,
        };
        let mut c = [kevent_struct(
            fd as usize,
            filter,
            libc::EV_ADD | libc::EV_ONESHOT,
            0,
            key,
        )];
        submit(self.kq(), &mut c)
    }

    pub(crate) fn watch(&mut self, sigs: &[Sig]) -> io::Result<()> {
        for s in sigs {
            // SAFETY: SIG_IGN installs no handler; nothing of ours runs
            // in signal context.
            unsafe {
                libc::signal(s.number(), libc::SIG_IGN);
            }
            let mut c = [kevent_struct(
                s.number() as usize,
                libc::EVFILT_SIGNAL,
                libc::EV_ADD,
                0,
                0,
            )];
            submit(self.kq(), &mut c)?;
        }
        Ok(())
    }

    pub(crate) fn wait(
        &mut self,
        timeout: Option<Duration>,
        out: &mut Vec<Event>,
    ) -> io::Result<()> {
        let ts = timeout.map(|d| libc::timespec {
            tv_sec: d.as_secs().min(i64::MAX as u64) as libc::time_t,
            tv_nsec: d.subsec_nanos() as libc::c_long,
        });
        let mut events = [kevent_struct(0, 0, 0, 0, 0); 64];
        // SAFETY: `events` is ours and sized as passed; the timeout, if
        // any, lives across the call.
        let n = unsafe {
            libc::kevent(
                self.kq(),
                std::ptr::null(),
                0,
                events.as_mut_ptr(),
                events.len() as libc::c_int,
                ts.as_ref()
                    .map_or(std::ptr::null(), |t| t as *const libc::timespec),
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
            out.push(match e.filter {
                libc::EVFILT_USER => Event::Wake,
                libc::EVFILT_SIGNAL => match Sig::from_number(e.ident as libc::c_int) {
                    Some(sig) => Event::Signal {
                        sig,
                        count: e.data.max(1) as u32,
                    },
                    None => continue,
                },
                f => Event::Io {
                    key: e.udata as u64,
                    interest: if f == libc::EVFILT_WRITE {
                        Interest::Write
                    } else {
                        Interest::Read
                    },
                    avail: Some(e.data.max(0) as usize),
                    eof: e.flags & libc::EV_EOF != 0,
                },
            });
        }
        Ok(())
    }

    pub(crate) fn waker(&self) -> Arc<WakeFd> {
        self.wake.clone()
    }
}
