//! The programs of tests/kfx-io, each a binary (src/bin/, which
//! scripts/io-programs.py writes): a KFX program that does I/O, run as
//! Fixpt runs it (`fixpt run`), to compare the two (CLAUDES_CHOICES.md C49).

use kayak_rt::{Int, int_to_i64};

/// The exit status of a run of `main`, as Fixpt's: `main`'s result's low
/// byte, or 70 if the run fails, its message on the standard error.
pub fn status(main: fn() -> Int) -> i32 {
    std::panic::set_hook(Box::new(|info| {
        let message = match info.payload().downcast_ref::<String>() {
            Some(s) => s.clone(),
            None => info
                .payload()
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .unwrap_or_default(),
        };
        eprintln!("{message}");
    }));
    let status = match std::panic::catch_unwind(main) {
        Ok(n) => (int_to_i64(n) & 0xff) as i32,
        Err(_) => 70,
    };
    // The terminal as the run found it, as `fixpt` leaves it (C109).
    kayak_rt::finish();
    status
}
