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
    Int, List, Show, Str, Void, fail, heap_alloc, heap_cons, i64_to_int, int_to_i64, leak_string,
    nil,
};
use std::cell::{Cell, OnceCell};
use std::io::{BufRead, Write};
use std::time::Instant;

// The standard shapes, as `put-shape-decls` writes a program's.

#[allow(non_camel_case_types)]
pub enum Sum_4text_3bad<A0, A1> {
    Text(A0),
    Bad(A1),
}

#[allow(non_camel_case_types)]
pub enum Sum_4done_6failed<A0, A1> {
    Done(A0),
    Failed(A1),
}

#[allow(non_camel_case_types)]
pub enum Sum_4line_3eof_6failed<A0, A1, A2> {
    Line(A0),
    Eof(A1),
    Failed(A2),
}

#[allow(non_camel_case_types)]
pub enum Sum_4data_6failed<A0, A1> {
    Data(A0),
    Failed(A1),
}

#[allow(non_camel_case_types)]
pub enum Sum_5names_6failed<A0, A1> {
    Names(A0),
    Failed(A1),
}

#[allow(non_camel_case_types)]
pub enum Sum_9not_found_17permission_denied_14already_exists_15not_a_directory_14is_a_directory_11broken_pipe_12invalid_data_5other<
    A0,
    A1,
    A2,
    A3,
    A4,
    A5,
    A6,
    A7,
> {
    NotFound(A0),
    PermissionDenied(A1),
    AlreadyExists(A2),
    NotADirectory(A3),
    IsADirectory(A4),
    BrokenPipe(A5),
    InvalidData(A6),
    Other(A7),
}

#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub struct Product_4kind_6detail<A0, A1>(pub A0, pub A1);

macro_rules! show_sum {
    ($name:ident < $($a:ident),* > { $($v:ident => $l:literal),* }) => {
        impl<$($a),*> Show for $name<$($a),*> {
            fn show(&self, out: &mut String) {
                match self {
                    $($name::$v(..) => out.push_str(concat!("#<sum ", $l, ">")),)*
                }
            }
        }
    };
}

show_sum!(Sum_4text_3bad<A0, A1> { Text => "text", Bad => "bad" });
show_sum!(Sum_4done_6failed<A0, A1> { Done => "done", Failed => "failed" });
show_sum!(Sum_4line_3eof_6failed<A0, A1, A2> { Line => "line", Eof => "eof", Failed => "failed" });
show_sum!(Sum_4data_6failed<A0, A1> { Data => "data", Failed => "failed" });
show_sum!(Sum_5names_6failed<A0, A1> { Names => "names", Failed => "failed" });
show_sum!(Sum_9not_found_17permission_denied_14already_exists_15not_a_directory_14is_a_directory_11broken_pipe_12invalid_data_5other<A0, A1, A2, A3, A4, A5, A6, A7> {
    NotFound => "not-found", PermissionDenied => "permission-denied", AlreadyExists => "already-exists",
    NotADirectory => "not-a-directory", IsADirectory => "is-a-directory", BrokenPipe => "broken-pipe",
    InvalidData => "invalid-data", Other => "other"
});

impl<A0, A1> Show for Product_4kind_6detail<A0, A1> {
    fn show(&self, out: &mut String) {
        out.push_str("#<product of 2>");
    }
}

/// `io-error`'s kind.
pub type IoKind = &'static Sum_9not_found_17permission_denied_14already_exists_15not_a_directory_14is_a_directory_11broken_pipe_12invalid_data_5other<
    (),
    (),
    (),
    (),
    (),
    (),
    (),
    (),
>;

/// `io-error`: its kind, for a `tagcase`, and the system's words.
pub type IoError = Product_4kind_6detail<IoKind, Str>;

fn io_error(e: &std::io::Error) -> IoError {
    use std::io::ErrorKind as K;
    type S = Sum_9not_found_17permission_denied_14already_exists_15not_a_directory_14is_a_directory_11broken_pipe_12invalid_data_5other<
        (),
        (),
        (),
        (),
        (),
        (),
        (),
        (),
    >;
    let kind: IoKind = heap_alloc(match e.kind() {
        K::NotFound => S::NotFound(()),
        K::PermissionDenied => S::PermissionDenied(()),
        K::AlreadyExists => S::AlreadyExists(()),
        K::NotADirectory => S::NotADirectory(()),
        K::IsADirectory => S::IsADirectory(()),
        K::BrokenPipe => S::BrokenPipe(()),
        K::InvalidData => S::InvalidData(()),
        _ => S::Other(()),
    });
    Product_4kind_6detail(kind, leak_string(e.to_string()))
}

fn invalid_data(what: &str) -> IoError {
    io_error(&std::io::Error::new(std::io::ErrorKind::InvalidData, what))
}

// Bytevectors.

/// `(bytevectorof R)`: bytes in a row, each a cell.
#[derive(Clone, Copy)]
pub struct Bytes(pub &'static [Cell<u8>]);

/// As Fixpt writes one, R7RS's `#u8(…)`.
impl Show for Bytes {
    fn show(&self, out: &mut String) {
        out.push_str("#u8(");
        for (i, c) in self.0.iter().enumerate() {
            if i > 0 {
                out.push(' ');
            }
            out.push_str(&c.get().to_string());
        }
        out.push(')');
    }
}

fn bytes_of(v: Vec<u8>) -> Bytes {
    Bytes(Box::leak(
        v.into_iter()
            .map(Cell::new)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    ))
}

fn index(n: Int, what: &str) -> usize {
    match usize::try_from(int_to_i64(n)) {
        Ok(i) => i,
        Err(_) => fail(what),
    }
}

fn span(b: Bytes, start: Int, end: Int, what: &str) -> (usize, usize) {
    let (s, e) = (index(start, what), index(end, what));
    if s > e || e > b.0.len() {
        fail(what)
    }
    (s, e)
}

fn to_vec(b: Bytes) -> Vec<u8> {
    b.0.iter().map(Cell::get).collect()
}

pub fn make_bytevector(n: Int, v: u8) -> Bytes {
    bytes_of(vec![v; index(n, "make-bytevector: a negative length")])
}

pub fn bytevector_length(b: Bytes) -> Int {
    i64_to_int(b.0.len() as i64)
}

pub fn bytevector_u8_ref(b: Bytes, i: Int) -> u8 {
    match b.0.get(index(i, "bytevector-u8-ref: index out of range")) {
        Some(c) => c.get(),
        None => fail("bytevector-u8-ref: index out of range"),
    }
}

pub fn bytevector_u8_set_x(b: Bytes, i: Int, v: u8) {
    match b.0.get(index(i, "bytevector-u8-set!: index out of range")) {
        Some(c) => c.set(v),
        None => fail("bytevector-u8-set!: index out of range"),
    }
}

pub fn bytevector_copy(b: Bytes, start: Int, end: Int) -> Bytes {
    let (s, e) = span(b, start, end, "bytevector-copy: a span that does not fit");
    bytes_of(b.0[s..e].iter().map(Cell::get).collect())
}

/// R7RS's: `to`, `at`, `from`, `start`, `end`; overlapping spans copied
/// as if through a buffer.
pub fn bytevector_copy_x(to: Bytes, at: Int, from: Bytes, start: Int, end: Int) {
    let (s, e) = span(
        from,
        start,
        end,
        "bytevector-copy!: a span that does not fit",
    );
    let at = index(at, "bytevector-copy!: a span that does not fit");
    if at + (e - s) > to.0.len() {
        fail("bytevector-copy!: a span that does not fit")
    }
    let moved: Vec<u8> = from.0[s..e].iter().map(Cell::get).collect();
    for (k, v) in moved.into_iter().enumerate() {
        to.0[at + k].set(v);
    }
}

pub fn bytevector_eq_p(a: Bytes, b: Bytes) -> bool {
    a.0.len() == b.0.len() && a.0.iter().zip(b.0).all(|(x, y)| x.get() == y.get())
}

pub fn string_to_utf8(s: Str) -> Bytes {
    bytes_of(s.as_str().as_bytes().to_vec())
}

pub fn utf8_to_string(b: Bytes) -> &'static Sum_4text_3bad<Str, Int> {
    match String::from_utf8(to_vec(b)) {
        Ok(s) => heap_alloc(Sum_4text_3bad::Text(leak_string(s))),
        Err(e) => heap_alloc(Sum_4text_3bad::Bad(i64_to_int(
            e.utf8_error().valid_up_to() as i64,
        ))),
    }
}

// The world. Each write flushed, as Fixpt's are, so that the two streams
// interleave as written.

fn written(r: std::io::Result<()>) -> &'static Sum_4done_6failed<(), IoError> {
    heap_alloc(match r {
        Ok(()) => Sum_4done_6failed::Done(()),
        Err(e) => Sum_4done_6failed::Failed(io_error(&e)),
    })
}

pub fn write_out(s: Str) -> &'static Sum_4done_6failed<(), IoError> {
    let mut out = std::io::stdout().lock();
    written(
        out.write_all(s.as_str().as_bytes())
            .and_then(|()| out.flush()),
    )
}

pub fn write_err(s: Str) -> &'static Sum_4done_6failed<(), IoError> {
    let mut err = std::io::stderr().lock();
    written(
        err.write_all(s.as_str().as_bytes())
            .and_then(|()| err.flush()),
    )
}

/// Ends the run at once, with `n`'s low byte.
pub fn exit(n: Int) -> Void {
    let _ = std::io::stdout().flush();
    std::process::exit((int_to_i64(n) & 0xff) as i32)
}

/// The next line, its ending (`\n` or `\r\n`) left off; a line not UTF-8
/// an `invalid-data` failure, reading going on after it.
pub fn read_line() -> &'static Sum_4line_3eof_6failed<Str, (), IoError> {
    let mut buf = Vec::new();
    heap_alloc(match std::io::stdin().lock().read_until(b'\n', &mut buf) {
        Ok(0) => Sum_4line_3eof_6failed::Eof(()),
        Ok(_) => {
            if buf.last() == Some(&b'\n') {
                buf.pop();
                if buf.last() == Some(&b'\r') {
                    buf.pop();
                }
            }
            match String::from_utf8(buf) {
                Ok(s) => Sum_4line_3eof_6failed::Line(leak_string(s)),
                Err(_) => Sum_4line_3eof_6failed::Failed(invalid_data("a line not UTF-8")),
            }
        }
        Err(e) => Sum_4line_3eof_6failed::Failed(io_error(&e)),
    })
}

thread_local!(static START: OnceCell<Instant> = const { OnceCell::new() });

/// Monotonic nanoseconds since the clock was first asked.
pub fn clock() -> i64 {
    START.with(|s| s.get_or_init(Instant::now).elapsed().as_nanos() as i64)
}

pub fn sleep_until(t: i64) {
    let now = clock();
    if t > now {
        std::thread::sleep(std::time::Duration::from_nanos((t - now) as u64));
    }
}

/// The program's arguments: what follows the program's name.
pub fn args() -> List<Str> {
    let all: Vec<String> = std::env::args().skip(1).collect();
    all.into_iter()
        .rev()
        .fold(nil(), |acc, a| heap_cons(leak_string(a), acc))
}

pub fn read_file(path: Str) -> &'static Sum_4data_6failed<Bytes, IoError> {
    heap_alloc(match std::fs::read(path.as_str()) {
        Ok(v) => Sum_4data_6failed::Data(bytes_of(v)),
        Err(e) => Sum_4data_6failed::Failed(io_error(&e)),
    })
}

/// The names, sorted by their bytes; a name not UTF-8 an `invalid-data`
/// failure.
pub fn list_dir(path: Str) -> &'static Sum_5names_6failed<List<Str>, IoError> {
    let names = std::fs::read_dir(path.as_str()).and_then(|d| {
        d.map(|e| {
            e.and_then(|e| {
                e.file_name().into_string().map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, "a name not UTF-8")
                })
            })
        })
        .collect::<std::io::Result<Vec<String>>>()
    });
    heap_alloc(match names {
        Ok(mut ns) => {
            ns.sort();
            Sum_5names_6failed::Names(
                ns.into_iter()
                    .rev()
                    .fold(nil(), |acc, n| heap_cons(leak_string(n), acc)),
            )
        }
        Err(e) => Sum_5names_6failed::Failed(io_error(&e)),
    })
}

/// Replaces the file whole and atomically: written beside it, flushed to
/// the disk, and renamed over it.
pub fn write_file(path: Str, data: Bytes) -> &'static Sum_4done_6failed<(), IoError> {
    let path = std::path::Path::new(path.as_str());
    let mut beside = path.as_os_str().to_owned();
    beside.push(format!(".kayak-{}.tmp", std::process::id()));
    let beside = std::path::PathBuf::from(beside);
    let r = (|| {
        let mut f = std::fs::File::create(&beside)?;
        f.write_all(&to_vec(data))?;
        f.sync_all()?;
        std::fs::rename(&beside, path)
    })();
    if r.is_err() {
        let _ = std::fs::remove_file(&beside);
    }
    written(r)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{int_lit, str_lit};

    #[test]
    fn bytes_made_read_and_copied() {
        let b = make_bytevector(int_lit(4), 7);
        bytevector_u8_set_x(b, int_lit(1), 250);
        let hi = string_to_utf8(str_lit("héllo"));
        bytevector_copy_x(b, int_lit(2), hi, int_lit(0), int_lit(2));
        assert_eq!(to_vec(b), vec![7, 250, 104, 195]);
        assert!(bytevector_eq_p(
            b,
            bytevector_copy(b, int_lit(0), int_lit(4))
        ));
        match utf8_to_string(bytevector_copy(hi, int_lit(0), int_lit(2))) {
            Sum_4text_3bad::Bad(k) => assert_eq!(int_to_i64(*k), 1),
            Sum_4text_3bad::Text(_) => panic!("valid"),
        }
        let mut s = String::new();
        b.show(&mut s);
        assert_eq!(s, "#u8(7 250 104 195)");
    }

    #[test]
    fn a_file_written_listed_and_read() {
        let dir = std::env::temp_dir().join(format!("kayak-world-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = leak_string(dir.join("a.txt").to_string_lossy().into_owned());
        assert!(matches!(
            write_file(p, string_to_utf8(str_lit("ay"))),
            Sum_4done_6failed::Done(())
        ));
        match read_file(p) {
            Sum_4data_6failed::Data(b) => assert_eq!(to_vec(*b), b"ay"),
            Sum_4data_6failed::Failed(_) => panic!("unread"),
        }
        match list_dir(leak_string(dir.to_string_lossy().into_owned())) {
            Sum_5names_6failed::Names(ns) => assert_eq!(ns.map(|c| c.car.as_str()), Some("a.txt")),
            Sum_5names_6failed::Failed(_) => panic!("unlisted"),
        }
        let none = leak_string(dir.join("none").to_string_lossy().into_owned());
        match read_file(none) {
            Sum_4data_6failed::Failed(Product_4kind_6detail(k, _)) => {
                assert!(matches!(k, Sum_9not_found_17permission_denied_14already_exists_15not_a_directory_14is_a_directory_11broken_pipe_12invalid_data_5other::NotFound(())))
            }
            Sum_4data_6failed::Data(_) => panic!("read"),
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
