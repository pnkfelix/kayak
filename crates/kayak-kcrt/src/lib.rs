//! The runtime of kc's native code (`docs/research/kc.md`), grown with kc's
//! steps, as Ghuloum's is: at step 1, a `main` that runs the program's
//! `kc_entry` and writes the value it gives, in Fixpt's format
//! (`fixpt-heap`'s `value.rs`: three tag bits, a fixnum's tag 0, its value
//! above them), as Fixpt writes it.
//!
//! kc's code is called through `extern "C"`: the one place here that is
//! not safe Rust.
#![allow(unsafe_code)]

const TAG_MASK: u64 = 0b111;
const TAG_FIXNUM: u64 = 0b000;

unsafe extern "C" {
    fn kc_entry() -> u64;
}

/// A value as Fixpt writes it.
fn written(v: u64) -> String {
    if v & TAG_MASK == TAG_FIXNUM {
        ((v as i64) >> 3).to_string()
    } else {
        format!("#<value {v:#x}>")
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    // SAFETY: `kc_entry` is kc's code, which takes nothing and gives a value.
    let v = unsafe { kc_entry() };
    println!("{}", written(v));
    0
}
