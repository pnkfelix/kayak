//! kayak-cells, tier 1 (`docs/research/native.md`, C122): Fixpt's cellular
//! machine, written in KFX (`cells/`), run on the words of a listing
//! Fixpt's compiler wrote. `cells/main.kfx`'s `cells-main`, on a big
//! stack. Generated from `cells/` (`scripts/cells-programs.py --bless`);
//! this file is the only hand-written part.

#[allow(warnings, clippy::all)]
#[rustfmt::skip]
mod cells;

use kayak_rt::{int_to_i64, with_big_stack};

/// A procedure of tier 1's that native code calls by the convention's
/// `foreign` routine (C227, C229): given to kayak-jit, through kayak-rt, as
/// the host's; run by `cells/machine.kfx`'s `native-callback`, its words in
/// a mailbox, and the native frames' values in another, kept by tier 1
/// where its collector moves them, and given back.
fn host_foreign(
    closure: u64,
    args: &[u64],
    held: &mut [u64],
    next: usize,
) -> (Result<u64, ()>, usize, usize) {
    let b = kayak_rt::make_flatarray_i64(14, 0);
    b[0].set(closure as i64);
    b[1].set(args.len() as i64);
    for (i, &a) in args.iter().enumerate() {
        b[2 + i].set(a as i64);
    }
    b[10].set(next as i64);
    b[12].set(held.len() as i64);
    let h = kayak_rt::make_flatarray_i64(held.len().max(1) as i64, 0);
    for (i, &w) in held.iter().enumerate() {
        h[i].set(w as i64);
    }
    let status = cells::native_callback(b, h);
    for (i, w) in held.iter_mut().enumerate() {
        *w = h[i].get() as u64;
    }
    let answer = if status == 0 {
        Ok(b[0].get() as u64)
    } else {
        Err(())
    };
    (answer, b[10].get() as usize, b[11].get() as usize)
}

fn main() {
    kayak_rt::set_host_foreign(host_foreign);
    let status = with_big_stack(|| int_to_i64(cells::cells_main()));
    kayak_rt::finish();
    std::process::exit((status & 0xff) as i32);
}
