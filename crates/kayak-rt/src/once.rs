//! Constants made once (`CLAUDES_CHOICES.md` C21): a KFX constant whose
//! value allocates is made at its first use and kept, as FX-26 makes it
//! once, where it is defined. Making it may only allocate and read
//! globals, so when is not seen.
//!
//! Where it is kept (C102): a constant of a transmissible type, which a
//! process may hold, once for the whole program, a `static OnceLock`, which
//! rustc lets threads share because the type's Rust is `Sync`; any other
//! once for each thread, which no process can tell, holding none; or, with
//! the `unsafe-par` feature, once for the program too, since a `par`
//! branch on another thread may read it (`kayak_sync::Global`).

use std::cell::OnceCell;
use std::sync::OnceLock;
use std::thread::LocalKey;

/// Whether a constant of no transmissible type is kept once for the
/// program (`unsafe-par`) rather than once for each thread: a program that
/// gives each thread a session of its own (kayak-fx26's REPL, C103) needs
/// it not to be.
pub const KEPT_FOR_PROGRAM: bool = cfg!(feature = "unsafe-par");

/// Where a constant is kept: what it keeps, if made yet, asked first, as
/// nearly every use finds it made; else made.
pub trait OnceSlot<T> {
    fn made(&'static self) -> Option<T>;
    fn kept(&'static self, make: &mut dyn FnMut() -> T) -> T;
}

impl<T: Copy + 'static> OnceSlot<T> for LocalKey<OnceCell<T>> {
    #[inline]
    fn made(&'static self) -> Option<T> {
        self.with(|c| c.get().copied())
    }
    fn kept(&'static self, make: &mut dyn FnMut() -> T) -> T {
        self.with(|c| *c.get_or_init(make))
    }
}

impl<T: Copy> OnceSlot<T> for OnceLock<T> {
    #[inline]
    fn made(&'static self) -> Option<T> {
        self.get().copied()
    }
    fn kept(&'static self, make: &mut dyn FnMut() -> T) -> T {
        *self.get_or_init(make)
    }
}

#[cfg(feature = "unsafe-par")]
impl<T: Copy> OnceSlot<T> for kayak_sync::Global<T> {
    #[inline]
    fn made(&'static self) -> Option<T> {
        self.get().copied()
    }
    fn kept(&'static self, make: &mut dyn FnMut() -> T) -> T {
        *self.get_or_init(make)
    }
}

/// The value `slot` keeps, made by `make` the first time.
#[inline]
pub fn once<T, S: OnceSlot<T> + ?Sized>(slot: &'static S, make: impl FnOnce() -> T) -> T {
    match slot.made() {
        Some(v) => v,
        None => made_first(slot, make),
    }
}

#[cold]
#[inline(never)]
fn made_first<T, S: OnceSlot<T> + ?Sized>(slot: &'static S, make: impl FnOnce() -> T) -> T {
    let mut make = Some(make);
    slot.kept(&mut || (make.take().expect("a constant made twice"))())
}

/// A constant of no transmissible type, kept for each thread; or with the
/// `unsafe-par` feature, for the program (C102).
#[cfg(not(feature = "unsafe-par"))]
#[macro_export]
macro_rules! kept_constant {
    ($name:ident : $t:ty) => {
        thread_local!(static $name: $crate::OnceCell<$t> = const { $crate::OnceCell::new() });
    };
}

#[cfg(feature = "unsafe-par")]
#[macro_export]
macro_rules! kept_constant {
    ($name:ident : $t:ty) => {
        static $name: $crate::Global<$t> = $crate::Global::new();
    };
}
