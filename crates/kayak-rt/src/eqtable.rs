//! FX-26's tables keyed by identity, `(eqtable k v kr r)` (Fixpt's fx26.md,
//! "Identity"; C194): keys whose identity is their address, mutable objects
//! alone, a reference (`&Cell`), an array (a slice of `Cell`s) or an I-cell,
//! none of which is copied where a `Value` boxes it (`value::other` boxes the
//! pointer). A table is a map from a key's address to its entry; its type
//! carries the keys' lifetime, so that rustc refuses a table that outlives a
//! `letrena` whose objects key it. No operation iterates, so the order of
//! addresses is never seen: answers are the same on every run, and Fixpt's.
//! A pair's identity waits (TODO §36): a list seen as a union might be
//! rebuilt.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};
use std::marker::PhantomData;

use crate::array::Array;
use crate::process::ICellBox;
use crate::{Int, int_lit};

/// What has identity: a mutable object, by its address.
pub trait Identity: Copy {
    fn address(self) -> usize;
}

impl<T> Identity for &Cell<T> {
    fn address(self) -> usize {
        std::ptr::from_ref(self) as usize
    }
}

impl<T: Copy> Identity for Array<T> {
    fn address(self) -> usize {
        self.0.as_ptr() as usize
    }
}

impl<T> Identity for &ICellBox<T> {
    fn address(self) -> usize {
        std::ptr::from_ref(self) as usize
    }
}

/// `(identity k kr)`: that `k` has identity, made only by the standard
/// procedures below.
pub struct IdentityOf<K>(PhantomData<K>);

impl<K> Clone for IdentityOf<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K> Copy for IdentityOf<K> {}

/// `ref-identity`.
pub fn ref_identity<'a, T>() -> IdentityOf<&'a Cell<T>> {
    IdentityOf(PhantomData)
}

/// `array-identity`.
pub fn array_identity<T: Copy>() -> IdentityOf<Array<T>> {
    IdentityOf(PhantomData)
}

/// `icell-identity`.
pub fn icell_identity<'a, T>() -> IdentityOf<&'a ICellBox<T>> {
    IdentityOf(PhantomData)
}

/// An address's hash: the address itself, its low bits (alignment) mixed
/// in; fixed, so no run differs from another.
#[derive(Default)]
pub struct AddressHasher(u64);

impl Hasher for AddressHasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.0 = self.0.rotate_left(8) ^ u64::from(*b);
        }
    }
    fn write_usize(&mut self, n: usize) {
        self.0 = (n as u64) ^ ((n as u64) >> 7);
    }
}

/// A table: each key's address to the key and its value.
pub struct EqTableBox<K, V> {
    entries: RefCell<HashMap<usize, (K, V), BuildHasherDefault<AddressHasher>>>,
}

/// `(eqtable k v kr r)`, as long as its keys live, `'t`.
pub type EqTableAt<'t, K, V> = &'t EqTableBox<K, V>;
/// One whose keys live as long as the heap.
pub type EqTable<K, V> = EqTableAt<'static, K, V>;

/// `make-eqtable`.
pub fn make_eqtable<'t, K: Identity + 't, V: Copy + 't>(_: IdentityOf<K>) -> EqTableAt<'t, K, V> {
    crate::alloc(EqTableBox {
        entries: RefCell::new(HashMap::default()),
    })
}

/// `eqtable-ref`: the value of `k`, or `none` if it has none.
pub fn eqtable_ref<K: Identity, V: Copy>(t: EqTableAt<'_, K, V>, k: K, none: V) -> V {
    t.entries
        .borrow()
        .get(&k.address())
        .map_or(none, |(_, v)| *v)
}

/// `eqtable-has?`.
pub fn eqtable_has_p<K: Identity, V: Copy>(t: EqTableAt<'_, K, V>, k: K) -> bool {
    t.entries.borrow().contains_key(&k.address())
}

/// `eqtable-set!`.
pub fn eqtable_set_x<K: Identity, V: Copy>(t: EqTableAt<'_, K, V>, k: K, v: V) {
    t.entries.borrow_mut().insert(k.address(), (k, v));
}

/// `eqtable-delete!`.
pub fn eqtable_delete_x<K: Identity, V: Copy>(t: EqTableAt<'_, K, V>, k: K) {
    t.entries.borrow_mut().remove(&k.address());
}

/// `eqtable-count`.
pub fn eqtable_count<K: Identity, V: Copy>(t: EqTableAt<'_, K, V>) -> Int {
    int_lit(t.entries.borrow().len() as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn by_address() {
        let a = crate::heap_new(1i64);
        let b = crate::heap_new(1i64);
        let t: EqTable<&Cell<i64>, i64> = make_eqtable(ref_identity());
        eqtable_set_x(t, a, 10);
        eqtable_set_x(t, b, 20);
        eqtable_set_x(t, a, 30);
        assert_eq!(eqtable_ref(t, a, 0) + eqtable_ref(t, b, 0), 50);
        eqtable_delete_x(t, b);
        assert!(eqtable_has_p(t, a) && !eqtable_has_p(t, b));
    }
}
