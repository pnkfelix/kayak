//! A program kc builds (`docs/research/kc.md`): its `main`, which runs the
//! program's `kc_entry` on a state of Fixpt's native convention (C451), and
//! the routines kc's code calls, `kayak-kcrt-core`'s (C452: one copy, which
//! kc's code run in process calls too). Linked statically into a program,
//! or shared, into a procedural macro's transformer (C447).
#![allow(unsafe_code)]

pub use kayak_kcrt_core::*;

unsafe extern "C" {
    /// The program: the heap's first free word and its end, and the state
    /// kc's code keeps in `%r15` (C451, Fixpt's native convention's).
    fn kc_entry(heap: *mut u64, end: *mut u64, state: *const u64) -> u64;
    /// A procedure of kc's code (its address `f`) called on `a` and `b`,
    /// the heap where `kc_entry` left it (C448: kc's entry object's; its
    /// end now the collector's, C453, `end` not read), and the state.
    fn kc_call2(f: u64, a: u64, b: u64, end: u64, state: *const u64) -> u64;
    /// `%phase1-apply`'s address, if the program defines it, else 0.
    static kc_apply_entry: u64;
}

/// The heap's first space, its words: collected (C453), grown as what
/// lives needs.
const HEAP_WORDS: usize = 1 << 23;

/// The state's words, and its fields kc's code reads (Fixpt's native
/// convention's `state`, kayak-jit's `state::…`): the stack's limit, the
/// fuel, the code distance, the fuel's reserve and the `poll` routine.
const STATE_WORDS: usize = 83;
const STATE_STACK_LIMIT: usize = 16;
const STATE_FUEL: usize = 24;
const STATE_DELTA: usize = 208;
const STATE_RESERVE: usize = 552;
const STATE_POLL: usize = 560;
/// Fuel given at each `poll`, as kayak-jit gives it.
const SLICE: u64 = 1 << 20;
/// The program's stack (C451): a thread's of this size, its limit what
/// kc's code checks at each entry, with room below it for what a failure
/// calls.
const STACK_BYTES: usize = 64 << 20;
const STACK_ROOM: usize = 1 << 20;

// `poll` (C451), as kayak-jit's: the next slice of fuel from the reserve,
// every register kept, as Fixpt's convention has a routine keep them.
std::arch::global_asm!(
    ".globl kc_poll_routine",
    ".p2align 4",
    "kc_poll_routine:",
    "push rax",
    "mov rax, [r15 + 552]",
    "mov [r15 + 24], rax",
    "pop rax",
    "ret",
);

unsafe extern "C" {
    fn kc_poll_routine();
}

/// A state for kc's code (C451), as kayak-jit's is in process (`kc run`):
/// there the code runs from an executable view of its pages, a cell
/// written through the writable view, at the code distance; here the code
/// is the executable's own, so the distance is of no pages, `-4` as
/// Fixpt's counts it from a reference (its tag 4). The stack's limit
/// `limit`; fuel, and `poll` to give more.
fn new_state(limit: usize) -> *const u64 {
    let s: &'static mut [u64] = Vec::leak(vec![0u64; STATE_WORDS]);
    s[STATE_DELTA / 8] = (-4i64) as u64;
    s[STATE_STACK_LIMIT / 8] = limit as u64;
    s[STATE_FUEL / 8] = SLICE;
    s[STATE_RESERVE / 8] = SLICE;
    s[STATE_POLL / 8] = kc_poll_routine as *const () as usize as u64;
    s.as_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn main(_argc: i32, _argv: *const *const u8) -> i32 {
    // The program on a stack of a known size (C451), so that kc's code
    // checks its limit.
    match std::thread::Builder::new()
        .stack_size(STACK_BYTES)
        .spawn(program)
    {
        Ok(t) => t.join().unwrap_or(1),
        Err(_) => failed("kc: no thread for the program"),
    }
}

/// The program's run: its value, or its `--apply` (below).
fn program() -> i32 {
    let here = 0u8;
    let limit = (&here as *const u8 as usize).saturating_sub(STACK_BYTES - STACK_ROOM);
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
    let state = new_state(limit);
    // SAFETY: `kc_entry` is kc's code, which takes the heap's first free
    // word and its end and the state, and writes each top-level
    // expression's value.
    unsafe { kc_entry(range.start, range.end, state) };
    if apply {
        let (tag, used) = input.split_once('\n').unwrap_or((input.as_str(), ""));
        // SAFETY: kc's entry object defines it, a word.
        let f = unsafe { kc_apply_entry };
        if f == 0 {
            failed("kc: this program has no transformer to apply");
        }
        // SAFETY: `f` is a procedure of kc's code of two arguments, strings
        // of Fixpt's layout; `kc_call2` gives it the heap `kc_entry` left.
        let v = unsafe {
            kc_call2(
                f,
                make_string(used),
                make_string(tag),
                range.end as u64,
                state,
            )
        };
        println!("{}", written(v));
    }
    0
}
