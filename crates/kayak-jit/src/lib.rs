//! Machine code a program made, placed and run (`docs/research/native.md`,
//! decision 4; `CLAUDES_CHOICES.md` C116): FX-26's `seal-code` and
//! `native-procedure` (Fixpt's TODO §97), under Fixpt's native convention
//! (`(native-convention)`, Fixpt's `native-arm64.conv`, or on x86_64 its
//! `native-x86_64.conv`).
//!
//! - **The code area**: code bloblets, laid out as Fixpt's (a header, the
//!   fields, a trailer, the suffix), mapped twice, as Fixpt's is
//!   (`docs/object-model.md`, "A collected code area"): written through a
//!   read+write view, run from a read+execute view of the same pages, at a
//!   fixed distance, so nothing is ever both and nothing is re-protected.
//!   Nothing in it is ever freed, as nothing of kayak's heap is (C116).
//! - **Objects in Fixpt's layout** that code reads and makes (closures,
//!   pairs), in memory of their own, never freed either.
//! - **The trampoline**: from Rust into the convention (on arm64 the
//!   state in `x24`, fuel `x28`, the stack's limit `x27`, the code's
//!   distance `x26`, the closure `x10`, the arguments `x1`–`x8`, their
//!   count `x9`; on x86_64 the state in `r15`, and fuel, the limit and the
//!   distance in it, the closure `rbx`, the arguments `rdi`, `rsi`, `r8`
//!   and the state's `reg4`–`reg8`, their count `r10`) and back, every
//!   register Rust keeps kept; nested, as native code calls Rust that
//!   calls native code.
//! - **The routines** the convention declares (`routines`), at the state's
//!   fields: `poll` gives the next slice of fuel; `overflow` and `trap` end
//!   the call in a failure; `cons` makes a pair; `foreign` calls what is
//!   not native code, a Rust procedure here ([`foreign_procedure`]), whose
//!   panic goes back through the native frames by the way the trampoline
//!   left and is resumed beyond. `bloblet` and `closure` are not kayak's
//!   yet: they end the call, naming themselves.
//!
//! Only on arm64 macOS and x86_64 Linux; elsewhere [`native_code`] is
//! false, and FX-26's `native-code?` with it.
//!
//! Ported from Fixpt's object model (C121; `scripts/fixpt-drift.py`); its
//! constants are tested against Fixpt's own (`tests/layout.rs`,
//! `tests/declaration.rs`):
//! fixpt-source: docs/object-model.md@7641962

use std::cell::Cell;
use std::sync::{Mutex, OnceLock};

/// The native convention kayak keeps on this machine, declared: one clause
/// a line, Fixpt's at the pin (`scripts/built-in.py` copies both), which
/// the constants below are tested against (`tests/declaration.rs`):
/// x86_64's on x86_64, arm64's everywhere else, as Fixpt chooses.
#[cfg(target_arch = "x86_64")]
pub const DECLARATION: &str = include_str!("native-x86_64.conv");
#[cfg(not(target_arch = "x86_64"))]
pub const DECLARATION: &str = include_str!("native-arm64.conv");

/// Fixpt's value layout, as its native convention declares it (`values`).
pub mod layout {
    pub const TAG_MASK: u64 = 0b111;
    pub const TAG_FIXNUM: u64 = 0;
    pub const TAG_IMMEDIATE: u64 = 3;
    pub const TAG_BLOBLET: u64 = 4;
    pub const TAG_TRAILER: u64 = 5;
    pub const TAG_HEADER: u64 = 6;
    pub const FIXNUM_SHIFT: u32 = 3;
    pub const FIXNUM_BITS: u32 = 61;
    pub const FALSE: u64 = 3;
    pub const TRUE: u64 = 11;
    pub const NULL: u64 = 19;
    pub const UNIT: u64 = 27;
    /// A character: its code point above bit 8, subtag 8.
    pub const CHAR_SUBTAG: u64 = 8;
    pub const CHAR_SHIFT: u32 = 8;
    pub const TAG_PAIR: u64 = 1;
    pub const KIND_STRING: u8 = 1;
    pub const KIND_BLOBLET: u8 = 32;
    pub const KIND_CELLULAR_CLOSURE: u8 = 38;
    pub const KIND_NATIVE_CLOSURE: u8 = 41;
    pub const KIND_FLAT_ARRAY: u8 = 42;
    /// A flat array's element code for `i64` (Fixpt's `FLAT_I64`), its one
    /// field, slot 2, beside its trailer.
    pub const FLAT_I64: u64 = 2;
    pub const FLAT_CODE: usize = 2;
    /// A native closure's fields, numbered as slots: the trailer is 1.
    pub const CLOSURE_CODE: usize = 2;

    /// A header: kind bits 3–10, fields frozen 12, suffix frozen 13, the
    /// count of fields (the trailer one) 14–31, the suffix's bytes 32–63.
    pub const fn header(
        kind: u8,
        fields: usize,
        bytes: usize,
        fields_frozen: bool,
        suffix_frozen: bool,
    ) -> u64 {
        TAG_HEADER
            | (kind as u64) << 3
            | (fields_frozen as u64) << 12
            | (suffix_frozen as u64) << 13
            | (fields as u64) << 14
            | (bytes as u64) << 32
    }

    /// A trailer, `distance` words after its header.
    pub const fn trailer(distance: usize) -> u64 {
        (distance as u64) << 3 | TAG_TRAILER
    }
}

/// The state's fields code reads and writes, by their byte offsets
/// (`state`), and its size in words.
pub mod state {
    pub const RUST_SP: usize = 0;
    pub const STACK_LIMIT: usize = 16;
    pub const FUEL: usize = 24;
    pub const TRAP: usize = 32;
    pub const PC: usize = 40;
    pub const ARGS: usize = 48;
    pub const CALLOUT: usize = 112;
    pub const NATIVE_SP: usize = 136;
    pub const FP: usize = 144;
    pub const TOP: usize = 152;
    pub const WORDS: usize = 160;
    pub const ALLOC_LIMIT: usize = 168;
    pub const CODE: usize = 176;
    pub const AUX: usize = 184;
    pub const NARGS: usize = 192;
    pub const CLO: usize = 200;
    pub const DELTA: usize = 208;
    pub const CODE_LO: usize = 216;
    pub const TRAP_STUB: usize = 280;
    pub const FOREIGN: usize = 288;
    pub const CLOSURE: usize = 296;
    pub const CARDS: usize = 304;
    pub const OVERFLOW: usize = 480;
    pub const RESERVE: usize = 552;
    pub const POLL: usize = 560;
    pub const CONS: usize = 568;
    pub const BLOBLET: usize = 576;
    /// x86_64's arguments past the third (`(state reg4)` … `(state reg8)`).
    pub const REG4: usize = 616;
    /// kayak's own, past what either machine's convention declares: where
    /// a routine that ends the call goes back to the trampoline.
    pub const LEAVE: usize = 656;
    pub const WORDS_IN_STATE: usize = 83;
}

/// Whether this machine runs native code: arm64 macOS or x86_64 Linux, its
/// code area made.
pub fn native_code() -> bool {
    area().is_some()
}

// ---------------------------------------------------------------- the area

/// The code area's size: reserved whole, nothing committed until touched.
const AREA_BYTES: usize = 256 << 20;

struct Area {
    /// Where the area's first byte is, in each view.
    rw: usize,
    rx: usize,
    /// The next free byte's offset.
    top: Mutex<usize>,
}

/// The code area and the data region after it, one reservation, so that
/// one card table covers both: a store into a code bloblet's field marks
/// a card as a store into any object's does.
struct Space {
    area: Area,
    data: Data,
}

fn space() -> Option<&'static Space> {
    static SPACE: OnceLock<Option<Space>> = OnceLock::new();
    SPACE
        .get_or_init(|| {
            let (rw, rx) = sys::map_space(AREA_BYTES, DATA_BYTES)?;
            let cards = sys::reserve((AREA_BYTES + DATA_BYTES) >> CARD_SHIFT)?;
            Some(Space {
                area: Area {
                    rw,
                    rx,
                    top: Mutex::new(0),
                },
                data: Data {
                    base: rw + AREA_BYTES,
                    top: std::sync::atomic::AtomicUsize::new(0),
                    cards,
                },
            })
        })
        .as_ref()
}

fn area() -> Option<&'static Area> {
    space().map(|s| &s.area)
}

/// A code bloblet in the code area: where its suffix, its code, is in the
/// read+write view; how many fields it has, its trailer counted; and its
/// suffix's bytes. It never moves and is never freed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Code {
    suffix: usize,
    fields: usize,
    bytes: usize,
}

/// Room for `n` words in the code area, 16-byte aligned.
fn take(n: usize) -> usize {
    let a = area().unwrap_or_else(|| panic!("seal-code: this machine runs no native code"));
    let mut top = a.top.lock().unwrap_or_else(|e| e.into_inner());
    let at = top.next_multiple_of(16);
    let end = at + 8 * n;
    if end > AREA_BYTES {
        panic!("seal-code: the code area is full");
    }
    *top = end;
    a.rw + at
}

/// Seal a copy of a bloblet into the code area (FX-26's `seal-code`): of
/// kind `kind`, its fields `fields` (slot 1 first: the trailer, if it has
/// one, then field 0, ...), its suffix `suffix`, `bytes` long, frozen and
/// flushed from the instruction cache.
pub fn seal(kind: u8, fields: &[u64], fields_frozen: bool, suffix: &[u8]) -> Code {
    let n = fields.len();
    let words = suffix.len().div_ceil(8);
    // The header, so that the suffix is 16-byte aligned: instructions need
    // only 4, but frames and literal pools are simpler so, and kayak's
    // tier 5 keeps its 8-aligned return points in frames as fixnums
    // (C131; `tests/code.rs` holds this to it).
    let pad = (1 + n) % 2;
    let base = take(pad + 1 + n + words);
    let header = base + 8 * pad;
    let code = Code {
        suffix: header + 8 * (1 + n),
        fields: n,
        bytes: suffix.len(),
    };
    sys::put(
        header,
        layout::header(kind, n, suffix.len(), fields_frozen, true),
    );
    for (k, &w) in fields.iter().enumerate() {
        code.set_slot(k + 1, w);
    }
    for (i, chunk) in suffix.chunks(8).enumerate() {
        let mut w = [0u8; 8];
        w[..chunk.len()].copy_from_slice(chunk);
        sys::put(code.suffix + 8 * i, u64::from_le_bytes(w));
    }
    sys::flush(code.entry(), suffix.len().max(4));
    code
}

impl Code {
    /// Its reference, a value of Fixpt's layout: its suffix, tagged.
    pub fn reference(self) -> u64 {
        self.suffix as u64 | layout::TAG_BLOBLET
    }

    /// Where its code runs: its suffix in the execute view.
    pub fn entry(self) -> usize {
        let a = area().unwrap_or_else(|| panic!("native code: no code area"));
        self.suffix - a.rw + a.rx
    }

    pub fn fields(self) -> usize {
        self.fields
    }

    pub fn bytes(self) -> usize {
        self.bytes
    }

    /// Slot `k` (1 the trailer, if any): a word `8k` bytes before the code.
    pub fn slot(self, k: usize) -> u64 {
        assert!((1..=self.fields).contains(&k), "a code bloblet's field");
        sys::get(self.suffix - 8 * k)
    }

    /// Write slot `k`. A field is data: no flush.
    pub fn set_slot(self, k: usize, w: u64) {
        assert!((1..=self.fields).contains(&k), "a code bloblet's field");
        sys::put(self.suffix - 8 * k, w)
    }

    /// The suffix's byte `i`.
    pub fn byte(self, i: usize) -> u8 {
        assert!(i < self.bytes, "a code bloblet's byte");
        (sys::get(self.suffix + (i & !7)) >> (8 * (i & 7))) as u8
    }
}

// ---------------------------------------------------------------- objects

/// The memory objects of Fixpt's layout are made in: reserved whole,
/// nothing committed until touched, never freed; and its card table, a
/// byte for each 512 bytes, which made code marks as the convention's
/// `barrier` says (`cards + (address >> 9)`).
struct Data {
    base: usize,
    top: std::sync::atomic::AtomicUsize,
    cards: usize,
}

const DATA_BYTES: usize = 64 << 30;
const CARD_SHIFT: u32 = 9;

fn data() -> Option<&'static Data> {
    space().map(|s| &s.data)
}

/// The card table's address biased by the space's start, as the
/// convention's state gives it: `cards + (address >> 9)` is an address's
/// card.
fn biased_cards() -> u64 {
    space().map_or(0, |s| {
        (s.data.cards as u64).wrapping_sub((s.area.rw >> CARD_SHIFT) as u64)
    })
}

/// Whether the card of `address` is marked; a test's.
pub fn card_marked(address: u64) -> bool {
    sys::get_byte((biased_cards().wrapping_add(address >> CARD_SHIFT)) as usize) != 0
}

/// Words of memory for objects, 16-byte aligned, never freed: this
/// thread's chunk of the data region, taken from in turn.
fn words(n: usize) -> usize {
    thread_local! {
        static CHUNK: Cell<(usize, usize)> = const { Cell::new((0, 0)) };
    }
    let d = data().unwrap_or_else(|| panic!("native code: no memory for objects on this machine"));
    CHUNK.with(|c| {
        let (mut at, mut end) = c.get();
        let bytes = (8 * n).next_multiple_of(16);
        if at + bytes > end {
            let size = (1 << 16).max(bytes);
            let off = d.top.fetch_add(size, std::sync::atomic::Ordering::Relaxed);
            if off + size > DATA_BYTES {
                panic!("native code: the memory for objects is full");
            }
            at = d.base + off;
            end = at + size;
        }
        c.set((at + bytes, end));
        at
    })
}

/// An object of Fixpt's layout outside the code area: a header, its slots
/// (`slots[0]` slot 1), and its suffix's words, `bytes` of them used.
/// Never freed.
fn object_with(kind: u8, slots: &[u64], suffix: &[u64], bytes: usize) -> u64 {
    let n = slots.len();
    // The suffix 16-byte aligned, as references are made to it.
    let pad = (1 + n) % 2;
    let header = words(pad + 1 + n + suffix.len()) + 8 * pad;
    let at = header + 8 * (1 + n);
    sys::put(header, layout::header(kind, n, bytes, false, false));
    for (k, &w) in slots.iter().enumerate() {
        sys::put(at - 8 * (k + 1), w);
    }
    for (i, &w) in suffix.iter().enumerate() {
        sys::put(at + 8 * i, w);
    }
    at as u64 | layout::TAG_BLOBLET
}

/// An object of no suffix.
fn object(kind: u8, slots: &[u64]) -> u64 {
    object_with(kind, slots, &[], 0)
}

/// What an object of Fixpt's layout is: its kind, its count of fields
/// (the trailer counted), its suffix's bytes; none if `w` is no bloblet.
pub fn header_of(w: u64) -> Option<(u8, usize, usize)> {
    (w & layout::TAG_MASK == layout::TAG_BLOBLET).then(|| {
        let h = slot_of(w, 0);
        (
            ((h >> 3) & 0xff) as u8,
            ((h >> 14) & 0x3_ffff) as usize,
            (h >> 32) as usize,
        )
    })
}

/// Its slot `k` (0, the header, if it has no fields; 1, the trailer, if it
/// has some); trusted to be within it.
pub fn slot_of(w: u64, k: usize) -> u64 {
    let at = (w & !layout::TAG_MASK) as usize;
    if k == 0 {
        // The header: just before the fields, or the suffix if none.
        let fields = (sys::get(at - 8) & layout::TAG_MASK == layout::TAG_TRAILER)
            .then(|| (sys::get(at - 8) >> 3) as usize);
        sys::get(at - 8 * (1 + fields.unwrap_or(0)))
    } else {
        sys::get(at - 8 * k)
    }
}

/// Its suffix's word `i`.
pub fn suffix_word(w: u64, i: usize) -> u64 {
    sys::get((w & !layout::TAG_MASK) as usize + 8 * i)
}

/// A plain bloblet (kind `bloblet`) of `fields` (a trailer before them,
/// as the object model has it) and `bytes` suffix bytes, `suffix` its words.
pub fn bloblet(fields: &[u64], suffix: &[u64], bytes: usize) -> u64 {
    let mut slots = vec![layout::trailer(fields.len() + 1)];
    slots.extend_from_slice(fields);
    object_with(layout::KIND_BLOBLET, &slots, suffix, bytes)
}

/// A string of Fixpt's layout: its length, then its code points two to a
/// word, the first in the low half.
pub fn string(s: &str) -> u64 {
    let points: Vec<u32> = s.chars().map(|c| c as u32).collect();
    let mut suffix = vec![points.len() as u64];
    suffix.extend(
        points
            .chunks(2)
            .map(|p| p[0] as u64 | p.get(1).map_or(0, |&c| (c as u64) << 32)),
    );
    object_with(layout::KIND_STRING, &[], &suffix, 8 * suffix.len())
}

/// A string's text; none if `w` is no string.
pub fn string_of(w: u64) -> Option<String> {
    match header_of(w) {
        Some((layout::KIND_STRING, 0, _)) => {
            let n = suffix_word(w, 0) as usize;
            (0..n)
                .map(|i| char::from_u32((suffix_word(w, 1 + i / 2) >> (32 * (i % 2))) as u32))
                .collect()
        }
        _ => None,
    }
}

/// A native closure of `code` over nothing (FX-26's `native-procedure`).
pub fn closure(code: Code) -> u64 {
    object(
        layout::KIND_NATIVE_CLOSURE,
        &[layout::trailer(2), code.reference()],
    )
}

/// What a closure's code is, its slot 2.
pub fn closure_code(closure: u64) -> u64 {
    sys::get((closure & !layout::TAG_MASK) as usize - 8 * layout::CLOSURE_CODE)
}

/// A pair of Fixpt's layout: its car at the reference less 1, its cdr at
/// 7 more (`(pair (car -1) (cdr 7))`).
pub fn cons(car: u64, cdr: u64) -> u64 {
    let at = words(2);
    sys::put(at, car);
    sys::put(at + 8, cdr);
    at as u64 | layout::TAG_PAIR
}

/// A pair's car and cdr; none if `w` is not a pair.
pub fn pair(w: u64) -> Option<(u64, u64)> {
    (w & layout::TAG_MASK == layout::TAG_PAIR).then(|| {
        let at = (w & !layout::TAG_MASK) as usize;
        (sys::get(at), sys::get(at + 8))
    })
}

/// A procedure of Rust's, given Fixpt's values and giving one, that native
/// code calls by the `foreign` routine.
pub type Foreign = Box<dyn Fn(&[u64]) -> u64>;
type Given = &'static dyn Fn(&[u64]) -> u64;

thread_local! {
    /// The Rust procedures made closures of on this thread, by number.
    static FOREIGN: std::cell::RefCell<Vec<Given>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// A closure of Fixpt's layout over a procedure of Rust's: of kind
/// `cellular-closure`, its code field a fixnum, its number here, which is
/// below the code area as the convention has it of what is not native
/// code, so that native code calls it by `foreign`. On this thread only.
pub fn foreign_procedure(f: Foreign) -> u64 {
    let f: Given = Box::leak(f);
    let k = FOREIGN.with(|t| {
        let mut t = t.borrow_mut();
        t.push(f);
        t.len() - 1
    });
    object(
        layout::KIND_CELLULAR_CLOSURE,
        &[layout::trailer(2), (k as u64) << layout::FIXNUM_SHIFT],
    )
}

// ---------------------------------------------------------------- calls

thread_local! {
    /// This thread's state, at an address that never changes.
    static STATE: &'static [Cell<u64>] = new_state();
    /// A panic a foreign procedure ended in, carried past native frames.
    static UNWOUND: Cell<Option<Box<dyn std::any::Any + Send>>> = const { Cell::new(None) };
    /// What a routine that refused said, as Fixpt's says it.
    static REFUSAL: Cell<Option<String>> = const { Cell::new(None) };
}

/// What ends a call other than its code's returning, as the trap word
/// says it: the routine's number above bit 8, a trap's kind below.
const BY_TRAP: u64 = 1;
const BY_UNWINDING: u64 = 8;
const NOT_KAYAKS: [(&str, u64); 1] = [("closure", 7)];

/// What made code ended in, when not by returning: one of the
/// convention's traps (by the `trap` routine, or `overflow`'s, a stack
/// overflow), or a routine kayak does not have yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Trap {
    Kind(u64),
    Routine(&'static str),
    /// A trap a routine said more of, as Fixpt's says it.
    Failed(String),
}

/// The trap kind of a stack overflow (`(trap (kinds (2 stack-overflow)))`).
pub const STACK_OVERFLOW: u64 = 2;
/// Fuel given at each `poll`.
const SLICE: u64 = 1 << 20;

fn new_state() -> &'static [Cell<u64>] {
    let s: &'static [Cell<u64>] =
        Box::leak((0..state::WORDS_IN_STATE).map(|_| Cell::new(0)).collect());
    let set = |off: usize, w: u64| s[off / 8].set(w);
    if let Some(a) = area() {
        set(
            state::DELTA,
            (a.rx as u64)
                .wrapping_sub(a.rw as u64)
                .wrapping_sub(layout::TAG_BLOBLET),
        );
        set(state::CODE_LO, a.rw as u64);
        set(state::CARDS, biased_cards());
        for (field, at) in sys::routines() {
            set(field, at as u64);
        }
    }
    set(state::RESERVE, SLICE);
    set(state::FUEL, SLICE);
    set(state::STACK_LIMIT, sys::stack_limit() as u64);
    s
}

/// Call native closure `closure` (a reference of Fixpt's layout), whose
/// code runs at `entry`, with `args`, Fixpt's values, at most eight; its
/// answer, one too, or the trap it ended in. A panic of a foreign procedure
/// it called goes on here.
pub fn call(closure: u64, entry: usize, args: &[u64]) -> Result<u64, Trap> {
    assert!(
        args.len() <= 8,
        "a native procedure called with more than eight arguments"
    );
    let mut a = [0u64; 8];
    a[..args.len()].copy_from_slice(args);
    STATE.with(|s| {
        let answer = sys::enter(s.as_ptr() as usize, entry, closure, &a, args.len());
        let trap = s[state::TRAP / 8].replace(0);
        match trap >> 8 {
            0 => Ok(answer),
            BY_TRAP => Err(match REFUSAL.with(|r| r.take()) {
                Some(m) => Trap::Failed(m),
                None => Trap::Kind(trap & 0xff),
            }),
            BY_UNWINDING => match UNWOUND.with(|u| u.take()) {
                Some(p) => std::panic::resume_unwind(p),
                None => panic!("native code: unwound, and nothing to resume"),
            },
            k => match NOT_KAYAKS.iter().find(|(_, n)| *n == k) {
                Some((r, _)) => Err(Trap::Routine(r)),
                None => panic!("native code: ended by {trap:#x}, which nothing of kayak's says"),
            },
        }
    })
}

/// The `foreign` routine's Rust: the procedure closure `closure` is over,
/// called with `count` arguments from the state; a panic kept, and the
/// trap word set so that the routine goes back to the trampoline.
// Called only from the trampoline: arm64 macOS's and x86_64 Linux's.
#[cfg_attr(
    not(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "linux", target_arch = "x86_64")
    )),
    allow(dead_code)
)]
extern "C" fn foreign(state: usize, closure: u64, count: u64) -> u64 {
    let s = |off: usize| sys::get(state + off);
    let run = || {
        let k = (closure_code(closure) >> layout::FIXNUM_SHIFT) as usize;
        let f = FOREIGN.with(|t| t.borrow().get(k).copied());
        let f = f.unwrap_or_else(|| {
            panic!("native code: called a procedure of another thread's, or none")
        });
        if count > 8 {
            panic!(
                "native code: called a procedure with more than eight arguments, which kayak cannot yet"
            )
        }
        let args: Vec<u64> = (0..count as usize)
            .map(|i| s(state::ARGS + 8 * i))
            .collect();
        f(&args)
    };
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(run)) {
        Ok(w) => w,
        Err(p) => {
            UNWOUND.with(|u| u.set(Some(p)));
            sys::put(state + state::TRAP, BY_UNWINDING << 8);
            0
        }
    }
}

/// The trap kind of a routine's refusal (`(3 a-primitive-failed)`).
pub const PRIMITIVE_FAILED: u64 = 3;

/// The `bloblet` routine's Rust: a plain bloblet of `bytes` suffix bytes
/// and `fields` fields, each a fixnum, zeroed (`(makes (kind bloblet)
/// (trailer 1) zeroed)`); or, for sizes the header cannot say, the trap
/// word set (`(traps-when-refused 3)`), so that the routine goes back.
// Called only from the trampoline: arm64 macOS's and x86_64 Linux's.
#[cfg_attr(
    not(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "linux", target_arch = "x86_64")
    )),
    allow(dead_code)
)]
extern "C" fn bloblet_routine(state: usize, bytes: u64, fields: u64) -> u64 {
    let size = |w: u64, most: u64| {
        (w & layout::TAG_MASK == layout::TAG_FIXNUM
            && (w as i64) >= 0
            && (w >> layout::FIXNUM_SHIFT) < most)
            .then_some((w >> layout::FIXNUM_SHIFT) as usize)
    };
    match (size(bytes, 1 << 32), size(fields, (1 << 18) - 1)) {
        (Some(b), Some(f)) => bloblet(&vec![0; f], &vec![0; b.div_ceil(8)], b),
        _ => {
            let n = |w: u64| (w as i64) >> layout::FIXNUM_SHIFT;
            REFUSAL.with(|r| {
                r.set(Some(format!(
                    "bloblet: no bloblet of {} bytes and {} fields",
                    n(bytes),
                    n(fields)
                )))
            });
            sys::put(state + state::TRAP, BY_TRAP << 8 | PRIMITIVE_FAILED);
            0
        }
    }
}

/// The `cons` routine's Rust.
// Called only from the trampoline: arm64 macOS's and x86_64 Linux's.
#[cfg_attr(
    not(any(
        all(target_os = "macos", target_arch = "aarch64"),
        all(target_os = "linux", target_arch = "x86_64")
    )),
    allow(dead_code)
)]
extern "C" fn cons_routine(car: u64, cdr: u64) -> u64 {
    cons(car, cdr)
}

// ---------------------------------------------------------------- the system

/// Words of memory read and written, on the machines that run native code.
#[cfg(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(target_os = "linux", target_arch = "x86_64")
))]
mod memory {
    /// The word at `at`: an 8-aligned address of a word `take` or `words`
    /// handed out, or of the state, or one native code gave as a pair's or
    /// a closure's (trusted, as the code is, by `(io native)`).
    pub(crate) fn get(at: usize) -> u64 {
        // SAFETY: as said; all such words are mapped and never freed.
        unsafe { std::ptr::read_volatile(at as *const u64) }
    }

    pub(crate) fn put(at: usize, w: u64) {
        // SAFETY: as `get`.
        unsafe { std::ptr::write_volatile(at as *mut u64, w) }
    }

    /// The byte at `at`: of the card table, which is mapped for good.
    pub(crate) fn get_byte(at: usize) -> u8 {
        // SAFETY: as `get`.
        unsafe { std::ptr::read_volatile(at as *const u8) }
    }
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
mod sys {
    use super::state;

    type Port = u32;
    unsafe extern "C" {
        fn mmap(addr: *mut u8, len: usize, prot: i32, flags: i32, fd: i32, off: i64) -> *mut u8;
        fn mprotect(addr: *mut u8, len: usize, prot: i32) -> i32;
        fn munmap(addr: *mut u8, len: usize) -> i32;
        fn sys_icache_invalidate(start: *mut u8, len: usize);
        fn mach_task_self() -> Port;
        fn mach_vm_remap(
            target: Port,
            target_addr: *mut u64,
            size: u64,
            mask: u64,
            flags: i32,
            src_task: Port,
            src_addr: u64,
            copy: u32,
            cur: *mut i32,
            max: *mut i32,
            inherit: u32,
        ) -> i32;
        fn pthread_self() -> usize;
        fn pthread_get_stackaddr_np(t: usize) -> *mut u8;
        fn pthread_get_stacksize_np(t: usize) -> usize;
        fn kayak_jit_enter(
            state: usize,
            entry: usize,
            closure: u64,
            args: *const u64,
            count: u64,
        ) -> u64;
        fn kayak_jit_leave();
        fn kayak_jit_trap();
        fn kayak_jit_overflow();
        fn kayak_jit_poll();
        fn kayak_jit_cons();
        fn kayak_jit_foreign();
        fn kayak_jit_bloblet();
        fn kayak_jit_closure();
    }
    const PROT_READ: i32 = 1;
    const PROT_WRITE: i32 = 2;
    const PROT_EXEC: i32 = 4;
    const MAP_SHARED: i32 = 1;
    const MAP_PRIVATE: i32 = 2;
    const MAP_FIXED: i32 = 0x10;
    const MAP_ANON: i32 = 0x1000;
    const VM_FLAGS_ANYWHERE: i32 = 1;
    const VM_INHERIT_SHARE: u32 = 1;

    /// Memory reserved: read+write, private, nothing committed until
    /// touched; none if the system says no.
    pub(super) fn reserve(bytes: usize) -> Option<usize> {
        // SAFETY: a fresh mapping the kernel places; nothing refers to it.
        let at = unsafe {
            mmap(
                std::ptr::null_mut(),
                bytes,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANON,
                -1,
                0,
            )
        };
        (at as isize != -1).then_some(at as usize)
    }

    /// The space: `code + data` bytes reserved; the first `code` of them
    /// replaced by a shared anonymous mapping, read+write, which a second
    /// view, read+execute, shares (as Fixpt's `exec_view`). The code
    /// area's two addresses.
    pub(super) fn map_space(code: usize, data: usize) -> Option<(usize, usize)> {
        let base = reserve(code + data)?;
        // SAFETY: the first `code` bytes of the reservation just made,
        // which nothing refers to, replaced, zero as they were.
        let rw = unsafe {
            mmap(
                base as *mut u8,
                code,
                PROT_READ | PROT_WRITE,
                MAP_FIXED | MAP_SHARED | MAP_ANON,
                -1,
                0,
            )
        };
        if rw as isize == -1 {
            return None;
        }
        let mut rx: u64 = 0;
        let (mut cur, mut max) = (0, 0);
        // SAFETY: remaps the shared mapping just made at an address the
        // kernel chooses, sharing its pages.
        let kr = unsafe {
            mach_vm_remap(
                mach_task_self(),
                &mut rx,
                code as u64,
                0,
                VM_FLAGS_ANYWHERE,
                mach_task_self(),
                rw as u64,
                0,
                &mut cur,
                &mut max,
                VM_INHERIT_SHARE,
            )
        };
        // SAFETY: `rx` is the view just made, `code` long.
        if kr != 0 || unsafe { mprotect(rx as *mut u8, code, PROT_READ | PROT_EXEC) } != 0 {
            // SAFETY: unmapping what was mapped above, which nothing holds.
            unsafe {
                if kr == 0 {
                    munmap(rx as *mut u8, code);
                }
                munmap(base as *mut u8, code + data);
            }
            return None;
        }
        Some((rw as usize, rx as usize))
    }

    pub(super) use super::memory::{get, get_byte, put};

    pub(super) fn flush(rx: usize, len: usize) {
        // SAFETY: a range of the execute view, mapped for good.
        unsafe { sys_icache_invalidate(rx as *mut u8, len) }
    }

    /// The lowest address this thread's stack may reach, less room for
    /// Rust's own frames below native code's (a call-out's).
    pub(super) fn stack_limit() -> usize {
        // SAFETY: asks of this thread, which is running.
        let (top, size) = unsafe {
            let t = pthread_self();
            (
                pthread_get_stackaddr_np(t) as usize,
                pthread_get_stacksize_np(t),
            )
        };
        top - size + (1 << 20)
    }

    /// Each routine's state field and where it runs; and the way back.
    pub(super) fn routines() -> [(usize, usize); 8] {
        let at = |f: unsafe extern "C" fn()| f as *const () as usize;
        [
            (state::TRAP_STUB, at(kayak_jit_trap)),
            (state::OVERFLOW, at(kayak_jit_overflow)),
            (state::POLL, at(kayak_jit_poll)),
            (state::CONS, at(kayak_jit_cons)),
            (state::FOREIGN, at(kayak_jit_foreign)),
            (state::BLOBLET, at(kayak_jit_bloblet)),
            (state::CLOSURE, at(kayak_jit_closure)),
            (state::LEAVE, at(kayak_jit_leave)),
        ]
    }

    pub(super) fn enter(
        state: usize,
        entry: usize,
        closure: u64,
        args: &[u64; 8],
        count: usize,
    ) -> u64 {
        // SAFETY: `entry` is a sealed code bloblet's code, which keeps the
        // native convention by the program's word (`(io native)`, the one
        // trusted step); the trampoline keeps every register Rust keeps.
        unsafe { kayak_jit_enter(state, entry, closure, args.as_ptr(), count as u64) }
    }

    // The trampoline. In: x0 the state, x1 the entry, x2 the closure, x3
    // the arguments, x4 their count. Rust's callee-saved registers (x19–x28,
    // d8–d15, the frame and link) kept on the stack, with the state's
    // `rust-sp` of a trampoline outside this one; the convention's pinned
    // registers set from the state, and fuel given back to it after.
    //
    // The routines, each where the state's field says. `leave`: the trap
    // word in x16 kept, Rust's stack as the innermost trampoline left it,
    // and back as from code. `foreign` and `cons` call Rust (C's
    // convention keeps x19–x28, so the pinned registers), the arguments in
    // the state's `args`, and fuel there too, for a trampoline inside.
    std::arch::global_asm!(
        ".globl _kayak_jit_enter",
        ".p2align 2",
        "_kayak_jit_enter:",
        "stp x29, x30, [sp, #-176]!",
        "mov x29, sp",
        "stp x19, x20, [sp, #16]",
        "stp x21, x22, [sp, #32]",
        "stp x23, x24, [sp, #48]",
        "stp x25, x26, [sp, #64]",
        "stp x27, x28, [sp, #80]",
        "stp d8, d9, [sp, #96]",
        "stp d10, d11, [sp, #112]",
        "stp d12, d13, [sp, #128]",
        "stp d14, d15, [sp, #144]",
        "mov x24, x0",
        "ldr x17, [x24, #0]",
        "str x17, [sp, #160]",
        "mov x17, sp",
        "str x17, [x24, #0]",
        "ldr x28, [x24, #24]",
        "ldr x27, [x24, #16]",
        "ldr x26, [x24, #208]",
        "mov x16, x1",
        "mov x10, x2",
        "mov x9, x4",
        "mov x11, x3",
        "ldp x1, x2, [x11]",
        "ldp x3, x4, [x11, #16]",
        "ldp x5, x6, [x11, #32]",
        "ldp x7, x8, [x11, #48]",
        "blr x16",
        "1:",
        "str x28, [x24, #24]",
        "ldr x17, [sp, #160]",
        "str x17, [x24, #0]",
        "ldp d14, d15, [sp, #144]",
        "ldp d12, d13, [sp, #128]",
        "ldp d10, d11, [sp, #112]",
        "ldp d8, d9, [sp, #96]",
        "ldp x27, x28, [sp, #80]",
        "ldp x25, x26, [sp, #64]",
        "ldp x23, x24, [sp, #48]",
        "ldp x21, x22, [sp, #32]",
        "ldp x19, x20, [sp, #16]",
        "ldp x29, x30, [sp], #176",
        "ret",
        ".globl _kayak_jit_leave",
        ".p2align 2",
        "_kayak_jit_leave:",
        "str x16, [x24, #32]",
        "ldr x17, [x24, #0]",
        "mov sp, x17",
        "mov x0, #0",
        "b 1b",
        ".globl _kayak_jit_trap",
        ".p2align 2",
        "_kayak_jit_trap:",
        "add x16, x9, #256",
        "b _kayak_jit_leave",
        ".globl _kayak_jit_overflow",
        ".p2align 2",
        "_kayak_jit_overflow:",
        "movz x16, #258",
        "b _kayak_jit_leave",
        ".globl _kayak_jit_bloblet",
        ".p2align 2",
        "_kayak_jit_bloblet:",
        "stp x29, x30, [sp, #-16]!",
        "mov x29, sp",
        "mov x0, x24",
        "bl {bloblet}",
        "ldr x16, [x24, #32]",
        "cbnz x16, 3f",
        "ldp x29, x30, [sp], #16",
        "ret",
        "3:",
        "b _kayak_jit_leave",
        ".globl _kayak_jit_closure",
        ".p2align 2",
        "_kayak_jit_closure:",
        "movz x16, #1792",
        "b _kayak_jit_leave",
        ".globl _kayak_jit_poll",
        ".p2align 2",
        "_kayak_jit_poll:",
        "ldr x28, [x24, #552]",
        "ret",
        ".globl _kayak_jit_cons",
        ".p2align 2",
        "_kayak_jit_cons:",
        "stp x29, x30, [sp, #-16]!",
        "mov x29, sp",
        "mov x0, x1",
        "mov x1, x2",
        "bl {cons}",
        "ldp x29, x30, [sp], #16",
        "ret",
        ".globl _kayak_jit_foreign",
        ".p2align 2",
        "_kayak_jit_foreign:",
        "stp x29, x30, [sp, #-16]!",
        "mov x29, sp",
        "stp x1, x2, [x24, #48]",
        "stp x3, x4, [x24, #64]",
        "stp x5, x6, [x24, #80]",
        "stp x7, x8, [x24, #96]",
        "str x28, [x24, #24]",
        "mov x0, x24",
        "mov x1, x10",
        "mov x2, x9",
        "bl {foreign}",
        "ldr x28, [x24, #24]",
        "ldr x16, [x24, #32]",
        "cbnz x16, 2f",
        "ldp x29, x30, [sp], #16",
        "ret",
        "2:",
        "b _kayak_jit_leave",
        cons = sym super::cons_routine,
        bloblet = sym super::bloblet_routine,
        foreign = sym super::foreign,
    );
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod sys {
    use super::state;

    unsafe extern "C" {
        fn mmap(addr: *mut u8, len: usize, prot: i32, flags: i32, fd: i32, off: i64) -> *mut u8;
        fn mremap(old: *mut u8, old_len: usize, new_len: usize, flags: i32, ...) -> *mut u8;
        fn mprotect(addr: *mut u8, len: usize, prot: i32) -> i32;
        fn munmap(addr: *mut u8, len: usize) -> i32;
        fn pthread_self() -> usize;
        fn pthread_getattr_np(t: usize, attr: *mut u64) -> i32;
        fn pthread_attr_getstack(attr: *const u64, addr: *mut *mut u8, size: *mut usize) -> i32;
        fn pthread_attr_destroy(attr: *mut u64) -> i32;
        fn kayak_jit_enter(
            state: usize,
            entry: usize,
            closure: u64,
            args: *const u64,
            count: u64,
        ) -> u64;
        fn kayak_jit_leave();
        fn kayak_jit_trap();
        fn kayak_jit_overflow();
        fn kayak_jit_poll();
        fn kayak_jit_cons();
        fn kayak_jit_foreign();
        fn kayak_jit_bloblet();
        fn kayak_jit_closure();
    }
    const PROT_READ: i32 = 1;
    const PROT_WRITE: i32 = 2;
    const PROT_EXEC: i32 = 4;
    const MAP_SHARED: i32 = 1;
    const MAP_PRIVATE: i32 = 2;
    const MAP_FIXED: i32 = 0x10;
    const MAP_ANON: i32 = 0x20;
    const MAP_NORESERVE: i32 = 0x4000;
    const MREMAP_MAYMOVE: i32 = 1;

    pub(super) use super::memory::{get, get_byte, put};

    /// Memory reserved: read+write, private, nothing committed until
    /// touched, nor counted against the system's commit; none if the
    /// system says no.
    pub(super) fn reserve(bytes: usize) -> Option<usize> {
        // SAFETY: a fresh mapping the kernel places; nothing refers to it.
        let at = unsafe {
            mmap(
                std::ptr::null_mut(),
                bytes,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANON | MAP_NORESERVE,
                -1,
                0,
            )
        };
        (at as isize != -1).then_some(at as usize)
    }

    /// The space, as arm64 macOS's: `code + data` bytes reserved, the first
    /// `code` of them a shared anonymous mapping, read+write, and a second
    /// view of the same pages, read+execute, made by `mremap` of size 0
    /// (as Fixpt's `fixpt-memmgmt` makes its own).
    pub(super) fn map_space(code: usize, data: usize) -> Option<(usize, usize)> {
        let base = reserve(code + data)?;
        // SAFETY: the first `code` bytes of the reservation just made,
        // which nothing refers to, replaced, zero as they were.
        let rw = unsafe {
            mmap(
                base as *mut u8,
                code,
                PROT_READ | PROT_WRITE,
                MAP_FIXED | MAP_SHARED | MAP_ANON,
                -1,
                0,
            )
        };
        if rw as isize == -1 {
            return None;
        }
        // SAFETY: a second mapping of the shared one just made, where the
        // kernel chooses; then that one made read+execute.
        let rx = unsafe { mremap(rw, 0, code, MREMAP_MAYMOVE) };
        // SAFETY: `rx` is the view just made, `code` long.
        if rx as isize == -1 || unsafe { mprotect(rx, code, PROT_READ | PROT_EXEC) } != 0 {
            // SAFETY: unmapping what was mapped above, which nothing holds.
            unsafe {
                if rx as isize != -1 {
                    munmap(rx, code);
                }
                munmap(base as *mut u8, code + data);
            }
            return None;
        }
        Some((rw as usize, rx as usize))
    }

    /// x86_64's instruction fetch sees what was written to the same pages
    /// by another view: nothing to flush.
    pub(super) fn flush(_: usize, _: usize) {}

    /// The lowest address this thread's stack may reach, less room for
    /// Rust's own frames below native code's (a call-out's).
    pub(super) fn stack_limit() -> usize {
        // glibc's `pthread_attr_t`, 56 bytes on x86_64, here in 64.
        let mut attr = [0u64; 8];
        let (mut low, mut size) = (std::ptr::null_mut(), 0usize);
        // SAFETY: asks of this thread, which is running, into `attr`,
        // which is big enough, and frees what it filled.
        let ok = unsafe {
            pthread_getattr_np(pthread_self(), attr.as_mut_ptr()) == 0 && {
                let r = pthread_attr_getstack(attr.as_ptr(), &mut low, &mut size);
                pthread_attr_destroy(attr.as_mut_ptr());
                r == 0
            }
        };
        if !ok {
            panic!("native code: this thread's stack cannot be found");
        }
        low as usize + (1 << 20)
    }

    /// Each routine's state field and where it runs; and the way back.
    pub(super) fn routines() -> [(usize, usize); 8] {
        let at = |f: unsafe extern "C" fn()| f as *const () as usize;
        [
            (state::TRAP_STUB, at(kayak_jit_trap)),
            (state::OVERFLOW, at(kayak_jit_overflow)),
            (state::POLL, at(kayak_jit_poll)),
            (state::CONS, at(kayak_jit_cons)),
            (state::FOREIGN, at(kayak_jit_foreign)),
            (state::BLOBLET, at(kayak_jit_bloblet)),
            (state::CLOSURE, at(kayak_jit_closure)),
            (state::LEAVE, at(kayak_jit_leave)),
        ]
    }

    pub(super) fn enter(
        state: usize,
        entry: usize,
        closure: u64,
        args: &[u64; 8],
        count: usize,
    ) -> u64 {
        // SAFETY: `entry` is a sealed code bloblet's code, which keeps the
        // native convention by the program's word (`(io native)`, the one
        // trusted step); the trampoline keeps every register Rust keeps.
        unsafe { kayak_jit_enter(state, entry, closure, args.as_ptr(), count as u64) }
    }

    // The trampoline. In (C's convention): rdi the state, rsi the entry,
    // rdx the closure, rcx the arguments, r8 their count. Rust's
    // callee-saved registers (rbx, rbp, r12–r15) pushed, and the state's
    // `rust-sp` of a trampoline outside this one; the state in r15, the
    // arguments past the third in its `reg4`–`reg8`. Fuel, the stack's
    // limit and the code's distance are the state's already.
    //
    // The routines, each entered by `call [r15 + field]`. `leave`: the trap
    // word in r11 kept, Rust's stack as the innermost trampoline left it,
    // and back as from code. `foreign`, `cons` and `bloblet` call Rust on
    // a stack 16-byte aligned (C's convention keeps r15); `foreign` with
    // its arguments in the state's `args`.
    std::arch::global_asm!(
        ".globl kayak_jit_enter",
        ".p2align 4",
        "kayak_jit_enter:",
        "push rbp",
        "mov rbp, rsp",
        "push rbx",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "push qword ptr [rdi]",
        "mov r15, rdi",
        "mov [r15], rsp",
        "mov rbx, rdx",
        "mov r10, r8",
        "mov r11, rsi",
        "mov r9, rcx",
        "mov rax, [r9 + 24]",
        "mov [r15 + 616], rax",
        "mov rax, [r9 + 32]",
        "mov [r15 + 624], rax",
        "mov rax, [r9 + 40]",
        "mov [r15 + 632], rax",
        "mov rax, [r9 + 48]",
        "mov [r15 + 640], rax",
        "mov rax, [r9 + 56]",
        "mov [r15 + 648], rax",
        "mov rdi, [r9]",
        "mov rsi, [r9 + 8]",
        "mov r8, [r9 + 16]",
        "call r11",
        ".Lkayak_jit_back:",
        "pop qword ptr [r15]",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbx",
        "pop rbp",
        "ret",
        ".globl kayak_jit_leave",
        ".p2align 4",
        "kayak_jit_leave:",
        "mov [r15 + 32], r11",
        "mov rsp, [r15]",
        "xor eax, eax",
        "jmp .Lkayak_jit_back",
        ".globl kayak_jit_trap",
        ".p2align 4",
        "kayak_jit_trap:",
        "lea r11, [r10 + 256]",
        "jmp kayak_jit_leave",
        ".globl kayak_jit_overflow",
        ".p2align 4",
        "kayak_jit_overflow:",
        "mov r11d, 258",
        "jmp kayak_jit_leave",
        ".globl kayak_jit_closure",
        ".p2align 4",
        "kayak_jit_closure:",
        "mov r11d, 1792",
        "jmp kayak_jit_leave",
        ".globl kayak_jit_poll",
        ".p2align 4",
        "kayak_jit_poll:",
        "push rax",
        "mov rax, [r15 + 552]",
        "mov [r15 + 24], rax",
        "pop rax",
        "ret",
        ".globl kayak_jit_cons",
        ".p2align 4",
        "kayak_jit_cons:",
        "push rbp",
        "mov rbp, rsp",
        "and rsp, -16",
        "call {cons}",
        "mov rsp, rbp",
        "pop rbp",
        "ret",
        ".globl kayak_jit_bloblet",
        ".p2align 4",
        "kayak_jit_bloblet:",
        "push rbp",
        "mov rbp, rsp",
        "and rsp, -16",
        "mov rdx, rsi",
        "mov rsi, rdi",
        "mov rdi, r15",
        "call {bloblet}",
        "mov rsp, rbp",
        "pop rbp",
        "mov r11, [r15 + 32]",
        "test r11, r11",
        "jnz kayak_jit_leave",
        "ret",
        ".globl kayak_jit_foreign",
        ".p2align 4",
        "kayak_jit_foreign:",
        "push rbp",
        "mov rbp, rsp",
        "and rsp, -16",
        "mov [r15 + 48], rdi",
        "mov [r15 + 56], rsi",
        "mov [r15 + 64], r8",
        "mov rax, [r15 + 616]",
        "mov [r15 + 72], rax",
        "mov rax, [r15 + 624]",
        "mov [r15 + 80], rax",
        "mov rax, [r15 + 632]",
        "mov [r15 + 88], rax",
        "mov rax, [r15 + 640]",
        "mov [r15 + 96], rax",
        "mov rax, [r15 + 648]",
        "mov [r15 + 104], rax",
        "mov rdi, r15",
        "mov rsi, rbx",
        "mov rdx, r10",
        "call {foreign}",
        "mov rsp, rbp",
        "pop rbp",
        "mov r11, [r15 + 32]",
        "test r11, r11",
        "jnz kayak_jit_leave",
        "ret",
        cons = sym super::cons_routine,
        bloblet = sym super::bloblet_routine,
        foreign = sym super::foreign,
    );
}

#[cfg(not(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(target_os = "linux", target_arch = "x86_64")
)))]
mod sys {
    pub(super) fn map_space(_: usize, _: usize) -> Option<(usize, usize)> {
        None
    }
    pub(super) fn reserve(_: usize) -> Option<usize> {
        None
    }
    pub(super) fn get_byte(_: usize) -> u8 {
        panic!("native code: none on this machine")
    }
    pub(super) fn get(_: usize) -> u64 {
        panic!("native code: none on this machine")
    }
    pub(super) fn put(_: usize, _: u64) {
        panic!("native code: none on this machine")
    }
    pub(super) fn flush(_: usize, _: usize) {}
    pub(super) fn stack_limit() -> usize {
        0
    }
    pub(super) fn routines() -> [(usize, usize); 0] {
        []
    }
    pub(super) fn enter(_: usize, _: usize, _: u64, _: &[u64; 8], _: usize) -> u64 {
        panic!("native code: none on this machine")
    }
}
