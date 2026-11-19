//! FX-26's objects as Fixpt lays them (C482; Fixpt's `fixpt-heap`'s
//! `layout.rs`): a header word (its tag, kind, field count and suffix's
//! bytes, `kayak_jit::layout::header`), `F` fields, a trailer word (its
//! distance back to the header), then `B` bytes of suffix. One word, a
//! pointer to the header, reaches it; the object says its own size, so a
//! collector walking the heap, or reaching it from an interior word by its
//! trailer, needs nothing else.
//!
//! Strings first: kind 1, no fields, the text the suffix. The trailer's
//! payload is the runtime's (`docs/object-model.md` reserves it): Fixpt's
//! keeps the distance alone; kayak keeps there too whether the text is
//! ASCII (bit 62), which Fixpt finds again when it needs it.
//!
//! The one place, with `sum.rs` and the census, where kayak-rt has
//! `unsafe`: an object of a size known at run time, its bytes after its
//! header, is none safe Rust allocates. Safe here: an object is made once,
//! whole, and never written nor freed after (kayak-rt has no collector
//! yet); its text is checked UTF-8 when made.

#![allow(unsafe_code)]

use kayak_jit::layout;
use std::cell::Cell;
use std::ptr::NonNull;

/// The trailer's bit: the text is ASCII.
const ASCII: u64 = 1 << 62;

/// The words a string of `n` bytes takes: header, trailer, its bytes.
pub const fn string_words(n: usize) -> usize {
    2 + n.div_ceil(8)
}

/// `string`: one word, to its object.
#[derive(Clone, Copy)]
pub struct Str(NonNull<u64>);

// SAFETY: a string's object is never written after it is made, nor freed.
unsafe impl Send for Str {}
unsafe impl Sync for Str {}

impl Str {
    #[inline]
    fn header(self) -> u64 {
        // SAFETY: a `Str` is made only by this module, of an object whose
        // header is its first word.
        unsafe { *self.0.as_ptr() }
    }

    #[inline]
    fn trailer(self) -> u64 {
        // SAFETY: as `header`; a string's trailer is its second word.
        unsafe { *self.0.as_ptr().add(1) }
    }

    /// Its text's bytes.
    #[inline]
    pub fn len(self) -> usize {
        (self.header() >> 32) as usize
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn is_ascii(self) -> bool {
        self.trailer() & ASCII != 0
    }

    /// Its text.
    #[inline]
    pub fn as_str(self) -> &'static str {
        // SAFETY: its bytes follow the trailer, `len` of them, within the
        // words it was made of (whose pointer it keeps, so its provenance
        // is theirs), checked UTF-8 when made, never written nor freed.
        unsafe {
            let bytes = std::slice::from_raw_parts(self.0.as_ptr().add(2).cast::<u8>(), self.len());
            std::str::from_utf8_unchecked(bytes)
        }
    }

    /// Its object's first word, for a census: where it is.
    #[inline]
    pub fn addr(self) -> usize {
        self.0.as_ptr() as usize
    }
}

impl PartialEq for Str {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 || self.as_str() == other.as_str()
    }
}

impl Eq for Str {}

/// A string's header and trailer, of `n` bytes.
const fn string_head(n: usize, ascii: bool) -> (u64, u64) {
    (
        layout::header(layout::KIND_STRING, 0, n, true, true),
        layout::trailer(1) | if ascii { ASCII } else { 0 },
    )
}

/// A string's words, made at compile time (a literal): `W` is
/// `string_words` of its length.
pub struct StrLit<const W: usize> {
    words: [u64; W],
}

impl<const W: usize> StrLit<W> {
    pub const fn new(text: &str) -> Self {
        let b = text.as_bytes();
        let (h, t) = string_head(b.len(), text.is_ascii());
        let mut words = [0u64; W];
        words[0] = h;
        words[1] = t;
        // The bytes in each word as memory has them, as a string made at
        // run time writes them.
        let mut w = 0;
        while w * 8 < b.len() {
            let mut chunk = [0u8; 8];
            let mut i = 0;
            while i < 8 && w * 8 + i < b.len() {
                chunk[i] = b[w * 8 + i];
                i += 1;
            }
            words[2 + w] = u64::from_ne_bytes(chunk);
            w += 1;
        }
        StrLit { words }
    }

    /// The string, its object this literal's words.
    #[inline]
    pub fn get(&'static self) -> Str {
        // `as_ptr`: a pointer whose provenance is all the words.
        Str(NonNull::from(&self.words).cast())
    }
}

/// A string literal: its object made at compile time, a `static`.
#[macro_export]
macro_rules! str_lit {
    ($s:expr) => {{
        const S: &str = $s;
        static O: $crate::StrLit<{ $crate::string_words(S.len()) }> = $crate::StrLit::new(S);
        O.get()
    }};
}

/// Words to make strings in, a chunk at a time (C364's bumping): never
/// freed, as nothing of the heap is yet.
const CHUNK_WORDS: usize = 1 << 17;

thread_local! {
    static WORDS: Cell<Option<&'static mut [u64]>> = const { Cell::new(None) };
}

/// The words of a new object of `w` words, zeroed.
fn words(w: usize) -> &'static mut [u64] {
    if w >= CHUNK_WORDS / 16 {
        // A long one its own, leaving the chunk to the short ones.
        return Box::leak(vec![0u64; w].into_boxed_slice());
    }
    let free = WORDS.take().filter(|b| b.len() >= w);
    let free = free.unwrap_or_else(|| Box::leak(vec![0u64; CHUNK_WORDS].into_boxed_slice()));
    let (mine, rest) = free.split_at_mut(w);
    WORDS.set(Some(rest));
    mine
}

/// A string of `n` bytes, which `fill` writes; `None` if they are no UTF-8.
pub(crate) fn string_made(n: usize, fill: impl FnOnce(&mut [u8])) -> Option<Str> {
    let ws = words(string_words(n));
    let (head, body) = ws.split_at_mut(2);
    // SAFETY: the words after the header and trailer, `n` bytes of them
    // (`string_words` made room), zeroed, borrowed only here.
    let bytes: &mut [u8] =
        unsafe { std::slice::from_raw_parts_mut(body.as_mut_ptr().cast::<u8>(), n) };
    fill(bytes);
    if std::str::from_utf8(bytes).is_err() {
        return None;
    }
    let (h, t) = string_head(n, bytes.is_ascii());
    head[0] = h;
    head[1] = t;
    Some(Str(NonNull::from(ws).cast()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_literal_and_a_made_string_alike() {
        let a = crate::str_lit!("héllo, world");
        let b = string_made(13, |bs| bs.copy_from_slice("héllo, world".as_bytes())).unwrap();
        assert_eq!(a.as_str(), "héllo, world");
        assert!(a == b);
        assert!(!a.is_ascii());
        assert_eq!(a.len(), 13);
        assert_eq!(std::mem::size_of::<Str>(), 8);
        assert_eq!(a.header() & layout::TAG_MASK, layout::TAG_HEADER);
        assert!(crate::str_lit!("").is_empty());
        assert!(crate::str_lit!("abc").is_ascii());
    }

    #[test]
    fn not_utf8_refused() {
        assert!(string_made(1, |bs| bs[0] = 0xff).is_none());
    }
}

/// `(bytevectorof R)` (C482): one word, to its object (kind 4, no fields,
/// its bytes the suffix, which may be written: the object's words are
/// cells). At a `letrena`'s region, `BytesAt<'r>`, its object in the
/// region's arena, freed with it (C267); in the heap, `Bytes`, kept.
#[derive(Clone, Copy)]
pub struct BytesAt<'r>(NonNull<Cell<u64>>, std::marker::PhantomData<&'r [Cell<u8>]>);
pub type Bytes = BytesAt<'static>;

/// A bytevector's object, as a region's arena holds it.
pub type BytesCells = Box<[Cell<u64>]>;

impl<'r> BytesAt<'r> {
    /// Its length in bytes.
    #[inline]
    pub fn len(self) -> usize {
        // SAFETY: a `BytesAt` is made only by this module, of an object
        // whose header is its first word, alive for `'r`.
        (unsafe { (*self.0.as_ptr()).get() } >> 32) as usize
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    /// Its bytes, each a cell.
    #[inline]
    pub fn cells(self) -> &'r [Cell<u8>] {
        // SAFETY: its bytes follow the header and trailer, `len` of them,
        // within its object's words (whose pointer it keeps), alive for
        // `'r`; a `Cell<u64>`'s bytes are cells too, each `UnsafeCell`.
        unsafe { std::slice::from_raw_parts(self.0.as_ptr().add(2).cast::<Cell<u8>>(), self.len()) }
    }
}

/// A bytevector's object of `bytes`, to be kept in the heap or an arena.
pub(crate) fn bytes_object(bytes: &[u8]) -> BytesCells {
    let n = bytes.len();
    let ws: BytesCells = (0..string_words(n)).map(|_| Cell::new(0)).collect();
    ws[0].set(layout::header(4, 0, n, true, false));
    ws[1].set(layout::trailer(1));
    for (i, chunk) in bytes.chunks(8).enumerate() {
        let mut w = [0u8; 8];
        w[..chunk.len()].copy_from_slice(chunk);
        ws[2 + i].set(u64::from_ne_bytes(w));
    }
    ws
}

/// The bytevector whose object is `ws` (`bytes_object`'s), kept for `'r`.
pub(crate) fn bytes_at(ws: &[Cell<u64>]) -> BytesAt<'_> {
    BytesAt(NonNull::from(ws).cast(), std::marker::PhantomData)
}

#[cfg(test)]
mod bytes_tests {
    use super::*;

    #[test]
    fn a_bytevector_written_and_read() {
        let ws = bytes_object(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let b = bytes_at(&ws);
        assert_eq!(b.len(), 9);
        b.cells()[8].set(42);
        assert_eq!(
            b.cells().iter().map(Cell::get).collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5, 6, 7, 8, 42]
        );
        assert_eq!(std::mem::size_of::<Bytes>(), 8);
    }
}
