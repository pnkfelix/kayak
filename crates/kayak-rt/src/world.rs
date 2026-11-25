//! FX-26's bytes and I/O (Fixpt 17ac0c5, `docs/fx26.md`, "Bytevectors" and
//! "I/O: the `io` effects"; `CLAUDES_CHOICES.md` C49).
//!
//! A bytevector is a row of cells, as a flat array is: FX-26's may be
//! aliased and written through any alias. The I/O primitives answer with
//! FX-26's structural sums and products, which a program's own Rust would
//! declare (`kfx/emit-tops.kfx`, `put-shape-decls`); these are declared
//! here instead, the same way, and the emitter leaves them out
//! (`standard-shape?`), so a program and kayak-rt name one type.

use crate::{
    Holds, Int, List, Show, Str, Sum, SumOf, Void, fail, heap_cons, i64_to_int, int_to_i64,
    leak_string, nil, sum_of,
};
use std::cell::Cell;
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::sync::{Condvar, Mutex, MutexGuard, Once};
use std::time::{Duration, Instant};

// The standard shapes, as `put-shape-decls` writes a program's.

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub struct Product_4rows_7columns<A0, A1>(pub A0, pub A1);

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub struct Product_4kind_4size_8modified_4mode<A0, A1, A2, A3>(pub A0, pub A1, pub A2, pub A3);

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub struct Product_4kind_6detail<A0, A1>(pub A0, pub A1);

impl<A0, A1> Show for Product_4kind_6detail<A0, A1> {
    fn show(&self, out: &mut String) {
        out.push_str("#<product of 2>");
    }
}

impl<A0, A1, A2, A3> Show for Product_4kind_4size_8modified_4mode<A0, A1, A2, A3> {
    fn show(&self, out: &mut String) {
        out.push_str("#<product of 4>");
    }
}

impl<A0, A1> Show for Product_4rows_7columns<A0, A1> {
    fn show(&self, out: &mut String) {
        out.push_str("#<product of 2>");
    }
}

/// `io-error`'s kind.
pub type IoKind = Sum;

// The products' `Trace` (C480, C481), each field's.
crate::trace_product!(Product_4rows_7columns, A0 0, A1 1);
crate::trace_product!(Product_4kind_4size_8modified_4mode, A0 0, A1 1, A2 2, A3 3);
crate::trace_product!(Product_4kind_6detail, A0 0, A1 1);

/// `io-error`: its kind, for a `tagcase`, and the system's words.
pub type IoError = Product_4kind_6detail<IoKind, Str>;

pub(crate) fn io_error(e: &std::io::Error) -> IoError {
    use std::io::ErrorKind as K;
    let kind: IoKind = match e.kind() {
        K::NotFound => sum_of(crate::tag_lit!("not-found"), ()),
        K::PermissionDenied => sum_of(crate::tag_lit!("permission-denied"), ()),
        K::AlreadyExists => sum_of(crate::tag_lit!("already-exists"), ()),
        K::NotADirectory => sum_of(crate::tag_lit!("not-a-directory"), ()),
        K::IsADirectory => sum_of(crate::tag_lit!("is-a-directory"), ()),
        K::BrokenPipe => sum_of(crate::tag_lit!("broken-pipe"), ()),
        K::InvalidData => sum_of(crate::tag_lit!("invalid-data"), ()),
        _ => sum_of(crate::tag_lit!("other"), ()),
    };
    Product_4kind_6detail(kind, leak_string(e.to_string()))
}

pub(crate) fn invalid_data(what: &str) -> IoError {
    io_error(&std::io::Error::new(std::io::ErrorKind::InvalidData, what))
}

// Bytevectors.

/// `(bytevectorof R)`: one word to an object of Fixpt's layout (C482,
/// `obj.rs`), its bytes the suffix, each a cell.
pub use crate::obj::{Bytes, BytesAt, BytesCells};

/// As Fixpt writes one, R7RS's `#u8(…)`.
impl Show for BytesAt<'_> {
    fn show(&self, out: &mut String) {
        out.push_str("#u8(");
        for (i, c) in self.cells().iter().enumerate() {
            if i > 0 {
                out.push(' ');
            }
            out.push_str(&c.get().to_string());
        }
        out.push(')');
    }
}

pub(crate) fn bytes_of(v: Vec<u8>) -> Bytes {
    crate::stats::note::<Bytes>(v.len());
    crate::obj::bytes_at(crate::obj::bytes_kept(&v))
}

/// Bytes kept in region `r`'s arena (C267).
fn bytes_in<'r, R: Holds<BytesCells>>(r: &'r R, v: Vec<u8>) -> BytesAt<'r> {
    crate::obj::bytes_at(r.arena().alloc(crate::obj::bytes_object(&v)).to_ref())
}

fn index(n: Int, what: &str) -> usize {
    match usize::try_from(int_to_i64(n)) {
        Ok(i) => i,
        Err(_) => fail(what),
    }
}

pub(crate) fn span(b: BytesAt<'_>, start: Int, end: Int, what: &str) -> (usize, usize) {
    let (s, e) = (index(start, what), index(end, what));
    if s > e || e > b.cells().len() {
        fail(what)
    }
    (s, e)
}

pub(crate) fn to_vec(b: BytesAt<'_>) -> Vec<u8> {
    b.cells().iter().map(Cell::get).collect()
}

pub fn make_bytevector(n: Int, v: u8) -> Bytes {
    bytes_of(vec![v; index(n, "make-bytevector: a negative length")])
}

/// The same, in region `r` (C267).
pub fn make_bytevector_in<'r, R: Holds<BytesCells>>(r: &'r R, n: Int, v: u8) -> BytesAt<'r> {
    bytes_in(r, vec![v; index(n, "make-bytevector: a negative length")])
}

pub fn bytevector_length(b: BytesAt<'_>) -> Int {
    i64_to_int(b.cells().len() as i64)
}

pub fn bytevector_u8_ref(b: BytesAt<'_>, i: Int) -> u8 {
    match b
        .cells()
        .get(index(i, "bytevector-u8-ref: index out of range"))
    {
        Some(c) => c.get(),
        None => fail("bytevector-u8-ref: index out of range"),
    }
}

pub fn bytevector_u8_set_x(b: BytesAt<'_>, i: Int, v: u8) {
    match b
        .cells()
        .get(index(i, "bytevector-u8-set!: index out of range"))
    {
        Some(c) => c.set(v),
        None => fail("bytevector-u8-set!: index out of range"),
    }
}

pub fn bytevector_copy(b: BytesAt<'_>, start: Int, end: Int) -> Bytes {
    let (s, e) = span(b, start, end, "bytevector-copy: a span that does not fit");
    bytes_of(b.cells()[s..e].iter().map(Cell::get).collect())
}

/// The same, in region `r` (C267).
pub fn bytevector_copy_in<'r, R: Holds<BytesCells>>(
    r: &'r R,
    b: BytesAt<'_>,
    start: Int,
    end: Int,
) -> BytesAt<'r> {
    let (s, e) = span(b, start, end, "bytevector-copy: a span that does not fit");
    bytes_in(r, b.cells()[s..e].iter().map(Cell::get).collect())
}

/// R7RS's: `to`, `at`, `from`, `start`, `end`; overlapping spans copied
/// as if through a buffer.
pub fn bytevector_copy_x(to: BytesAt<'_>, at: Int, from: BytesAt<'_>, start: Int, end: Int) {
    let (s, e) = span(
        from,
        start,
        end,
        "bytevector-copy!: a span that does not fit",
    );
    let at = index(at, "bytevector-copy!: a span that does not fit");
    if at + (e - s) > to.cells().len() {
        fail("bytevector-copy!: a span that does not fit")
    }
    let moved: Vec<u8> = from.cells()[s..e].iter().map(Cell::get).collect();
    for (k, v) in moved.into_iter().enumerate() {
        to.cells()[at + k].set(v);
    }
}

pub fn bytevector_eq_p(a: BytesAt<'_>, b: BytesAt<'_>) -> bool {
    a.cells().len() == b.cells().len()
        && a.cells()
            .iter()
            .zip(b.cells())
            .all(|(x, y)| x.get() == y.get())
}

/// `string->utf8` (Fixpt's, since its 279c16cf; C525): the text, as
/// `utf8`. A copy in Fixpt; here the same object, which neither writes.
pub fn string_to_utf8(s: Str) -> crate::Utf8 {
    crate::Utf8(s)
}

/// `utf8->string`: the text, as a `string`.
pub fn utf8_to_string(u: crate::Utf8) -> Str {
    u.0
}

/// `utf8->bytes`: the text's bytes, a bytevector of the heap's.
pub fn utf8_to_bytes(u: crate::Utf8) -> Bytes {
    bytes_of(u.0.as_str().as_bytes().to_vec())
}

/// The same, in region `r` (C267).
pub fn utf8_to_bytes_in<'r, R: Holds<BytesCells>>(r: &'r R, u: crate::Utf8) -> BytesAt<'r> {
    bytes_in(r, u.0.as_str().as_bytes().to_vec())
}

/// `bytes->utf8`: bytes that are UTF-8, as `utf8`; else `(sum bad k)`,
/// `k` the offset they are valid up to.
pub fn bytes_to_utf8(b: BytesAt<'_>) -> Sum {
    match String::from_utf8(to_vec(b)) {
        Ok(s) => sum_of(crate::tag_lit!("text"), crate::Utf8(leak_string(s))),
        Err(e) => sum_of(
            crate::tag_lit!("bad"),
            i64_to_int(e.utf8_error().valid_up_to() as i64),
        ),
    }
}

// The world. Each write flushed, as Fixpt's are, so that the two streams
// interleave as written.

// The results that carry nothing new, or one of few values, made once,
// not at every call (C233): an I/O loop of a program bounded in space is
// bounded here too. Only a failure, or a result with text or data, is
// made anew.

/// The 256 results `read-byte` gives of a byte.
static BYTES: [SumOf<u8>; 256] = {
    let mut a = [const { SumOf::new(crate::tag_lit!("byte"), crate::Held::byte(0u8)) }; 256];
    let mut i = 0;
    while i < 256 {
        a[i] = SumOf::new(crate::tag_lit!("byte"), crate::Held::byte(i as u8));
        i += 1;
    }
    a
};

/// How many counts of `read-bytes` are made once.
const COUNTS_KEPT: usize = 4096;
static COUNTS: [SumOf<Int>; COUNTS_KEPT + 1] = {
    let mut a = [const { SumOf::new(crate::tag_lit!("count"), crate::Held::int(Int::fixnum(0))) };
        COUNTS_KEPT + 1];
    let mut i = 0;
    while i <= COUNTS_KEPT {
        a[i] = SumOf::new(
            crate::tag_lit!("count"),
            crate::Held::int(Int::fixnum(i as i64)),
        );
        i += 1;
    }
    a
};

pub(crate) fn counted(k: usize) -> Sum {
    if k <= COUNTS_KEPT {
        COUNTS[k].sum()
    } else {
        sum_of(crate::tag_lit!("count"), i64_to_int(k as i64))
    }
}

pub(crate) fn written(r: std::io::Result<()>) -> Sum {
    match r {
        Ok(()) => {
            static C: SumOf<()> = SumOf::new(crate::tag_lit!("done"), crate::Held::UNIT);
            C.sum()
        }
        Err(e) => sum_of(crate::tag_lit!("failed"), io_error(&e)),
    }
}

pub fn write_out(s: Str) -> Sum {
    if crate::async_io::installed() {
        return crate::async_io::write_now(1, s);
    }
    let mut out = std::io::stdout().lock();
    written(
        out.write_all(s.as_str().as_bytes())
            .and_then(|()| out.flush()),
    )
}

pub fn write_err(s: Str) -> Sum {
    if crate::async_io::installed() {
        return crate::async_io::write_now(2, s);
    }
    let mut err = std::io::stderr().lock();
    written(
        err.write_all(s.as_str().as_bytes())
            .and_then(|()| err.flush()),
    )
}

/// Ends the run at once, with `n`'s low byte.
pub fn exit(n: Int) -> Void {
    crate::async_io::drain();
    let _ = std::io::stdout().flush();
    std::process::exit((int_to_i64(n) & 0xff) as i32)
}

// The standard input (C109): one stream, read by a thread of its own into
// a buffer, so that reads by lines and by bytes take from it alike, and
// whether a read would wait is known without one (a `pselect`'s key arm).
// Safe Rust has no read that does not wait, nor a poll: the thread is how.

pub(crate) struct Input {
    /// Read and not yet taken.
    pub(crate) buf: VecDeque<u8>,
    /// The input is at its end, or failed, once `buf` is taken.
    pub(crate) ended: bool,
    /// The failure the next read gives, once `buf` is taken.
    pub(crate) error: Option<std::io::Error>,
}

static INPUT: Mutex<Input> = Mutex::new(Input {
    buf: VecDeque::new(),
    ended: false,
    error: None,
});
static ARRIVED: Condvar = Condvar::new();
/// Signalled when a read takes from the buffer: the reader waits for it
/// while `READ_AHEAD` bytes are unread, so that the input is never held
/// whole (C233: a program reading a long input in bounded space does so
/// here too).
static TAKEN: Condvar = Condvar::new();
pub(crate) const READ_AHEAD: usize = 1 << 16;
static READER: Once = Once::new();

pub(crate) fn input() -> MutexGuard<'static, Input> {
    INPUT.lock().unwrap_or_else(|p| p.into_inner())
}

/// The reader's thread, started at the first read or question: each
/// chunk as it comes, and the end; every waiter told.
fn reading() {
    // In the async mode, a task reads it, through the runner (C381).
    if crate::async_io::installed() {
        crate::async_io::want_input();
        return;
    }
    READER.call_once(|| {
        let started = std::thread::Builder::new()
            .name("kayak-stdin".to_string())
            .spawn(|| {
                let mut stdin = std::io::stdin();
                let mut chunk = [0u8; 4096];
                loop {
                    {
                        let mut i = input();
                        while i.buf.len() >= READ_AHEAD {
                            i = TAKEN.wait(i).unwrap_or_else(|p| p.into_inner());
                        }
                    }
                    let r = stdin.read(&mut chunk);
                    let mut i = input();
                    match r {
                        Ok(0) => i.ended = true,
                        Ok(n) => i.buf.extend(&chunk[..n]),
                        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                        Err(e) => {
                            i.error = Some(e);
                            i.ended = true;
                        }
                    }
                    let ended = i.ended;
                    drop(i);
                    ARRIVED.notify_all();
                    crate::process::input_changed();
                    if ended {
                        return;
                    }
                }
            });
        if let Err(e) = started {
            let mut i = input();
            i.error = Some(e);
            i.ended = true;
        }
    });
}

/// The input once a read would not wait: a byte there, or the end.
fn arrived() -> MutexGuard<'static, Input> {
    reading();
    let mut i = input();
    if crate::async_io::installed() && i.buf.is_empty() && !i.ended {
        drop(i);
        crate::fail(
            "a read of the standard input where nothing may wait (the async mode's `block`)",
        )
    }
    while i.buf.is_empty() && !i.ended {
        i = ARRIVED.wait(i).unwrap_or_else(|p| p.into_inner());
    }
    i
}

/// Whether a read of the standard input would not wait: a byte has come,
/// or it is at its end. Asked of the descriptor too, as Fixpt's
/// `stdin_ready` does (`tty::ready(0)`), so that input there before the
/// reader has read it counts (C232); then of the buffer again, which the
/// reader may have filled meanwhile.
pub fn stdin_ready_p() -> bool {
    reading();
    {
        let i = input();
        if !i.buf.is_empty() || i.ended || crate::async_io::installed() {
            return !i.buf.is_empty() || i.ended;
        }
    }
    if kayak_jit::stdin_readable() {
        return true;
    }
    let i = input();
    !i.buf.is_empty() || i.ended
}

/// Whether the standard input is ready, having waited until it is or the
/// clock reaches `until` (-1: no limit), whichever is first (Fixpt's
/// `World::wait_stdin`; the syscall `io:stdin-wait/1`, C232).
pub fn stdin_wait(until: i64) -> bool {
    reading();
    let mut i = input();
    loop {
        if !i.buf.is_empty() || i.ended {
            return true;
        }
        if until < 0 {
            i = ARRIVED.wait(i).unwrap_or_else(|p| p.into_inner());
            continue;
        }
        let now = clock();
        if now >= until {
            return false;
        }
        i = ARRIVED
            .wait_timeout(i, Duration::from_nanos((until - now) as u64))
            .unwrap_or_else(|p| p.into_inner())
            .0;
    }
}

/// The next line, its ending (`\n` or `\r\n`) left off; a line not UTF-8
/// an `invalid-data` failure, reading going on after it.
pub fn read_line() -> Sum {
    line_answer(read_line_bytes())
}

/// A line read, as `read-line` answers it.
pub(crate) fn line_answer(r: std::io::Result<Option<Vec<u8>>>) -> Sum {
    match r {
        Ok(None) => {
            static C: SumOf<()> = SumOf::new(crate::tag_lit!("eof"), crate::Held::UNIT);
            C.sum()
        }
        Ok(Some(line)) => match String::from_utf8(line) {
            Ok(s) => sum_of(crate::tag_lit!("line"), leak_string(s)),
            Err(_) => sum_of(crate::tag_lit!("failed"), invalid_data("a line not UTF-8")),
        },
        Err(e) => sum_of(crate::tag_lit!("failed"), io_error(&e)),
    }
}

/// The next line's bytes, its ending left off; none at the end; or the
/// failure the reader met.
pub(crate) fn read_line_bytes() -> std::io::Result<Option<Vec<u8>>> {
    let mut line = Vec::new();
    let mut i = arrived();
    loop {
        match i.buf.iter().position(|&b| b == b'\n') {
            Some(k) => {
                line.extend(i.buf.drain(..=k));
                break;
            }
            None => {
                line.extend(i.buf.drain(..));
                if i.ended {
                    break;
                }
                drop(i);
                // The reader may wait for the buffer to drain: a line
                // longer than it holds.
                TAKEN.notify_one();
                i = arrived();
            }
        }
    }
    if line.is_empty() {
        return match i.error.take() {
            Some(e) => Err(e),
            None => Ok(None),
        };
    }
    drop(i);
    TAKEN.notify_one();
    if line.last() == Some(&b'\n') {
        line.pop();
        if line.last() == Some(&b'\r') {
            line.pop();
        }
    }
    Ok(Some(line))
}

// Text as code points in a runtime's own words (C233): written from them,
// and a line read into them, so that no KFX string is made, which would
// never be freed: a machine's I/O loop then bounded in space. A failure is
// kept, for the machine to ask its kind and detail.
thread_local! {
    static FAILURE: std::cell::RefCell<Option<std::io::Error>> = const { std::cell::RefCell::new(None) };
    static POINTS: std::cell::RefCell<Vec<u32>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn failed_with(e: std::io::Error) {
    FAILURE.with(|f| *f.borrow_mut() = Some(e));
}

/// `n` code points from `words`, `per` to a word (1, or 2: the low half
/// first), written to the standard output (`stream` 1) or error (2): 0,
/// or -1, the failure kept.
pub fn write_points(words: &[u64], n: usize, per: usize, stream: i64) -> i64 {
    let mut s = String::with_capacity(n);
    for k in 0..n {
        let w = words[k / per];
        let cp = if per == 2 {
            (w >> (32 * (k % 2))) as u32
        } else {
            w as u32
        };
        s.push(char::from_u32(cp).unwrap_or('\u{fffd}'));
    }
    let r = if stream == 2 {
        let mut err = std::io::stderr().lock();
        err.write_all(s.as_bytes()).and_then(|()| err.flush())
    } else {
        let mut out = std::io::stdout().lock();
        out.write_all(s.as_bytes()).and_then(|()| out.flush())
    };
    match r {
        Ok(()) => 0,
        Err(e) => {
            failed_with(e);
            -1
        }
    }
}

/// The next line, as `read-line` reads it, its code points kept for
/// `take_points`: how many; -1 at the end; -2 a failure, kept.
pub fn read_line_points() -> i64 {
    match read_line_bytes() {
        Ok(None) => -1,
        Ok(Some(line)) => match String::from_utf8(line) {
            Ok(s) => POINTS.with(|p| {
                let mut p = p.borrow_mut();
                p.clear();
                p.extend(s.chars().map(|c| c as u32));
                p.len() as i64
            }),
            Err(_) => {
                failed_with(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "a line not UTF-8",
                ));
                -2
            }
        },
        Err(e) => {
            failed_with(e);
            -2
        }
    }
}

/// The line `read_line_points` kept, as words, `per` code points to a word.
pub fn taken_points(per: usize) -> Vec<u64> {
    POINTS.with(|p| {
        let p = p.borrow();
        if per == 2 {
            p.chunks(2)
                .map(|c| c[0] as u64 | c.get(1).map_or(0, |&h| (h as u64) << 32))
                .collect()
        } else {
            p.iter().map(|&c| c as u64).collect()
        }
    })
}

/// The failure kept: its kind, as FX-26's `io-error` orders them
/// (`not-found` 0 … `other` 7), and its detail.
pub fn failure_kind() -> i64 {
    use std::io::ErrorKind as K;
    FAILURE.with(|f| match f.borrow().as_ref().map(|e| e.kind()) {
        Some(K::NotFound) => 0,
        Some(K::PermissionDenied) => 1,
        Some(K::AlreadyExists) => 2,
        Some(K::NotADirectory) => 3,
        Some(K::IsADirectory) => 4,
        Some(K::BrokenPipe) => 5,
        Some(K::InvalidData) => 6,
        _ => 7,
    })
}
pub fn failure_detail() -> Str {
    FAILURE.with(|f| leak_string(f.borrow().as_ref().map_or(String::new(), |e| e.to_string())))
}

/// Machine code at `start`, `len` bytes, named `name`, its source lines
/// with it (C246): `pairs` an address and a source position (a character
/// offset into `file`) each, the code from that address on the
/// position's. With `KAYAK_JITDUMP` set, the positions made lines of
/// `file` and a jitdump's debug-info record written before the code's;
/// else as `code_noted`. With `KAYAK_JITDUMP=expr` (C247), each
/// expression's position a line of its own, of a side file (`side_file`).
pub fn code_lines_noted(start: i64, len: i64, name: &str, file: &str, pairs: &[u64]) {
    let Some(mode) = std::env::var_os("KAYAK_JITDUMP") else {
        return code_noted(start, len, name);
    };
    if len > 0 {
        let exprs = mode == "expr";
        // Each file's line starts, in characters, as the reader counts,
        // and the path its records name; none if the file cannot be read
        // (the listing's path relative to another directory), so no lines
        // rather than wrong ones.
        type Known = (String, Vec<usize>, String, u64);
        static KNOWN: Mutex<Vec<Known>> = Mutex::new(Vec::new());
        let mut known = KNOWN.lock().unwrap_or_else(|p| p.into_inner());
        if !known.iter().any(|k| k.0 == file) {
            let (starts, (path, n)) = match std::fs::read_to_string(file) {
                Err(_) => (Vec::new(), (String::new(), 0)),
                Ok(text) => {
                    let mut starts = vec![0];
                    for (i, c) in text.chars().enumerate() {
                        if c == '\n' {
                            starts.push(i + 1);
                        }
                    }
                    let path = std::fs::canonicalize(file)
                        .map_or(file.to_string(), |p| p.display().to_string());
                    if exprs {
                        (starts, side_file(&path, &text).unwrap_or_default())
                    } else {
                        (starts, (path, 0))
                    }
                }
            };
            known.push((file.to_string(), starts, path, n));
        }
        let (_, starts, path, n) = known.iter().find(|k| k.0 == file).expect("just noted");
        if !starts.is_empty() && !path.is_empty() {
            let lines: Vec<(u64, u32)> = pairs
                .as_chunks::<2>()
                .0
                .iter()
                .map(|&[at, pos]| {
                    if exprs {
                        (at, (n + 2 + pos) as u32)
                    } else {
                        (at, starts.partition_point(|&s| s <= pos as usize) as u32)
                    }
                })
                .collect();
            kayak_jit::jitdump::debug_info(start as usize, path, &lines);
        }
    }
    code_noted(start, len, name);
}

/// `KAYAK_JITDUMP=expr`'s side file for source `path` (C247), so that
/// a line of perf's tells an expression, not a whole line: its first N
/// lines the source's (N `text`'s lines), then a line saying so, then
/// line N + 2 + P for character position P: `LINE:COLUMN  TEXT`, the
/// line from 1, the column from 0, the text from there to the line's end
/// (cut at 60), at every position but a blank's, else empty. So a line of N or
/// less is the source's own, and one past N + 1 a position, which
/// `scripts/perf-expr.py` reads back. At `/tmp/jit-PID.NAME.expr`,
/// beside the jitdump. Its path and N; none if it cannot be written.
fn side_file(path: &str, text: &str) -> Option<(String, u64)> {
    let name = std::path::Path::new(path)
        .file_name()?
        .to_string_lossy()
        .into_owned();
    let side = format!("/tmp/jit-{}.{name}.expr", std::process::id());
    let n = text.lines().count() as u64;
    let mut out = String::with_capacity(4 * text.len());
    for l in text.lines() {
        out.push_str(l);
        out.push('\n');
    }
    out.push_str(&format!(
        ";;; kayak expr (C247): the {n} lines above are {path}'s; line {n} + 2 + P below, the expression at its character P\n"
    ));
    let cs: Vec<char> = text.chars().collect();
    let (mut line, mut col) = (1, 0);
    for (p, &c) in cs.iter().enumerate() {
        // Every position but a blank's: an entry's parts are at the
        // letters of its `(lambda` (C250).
        if !c.is_whitespace() {
            let rest: String = cs[p..].iter().take_while(|&&c| c != '\n').collect();
            let cut: String = rest.chars().take(60).collect();
            let more = if rest.chars().count() > 60 { "…" } else { "" };
            out.push_str(&format!("{line}:{col}  {cut}{more}"));
        }
        out.push('\n');
        if c == '\n' {
            (line, col) = (line + 1, 0);
        } else {
            col += 1;
        }
    }
    std::fs::write(&side, out).ok()?;
    Some((side, n))
}

/// Machine code at `start`, `len` bytes, named `name`, for profilers
/// (C239): with `KAYAK_PERF_MAP` set, a line of `/tmp/perf-PID.map`, the
/// format `perf report` reads (`START LEN NAME`, both in hex), as Fixpt's
/// `FIXPT_PERF_MAP`; else nothing.
pub fn code_noted(start: i64, len: i64, name: &str) {
    static MAP: Mutex<Option<std::fs::File>> = Mutex::new(None);
    if len <= 0 {
        return;
    }
    // With `KAYAK_JITDUMP` set, a jitdump record too, its bytes (C240).
    if std::env::var_os("KAYAK_JITDUMP").is_some() {
        kayak_jit::jitdump::code_load(start as usize, len as usize, name);
    }
    if std::env::var_os("KAYAK_PERF_MAP").is_none() {
        return;
    }
    let mut m = MAP.lock().unwrap_or_else(|p| p.into_inner());
    if m.is_none() {
        let path = format!("/tmp/perf-{}.map", std::process::id());
        *m = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .ok();
    }
    if let Some(f) = m.as_mut() {
        let _ = writeln!(f, "{start:x} {len:x} {name}");
    }
}

/// The next byte, waiting for one; `eof` at the end.
pub fn read_byte() -> Sum {
    let mut i = arrived();
    match i.buf.pop_front() {
        Some(b) => {
            drop(i);
            TAKEN.notify_one();
            BYTES[b as usize].sum()
        }
        None => match i.error.take() {
            Some(e) => sum_of(crate::tag_lit!("failed"), io_error(&e)),
            None => {
                static C: SumOf<()> = SumOf::new(crate::tag_lit!("eof"), crate::Held::UNIT);
                C.sum()
            }
        },
    }
}

/// `(read-bytes bv start end)`: waits for one byte, then takes as many as
/// have come, at most `end - start`, into `bv` from `start`; how many, or
/// `eof`. None asked for, 0 at once.
pub fn read_bytes(bv: BytesAt<'_>, start: Int, end: Int) -> Sum {
    let (s, e) = span(bv, start, end, "read-bytes: a span that does not fit");
    if s == e {
        return counted(0);
    }
    let mut i = arrived();
    if i.buf.is_empty() {
        return match i.error.take() {
            Some(e) => sum_of(crate::tag_lit!("failed"), io_error(&e)),
            None => {
                static C: SumOf<()> = SumOf::new(crate::tag_lit!("eof"), crate::Held::UNIT);
                C.sum()
            }
        };
    }
    let k = i.buf.len().min(e - s);
    for (cell, b) in bv.cells()[s..s + k].iter().zip(i.buf.drain(..k)) {
        cell.set(b);
    }
    drop(i);
    TAKEN.notify_one();
    counted(k)
}

// The terminal (C109), by `stty`, as kayak-fx26's REPL set it before: safe
// Rust has no `tcsetattr`. Raw mode as Fixpt's: no echo, no line editing,
// each key as it comes, ^C, ^Z and ^\ keys, no flow control, a return not
// made a newline; output as before.

/// The modes before raw mode was first asked for, to put back.
static SAVED: Mutex<Option<String>> = Mutex::new(None);

/// `stty` with `args`, on the standard input: what it says; none if the
/// standard input is no terminal.
pub(crate) fn stty(args: &[&str]) -> Option<std::io::Result<String>> {
    use std::io::IsTerminal;
    if !std::io::stdin().is_terminal() {
        return None;
    }
    Some(
        std::process::Command::new("stty")
            .args(args)
            .stdin(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::piped())
            .output()
            .and_then(|o| {
                if o.status.success() {
                    Ok(String::from_utf8_lossy(&o.stdout).trim().to_string())
                } else {
                    Err(std::io::Error::other(
                        String::from_utf8_lossy(&o.stderr).trim().to_string(),
                    ))
                }
            }),
    )
}

pub(crate) fn moded(r: Option<std::io::Result<String>>) -> Sum {
    match r {
        None => {
            static C: SumOf<()> = SumOf::new(crate::tag_lit!("not-a-terminal"), crate::Held::UNIT);
            C.sum()
        }
        Some(Ok(_)) => {
            static C: SumOf<()> = SumOf::new(crate::tag_lit!("done"), crate::Held::UNIT);
            C.sum()
        }
        Some(Err(e)) => sum_of(crate::tag_lit!("failed"), io_error(&e)),
    }
}

/// The terminal in raw mode, its modes before saved the first time.
pub fn terminal_raw() -> Sum {
    moded(terminal_raw_stty())
}

/// What `stty` said, putting the terminal in raw mode.
pub(crate) fn terminal_raw_stty() -> Option<std::io::Result<String>> {
    let mut saved = SAVED.lock().unwrap_or_else(|p| p.into_inner());
    if saved.is_none() {
        match stty(&["-g"]) {
            Some(Ok(modes)) => *saved = Some(modes),
            other => return other,
        }
    }
    stty(&[
        "-echo", "-icanon", "-iexten", "-isig", "-ixon", "-icrnl", "-brkint", "-inpck", "-istrip",
        "cs8", "min", "1", "time", "0",
    ])
}

/// The terminal's modes as before raw mode, if it was asked for.
pub fn terminal_restore() -> Sum {
    moded(terminal_restore_stty())
}

/// What `stty` said, putting the terminal's modes back.
pub(crate) fn terminal_restore_stty() -> Option<std::io::Result<String>> {
    let saved = SAVED.lock().unwrap_or_else(|p| p.into_inner()).take();
    match saved {
        Some(modes) => stty(&[modes.as_str()]),
        None => stty(&["-g"]),
    }
}

/// The terminal's rows and columns.
pub fn terminal_size() -> Sum {
    terminal_size_answer(stty(&["size"]))
}

pub(crate) fn terminal_size_answer(said: Option<std::io::Result<String>>) -> Sum {
    match said {
        None => sum_of(crate::tag_lit!("not-a-terminal"), ()),
        Some(Err(e)) => sum_of(crate::tag_lit!("failed"), io_error(&e)),
        Some(Ok(s)) => {
            let ws: Vec<i64> = s
                .split_whitespace()
                .filter_map(|w| w.parse().ok())
                .collect();
            match ws[..] {
                [rows, columns] => sum_of(
                    crate::tag_lit!("size"),
                    Product_4rows_7columns(i64_to_int(rows), i64_to_int(columns)),
                ),
                _ => sum_of(
                    crate::tag_lit!("failed"),
                    invalid_data("stty size: no rows and columns"),
                ),
            }
        }
    }
}

/// The terminal put back as the run found it, when the run ends, failed or
/// not, as `fixpt` does: for a program's driver to call.
pub fn finish() {
    let _ = terminal_restore();
}

static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

/// Monotonic nanoseconds since the clock was first asked, by any thread:
/// one clock for every process, so that a deadline one makes is another's
/// too (C108).
pub fn clock() -> i64 {
    START.get_or_init(Instant::now).elapsed().as_nanos() as i64
}

pub fn sleep_until(t: i64) {
    let now = clock();
    if t > now {
        std::thread::sleep(std::time::Duration::from_nanos((t - now) as u64));
    }
}

/// The program's arguments as its host gives them, in place of what
/// follows the program's name on the command line (C111): FX-26 reads no
/// environment, so a host that knows more (kayak-fx26, the REPL's history
/// file under `$HOME`) says it as arguments. The first given is kept.
static GIVEN_ARGS: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();

pub fn set_args(args: Vec<String>) {
    let _ = GIVEN_ARGS.set(args);
}

/// The program's arguments: what follows the program's name, or what its
/// host gave (`set_args`).
pub fn args() -> List<Str> {
    let all: Vec<String> = match GIVEN_ARGS.get() {
        Some(given) => given.clone(),
        None => std::env::args().skip(1).collect(),
    };
    all.into_iter()
        .rev()
        .fold(nil(), |acc, a| heap_cons(leak_string(a), acc))
}

pub fn read_file(path: Str) -> Sum {
    read_file_answer(read_regular_file(path.as_str()))
}

/// A regular file's bytes; a directory, or anything else that could wait
/// on another program (a FIFO, a device), refused, as Fixpt's
/// `read_regular_file` refuses it (Fixpt 0d933ff, its TODO §225; C393).
/// Asked before opening, so that a FIFO no one writes is never opened.
pub(crate) fn read_regular_file(path: &str) -> std::io::Result<Vec<u8>> {
    let meta = std::fs::metadata(path)?;
    if meta.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::IsADirectory,
            "is a directory",
        ));
    }
    if !meta.is_file() {
        return Err(std::io::Error::other(
            "not a regular file, which could wait on another program",
        ));
    }
    std::fs::read(path)
}

/// `read-file`'s answer, of what the system gave (C381: in the async mode,
/// a worker of the runner's reads it; the answer is made here, in the
/// program's heap).
pub(crate) fn read_file_answer(r: std::io::Result<Vec<u8>>) -> Sum {
    match r {
        Ok(v) => sum_of(crate::tag_lit!("data"), bytes_of(v)),
        Err(e) => sum_of(crate::tag_lit!("failed"), io_error(&e)),
    }
}

/// The names, sorted by their bytes; a name not UTF-8 an `invalid-data`
/// failure.
pub fn list_dir(path: Str) -> Sum {
    list_dir_answer(list_dir_names(path.as_str()))
}

pub(crate) fn list_dir_names(path: &str) -> std::io::Result<Vec<String>> {
    std::fs::read_dir(path).and_then(|d| {
        d.map(|e| {
            e.and_then(|e| {
                e.file_name().into_string().map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "a name not UTF-8")
                })
            })
        })
        .collect::<std::io::Result<Vec<String>>>()
    })
}

pub(crate) fn list_dir_answer(names: std::io::Result<Vec<String>>) -> Sum {
    match names {
        Ok(mut ns) => {
            ns.sort();
            sum_of(
                crate::tag_lit!("names"),
                ns.into_iter()
                    .rev()
                    .fold(nil(), |acc, n| heap_cons(leak_string(n), acc)),
            )
        }
        Err(e) => sum_of(crate::tag_lit!("failed"), io_error(&e)),
    }
}

/// `file-info`'s kind of file.
pub type FileKind = Sum;

/// `file-info`'s answer: the path itself (`lstat`, a link not followed),
/// its kind, size, modification time in seconds and permission bits, as
/// Fixpt's `file_info` (C293).
pub type FileInfo = Product_4kind_4size_8modified_4mode<FileKind, Int, Int, Int>;

pub fn file_info(path: Str) -> Sum {
    file_info_answer(std::fs::symlink_metadata(path.as_str()))
}

pub(crate) fn file_info_answer(r: std::io::Result<std::fs::Metadata>) -> Sum {
    use std::os::unix::fs::MetadataExt;
    match r {
        Ok(m) => {
            let t = m.file_type();
            let kind: FileKind = if t.is_symlink() {
                sum_of(crate::tag_lit!("symlink"), ())
            } else if t.is_dir() {
                sum_of(crate::tag_lit!("directory"), ())
            } else if t.is_file() {
                sum_of(crate::tag_lit!("file"), ())
            } else {
                sum_of(crate::tag_lit!("other"), ())
            };
            sum_of(
                crate::tag_lit!("info"),
                Product_4kind_4size_8modified_4mode(
                    kind,
                    i64_to_int(m.len() as i64),
                    i64_to_int(m.mtime()),
                    i64_to_int((m.mode() & 0o7777) as i64),
                ),
            )
        }
        Err(e) => sum_of(crate::tag_lit!("failed"), io_error(&e)),
    }
}

/// The target of the symbolic link `path` names, as written; one not UTF-8
/// an `invalid-data` failure (Fixpt's `read_symlink`).
pub fn read_symlink(path: Str) -> Sum {
    read_symlink_answer(std::fs::read_link(path.as_str()))
}

pub(crate) fn read_symlink_answer(r: std::io::Result<std::path::PathBuf>) -> Sum {
    match r {
        Ok(t) => match t.into_os_string().into_string() {
            Ok(t) => sum_of(crate::tag_lit!("target"), leak_string(t)),
            Err(_) => sum_of(
                crate::tag_lit!("failed"),
                invalid_data("a target that is not UTF-8"),
            ),
        },
        Err(e) => sum_of(crate::tag_lit!("failed"), io_error(&e)),
    }
}

/// Replaces the file whole and atomically: written beside it, flushed to
/// the disk, and renamed over it.
pub fn write_file(path: Str, data: BytesAt<'_>) -> Sum {
    written(write_file_bytes(path.as_str(), &to_vec(data)))
}

/// `(write-file-unflushed path data)` (C475; Fixpt's 6f7bd73, kayak's
/// request): replaced whole and atomically, as `write-file` is, but
/// nothing flushed to the disk before the rename: after a crash the file
/// may hold its old contents, its new, nothing, or be missing.
pub fn write_file_unflushed(path: Str, data: BytesAt<'_>) -> Sum {
    written(replace_file_bytes(path.as_str(), &to_vec(data), false))
}

pub(crate) fn write_file_bytes(path: &str, bytes: &[u8]) -> std::io::Result<()> {
    replace_file_bytes(path, bytes, true)
}

/// A file replaced whole: written beside it, flushed to the disk first if
/// `durable`, and renamed over it, so a reader sees the old or the new.
pub(crate) fn replace_file_bytes(path: &str, bytes: &[u8], durable: bool) -> std::io::Result<()> {
    let path = std::path::Path::new(path);
    let mut beside = path.as_os_str().to_owned();
    beside.push(format!(".kayak-{}.tmp", std::process::id()));
    let beside = std::path::PathBuf::from(beside);
    let r = (|| {
        let mut f = std::fs::File::create(&beside)?;
        f.write_all(bytes)?;
        if durable {
            f.sync_all()?;
        }
        std::fs::rename(&beside, path)
    })();
    if r.is_err() {
        let _ = std::fs::remove_file(&beside);
    }
    r
}

/// Programs run beside this one (C295; Heist's F4 and F6, Fixpt's
/// `World::process_start` and the rest): by number, their standard output
/// and error one pipe, read by a thread of their own, so that asking never
/// waits. Without `unsafe`, as kayak-rt is: a process group of its own
/// (Fixpt's is a session, without a terminal), its signal sent by `kill`,
/// and its end found by polling.
struct Child {
    child: std::process::Child,
    /// Its own lock, so that a write that waits for the process to read
    /// holds no other (C381: in the async mode, a worker writes it).
    input: Option<std::sync::Arc<std::sync::Mutex<std::process::ChildStdin>>>,
    output: std::sync::Arc<std::sync::Mutex<ChildOutput>>,
    status: Option<std::process::ExitStatus>,
    seen_end: bool,
}

#[derive(Default)]
struct ChildOutput {
    bytes: Vec<u8>,
    ended: bool,
}

static CHILDREN: std::sync::Mutex<Vec<Child>> = std::sync::Mutex::new(Vec::new());

/// What Fixpt's wake pipe says: news of a child, its output read or ended,
/// or its end found, counted, for `process_wait` to see change.
static CHILD_NEWS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Never called with `CHILDREN` held: a `pselect`'s waiters look again
/// (`process::input_changed`), and the scheduler's lock is taken before
/// `CHILDREN` where a waiter asks whether a child is ready.
fn child_news() {
    CHILD_NEWS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    crate::process::input_changed();
    // In the async mode, the runner's reactor woken, so that tasks waiting
    // for a child look again (C381).
    crate::async_io::wake();
}

/// Program `argv`'s first started with the rest as arguments, in `dir`
/// (this one's, if empty): its number.
pub fn process_start(argv: crate::ListAt<'_, Str>, dir: Str) -> Sum {
    use std::os::unix::process::CommandExt as _;
    let mut words = Vec::new();
    let mut l = argv;
    while let Some(c) = l.pair() {
        words.push(c.car().as_str().to_string());
        l = c.cdr;
    }
    let started = (|| {
        let Some((program, args)) = words.split_first() else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "no program named",
            ));
        };
        let (reader, writer) = std::io::pipe()?;
        let mut command = std::process::Command::new(program);
        command
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(writer.try_clone()?)
            .stderr(writer)
            .process_group(0);
        if !dir.as_str().is_empty() {
            command.current_dir(dir.as_str());
        }
        let mut child = command.spawn()?;
        drop(command);
        let input = child
            .stdin
            .take()
            .map(|i| std::sync::Arc::new(std::sync::Mutex::new(i)));
        let output = std::sync::Arc::new(std::sync::Mutex::new(ChildOutput::default()));
        let out = output.clone();
        std::thread::spawn(move || {
            use std::io::Read as _;
            let mut reader = reader;
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => out.lock().unwrap().bytes.extend_from_slice(&buf[..n]),
                }
                child_news();
            }
            out.lock().unwrap().ended = true;
            child_news();
        });
        let p = {
            let mut children = CHILDREN.lock().unwrap();
            children.push(Child {
                child,
                input,
                output,
                status: None,
                seen_end: false,
            });
            children.len() - 1
        };
        // Its end, said too, as Fixpt's thread waiting with `WNOWAIT` says
        // it: it may end after its output does, or before.
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_millis(2));
                let ended = {
                    let mut children = CHILDREN.lock().unwrap();
                    let c = &mut children[p];
                    if c.status.is_none() {
                        c.status = c.child.try_wait().ok().flatten();
                    }
                    c.status.is_some()
                };
                if ended {
                    child_news();
                    break;
                }
            }
        });
        Ok(p as i64)
    })();
    match started {
        Ok(p) => sum_of(crate::tag_lit!("started"), i64_to_int(p)),
        Err(e) => sum_of(crate::tag_lit!("failed"), io_error(&e)),
    }
}

/// How many bytes at the end of `bytes` begin a UTF-8 character not yet
/// whole (Fixpt's `incomplete_tail`).
fn incomplete_tail(bytes: &[u8]) -> usize {
    for back in 1..=bytes.len().min(3) {
        let b = bytes[bytes.len() - back];
        if b & 0xC0 != 0x80 {
            let need = if b >= 0xF0 {
                4
            } else if b >= 0xE0 {
                3
            } else if b >= 0xC0 {
                2
            } else {
                1
            };
            return if need > back { back } else { 0 };
        }
    }
    0
}

fn with_child<R>(p: Int, none: R, f: impl FnOnce(&mut Child) -> R) -> R {
    let mut children = CHILDREN.lock().unwrap();
    match usize::try_from(int_to_i64(p))
        .ok()
        .and_then(|p| children.get_mut(p))
    {
        Some(c) => f(c),
        None => none,
    }
}

/// What process `p` has written since last asked, as text: a character
/// cut by the read waits for the rest, bytes not UTF-8 U+FFFD.
pub fn process_output(p: Int) -> Str {
    leak_string(with_child(p, String::new(), |c| {
        let mut out = c.output.lock().unwrap();
        let bytes = std::mem::take(&mut out.bytes);
        let keep = if out.ended {
            0
        } else {
            incomplete_tail(&bytes)
        };
        let (text, rest) = bytes.split_at(bytes.len() - keep);
        out.bytes = rest.to_vec();
        String::from_utf8_lossy(text).into_owned()
    }))
}

/// How `c` ended, once it has exited and its output has all been read;
/// else none.
fn ended(c: &mut Child) -> Option<(Option<i32>, Option<i32>)> {
    use std::os::unix::process::ExitStatusExt as _;
    if c.status.is_none() {
        c.status = c.child.try_wait().ok().flatten();
    }
    match c.status {
        Some(s) if c.output.lock().unwrap().ended => Some((s.code(), s.signal())),
        _ => None,
    }
}

/// Whether process `p` runs: until it has exited and its output has all
/// been read, so that a last `process-output` after gives the rest.
pub fn process_status(p: Int) -> Sum {
    with_child(p, sum_of(crate::tag_lit!("unknown"), ()), |c| {
        match ended(c) {
            None => sum_of(crate::tag_lit!("running"), ()),
            Some(how) => {
                c.seen_end = true;
                match how {
                    (Some(code), _) => sum_of(crate::tag_lit!("exited"), i64_to_int(code.into())),
                    (None, Some(sig)) => {
                        sum_of(crate::tag_lit!("signalled"), i64_to_int(sig.into()))
                    }
                    (None, None) => sum_of(crate::tag_lit!("unknown"), ()),
                }
            }
        }
    })
}

/// How process `p` ended, waiting until it has and its reader has read
/// all it wrote (F6).
pub fn process_exit(p: Int) -> Sum {
    loop {
        match process_exited(p) {
            Some(how) => return how,
            None => std::thread::sleep(std::time::Duration::from_millis(1)),
        }
    }
}

/// How process `p` ended, if it has and its output is all read; else none
/// (C381: in the async mode, a task waits for it).
pub(crate) fn process_exited(p: Int) -> Option<Sum> {
    let how = with_child(p, Some(None), |c| {
        ended(c).map(|how| {
            c.seen_end = true;
            Some(how)
        })
    })?;
    Some(match how {
        None => sum_of(crate::tag_lit!("unknown"), ()),
        Some((Some(code), _)) => sum_of(crate::tag_lit!("exited"), i64_to_int(code.into())),
        Some((None, Some(sig))) => sum_of(crate::tag_lit!("signalled"), i64_to_int(sig.into())),
        Some((None, None)) => sum_of(crate::tag_lit!("unknown"), ()),
    })
}

/// The first process with news (F6): output to take, or an end the
/// program has not seen; -1 if none has.
pub fn process_news() -> Int {
    let mut children = CHILDREN.lock().unwrap();
    for (p, c) in children.iter_mut().enumerate() {
        let ready = {
            let out = c.output.lock().unwrap();
            let n = out.bytes.len();
            n > 0 && (out.ended || n > incomplete_tail(&out.bytes))
        };
        if ready || (!c.seen_end && ended(c).is_some()) {
            return i64_to_int(p as i64);
        }
    }
    i64_to_int(-1)
}

/// Whether process `p` has output to take, text `process-output` gives
/// (not only part of a character); or is no process (the scheduler's, F6).
pub fn process_output_ready(p: i64) -> bool {
    with_child(i64_to_int(p), true, |c| {
        let out = c.output.lock().unwrap();
        let n = out.bytes.len();
        n > 0 && (out.ended || n > incomplete_tail(&out.bytes))
    })
}

/// Whether process `p` has ended, its output all read, as `process-status`
/// says, which then has seen it; or is no process.
pub fn process_ended(p: i64) -> bool {
    crate::sum_name(process_status(i64_to_int(p))) != "running"
}

/// Waits until a child has news (its output read, or ended, or its end
/// found), or the standard input is ready if `stdin`, or the clock reaches
/// `until` (-1: no limit): whether it has not run out (Fixpt's
/// `wait_processes`, its wake pipe a count polled).
pub fn process_wait(until: i64, stdin: bool) -> bool {
    use std::sync::atomic::Ordering;
    let seen = CHILD_NEWS.load(Ordering::SeqCst);
    loop {
        if stdin && stdin_ready_p() {
            return true;
        }
        if CHILD_NEWS.load(Ordering::SeqCst) != seen {
            return true;
        }
        let now = clock();
        if until >= 0 && now >= until {
            return false;
        }
        let step = if until >= 0 {
            (until - now).min(1_000_000)
        } else {
            1_000_000
        };
        std::thread::sleep(Duration::from_nanos(step as u64));
    }
}

/// `text` written to process `p`'s input.
pub fn process_write(p: Int, text: Str) -> Sum {
    written(process_write_bytes(
        process_input(p),
        text.as_str().as_bytes(),
    ))
}

/// Process `p`'s input, if it is open.
pub(crate) fn process_input(
    p: Int,
) -> Option<std::sync::Arc<std::sync::Mutex<std::process::ChildStdin>>> {
    with_child(p, None, |c| c.input.clone())
}

/// `bytes` written to a process's `input`, waiting for it to read them.
pub(crate) fn process_write_bytes(
    input: Option<std::sync::Arc<std::sync::Mutex<std::process::ChildStdin>>>,
    bytes: &[u8],
) -> std::io::Result<()> {
    use std::io::Write as _;
    match input {
        Some(input) => {
            let mut input = input.lock().unwrap_or_else(|p| p.into_inner());
            input.write_all(bytes).and_then(|()| input.flush())
        }
        None => Err(std::io::Error::new(
            std::io::ErrorKind::BrokenPipe,
            "the process's input is closed",
        )),
    }
}

/// Process `p`'s input closed: it reads its end.
pub fn process_close_input(p: Int) {
    with_child(p, (), |c| c.input = None)
}

/// Signal `sig` sent to process `p`'s group, unless it has ended.
pub fn process_signal(p: Int, sig: Int) {
    with_child(p, (), |c| {
        if c.status.is_none() {
            let _ = std::process::Command::new("kill")
                .arg(format!("-{}", int_to_i64(sig)))
                .arg("--")
                .arg(format!("-{}", c.child.id()))
                .status();
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{int_lit, str_lit};

    #[test]
    fn bytes_made_read_and_copied() {
        let b = make_bytevector(int_lit(4), 7);
        bytevector_u8_set_x(b, int_lit(1), 250);
        let hi = utf8_to_bytes(string_to_utf8(str_lit("héllo")));
        bytevector_copy_x(b, int_lit(2), hi, int_lit(0), int_lit(2));
        assert_eq!(to_vec(b), vec![7, 250, 104, 195]);
        assert!(bytevector_eq_p(
            b,
            bytevector_copy(b, int_lit(0), int_lit(4))
        ));
        let r = bytes_to_utf8(bytevector_copy(hi, int_lit(0), int_lit(2)));
        assert_eq!(crate::sum_name(r), "bad");
        assert_eq!(int_to_i64(crate::sum_payload::<Int>(r)), 1);
        let mut s = String::new();
        b.show(&mut s);
        assert_eq!(s, "#u8(7 250 104 195)");
    }

    #[test]
    fn a_file_written_listed_and_read() {
        let dir = std::env::temp_dir().join(format!("kayak-world-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = leak_string(dir.join("a.txt").to_string_lossy().into_owned());
        use crate::{sum_name, sum_payload};
        assert_eq!(
            sum_name(write_file(p, utf8_to_bytes(string_to_utf8(str_lit("ay"))))),
            "done"
        );
        let r = read_file(p);
        assert_eq!(sum_name(r), "data");
        assert_eq!(to_vec(sum_payload::<Bytes>(r)), b"ay");
        let r = list_dir(leak_string(dir.to_string_lossy().into_owned()));
        assert_eq!(sum_name(r), "names");
        assert_eq!(
            sum_payload::<List<Str>>(r).pair().map(|c| c.car().as_str()),
            Some("a.txt")
        );
        let none = leak_string(dir.join("none").to_string_lossy().into_owned());
        let r = read_file(none);
        assert_eq!(sum_name(r), "failed");
        assert_eq!(sum_name(sum_payload::<IoError>(r).0), "not-found");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// `write-file-unflushed` (C475), as Fixpt's `a_file_replaced_unflushed`
    /// tests it: written long then short, it reads back the short alone,
    /// nothing left beside it; one in a missing directory, `not-found`.
    #[test]
    fn a_file_replaced_unflushed() {
        let dir = std::env::temp_dir().join(format!("kayak-unflushed-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = leak_string(dir.join("v1").to_string_lossy().into_owned());
        use crate::{sum_name, sum_payload};
        assert_eq!(
            sum_name(write_file_unflushed(
                p,
                utf8_to_bytes(string_to_utf8(str_lit("a longer text")))
            )),
            "done"
        );
        assert_eq!(
            sum_name(write_file_unflushed(
                p,
                utf8_to_bytes(string_to_utf8(str_lit("short")))
            )),
            "done"
        );
        let r = read_file(p);
        assert_eq!(to_vec(sum_payload::<Bytes>(r)), b"short");
        let names: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, vec!["v1".to_string()]);
        let missing = leak_string(dir.join("none").join("v2").to_string_lossy().into_owned());
        let r = write_file_unflushed(missing, utf8_to_bytes(string_to_utf8(str_lit("x"))));
        assert_eq!(sum_name(r), "failed");
        assert_eq!(sum_name(sum_payload::<IoError>(r).0), "not-found");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
