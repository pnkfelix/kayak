//! The heap's space, the shared heap's since C539 (`kayak_heap::space`,
//! which was this file): kayak-rt's collector given it before kayak-rt's
//! first allocation, so that a collection its policy finds due is
//! kayak-rt's (`obj::collect`). Each way into the space's `refill` (where
//! a collection may start) passes `ready` first; the rest is the space's.

pub(crate) use kayak_heap::space::*;

static READY: std::sync::Once = std::sync::Once::new();

/// kayak-rt's collector the space's, once.
#[inline(always)]
fn ready() {
    if !READY.is_completed() {
        READY.call_once(|| {
            kayak_heap::space::set_collector(kayak_heap::space::Collector {
                major: super::collect::collect,
                minor: super::collect::collect_minor,
            })
        });
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
