//! Closures as FX-26's heap grammar has them (C481): a closure is a heap
//! object of its traced fields, then its untraced suffix, then its code,
//! as Fixpt's closure (kind 8) is a bloblet of its captures and code. Not
//! Fixpt's bytes, but its structure: what a collector follows is the
//! fields, apart from the plain bytes (an `i64`, an `f64`, a character)
//! and the code pointer.
//!
//! kfx writes a closure as its captures, a tuple, and its code, a
//! function of them and its parameters that captures nothing (rustc
//! checks that: a closure that captures is no `fn`). The tuple is split by
//! its types' `Trace` (`trace.rs`) into fields and bytes, kept so, and
//! joined again at each call.
//!
//! A procedure value is a handle to `dyn CallN`, called by `.call(...)`:
//! stable Rust lets no type of ours be called as `f(x)`.

use crate::Handle;
use crate::trace::{Trace, Tracer, Visit};

/// A closure: its captures' traced fields, their plain bytes, its code.
#[repr(C)]
pub struct Closure<E: Trace, C> {
    fields: E::Field,
    suffix: E::Byte,
    code: C,
}

impl<E: Trace, C> Visit for Closure<E, C> {
    fn visit<'a>(&'a self, t: &mut Tracer<'a>) {
        self.fields.trace(t);
    }
}

/// A closure in a block of its own, never freed (kayak-rt has no
/// collector yet), noted for a census (C480).
fn kept<'r, T: Visit + 'r>(v: T) -> &'r T {
    crate::stats::note::<T>(std::mem::size_of::<T>());
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
        impl<E: Trace, $($a,)* R> $call<$($a,)* R> for Closure<E, fn(E, $($a),*) -> R> {
            #[inline]
            fn call(&self, $($a: $a),*) -> R {
                (self.code)(E::join(self.fields, self.suffix), $($a),*)
            }
        }

        /// A procedure value of this arity.
        pub type $proc<'r, $($a,)* R> = Handle<'r, dyn $call<$($a,)* R> + 'r>;
        /// One of an effect a process may have (C102): `Send + Sync`.
        pub type $procs<'r, $($a,)* R> = Handle<'r, dyn $call<$($a,)* R> + Send + Sync + 'r>;

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

        /// A closure of captures `env` and code `code`.
        #[inline]
        pub fn $make<'r, E: Trace + 'r, $($a: 'r,)* R: 'r>(env: E, code: fn(E, $($a),*) -> R) -> $proc<'r, $($a,)* R> {
            let (fields, suffix) = env.split();
            Handle::made(kept(Closure::<E, fn(E, $($a),*) -> R> { fields, suffix, code }))
        }

        /// A closure of captures `env`, all `Send + Sync`, and code `code`.
        #[inline]
        pub fn $makes<'r, E: Trace + Send + Sync + 'r, $($a: 'r,)* R: 'r>(env: E, code: fn(E, $($a),*) -> R) -> $procs<'r, $($a,)* R>
        where
            E::Field: Send + Sync,
            E::Byte: Send + Sync,
        {
            let (fields, suffix) = env.split();
            Handle::made(kept(Closure::<E, fn(E, $($a),*) -> R> { fields, suffix, code }))
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
    fn a_closure_called_and_traced() {
        let s = crate::str_lit("hello");
        let xs = crate::heap_cons(1i64, None);
        let f: Proc1<'static, i64, i64> = closure1((s, 40i64, xs), |(s, k, xs), n| {
            s.as_str().len() as i64 + k + n + xs.unwrap().car()
        });
        assert_eq!(f.call(1), 5 + 40 + 1 + 1);
        let t = crate::traced(f);
        let (rows, strings) = t.census();
        // The closure, and the pair its fields hold; the string.
        assert_eq!(rows.iter().map(|r| r.1).sum::<u64>(), 2);
        assert_eq!(strings.0, 1);
    }

    #[test]
    fn its_bytes_kept_apart() {
        type E = (i64, crate::Str, f64);
        let c = Closure::<E, fn(E) -> i64> {
            fields: ((), crate::str_lit("x"), ()),
            suffix: (7, (), 1.5),
            code: |(k, _, _)| k,
        };
        let f: Proc0<'_, i64> = Handle::made(&c);
        assert_eq!(f.call(), 7);
        // The fields one string's size; the bytes two words.
        assert_eq!(
            std::mem::size_of::<<E as Trace>::Field>(),
            std::mem::size_of::<crate::Str>()
        );
        assert_eq!(std::mem::size_of::<<E as Trace>::Byte>(), 16);
    }
}
