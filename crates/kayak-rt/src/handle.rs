//! `Handle<'r, T>` (C478; the user's, 2026-11-18): a reference to a value
//! in the heap (`'static`) or in a `letrena`'s region (`'r`), the one type
//! every such reference has: what KFX's Rust holds of the heap, made only
//! by the heap's and the regions' allocations (`heap_alloc`, an arena's
//! `alloc`), read through it (`Deref`). Today a reference and nothing more,
//! the same size, so no cost: a step to a collector (`TODO.md` §54), which
//! needs to know which words of a value are the heap's (each datatype's
//! handles, traced precisely) apart from the Rust stack's (scanned
//! conservatively, Bartlett's mostly-copying).

use std::ops::Deref;

/// A value of the heap's or a region's, `T`, for as long as `'r`.
pub struct Handle<'r, T: ?Sized>(&'r T);

impl<'r, T: ?Sized> Handle<'r, T> {
    /// A handle of a value just allocated: the heap's and the regions'.
    #[inline(always)]
    pub(crate) fn made(r: &'r T) -> Self {
        Handle(r)
    }

    /// The value, for as long as the handle's lifetime: what a reader may
    /// keep, as a reference was.
    #[inline(always)]
    pub fn to_ref(self) -> &'r T {
        self.0
    }

    /// Whether two handles are of one value (FX-26's `eq?` of them).
    #[inline(always)]
    pub fn same(self, other: Self) -> bool {
        std::ptr::eq(self.0, other.0)
    }
}

impl<T: ?Sized> Clone for Handle<'_, T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for Handle<'_, T> {}

impl<T: ?Sized> Deref for Handle<'_, T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        self.0
    }
}

impl<T: ?Sized + std::fmt::Debug> std::fmt::Debug for Handle<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
