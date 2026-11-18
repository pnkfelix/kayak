//! Suspension: FX-26's composable continuations, where they are used as
//! a task's suspension is, as Rust futures (`docs/research/async-rust.md`).
//!
//! The one form: `(abort-with-continuation tag f)`, which is FX-26's
//! `(call-with-composable-continuation (lambda (k) (abort-current-continuation
//! tag (f k))) tag)` with `k` resumable once (`kfx/prelude.fx` defines it so
//! for Fixpt, and a second resume is an error in both). The rest of the
//! computation up to `tag`'s prompt is then the prompt's body, a future
//! suspended at the capture: `k` is that future. A procedure that may
//! capture is an `async fn`; a prompt polls its body; a capture registers
//! itself, here, and returns `Pending`; the prompt whose tag it is takes the
//! body as `k`, and gives `f k` to its handler. Nothing waits on a `Waker`:
//! the scheduler is the program's own (a run queue of continuations), and
//! the top level polls with a no-op one.

use crate::escape::{Tag, Unwind};
use std::any::Any;
use std::cell::{Cell, RefCell};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

/// A prompt's body, or a continuation's: its answer, or an abort.
pub type Body<R> = BodyAt<'static, R>;

/// `Body`, of a future that may hold what lives for `'a` (C376): a
/// `letrena` region's data, in the emitter's async mode.
pub type BodyAt<'a, R> = Pin<Box<dyn Future<Output = Result<R, Unwind>> + 'a>>;

/// A capture on its way to its prompt: the tag's identity, and what makes
/// the abort's payload from the body (erased: the prompt downcasts it).
struct Pending {
    tag: u64,
    make: Box<dyn Any>,
}

thread_local! {
    static CAPTURE: RefCell<Option<Pending>> = const { RefCell::new(None) };
}

/// `(composable V R E r)`: resumed with a `V`, the body's rest up to the
/// prompt, answering an `R`. A KFX value is `Copy`, so this is a reference;
/// the body is taken out at its one resume.
pub struct Cont<V: 'static, R: 'static> {
    body: RefCell<Option<Body<R>>>,
    given: &'static Cell<Option<V>>,
}

/// What a capture site leaves for the prompt: given the body, the payload.
type Make<R, H> = Box<dyn FnOnce(Body<R>) -> H>;

/// A continuation as KFX has it: a procedure that may suspend, of the
/// value it is resumed with (`(subr (maxeff D (goto r) (comefrom r) ...)
/// (V) A)`), resumable once.
pub type Resume<V, R> = &'static dyn Fn(V) -> Body<R>;

/// A future, boxed: what a procedure value that may suspend returns, and a
/// call of a procedure that may suspend awaits (so that an `async fn` may
/// call itself).
pub fn body<'a, R>(f: impl Future<Output = Result<R, Unwind>> + 'a) -> BodyAt<'a, R> {
    Box::pin(f)
}

/// A future of a plain value, boxed: what a procedure value of the emitter's
/// async mode returns where its effect cannot abort (C372), and the box of
/// a call of such a procedure that may close a cycle.
pub type Fut<R> = FutAt<'static, R>;

/// `Fut`, of a future that may hold what lives for `'a` (C376).
pub type FutAt<'a, R> = Pin<Box<dyn Future<Output = R> + 'a>>;

/// `body`'s, for a future of a plain value (C372).
pub fn boxed<'a, R>(f: impl Future<Output = R> + 'a) -> FutAt<'a, R> {
    Box::pin(f)
}

/// The capture's future: `Pending` once, having registered; then the value
/// the continuation was resumed with.
struct Capture<V: 'static> {
    given: &'static Cell<Option<V>>,
    registered: bool,
}

impl<V: 'static> Future for Capture<V> {
    type Output = Result<V, Unwind>;
    fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.registered {
            self.registered = true;
            return Poll::Pending;
        }
        match self.given.take() {
            Some(v) => Poll::Ready(Ok(v)),
            None => crate::fail("a continuation polled before it was resumed"),
        }
    }
}

/// `(abort-with-continuation tag f)`: the rest of the computation up to
/// `tag`'s prompt, `k`, is given to `f`, and `f k` is the abort's payload.
/// Its value, when `k` is resumed, is what `k` is given.
pub fn abort_with_continuation<V, R, H>(
    tag: Tag<R, H>,
    f: impl FnOnce(Resume<V, R>) -> H + 'static,
) -> impl Future<Output = Result<V, Unwind>>
where
    V: 'static,
    R: 'static,
    H: 'static,
{
    let given: &'static Cell<Option<V>> = crate::leak(Cell::new(None));
    let make: Make<R, H> = Box::new(move |body| {
        let k: &'static Cont<V, R> = crate::leak(Cont {
            body: RefCell::new(Some(body)),
            given,
        });
        f(crate::leak(move |v| -> Body<R> { Box::pin(k.resume(v)) }))
    });
    CAPTURE.with(|c| {
        if c.borrow().is_some() {
            crate::fail("abort-with-continuation: a capture already pending");
        }
        *c.borrow_mut() = Some(Pending {
            tag: tag.id(),
            make: Box::new(make),
        });
    });
    Capture {
        given,
        registered: false,
    }
}

impl<V: 'static, R: 'static> Cont<V, R> {
    /// `(k v)`: the body, resumed with `v`, run until it ends or suspends
    /// again (to the prompt around the resume, which the `.await` reaches).
    pub async fn resume(&'static self, v: V) -> Result<R, Unwind> {
        let body = match self.body.borrow_mut().take() {
            Some(b) => b,
            None => crate::fail("a continuation resumed twice"),
        };
        self.given.set(Some(v));
        body.await
    }
}

/// How a prompt's body ended: its value, or an abort to the prompt's tag
/// with a payload (from `abort-with-continuation`, or a plain abort).
pub enum Prompted<R, H> {
    Done(R),
    Aborted(H),
}

/// `(prompt tag body handler)`, where `body` may capture: run `body`, and
/// say how it ended; the caller runs the handler on an abort. A capture to
/// another tag, and an abort to one, go on outward.
pub fn prompt<R: 'static, H: Copy + 'static>(
    tag: Tag<R, H>,
    body: Body<R>,
) -> impl Future<Output = Result<Prompted<R, H>, Unwind>> {
    PromptFuture {
        tag,
        body: Some(body),
    }
}

/// `prompt`, of a body that is not `'static` (C434: a module maker's item's,
/// which holds its place and its state): an abort to the tag ends it, as
/// `prompt`'s does; a capture, which would keep the body past the place it
/// holds, is refused, said where it is pending.
pub fn prompt_at<'a, R: 'a, H: Copy + 'static>(
    tag: Tag<R, H>,
    body: BodyAt<'a, R>,
) -> impl Future<Output = Result<Prompted<R, H>, Unwind>> + 'a {
    PromptAtFuture { tag, body }
}

struct PromptAtFuture<'a, R: 'a, H: 'static> {
    tag: Tag<R, H>,
    body: BodyAt<'a, R>,
}

impl<'a, R: 'a, H: Copy + 'static> Future for PromptAtFuture<'a, R, H> {
    type Output = Result<Prompted<R, H>, Unwind>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let tag = self.tag;
        match self.body.as_mut().poll(cx) {
            Poll::Ready(Ok(v)) => Poll::Ready(Ok(Prompted::Done(v))),
            Poll::Ready(Err(u)) if u.is(tag) => Poll::Ready(Ok(Prompted::Aborted(tag.payload(u)))),
            Poll::Ready(Err(u)) => Poll::Ready(Err(u)),
            Poll::Pending => {
                let mine = CAPTURE.with(|c| match &*c.borrow() {
                    Some(p) => p.tag == tag.id(),
                    None => false,
                });
                if mine {
                    crate::fail(
                        "a continuation captured under a prompt whose body holds a module maker's place: not yet",
                    )
                }
                Poll::Pending
            }
        }
    }
}

struct PromptFuture<R: 'static, H: 'static> {
    tag: Tag<R, H>,
    body: Option<Body<R>>,
}

impl<R: 'static, H: Copy + 'static> Future for PromptFuture<R, H> {
    type Output = Result<Prompted<R, H>, Unwind>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let tag = self.tag;
        let Some(body) = self.body.as_mut() else {
            crate::fail("prompt: polled after it ended")
        };
        match body.as_mut().poll(cx) {
            Poll::Ready(Ok(v)) => Poll::Ready(Ok(Prompted::Done(v))),
            Poll::Ready(Err(u)) if u.is(tag) => Poll::Ready(Ok(Prompted::Aborted(tag.payload(u)))),
            Poll::Ready(Err(u)) => Poll::Ready(Err(u)),
            Poll::Pending => {
                let mine = CAPTURE.with(|c| match &*c.borrow() {
                    Some(p) => p.tag == tag.id(),
                    None => false,
                });
                if !mine {
                    return Poll::Pending;
                }
                let Some(p) = CAPTURE.with(|c| c.borrow_mut().take()) else {
                    crate::fail("prompt: the capture went missing")
                };
                let Ok(make) = p.make.downcast::<Make<R, H>>() else {
                    crate::fail("prompt: a capture not of its tag's types")
                };
                let Some(body) = self.body.take() else {
                    crate::fail("prompt: polled after it ended")
                };
                Poll::Ready(Ok(Prompted::Aborted(make(body))))
            }
        }
    }
}

/// A computation that may capture, where nothing around it can suspend
/// (the top level; a prompt that delimits every capture its body makes, in
/// a procedure that cannot suspend): run to its end. Nothing waits on the
/// outside world yet, so one poll does: a capture still pending has no
/// prompt for its tag, an error, as in FX-26.
///
/// Fuel charged in it is never settled with a scheduler, as it cannot
/// yield: it is owed, and paid at the next charge that may (C378).
pub fn block<R>(f: impl Future<Output = R>) -> R {
    struct Blocking;
    impl Drop for Blocking {
        fn drop(&mut self) {
            BLOCKING.with(|b| b.set(b.get() - 1));
        }
    }
    BLOCKING.with(|b| b.set(b.get() + 1));
    let _blocking = Blocking;
    let mut f = std::pin::pin!(f);
    let mut cx = Context::from_waker(Waker::noop());
    let _ = crate::async_process::wait_pending();
    match f.as_mut().poll(&mut cx) {
        Poll::Ready(r) => r,
        // A wait (C388): the checker refuses one where nothing may wait (a
        // constant's value, a native procedure's callback, a monitor's
        // `on-failure`), so this is a program it did not see so.
        Poll::Pending if crate::async_process::wait_pending() => crate::fail(
            "a wait where nothing may wait (a constant's value, a callback of native code, \
             a monitor's on-failure): it could only be waited for by another task",
        ),
        Poll::Pending => crate::fail("abort-with-continuation: no prompt for the tag"),
    }
}

/// How much fuel a thread spends before it settles with its scheduler
/// (C378): a charge is a decrement of a thread local, and only one in
/// `QUANTUM` asks the scheduler, which may end the task's slice there.
pub const QUANTUM: u64 = 1024;

thread_local! {
    /// What is left of this thread's quantum.
    static FUEL: Cell<u64> = const { Cell::new(QUANTUM) };
    /// Quanta spent where no scheduler could be paid (in `block`, or with
    /// none), to pay at the next settling.
    static OWED: Cell<u64> = const { Cell::new(0) };
    /// How many `block`s this thread is in.
    static BLOCKING: Cell<u32> = const { Cell::new(0) };
    /// The scheduler's handle of the task this thread runs (`run`).
    static TASK: RefCell<Option<kayak_sched::Cx>> = const { RefCell::new(None) };
}

/// Fuel, one unit (C378): what the async mode's procedures pay as they
/// begin, and each turn of a loop, so that a program, however long it
/// runs, reaches a point where its scheduler may preempt it. Ready at once
/// but at the end of a quantum, when it settles with the scheduler of
/// the task it runs in (`run`), which may end the task's slice: then
/// `Pending`, and polled again in the next.
#[inline]
pub fn charge() -> Charge {
    Charge { settling: None }
}

/// `charge`'s first half, as the emitter writes it: one unit spent, and
/// whether the quantum is, so that its end is settled (`settle().await`);
/// a decrement, where an awaited `charge` is a future polled each time.
#[inline]
pub fn spent() -> bool {
    let left = FUEL.with(Cell::get);
    if left > 0 {
        FUEL.with(|f| f.set(left - 1));
        false
    } else {
        true
    }
}

/// `charge`'s second half: a spent quantum settled with the scheduler.
#[cold]
pub fn settle() -> Charge {
    Charge { settling: None }
}

pub struct Charge {
    settling: Option<kayak_sched::Charge>,
}

impl Future for Charge {
    type Output = ();

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let this = self.get_mut();
        if this.settling.is_none() {
            let left = FUEL.with(Cell::get);
            if left > 0 {
                FUEL.with(|f| f.set(left - 1));
                return Poll::Ready(());
            }
        }
        this.settle(cx)
    }
}

impl Charge {
    #[cold]
    fn settle(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if self.settling.is_none() {
            let task = if BLOCKING.with(Cell::get) == 0 {
                TASK.with(|t| t.borrow().clone())
            } else {
                None
            };
            let Some(task) = task else {
                OWED.with(|o| o.set(o.get() + QUANTUM));
                FUEL.with(|f| f.set(QUANTUM));
                return Poll::Ready(());
            };
            self.settling = Some(task.charge(QUANTUM + OWED.with(|o| o.replace(0))));
        }
        let Some(settling) = self.settling.as_mut() else {
            return Poll::Ready(());
        };
        match Pin::new(settling).poll(cx) {
            Poll::Ready(()) => {
                self.settling = None;
                FUEL.with(|f| f.set(QUANTUM));
                Poll::Ready(())
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// A program's `main`, a future, run to its end as a task of a scheduler
/// of its own (kayak-sched, Coopt's, C368), which its fuel preempts (C378):
/// the async mode's top level, where the synchronous mode's is a call.
/// Still pending when nothing can run, a capture with no prompt for its
/// tag, an error, as `block` says it.
///
/// The program's processes, in the async mode, are tasks of the same
/// scheduler (`async_process`, C380): the run ends when `main` does, as the
/// program's threads end with it in the synchronous mode; a process's
/// failure is the program's; and when no task can run and none waits for
/// a deadline or the standard input, `main`'s wait is a deadlock.
pub fn run<T: 'static>(f: impl Future<Output = T> + 'static) -> T {
    let mut sched =
        kayak_sched::Scheduler::new(kayak_sched::Config::default(), kayak_trace::Discard).quiet();
    let f = crate::async_process::program(f);
    let mut main = sched.spawn_join(kayak_sched::Spawn::new("main"), |cx| async move {
        struct Running;
        impl Drop for Running {
            fn drop(&mut self) {
                TASK.with(|t| *t.borrow_mut() = None);
            }
        }
        TASK.with(|t| *t.borrow_mut() = Some(cx));
        let _running = Running;
        f.await
    });
    // Its I/O the runner's (C381), whose writes are all written at its
    // end, however it ends.
    struct Installed;
    impl Drop for Installed {
        fn drop(&mut self) {
            crate::async_io::uninstall();
        }
    }
    let _installed = match kayak_runner::live::Live::cooked() {
        Ok(live) => {
            crate::async_io::install(live, sched.inbox());
            Some(Installed)
        }
        Err(_) => None,
    };
    while !main.is_finished() {
        crate::async_process::check_failed();
        crate::async_io::poll();
        if sched.step() == kayak_sched::Step::Idle && !crate::async_process::waited() {
            break;
        }
    }
    crate::async_process::check_failed();
    match main.try_take() {
        Some(v) => v,
        None => crate::fail(crate::async_process::stuck()),
    }
}

/// The scheduler's handle of the task this thread runs, if it runs one
/// (`run`): what a process is spawned by (C380).
pub(crate) fn task_cx() -> Option<kayak_sched::Cx> {
    TASK.with(|t| t.borrow().clone())
}

/// A synchronous procedure, a native one (C116), as the emitter's async
/// mode calls procedures (C376): each answer a future ready at once, a
/// `Body` if `readyN_body`'s.
macro_rules! ready {
    ($name:ident, $body:ident, $($a:ident $x:ident),*) => {
        pub fn $name<'a, $($a: 'a,)* R: 'a>(
            f: &'a (dyn Fn($($a),*) -> R + Send + Sync),
        ) -> &'a (dyn Fn($($a),*) -> FutAt<'a, R> + Send + Sync) {
            crate::leak(move |$($x: $a),*| boxed(std::future::ready(f($($x),*))))
        }

        pub fn $body<'a, $($a: 'a,)* R: 'a>(
            f: &'a (dyn Fn($($a),*) -> R + Send + Sync),
        ) -> &'a (dyn Fn($($a),*) -> BodyAt<'a, R> + Send + Sync) {
            crate::leak(move |$($x: $a),*| body(std::future::ready(Ok(f($($x),*)))))
        }
    };
}

ready!(ready0, ready0_body,);
ready!(ready1, ready1_body, A a);
ready!(ready2, ready2_body, A a, B b);
ready!(ready3, ready3_body, A a, B b, C c);
ready!(ready4, ready4_body, A a, B b, C c, D d);

/// The other way, a procedure of the async mode where the runtime wants
/// one that answers at once (`spawn-monitored`'s `on-failure`, C376): each
/// call run to its end, as `block` runs one.
pub fn blocking1<A, R>(
    f: &'static (dyn Fn(A) -> Fut<R> + Send + Sync),
) -> impl Fn(A) -> R + Send + Sync + 'static {
    move |a| block(f(a))
}

#[cfg(test)]
mod tests {
    //! Fixpt's `docs/research/examples/async/ping-pong.fx` and `promise.fx`,
    //! written by hand as the emitter is to write them.
    use super::*;
    use crate::List;
    use crate::escape::make_continuation_prompt_tag;

    fn reverse<T: Copy + 'static>(xs: List<T>) -> List<T> {
        let mut acc = None;
        let mut xs = xs;
        while let Some(c) = xs {
            acc = crate::cons(c.car, acc);
            xs = c.cdr;
        }
        acc
    }

    fn list_of(xs: List<i64>) -> Vec<i64> {
        let mut v = vec![];
        let mut xs = xs;
        while let Some(c) = xs {
            v.push(c.car);
            xs = c.cdr;
        }
        v
    }

    // (define-type task (composable unit unit D @p))
    type Task = Resume<(), ()>;

    struct PingPong {
        sched: Tag<(), Task>,
        queue: Cell<List<Task>>,
        log: Cell<List<i64>>,
    }

    // (define* yield! ...) : the capture, its payload `k` itself.
    async fn yield_(g: &'static PingPong) -> Result<(), Unwind> {
        abort_with_continuation(g.sched, |k| k).await
    }

    fn enqueue(g: &'static PingPong, k: Task) {
        let back = reverse(g.queue.get());
        g.queue.set(reverse(crate::cons(k, back)));
    }

    async fn spawn(g: &'static PingPong, thunk: Body<()>) -> Result<(), Unwind> {
        let body: Body<()> = Box::pin(async move {
            yield_(g).await?;
            thunk.await
        });
        match prompt(g.sched, body).await? {
            Prompted::Done(()) => Ok(()),
            Prompted::Aborted(k) => {
                enqueue(g, k);
                Ok(())
            }
        }
    }

    async fn run_(g: &'static PingPong) -> Result<(), Unwind> {
        // A tail call, a loop.
        loop {
            let Some(q) = g.queue.get() else {
                return Ok(());
            };
            g.queue.set(q.cdr);
            let k = q.car;
            let body: Body<()> = k(());
            if let Prompted::Aborted(k) = prompt(g.sched, body).await? {
                enqueue(g, k);
            }
        }
    }

    async fn pinger(g: &'static PingPong, who: i64, n: i64) -> Result<(), Unwind> {
        let mut n = n;
        loop {
            if n == 0 {
                return Ok(());
            }
            g.log.set(crate::cons(who + n, g.log.get()));
            yield_(g).await?;
            n -= 1;
        }
    }

    #[test]
    fn ping_pong() {
        let g: &'static PingPong = crate::leak(PingPong {
            sched: make_continuation_prompt_tag(),
            queue: Cell::new(None),
            log: Cell::new(None),
        });
        let main: Body<List<i64>> = Box::pin(async move {
            spawn(g, Box::pin(pinger(g, 100, 3))).await?;
            spawn(g, Box::pin(pinger(g, 200, 3))).await?;
            run_(g).await?;
            Ok(reverse(g.log.get()))
        });
        let Ok(log) = block(main) else {
            panic!("aborted")
        };
        assert_eq!(list_of(log), vec![103, 203, 102, 202, 101, 201]);
    }

    // promise.fx: the abort's payload a thunk that files `k` away.
    type File = &'static dyn Fn();
    #[derive(Clone, Copy)]
    enum Promise {
        Pending(List<Task>),
        Resolved(i64),
    }

    struct Promises {
        sched: Tag<(), File>,
        queue: Cell<List<Task>>,
        log: Cell<List<i64>>,
        fut: Cell<Promise>,
    }

    fn enqueue2(g: &'static Promises, k: Task) {
        let back = reverse(g.queue.get());
        g.queue.set(reverse(crate::cons(k, back)));
    }

    async fn suspend(g: &'static Promises, file: &'static dyn Fn(Task)) -> Result<(), Unwind> {
        abort_with_continuation(g.sched, move |k: Task| -> File {
            crate::leak(move || file(k))
        })
        .await
    }

    async fn yield2(g: &'static Promises) -> Result<(), Unwind> {
        suspend(g, crate::leak(move |k| enqueue2(g, k))).await
    }

    async fn await_(g: &'static Promises) -> Result<i64, Unwind> {
        loop {
            match g.fut.get() {
                Promise::Resolved(v) => return Ok(v),
                Promise::Pending(ws) => {
                    suspend(
                        g,
                        crate::leak(move |k| g.fut.set(Promise::Pending(crate::cons(k, ws)))),
                    )
                    .await?
                }
            }
        }
    }

    fn resolve(g: &'static Promises, v: i64) {
        if let Promise::Pending(ws) = g.fut.get() {
            g.fut.set(Promise::Resolved(v));
            let mut ks = reverse(ws);
            while let Some(c) = ks {
                enqueue2(g, c.car);
                ks = c.cdr;
            }
        }
    }

    fn note(g: &'static Promises, x: i64) {
        g.log.set(crate::cons(x, g.log.get()));
    }

    async fn spawn2(g: &'static Promises, thunk: Body<()>) -> Result<(), Unwind> {
        let body: Body<()> = Box::pin(async move {
            yield2(g).await?;
            thunk.await
        });
        if let Prompted::Aborted(file) = prompt(g.sched, body).await? {
            file();
        }
        Ok(())
    }

    async fn run2(g: &'static Promises) -> Result<(), Unwind> {
        loop {
            let Some(q) = g.queue.get() else {
                return Ok(());
            };
            g.queue.set(q.cdr);
            let body: Body<()> = (q.car)(());
            if let Prompted::Aborted(file) = prompt(g.sched, body).await? {
                file();
            }
        }
    }

    #[test]
    fn promise() {
        let g: &'static Promises = crate::leak(Promises {
            sched: make_continuation_prompt_tag(),
            queue: Cell::new(None),
            log: Cell::new(None),
            fut: Cell::new(Promise::Pending(None)),
        });
        let main: Body<List<i64>> = Box::pin(async move {
            spawn2(
                g,
                Box::pin(async move {
                    note(g, 1);
                    let v = await_(g).await?;
                    note(g, v);
                    Ok(())
                }),
            )
            .await?;
            spawn2(
                g,
                Box::pin(async move {
                    note(g, 2);
                    yield2(g).await?;
                    note(g, 3);
                    resolve(g, 42);
                    Ok(())
                }),
            )
            .await?;
            run2(g).await?;
            Ok(reverse(g.log.get()))
        });
        let Ok(log) = block(main) else {
            panic!("aborted")
        };
        assert_eq!(list_of(log), vec![1, 2, 3, 42]);
    }

    #[test]
    #[should_panic(expected = "resumed twice")]
    fn twice() {
        let tag: Tag<i64, Resume<i64, i64>> = make_continuation_prompt_tag();
        let main: Body<i64> = Box::pin(async move {
            let inner: Body<i64> = Box::pin(async move {
                let v: i64 = abort_with_continuation(tag, |k| k).await?;
                Ok(v + 1)
            });
            match prompt(tag, inner).await? {
                Prompted::Done(v) => Ok(v),
                Prompted::Aborted(k) => {
                    let a = k(10).await?;
                    let b = k(20).await?;
                    Ok(a + b)
                }
            }
        });
        let _ = block(main);
    }

    /// A capture to an outer tag from inside an inner prompt takes the
    /// inner prompt with it: resumed, an abort to the inner tag finds it.
    #[test]
    fn inner_prompt_kept() {
        let outer: Tag<i64, Resume<(), i64>> = make_continuation_prompt_tag();
        let inner: Tag<i64, i64> = make_continuation_prompt_tag();
        let main: Body<i64> = Box::pin(async move {
            let body: Body<i64> = Box::pin(async move {
                let b: Body<i64> = Box::pin(async move {
                    abort_with_continuation(outer, |k| k).await?;
                    Err(crate::unwind(inner, 7))
                });
                match prompt(inner, b).await? {
                    Prompted::Done(v) => Ok(v),
                    Prompted::Aborted(v) => Ok(v * 100),
                }
            });
            match prompt(outer, body).await? {
                Prompted::Done(v) => Ok(v),
                Prompted::Aborted(k) => Ok(k(()).await? + 1),
            }
        });
        let Ok(v) = block(main) else {
            panic!("aborted")
        };
        assert_eq!(v, 701);
    }

    /// Fuel preempts (C378): a task that charges for long is ended at a
    /// quantum, so another (one its open section allows) runs before it
    /// ends; and in a `block`, what it charges is only owed.
    #[test]
    fn fuel_preempts() {
        use std::rc::Rc;
        let order: Rc<RefCell<Vec<&str>>> = Rc::default();
        let mut sched =
            kayak_sched::Scheduler::new(kayak_sched::Config::default(), kayak_trace::Discard);
        let (a, b) = (order.clone(), order.clone());
        sched.spawn(kayak_sched::Spawn::new("long"), move |cx| async move {
            TASK.with(|t| *t.borrow_mut() = Some(cx));
            for _ in 0..100 * QUANTUM {
                charge().await;
            }
            block(async {
                for _ in 0..10 * QUANTUM {
                    charge().await;
                }
            });
            a.borrow_mut().push("long");
            TASK.with(|t| *t.borrow_mut() = None);
        });
        // Pure, so that the long one's open section does not exclude it.
        let short = kayak_sched::Spawn::new("short").effect(kayak_sched::Effect::pure());
        sched.spawn(short, move |_| async move {
            b.borrow_mut().push("short");
        });
        let slices = sched.run(u64::MAX);
        assert_eq!(*order.borrow(), ["short", "long"]);
        assert!(slices > 2, "{slices} slices");
    }

    /// A wait `block` cannot run to its end is said to be one (C388), not
    /// a capture with no prompt.
    #[test]
    #[should_panic(expected = "a wait where nothing may wait")]
    fn a_wait_in_block_is_said_so() {
        let mailbox: crate::process::Inbox<i64> = crate::leak(crate::process::MailboxBox {
            queue: std::sync::Mutex::new(std::collections::VecDeque::new()),
        });
        block(crate::async_process::receive(mailbox));
    }
}
