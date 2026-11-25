//! The heap's space, the shared heap's since C539 (`kayak_heap::space`,
//! which was this file): kayak-rt's roots given the shared collector
//! before kayak-rt's first allocation (`obj::collect`'s `Rt`, C540), so
//! that a collection the space's policy finds due can run. Each way into the space's `refill` (where
//! a collection may start) passes `ready` first; the rest is the space's.

pub(crate) use kayak_heap::space::*;

static READY: std::sync::Once = std::sync::Once::new();

/// kayak-rt's collector the space's, once.
#[inline(always)]
fn ready() {
    if !READY.is_completed() {
        READY.call_once(|| kayak_heap::collect::set_host(&super::collect::Rt));
    }
}

#[inline]
pub(crate) fn words(w: usize) -> *mut u64 {
    ready();
    kayak_heap::space::words(w)
}

#[inline]
pub(crate) fn words_slice(w: usize) -> &'static mut [u64] {
    ready();
    kayak_heap::space::words_slice(w)
}

#[inline]
pub(crate) fn put<'a, T: 'a>(v: T) -> &'a T {
    ready();
    kayak_heap::space::put(v)
}

pub(crate) fn lend(w: usize) -> (usize, usize, bool) {
    ready();
    kayak_heap::space::lend(w)
}
