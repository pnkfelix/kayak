//! FX-26's continuation marks (`CLAUDES_CHOICES.md` C72, C106):
//! `make-continuation-mark-key`, `with-mark`, `first-mark`. The marks of
//! the running computation are a stack, the thread's: `with-mark` pushes
//! one for its body's run and takes it off after, however the body ends
//! (a value, an abort's early return, a failure). A body that may suspend
//! is a future, and its mark is pushed for each poll of it, on whatever
//! marks the poller has: so a continuation resumed elsewhere sees its own
//! marks on the resumer's, as FX-26 composes them. A mark's value is a
//! KFX value, `Copy`, kept in the heap, as every value is.
//!
//! What FX-26 says of a `with-mark` in tail position (its mark replaces
//! the frame's of the same key) no `first-mark` can tell from a push;
//! `current-marks` could, and KFX has it not yet.

use std::any::Any;
use std::cell::RefCell;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::task::{Context, Poll};

/// `(mark-key T R)`: a key, compared by identity; its region the
/// checker's, erased here.
pub struct MarkKey<T> {
    id: u64,
    kinds: PhantomData<fn(T) -> T>,
}

impl<T> Clone for MarkKey<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for MarkKey<T> {}

static KEYS: AtomicU64 = AtomicU64::new(0);

/// The marks, the oldest first: each its key's identity and its value.
pub type Marks = Vec<(u64, &'static dyn Any)>;

thread_local! {
    static MARKS: RefCell<Marks> = const { RefCell::new(Vec::new()) };
}

pub fn make_continuation_mark_key<T>() -> MarkKey<T> {
    MarkKey {
        id: KEYS.fetch_add(1, Ordering::Relaxed),
        kinds: PhantomData,
    }
}

/// The marks pushed while this lives, taken off when it is dropped.
struct Pushed(usize);

impl Pushed {
    fn new(key: u64, value: &'static dyn Any) -> Pushed {
        MARKS.with(|m| {
            let mut m = m.borrow_mut();
            m.push((key, value));
            Pushed(m.len() - 1)
        })
    }
}

impl Drop for Pushed {
    fn drop(&mut self) {
        MARKS.with(|m| m.borrow_mut().truncate(self.0));
    }
}

/// `(with-mark key v body)`: `body` run with `v` the newest mark of `key`.
pub fn with_mark<T: 'static, A>(key: MarkKey<T>, v: T, body: impl FnOnce() -> A) -> A {
    let _pushed = Pushed::new(key.id, crate::alloc(v));
    body()
}

/// A body that may suspend, its mark pushed for each poll.
struct Marked<F> {
    key: u64,
    value: &'static dyn Any,
    body: Pin<Box<F>>,
}

impl<F: Future> Future for Marked<F> {
    type Output = F::Output;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<F::Output> {
        let _pushed = Pushed::new(self.key, self.value);
        self.body.as_mut().poll(cx)
    }
}

/// `with-mark` of a body that may suspend: its future, marked.
pub fn with_mark_async<T: 'static, A: 'static>(
    key: MarkKey<T>,
    v: T,
    body: impl FnOnce() -> crate::suspend::Body<A>,
) -> crate::suspend::Body<A> {
    Box::pin(Marked {
        key: key.id,
        value: crate::alloc(v),
        body: Box::pin(body()),
    })
}

/// `(first-mark key default)`: the newest mark of `key`, or `default`.
pub fn first_mark<T: Copy + 'static>(key: MarkKey<T>, default: T) -> T {
    MARKS.with(|m| {
        m.borrow()
            .iter()
            .rev()
            .find(|(k, _)| *k == key.id)
            .map(|(_, v)| match v.downcast_ref::<T>() {
                Some(t) => *t,
                None => crate::fail("first-mark: a mark not of its key's type"),
            })
            .unwrap_or(default)
    })
}

/// This thread's marks, for a `par` branch run on another (`unsafe-par`).
pub fn snapshot() -> Marks {
    MARKS.with(|m| m.borrow().clone())
}

/// A thread's marks, its `par` branch's maker's.
pub fn install(marks: Marks) {
    MARKS.with(|m| *m.borrow_mut() = marks);
}
