//! Constants made once (`CLAUDES_CHOICES.md` C21): a KFX constant whose
//! value allocates is made at its first use and kept, as FX-26 makes it
//! once, where it is defined. Making it may only allocate and read
//! globals, so when is not seen.

use std::cell::OnceCell;
use std::thread::LocalKey;

/// The value `cell` keeps, made by `make` the first time.
pub fn once<T: Copy + 'static>(
    cell: &'static LocalKey<OnceCell<T>>,
    make: impl FnOnce() -> T,
) -> T {
    cell.with(|c| *c.get_or_init(make))
}
