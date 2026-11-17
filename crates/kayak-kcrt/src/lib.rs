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
//! its global slot. What kc's code makes of them past a literal (an
//! append, a symbol interned) is made here, in this runtime's own memory:
//! no collector yet, so nothing is freed.
//!
//! kc's code is called through `extern "C"`: the one place here that is
//! not safe Rust.
#![allow(unsafe_code)]

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

unsafe extern "C" {
    fn kc_entry(heap: *mut u64, end: *mut u64) -> u64;
    /// A procedure of kc's code (its address `f`) called on `a` and `b`,
    /// the heap's next word where `kc_entry` left it, its end `end`
    /// (C448: kc's entry object's).
    fn kc_call2(f: u64, a: u64, b: u64, end: u64) -> u64;
    /// `%phase1-apply`'s address, if the program defines it, else 0.
    static kc_apply_entry: u64;
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

/// Words this runtime makes, as long as the run.
fn made(words: Vec<u64>) -> u64 {
    Vec::leak(words).as_ptr() as u64
}

/// A string of `s`, as Fixpt lays one out: a header of kind 1, its words
/// all suffix; the length; the code points, two a word.
fn make_string(s: &str) -> u64 {
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
    made(w) + 8 + TAG_BLOBLET
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

#[unsafe(no_mangle)]
pub extern "C" fn kc_string_append(a: u64, b: u64) -> u64 {
    make_string(&(string_text(a) + &string_text(b)))
}

#[unsafe(no_mangle)]
pub extern "C" fn kc_substring(s: u64, i: u64, j: u64) -> u64 {
    let cs: Vec<char> = string_text(s).chars().collect();
    let (i, j) = ((i as i64) >> 3, (j as i64) >> 3);
    if i < 0 || j < i || j as usize > cs.len() {
        failed("substring range out of bounds")
    }
    make_string(&cs[i as usize..j as usize].iter().collect::<String>())
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
/// this runtime's memory; `a` not a list fails as `cdr` of a non-pair does.
#[unsafe(no_mangle)]
pub extern "C" fn kc_append(a: u64, b: u64) -> u64 {
    let mut cars = Vec::new();
    let mut rest = a;
    while rest != null() {
        if rest & TAG_MASK != TAG_PAIR {
            kc_fail(4);
        }
        let (car, cdr) = pair(rest);
        cars.push(car);
        rest = cdr;
    }
    cars.iter()
        .rev()
        .fold(b, |tail, &car| made(vec![car, tail]) + TAG_PAIR)
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

fn failed(message: &str) -> ! {
    eprintln!("evaluation failed: {message}");
    std::process::exit(1)
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
        // kc's own: the checker finds every tag a sum may have its arm.
        7 => "kc: a sum no arm of its tagcase is for",
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
    // `--apply` (C448): a procedural macro's transformer's program, given a
    // use on the standard input, the expansion's tag on its first line and
    // the use's text after; the program's definitions made, then its
    // `%phase1-apply` called on the two, and what it gives written.
    let apply = std::env::args().nth(1).as_deref() == Some("--apply");
    let mut input = String::new();
    if apply {
        use std::io::Read as _;
        if std::io::stdin().read_to_string(&mut input).is_err() {
            failed("kc: the use given is not text");
        }
    }
    let heap: &'static mut [u64] = Vec::leak(vec![0u64; HEAP_WORDS]);
    let range = heap.as_mut_ptr_range();
    // SAFETY: `kc_entry` is kc's code, which takes the heap's first free
    // word and its end, and writes each top-level expression's value.
    unsafe { kc_entry(range.start, range.end) };
    if apply {
        let (tag, used) = input.split_once('\n').unwrap_or((input.as_str(), ""));
        // SAFETY: kc's entry object defines it, a word.
        let f = unsafe { kc_apply_entry };
        if f == 0 {
            failed("kc: this program has no transformer to apply");
        }
        // SAFETY: `f` is a procedure of kc's code of two arguments, strings
        // of Fixpt's layout; `kc_call2` gives it the heap `kc_entry` left.
        let v = unsafe { kc_call2(f, make_string(used), make_string(tag), range.end as u64) };
        println!("{}", written(v));
    }
    0
}
