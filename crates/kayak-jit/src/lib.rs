//! Machine code a program made, placed and run (`docs/research/native.md`,
//! decision 4; `CLAUDES_CHOICES.md` C116): FX-26's `seal-code` and
//! `native-procedure` (Fixpt's TODO §97), under Fixpt's native convention
//! (`(native-convention)`, Fixpt's `native-arm64.conv`).
//!
//! - **The code area**: code bloblets, laid out as Fixpt's (a header, the
//!   fields, a trailer, the suffix), mapped twice, as Fixpt's is
//!   (`docs/object-model.md`, "A collected code area"): written through a
//!   read+write view, run from a read+execute view of the same pages, at a
//!   fixed distance, so nothing is ever both and nothing is re-protected.
//!   Nothing in it is ever freed, as nothing of kayak's heap is (C116).
//! - **Objects in Fixpt's layout** that code reads (a native closure), in
//!   memory of their own, never freed either.
//! - **The trampoline**: from Rust into the convention (the state in `x24`,
//!   fuel `x28`, the stack's limit `x27`, the code's distance `x26`, the
//!   closure `x10`, the arguments `x1`–`x8`, their count `x9`) and back,
//!   every register Rust keeps kept.
//!
//! Only on arm64 macOS; elsewhere [`native_code`] is false, and FX-26's
//! `native-code?` with it.

use std::cell::Cell;
use std::sync::{Mutex, OnceLock};

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
    pub const KIND_BLOBLET: u8 = 32;
    pub const KIND_NATIVE_CLOSURE: u8 = 41;
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
    /// kayak's own, past what the convention declares: where a routine
    /// kayak does not have yet goes back to the trampoline.
    pub const LEAVE: usize = 584;
    pub const WORDS_IN_STATE: usize = 80;
}

/// Whether this machine runs native code: arm64 macOS, its code area made.
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

fn area() -> Option<&'static Area> {
    static AREA: OnceLock<Option<Area>> = OnceLock::new();
    AREA.get_or_init(sys::map_area).as_ref()
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
    // only 4, but frames and literal pools are simpler so.
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

/// An object of Fixpt's layout outside the code area, for code to read: a
/// header, its slots (`slots[0]` slot 1), no suffix. Never freed.
fn object(kind: u8, slots: &[u64]) -> u64 {
    let n = slots.len();
    let words: &'static [Cell<u64>] = Box::leak((0..n + 2).map(|_| Cell::new(0)).collect());
    // The suffix (empty) 16-byte aligned, as references are made to it.
    let first = words.as_ptr() as usize;
    let header = if (first + 8 * (1 + n)).is_multiple_of(16) {
        0
    } else {
        1
    };
    words[header].set(layout::header(kind, n, 0, true, true));
    for (k, &w) in slots.iter().enumerate() {
        words[header + n - k].set(w);
    }
    (first + 8 * (header + 1 + n)) as u64 | layout::TAG_BLOBLET
}

/// A native closure of `code` over nothing (FX-26's `native-procedure`).
pub fn closure(code: Code) -> u64 {
    object(
        layout::KIND_NATIVE_CLOSURE,
        &[layout::trailer(2), code.reference()],
    )
}

// ---------------------------------------------------------------- calls

thread_local! {
    /// This thread's state, at an address that never changes.
    static STATE: &'static [Cell<u64>] = new_state();
}

/// The routines the convention declares, numbered as kayak's stubs say
/// them, each with its state field.
const ROUTINES: [(&str, usize); 7] = [
    ("trap", state::TRAP_STUB),
    ("foreign", state::FOREIGN),
    ("poll", state::POLL),
    ("cons", state::CONS),
    ("bloblet", state::BLOBLET),
    ("overflow", state::OVERFLOW),
    ("closure", state::CLOSURE),
];

/// What made code ended in, when not by returning: a routine kayak does
/// not have yet, or, by the `trap` routine, one of the convention's traps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trap {
    Routine(&'static str),
    Kind(u64),
}

/// A stub for each routine, in the code area: its number in `x16` (the
/// trap's kind, from `x9`, besides), to the trampoline's way back.
fn stubs() -> &'static [usize] {
    static STUBS: OnceLock<Vec<usize>> = OnceLock::new();
    STUBS.get_or_init(|| {
        let ldr_leave = 0xf940_0000 | ((state::LEAVE / 8) as u32) << 10 | 24 << 5 | 17; // ldr x17, [x24, #LEAVE]
        let br = 0xd61f_0220; // br x17
        (1..=ROUTINES.len() as u32)
            .map(|id| {
                let first = if id == 1 {
                    0xaa09_03f0 // mov x16, x9
                } else {
                    0xd280_0010 | (id << 8) << 5 // movz x16, #(id << 8)
                };
                let second = if id == 1 { 0x9104_0210 } else { 0xd503_201f }; // add x16, x16, #256; nop
                let code: Vec<u8> = [first, second, ldr_leave, br]
                    .iter()
                    .flat_map(|w| w.to_le_bytes())
                    .collect();
                seal(layout::KIND_BLOBLET, &[], true, &code).entry()
            })
            .collect()
    })
}

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
    }
    set(state::FUEL, 1 << 40);
    set(state::STACK_LIMIT, sys::stack_limit() as u64);
    if area().is_some() {
        for (&(_, at), &stub) in ROUTINES.iter().zip(stubs()) {
            set(at, stub as u64);
        }
        set(state::LEAVE, sys::leave() as u64);
    }
    s
}

/// Call native closure `closure` (a reference of Fixpt's layout), whose
/// code runs at `entry`, with `args`, Fixpt's values, at most eight; its
/// answer, one too, or the trap it ended in.
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
            1 => Err(Trap::Kind(trap & 0xff)),
            k => Err(Trap::Routine(ROUTINES[k as usize - 1].0)),
        }
    })
}

/// What a native closure's code is, its slot 2.
pub fn closure_code(closure: u64) -> u64 {
    sys::get((closure & !layout::TAG_MASK) as usize - 8 * layout::CLOSURE_CODE)
}

// ---------------------------------------------------------------- the system

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
mod sys {
    use super::{AREA_BYTES, Area};
    use std::sync::Mutex;

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
    }
    const PROT_READ: i32 = 1;
    const PROT_WRITE: i32 = 2;
    const PROT_EXEC: i32 = 4;
    const MAP_SHARED: i32 = 1;
    const MAP_ANON: i32 = 0x1000;
    const VM_FLAGS_ANYWHERE: i32 = 1;
    const VM_INHERIT_SHARE: u32 = 1;

    /// The area: a shared anonymous mapping, read+write, and a second view
    /// of the same pages, read+execute (as Fixpt's `exec_view`).
    pub(super) fn map_area() -> Option<Area> {
        // SAFETY: a fresh mapping the kernel places; nothing refers to it.
        let rw = unsafe {
            mmap(
                std::ptr::null_mut(),
                AREA_BYTES,
                PROT_READ | PROT_WRITE,
                MAP_SHARED | MAP_ANON,
                -1,
                0,
            )
        };
        if rw as isize == -1 {
            return None;
        }
        let mut rx: u64 = 0;
        let (mut cur, mut max) = (0, 0);
        // SAFETY: remaps the mapping just made at an address the kernel
        // chooses, sharing its pages.
        let kr = unsafe {
            mach_vm_remap(
                mach_task_self(),
                &mut rx,
                AREA_BYTES as u64,
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
        // SAFETY: `rx` is the view just made, AREA_BYTES long.
        if kr != 0 || unsafe { mprotect(rx as *mut u8, AREA_BYTES, PROT_READ | PROT_EXEC) } != 0 {
            // SAFETY: unmapping what was mapped above, which nothing holds.
            unsafe {
                if kr == 0 {
                    munmap(rx as *mut u8, AREA_BYTES);
                }
                munmap(rw, AREA_BYTES);
            }
            return None;
        }
        Some(Area {
            rw: rw as usize,
            rx: rx as usize,
            top: Mutex::new(0),
        })
    }

    /// The word at `at`, an 8-aligned address in the area's read+write view
    /// or in an object `object` made.
    pub(super) fn get(at: usize) -> u64 {
        // SAFETY: callers give only addresses of words `take` or `object`
        // handed out, which are mapped, aligned, and never freed.
        unsafe { std::ptr::read_volatile(at as *const u64) }
    }

    pub(super) fn put(at: usize, w: u64) {
        // SAFETY: as `get`.
        unsafe { std::ptr::write_volatile(at as *mut u64, w) }
    }

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

    /// Where a routine's stub goes back to the trampoline.
    pub(super) fn leave() -> usize {
        kayak_jit_leave as *const () as usize
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
    // d8–d15, the frame and link) kept on the stack; the convention's pinned
    // registers set from the state, and fuel given back to it after.
    std::arch::global_asm!(
        ".globl _kayak_jit_enter",
        ".p2align 2",
        "_kayak_jit_enter:",
        "stp x29, x30, [sp, #-160]!",
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
        "ldp d14, d15, [sp, #144]",
        "ldp d12, d13, [sp, #128]",
        "ldp d10, d11, [sp, #112]",
        "ldp d8, d9, [sp, #96]",
        "ldp x27, x28, [sp, #80]",
        "ldp x25, x26, [sp, #64]",
        "ldp x23, x24, [sp, #48]",
        "ldp x21, x22, [sp, #32]",
        "ldp x19, x20, [sp, #16]",
        "ldp x29, x30, [sp], #160",
        "ret",
        // A routine kayak does not have yet: its number in x16, kept as the
        // trap; Rust's stack as the trampoline left it; back as from code.
        ".globl _kayak_jit_leave",
        ".p2align 2",
        "_kayak_jit_leave:",
        "str x16, [x24, #32]",
        "ldr x17, [x24, #0]",
        "mov sp, x17",
        "mov x0, #0",
        "b 1b",
    );
}

#[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
mod sys {
    use super::Area;

    pub(super) fn map_area() -> Option<Area> {
        None
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
    pub(super) fn enter(_: usize, _: usize, _: u64, _: &[u64; 8], _: usize) -> u64 {
        panic!("native code: none on this machine")
    }
    pub(super) fn leave() -> usize {
        0
    }
}
