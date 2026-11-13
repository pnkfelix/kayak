//! The runtime of kc's native code (`docs/research/kc.md`), grown with kc's
//! steps, as Ghuloum's is: a `main` that runs the program's `kc_entry` and
//! writes the value it gives, in Fixpt's format (`fixpt-heap`'s `value.rs`),
//! as Fixpt writes it (`fixpt-runtime`'s `print.rs`). Step 1: fixnums (three
//! tag bits, tag 0, the value above them). Step 2: immediates (tag 3, a kind
//! in the five bits above it, a payload from bit 8). Step 3: a failure,
//! `kc_fail`, said as Fixpt says it. Step 7: the heap, given to `kc_entry`,
//! and pairs (tag 1, the address of a cell of two words) written as lists.
//! Step 8: a program of many forms, each top-level expression's value
//! written by `kc_show`, called by kc's code, on a line of its own.
//!
//! kc's code is called through `extern "C"`: the one place here that is
//! not safe Rust.
#![allow(unsafe_code)]

const TAG_MASK: u64 = 0b111;
const TAG_FIXNUM: u64 = 0b000;
const TAG_PAIR: u64 = 0b001;
const TAG_IMMEDIATE: u64 = 0b011;
const IMM_SHIFT: u32 = 3;
const IMM_MASK: u64 = 0b1_1111;
const IMM_PAYLOAD_SHIFT: u32 = 8;
const IMM_FALSE: u64 = 0;
const IMM_TRUE: u64 = 1;
const IMM_NULL: u64 = 2;
const IMM_UNIT: u64 = 3;
const IMM_CHAR: u64 = 8;

unsafe extern "C" {
    fn kc_entry(heap: *mut u64, end: *mut u64) -> u64;
}

/// The heap's words: no collector yet, so all a run may use (C406).
const HEAP_WORDS: usize = 1 << 23;

/// A pair's car and cdr.
fn pair(v: u64) -> (u64, u64) {
    let p = (v - TAG_PAIR) as *const u64;
    // SAFETY: a pair's value is its cell's address, tag 1, a cell of two
    // words kc's code made on the heap, which lives as long as the run.
    unsafe { (*p, *p.add(1)) }
}

/// A character as Fixpt writes it (`print.rs`'s `put_char`).
fn put_char(out: &mut String, c: char) {
    out.push_str("#\\");
    match c {
        ' ' => out.push_str("space"),
        '\n' => out.push_str("newline"),
        '\t' => out.push_str("tab"),
        '\r' => out.push_str("return"),
        '\0' => out.push_str("null"),
        '\u{7}' => out.push_str("alarm"),
        '\u{8}' => out.push_str("backspace"),
        '\u{7f}' => out.push_str("delete"),
        '\u{1b}' => out.push_str("escape"),
        c if (c as u32) < 0x20 => out.push_str(&format!("x{:x}", c as u32)),
        c => out.push(c),
    }
}

/// A value as Fixpt writes it.
fn written(v: u64) -> String {
    let mut out = String::new();
    put(&mut out, v);
    out
}

/// A list as Fixpt writes it (`print.rs`'s `put`): its elements, and a
/// tail that is not a list after ` . `.
fn put(out: &mut String, v: u64) {
    if v & TAG_MASK != TAG_PAIR {
        out.push_str(&atom(v));
        return;
    }
    out.push('(');
    let (car, mut rest) = pair(v);
    put(out, car);
    loop {
        if rest == null() {
            break;
        }
        if rest & TAG_MASK == TAG_PAIR {
            let (a, d) = pair(rest);
            out.push(' ');
            put(out, a);
            rest = d;
        } else {
            out.push_str(" . ");
            put(out, rest);
            break;
        }
    }
    out.push(')');
}

const fn null() -> u64 {
    (IMM_NULL << IMM_SHIFT) | TAG_IMMEDIATE
}

/// A value other than a pair as Fixpt writes it.
fn atom(v: u64) -> String {
    match v & TAG_MASK {
        TAG_FIXNUM => ((v as i64) >> 3).to_string(),
        TAG_IMMEDIATE => {
            let payload = v >> IMM_PAYLOAD_SHIFT;
            match (v >> IMM_SHIFT) & IMM_MASK {
                IMM_FALSE => "#f".into(),
                IMM_TRUE => "#t".into(),
                IMM_NULL => "()".into(),
                IMM_UNIT => "#u".into(),
                IMM_CHAR => {
                    let mut out = String::new();
                    put_char(
                        &mut out,
                        char::from_u32(payload as u32).unwrap_or('\u{fffd}'),
                    );
                    out
                }
                k => format!("#<immediate {k}>"),
            }
        }
        _ => format!("#<value {v:#x}>"),
    }
}

/// A failure of kc's code, by number (`kc/x64.kfx`'s `fail-…`), said in
/// Fixpt's words, as `fixpt eval` says a failure, and the run ended.
#[unsafe(no_mangle)]
pub extern "C" fn kc_fail(what: u64) -> ! {
    let message = match what {
        1 => "not a Unicode scalar value",
        4 => "car or cdr of a non-pair",
        // kc's own: no collector yet.
        5 => "kc: not yet: the heap used up (no collector yet)",
        // kc's own, not Fixpt's: its integers are fixnums alone, as yet.
        2 => "kc: not yet: an integer past a fixnum",
        3 => "division by zero",
        _ => "a failure kc's runtime has no words for",
    };
    eprintln!("evaluation failed: {message}");
    std::process::exit(1)
}

/// A top-level expression's value, written on a line of its own, as
/// `fixpt eval` writes each (C407).
#[unsafe(no_mangle)]
pub extern "C" fn kc_show(v: u64) {
    println!("{}", written(v));
}

#[unsafe(no_mangle)]
pub extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    let heap: &'static mut [u64] = Vec::leak(vec![0u64; HEAP_WORDS]);
    let range = heap.as_mut_ptr_range();
    // SAFETY: `kc_entry` is kc's code, which takes the heap's first free
    // word and its end, and writes each top-level expression's value.
    unsafe { kc_entry(range.start, range.end) };
    0
}
