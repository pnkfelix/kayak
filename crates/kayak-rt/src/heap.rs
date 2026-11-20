//! The heap's data, made by bumping (C43): an [`Arena`] for each type, kept
//! for the run (the heap is never freed), found by the type's `TypeId` in
//! a table of this thread's. In safe Rust, as `letrena`'s arenas are; the
//! price, a type's arena found at each allocation, and a chunk each type
//! has partly empty. A `TypeId` is only a `'static` type's, so the emitter
//! makes heap data so only where every type is `'static`: no `letrena`, no
//! place, no region binder around (else `alloc`, a `Box` each).

use crate::{Arena, Cons, List, Ref};
use std::any::{Any, TypeId};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

/// A `TypeId`'s hash: its own bits, which are a hash already.
#[derive(Default)]
struct IdHasher(u64);

impl Hasher for IdHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.0 = self.0.rotate_left(8) ^ u64::from(*b);
        }
    }
    fn write_u64(&mut self, n: u64) {
        self.0 ^= n;
    }
}

type Table = HashMap<TypeId, &'static dyn Any, BuildHasherDefault<IdHasher>>;

thread_local! {
    static ARENAS: RefCell<Table> = RefCell::new(Table::default());
    /// The arena found last, which the next allocation most often wants.
    static LAST: Cell<Option<(TypeId, &'static dyn Any)>> = const { Cell::new(None) };
}

/// This thread's heap arena for `T`, made at its first use.
#[inline]
fn arena_of<T: 'static>() -> &'static Arena<T> {
    let id = TypeId::of::<T>();
    if let Some((k, a)) = LAST.with(Cell::get)
        && k == id
        && let Some(arena) = a.downcast_ref::<Arena<T>>()
    {
        return arena;
    }
    let a: &'static dyn Any = ARENAS.with(|t| {
        *t.borrow_mut().entry(id).or_insert_with(|| {
            let arena: &'static Arena<T> = Box::leak(Box::default());
            arena
        })
    });
    LAST.with(|l| l.set(Some((id, a))));
    match a.downcast_ref::<Arena<T>>() {
        Some(arena) => arena,
        None => crate::fail("heap: an arena of another type"),
    }
}

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
        arena_of::<T>().alloc_noting(v, crate::census::block::<T>)
    }
    #[cfg(not(feature = "heap-stats"))]
    arena_of::<T>().alloc(v)
}

/// `cons`, in the heap's arena of pairs.
#[inline]
pub fn heap_cons<T: 'static + crate::Trace>(car: T, cdr: List<T>) -> List<T> {
    crate::ListAt::of_pair(heap_alloc(Cons::new(car, cdr)))
}

/// `new`, in the heap's arena of cells.
#[inline]
pub fn heap_new<T: Copy + 'static + crate::Trace>(v: T) -> Ref<T> {
    heap_alloc(crate::Slot::new(v))
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
