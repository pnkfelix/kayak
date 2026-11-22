//! The heap's data, made by bumping (C43): since C507, in the heap's space
//! (`obj/space.rs`), every object of every type in the thread's page, of
//! Fixpt's layout or a pair, where each type had an arena of its own,
//! found by its `TypeId` at each allocation. Kept for the run (the heap is
//! never freed yet). The emitter makes heap data so only where every type
//! is `'static`: no `letrena`, no place, no region binder around (else
//! `alloc`, a `Box` each).

use crate::{Cons, List, Ref};

/// `v`, in the heap, by bumping.
#[inline]
pub fn heap_alloc<T: 'static + crate::Visit>(v: T) -> crate::Handle<'static, T> {
    crate::stats::note::<T>(std::mem::size_of::<T>());
    #[cfg(feature = "heap-stats")]
    {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNT: AtomicU64 = AtomicU64::new(0);
        static EVERY: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
        let every = *EVERY.get_or_init(|| {
            std::env::var("KAYAK_HEAP_MARK")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0)
        });
        let n = COUNT.fetch_add(1, Ordering::Relaxed) + 1;
        if every > 0 && n.is_multiple_of(every) {
            crate::census::mark(n);
        }
        let r = crate::obj::space::put(v);
        crate::census::kept(r);
        crate::Handle::made(r)
    }
    #[cfg(not(feature = "heap-stats"))]
    crate::Handle::made(crate::obj::space::put(v))
}

/// `cons`, in the heap's arena of pairs.
#[inline]
pub fn heap_cons<T: 'static + crate::Trace>(car: T, cdr: List<T>) -> List<T> {
    crate::ListAt::of_pair(heap_alloc(Cons::new(car, cdr)))
}

/// `new`, in the heap's arena of cells.
#[inline]
pub fn heap_new<T: Copy + 'static + crate::Trace>(v: T) -> Ref<T> {
    crate::RefAt::of_obj(heap_alloc(crate::RefObj::new(v)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn types_apart() {
        let xs = heap_cons(1i64, heap_cons(2i64, crate::nil()));
        let r = heap_new(7i64);
        let s = heap_new(true);
        let ys = heap_cons(crate::str_lit("a"), crate::nil());
        r.set(8);
        assert_eq!(
            xs.pair().unwrap().car() + xs.pair().unwrap().cdr.pair().unwrap().car(),
            3
        );
        assert_eq!(
            (r.get(), s.get(), ys.pair().unwrap().car().as_str()),
            (8, true, "a")
        );
        let many: Vec<Ref<i64>> = (0..1000).map(heap_new).collect();
        assert!(many.iter().enumerate().all(|(i, v)| v.get() == i as i64));
    }
}
