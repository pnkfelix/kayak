//! A value copied whole into a fragment (TODO §54 step 4b's (b), Erlang's
//! send): its graph, every allocation of the space it reaches, copied into
//! pages of no thread's (`space::Fragment`), which the thread that takes it
//! adopts as its own; so that no thread's heap is reached from another's.
//! Precise: the value's Fixpt words (`Trace::store`), and each object's
//! fields as its layout says (`laid`), as a collection finds them; each
//! allocation copied once (shared structure and cycles kept so), and what
//! is outside the space (a static, a kept block: a mailbox's or an I-cell's
//! address, a capability) left as it is.

use super::collect::{each_object, target_word};
use super::space::{self, Fragment, View};
use std::collections::HashMap;

/// `v`'s copy, its graph in `into`. A value held as itself (`Trace::WHOLE`,
/// no Fixpt word yet) is returned as it is: shared, as before (a debt of
/// §54 step 4b's (b)).
pub(crate) fn copied<T: crate::Trace>(v: T, into: &mut Fragment) -> T {
    if T::WHOLE {
        return v;
    }
    const {
        assert!(
            std::mem::size_of::<T::Stored>().is_multiple_of(8),
            "a value's stored form whole words"
        )
    };
    let stored = v.store();
    // Nothing collects as it copies: the words it reads stay where they
    // are.
    let _held = space::NoCollect::new();
    let mut c = Copier {
        view: View::new(),
        into,
        copies: HashMap::new(),
        work: Vec::new(),
    };
    let n = std::mem::size_of::<T::Stored>() / 8;
    let mut out = stored;
    // SAFETY: a stored value of no `WHOLE` part is its parts' Fixpt words,
    // `n` of them, read and written as `u64`s.
    let words =
        unsafe { std::slice::from_raw_parts_mut(std::ptr::from_mut(&mut out).cast::<u64>(), n) };
    for w in words.iter_mut() {
        *w = c.word(*w);
    }
    c.drain();
    T::load(out)
}

struct Copier<'f> {
    view: View,
    into: &'f mut Fragment,
    /// Each allocation copied, by its first word: its copy's.
    copies: HashMap<usize, usize>,
    /// Allocations copied whose fields are not yet: (from, to, words).
    work: Vec<(usize, usize, usize)>,
}

impl Copier<'_> {
    /// The word `w`, made to point at its target's copy (made now if not
    /// yet) where it points into the space; else as it is.
    fn word(&mut self, w: u64) -> u64 {
        let Some(t) = target_word(&self.view, w) else {
            return w;
        };
        if self.into.holds(t) {
            return w;
        }
        let to = match self.copies.get(&t) {
            Some(&to) => to,
            None => self.copy(t),
        };
        w.wrapping_add(to as u64).wrapping_sub(t as u64)
    }

    /// The allocation at `a` (its first word) copied whole into the
    /// fragment, its fields to be made to point at their copies.
    fn copy(&mut self, a: usize) -> usize {
        let n = match self.view.page_of(a) {
            Some(_) => {
                let mut end = a;
                each_object(&self.view, a, |o| end = o.end);
                self.view.extent_of(a, end)
            }
            None => match self.view.large_owner(a) {
                Some((_, w, _)) => w,
                None => unreachable!("a target of the space"),
            },
        };
        let to = self.into.words(n);
        // SAFETY: `n` words of an allocation of the space, read whole into
        // `n` new words of the fragment, which nothing else overlaps.
        unsafe {
            std::ptr::copy_nonoverlapping(std::ptr::with_exposed_provenance::<u64>(a), to, n);
        }
        let to = to.expose_provenance();
        self.copies.insert(a, to);
        self.work.push((a, to, n));
        to
    }

    /// Each copy's fields made to point at their targets' copies, those
    /// copied as they are found.
    fn drain(&mut self) {
        while let Some((from, to, _)) = self.work.pop() {
            let mut fields = Vec::new();
            each_object(&self.view, from, |o| {
                for f in o.fields {
                    fields.push((std::ptr::from_ref(f).addr() - from, f.raw() as u64));
                }
            });
            for (off, w) in fields {
                let new = self.word(w);
                if new != w {
                    // SAFETY: a field's word of the copy, which only this
                    // copier writes, as the fragment is no thread's.
                    unsafe { std::ptr::with_exposed_provenance_mut::<u64>(to + off).write(new) };
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Int, List, heap_cons, i64_to_int, int_to_i64, nil};

    fn sum(xs: List<Int>) -> i64 {
        let mut s = 0;
        let mut at = xs;
        while let Some(c) = at.pair() {
            s += int_to_i64(c.car());
            at = c.cdr;
        }
        s
    }

    #[test]
    fn a_list_copied_whole_into_a_fragment() {
        let xs = (0..1000)
            .rev()
            .fold(nil(), |acc, i| heap_cons(i64_to_int(i), acc));
        let mut f = Fragment::new();
        let ys = copied(xs, &mut f);
        assert_eq!(sum(ys), 999 * 1000 / 2);
        // Every pair of the copy in the fragment, none the original's.
        let mut at = ys;
        let mut n = 0;
        while let Some(c) = at.pair() {
            assert!(
                f.holds(at.0.raw() & !7),
                "a pair of the copy outside the fragment"
            );
            at = c.cdr;
            n += 1;
        }
        assert_eq!(n, 1000);
        assert_ne!(xs.0.raw(), ys.0.raw());
        // Adopted, the copy is this thread's, and survives its collections.
        space::adopt_fragment(f);
        let _ = std::hint::black_box((0..300).fold(nil(), |acc, i| heap_cons(i64_to_int(i), acc)));
        super::super::collect::collect();
        assert_eq!(sum(ys), 999 * 1000 / 2);
    }

    #[test]
    fn shared_structure_copied_once() {
        // A pair whose car and cdr are one list: its copy's are one too.
        let xs = (0..10).fold(nil(), |acc, i| heap_cons(i64_to_int(i), acc));
        let p = heap_cons(xs, heap_cons(xs, nil()));
        let mut f = Fragment::new();
        let q = copied(p, &mut f);
        let c = q.pair().expect("a pair");
        let d = c.cdr.pair().expect("a pair");
        assert_eq!(c.car().0.raw(), d.car().0.raw());
        assert_ne!(c.car().0.raw(), xs.0.raw());
        assert_eq!(sum(c.car()), 45);
    }
}
