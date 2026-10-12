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
pub type Body<R> = Pin<Box<dyn Future<Output = Result<R, Unwind>>>>;

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
type Make<R, H> = Box<dyn FnOnce(Body<R>) -> Result<H, Unwind>>;

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
    f: impl FnOnce(&'static Cont<V, R>) -> Result<H, Unwind> + 'static,
) -> impl Future<Output = Result<V, Unwind>>
where
    V: 'static,
    R: 'static,
    H: 'static,
{
    let given: &'static Cell<Option<V>> = crate::alloc(Cell::new(None));
    let make: Make<R, H> = Box::new(move |body| {
        f(crate::alloc(Cont {
            body: RefCell::new(Some(body)),
            given,
        }))
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
                Poll::Ready(make(body).map(Prompted::Aborted))
            }
        }
    }
}

/// The top level of a program that may capture: run to its end. A capture
/// with no prompt for its tag is an error, as in FX-26.
pub fn run<R: 'static>(body: Body<R>) -> Result<R, Unwind> {
    let mut body = body;
    let mut cx = Context::from_waker(Waker::noop());
    match body.as_mut().poll(&mut cx) {
        Poll::Ready(r) => r,
        Poll::Pending => crate::fail("call-with-composable-continuation: no prompt for the tag"),
    }
}

#[cfg(test)]
mod tests {
    //! Fixpt's `docs/research/examples/async/ping-pong.fx` and `promise.fx`,
    //! written by hand as the emitter is to write them.
    use super::*;
    use crate::escape::make_continuation_prompt_tag;
    use crate::{Cons, List};

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
        while let Some(Cons { car, cdr }) = xs {
            v.push(*car);
            xs = *cdr;
        }
        v
    }

    // (define-type task (composable unit unit D @p))
    type Task = &'static Cont<(), ()>;

    struct PingPong {
        sched: Tag<(), Task>,
        queue: Cell<List<Task>>,
        log: Cell<List<i64>>,
    }

    // (define* yield! ...) : the capture, its payload `k` itself.
    async fn yield_(g: &'static PingPong) -> Result<(), Unwind> {
        abort_with_continuation(g.sched, Ok).await
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
            let body: Body<()> = Box::pin(k.resume(()));
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
        let g: &'static PingPong = crate::alloc(PingPong {
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
        let Ok(log) = run(main) else {
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
        abort_with_continuation(g.sched, move |k: Task| -> Result<File, Unwind> {
            Ok(crate::alloc(move || file(k)) as File)
        })
        .await
    }

    async fn yield2(g: &'static Promises) -> Result<(), Unwind> {
        suspend(g, crate::alloc(move |k| enqueue2(g, k))).await
    }

    async fn await_(g: &'static Promises) -> Result<i64, Unwind> {
        loop {
            match g.fut.get() {
                Promise::Resolved(v) => return Ok(v),
                Promise::Pending(ws) => {
                    suspend(
                        g,
                        crate::alloc(move |k| g.fut.set(Promise::Pending(crate::cons(k, ws)))),
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
            let body: Body<()> = Box::pin(q.car.resume(()));
            if let Prompted::Aborted(file) = prompt(g.sched, body).await? {
                file();
            }
        }
    }

    #[test]
    fn promise() {
        let g: &'static Promises = crate::alloc(Promises {
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
        let Ok(log) = run(main) else {
            panic!("aborted")
        };
        assert_eq!(list_of(log), vec![1, 2, 3, 42]);
    }

    #[test]
    #[should_panic(expected = "resumed twice")]
    fn twice() {
        let tag: Tag<i64, &'static Cont<i64, i64>> = make_continuation_prompt_tag();
        let main: Body<i64> = Box::pin(async move {
            let inner: Body<i64> = Box::pin(async move {
                let v: i64 = abort_with_continuation(tag, Ok).await?;
                Ok(v + 1)
            });
            match prompt(tag, inner).await? {
                Prompted::Done(v) => Ok(v),
                Prompted::Aborted(k) => {
                    let a = k.resume(10).await?;
                    let b = k.resume(20).await?;
                    Ok(a + b)
                }
            }
        });
        let _ = run(main);
    }

    /// A capture to an outer tag from inside an inner prompt takes the
    /// inner prompt with it: resumed, an abort to the inner tag finds it.
    #[test]
    fn inner_prompt_kept() {
        let outer: Tag<i64, &'static Cont<(), i64>> = make_continuation_prompt_tag();
        let inner: Tag<i64, i64> = make_continuation_prompt_tag();
        let main: Body<i64> = Box::pin(async move {
            let body: Body<i64> = Box::pin(async move {
                let b: Body<i64> = Box::pin(async move {
                    abort_with_continuation(outer, Ok).await?;
                    Err(crate::unwind(inner, 7))
                });
                match prompt(inner, b).await? {
                    Prompted::Done(v) => Ok(v),
                    Prompted::Aborted(v) => Ok(v * 100),
                }
            });
            match prompt(outer, body).await? {
                Prompted::Done(v) => Ok(v),
                Prompted::Aborted(k) => Ok(k.resume(()).await? + 1),
            }
        });
        let Ok(v) = run(main) else { panic!("aborted") };
        assert_eq!(v, 701);
    }
}
