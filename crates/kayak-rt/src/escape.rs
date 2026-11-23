//! Escapes: FX-26's prompt tags, prompts and aborts, as `Result`
//! (`docs/research/escapes.md`, `CLAUDES_CHOICES.md` C18). A procedure
//! whose effect may abort returns `Result<T, Unwind>`; an abort is
//! `return Err(unwind(tag, v))`; a prompt matches on its body's `Result`.

use std::any::Any;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU64, Ordering};

/// `(prompt-tag A H D R)`: its prompts deliver an `A`, its aborts carry an
/// `H`. Its effect bound and region are the checker's, erased here.
pub struct Tag<A, H> {
    id: u64,
    kinds: PhantomData<fn(A, H)>,
}

/// A tag: its number alone, holding nothing of the heap (C480).
impl<A, H> crate::Trace for Tag<A, H> {
    fn trace<'a>(self, _: &mut crate::Tracer<'a>)
    where
        Self: 'a,
    {
    }
    // Its number, held as a Fixpt word (C496).
    type Stored = crate::HeapWord;
    fn store(self) -> crate::HeapWord {
        crate::HeapWord::of_u64(self.id)
    }
    fn load(w: crate::HeapWord) -> Self {
        Tag {
            id: w.to_u64(),
            kinds: PhantomData,
        }
    }
}

impl<A, H> Clone for Tag<A, H> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<A, H> Copy for Tag<A, H> {}

/// An abort on its way to its prompt: the tag's identity, and the payload,
/// erased, so that aborts to tags of different payloads share a `Result`.
pub struct Unwind {
    tag: u64,
    payload: &'static dyn Any,
}

/// Tags are numbered in the order they are made: deterministic.
static TAGS: AtomicU64 = AtomicU64::new(0);

pub fn make_continuation_prompt_tag<A, H>() -> Tag<A, H> {
    Tag {
        id: TAGS.fetch_add(1, Ordering::Relaxed),
        kinds: PhantomData,
    }
}

/// `(make-escape-tag)` (C105): an escape-only tag, at run time a prompt
/// tag; that nothing captures with one is the checker's.
pub fn make_escape_tag<A, H>() -> Tag<A, H> {
    make_continuation_prompt_tag()
}

pub fn unwind<A, H: 'static>(tag: Tag<A, H>, payload: H) -> Unwind {
    Unwind {
        tag: tag.id,
        payload: crate::leak_collected(payload),
    }
}

impl<A, H> Tag<A, H> {
    /// The tag's identity, which a capture carries to its prompt.
    pub(crate) fn id(self) -> u64 {
        self.id
    }
}

impl Unwind {
    /// Whether this abort is to `tag`.
    pub fn is<A, H>(&self, tag: Tag<A, H>) -> bool {
        self.tag == tag.id
    }
}

impl<A, H: Copy + 'static> Tag<A, H> {
    /// An abort's payload, which the checker has seen is an `H`.
    pub fn payload(self, u: Unwind) -> H {
        match u.payload.downcast_ref::<H>() {
            Some(h) => *h,
            None => crate::fail("prompt: a payload not of its tag's type"),
        }
    }
}

/// An abort that no prompt delimits: to a tag whose prompt has returned,
/// an error in FX-26 too.
pub fn unreachable_unwind(_: Unwind) -> ! {
    crate::fail("abort-current-continuation: no prompt for the tag")
}

/// `(the T e)`: `e`, of type `T`, said to rustc.
pub fn the<T>(x: T) -> T {
    x
}
