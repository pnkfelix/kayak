//! Closures as FX-26's heap grammar has them (C481, C496): a closure is a
//! heap object of its captures, each a Fixpt word (its type's
//! `Trace::Stored`, `Captures`), as Fixpt's closure is a bloblet of its free values,
//! each a word. Its code is its type's (C494), no word of it: so one of no
//! captures is no bytes. No header yet (TODO §54).
//!
//! kfx writes a closure as its captures, a tuple, and its code, a
//! function of them and its parameters that captures nothing (checked
//! when built). The tuple is stored as its words, and loaded again at
//! each call.
//!
//! A procedure value is a handle to `dyn CallN`, called by `.call(...)`:
//! stable Rust lets no type of ours be called as `f(x)`.

use crate::Handle;
use crate::trace::{Captures, Trace, Tracer, Visit};

/// A closure: its captures' words, its code.
#[repr(C)]
pub struct Closure<E: Captures, C> {
    words: E::Words,
    code: C,
}

impl<E: Captures, C> Visit for Closure<E, C> {
    fn visit<'a>(&'a self, t: &mut Tracer<'a>) {
        E::of_words(self.words).trace(t);
    }
}

/// A closure in a block of its own, never freed (kayak-rt has no
/// collector yet), noted for a census (C480).
fn kept<'r, T: Visit + 'r>(v: T) -> &'r T {
    // One of no bytes (no captures, C493) allocates nothing: not noted.
    if std::mem::size_of::<T>() > 0 {
        crate::stats::note::<T>(std::mem::size_of::<T>());
    }
    let r: &'r T = Box::leak(Box::new(v));
    #[cfg(feature = "heap-stats")]
    if std::mem::size_of::<T>() > 0 {
        crate::census::one(r);
    }
    r
}

/// A function as a procedure value: its code alone, nothing captured, no
/// bytes (a top-level procedure's static closure, as Fixpt's).
pub struct FnProc<F>(F);

impl<F> Visit for FnProc<F> {
    fn visit<'a>(&'a self, _: &mut Tracer<'a>) {}
}

/// What may be a procedure value `P`: a function, of `P`'s arity.
pub trait AsProc<P> {
    fn into_proc(self) -> P;
}

/// A function as a procedure value, its arity the one wanted.
#[inline]
pub fn as_proc<P, F: AsProc<P>>(f: F) -> P {
    f.into_proc()
}

macro_rules! arities {
    ($($call:ident $proc:ident $procs:ident $make:ident $makes:ident ($($a:ident)*);)*) => {$(
        /// A procedure of this arity, as a value.
        pub trait $call<$($a,)* R>: Visit {
            #[allow(clippy::too_many_arguments, non_snake_case)]
            fn call(&self, $($a: $a),*) -> R;
        }

        #[allow(non_snake_case)]
        impl<E: Captures, Code: Fn(E, $($a),*) -> R, $($a,)* R> $call<$($a,)* R> for Closure<E, Code> {
            #[inline]
            fn call(&self, $($a: $a),*) -> R {
                (self.code)(E::of_words(self.words), $($a),*)
            }
        }

        /// A procedure value of this arity.
        pub type $proc<'r, $($a,)* R> = Handle<'r, dyn $call<$($a,)* R> + 'r>;
        /// One of an effect a process may have (C102): `Send + Sync`.
        pub type $procs<'r, $($a,)* R> = Handle<'r, dyn $call<$($a,)* R> + Send + Sync + 'r>;

        // A procedure value as a union's member (C491): boxed, its closure
        // behind `dyn`, not `Any`.
        impl<$($a: 'static,)* R: 'static> crate::value::Member for $proc<'static, $($a,)* R> {
            #[inline]
            fn object(self) -> &'static dyn crate::value::Obj {
                crate::value::boxed_object(self)
            }
            #[inline]
            fn of_object(o: &'static dyn crate::value::Obj) -> Option<Self> {
                crate::value::of_boxed(o)
            }
        }

        impl<$($a: 'static,)* R: 'static> crate::value::Member for $procs<'static, $($a,)* R> {
            #[inline]
            fn object(self) -> &'static dyn crate::value::Obj {
                crate::value::boxed_object(self)
            }
            #[inline]
            fn of_object(o: &'static dyn crate::value::Obj) -> Option<Self> {
                crate::value::of_boxed(o)
            }
        }

        impl<'r, $($a,)* R> Trace for $proc<'r, $($a,)* R> {
            crate::trace_whole!();
            #[inline]
            fn trace<'a>(self, t: &mut Tracer<'a>) where Self: 'a {
                t.reach(self.to_ref());
            }
        }

        impl<'r, $($a,)* R> Trace for $procs<'r, $($a,)* R> {
            crate::trace_whole!();
            #[inline]
            fn trace<'a>(self, t: &mut Tracer<'a>) where Self: 'a {
                t.reach(self.to_ref());
            }
        }

        #[allow(non_snake_case)]
        impl<Fun: Fn($($a),*) -> R, $($a,)* R> $call<$($a,)* R> for FnProc<Fun> {
            #[inline]
            fn call(&self, $($a: $a),*) -> R {
                (self.0)($($a),*)
            }
        }

        impl<'r, Fun: Fn($($a),*) -> R + 'r, $($a: 'r,)* R: 'r> AsProc<$proc<'r, $($a,)* R>> for Fun {
            #[inline]
            fn into_proc(self) -> $proc<'r, $($a,)* R> {
                Handle::made(kept(FnProc(self)))
            }
        }

        impl<'r, Fun: Fn($($a),*) -> R + Send + Sync + 'r, $($a: 'r,)* R: 'r> AsProc<$procs<'r, $($a,)* R>> for Fun {
            #[inline]
            fn into_proc(self) -> $procs<'r, $($a,)* R> {
                Handle::made(kept(FnProc(self)))
            }
        }

        /// A procedure value `Send + Sync`, as one that may not be.
        impl<'r, $($a: 'r,)* R: 'r> AsProc<$proc<'r, $($a,)* R>> for $procs<'r, $($a,)* R> {
            #[inline]
            fn into_proc(self) -> $proc<'r, $($a,)* R> {
                Handle::made(self.to_ref() as &(dyn $call<$($a,)* R> + 'r))
            }
        }

        impl<'r, $($a: 'r,)* R: 'r> AsProc<$procs<'r, $($a,)* R>> for $procs<'r, $($a,)* R> {
            #[inline]
            fn into_proc(self) -> $procs<'r, $($a,)* R> {
                self
            }
        }

        /// A closure of captures `env` and code `code`: a Rust closure that
        /// captures nothing (C493: zero bytes, checked when built, as a `fn`
        /// was), so that one of no captures is no bytes at all, made at no
        /// cost (a box of nothing allocates nothing).
        #[inline]
        pub fn $make<'r, E: Captures + 'r, Code: Fn(E, $($a),*) -> R + 'r, $($a: 'r,)* R: 'r>(env: E, code: Code) -> $proc<'r, $($a,)* R> {
            const { assert!(std::mem::size_of::<Code>() == 0, "a closure's code captures") };
            Handle::made(kept(Closure::<E, Code> { words: env.words(), code }))
        }

        /// A closure of captures `env`, all `Send + Sync`, and code `code`.
        #[inline]
        pub fn $makes<'r, E: Captures + Send + Sync + 'r, Code: Fn(E, $($a),*) -> R + Send + Sync + 'r, $($a: 'r,)* R: 'r>(env: E, code: Code) -> $procs<'r, $($a,)* R>
        where
            E::Words: Send + Sync,
        {
            const { assert!(std::mem::size_of::<Code>() == 0, "a closure's code captures") };
            Handle::made(kept(Closure::<E, Code> { words: env.words(), code }))
        }
    )*};
}

arities! {
    Call0 Proc0 Proc0s closure0 closure0s ();
    Call1 Proc1 Proc1s closure1 closure1s (A);
    Call2 Proc2 Proc2s closure2 closure2s (A B);
    Call3 Proc3 Proc3s closure3 closure3s (A B C);
    Call4 Proc4 Proc4s closure4 closure4s (A B C D);
    Call5 Proc5 Proc5s closure5 closure5s (A B C D F);
    Call6 Proc6 Proc6s closure6 closure6s (A B C D F G);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_closure_of_no_captures_no_bytes() {
        // Its code no pointer (C493): of no captures, nothing; of one
        // `i64`, that word alone.
        let f: Proc1<'static, i64, i64> = closure1((), |(), x| x + 1);
        assert_eq!(std::mem::size_of_val(f.to_ref()), 0);
        assert_eq!(f.call(1), 2);
        let g: Proc1<'static, i64, i64> = closure1((40i64,), |(k,), x| x + k);
        assert_eq!(std::mem::size_of_val(g.to_ref()), 8);
        assert_eq!(g.call(2), 42);
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
    fn its_captures_words() {
        // Each capture a Fixpt word (C496): an `i64` a fixnum, a string
        // its tagged pointer, an `f64` a flonum's.
        let f: Proc0<'static, f64> = closure0((7i64, crate::str_lit("x"), 1.5f64), |(k, s, x)| {
            k as f64 + s.as_str().len() as f64 + x
        });
        assert_eq!(std::mem::size_of_val(f.to_ref()), 24);
        assert_eq!(f.call(), 9.5);
    }
}
