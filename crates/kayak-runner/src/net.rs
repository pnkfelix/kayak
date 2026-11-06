//! The sim's localhost (M14): listeners by port, and connections as pairs
//! of streams, each with the bytes its peer wrote and not yet read. Fds
//! and ports come from counters, so a run is the same run every time.
//!
//! An operation that cannot be answered yet (an accept with no one
//! connecting, a read with nothing there) waits, in the order it came;
//! [`Net::serve`] answers whichever can be, after each change.
// coopt-source: crates/coopt-runner/src/net.rs@9283bba

use std::collections::{BTreeMap, VecDeque};

use crate::io::{Answer, Fd};

/// The first fd a socket gets: 0, 1 and 2 are the terminal's.
const FIRST_FD: u32 = 3;
/// Where ports for listeners at port 0 start: the IANA dynamic range.
const FIRST_PORT: u16 = 49152;

enum Sock {
    Listener {
        port: u16,
        /// Server ends of connections not yet accepted.
        backlog: VecDeque<u32>,
    },
    Stream {
        /// The other end, while it is open.
        peer: Option<u32>,
        /// What the peer wrote, not yet read.
        inbox: VecDeque<u8>,
        /// The peer closed: once the inbox is read, a read is the end.
        eof: bool,
        /// Reset: a read or a write is an error.
        reset: bool,
        /// Shut down for writing: a write is an error.
        shut: bool,
    },
}

/// An operation waiting for its answer.
#[derive(Clone, Copy)]
pub(crate) enum Wait {
    Accept(Fd),
    Read(Fd, u32),
}

#[derive(Default)]
pub(crate) struct Net {
    socks: BTreeMap<u32, Sock>,
    ports: BTreeMap<u16, u32>,
    next_fd: u32,
    next_port: u16,
    waiting: VecDeque<(u64, Wait)>,
}

fn stream() -> Sock {
    Sock::Stream {
        peer: None,
        inbox: VecDeque::new(),
        eof: false,
        reset: false,
        shut: false,
    }
}

impl Net {
    fn fresh_fd(&mut self) -> u32 {
        let fd = FIRST_FD + self.next_fd;
        self.next_fd += 1;
        fd
    }

    /// Whether `fd` is one of this network's.
    pub(crate) fn owns(fd: Fd) -> bool {
        fd.0 >= FIRST_FD
    }

    pub(crate) fn listen(&mut self, port: u16) -> Answer {
        let port = if port == 0 {
            loop {
                let p = FIRST_PORT.wrapping_add(self.next_port);
                self.next_port = self.next_port.wrapping_add(1);
                if p >= FIRST_PORT && !self.ports.contains_key(&p) {
                    break p;
                }
            }
        } else if self.ports.contains_key(&port) {
            return Answer::Err(format!("port {port}: address in use"));
        } else {
            port
        };
        let fd = self.fresh_fd();
        self.socks.insert(
            fd,
            Sock::Listener {
                port,
                backlog: VecDeque::new(),
            },
        );
        self.ports.insert(port, fd);
        Answer::Count(fd)
    }

    pub(crate) fn port(&self, fd: Fd) -> Answer {
        match self.socks.get(&fd.0) {
            Some(Sock::Listener { port, .. }) => Answer::Count(u32::from(*port)),
            Some(_) => Answer::Err(format!("fd {}: not a listener", fd.0)),
            None => Answer::Err(format!("fd {}: not open", fd.0)),
        }
    }

    /// A connection: the client's end answered, the server's put in the
    /// listener's backlog.
    pub(crate) fn connect(&mut self, port: u16) -> Answer {
        let Some(&l) = self.ports.get(&port) else {
            return Answer::Err(format!("port {port}: connection refused"));
        };
        let (c, s) = (self.fresh_fd(), self.fresh_fd());
        for (me, other) in [(c, s), (s, c)] {
            let mut sock = stream();
            if let Sock::Stream { peer, .. } = &mut sock {
                *peer = Some(other);
            }
            self.socks.insert(me, sock);
        }
        if let Some(Sock::Listener { backlog, .. }) = self.socks.get_mut(&l) {
            backlog.push_back(s);
        }
        Answer::Count(c)
    }

    /// Bytes to the peer's inbox, whole.
    pub(crate) fn write(&mut self, fd: Fd, bytes: Vec<u8>) -> Answer {
        let to = match self.socks.get(&fd.0) {
            Some(Sock::Stream { reset: true, .. }) => {
                return Answer::Err(format!("fd {}: connection reset", fd.0));
            }
            Some(Sock::Stream { shut: true, .. }) => {
                return Answer::Err(format!("fd {}: shut down for writing", fd.0));
            }
            Some(Sock::Stream { peer: Some(p), .. }) => *p,
            Some(Sock::Stream { peer: None, .. }) => {
                return Answer::Err(format!("fd {}: broken pipe", fd.0));
            }
            Some(_) => return Answer::Err(format!("fd {}: not a connection", fd.0)),
            None => return Answer::Err(format!("fd {}: not open", fd.0)),
        };
        let n = bytes.len() as u32;
        if let Some(Sock::Stream { inbox, .. }) = self.socks.get_mut(&to) {
            inbox.extend(bytes);
        }
        Answer::Count(n)
    }

    /// The connection `fd` is an end of, reset at both ends (a fault).
    pub(crate) fn reset(&mut self, fd: Fd) {
        let peer = match self.socks.get_mut(&fd.0) {
            Some(Sock::Stream { peer, reset, .. }) => {
                *reset = true;
                peer.take()
            }
            _ => None,
        };
        if let Some(p) = peer
            && let Some(Sock::Stream { peer, reset, .. }) = self.socks.get_mut(&p)
        {
            *reset = true;
            *peer = None;
        }
    }

    /// `fd` closed: its peer sees the end, once it has read what is
    /// there; a listener's connections not yet accepted are closed too.
    pub(crate) fn close(&mut self, fd: Fd) -> Answer {
        match self.socks.remove(&fd.0) {
            None => Answer::Err(format!("fd {}: not open", fd.0)),
            Some(Sock::Listener { port, backlog }) => {
                self.ports.remove(&port);
                for s in backlog {
                    self.close(Fd(s));
                }
                Answer::Count(0)
            }
            Some(Sock::Stream { peer, .. }) => {
                if let Some(p) = peer
                    && let Some(Sock::Stream { peer, eof, .. }) = self.socks.get_mut(&p)
                {
                    *peer = None;
                    *eof = true;
                }
                Answer::Count(0)
            }
        }
    }

    /// `fd` done writing: its peer sees the end, once it has read what is
    /// there, and may still write to it.
    pub(crate) fn shutdown(&mut self, fd: Fd) -> Answer {
        let peer = match self.socks.get_mut(&fd.0) {
            Some(Sock::Stream { shut, peer, .. }) => {
                *shut = true;
                *peer
            }
            Some(_) => return Answer::Err(format!("fd {}: not a connection", fd.0)),
            None => return Answer::Err(format!("fd {}: not open", fd.0)),
        };
        if let Some(p) = peer
            && let Some(Sock::Stream { eof, .. }) = self.socks.get_mut(&p)
        {
            *eof = true;
        }
        Answer::Count(0)
    }

    /// Whether `token` waited, and no longer does.
    pub(crate) fn cancel(&mut self, token: u64) -> bool {
        let before = self.waiting.len();
        self.waiting.retain(|(t, _)| *t != token);
        self.waiting.len() < before
    }

    pub(crate) fn wait(&mut self, token: u64, w: Wait) {
        self.waiting.push_back((token, w));
    }

    /// Answers whatever waits and can be answered, in the order they came.
    /// `take(n)` says how many of the `n` bytes there a read takes.
    pub(crate) fn serve(&mut self, take: &mut dyn FnMut(usize) -> usize) -> Vec<(u64, Answer)> {
        let mut out = Vec::new();
        let mut still = VecDeque::new();
        for (token, w) in std::mem::take(&mut self.waiting) {
            match self.try_answer(w, take) {
                Some(a) => out.push((token, a)),
                None => still.push_back((token, w)),
            }
        }
        self.waiting = still;
        out
    }

    fn try_answer(&mut self, w: Wait, take: &mut dyn FnMut(usize) -> usize) -> Option<Answer> {
        match w {
            Wait::Accept(fd) => match self.socks.get_mut(&fd.0) {
                Some(Sock::Listener { backlog, .. }) => backlog.pop_front().map(Answer::Count),
                Some(_) => Some(Answer::Err(format!("fd {}: not a listener", fd.0))),
                None => Some(Answer::Err(format!("fd {}: closed", fd.0))),
            },
            Wait::Read(fd, max) => match self.socks.get_mut(&fd.0) {
                Some(Sock::Stream { reset: true, .. }) => {
                    Some(Answer::Err(format!("fd {}: connection reset", fd.0)))
                }
                Some(Sock::Stream { inbox, eof, .. }) => {
                    if inbox.is_empty() {
                        return eof.then(|| Answer::Bytes(Vec::new()));
                    }
                    let n = inbox.len().min(max.max(1) as usize);
                    let n = take(n).clamp(1, n);
                    Some(Answer::Bytes(inbox.drain(..n).collect()))
                }
                Some(_) => Some(Answer::Err(format!("fd {}: not a connection", fd.0))),
                None => Some(Answer::Err(format!("fd {}: closed", fd.0))),
            },
        }
    }
}
