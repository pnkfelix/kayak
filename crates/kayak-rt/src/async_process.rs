//! FX-26's processes in the emitter's async mode (C380): each a task of
//! the scheduler `suspend::run` runs the program on (kayak-sched, Coopt's,
//! C368), where the synchronous mode's are threads (`process`, C101).
//!
//! A wait (`receive`, `icell-get`, `pselect`) is a future: not ready, it
//! keeps its task's waker and is pending; `send` and `icell-put!` wake
//! every task that waits, which looks again, as the threads' condition
//! variable does. Every task runs on the one thread, so what is shared is
//! in thread locals. A process is spawned pure: it shares with the rest
//! only what is transmissible, which none writes, and mailboxes and
//! I-cells, which it reaches only at its waits; so its sections may
//! interleave with any other's (kayak-sched's effects).
//!
//! `par`'s branches are futures of the same task, polled in turn: each
//! runs until it waits, or ends.

use crate::process::{Addr, ICell, Inbox, MailboxBox, Waitable, held, panic_message};
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;
use std::task::{Context, Poll, Waker};

thread_local! {
    /// The tasks waiting, woken at the next change.
    static WAITERS: RefCell<Vec<Waker>> = const { RefCell::new(Vec::new()) };
    /// The earliest deadline a waiter waits for, if any does.
    static DEADLINE: Cell<Option<i64>> = const { Cell::new(None) };
    /// Whether a waiter waits for what comes from outside (the standard
    /// input, programs run beside this one).
    static OUTSIDE: Cell<bool> = const { Cell::new(false) };
    /// Whether the task polled is the program's (`main`'s).
    static PROGRAM: Cell<bool> = const { Cell::new(false) };
    /// Whether it is a monitored process, whose failure no hook says.
    static MONITORED: Cell<bool> = const { Cell::new(false) };
    /// What the program's latest wait says if no task can end it.
    static STUCK: Cell<&'static str> = const { Cell::new("abort-with-continuation: no prompt for the tag") };
    /// An unmonitored process's failure, the program's.
    static FAILED: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub(crate) fn monitored() -> bool {
    MONITORED.with(Cell::get)
}

/// A change a waiter may be waiting for: every waiter looks again.
pub(crate) fn changed() {
    let woken = WAITERS.with(|w| std::mem::take(&mut *w.borrow_mut()));
    DEADLINE.with(|d| d.set(None));
    OUTSIDE.with(|o| o.set(false));
    for w in woken {
        w.wake();
    }
}

/// A wait: ready when `ready` gives a value.
struct Wait<F> {
    ready: F,
    deadline: Option<i64>,
    outside: bool,
    deadlock: &'static str,
}

impl<R, F: FnMut() -> Option<R> + Unpin> Future for Wait<F> {
    type Output = R;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<R> {
        if let Some(r) = (self.ready)() {
            return Poll::Ready(r);
        }
        WAITERS.with(|w| w.borrow_mut().push(cx.waker().clone()));
        if let Some(t) = self.deadline {
            DEADLINE.with(|d| d.set(Some(d.get().map_or(t, |u| u.min(t)))));
        }
        if self.outside {
            OUTSIDE.with(|o| o.set(true));
        }
        if PROGRAM.with(Cell::get) {
            STUCK.with(|s| s.set(self.deadlock));
        }
        Poll::Pending
    }
}

/// A task's wait until `ready` gives a value (`async_io`, C381).
pub(crate) fn wait_for<R>(ready: impl FnMut() -> Option<R> + Unpin) -> impl Future<Output = R> {
    Wait {
        ready,
        deadline: None,
        outside: false,
        deadlock: "a wait no task is left to end: a deadlock",
    }
}

/// A wait for the standard input, which comes from outside.
pub(crate) fn wait_for_input<R>(
    ready: impl FnMut() -> Option<R> + Unpin,
) -> impl Future<Output = R> {
    Wait {
        ready,
        deadline: None,
        outside: true,
        deadlock: "a read of the standard input that cannot end",
    }
}

/// A wait for news from outside the runner's sight (a program run beside
/// this one), which wakes the runner when it comes (C382).
pub(crate) fn wait_for_outside<R>(
    ready: impl FnMut() -> Option<R> + Unpin,
) -> impl Future<Output = R> {
    Wait {
        ready,
        deadline: None,
        outside: true,
        deadlock: "a wait for a program run beside this one that cannot end",
    }
}

/// A wait until the clock (`world::clock`'s) reaches `t`.
pub(crate) fn wait_until_clock(t: i64) -> impl Future<Output = ()> {
    Wait {
        ready: move || (crate::world::clock() >= t).then_some(()),
        deadline: Some(t),
        outside: false,
        deadlock: "a sleep that cannot end",
    }
}

/// `(make-icell)`, as the synchronous mode's.
pub use crate::process::make_icell;

/// `(icell-put! c v)`: `c`'s one write, which wakes whoever waits.
pub fn icell_put_x<T: Copy>(c: ICell<T>, v: T) {
    let mut slot = held(&c.value);
    if slot.is_some() {
        drop(slot);
        crate::fail("an i-cell written twice")
    }
    *slot = Some(v);
    drop(slot);
    changed();
}

/// `(icell-get c)`: `c`'s value, once written.
pub fn icell_get<T: Copy>(c: ICell<T>) -> impl Future<Output = T> {
    Wait {
        ready: move || *held(&c.value),
        deadline: None,
        outside: false,
        deadlock: "an i-cell read before it was written, and no task is left to write it: a deadlock",
    }
}

/// `(send a m)`: `m` to `a`'s process, after what was sent before.
pub fn send<M>(a: Addr<M>, m: M) {
    held(&a.queue).push_back(m);
    changed();
}

/// `(receive box)`: the oldest message, once there is one.
pub fn receive<M>(b: Inbox<M>) -> impl Future<Output = M> {
    Wait {
        ready: move || held(&b.queue).pop_front(),
        deadline: None,
        outside: false,
        deadlock: "a message no task is left to send: a deadlock",
    }
}

/// `pselect`'s wait: the number of the first of `events` ready, once one is.
pub fn sync<'a>(events: &'a [&'a dyn Waitable]) -> impl Future<Output = usize> + 'a {
    Wait {
        ready: move || events.iter().position(|e| e.ready()),
        deadline: events.iter().filter_map(|e| e.deadline()).min(),
        outside: events.iter().any(|e| e.outside()),
        deadlock: "a `pselect` whose events no task is left to make ready: a deadlock",
    }
}

/// A process: its future, polled as the scheduler's task, its failure
/// caught at each poll; an unmonitored one's the program's, a monitored
/// one's said to its watcher (`on_failure`).
struct Process {
    body: Pin<Box<dyn Future<Output = ()>>>,
    on_failure: Option<Box<dyn Fn(String)>>,
}

impl Future for Process {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let monitored = self.on_failure.is_some();
        let (program, was) = (
            PROGRAM.with(|p| p.replace(false)),
            MONITORED.with(|m| m.replace(monitored)),
        );
        let r =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.body.as_mut().poll(cx)));
        PROGRAM.with(|p| p.set(program));
        MONITORED.with(|m| m.set(was));
        match r {
            Ok(p) => p,
            Err(e) => {
                let message = panic_message(&*e);
                match &self.on_failure {
                    Some(said) => said(message),
                    None => FAILED.with(|f| {
                        f.borrow_mut().get_or_insert(message);
                    }),
                }
                Poll::Ready(())
            }
        }
    }
}

fn start<M: 'static, F: Future<Output = ()> + 'static>(
    on_failure: Option<Box<dyn Fn(String)>>,
    body: impl FnOnce(Inbox<M>) -> F,
) -> Addr<M> {
    let mb: Addr<M> = crate::alloc(MailboxBox {
        queue: Mutex::new(VecDeque::new()),
    });
    let Some(cx) = crate::suspend::task_cx() else {
        crate::fail("spawn: no scheduler to run the process")
    };
    let process = Process {
        body: Box::pin(body(mb)),
        on_failure,
    };
    let spawn = kayak_sched::Spawn::new("process").effect(kayak_sched::Effect::pure());
    cx.spawn(spawn, move |_| process);
    mb
}

/// `(spawn (box M r) body ...)`: a process, a task running `body` with its
/// mailbox; its address. Its failure is the program's.
pub fn spawn<M: 'static, F: Future<Output = ()> + 'static>(
    body: impl FnOnce(Inbox<M>) -> F,
) -> Addr<M> {
    start(None, body)
}

/// `(spawn-monitored w on-failure (box M r) body ...)` (C108): its
/// failure no failure of the program: `watcher` is sent `on_failure` of
/// what it says.
pub fn spawn_monitored<W: 'static, M: 'static, F: Future<Output = ()> + 'static>(
    watcher: Addr<W>,
    on_failure: impl Fn(crate::Str) -> W + 'static,
    body: impl FnOnce(Inbox<M>) -> F,
) -> Addr<M> {
    start(
        Some(Box::new(move |m| {
            send(watcher, on_failure(crate::leak_string(m)))
        })),
        body,
    )
}

/// `par`'s two branches, futures of this task, polled in turn until both
/// end: both their values.
pub fn par2<A, B>(
    f: impl Future<Output = A>,
    g: impl Future<Output = B>,
) -> impl Future<Output = (A, B)> {
    Par2 {
        f: Box::pin(f),
        g: Box::pin(g),
        a: None,
        b: None,
    }
}

struct Par2<F: Future, G: Future> {
    f: Pin<Box<F>>,
    g: Pin<Box<G>>,
    a: Option<F::Output>,
    b: Option<G::Output>,
}

impl<F: Future, G: Future> Future for Par2<F, G> {
    type Output = (F::Output, G::Output);

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = &mut *self;
        if this.a.is_none()
            && let Poll::Ready(a) = this.f.as_mut().poll(cx)
        {
            this.a = Some(a);
        }
        if this.b.is_none()
            && let Poll::Ready(b) = this.g.as_mut().poll(cx)
        {
            this.b = Some(b);
        }
        match (this.a.take(), this.b.take()) {
            (Some(a), Some(b)) => Poll::Ready((a, b)),
            (a, b) => {
                (this.a, this.b) = (a, b);
                Poll::Pending
            }
        }
    }
}

impl<F: Future, G: Future> Unpin for Par2<F, G> {}

/// The program's computation, `main`'s: its waits say what a deadlock is.
pub(crate) fn program<T>(f: impl Future<Output = T>) -> impl Future<Output = T> {
    Program { inner: Box::pin(f) }
}

struct Program<F: Future> {
    inner: Pin<Box<F>>,
}

impl<F: Future> Future for Program<F> {
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<F::Output> {
        let was = PROGRAM.with(|p| p.replace(true));
        let r = self.inner.as_mut().poll(cx);
        PROGRAM.with(|p| p.set(was));
        r
    }
}

impl<F: Future> Unpin for Program<F> {}

/// An unmonitored process's failure, which is the program's.
pub(crate) fn check_failed() {
    if let Some(m) = FAILED.with(|f| f.borrow_mut().take()) {
        crate::fail(&m)
    }
}

/// No task can run: whether a waiter waits for a deadline or for what
/// comes from outside, and has been woken to look again once it may be
/// ready (the clock reaching the deadline; a little time for the outside).
pub(crate) fn waited() -> bool {
    let deadline = DEADLINE.with(Cell::get);
    let outside = OUTSIDE.with(Cell::get);
    if crate::async_io::installed() {
        // The runner waits for what comes (C381): input, or a write taken;
        // news of a program run beside this one wakes it too (C382), so a
        // wait for one is looked at again within a second, at the latest.
        let soon = outside.then(|| crate::world::clock() + 1_000_000_000);
        let until = match (deadline, soon) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        };
        let more = crate::async_io::idle(until);
        changed();
        return more;
    }
    if deadline.is_none() && !outside {
        return false;
    }
    let mut pause = std::time::Duration::from_millis(1);
    if let Some(t) = deadline {
        let left = t.saturating_sub(crate::world::clock());
        if !outside {
            pause = std::time::Duration::from_nanos(left.max(0) as u64);
        } else {
            pause = pause.min(std::time::Duration::from_nanos(left.max(0) as u64));
        }
    }
    std::thread::sleep(pause);
    changed();
    true
}

/// What the program's wait says, when no task is left to end it.
pub(crate) fn stuck() -> &'static str {
    STUCK.with(Cell::get)
}
