//! What the Rust that KFX-0 generates calls (`PLAN.md` §2).
//!
//! Every KFX value is a `Copy` Rust value: a scalar, or a word into the
//! heap. Since C511 the heap is collected (`obj::collect`): an object lives
//! while a word the collector sees reaches it (the stack and registers, a
//! registered block, what is kept, another object), and moves only where
//! no ambiguous word does. Rust that keeps a value where the collector
//! sees none (a `Vec` of its own) across an allocation holds a
//! `obj::space::NoCollect` while it does. Each primitive is named by the
//! mangling of its FX-26 name: `string->list` is `string_to_list`, `char=?`
//! is `char_eq_p`, `i64+` is `i64_plus`.
//!
//! Strings are of characters, as FX-26's are: a `Str` knows whether it is
//! ASCII, so that indexing one that is costs nothing, and indexing one that
//! is not counts characters (`CLAUDES_CHOICES.md`, C14).

mod arena;
mod array;
pub mod async_io;
pub mod async_process;
mod blob;
#[cfg(feature = "heap-stats")]
mod census;
mod closure;
mod eqtable;
mod escape;
mod fixed;
mod float;
mod handle;
mod heap;
mod int;
pub mod marks;
mod native;
mod obj;
mod once;
pub mod process;
mod reps;
mod show;
mod stats;
mod sum;
pub mod suspend;
mod syscall;
mod text;
mod trace;
pub mod value;
mod world;

pub use arena::*;
pub use array::*;
pub use blob::*;
pub use closure::*;
pub use eqtable::*;
pub use escape::*;
pub use fixed::*;
pub use float::*;
pub use handle::Handle;
pub use heap::*;
pub use int::*;
/// For a runtime's binary whose heap native code allocates in (C227).
pub use kayak_jit::{HostForeign, set_host_foreign};
#[cfg(feature = "unsafe-par")]
pub use kayak_sync::Global;
pub use marks::{MarkKey, first_mark, make_continuation_mark_key, with_mark, with_mark_async};
pub use native::*;
pub use once::*;
pub use process::{Addr, ICell, Inbox, icell_get, icell_put_x, make_icell};
pub use reps::*;
pub use show::*;
pub use stats::heap_stats_report;
use std::cell::Cell;
pub use std::cell::OnceCell;
pub use sum::*;
pub use syscall::*;
pub use text::*;
pub use trace::{Captures, CensusRow, Held, Product, Slot, Trace, Tracer, Visit, traced};
pub use value::{
    array_p, bool_p, bytevector_p, char_p, datum_list_to_vector, f32_p, f64_p, int_p, pair_p,
    procedure_p, product_p, ref_p, string_p, sum_p, symbol_p, vector_p,
};
pub use world::*;

pub use obj::{Str, StrLit, bytes_lit_made, string_words};

impl std::fmt::Display for Str {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::fmt::Debug for Str {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

/// `utf8`, Fixpt's UTF-8 text (its 279c16cf; C525): frozen, made whole.
/// Held as a string's object for now, which kayak-rt's is UTF-8 already
/// (C482): a gap of the object's kind, 1 where Fixpt's is 49 (50 all
/// ASCII), to close as kayak-rt's `string` takes Fixpt's UTF-32 layout
/// and this its own (`TODO.md` §54's pin note). No KFX program tells the
/// two apart: `utf8?` and a `utf8` in a union or a datum are refused.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Utf8(pub Str);

impl std::fmt::Debug for Utf8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#utf8{:?}", self.0.as_str())
    }
}

/// `(ref T R)` in the heap (`RefAt`, `obj.rs`: one Fixpt word, to a box).
pub type Ref<T> = RefAt<'static, T>;

pub use obj::{FlatArray, HeapWord, ListAt, Prod, ProdObj, RefAt, RefObj};

/// A pair (C497): Fixpt's headerless two-word cell, its element's word
/// (its car, held: C490, C495) and the rest of the list's (its cdr), in
/// that order (C506: a dynamic value reads them by its word).
#[repr(C)]
pub struct Cons<'r, T: Trace> {
    car: Held<T>,
    pub cdr: ListAt<'r, T>,
}

impl<'r, T: Trace> Cons<'r, T> {
    #[inline(always)]
    pub fn new(car: T, cdr: ListAt<'r, T>) -> Self {
        Cons {
            car: Held::new(car),
            cdr,
        }
    }
    /// Its element.
    #[inline(always)]
    pub fn car(&self) -> T {
        self.car.get()
    }
    /// Its element as it holds it.
    #[inline(always)]
    pub fn car_held(&self) -> Held<T> {
        self.car
    }
}

/// `(listof T R)` in the heap (`ListAt`, `obj.rs`: one Fixpt word).
pub type List<T> = ListAt<'static, T>;

/// What kayak-rt keeps of its own that no root would show (a closure given
/// to native code): a block never freed, its words roots (`obj::roots`),
/// so it lives as long as anything wants, `'static` or a region's `'r`.
/// What a root shows is `leak_collected`'s, freed once no word reaches it.
#[inline]
pub fn leak<'a, T: 'a>(v: T) -> &'a T {
    stats::note::<T>(std::mem::size_of::<T>());
    let r: &'a T = Box::leak(Box::new(v));
    // Kept for the run, a root (`obj::roots`).
    obj::roots::kept(std::ptr::from_ref(r).addr(), std::mem::size_of::<T>());
    // An opaque block, for a census (C480): its size alone.
    #[cfg(feature = "heap-stats")]
    census::opaque(std::ptr::from_ref(r) as usize, std::mem::size_of::<T>());
    r
}

/// A KFX value allocated, as a handle (C478): a module's state (a pair's
/// and a cell's are the space's, C510), kept while a word reaches it
/// (C513).
#[inline]
pub fn alloc<'a, T: 'a>(v: T) -> Handle<'a, T> {
    Handle::made(leak_collected(v))
}

/// `v` kept while a word reaches it (`obj::roots`, C513), scanned by its
/// bytes: kayak-rt's own (a continuation, a mark's value, an escape's
/// payload), and a module's state.
pub(crate) fn leak_collected<'a, T: 'a>(v: T) -> &'a T {
    stats::note::<T>(std::mem::size_of::<T>());
    let r: &'a T = Box::leak(Box::new(v));
    obj::roots::kept_collected(r);
    #[cfg(feature = "heap-stats")]
    census::opaque(std::ptr::from_ref(r) as usize, std::mem::size_of::<T>());
    r
}

/// `v` in the heap's space, whatever its lifetimes (a pair, a cell): of
/// whole words, found there by a word, so not kept beside it.
#[inline]
fn space_alloc<'a, T: 'a>(v: T) -> Handle<'a, T> {
    stats::note::<T>(std::mem::size_of::<T>());
    Handle::made(obj::space::put(v))
}

/// `v` kept while a word reaches it, scanned as its type says
/// (`obj::roots`, C513): kayak-rt's own (an eq table).
pub(crate) fn leak_scanned<'a, T: obj::roots::Scanned + 'a>(v: T) -> &'a T {
    stats::note::<T>(std::mem::size_of::<T>());
    let r: &'a T = Box::leak(Box::new(v));
    obj::roots::kept_scanned(r);
    #[cfg(feature = "heap-stats")]
    census::opaque(std::ptr::from_ref(r) as usize, std::mem::size_of::<T>());
    r
}

/// `v` kept as `leak_scanned` keeps it, read by other threads (a mailbox,
/// an I-cell, a group): what it holds every collection pins whole
/// (`obj::roots::kept_shared`, C512).
pub(crate) fn leak_shared<'a, T: obj::roots::Scanned + 'a>(v: T) -> &'a T {
    stats::note::<T>(std::mem::size_of::<T>());
    let r: &'a T = Box::leak(Box::new(v));
    obj::roots::kept_shared(r);
    #[cfg(feature = "heap-stats")]
    census::opaque(std::ptr::from_ref(r) as usize, std::mem::size_of::<T>());
    r
}

/// `parts`, one after another, as one string kept for the run.
pub fn str_of_parts(parts: &[&str]) -> Str {
    let n: usize = parts.iter().map(|p| p.len()).sum();
    str_made(n, |bytes| {
        let mut at = 0;
        for p in parts {
            bytes[at..at + p.len()].copy_from_slice(p.as_bytes());
            at += p.len();
        }
    })
}

/// A string of `n` bytes, which `fill` writes, kept for the run: its
/// object, header and bytes (C482, `obj.rs`).
fn str_made(n: usize, fill: impl FnOnce(&mut [u8])) -> Str {
    stats::note::<Str>(n);
    if n == 0 {
        return str_lit!("");
    }
    match obj::string_made(n, fill) {
        Some(s) => s,
        None => fail("a string's parts are not UTF-8"),
    }
}

#[inline]
pub fn leak_string(s: String) -> Str {
    str_of_parts(&[&s])
}

/// FX-26's `error`: a run-time error, which ends the run.
/// FX-26's `void`, the type of no value, as Rust's stable empty type: an
/// enum of no variants. Where a `void` is made (`error`, which returns
/// `!`), `!` coerces to it; nothing else makes one.
#[derive(Clone, Copy)]
pub enum Void {}

#[inline]
pub fn error(message: Str) -> ! {
    fail(message.as_str())
}

/// A run-time error of the runtime's own, as `error` is a program's: a
/// panic, which the program's hook says; in a monitored process (C108),
/// one no hook sees, since its watcher is told instead.
pub(crate) fn fail(message: &str) -> ! {
    if process::monitored() {
        std::panic::resume_unwind(Box::new(message.to_string()))
    }
    panic!("{message}")
}

thread_local! {
    /// How many `attempt`s the running code is inside (C415).
    static ATTEMPTING: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

static QUIET_HOOK: std::sync::Once = std::sync::Once::new();

/// FX-26's `(attempt e on-failure)` (C415; Fixpt's `docs/fx26.md`,
/// `attempt`): `body`'s value, or, should it fail (`error`, a trap that
/// panics), `on_failure` of what the failure says. The checker has made
/// `body` cancellable (it writes nothing another could see), so unwinding
/// out of it half way leaves nothing to undo. A failure caught is not
/// written: the panic hook says only those no `attempt` is around.
pub fn attempt<T>(body: impl FnOnce() -> T, on_failure: impl Fn(Str) -> T) -> T {
    quiet_hook();
    ATTEMPTING.with(|a| a.set(a.get() + 1));
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(body));
    ATTEMPTING.with(|a| a.set(a.get() - 1));
    match r {
        Ok(v) => v,
        Err(e) => on_failure(leak_string(panic_message(e))),
    }
}

/// `attempt` in an `async` function (C415, the async mode): `body` polled,
/// each poll inside `catch_unwind`, so that a failure in it, whenever it
/// comes, is `on_failure`'s of its message.
pub async fn attempt_async<T, H: std::future::Future<Output = T>>(
    body: impl std::future::Future<Output = T>,
    on_failure: impl FnOnce(Str) -> H,
) -> T {
    struct Caught<F>(std::pin::Pin<Box<F>>);
    impl<F: std::future::Future> std::future::Future for Caught<F> {
        type Output = Result<F::Output, Box<dyn std::any::Any + Send>>;
        fn poll(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            ATTEMPTING.with(|a| a.set(a.get() + 1));
            let r =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.0.as_mut().poll(cx)));
            ATTEMPTING.with(|a| a.set(a.get() - 1));
            match r {
                Ok(std::task::Poll::Ready(v)) => std::task::Poll::Ready(Ok(v)),
                Ok(std::task::Poll::Pending) => std::task::Poll::Pending,
                Err(e) => std::task::Poll::Ready(Err(e)),
            }
        }
    }
    quiet_hook();
    match (Caught(obj::roots::pinned(body))).await {
        Ok(v) => v,
        Err(e) => on_failure(leak_string(panic_message(e))).await,
    }
}

fn panic_message(e: Box<dyn std::any::Any + Send>) -> String {
    e.downcast_ref::<String>()
        .cloned()
        .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_default()
}

fn quiet_hook() {
    QUIET_HOOK.call_once(|| {
        let said = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            if ATTEMPTING.with(|a| a.get()) == 0 {
                said(info)
            }
        }));
    });
}

/// A `letrec`'s knot: a procedure's place, empty until the procedure is
/// made, so that the procedures of a group, closures that cannot name
/// themselves, can name each other through it. FX-26's `letrec` binds only
/// lambdas, so no knot is read before it is tied; tied once, it is `Sync`
/// when the procedure is (C102), so a closure holding it may cross.
///
/// Since C511 a box of the space (`obj::Knot`), where it was a block kept
/// for the run, a root: every closure a knot held, and all it held, was
/// kept and pinned for the run.
pub use obj::{Knot, knot};

// Lists.

#[inline]
pub fn nil<'r, T: Trace>() -> ListAt<'r, T> {
    ListAt::NIL
}

#[inline]
pub fn cons<'r, T: Trace>(car: T, cdr: ListAt<'r, T>) -> ListAt<'r, T> {
    // In the space, as `heap_cons`'s, whatever its lifetimes: no `Box`, so
    // nothing kept beside it for the roots (C510).
    ListAt::of_pair(space_alloc(Cons::new(car, cdr)))
}

#[inline]
pub fn car<T: Trace>(xs: ListAt<'_, T>) -> T {
    match xs.pair() {
        Some(p) => p.car(),
        None => fail("car or cdr of a non-pair"),
    }
}

#[inline]
pub fn cdr<'r, T: Trace>(xs: ListAt<'r, T>) -> ListAt<'r, T> {
    match xs.pair() {
        Some(p) => p.cdr,
        None => fail("car or cdr of a non-pair"),
    }
}

#[inline]
pub fn null_p<T: Trace>(xs: ListAt<'_, T>) -> bool {
    xs.is_nil()
}

// Booleans.

#[inline]
pub fn not(b: bool) -> bool {
    !b
}

// `i64`: FX-26's fixed-width arithmetic wraps, as Rust's `wrapping_*`
// does: `MIN / -1` is `MIN`, and a shift counts modulo 64. Division by zero
// is an error in both.

#[inline]
pub fn i64_plus(a: i64, b: i64) -> i64 {
    a.wrapping_add(b)
}

#[inline]
pub fn i64_minus(a: i64, b: i64) -> i64 {
    a.wrapping_sub(b)
}

#[inline]
pub fn i64_star(a: i64, b: i64) -> i64 {
    a.wrapping_mul(b)
}

#[inline]
pub fn i64_quotient(a: i64, b: i64) -> i64 {
    if b == 0 {
        fail("i64-quotient: division by zero")
    } else {
        a.wrapping_div(b)
    }
}

#[inline]
pub fn i64_remainder(a: i64, b: i64) -> i64 {
    if b == 0 {
        fail("i64-remainder: division by zero")
    } else {
        a.wrapping_rem(b)
    }
}

#[inline]
pub fn i64_and(a: i64, b: i64) -> i64 {
    a & b
}

#[inline]
pub fn i64_or(a: i64, b: i64) -> i64 {
    a | b
}

#[inline]
pub fn i64_xor(a: i64, b: i64) -> i64 {
    a ^ b
}

#[inline]
pub fn i64_not(a: i64) -> i64 {
    !a
}

#[inline]
pub fn i64_eq(a: i64, b: i64) -> bool {
    a == b
}

#[inline]
pub fn i64_lt(a: i64, b: i64) -> bool {
    a < b
}

#[inline]
pub fn i64_gt(a: i64, b: i64) -> bool {
    a > b
}

#[inline]
pub fn i64_lt_eq(a: i64, b: i64) -> bool {
    a <= b
}

#[inline]
pub fn i64_gt_eq(a: i64, b: i64) -> bool {
    a >= b
}

// The prelude (`kfx/prelude.fx`): FX-26's `int` primitives, at `i64`.

/// Characters, as FX-26 counts: bytes, if the string is ASCII.
#[inline]
pub fn string_length_i64(s: Str) -> i64 {
    if s.is_ascii() {
        s.as_str().len() as i64
    } else {
        s.as_str().chars().count() as i64
    }
}

thread_local! {
    /// Where the last character counted to in a non-ASCII string was: the
    /// string's text's address and length, the character, its byte. A
    /// program reading a string through (a line at a time, a character at
    /// a time) counts each from the last rather than from the start.
    static CURSOR: Cell<(usize, usize, usize, usize)> = const { Cell::new((0, 0, 0, 0)) };
}

/// Where the `i`th character of `s` starts, in bytes; `s.len()` for the
/// character after the last; `None` past that.
pub fn char_byte(s: Str, i: usize) -> Option<usize> {
    let t = s.as_str();
    if s.is_ascii() {
        return (i <= t.len()).then_some(i);
    }
    let key = (t.as_ptr() as usize, t.len());
    let (ptr, len, ci, bi) = CURSOR.get();
    let (mut ci, mut bi) = if (ptr, len) == key && i.abs_diff(ci) < i {
        (ci, bi)
    } else {
        (0, 0)
    };
    let b = t.as_bytes();
    while ci < i {
        if bi >= t.len() {
            return None;
        }
        bi += 1;
        while bi < t.len() && (b[bi] & 0xc0) == 0x80 {
            bi += 1;
        }
        ci += 1;
    }
    while ci > i {
        bi -= 1;
        while (b[bi] & 0xc0) == 0x80 {
            bi -= 1;
        }
        ci -= 1;
    }
    CURSOR.set((key.0, key.1, ci, bi));
    Some(bi)
}

/// The cursor forgotten: its string may have moved, and another be made
/// where it was (a collection, C511).
pub(crate) fn forget_cursor() {
    CURSOR.set((0, 0, 0, 0));
}

/// That the `i`th character of `s` starts at byte `b`, for `char_byte`.
pub(crate) fn cursor_at(s: Str, i: usize, b: usize) {
    if !s.is_ascii() {
        CURSOR.set((s.as_str().as_ptr() as usize, s.as_str().len(), i, b));
    }
}

/// The `i`th character: at once if the string is ASCII, else counted to
/// (from the last counted to, `char_byte`).
#[inline]
pub fn string_ref_i64(s: Str, i: i64) -> char {
    let found = usize::try_from(i).ok().and_then(|i| {
        if s.is_ascii() {
            s.as_str().as_bytes().get(i).map(|&b| b as char)
        } else {
            char_byte(s, i).and_then(|b| s.as_str()[b..].chars().next())
        }
    });
    found.unwrap_or_else(|| string_index_failed(s))
}

/// `string-ref` past a string's end, as Fixpt says it (C421).
#[cold]
fn string_index_failed(s: Str) -> ! {
    fail(&format!(
        "string index out of range (length {})",
        string_length_i64(s)
    ))
}

#[inline]
pub fn char_to_i64(c: char) -> i64 {
    c as i64
}

/// FX-26's `integer->char`, at `i64`: a code that is no character fails.
#[inline]
pub fn i64_to_char(n: i64) -> char {
    match u32::try_from(n).ok().and_then(char::from_u32) {
        Some(c) => c,
        None => fail("integer->char: no character has that code"),
    }
}

#[inline]
pub fn i64_to_string(n: i64) -> Str {
    // Written on the stack, then kept (C365): `to_string` was a `malloc`.
    let mut buf = [0u8; 20];
    let mut at = buf.len();
    let mut m = n.unsigned_abs();
    loop {
        at -= 1;
        buf[at] = b'0' + (m % 10) as u8;
        m /= 10;
        if m == 0 {
            break;
        }
    }
    let digits = std::str::from_utf8(&buf[at..]).unwrap_or("");
    str_of_parts(&[if n < 0 { "-" } else { "" }, digits])
}

// Mutable data: references and flat arrays. Regions are erased (every
// one's place is the heap, so every reference `'static`); effects too.

#[inline]
pub fn new<'r, T: Trace>(v: T) -> RefAt<'r, T> {
    RefAt::of_obj(space_alloc(RefObj::new(v)))
}

#[inline]
pub fn get<T: Trace>(r: RefAt<'_, T>) -> T {
    r.get()
}

#[inline]
pub fn set<T: Trace>(r: RefAt<'_, T>, v: T) {
    r.set(v)
}

/// Elements from which a flat array of 0s is reserved rather than written:
/// committed as it is touched, so that one used little costs little. From
/// 2^16 (512 KB; C243), so that tier 1's stacks, 1 and 2 MB of which a
/// run touches what it uses, are not written at every start; not lower, a
/// reservation being a mapping of its own, of which a process may have
/// some 65000.
const BIG_FLATARRAY: usize = 1 << 16;

#[inline]
pub fn make_flatarray_i64(n: i64, v: i64) -> FlatArray<i64> {
    use kayak_jit::layout as l;
    match usize::try_from(n) {
        Ok(n) => {
            let prefix = [
                l::header(l::KIND_FLAT_ARRAY, 2, 8 * n, false, false),
                l::FLAT_I64 << l::FIXNUM_SHIFT,
                2 << 3 | l::TAG_TRAILER,
            ];
            // A big one of 0s reserved, committed as it is touched (C139).
            if v == 0 && n >= BIG_FLATARRAY {
                let prefix = prefix.map(|w| w as i64);
                if let Some(all) = kayak_jit::zeroed_cells(&prefix, n) {
                    return FlatArray::of_cells(all);
                }
            }
            // In the heap's space (C507).
            let all = obj::kept_cells::<i64>(prefix.len() + n);
            for (c, &w) in all.iter().zip(&prefix) {
                c.set(w as i64);
            }
            if v != 0 {
                for c in &all[prefix.len()..] {
                    c.set(v);
                }
            }
            FlatArray::of_cells(all)
        }
        Err(_) => fail("make-flatarray: a negative length"),
    }
}

fn element<T>(a: FlatArray<T>, i: i64) -> &'static Cell<T> {
    match usize::try_from(i).ok().and_then(|i| a.cells().get(i)) {
        Some(c) => c,
        None => fail("flatarray: index out of range"),
    }
}

#[inline]
pub fn flatarray_ref_i64(a: FlatArray<i64>, i: i64) -> i64 {
    element(a, i).get()
}

#[inline]
pub fn flatarray_set_i64_x(a: FlatArray<i64>, i: i64, v: i64) {
    element(a, i).set(v)
}

#[inline]
pub fn flatarray_length_i64(a: FlatArray<i64>) -> i64 {
    a.len() as i64
}

// Characters.

#[inline]
pub fn char_eq_p(a: char, b: char) -> bool {
    a == b
}

#[inline]
pub fn char_whitespace_p(c: char) -> bool {
    c.is_whitespace()
}

/// Any numeric character, as Fixpt's (`²`, `½`, `٣` too).
#[inline]
pub fn char_numeric_p(c: char) -> bool {
    c.is_numeric()
}

#[inline]
pub fn char_alphabetic_p(c: char) -> bool {
    c.is_alphabetic()
}

/// Unicode's upper case, its first character where it is more than one
/// (`ß` is `S`), as Fixpt's.
#[inline]
pub fn char_upcase(c: char) -> char {
    c.to_uppercase().next().unwrap_or(c)
}

#[inline]
pub fn char_to_string(c: char) -> Str {
    str_of_parts(&[c.encode_utf8(&mut [0u8; 4])])
}

// Strings.

#[inline]
pub fn string_eq_p(a: Str, b: Str) -> bool {
    a == b
}

#[inline]
pub fn string_append(a: Str, b: Str) -> Str {
    str_of_parts(&[a.as_str(), b.as_str()])
}

/// kfx's `string-list-append` (C476; `kfx/base.kfx`): the strings of `xs`
/// joined, in order, one new string: each copied once, where the KFX
/// joins them pairwise, a new string each round.
pub fn strings_joined(xs: List<Str>) -> Str {
    // Values in a `Vec` of its own, which no root sees, across allocations.
    let _held = crate::obj::space::NoCollect::new();
    let mut parts: Vec<&str> = Vec::new();
    let mut at = xs;
    while let Some(c) = at.pair() {
        parts.push(c.car().as_str());
        at = c.cdr;
    }
    match parts.len() {
        0 => str_lit(""),
        1 => at_one(xs),
        _ => str_of_parts(&parts),
    }
}

/// The one string of a list of one, itself, as the KFX gives it.
fn at_one(xs: List<Str>) -> Str {
    xs.pair().expect("a list of one").car()
}

/// `nat`s, which are `Int`s: a string's length in characters, a list's.
#[inline]
pub fn string_length(s: Str) -> Int {
    i64_to_int(string_length_i64(s))
}

/// `string-ref` at an `int`, as FX-26 has it.
#[inline]
pub fn string_ref(s: Str, i: Int) -> char {
    if _lt(i, int_lit(0)) || _gt(i, int_lit(i64::MAX)) {
        string_index_failed(s)
    }
    string_ref_i64(s, int_to_i64(i))
}

#[inline]
pub fn length<T: Trace>(xs: ListAt<'_, T>) -> Int {
    list_length(xs)
}

#[inline]
pub fn list_length<T: Trace>(mut xs: ListAt<'_, T>) -> Int {
    let mut n = 0i64;
    while let Some(p) = xs.pair() {
        n += 1;
        xs = p.cdr;
    }
    i64_to_int(n)
}

#[inline]
pub fn reverse<'b, T: Trace + 'b>(mut xs: ListAt<'_, T>) -> ListAt<'b, T> {
    let mut out = nil();
    while let Some(p) = xs.pair() {
        out = cons(p.car(), out);
        xs = p.cdr;
    }
    out
}

#[inline]
pub fn append<'b, T: Trace + 'b>(xs: ListAt<'_, T>, ys: ListAt<'b, T>) -> ListAt<'b, T> {
    // Values in a `Vec` of its own, which no root sees, across allocations.
    let _held = crate::obj::space::NoCollect::new();
    let mut items = Vec::new();
    let mut p = xs;
    while let Some(c) = p.pair() {
        items.push(c.car());
        p = c.cdr;
    }
    items.into_iter().rev().fold(ys, |rest, x| cons(x, rest))
}

/// A string of a Rust text: its object made now. A literal is `str_lit!`'s,
/// made at compile time (C482), so that one in a loop costs nothing.
#[inline]
pub fn str_lit(text: &str) -> Str {
    str_of_parts(&[text])
}

#[inline]
pub fn string_to_list(s: Str) -> List<char> {
    s.as_str()
        .chars()
        .rev()
        .fold(nil(), |rest, c| cons(c, rest))
}

#[inline]
pub fn list_to_string(cs: List<char>) -> Str {
    // Measured, then written in place (C365): a `String` grown a character
    // at a time was a `malloc` and `realloc`s.
    let mut n = 0;
    let mut at = cs;
    while let Some(p) = at.pair() {
        n += p.car().len_utf8();
        at = p.cdr;
    }
    str_made(n, |bytes| {
        let (mut i, mut at) = (0, cs);
        while let Some(p) = at.pair() {
            i += p.car().encode_utf8(&mut bytes[i..]).len();
            at = p.cdr;
        }
    })
}

/// Runs `f` on a thread with a stack of 1 GiB: KFX-0 has no loops, and
/// Rust no tail calls, so a walk of a long list is as deep as the list.
#[inline]
pub fn with_big_stack<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(1 << 30)
        .spawn(move || {
            // The run's stack's top, for a census's mark (C480).
            #[cfg(feature = "heap-stats")]
            {
                let top = 0u8;
                census::stack_top(std::ptr::from_ref(&top) as usize + 64);
            }
            f()
        })
        .expect("a thread for the run")
        .join()
        .unwrap_or_else(|_| std::process::exit(1))
}
