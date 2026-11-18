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
//! Step 9: closures, bloblets (tag 4) of Fixpt's layout, written by their
//! kind. Step 11: strings and symbols (C421), of Fixpt's layout too
//! (`fixpt-heap`'s `make_string`, `intern`): a string's words its length
//! and its code points, two a word; a symbol's fields its name, its hash and
//! its global slot. What kc's code makes of them past a literal (a
//! string appended, a substring, an append's pairs) is made here, in kc's
//! own heap, which is collected (C453, `gc.rs`; C455): a routine that makes
//! one is given its arguments where the collector moves them. A symbol is
//! this runtime's own, interned for the run, as Fixpt's symbol table keeps
//! each symbol for its heap's life.
//!
//! kc's code is called through `extern "C"`: the one place here that is
//! not safe Rust.
//!
//! The routines alone (C452): `kayak-kcrt` adds a built program's `main`
//! and state; kayak-rt links this crate into a KFX program (kc), whose
//! syscall `kayak:kc-routine/1` gives each routine's address by name, so
//! that kc's code run in process (`kc run`) calls these same routines. A
//! routine that fails ends a built program (`failed`); in process its
//! variant (`routine`'s, `here_…`) keeps the words and answers 0, and the
//! code goes back to the trampoline (x64.kfx's `kc_failed`), as Fixpt's
//! call-outs keep their message and say they failed.
#![allow(unsafe_code)]

mod gc;
pub use gc::{Heap, collections};

const TAG_MASK: u64 = 0b111;
const TAG_FIXNUM: u64 = 0b000;
const TAG_PAIR: u64 = 0b001;
const TAG_IMMEDIATE: u64 = 0b011;
const TAG_BLOBLET: u64 = 0b100;
const IMM_SHIFT: u32 = 3;
const IMM_MASK: u64 = 0b1_1111;
const IMM_PAYLOAD_SHIFT: u32 = 8;
const IMM_FALSE: u64 = 0;
const IMM_TRUE: u64 = 1;
const IMM_NULL: u64 = 2;
const IMM_UNIT: u64 = 3;
const IMM_CHAR: u64 = 8;

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
pub fn written(v: u64) -> String {
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

// ------------------------------------------------------------ objects

const KIND_STRING: u64 = 1;
const KIND_SYMBOL: u64 = 2;
const KIND_CLOSURE: u64 = 8;
const KIND_BOX: u64 = 10;
const KIND_BLOBLET: u64 = 32;
const KIND_SUM: u64 = 36;
const KIND_PRODUCT: u64 = 37;
const TAG_HEADER: u64 = 0b110;
const TAG_TRAILER: u64 = 0b101;

fn word(addr: u64) -> u64 {
    // SAFETY: an address kc's code or this runtime made an object at, the
    // object's own words, which live as long as the run.
    unsafe { *(addr as *const u64) }
}

/// A bloblet's header (its suffix's address `v - 4`): the word before it,
/// or, before that, a trailer saying how far back the header is.
fn header(v: u64) -> u64 {
    let p = v - TAG_BLOBLET;
    let w = word(p - 8);
    if w & TAG_MASK == TAG_TRAILER {
        word(p - 8 - 8 * (w >> 3))
    } else {
        w
    }
}

fn kind(v: u64) -> u64 {
    (header(v) >> 3) & 0xff
}

/// A string's code points.
fn string_text(v: u64) -> String {
    let p = v - TAG_BLOBLET;
    let n = word(p) as usize;
    (0..n)
        .map(|i| {
            let w = word(p + 8 + 8 * (i as u64 / 2));
            let c = if i % 2 == 0 {
                w as u32
            } else {
                (w >> 32) as u32
            };
            char::from_u32(c).unwrap_or('\u{fffd}')
        })
        .collect()
}

/// Words this runtime makes, as long as the run: a symbol and its name.
fn made(words: Vec<u64>) -> u64 {
    Vec::leak(words).as_ptr() as u64
}

fn set(addr: u64, w: u64) {
    // SAFETY: a word of room `gc::words` gave in kc's heap, live and
    // writable for the run.
    unsafe { *(addr as *mut u64) = w }
}

/// Argument `k` of a routine that makes something in kc's heap (C455):
/// from the stack word its wrapper put it in (x64.kfx's
/// `x64-heap-routine`), which the collector moves, so read after room is
/// had.
fn arg(args: *const u64, k: u64) -> u64 {
    word(args as u64 + 8 * k)
}

/// A string of `s`, in this runtime's own memory: a symbol's name.
fn make_string(s: &str) -> u64 {
    made(string_words(s)) + 8 + TAG_BLOBLET
}

/// A string of `s` in kc's heap (C455), its block's.
fn string_in(block: *mut u64, s: &str) -> Result<u64, String> {
    let w = string_words(s);
    let at = gc::words(block, w.len() as u64)?;
    for (k, &x) in w.iter().enumerate() {
        set(at + 8 * k as u64, x);
    }
    Ok(at + 8 + TAG_BLOBLET)
}

/// A string's words, as Fixpt lays one out: a header of kind 1, its words
/// all suffix; the length; the code points, two a word.
fn string_words(s: &str) -> Vec<u64> {
    let cs: Vec<u32> = s.chars().map(|c| c as u32).collect();
    let n = cs.len();
    let words = 1 + n.div_ceil(2);
    let mut w = vec![
        TAG_HEADER | (KIND_STRING << 3) | ((8 * words as u64) << 32),
        n as u64,
    ];
    for k in 0..n.div_ceil(2) {
        let lo = cs[2 * k] as u64;
        let hi = if 2 * k + 1 < n {
            (cs[2 * k + 1] as u64) << 32
        } else {
            0
        };
        w.push(lo | hi);
    }
    w
}

std::thread_local! {
    static SYMBOLS: std::cell::RefCell<std::collections::HashMap<String, u64>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

/// A symbol of the name string `s`, the one of that name: as Fixpt's
/// `intern` makes one, its fields its name, its hash (FNV-1a, a fixnum) and
/// a global slot (none here, -1), and a trailer.
#[unsafe(no_mangle)]
pub extern "C" fn kc_intern(s: u64) -> u64 {
    let name = string_text(s);
    SYMBOLS.with(|t| {
        if let Some(&v) = t.borrow().get(&name) {
            return v;
        }
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for b in name.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100_0000_01b3);
        }
        let hash = ((h as i64 & i64::MAX) << 3) as u64;
        let name_v = make_string(&name);
        // Header (kind 2, four fields with the trailer), the fields, the
        // trailer four back to the header; the value past it.
        let w = vec![
            TAG_HEADER | (KIND_SYMBOL << 3) | (4 << 14),
            name_v,
            hash,
            (-1i64 << 3) as u64,
            TAG_TRAILER | (4 << 3),
        ];
        let v = made(w) + 40 + TAG_BLOBLET;
        t.borrow_mut().insert(name, v);
        v
    })
}

fn boolean(b: bool) -> u64 {
    if b {
        (IMM_TRUE << IMM_SHIFT) | TAG_IMMEDIATE
    } else {
        (IMM_FALSE << IMM_SHIFT) | TAG_IMMEDIATE
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kc_string_eq(a: u64, b: u64) -> u64 {
    boolean(string_text(a) == string_text(b))
}

#[unsafe(no_mangle)]
pub extern "C" fn kc_string_lt(a: u64, b: u64) -> u64 {
    boolean(string_text(a) < string_text(b))
}

fn string_appended(args: *const u64, block: *mut u64) -> Result<u64, String> {
    string_in(
        block,
        &(string_text(arg(args, 0)) + &string_text(arg(args, 1))),
    )
}

/// `string-append` of its two arguments, in kc's heap (C455), as each
/// routine that makes something there is called: its arguments' stack
/// words and the block (x64.kfx's `x64-heap-routine`).
#[unsafe(no_mangle)]
pub extern "C" fn kcrt_string_append(args: *const u64, block: *mut u64) -> u64 {
    string_appended(args, block).unwrap_or_else(|m| failed(&m))
}

fn substring_of(args: *const u64, block: *mut u64) -> Result<u64, String> {
    let cs: Vec<char> = string_text(arg(args, 0)).chars().collect();
    let (i, j) = ((arg(args, 1) as i64) >> 3, (arg(args, 2) as i64) >> 3);
    if i < 0 || j < i || j as usize > cs.len() {
        return Err("substring range out of bounds".to_string());
    }
    string_in(
        block,
        &cs[i as usize..j as usize].iter().collect::<String>(),
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn kcrt_substring(args: *const u64, block: *mut u64) -> u64 {
    substring_of(args, block).unwrap_or_else(|m| failed(&m))
}

/// A symbol's name: its field 0, the string it was made of.
#[unsafe(no_mangle)]
pub extern "C" fn kc_symbol_to_string(v: u64) -> u64 {
    word(v - TAG_BLOBLET - 40 + 8)
}

#[unsafe(no_mangle)]
pub extern "C" fn kc_is_string(v: u64) -> u64 {
    boolean(v & TAG_MASK == TAG_BLOBLET && kind(v) == KIND_STRING)
}

#[unsafe(no_mangle)]
pub extern "C" fn kc_is_symbol(v: u64) -> u64 {
    boolean(v & TAG_MASK == TAG_BLOBLET && kind(v) == KIND_SYMBOL)
}

#[unsafe(no_mangle)]
pub extern "C" fn kc_is_procedure(v: u64) -> u64 {
    boolean(v & TAG_MASK == TAG_BLOBLET && kind(v) == KIND_CLOSURE)
}

/// Whether `v` is a bloblet of `kind` (C448: a `typecase`'s shapes, as
/// Fixpt's heap has them).
#[unsafe(no_mangle)]
pub extern "C" fn kc_is_kind(v: u64, kind_wanted: u64) -> u64 {
    boolean(v & TAG_MASK == TAG_BLOBLET && kind(v) == kind_wanted)
}

/// `a`'s pairs copied, the last's cdr `b` (C447: a quasiquote's `,@`), in
/// kc's heap (C455), the room for them all had at once and `a` and `b`
/// read again after; `a` not a list fails as `cdr` of a non-pair does.
fn appended(args: *const u64, block: *mut u64) -> Result<u64, String> {
    let mut n = 0;
    let mut rest = arg(args, 0);
    while rest != null() {
        if rest & TAG_MASK != TAG_PAIR {
            return Err(fail_words(4).to_string());
        }
        n += 1;
        rest = pair(rest).1;
    }
    if n == 0 {
        return Ok(arg(args, 1));
    }
    let at = gc::words(block, 2 * n)?;
    let mut rest = arg(args, 0);
    for k in 0..n {
        let (car, cdr) = pair(rest);
        let tail = if k + 1 < n {
            at + 16 * (k + 1) + TAG_PAIR
        } else {
            arg(args, 1)
        };
        set(at + 16 * k, car);
        set(at + 16 * k + 8, tail);
        rest = cdr;
    }
    Ok(at + TAG_PAIR)
}

#[unsafe(no_mangle)]
pub extern "C" fn kcrt_append(args: *const u64, block: *mut u64) -> u64 {
    appended(args, block).unwrap_or_else(|m| failed(&m))
}

// ------------------------------------------------------------ control

/// A prompt's record (C458, x64.kfx's `x64-emit-prompt`): the prompt
/// before it, its tag, its frame's stack pointer, its handler's call.
const R_PREV: u64 = 0;
const R_TAG: u64 = 8;
const R_SP: u64 = 16;
/// An offset of no record, in a continuation's copy: past its segment.
const NO_RECORD: u64 = (-8i64) as u64;
const KIND_ARRAY: u64 = 32;
const ONCE: u64 = 8;
const RESUMED: u64 = 16;
const WHOLE: u64 = 32;
/// A mark's value in its record (C460): where a prompt's handler is.
const R_VALUE: u64 = 24;
/// The most fields a header holds (Fixpt's 18 bits; kc makes no large one).
const MOST_FIELDS: u64 = 0x3_ffff;

/// A continuation's copy's word `k` (C459): a bloblet of kind 32, its
/// fields below its trailer, the first just below it, so that `kc_resume`
/// reads its bytes at -20 from the value. Its words: the segment's bytes,
/// the innermost prompt's record's offset in it (or `NO_RECORD`), its
/// flags (8 once only, 16 resumed, 32 whole: C460's `cwcc`), then the
/// segment's words, each record's (a prompt's or a mark's) stack addresses
/// made offsets in it.
fn copy_at(v: u64, k: u64) -> u64 {
    v - TAG_BLOBLET - 8 * (k + 2)
}

/// `(call-with-composable-continuation f tag)` and its one-shot twin
/// (C459), as Fixpt's native convention takes one: the frames from the
/// capture's return (`args + 40`, x64.kfx's `kc_capture`) up to the return
/// to the prompt of `tag` (just below its record's stack pointer) copied
/// into the heap, and a closure of `kc_resume` over the copy. Its stack
/// words: the procedure, the tag, the innermost prompt, `kc_resume`'s
/// address, and its flags (8 once only; 32 whole, C460's `cwcc`, whose
/// second word is the run's stack pointer, `kc_run_base`, not a tag: the
/// segment then up to the run's return).
fn captured(args: *const u64, block: *mut u64) -> Result<u64, String> {
    let chain = arg(args, 2);
    let top = args as u64 + 40;
    let end = if arg(args, 4) & WHOLE != 0 {
        arg(args, 1) - 8
    } else {
        let mut at = chain;
        while at != 0 && word(at + R_TAG) != arg(args, 1) {
            at = word(at + R_PREV);
        }
        if at == 0 {
            return Err(fail_words(11).to_string());
        }
        word(at + R_SP) - 8
    };
    if end < top {
        return Err("kc: a continuation taken outside a run".to_string());
    }
    let len = (end - top) / 8;
    let fields = 3 + len;
    if fields + 1 > MOST_FIELDS {
        return Err("kc: not yet: a continuation of more than 262140 words".to_string());
    }
    // The copy and its closure at once; the stack's words moved by any
    // collection, so read after.
    let room = gc::words(block, fields + 2 + 5)?;
    let suffix = room + 8 * (fields + 2);
    set(room, TAG_HEADER | (KIND_ARRAY << 3) | ((fields + 1) << 14));
    set(suffix - 8, TAG_TRAILER | ((fields + 1) << 3));
    let v = suffix + TAG_BLOBLET;
    for j in 0..len {
        set(copy_at(v, 3 + j), word(top + 8 * j));
    }
    // The records inside the segment, innermost first: their stack
    // addresses made offsets in it.
    let inside = |r: u64| r >= top && r < end;
    set(copy_at(v, 0), 8 * len);
    set(
        copy_at(v, 1),
        if inside(chain) {
            chain - top
        } else {
            NO_RECORD
        },
    );
    set(copy_at(v, 2), arg(args, 4));
    let mut r = chain;
    while inside(r) {
        let j = (r - top) / 8;
        let prev = word(r + R_PREV);
        set(
            copy_at(v, 3 + j),
            if inside(prev) { prev - top } else { NO_RECORD },
        );
        set(copy_at(v, 3 + j + 2), word(r + R_SP) - top);
        r = prev;
    }
    // The closure (x64.kfx's `x64-emit-closure`, of one free value).
    let c = suffix;
    set(c, TAG_HEADER | (KIND_CLOSURE << 3) | (4 << 14));
    set(c + 8, v);
    set(c + 16, arg(args, 3));
    set(c + 24, 0);
    set(c + 32, TAG_TRAILER | (4 << 3));
    Ok(c + 40 + TAG_BLOBLET)
}

#[unsafe(no_mangle)]
pub extern "C" fn kcrt_capture(args: *const u64, block: *mut u64) -> u64 {
    captured(args, block).unwrap_or_else(|m| failed(&m))
}

/// A continuation resumed (C459, x64.kfx's `kc_resume`): its copy `v` put
/// at `dest`, its prompts' records made the innermost on the current ones
/// (`chain`; of a whole one, none: what it replaces had them); the
/// innermost record then, with 1 added (so never 0, which says a failure
/// in process).
fn resumed(v: u64, dest: u64, chain: u64) -> Result<u64, String> {
    let flags = word(copy_at(v, 2));
    if flags & ONCE != 0 {
        if flags & RESUMED != 0 {
            return Err(fail_words(12).to_string());
        }
        set(copy_at(v, 2), flags | RESUMED);
    }
    let chain = if flags & WHOLE != 0 { 0 } else { chain };
    let len = word(copy_at(v, 0)) / 8;
    for j in 0..len {
        set(dest + 8 * j, word(copy_at(v, 3 + j)));
    }
    let mut off = word(copy_at(v, 1));
    let head = if off == NO_RECORD { chain } else { dest + off };
    while off != NO_RECORD {
        let r = dest + off;
        let prev = word(r + R_PREV);
        set(r + R_SP, dest + word(r + R_SP));
        set(
            r + R_PREV,
            if prev == NO_RECORD {
                chain
            } else {
                dest + prev
            },
        );
        off = prev;
    }
    Ok(head | 1)
}

#[unsafe(no_mangle)]
pub extern "C" fn kcrt_resume_copy(v: u64, dest: u64, chain: u64) -> u64 {
    resumed(v, dest, chain).unwrap_or_else(|m| failed(&m))
}

/// A list of `values`, innermost first, in kc's heap (C460), room had at
/// once; `values` read after, as the collection may have moved them.
fn listed(block: *mut u64, n: u64, values: impl Fn() -> Vec<u64>) -> Result<u64, String> {
    if n == 0 {
        return Ok(null());
    }
    let at = gc::words(block, 2 * n)?;
    let vs = values();
    for (k, &x) in vs.iter().enumerate() {
        let k = k as u64;
        set(at + 16 * k, x);
        set(
            at + 16 * k + 8,
            if k + 1 < n {
                at + 16 * (k + 1) + TAG_PAIR
            } else {
                null()
            },
        );
    }
    Ok(at + TAG_PAIR)
}

/// The marks of key `arg 0` on the chain from `arg 1` (C460,
/// `current-marks`), innermost first.
fn current_marks(args: *const u64, block: *mut u64) -> Result<u64, String> {
    let of = || {
        let mut out = Vec::new();
        let mut r = arg(args, 1);
        while r != 0 {
            if word(r + R_TAG) == arg(args, 0) {
                out.push(word(r + R_VALUE));
            }
            r = word(r + R_PREV);
        }
        out
    };
    listed(block, of().len() as u64, of)
}

/// The marks of key `arg 1` a continuation `arg 0` took (C460,
/// `marks-of`): in its copy (the closure's free value), innermost first.
fn marks_of(args: *const u64, block: *mut u64) -> Result<u64, String> {
    let of = || {
        let v = word(arg(args, 0) - TAG_BLOBLET - 32);
        let mut out = Vec::new();
        let mut off = word(copy_at(v, 1));
        while off != NO_RECORD {
            let j = off / 8;
            if word(copy_at(v, 3 + j + 1)) == arg(args, 1) {
                out.push(word(copy_at(v, 3 + j + 3)));
            }
            off = word(copy_at(v, 3 + j));
        }
        out
    };
    listed(block, of().len() as u64, of)
}

#[unsafe(no_mangle)]
pub extern "C" fn kcrt_current_marks(args: *const u64, block: *mut u64) -> u64 {
    current_marks(args, block).unwrap_or_else(|m| failed(&m))
}

#[unsafe(no_mangle)]
pub extern "C" fn kcrt_marks_of(args: *const u64, block: *mut u64) -> u64 {
    marks_of(args, block).unwrap_or_else(|m| failed(&m))
}

extern "C" fn here_current_marks(args: *const u64, block: *mut u64) -> u64 {
    current_marks(args, block).unwrap_or_else(failed_here)
}

extern "C" fn here_marks_of(args: *const u64, block: *mut u64) -> u64 {
    marks_of(args, block).unwrap_or_else(failed_here)
}

extern "C" fn here_capture(args: *const u64, block: *mut u64) -> u64 {
    captured(args, block).unwrap_or_else(failed_here)
}

extern "C" fn here_resume_copy(v: u64, dest: u64, chain: u64) -> u64 {
    resumed(v, dest, chain).unwrap_or_else(failed_here)
}

/// An array's index past its end, or below 0 (C422): kc's words, in a
/// string's way; Fixpt's machines each say it their own way.
#[unsafe(no_mangle)]
pub extern "C" fn kc_array_index_fail(len: u64) -> ! {
    failed(&format!("array index out of range (length {len})"))
}

/// `make-array` of a length below 0.
#[unsafe(no_mangle)]
pub extern "C" fn kc_make_array_fail() -> ! {
    failed("a bloblet's field count must be non-negative")
}

/// `string-ref` past its string's end, as Fixpt says it.
#[unsafe(no_mangle)]
pub extern "C" fn kc_string_index_fail(len: u64) -> ! {
    failed(&format!("string index out of range (length {len})"))
}

/// The collector (C453), called by kc's code (x64.kfx's `kc_gc_call`)
/// where an allocation of `need` bytes does not fit: its stack pointer
/// `sp`, its block `block` (the writable address of `kc_gc`). The heap
/// after, or the run ended.
#[unsafe(no_mangle)]
pub extern "C" fn kc_collect(need: u64, sp: u64, block: *mut u64) -> Heap {
    gc::collect(need, sp, block).unwrap_or_else(|m| failed(&m))
}

/// A built program's failure: said, as `fixpt eval` says one, and the run
/// ended.
pub fn failed(message: &str) -> ! {
    eprintln!("evaluation failed: {message}");
    std::process::exit(1)
}

// ------------------------------------------------------------ in process

std::thread_local! {
    /// What the last routine that failed in process said (C452).
    static FAILURE: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// A routine's failure in process: its words kept, and 0, which no value
/// these routines make is, for the code to go back to the trampoline.
fn failed_here(message: String) -> u64 {
    FAILURE.with(|f| *f.borrow_mut() = Some(message));
    0
}

/// What the last failure in process said, taken (`kayak:kc-failure/1`).
pub fn failure_taken() -> String {
    FAILURE.with(|f| f.borrow_mut().take()).unwrap_or_default()
}

std::thread_local! {
    /// What kc's code wrote in process (C454), each value's line: given
    /// to KFX (`kayak:kc-output/1`), which writes it (`kc run`) or reads
    /// its last line (a transformer's answer).
    static OUTPUT: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
    /// What a transformer's run in process is given (C454,
    /// `kayak:kc-give/1`): its expansion's tag and its use, as a built
    /// transformer's `--apply` reads them from its standard input.
    static GIVEN: std::cell::RefCell<(String, String)> = const { std::cell::RefCell::new((String::new(), String::new())) };
}

/// kc's code's output in process, taken.
pub fn output_taken() -> String {
    OUTPUT.with(|o| std::mem::take(&mut *o.borrow_mut()))
}

/// A transformer's input in process: the tag on its first line, the use
/// after.
pub fn give(input: &str) {
    let (tag, used) = input.split_once('\n').unwrap_or((input, ""));
    GIVEN.with(|g| *g.borrow_mut() = (tag.to_string(), used.to_string()));
}

extern "C" fn here_show(v: u64) {
    let line = written(v);
    OUTPUT.with(|o| {
        let mut o = o.borrow_mut();
        o.push_str(&line);
        o.push('\n');
    });
}

/// What the transformer is given (`give`): the use (0) or the tag (the
/// fixnum 1), a string in kc's heap (C455).
fn given_in(args: *const u64, block: *mut u64) -> Result<u64, String> {
    let s = GIVEN.with(|g| {
        let g = g.borrow();
        if arg(args, 0) == 0 {
            g.1.clone()
        } else {
            g.0.clone()
        }
    });
    string_in(block, &s)
}

#[unsafe(no_mangle)]
pub extern "C" fn kcrt_given(args: *const u64, block: *mut u64) -> u64 {
    given_in(args, block).unwrap_or_else(|m| failed(&m))
}

extern "C" fn here_given(args: *const u64, block: *mut u64) -> u64 {
    given_in(args, block).unwrap_or_else(failed_here)
}

extern "C" fn here_string_append(args: *const u64, block: *mut u64) -> u64 {
    string_appended(args, block).unwrap_or_else(failed_here)
}

/// A run's start in process (C454): the heap the code is given is the
/// first space again, the collector's own spaces dropped (the run
/// before's region, dropped whole, what this runtime made there with it,
/// C455).
extern "C" fn here_gc_reset() {
    gc::reset();
}

extern "C" fn here_fail(what: u64) -> u64 {
    failed_here(fail_words(what).to_string())
}

extern "C" fn here_array_index_fail(len: u64) -> u64 {
    failed_here(format!("array index out of range (length {len})"))
}

extern "C" fn here_make_array_fail() -> u64 {
    failed_here("a bloblet's field count must be non-negative".to_string())
}

extern "C" fn here_string_index_fail(len: u64) -> u64 {
    failed_here(format!("string index out of range (length {len})"))
}

extern "C" fn here_substring(args: *const u64, block: *mut u64) -> u64 {
    substring_of(args, block).unwrap_or_else(failed_here)
}

extern "C" fn here_collect(need: u64, sp: u64, block: *mut u64) -> Heap {
    gc::collect(need, sp, block).unwrap_or_else(|m| {
        failed_here(m);
        Heap { next: 0, end: 0 }
    })
}

extern "C" fn here_append(args: *const u64, block: *mut u64) -> u64 {
    appended(args, block).unwrap_or_else(failed_here)
}

/// The address of the routine kc's code names `name` (`kc_show`, ...), as
/// it is called in process (`kayak:kc-routine/1`): the routine itself, or,
/// of one that may fail, its variant that answers 0 instead of ending the
/// run; 0 if there is none of that name.
pub fn routine(name: &str) -> u64 {
    let f: *const () = match name {
        "kc_show" => here_show as *const (),
        "kcrt_given" => here_given as *const (),
        "kc_gc_reset" => here_gc_reset as *const (),
        "kc_fail" => here_fail as *const (),
        "kc_intern" => kc_intern as *const (),
        "kc_string_eq" => kc_string_eq as *const (),
        "kc_string_lt" => kc_string_lt as *const (),
        "kcrt_string_append" => here_string_append as *const (),
        "kcrt_substring" => here_substring as *const (),
        "kc_symbol_to_string" => kc_symbol_to_string as *const (),
        "kc_is_string" => kc_is_string as *const (),
        "kc_is_symbol" => kc_is_symbol as *const (),
        "kc_is_procedure" => kc_is_procedure as *const (),
        "kc_is_kind" => kc_is_kind as *const (),
        "kcrt_append" => here_append as *const (),
        "kc_array_index_fail" => here_array_index_fail as *const (),
        "kc_make_array_fail" => here_make_array_fail as *const (),
        "kc_string_index_fail" => here_string_index_fail as *const (),
        "kc_collect" => here_collect as *const (),
        "kcrt_capture" => here_capture as *const (),
        "kcrt_resume_copy" => here_resume_copy as *const (),
        "kcrt_current_marks" => here_current_marks as *const (),
        "kcrt_marks_of" => here_marks_of as *const (),
        _ => return 0,
    };
    f as usize as u64
}

/// A string as Fixpt writes one (`print.rs`'s `put_string`).
fn put_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\x{:x};", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
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
        // A bloblet, by its kind: a string, a symbol (C421); a closure
        // (C412), which Fixpt writes by its machine's name for it, this
        // kc's.
        TAG_BLOBLET => match kind(v) {
            KIND_STRING => {
                let mut out = String::new();
                put_string(&mut out, &string_text(v));
                out
            }
            KIND_SYMBOL => string_text(kc_symbol_to_string(v)),
            KIND_CLOSURE => "#<procedure>".into(),
            // A ref, Fixpt's box, by what it holds (C424).
            KIND_BOX => format!("#<box {}>", written(word(v - TAG_BLOBLET - 16))),
            // A sum, by its tag; a product, by how many fields (C423), as
            // Fixpt writes them: its tag field 2 back, a symbol.
            KIND_SUM => format!(
                "#<sum {}>",
                string_text(kc_symbol_to_string(word(v - TAG_BLOBLET - 16)))
            ),
            KIND_PRODUCT => format!("#<product of {}>", (word(v - TAG_BLOBLET - 8) >> 3) - 1),
            // An array (C422), as Fixpt writes a bloblet: its fields with its
            // trailer.
            KIND_BLOBLET => format!(
                "#<bloblet {} fields 0 bytes>",
                word(v - TAG_BLOBLET - 8) >> 3
            ),
            k => format!("#<object of kind {k}>"),
        },
        _ => format!("#<value {v:#x}>"),
    }
}

/// A failure of kc's code, by number (`kc/x64.kfx`'s `fail-…`), in
/// Fixpt's words.
pub fn fail_words(what: u64) -> &'static str {
    match what {
        1 => "not a Unicode scalar value",
        4 => "car or cdr of a non-pair",
        // kc's own: the collector's space past its most (C453).
        5 => "kc: the heap past its most (2^31 words)",
        // kc's own, not Fixpt's: its integers are fixnums alone, as yet.
        2 => "kc: not yet: an integer past a fixnum",
        3 => "division by zero",
        // kc's own: the checker finds every tag a sum may have its arm.
        7 => "kc: a sum no arm of its tagcase is for",
        // The stack past its limit (C451), as kayak-rt says Fixpt's trap.
        8 => "stack overflow",
        // kc's own: a transformer's program run with no transformer (C448).
        9 => "kc: this program has no transformer to apply",
        // An abort no prompt of its tag delimits (C458), as Fixpt's native
        // machines say it.
        10 => "abort: no prompt for this tag",
        // A capture no prompt of its tag delimits (C459), as Fixpt's native
        // machines say it.
        11 => "call-with-composable-continuation: no prompt for this tag",
        // A one-shot continuation resumed again (C459), FX-26's words.
        12 => "a one-shot continuation resumed twice",
        _ => "a failure kc's runtime has no words for",
    }
}

/// A failure of kc's code, said as `fixpt eval` says a failure, and the
/// run ended.
#[unsafe(no_mangle)]
pub extern "C" fn kc_fail(what: u64) -> ! {
    failed(fail_words(what))
}

/// A top-level expression's value, written on a line of its own, as
/// `fixpt eval` writes each (C407).
#[unsafe(no_mangle)]
pub extern "C" fn kc_show(v: u64) {
    println!("{}", written(v));
}
