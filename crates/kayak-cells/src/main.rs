//! kayak-cells, tier 1 (`docs/research/native.md`, C122): Fixpt's cellular
//! machine, written in KFX (`cells/`), run on the words of a listing
//! Fixpt's compiler wrote. `cells/main.kfx`'s `cells-main`, on a big
//! stack. Generated from `cells/` (`scripts/cells-programs.py --bless`);
//! this file is the only hand-written part.

#[allow(warnings, clippy::all)]
#[rustfmt::skip]
mod cells;

use kayak_rt::{int_to_i64, with_big_stack};

fn main() {
    let status = with_big_stack(|| int_to_i64(cells::cells_main()));
    kayak_rt::finish();
    std::process::exit((status & 0xff) as i32);
}
