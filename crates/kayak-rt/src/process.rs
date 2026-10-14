//! FX-26's processes and `par` (`CLAUDES_CHOICES.md` C101, C102; Fixpt's
//! `docs/fx26.md`, "Processes and `par`"): each process an OS thread, each
//! `par` branch past the first a scoped one.
//!
//! In safe Rust: what crosses into a thread is what the checker lets cross,
//! and the emitter writes its type so that rustc sees that too (C102): a
//! transmissible procedure is a `dyn Fn` that is `Send + Sync`, and every
//! transmissible value's Rust type is `Send`, so `spawn`'s, `send`'s and
//! `par2`'s bounds are checked by rustc as well as by KFX's checker. A
//! message is shared, not copied: what is transmissible is never written.
//!
//! One lock, [`SCHED`], is held by every waiter while it looks, and one
//! condition variable wakes them all when anything changes. The lock also
//! counts the tasks that can run, so that a wait of the program's own (its
//! main thread, or a `par` branch of it) with none left to make it ready is
//! a deadlock, an error, as Fixpt's is; a process that waits for ever is no
//! error (a server whose clients are done).

use std::cell::Cell;
use std::collections::VecDeque;
use std::sync::{Condvar, Mutex, MutexGuard};

struct Sched {
    /// Tasks that can run: neither waiting nor joining a branch.
    running: usize,
    /// The first failure of a process, which fails the program.
    failed: Option<String>,
}

static SCHED: Mutex<Sched> = Mutex::new(Sched {
    running: 1,
    failed: None,
});
static WAKE: Condvar = Condvar::new();

thread_local! {
    /// Whether this thread is the program's computation: its main thread,
    /// or a `par` branch of it; not a process's.
    static PROGRAM: Cell<bool> = const { Cell::new(true) };
}

fn lock() -> MutexGuard<'static, Sched> {
    SCHED.lock().unwrap_or_else(|p| p.into_inner())
}

fn held<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|p| p.into_inner())
}

/// A process's failure, said by the program where it next meets the
/// scheduler, as its own.
fn check_failed(g: &MutexGuard<'static, Sched>) {
    if PROGRAM.with(Cell::get)
        && let Some(m) = &g.failed
    {
        crate::fail(&m.clone())
    }
}

/// Waits, the lock held, until `ready` gives a value; a wait of the
/// program's with no task left that can run fails with `deadlock`.
fn wait_until<R>(
    mut g: MutexGuard<'static, Sched>,
    mut ready: impl FnMut() -> Option<R>,
    deadlock: &str,
) -> R {
    loop {
        check_failed(&g);
        if let Some(r) = ready() {
            return r;
        }
        g.running -= 1;
        if g.running == 0 {
            if PROGRAM.with(Cell::get) {
                g.running += 1;
                drop(g);
                crate::fail(deadlock)
            }
            // The program may be waiting: it looks again, and finds it is stuck.
            WAKE.notify_all();
        }
        g = WAKE.wait(g).unwrap_or_else(|p| p.into_inner());
        g.running += 1;
    }
}

/// `(icell T R)`: empty until its one write. Shared between threads only
/// when `T` is `Send`, as a transmissible type's Rust is.
pub struct ICellBox<T> {
    value: Mutex<Option<T>>,
}

pub type ICell<T> = &'static ICellBox<T>;

/// `(make-icell)`: in the heap, wherever it was asked for, as FX-26's is.
pub fn make_icell<T>() -> ICell<T> {
    crate::alloc(ICellBox {
        value: Mutex::new(None),
    })
}

/// `(icell-put! c v)`: `c`'s one write, which wakes whoever waits for it.
pub fn icell_put_x<T: Copy>(c: ICell<T>, v: T) {
    let g = lock();
    check_failed(&g);
    let mut slot = held(&c.value);
    if slot.is_some() {
        drop(slot);
        drop(g);
        crate::fail("an i-cell written twice")
    }
    *slot = Some(v);
    drop(slot);
    drop(g);
    WAKE.notify_all();
}

/// `(icell-get c)`: `c`'s value, waiting for its write.
pub fn icell_get<T: Copy>(c: ICell<T>) -> T {
    let g = lock();
    wait_until(
        g,
        || *held(&c.value),
        "an i-cell read before it was written, and no task is left to write it: a deadlock",
    )
}

/// A process's mailbox: `(addr M R)` the right to send to it, `(inbox M
/// r)` the right to take from it, its process's alone.
pub struct MailboxBox<M> {
    queue: Mutex<VecDeque<M>>,
}

pub type Addr<M> = &'static MailboxBox<M>;
pub type Inbox<M> = &'static MailboxBox<M>;

/// `(send a m)`: `m` to the process whose address is `a`; a sender's
/// messages arrive in the order sent.
pub fn send<M: Send>(a: Addr<M>, m: M) {
    let g = lock();
    check_failed(&g);
    held(&a.queue).push_back(m);
    drop(g);
    WAKE.notify_all();
}

/// `(receive box)`: the oldest message, waiting for one.
pub fn receive<M>(b: Inbox<M>) -> M {
    let g = lock();
    wait_until(
        g,
        || held(&b.queue).pop_front(),
        "a message no task is left to send: a deadlock",
    )
}

/// What `pselect` waits on: an I-cell, ready once filled; a mailbox, once
/// it has a message.
pub trait Waitable {
    fn ready(&self) -> bool;
}

impl<T> Waitable for ICellBox<T> {
    fn ready(&self) -> bool {
        held(&self.value).is_some()
    }
}

impl<M> Waitable for MailboxBox<M> {
    fn ready(&self) -> bool {
        !held(&self.queue).is_empty()
    }
}

/// `pselect`'s wait: the number of the first of `events` that is ready,
/// once one is. Only the process takes from its own mailbox, and a cell
/// once filled stays so, so the event chosen is still ready when its arm
/// takes from it.
pub fn sync(events: &[&dyn Waitable]) -> usize {
    let g = lock();
    wait_until(
        g,
        || events.iter().position(|e| e.ready()),
        "a `pselect` whose events no task is left to make ready: a deadlock",
    )
}

/// A thread's stack, as big as the main thread's is made.
const STACK: usize = 1 << 30;

fn panic_message(e: &(dyn std::any::Any + Send)) -> String {
    e.downcast_ref::<String>()
        .cloned()
        .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_else(|| "a process failed".to_string())
}

/// `(spawn (box M r) body ...)`: a process on a thread of its own, running
/// `body` with its mailbox; its address.
pub fn spawn<M: Send + 'static>(body: impl FnOnce(Inbox<M>) + Send + 'static) -> Addr<M> {
    let mb: Addr<M> = crate::alloc(MailboxBox {
        queue: Mutex::new(VecDeque::new()),
    });
    {
        let mut g = lock();
        check_failed(&g);
        g.running += 1;
    }
    std::thread::Builder::new()
        .stack_size(STACK)
        .spawn(move || {
            PROGRAM.with(|p| p.set(false));
            let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| body(mb)));
            let mut g = lock();
            g.running -= 1;
            if let Err(e) = r {
                let m = panic_message(&*e);
                g.failed.get_or_insert(m);
            }
            drop(g);
            WAKE.notify_all();
        })
        .unwrap_or_else(|e| crate::fail(&format!("spawn: no thread: {e}")));
    mb
}

/// What a `par` branch on another thread does around its work: counted as
/// a task that can run, as the program's if its `par` is, and its failure
/// kept for the join.
fn branch<B>(program: bool, g: impl FnOnce() -> B) -> std::thread::Result<B> {
    PROGRAM.with(|p| p.set(program));
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(g));
    lock().running -= 1;
    WAKE.notify_all();
    r
}

/// `par`'s two branches, `f` here and `g` on a thread of its own, at once;
/// both their values. Each branch holds only what may cross (C102): rustc
/// checks it, by `Send`. A branch's failure is the `par`'s, its message
/// kept.
pub fn par2<A, B: Send>(f: impl FnOnce() -> A, g: impl FnOnce() -> B + Send) -> (A, B) {
    let program = PROGRAM.with(Cell::get);
    std::thread::scope(|scope| {
        lock().running += 1;
        let other = std::thread::Builder::new()
            .stack_size(STACK)
            .spawn_scoped(scope, move || branch(program, g))
            .unwrap_or_else(|e| crate::fail(&format!("par: no thread: {e}")));
        let a = f();
        joined(a, || other.join())
    })
}

/// `f`'s value `a` and the other branch's, joined: joining is waiting, as
/// the other branch may be what can run.
fn joined<A, B>(
    a: A,
    join: impl FnOnce() -> std::thread::Result<std::thread::Result<B>>,
) -> (A, B) {
    lock().running -= 1;
    WAKE.notify_all();
    let r = join();
    lock().running += 1;
    match r {
        Ok(Ok(b)) => (a, b),
        Ok(Err(e)) | Err(e) => std::panic::resume_unwind(e),
    }
}

/// `par`'s two branches when they share what may be written, in regions
/// none of the others touches (the checker's interference table, C101).
/// Rust cannot see that, so (C102): without the `unsafe-par` feature, the
/// two run in turn, which gives the same answer; with it, at once, the
/// crossing asserted by `kayak-sync`, the one crate with `unsafe`.
pub fn par2_shared<A, B>(f: impl FnOnce() -> A, g: impl FnOnce() -> B) -> (A, B) {
    #[cfg(feature = "unsafe-par")]
    {
        let program = PROGRAM.with(Cell::get);
        lock().running += 1;
        kayak_sync::par2_unchecked(f, move || branch(program, g), |a, join| joined(a, join))
    }
    #[cfg(not(feature = "unsafe-par"))]
    {
        let a = f();
        (a, g())
    }
}

/// Waits until no other task can run: the end of a top-level form, after
/// which a definition may change a global no running process then sees
/// half done (Fixpt's "Running"); and the end of `main`. A process's
/// failure is the program's.
pub fn quiesce() {
    let mut g = lock();
    loop {
        check_failed(&g);
        if g.running <= 1 {
            return;
        }
        g = WAKE.wait(g).unwrap_or_else(|p| p.into_inner());
    }
}
