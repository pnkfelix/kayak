//! Closures as FX-26's heap grammar has them (C481, C496, C501): a
//! procedure value is one Fixpt word, to Fixpt's native closure (kind 41),
//! its free values each a Fixpt word (`Free`), then a word to its code's
//! object (`obj.rs`'s `procs`, where what reads a closure again by its
//! code is). Its Rust code is its type's (C494), no bytes of it.
//!
//! kfx writes a closure as its captures, a tuple, and its code, a
//! function of them and its parameters that captures nothing (checked
//! when built). The tuple is stored as its words, and loaded again at
//! each call.
//!
//! A procedure value is called by `.call(...)`: stable Rust lets no type
//! of ours be called as `f(x)`.

pub use crate::obj::procs::*;

/// What may be a procedure value `P`: a function, of `P`'s arity.
pub trait AsProc<P> {
    fn into_proc(self) -> P;
}

/// A function as a procedure value, its arity the one wanted.
#[inline]
pub fn as_proc<P, F: AsProc<P>>(f: F) -> P {
    f.into_proc()
}

macro_rules! as_procs {
    ($($proc:ident $procs:ident $make:ident $makes:ident ($($a:ident)*);)*) => {$(
        /// A function as a procedure value: a closure of nothing, its code
        /// the function (a top-level procedure's, as Fixpt's).
        #[allow(non_snake_case)]
        impl<'r, Fun: Fn($($a),*) -> R + Copy + 'r, $($a: 'r,)* R: 'r> AsProc<$proc<'r, $($a,)* R>> for Fun {
            #[inline]
            fn into_proc(self) -> $proc<'r, $($a,)* R> {
                $make((), move |(), $($a: $a),*| self($($a),*))
            }
        }

        #[allow(non_snake_case)]
        impl<'r, Fun: Fn($($a),*) -> R + Copy + Send + Sync + 'r, $($a: 'r,)* R: 'r> AsProc<$procs<'r, $($a,)* R>> for Fun {
            #[inline]
            fn into_proc(self) -> $procs<'r, $($a,)* R> {
                $makes((), move |(), $($a: $a),*| self($($a),*))
            }
        }

        /// A procedure value `Send + Sync`, as one that may not be.
        impl<'r, $($a: 'r,)* R: 'r> AsProc<$proc<'r, $($a,)* R>> for $procs<'r, $($a,)* R> {
            #[inline]
            fn into_proc(self) -> $proc<'r, $($a,)* R> {
                self.unsent()
            }
        }

        impl<'r, $($a: 'r,)* R: 'r> AsProc<$procs<'r, $($a,)* R>> for $procs<'r, $($a,)* R> {
            #[inline]
            fn into_proc(self) -> $procs<'r, $($a,)* R> {
                self
            }
        }
    )*};
}

as_procs! {
    Proc0 Proc0s closure0 closure0s ();
    Proc1 Proc1s closure1 closure1s (A);
    Proc2 Proc2s closure2 closure2s (A B);
    Proc3 Proc3s closure3 closure3s (A B C);
    Proc4 Proc4s closure4 closure4s (A B C D);
    Proc5 Proc5s closure5 closure5s (A B C D F);
    Proc6 Proc6s closure6 closure6s (A B C D F G);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Trace;

    /// A word's object's `i`th word back from its suffix.
    fn back(w: crate::HeapWord, i: usize) -> u64 {
        // A test's reading of what it made: through the word's own
        // provenance, as a pair's or a box's test does (`obj.rs`).
        let words: &[u64] = crate::obj::words_before(w, i);
        words[0]
    }

    #[test]
    fn a_closure_one_word_to_fixpt_native_closure() {
        // One word; its object Fixpt's native closure (kind 41, frozen),
        // `[free…][code][trailer]`: of no free values, its code and
        // trailer, two fields.
        let f: Proc1<'static, i64, i64> = closure1((), |(), x| x + 1);
        assert_eq!(std::mem::size_of::<Proc1<'static, i64, i64>>(), 8);
        assert_eq!(f.call(1), 2);
        let w = f.store();
        assert_eq!(w.raw() & 7, 0b100);
        let h = back(w, 3);
        assert_eq!(
            (h & 7, (h >> 3) & 0xff, (h >> 12) & 3, (h >> 14) & 0x3ffff),
            (0b110, 41, 0b11, 2)
        );
        assert_eq!(back(w, 2) & 7, 0b100);
        // Of two: free value 0 nearest its code (slot 3), 1 before it.
        let g: Proc1<'static, i64, i64> = closure1((40i64, 2i64), |(k, j), x| x + k - j);
        assert_eq!(g.call(4), 42);
        let w = g.store();
        assert_eq!(back(w, 3), crate::HeapWord::fixnum(40).raw() as u64);
        assert_eq!(back(w, 4), crate::HeapWord::fixnum(2).raw() as u64);
        assert_eq!((back(w, 5) >> 14) & 0x3ffff, 4);
    }

    #[test]
    fn a_closure_called_and_traced() {
        let s = crate::str_lit("hello");
        let xs = crate::heap_cons(1i64, crate::nil());
        let f: Proc1<'static, i64, i64> = closure1((s, 40i64, xs), |(s, k, xs), n| {
            s.as_str().len() as i64 + k + n + xs.pair().unwrap().car()
        });
        assert_eq!(f.call(1), 5 + 40 + 1 + 1);
        let t = crate::traced(f);
        let (rows, strings) = t.census();
        // The closure, and the pair its captures hold; the string.
        assert_eq!(rows.iter().map(|r| r.1).sum::<u64>(), 2);
        assert_eq!(strings.0, 1);
    }

    #[test]
    fn a_function_as_a_procedure() {
        fn twice(x: i64) -> i64 {
            2 * x
        }
        let f: Proc1<'static, i64, i64> = as_proc(twice);
        assert_eq!(f.call(21), 42);
        let g: Proc1s<'static, i64, i64> = as_proc(twice);
        let h: Proc1<'static, i64, i64> = as_proc(g);
        assert_eq!(h.call(4), 8);
    }
}
