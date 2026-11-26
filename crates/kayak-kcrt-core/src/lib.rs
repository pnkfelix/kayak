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
//! own heap, which is collected (C455; since C541 the shared heap's,
//! `heap.rs`): a routine that makes one is given its arguments where a
//! collection finds them. A symbol is
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

mod heap;
pub use heap::{B_CELLS_HI, B_CELLS_LO, B_HI, B_NEXT, Heap, collections, done};

// Fixpt's tags, immediates and kinds, from kayak-layout's table, Fixpt's
// own (TODO §56, step 1), under this crate's names.
use kayak_layout as layout;
const TAG_MASK: u64 = layout::TAG_MASK;
const TAG_FIXNUM: u64 = layout::tag("fixnum");
const TAG_PAIR: u64 = layout::tag("pair");
const TAG_IMMEDIATE: u64 = layout::tag("immediate");
const TAG_BLOBLET: u64 = layout::tag("bloblet");
const IMM_SHIFT: u32 = layout::TAG_BITS;
const IMM_MASK: u64 = 0b1_1111;
const IMM_PAYLOAD_SHIFT: u32 =
    (Value::char('\u{1}').raw() ^ Value::char('\0').raw()).trailing_zeros();
use kayak_heap::write::{put_char, put_string};
use layout::value::Value;
/// An immediate's subtag, as Fixpt's `Value` makes its word.
const fn subtag(v: Value) -> u64 {
    (v.raw() >> IMM_SHIFT) & IMM_MASK
}
const IMM_FALSE: u64 = subtag(Value::FALSE);
const IMM_TRUE: u64 = subtag(Value::TRUE);
const IMM_NULL: u64 = subtag(Value::NULL);
const IMM_UNIT: u64 = subtag(Value::UNIT);
const IMM_CHAR: u64 = subtag(Value::char('\0'));

/// A pair's car and cdr.
fn pair(v: u64) -> (u64, u64) {
    let p = (v - TAG_PAIR) as *const u64;
    // SAFETY: a pair's value is its cell's address, tag 1, a cell of two
    // words kc's code made on the heap, which lives as long as the run.
    unsafe { (*p, *p.add(1)) }
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
    // A vector (C470), as Fixpt writes one: `#(`, its elements, `)`.
    if v & TAG_MASK == TAG_BLOBLET && kind(v) == KIND_VECTOR {
        out.push_str("#(");
        let n = vector_length(v);
        for k in 0..n {
            if k > 0 {
                out.push(' ');
            }
            put(out, word(v - TAG_BLOBLET - 8 * (n + 1) + 8 * k));
        }
        out.push(')');
        return;
    }
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

const KIND_STRING: u64 = layout::kind("string") as u64;
const KIND_SYMBOL: u64 = layout::kind("symbol") as u64;
const KIND_CLOSURE: u64 = layout::kind("closure") as u64;
const KIND_BOX: u64 = layout::kind("box") as u64;
const KIND_BLOBLET: u64 = layout::kind("bloblet") as u64;
const KIND_SUM: u64 = layout::kind("sum") as u64;
const KIND_PRODUCT: u64 = layout::kind("product") as u64;
const TAG_HEADER: u64 = layout::tag("header");
const TAG_TRAILER: u64 = layout::tag("trailer");

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

#[cfg(not(target_endian = "little"))]
compile_error!(
    "a string's code points are read as one `[u32]`: memory's order on a little-endian machine alone"
);

/// A string's code points (C536): its suffix's count, then the points,
/// two a word, the first in the low half, so one `[u32]` in memory.
fn string_points(v: u64) -> &'static [u32] {
    let p = v - TAG_BLOBLET;
    let n = word(p) as usize;
    // SAFETY: a string's suffix, its count then `n` points, within its
    // object, which nothing writes and which lives while kc's code holds
    // it (a collection moves it only where no routine is reading it).
    unsafe { std::slice::from_raw_parts((p + 8) as *const u32, n) }
}

/// A string's text, for what Rust takes as text (a failure's message, a
/// symbol's name).
fn string_text(v: u64) -> String {
    string_points(v)
        .iter()
        .map(|&c| char::from_u32(c).unwrap_or('\u{fffd}'))
        .collect()
}

/// String `ps`'s words written at `at`, room `heap::words` gave for them.
fn write_string(at: u64, ps: &[u32]) -> u64 {
    for (k, &x) in string_words(ps).iter().enumerate() {
        set(at + 8 * k as u64, x);
    }
    at + 8 + TAG_BLOBLET
}

/// Words this runtime makes, as long as the run: a symbol and its name.
fn made(words: Vec<u64>) -> u64 {
    Vec::leak(words).as_ptr() as u64
}

fn set(addr: u64, w: u64) {
    // SAFETY: a word of room `heap::words` gave in kc's heap, live and
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
    made(string_words(&points_of(s))) + 8 + TAG_BLOBLET
}

/// A Rust text's code points.
fn points_of(s: &str) -> Vec<u32> {
    s.chars().map(|c| c as u32).collect()
}

/// A string of `s` in kc's heap (C455), its block's.
fn string_in(block: *mut u64, s: &str) -> Result<u64, String> {
    points_in(block, &points_of(s))
}

/// A string of code points `ps` in kc's heap, its block's: written there
/// whole, its words kayak-rt's strings' (`kayak_heap::text`, C536).
fn points_in(block: *mut u64, ps: &[u32]) -> Result<u64, String> {
    let w = string_words(ps);
    let at = heap::words(block, w.len() as u64)?;
    for (k, &x) in w.iter().enumerate() {
        set(at + 8 * k as u64, x);
    }
    Ok(at + 8 + TAG_BLOBLET)
}

/// A string's words, as Fixpt lays one out (`kayak_heap::text`): a header
/// of kind 1, its words all suffix; the count; the code points, two a
/// word.
fn string_words(ps: &[u32]) -> Vec<u64> {
    let mut w = vec![0; kayak_heap::text::string_words(ps.len())];
    kayak_heap::text::fill_string(ps, &mut w);
    w
}

/// The symbols, by name: the process's, one table every thread interns
/// in (C467, TODO §55 item 3; K8), so that a name is one symbol whichever
/// thread made it, as `symbol=?` (identity) and FX-26's processes, which
/// send symbols, need, and as the image's cells of quoted symbols
/// (`kcy_…`, x64.kfx's `x64-emit-symbol`), which every thread running it
/// reads, assume. It was a thread's own. A symbol is made under the lock,
/// so two threads interning one name at once get one. Interning is rare
/// (once per quoted name per image, and each `string->symbol`), so a lock,
/// not yet a cache per thread. Symbols live in kcrt's own memory, outside
/// kc's heap, never moved; into the shared heap with §55 item 2.
static SYMBOLS: std::sync::Mutex<Option<std::collections::HashMap<String, u64>>> =
    std::sync::Mutex::new(None);

/// A symbol of the name string `s`, the one of that name: as Fixpt's
/// `intern` makes one, its fields its name, its hash (FNV-1a, a fixnum) and
/// a global slot (none here, -1), and a trailer.
#[unsafe(no_mangle)]
pub extern "C" fn kc_intern(s: u64) -> u64 {
    let name = string_text(s);
    let mut table = SYMBOLS.lock().unwrap_or_else(|e| e.into_inner());
    let t = table.get_or_insert_with(std::collections::HashMap::new);
    {
        if let Some(&v) = t.get(&name) {
            return v;
        }
        // kayak-rt's tags' too (`kayak_heap::text`, C536), a fixnum.
        let hash = (kayak_heap::text::symbol_hash(&name) << 3) as u64;
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
        t.insert(name, v);
        v
    }
}

#[cfg(test)]
mod symbols_shared {
    use super::*;

    /// One name, one symbol, whichever thread interns it (C467, TODO §55
    /// item 3): eight threads interning the same names at once.
    #[test]
    fn a_name_is_one_symbol_across_threads() {
        let names: Vec<String> = (0..64).map(|k| format!("sym-{k}")).collect();
        let seen: Vec<Vec<u64>> = std::thread::scope(|sc| {
            let hs: Vec<_> = (0..8)
                .map(|_| {
                    sc.spawn(|| {
                        names
                            .iter()
                            .map(|n| kc_intern(make_string(n)))
                            .collect::<Vec<u64>>()
                    })
                })
                .collect();
            hs.into_iter().map(|h| h.join().unwrap()).collect()
        });
        for s in &seen[1..] {
            assert_eq!(s, &seen[0]);
        }
        // And each its own name's.
        for (n, &v) in names.iter().zip(&seen[0]) {
            assert_eq!(&string_text(kc_symbol_to_string(v)), n);
        }
    }
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
    boolean(string_points(a) == string_points(b))
}

#[unsafe(no_mangle)]
pub extern "C" fn kc_string_lt(a: u64, b: u64) -> u64 {
    boolean(string_points(a) < string_points(b))
}

fn string_appended(args: *const u64, block: *mut u64) -> Result<u64, String> {
    // Room first, its arguments read after (the collector may move them).
    let n = string_points(arg(args, 0)).len() + string_points(arg(args, 1)).len();
    let at = heap::words(block, kayak_heap::text::string_words(n) as u64)?;
    let ps: Vec<u32> = [string_points(arg(args, 0)), string_points(arg(args, 1))].concat();
    Ok(write_string(at, &ps))
}

/// `string-append` of its two arguments, in kc's heap (C455), as each
/// routine that makes something there is called: its arguments' stack
/// words and the block (x64.kfx's `x64-heap-routine`).
#[unsafe(no_mangle)]
pub extern "C" fn kcrt_string_append(args: *const u64, block: *mut u64) -> u64 {
    string_appended(args, block).unwrap_or_else(|m| failed(&m))
}

fn substring_of(args: *const u64, block: *mut u64) -> Result<u64, String> {
    let n = string_points(arg(args, 0)).len();
    let (i, j) = ((arg(args, 1) as i64) >> 3, (arg(args, 2) as i64) >> 3);
    if i < 0 || j < i || j as usize > n {
        return Err("substring range out of bounds".to_string());
    }
    let at = heap::words(
        block,
        kayak_heap::text::string_words((j - i) as usize) as u64,
    )?;
    let ps = string_points(arg(args, 0))[i as usize..j as usize].to_vec();
    Ok(write_string(at, &ps))
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

/// A bytevector's length (C529), its suffix's first word (Fixpt's, C534),
/// a fixnum.
#[unsafe(no_mangle)]
pub extern "C" fn kc_bytevector_length(v: u64) -> u64 {
    word(v - TAG_BLOBLET) << 3
}

/// Bytevector `v`'s byte at fixnum `i`, plus one, a fixnum (kc's code
/// takes the one off), or the index's failure: answered plus one so that
/// in process a failure's 0 is no byte's.
fn bytevector_u8_ref(v: u64, i: u64) -> Result<u64, String> {
    let n = word(v - TAG_BLOBLET);
    let k = (i as i64) >> 3;
    if i & TAG_MASK != TAG_FIXNUM || k < 0 || k as u64 >= n {
        return Err(format!("bytevector index out of range (length {n})"));
    }
    // SAFETY: a bytevector's suffix, its length then its `n` bytes,
    // within its object, alive for the run.
    let b = unsafe { *((v - TAG_BLOBLET + 8 + k as u64) as *const u8) };
    Ok((b as u64 + 1) << 3)
}

#[unsafe(no_mangle)]
pub extern "C" fn kcrt_bytevector_u8_ref(v: u64, i: u64) -> u64 {
    bytevector_u8_ref(v, i).unwrap_or_else(|m| failed(&m))
}

extern "C" fn here_bytevector_u8_ref(v: u64, i: u64) -> u64 {
    bytevector_u8_ref(v, i).unwrap_or_else(failed_here)
}

const KIND_UTF8: u64 = layout::kind("utf8") as u64;
const KIND_UTF8_ASCII: u64 = layout::kind("utf8-ascii") as u64;
const KIND_UTF8_SLICE: u64 = layout::kind("utf8-slice") as u64;

/// Whether `v` is a `utf8`, whole (either kind) or a slice, as Fixpt's
/// `Heap::is_utf8` says.
#[unsafe(no_mangle)]
pub extern "C" fn kc_is_utf8(v: u64) -> u64 {
    boolean(
        v & TAG_MASK == TAG_BLOBLET
            && matches!(kind(v), KIND_UTF8 | KIND_UTF8_ASCII | KIND_UTF8_SLICE),
    )
}

/// A `utf8`'s text: a whole one's suffix, its header's count of bytes; a
/// slice's piece of its owner's suffix (`layout::utf8_slice`, fields by
/// negative offset from the slice's empty suffix).
fn utf8_text(v: u64) -> String {
    let (at, len) = if kind(v) == KIND_UTF8_SLICE {
        let p = v - TAG_BLOBLET;
        let field = |k: usize| word(p - 8 * k as u64);
        let (owner, start, end) = (
            field(layout::utf8_slice::OWNER),
            field(layout::utf8_slice::START) >> 3,
            field(layout::utf8_slice::END) >> 3,
        );
        (owner - TAG_BLOBLET + start, end - start)
    } else {
        (v - TAG_BLOBLET, layout::H_BYTES.get(header(v)))
    };
    // SAFETY: a utf8's bytes, `len` of them from `at`, within its object
    // (or its owner's), alive for the run, valid UTF-8 as it was made.
    let bytes = unsafe { std::slice::from_raw_parts(at as *const u8, len as usize) };
    String::from_utf8_lossy(bytes).into_owned()
}

/// `(utf8->string u)`: its text, a string in kc's heap.
fn utf8_string(args: *const u64, block: *mut u64) -> Result<u64, String> {
    string_in(block, &utf8_text(arg(args, 0)))
}

#[unsafe(no_mangle)]
pub extern "C" fn kcrt_utf8_to_string(args: *const u64, block: *mut u64) -> u64 {
    utf8_string(args, block).unwrap_or_else(|m| failed(&m))
}

extern "C" fn here_utf8_to_string(args: *const u64, block: *mut u64) -> u64 {
    utf8_string(args, block).unwrap_or_else(failed_here)
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
    let at = heap::words(block, 2 * n)?;
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

/// `(reverse l)` (C462), as Fixpt's: a new list of `l`'s elements, last
/// first, in kc's heap, the room for it had at once and `l` read again
/// after; `l` not a list fails as `cdr` of a non-pair does.
fn reversed(args: *const u64, block: *mut u64) -> Result<u64, String> {
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
        return Ok(null());
    }
    let at = heap::words(block, 2 * n)?;
    let mut rest = arg(args, 0);
    // The first element the last pair's, each next one the pair before.
    let mut head = null();
    for k in (0..n).rev() {
        let (car, cdr) = pair(rest);
        set(at + 16 * k, car);
        set(at + 16 * k + 8, head);
        head = at + 16 * k + TAG_PAIR;
        rest = cdr;
    }
    Ok(head)
}

#[unsafe(no_mangle)]
pub extern "C" fn kcrt_reverse(args: *const u64, block: *mut u64) -> u64 {
    reversed(args, block).unwrap_or_else(|m| failed(&m))
}

/// `(int->string n)` (C462), Fixpt's `number->string` of a fixnum: its
/// decimal digits, a `-` before a negative one's.
fn int_string(args: *const u64, block: *mut u64) -> Result<u64, String> {
    string_in(block, &((arg(args, 0) as i64) >> 3).to_string())
}

#[unsafe(no_mangle)]
pub extern "C" fn kcrt_int_to_string(args: *const u64, block: *mut u64) -> u64 {
    int_string(args, block).unwrap_or_else(|m| failed(&m))
}

// ------------------------------------------------- the expander's rest (C470)

/// A character's code point (an immediate, its payload from bit 8).
fn char_of(v: u64) -> char {
    char::from_u32((v >> IMM_PAYLOAD_SHIFT) as u32).unwrap_or('\u{fffd}')
}

/// A fixnum of `n`.
fn fixnum(n: i64) -> u64 {
    (n << 3) as u64
}

/// `(char->string c)`: a string of the one character.
fn char_string(args: *const u64, block: *mut u64) -> Result<u64, String> {
    string_in(block, &char_of(arg(args, 0)).to_string())
}

#[unsafe(no_mangle)]
pub extern "C" fn kcrt_char_to_string(args: *const u64, block: *mut u64) -> u64 {
    char_string(args, block).unwrap_or_else(|m| failed(&m))
}

/// `(char-whitespace? c)` and `(char-numeric? c)`, as Fixpt's: Rust's
/// Unicode `is_whitespace` and `is_numeric` (so `½` is numeric).
#[unsafe(no_mangle)]
pub extern "C" fn kc_char_whitespace(v: u64) -> u64 {
    boolean(char_of(v).is_whitespace())
}

#[unsafe(no_mangle)]
pub extern "C" fn kc_char_numeric(v: u64) -> u64 {
    boolean(char_of(v).is_numeric())
}

/// `(string-hash s)`, Fixpt's `%string-hash` (`Heap::string_fnv`): FNV-1a
/// of the code points, each a `u64`, shifted right 4 into a fixnum. Of the
/// contents alone, never the address (TODO §55: the heap may move).
#[unsafe(no_mangle)]
pub extern "C" fn kc_string_hash(v: u64) -> u64 {
    fixnum(kayak_heap::text::string_hash(string_points(v)))
}

/// `(symbol-name-hash s)`, Fixpt's `%symbol-hash`: the hash kept with the
/// symbol when it was interned (its field 1, `kc_intern`'s: FNV-1a of the
/// name's UTF-8 bytes, its top bit cleared, wrapped to a fixnum).
#[unsafe(no_mangle)]
pub extern "C" fn kc_symbol_hash(v: u64) -> u64 {
    word(v - TAG_BLOBLET - 40 + 16)
}

const KIND_VECTOR: u64 = layout::kind("vector") as u64;

/// A vector's element count: its header's fields, less the trailer.
fn vector_length(v: u64) -> u64 {
    ((header(v) >> 14) & MOST_FIELDS).saturating_sub(1)
}
const KIND_FLONUM: u64 = layout::kind("flonum") as u64;

/// A flonum's value: a bloblet of kind 5, its suffix the bits (Fixpt's).
fn f64_string(args: *const u64, block: *mut u64) -> Result<u64, String> {
    let v = arg(args, 0);
    if v & TAG_MASK != TAG_BLOBLET || kind(v) != KIND_FLONUM {
        return Err("kc: f64->string of a value not an f64".to_string());
    }
    string_in(
        block,
        &kayak_heap::write::f64_text(f64::from_bits(word(v - TAG_BLOBLET))),
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn kcrt_f64_to_string(args: *const u64, block: *mut u64) -> u64 {
    f64_string(args, block).unwrap_or_else(|m| failed(&m))
}

/// `(datum-list->vector l)`, Fixpt's `list->vector`: a vector (kind 3) of
/// `l`'s elements, laid out as Fixpt's `alloc` lays one: the header, the
/// elements, a trailer, the value past it; none, a header alone. The room
/// had at once and `l` read again after; `l` not a list fails as `cdr` of
/// a non-pair does.
fn list_vector(args: *const u64, block: *mut u64) -> Result<u64, String> {
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
        let at = heap::words(block, 1)?;
        set(at, TAG_HEADER | (KIND_VECTOR << 3));
        return Ok(at + 8 + TAG_BLOBLET);
    }
    if n + 1 > MOST_FIELDS {
        return Err("kc: not yet: a vector of more than 262142 elements".to_string());
    }
    let at = heap::words(block, n + 2)?;
    set(at, TAG_HEADER | (KIND_VECTOR << 3) | ((n + 1) << 14));
    let mut rest = arg(args, 0);
    for k in 0..n {
        let (car, cdr) = pair(rest);
        set(at + 8 * (k + 1), car);
        rest = cdr;
    }
    set(at + 8 * (n + 1), TAG_TRAILER | ((n + 1) << 3));
    Ok(at + 8 * (n + 2) + TAG_BLOBLET)
}

#[unsafe(no_mangle)]
pub extern "C" fn kcrt_list_to_vector(args: *const u64, block: *mut u64) -> u64 {
    list_vector(args, block).unwrap_or_else(|m| failed(&m))
}

/// `(string-search s sub from)` (C474), Fixpt's (`Heap::string_search`):
/// where `sub` is first in `s` at or after `from` (taken as 0 if below it),
/// by code points, as a fixnum; -1 if it is not.
#[unsafe(no_mangle)]
pub extern "C" fn kc_string_search(s: u64, sub: u64, from: u64) -> u64 {
    let from = ((from as i64) >> 3).max(0) as usize;
    let found = kayak_heap::text::search(string_points(s), string_points(sub), from);
    fixnum(found.map_or(-1, |i| i as i64))
}

/// `(error message)`, FX-26's: the run fails, saying the message.
#[unsafe(no_mangle)]
pub extern "C" fn kc_error(v: u64) -> u64 {
    failed(&string_text(v))
}

// ------------------------------------------------------------ control

/// A prompt's record (C458, x64.kfx's `x64-emit-prompt`): the prompt
/// before it, its tag, its frame's stack pointer, its handler's call.
const R_PREV: u64 = 0;
const R_TAG: u64 = 8;
const R_SP: u64 = 16;
/// An offset of no record, in a continuation's copy: past its segment.
const NO_RECORD: u64 = (-8i64) as u64;
const KIND_ARRAY: u64 = layout::kind("bloblet") as u64;
const ONCE: u64 = 8;
const RESUMED: u64 = 16;
const WHOLE: u64 = 32;
/// A mark's value in its record (C460): where a prompt's handler is.
const R_VALUE: u64 = 24;
/// The most fields a header holds (Fixpt's 18 bits; kc makes no large one).
const MOST_FIELDS: u64 = layout::H_FIELDS.max();

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
    let room = heap::words(block, fields + 2 + 5)?;
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
    let at = heap::words(block, 2 * n)?;
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

/// Room in the heap (C453; C541, the shared heap's), asked by kc's code
/// (x64.kfx's `kc_gc_call`) where an allocation of `need` bytes does not fit: its stack pointer
/// `sp`, its block `block` (the writable address of `kc_gc`). The heap
/// after, or the run ended.
#[unsafe(no_mangle)]
pub extern "C" fn kc_collect(need: u64, sp: u64, block: *mut u64) -> Heap {
    heap::collect(need, sp, block).unwrap_or_else(|m| failed(&m))
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
/// kc's code's write barrier (C538): after each store of a value into a
/// heap object (`set`, `array-set!`), the slot's address, the heap's
/// barrier's (C541: a built program's heap has generations too).
#[unsafe(no_mangle)]
pub extern "C" fn kc_remember(slot: u64) {
    heap::remember(slot);
}

extern "C" fn here_remember(slot: u64) {
    heap::remember(slot);
}

extern "C" fn here_gc_reset() {
    heap::reset();
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
    heap::collect(need, sp, block).unwrap_or_else(|m| {
        failed_here(m);
        Heap { next: 0, end: 0 }
    })
}

extern "C" fn here_append(args: *const u64, block: *mut u64) -> u64 {
    appended(args, block).unwrap_or_else(failed_here)
}

extern "C" fn here_reverse(args: *const u64, block: *mut u64) -> u64 {
    reversed(args, block).unwrap_or_else(failed_here)
}

extern "C" fn here_int_to_string(args: *const u64, block: *mut u64) -> u64 {
    int_string(args, block).unwrap_or_else(failed_here)
}

extern "C" fn here_char_to_string(args: *const u64, block: *mut u64) -> u64 {
    char_string(args, block).unwrap_or_else(failed_here)
}

extern "C" fn here_f64_to_string(args: *const u64, block: *mut u64) -> u64 {
    f64_string(args, block).unwrap_or_else(failed_here)
}

extern "C" fn here_list_to_vector(args: *const u64, block: *mut u64) -> u64 {
    list_vector(args, block).unwrap_or_else(failed_here)
}

extern "C" fn here_error(v: u64) -> u64 {
    failed_here(string_text(v))
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
        "kc_remember" => here_remember as *const (),
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
        "kcrt_reverse" => here_reverse as *const (),
        "kcrt_int_to_string" => here_int_to_string as *const (),
        "kcrt_char_to_string" => here_char_to_string as *const (),
        "kc_char_whitespace" => kc_char_whitespace as *const (),
        "kc_char_numeric" => kc_char_numeric as *const (),
        "kc_string_hash" => kc_string_hash as *const (),
        "kc_symbol_hash" => kc_symbol_hash as *const (),
        "kcrt_f64_to_string" => here_f64_to_string as *const (),
        "kcrt_list_to_vector" => here_list_to_vector as *const (),
        "kc_error" => here_error as *const (),
        "kc_string_search" => kc_string_search as *const (),
        "kc_bytevector_length" => kc_bytevector_length as *const (),
        "kcrt_bytevector_u8_ref" => here_bytevector_u8_ref as *const (),
        "kc_is_utf8" => kc_is_utf8 as *const (),
        "kcrt_utf8_to_string" => here_utf8_to_string as *const (),
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

#[cfg(test)]
mod layout_tests {
    use super::*;

    /// The constants drawn from Fixpt's table (TODO §56, step 1) are those
    /// this crate wrote out before.
    #[test]
    fn the_table_gives_the_values_written_before() {
        assert_eq!(
            (
                TAG_MASK,
                TAG_FIXNUM,
                TAG_PAIR,
                TAG_IMMEDIATE,
                TAG_BLOBLET,
                TAG_TRAILER,
                TAG_HEADER
            ),
            (7, 0, 1, 3, 4, 5, 6)
        );
        assert_eq!((IMM_SHIFT, IMM_MASK, IMM_PAYLOAD_SHIFT), (3, 0b1_1111, 8));
        assert_eq!(
            (IMM_FALSE, IMM_TRUE, IMM_NULL, IMM_UNIT, IMM_CHAR),
            (0, 1, 2, 3, 8)
        );
        assert_eq!(
            (
                KIND_STRING,
                KIND_SYMBOL,
                KIND_VECTOR,
                KIND_FLONUM,
                KIND_CLOSURE,
                KIND_BOX
            ),
            (1, 2, 3, 5, 8, 10)
        );
        assert_eq!(
            (
                KIND_BLOBLET,
                KIND_ARRAY,
                KIND_SUM,
                KIND_PRODUCT,
                MOST_FIELDS
            ),
            (32, 32, 36, 37, 0x3_ffff)
        );
    }
}

#[cfg(test)]
mod strings_as_points {
    use super::*;

    /// kc's string routines over code points (C536): equal, ordered,
    /// hashed and searched as kayak-rt's strings are (`kayak_heap::text`).
    #[test]
    fn compared_hashed_searched() {
        let (a, b, c) = (
            make_string("héllo"),
            make_string("héllo"),
            make_string("hëllo"),
        );
        assert_eq!(kc_string_eq(a, b), boolean(true));
        assert_eq!(kc_string_eq(a, c), boolean(false));
        assert_eq!(kc_string_lt(a, c), boolean(true));
        assert_eq!(kc_string_lt(c, a), boolean(false));
        assert_eq!(kc_string_hash(a), kc_string_hash(b));
        assert_eq!(
            kc_string_hash(a),
            fixnum(kayak_heap::text::string_hash(&points_of("héllo")))
        );
        assert_eq!(kc_string_search(a, make_string("l"), fixnum(0)), fixnum(2));
        assert_eq!(kc_string_search(a, make_string("l"), fixnum(4)), fixnum(-1));
        assert_eq!(string_text(a), "héllo");
    }
}
