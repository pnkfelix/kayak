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
//! counts the tasks that can run, so that a wait of the program's own (a
//! thread of its own, or a `par` branch of one) with none left to make it
//! ready is a deadlock, an error, as Fixpt's is; a process that waits for
//! ever is no error (a server whose clients are done).

use std::collections::VecDeque;
use std::sync::{Condvar, Mutex, MutexGuard};

struct Sched {
    /// Tasks that can run: neither waiting nor joining a branch.
    running: usize,
    /// Tasks waiting for an I-cell or a message.
    waiting: usize,
    /// Changes made that a waiter may be waiting for: a message sent, an
    /// I-cell filled. A waiter looks again after each.
    epoch: u64,
    /// Waiters that have not looked again since the last change: one made
    /// ready may not have woken yet, so no deadlock is said while any has
    /// not.
    stale: usize,
    /// The first failure of a process, which fails the program.
    failed: Option<String>,
}

static SCHED: Mutex<Sched> = Mutex::new(Sched {
    running: 0,
    waiting: 0,
    epoch: 0,
    stale: 0,
    failed: None,
});
static WAKE: Condvar = Condvar::new();

fn lock() -> MutexGuard<'static, Sched> {
    SCHED.lock().unwrap_or_else(|p| p.into_inner())
}

fn held<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|p| p.into_inner())
}

/// A thread, as the scheduler counts it: from when it first meets the
/// scheduler, if it is the program's own; from before it starts, if it is
/// a process or a `par` branch, counted by the thread making it, so that
/// the count never dips meanwhile. Uncounted when the thread ends, by this
/// value's drop.
struct Task {
    /// The program's computation (a thread of its own, or a `par` branch of
    /// one), or a process's.
    program: bool,
}

impl Drop for Task {
    fn drop(&mut self) {
        lock().running -= 1;
        WAKE.notify_all();
    }
}

thread_local! {
    static TASK: std::cell::OnceCell<Task> = const { std::cell::OnceCell::new() };
}

/// Whether this thread is the program's; counted, if it is new.
fn program() -> bool {
    TASK.with(|t| {
        t.get_or_init(|| {
            lock().running += 1;
            Task { program: true }
        })
        .program
    })
}

/// This thread, a task its maker counted already.
fn counted(program: bool) {
    TASK.with(|t| {
        if t.set(Task { program }).is_err() {
            crate::fail("a task counted twice")
        }
    })
}

/// A process's failure, said by the program where it next meets the
/// scheduler, as its own.
fn check_failed(g: &MutexGuard<'static, Sched>, program: bool) {
    if program && let Some(m) = &g.failed {
        crate::fail(&m.clone())
    }
}

/// A change a waiter may be waiting for, made with the lock held: every
/// waiter looks again.
fn changed(mut g: MutexGuard<'static, Sched>) {
    g.epoch += 1;
    g.stale = g.waiting;
    drop(g);
    WAKE.notify_all();
}

/// Waits until `ready` gives a value, looking again after each change; a
/// wait of the program's when no task can run and every waiter has looked
/// since the last change, so none can, fails with `deadlock`.
fn wait_until<R>(program: bool, mut ready: impl FnMut() -> Option<R>, deadlock: &str) -> R {
    let mut g = lock();
    loop {
        check_failed(&g, program);
        if let Some(r) = ready() {
            return r;
        }
        let seen = g.epoch;
        g.running -= 1;
        g.waiting += 1;
        loop {
            if g.running == 0 && g.stale == 0 {
                if program {
                    g.running += 1;
                    g.waiting -= 1;
                    drop(g);
                    crate::fail(deadlock)
                }
                // The program may be waiting: it looks again, and finds it is stuck.
                WAKE.notify_all();
            }
            g = WAKE.wait(g).unwrap_or_else(|p| p.into_inner());
            if g.epoch != seen {
                g.stale = g.stale.saturating_sub(1);
                break;
            }
            check_failed(&g, program);
        }
        g.running += 1;
        g.waiting -= 1;
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
    let program = program();
    let g = lock();
    check_failed(&g, program);
    let mut slot = held(&c.value);
    if slot.is_some() {
        drop(slot);
        drop(g);
        crate::fail("an i-cell written twice")
    }
    *slot = Some(v);
    drop(slot);
    changed(g);
}

/// `(icell-get c)`: `c`'s value, waiting for its write.
pub fn icell_get<T: Copy>(c: ICell<T>) -> T {
    wait_until(
        program(),
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
    let program = program();
    let g = lock();
    check_failed(&g, program);
    held(&a.queue).push_back(m);
    changed(g);
}

/// `(receive box)`: the oldest message, waiting for one.
pub fn receive<M>(b: Inbox<M>) -> M {
    wait_until(
        program(),
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
    wait_until(
        program(),
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
    let program = program();
    {
        let mut g = lock();
        check_failed(&g, program);
        g.running += 1;
    }
    std::thread::Builder::new()
        .stack_size(STACK)
        .spawn(move || {
            counted(false);
            if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| body(mb))) {
                let m = panic_message(&*e);
                lock().failed.get_or_insert(m);
            }
        })
        .unwrap_or_else(|e| crate::fail(&format!("spawn: no thread: {e}")));
    mb
}

/// A `par` branch on another thread: counted by its maker, as the
/// program's if its `par` is; its failure kept for the join.
fn branch<B>(program: bool, g: impl FnOnce() -> B) -> std::thread::Result<B> {
    counted(program);
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(g))
}

/// `par`'s two branches, `f` here and `g` on a thread of its own, at once;
/// both their values. Each branch holds only what may cross (C102): rustc
/// checks it, by `Send`. A branch's failure is the `par`'s, its message
/// kept.
pub fn par2<A, B: Send>(f: impl FnOnce() -> A, g: impl FnOnce() -> B + Send) -> (A, B) {
    let program = program();
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
/// two run in turn, in the order the checker found (each filling an I-cell
/// before each awaiting it), which gives the same answer; with it, at
/// once, the crossing asserted by `kayak-sync`, the one crate with
/// `unsafe`.
pub fn par2_shared<A, B>(f: impl FnOnce() -> A, g: impl FnOnce() -> B) -> (A, B) {
    #[cfg(feature = "unsafe-par")]
    {
        let program = program();
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
/// half done (Fixpt's "Running"). A process's failure is the program's.
pub fn quiesce() {
    let program = program();
    let mut g = lock();
    loop {
        check_failed(&g, program);
        if g.running <= 1 {
            return;
        }
        g = WAKE.wait(g).unwrap_or_else(|p| p.into_inner());
    }
}
