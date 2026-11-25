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
//! whole, and its text never written after; it is moved or freed only by
//! a collection (`obj::collect`, C511), which moves none that an ambiguous
//! word (a `Str` on the stack) reaches, and frees none reached; its text is
//! checked UTF-8 when made.

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

pub(crate) mod collect;
mod frames;
pub(crate) mod publish;
pub(crate) mod roots;
pub(crate) mod space;

/// The words of a new object of `w` words, zeroed, in the heap's space.
#[inline]
fn words(w: usize) -> &'static mut [u64] {
    space::words_slice(w)
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

/// `utf8` (C530; Fixpt's, its 279c16cf): an object of its own, as Fixpt's
/// `make_utf8` makes one: kind 49 (50, `utf8-ascii`, when its text is all
/// ASCII), no fields and no trailer, its suffix the text's bytes, the
/// header's count of them exact, frozen. One word, to its object's header.
/// Before C530 it was the string's object (kind 1), which a union could
/// not tell from a string.
#[derive(Clone, Copy)]
pub struct Utf8(NonNull<u64>);

// SAFETY: a `utf8`'s object is never written after it is made.
unsafe impl Send for Utf8 {}
unsafe impl Sync for Utf8 {}

const KIND_UTF8: u8 = kayak_layout::kind("utf8");
const KIND_UTF8_ASCII: u8 = kayak_layout::kind("utf8-ascii");

/// The words a `utf8` of `n` bytes takes: its header, its bytes.
pub const fn utf8_words(n: usize) -> usize {
    1 + n.div_ceil(8)
}

/// A `utf8`'s header, of `n` bytes.
const fn utf8_head(n: usize, ascii: bool) -> u64 {
    layout::header(
        if ascii { KIND_UTF8_ASCII } else { KIND_UTF8 },
        0,
        n,
        true,
        true,
    )
}

impl Utf8 {
    #[inline]
    fn header(self) -> u64 {
        // SAFETY: a `Utf8` is made only by this module, of an object whose
        // header is its first word.
        unsafe { *self.0.as_ptr() }
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

    /// Whether its text is all ASCII: its kind says.
    #[inline]
    pub fn is_ascii(self) -> bool {
        ((self.header() >> 3) & 0xff) as u8 == KIND_UTF8_ASCII
    }

    /// Its text.
    #[inline]
    pub fn as_str(self) -> &'static str {
        // SAFETY: its bytes follow the header, `len` of them, within the
        // words it was made of (whose pointer it keeps), UTF-8 when made
        // (from a `&str`), never written.
        unsafe {
            let bytes = std::slice::from_raw_parts(self.0.as_ptr().add(1).cast::<u8>(), self.len());
            std::str::from_utf8_unchecked(bytes)
        }
    }

    /// A `utf8` of `text`, made in the heap's space.
    pub fn of_str(text: &str) -> Utf8 {
        let n = text.len();
        let ws = words(utf8_words(n));
        let (head, body) = ws.split_at_mut(1);
        // SAFETY: the words after the header, `n` bytes of them
        // (`utf8_words` made room), zeroed, borrowed only here.
        let bytes: &mut [u8] =
            unsafe { std::slice::from_raw_parts_mut(body.as_mut_ptr().cast::<u8>(), n) };
        bytes.copy_from_slice(text.as_bytes());
        head[0] = utf8_head(n, text.is_ascii());
        Utf8(NonNull::from(ws).cast())
    }

    /// Its word, as a heap object holds it: its suffix's address, past its
    /// header, tagged `100`, as Fixpt's.
    #[inline(always)]
    pub fn word(self) -> HeapWord {
        HeapWord(
            self.0
                .as_ptr()
                .cast_const()
                .wrapping_add(1)
                .map_addr(|a| a | TAG_OBJECT),
        )
    }

    /// The `utf8` whose word is `w` (`word`'s).
    #[inline(always)]
    pub fn of_word(w: HeapWord) -> Utf8 {
        let p = w.0.map_addr(|a| a & !TAG_MASK).wrapping_sub(1);
        match NonNull::new(p.cast_mut()) {
            Some(p) => Utf8(p),
            None => crate::fail("a heap's word read as a utf8, which it is not"),
        }
    }
}

impl std::fmt::Debug for Utf8 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#utf8{:?}", self.as_str())
    }
}

/// A `utf8`'s words, made at compile time (a literal, `#utf8"…"`): `W` is
/// `utf8_words` of its length.
pub struct Utf8Lit<const W: usize> {
    words: [u64; W],
}

impl<const W: usize> Utf8Lit<W> {
    pub const fn new(text: &str) -> Self {
        let b = text.as_bytes();
        let mut words = [0u64; W];
        words[0] = utf8_head(b.len(), text.is_ascii());
        let mut w = 0;
        while w * 8 < b.len() {
            let mut chunk = [0u8; 8];
            let mut i = 0;
            while i < 8 && w * 8 + i < b.len() {
                chunk[i] = b[w * 8 + i];
                i += 1;
            }
            words[1 + w] = u64::from_ne_bytes(chunk);
            w += 1;
        }
        Utf8Lit { words }
    }

    /// The `utf8`, its object this literal's words.
    #[inline]
    pub fn get(&'static self) -> Utf8 {
        Utf8(NonNull::from(&self.words).cast())
    }
}

/// A `utf8` literal: its object made at compile time, a `static`.
#[macro_export]
macro_rules! utf8_lit {
    ($s:expr) => {{
        const S: &str = $s;
        static O: $crate::Utf8Lit<{ $crate::utf8_words(S.len()) }> = $crate::Utf8Lit::new(S);
        O.get()
    }};
}

/// `int` (C488): one word, as Fixpt's: a fixnum (tag 000, 61 bits, its
/// value the word shifted right three), or a bignum (tag 100, the address
/// of its object's suffix, as Fixpt's bloblet tag says): its object kind 6,
/// bignum, untraced, its suffix the words `[negative?, limb count, limbs…]`,
/// limbs of 64 bits, least significant first, as Fixpt's runtime writes
/// one. A value is a fixnum whenever it fits 61 bits: one representation
/// each.
#[derive(Clone, Copy)]
pub struct Int(*const u64);

// SAFETY: a fixnum is a number; a bignum's object is never written after
// it is made, and kept for the run.
unsafe impl Send for Int {}
unsafe impl Sync for Int {}

/// An `Int` as it is: a fixnum's value, or a bignum.
pub enum IntView {
    Small(i64),
    Big(BigAt),
}

/// A bignum's suffix: `[negative?, limb count, limbs…]`.
#[derive(Clone, Copy)]
pub struct BigAt(&'static [u64]);

impl BigAt {
    #[inline]
    pub fn negative(self) -> bool {
        self.0[0] != 0
    }

    /// Its limbs, 64 bits each, least significant first.
    #[inline]
    pub fn limbs(self) -> &'static [u64] {
        &self.0[2..]
    }

    /// Its magnitude, base 2^32, least significant first, as `int.rs`
    /// computes in.
    pub fn magnitude(self) -> Vec<u32> {
        let mut m: Vec<u32> = self
            .limbs()
            .iter()
            .flat_map(|l| [*l as u32, (*l >> 32) as u32])
            .collect();
        while m.last() == Some(&0) {
            m.pop();
        }
        m
    }

    /// Its object's bytes: its header and its suffix.
    pub fn object_bytes(self) -> usize {
        8 * (1 + self.0.len())
    }
}

/// A fixnum's bounds: 61 bits.
pub const FIXNUM_MIN: i64 = -(1 << 60);
pub const FIXNUM_MAX: i64 = (1 << 60) - 1;
const BIGNUM_TAG: usize = 0b100;

impl Int {
    /// The fixnum `n`, which fits 61 bits (a constant's).
    #[inline]
    pub const fn fixnum(n: i64) -> Int {
        assert!(FIXNUM_MIN <= n && n <= FIXNUM_MAX, "a fixnum past 61 bits");
        Int(std::ptr::without_provenance((n << 3) as usize))
    }

    /// `n`: a fixnum if it fits, else a bignum.
    #[inline]
    pub fn of_i64(n: i64) -> Int {
        if (FIXNUM_MIN..=FIXNUM_MAX).contains(&n) {
            Int(std::ptr::without_provenance((n << 3) as usize))
        } else {
            let m = n.unsigned_abs();
            Int::bignum(n < 0, &[m])
        }
    }

    /// A bignum of sign `negative` and limbs `limbs` (64 bits, least
    /// significant first, no zero at the top, its value past 61 bits).
    pub(crate) fn bignum(negative: bool, limbs: &[u64]) -> Int {
        let n = limbs.len();
        let ws = words(3 + n);
        ws[0] = layout::header(KIND_BIGNUM, 0, 8 * (2 + n), true, true);
        ws[1] = negative as u64;
        ws[2] = n as u64;
        ws[3..].copy_from_slice(limbs);
        crate::stats::note::<BigAt>(8 * (3 + n));
        #[cfg(feature = "heap-stats")]
        crate::census::object(ws.as_ptr() as usize, 8 * (3 + n), trace_nothing);
        // Its suffix's address, tagged: its provenance the object's.
        Int(ws[1..].as_ptr().map_addr(|a| a | BIGNUM_TAG))
    }

    /// It, as a fixnum or a bignum.
    #[inline]
    pub fn view(self) -> IntView {
        let w = self.0.addr();
        if w & 0b111 == 0 {
            IntView::Small((w as i64) >> 3)
        } else {
            let p = self.0.map_addr(|a| a & !0b111);
            // SAFETY: a bignum's suffix, made by `bignum` (its count the
            // second word), kept for the run and never written again.
            IntView::Big(BigAt(unsafe {
                std::slice::from_raw_parts(p, 2 + *p.add(1) as usize)
            }))
        }
    }

    /// Its value, if a fixnum.
    #[inline]
    pub fn small(self) -> Option<i64> {
        let w = self.0.addr();
        if w & 0b111 == 0 {
            Some((w as i64) >> 3)
        } else {
            None
        }
    }

    /// Its word, as Fixpt's: a fixnum's own, or a bignum's tagged address.
    #[inline]
    pub fn word(self) -> usize {
        self.0.addr()
    }
}

const KIND_BIGNUM: u8 = kayak_layout::kind("bignum");

/// An object a census notes and nothing in it to trace (a bignum's).
#[cfg(feature = "heap-stats")]
unsafe fn trace_nothing(_: usize, _: &mut crate::Tracer<'static>) {}

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
    ws[0].set(layout::header(
        kayak_layout::kind("bytevector"),
        0,
        n,
        true,
        false,
    ));
    ws[1].set(layout::trailer(1));
    for (i, chunk) in bytes.chunks(8).enumerate() {
        let mut w = [0u8; 8];
        w[..chunk.len()].copy_from_slice(chunk);
        ws[2 + i].set(u64::from_ne_bytes(w));
    }
    ws
}

impl<'r> BytesAt<'r> {
    /// Its word, as a heap object holds it (C508): its suffix's address,
    /// past its header and trailer, tagged `100`, as Fixpt's.
    #[inline(always)]
    pub fn word(self) -> HeapWord {
        HeapWord::object_at(self.0.as_ptr().cast_const().cast::<u64>().wrapping_add(2))
    }

    /// The bytevector whose word is `w` (`word`'s).
    #[inline(always)]
    pub fn of_word(w: HeapWord) -> Self {
        let p = w.0.map_addr(|a| a & !TAG_MASK).wrapping_sub(2);
        match NonNull::new(p.cast_mut().cast::<Cell<u64>>()) {
            Some(p) => BytesAt(p, std::marker::PhantomData),
            None => crate::fail("a heap's word read as a bytevector, which it is not"),
        }
    }
}

/// `w` words of the heap's space (C507), zeroed, each a cell, kept for the
/// run: an object's, which its maker writes.
pub(crate) fn kept_cells<T: Copy>(w: usize) -> &'static [Cell<T>] {
    const {
        assert!(
            std::mem::size_of::<T>() == 8 && std::mem::align_of::<T>() == 8,
            "a cell not a word"
        )
    };
    // SAFETY: `w` new words of the space, zeroed, kept for the run, given
    // to the caller alone; a cell of a word-sized `Copy` type all of whose
    // bit patterns are values (`u64`, `i64`).
    unsafe { std::slice::from_raw_parts(space::words(w).cast::<Cell<T>>(), w) }
}

/// A bytevector's object of `bytes`, in the heap's space (C507).
pub(crate) fn bytes_kept(bytes: &[u8]) -> &'static [Cell<u64>] {
    let n = bytes.len();
    let ws = kept_cells::<u64>(string_words(n));
    ws[0].set(layout::header(
        kayak_layout::kind("bytevector"),
        0,
        n,
        true,
        false,
    ));
    ws[1].set(layout::trailer(1));
    for (i, chunk) in bytes.chunks(8).enumerate() {
        let mut w = [0u8; 8];
        w[..chunk.len()].copy_from_slice(chunk);
        ws[2 + i].set(u64::from_ne_bytes(w));
    }
    #[cfg(feature = "heap-stats")]
    crate::census::object(ws.as_ptr() as usize, 8 * ws.len(), trace_nothing);
    ws
}

/// A bytevector literal's object (`#u8(…)`, `#u8"…"`, C527): `bytes`,
/// outside the heap's space, so no collection moves or frees it, kept for
/// the run, as a string literal's `static` is. Frozen: the checker lets
/// nothing write a `(bytevectorof const)`.
pub fn bytes_lit_made(bytes: &[u8]) -> Bytes {
    bytes_at(Box::leak(bytes_object(bytes)))
}

/// A bytevector literal: its object made once on each thread, at its
/// first use (a `Bytes` is no `static`'s, its words cells).
#[macro_export]
macro_rules! bytes_lit {
    ($b:expr) => {{
        ::std::thread_local! {
            static O: ::core::cell::Cell<::core::option::Option<$crate::Bytes>> =
                const { ::core::cell::Cell::new(::core::option::Option::None) };
        }
        O.with(|o| match o.get() {
            ::core::option::Option::Some(b) => b,
            ::core::option::Option::None => {
                let b = $crate::bytes_lit_made($b);
                o.set(::core::option::Option::Some(b));
                b
            }
        })
    }};
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

/// A bloblet's head (C484): its header, its fields, its trailer; its
/// suffix's words follow it, in the same block.
#[repr(C)]
pub struct BlobHead<Fs> {
    header: u64,
    pub fields: Fs,
    trailer: u64,
}

/// `(bloblet (fields T …) R)`, as a value: one word, to its object (kind
/// 32, bloblet): its fields, each a cell, and its suffix, words of cells.
pub struct BlobletAt<'r, Fs>(
    NonNull<BlobHead<Fs>>,
    std::marker::PhantomData<&'r BlobHead<Fs>>,
);

impl<Fs> Clone for BlobletAt<'_, Fs> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<Fs> Copy for BlobletAt<'_, Fs> {}

impl<'r, Fs> BlobletAt<'r, Fs> {
    #[inline]
    fn head(self) -> &'r BlobHead<Fs> {
        // SAFETY: made only by `Bloblet::at`, of its block, alive for `'r`.
        unsafe { self.0.as_ref() }
    }

    /// Its fields, each a cell.
    #[inline]
    pub fn fields(self) -> &'r Fs {
        &self.head().fields
    }

    /// Its suffix's length, in bytes.
    #[inline]
    pub fn bytes(self) -> usize {
        (self.head().header >> 32) as usize
    }

    /// Whether its suffix is frozen (a sealed copy's, C116), as its
    /// header's bit says.
    #[inline]
    pub fn suffix_frozen(self) -> bool {
        self.head().header & (1 << 13) != 0
    }

    /// Its suffix's words.
    #[inline]
    pub(crate) fn suffix(self) -> &'r [Cell<u64>] {
        // SAFETY: the words after its head, `bytes` rounded up to words of
        // them, in the block it was made in (whose pointer it keeps), alive
        // for `'r`; cells, written only through `Cell`.
        unsafe {
            std::slice::from_raw_parts(
                self.0
                    .as_ptr()
                    .cast::<u8>()
                    .add(std::mem::size_of::<BlobHead<Fs>>())
                    .cast::<Cell<u64>>(),
                self.bytes().div_ceil(8),
            )
        }
    }

    /// Where its object is: an identity, and a census's address.
    #[inline]
    pub fn addr(self) -> usize {
        self.0.as_ptr() as usize
    }

    /// Its word (C503): Fixpt's bloblet word, tagged `100`, to its suffix's
    /// start, one past its trailer.
    #[inline(always)]
    pub fn word(self) -> HeapWord {
        HeapWord::object_at(
            self.0
                .as_ptr()
                .cast::<u8>()
                .wrapping_add(std::mem::size_of::<BlobHead<Fs>>())
                .cast::<u64>(),
        )
    }

    /// The bloblet a word is.
    #[inline(always)]
    pub fn of_word(w: HeapWord) -> Self {
        // SAFETY: a bloblet's word is made only by `word`, of its object,
        // alive for `'r`, its trailer just before where the word points.
        let h = unsafe { w.header_ptr() };
        BlobletAt(
            // SAFETY: as above, its head's address.
            unsafe { NonNull::new_unchecked(h.cast::<BlobHead<Fs>>().cast_mut()) },
            std::marker::PhantomData,
        )
    }
}

/// A bloblet's head: its fields visited (C486), its suffix plain bytes.
impl<Fs: crate::Visit> crate::Visit for BlobHead<Fs> {
    fn visit<'a>(&'a self, t: &mut crate::Tracer<'a>) {
        self.fields.visit(t);
    }
    fn visit_size(&self) -> usize {
        std::mem::size_of::<Self>() + ((self.header >> 32) as usize).next_multiple_of(8)
    }
    fn visit_name(&self) -> &'static str {
        std::any::type_name::<Bloblet<Fs>>()
    }
}

/// A bloblet: its object reached (C486); held as its word (C503).
impl<Fs: crate::Visit> crate::Trace for BlobletAt<'_, Fs> {
    type Stored = HeapWord;
    const VALUE_WORD: bool = true;
    #[inline(always)]
    fn store(self) -> HeapWord {
        self.word()
    }
    #[inline(always)]
    fn load(w: HeapWord) -> Self {
        BlobletAt::of_word(w)
    }
    #[inline]
    fn trace<'a>(self, t: &mut crate::Tracer<'a>)
    where
        Self: 'a,
    {
        t.reach(self.head());
    }
}

/// A bloblet's object at `addr`, traced (a census's, C486).
///
/// # Safety
/// `addr` is a bloblet's object of fields `Fs`, kept for the run.
#[cfg(feature = "heap-stats")]
unsafe fn trace_bloblet<Fs: crate::Visit + 'static>(addr: usize, t: &mut crate::Tracer<'static>) {
    // SAFETY: the caller's.
    let b = BlobletAt::<'static, Fs>(
        unsafe { NonNull::new_unchecked(addr as *mut BlobHead<Fs>) },
        std::marker::PhantomData,
    );
    crate::Trace::trace(b, t);
}

/// A bloblet's object, owned: what a region's arena holds of one made in
/// it, freed with the region (C267); one of the heap's is kept.
pub struct Bloblet<Fs> {
    at: NonNull<BlobHead<Fs>>,
    layout: std::alloc::Layout,
}

impl<Fs> Bloblet<Fs> {
    /// One of `fields` and a zeroed suffix of `bytes`; its suffix frozen if
    /// `frozen` (a sealed copy's).
    pub(crate) fn new(fields: Fs, bytes: usize, frozen: bool) -> Self {
        let head = std::alloc::Layout::new::<BlobHead<Fs>>();
        // Its fields as words to the trailer, as Fixpt counts them.
        let to_trailer = std::mem::offset_of!(BlobHead<Fs>, trailer) / 8;
        let words = bytes.div_ceil(8);
        let layout = head
            .extend(std::alloc::Layout::array::<Cell<u64>>(words).expect("a bloblet's suffix"))
            .expect("a bloblet's size")
            .0
            .pad_to_align();
        // SAFETY: a layout of nonzero size (its head has two words);
        // written whole before it is read: the head, then the suffix zeroed.
        unsafe {
            let p = std::alloc::alloc(layout).cast::<BlobHead<Fs>>();
            let Some(at) = NonNull::new(p) else {
                std::alloc::handle_alloc_error(layout)
            };
            at.as_ptr().write(BlobHead {
                header: layout::header(layout::KIND_BLOBLET, to_trailer, bytes, false, frozen),
                fields,
                trailer: layout::trailer(to_trailer),
            });
            std::ptr::write_bytes(p.cast::<u8>().add(head.size()), 0, words * 8);
            Bloblet { at, layout }
        }
    }

    /// It, as a value, alive as long as it is.
    #[inline]
    pub fn at(&self) -> BlobletAt<'_, Fs> {
        BlobletAt(self.at, std::marker::PhantomData)
    }

    /// One of `fields` and a zeroed suffix of `bytes`, in the heap's space
    /// (C507), kept for the run; its suffix frozen if `frozen`.
    pub(crate) fn kept_new<'a>(fields: Fs, bytes: usize, frozen: bool) -> BlobletAt<'a, Fs>
    where
        Fs: crate::Visit + 'static,
    {
        const {
            assert!(
                std::mem::align_of::<BlobHead<Fs>>() <= 8,
                "a bloblet's fields aligned past a word"
            )
        };
        let to_trailer = std::mem::offset_of!(BlobHead<Fs>, trailer) / 8;
        let size = std::mem::size_of::<BlobHead<Fs>>() + bytes.div_ceil(8) * 8;
        let p = space::words(size / 8).cast::<BlobHead<Fs>>();
        // SAFETY: new words of the space, zeroed (the suffix so), enough
        // for the head and the suffix, aligned for the head; written once.
        unsafe {
            p.write(BlobHead {
                header: layout::header(layout::KIND_BLOBLET, to_trailer, bytes, false, frozen),
                fields,
                trailer: layout::trailer(to_trailer),
            });
        }
        #[cfg(feature = "heap-stats")]
        crate::census::object(p as usize, size, trace_bloblet::<Fs>);
        crate::stats::note::<Bloblet<Fs>>(size);
        // SAFETY: not null, a word of the space.
        BlobletAt(
            unsafe { NonNull::new_unchecked(p) },
            std::marker::PhantomData,
        )
    }
}

/// A bloblet a region's arena owns (C492): its object reached.
impl<Fs: crate::Visit> crate::Visit for Bloblet<Fs> {
    fn visit<'a>(&'a self, t: &mut crate::Tracer<'a>) {
        t.reach(self.at().head());
    }
    fn visit_size(&self) -> usize {
        0
    }
}

impl<Fs> Drop for Bloblet<Fs> {
    fn drop(&mut self) {
        // SAFETY: its block, made by `new` with `layout`, its head written
        // there; dropped once, here, its fields with it.
        unsafe {
            std::ptr::drop_in_place(std::ptr::addr_of_mut!((*self.at.as_ptr()).fields));
            std::alloc::dealloc(self.at.as_ptr().cast(), self.layout);
        }
    }
}

#[cfg(test)]
mod bloblet_tests {
    use super::*;

    #[test]
    fn a_bloblet_its_fields_and_suffix() {
        let b = Bloblet::new(
            (crate::Slot::new(crate::int_lit(7)), crate::Slot::new('x')),
            12,
            false,
        );
        let at = b.at();
        assert_eq!(at.fields().0.get().small(), Some(7));
        at.fields().1.set('y');
        assert_eq!(at.bytes(), 12);
        assert_eq!(at.suffix().len(), 2);
        at.suffix()[1].set(5);
        assert_eq!(at.suffix()[1].get(), 5);
        assert_eq!(at.fields().1.get(), 'y');
        assert!(!at.suffix_frozen());
        assert_eq!(std::mem::size_of::<BlobletAt<'_, (Cell<i64>,)>>(), 8);
        // Its fields Fixpt words (C503): a fixnum's, 7 << 3; its own word
        // Fixpt's, tagged `100`, past its trailer, back to the same bloblet.
        // SAFETY: its head, alive with `b`.
        let words = unsafe { std::slice::from_raw_parts(at.0.as_ptr().cast::<u64>(), 4) };
        assert_eq!(words[1], 7 << 3);
        assert_eq!(words[3] & 7, 5);
        use crate::Trace;
        let w = at.store();
        assert_eq!(w.raw() & 7, 4);
        assert_eq!(
            BlobletAt::<'_, (crate::Slot<crate::Int>, crate::Slot<char>)>::load(w).addr(),
            at.addr()
        );
    }
}

/// A flat as a typed suffix holds it (C32), all zero bytes to begin with,
/// as FX-26's suffix is: a scalar's cell, an array of flats, a product of
/// them (the emitter's shapes, C487).
///
/// # Safety
/// A value of all zero bytes is a valid one, and it needs no drop: a typed
/// suffix is made zeroed in its block, never built on a stack, never
/// dropped.
pub unsafe trait Flat {}

macro_rules! flat_scalar {
    ($($t:ty),*) => {
        // SAFETY: a cell of a number, zero a valid one, nothing to drop.
        $(unsafe impl Flat for Cell<$t> {})*
    };
}

flat_scalar!(i64, u64, f64, i32, u32, f32);

/// `(arrayof F n)` in a typed suffix: `n` `F`s, in place (C487).
#[repr(transparent)]
pub struct FlatArr<T, const N: usize>([T; N]);

// SAFETY: an array of flats, each zero valid and needing no drop.
unsafe impl<T: Flat, const N: usize> Flat for FlatArr<T, N> {}

/// A blobref to an array in a typed suffix (C492): a pointer into its
/// owner's suffix, which a collector finds by the owner's trailer
/// (Bartlett's interior pointer); opaque to a trace, holding no reference.
impl<T, const N: usize> crate::Trace for &'_ FlatArr<T, N> {
    crate::trace_whole!();
    fn trace<'a>(self, t: &mut crate::Tracer<'a>)
    where
        Self: 'a,
    {
        t.opaque(std::ptr::from_ref(self) as usize);
    }
}

impl<T, const N: usize> FlatArr<T, N> {
    /// `(blobref-index r i)`.
    #[inline]
    pub fn at(&self, i: crate::Int) -> &T {
        match i.small() {
            Some(i) if (0..N as i64).contains(&i) => &self.0[i as usize],
            _ => crate::fail("blobref-index: out of range"),
        }
    }
}

/// A typed-suffix bloblet's object (C485, C487): its header (kind 32, its
/// suffix's bytes), its fields, its trailer, its suffix, one block.
#[repr(C)]
pub struct TBlobHead<Fs, S> {
    header: u64,
    pub fields: Fs,
    trailer: u64,
    suffix: S,
}

/// A bloblet of a typed suffix, `S`, as a value: one word, to its object.
pub struct TBlobletAt<'r, Fs, S>(
    NonNull<TBlobHead<Fs, S>>,
    std::marker::PhantomData<&'r TBlobHead<Fs, S>>,
);

impl<Fs, S> Clone for TBlobletAt<'_, Fs, S> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<Fs, S> Copy for TBlobletAt<'_, Fs, S> {}

impl<'r, Fs, S> TBlobletAt<'r, Fs, S> {
    #[inline]
    fn head(self) -> &'r TBlobHead<Fs, S> {
        // SAFETY: made only by `TBloblet::at`, of its block, alive for `'r`.
        unsafe { self.0.as_ref() }
    }

    /// Its fields, each a cell.
    #[inline]
    pub fn fields(self) -> &'r Fs {
        &self.head().fields
    }

    /// Its suffix, the flat (a blobref at 0).
    #[inline]
    pub fn suffix(self) -> &'r S {
        &self.head().suffix
    }

    /// Its suffix's length, in bytes, as its header says.
    #[inline]
    pub fn byte_size(self) -> usize {
        (self.head().header >> 32) as usize
    }

    /// Where its object is.
    #[inline]
    pub fn addr(self) -> usize {
        self.0.as_ptr() as usize
    }

    /// Its word (C503): Fixpt's bloblet word, tagged `100`, to its suffix's
    /// start, one past its trailer.
    #[inline(always)]
    pub fn word(self) -> HeapWord {
        HeapWord::object_at(
            self.0
                .as_ptr()
                .cast::<u8>()
                .wrapping_add(std::mem::offset_of!(TBlobHead<Fs, S>, suffix))
                .cast::<u64>(),
        )
    }

    /// The bloblet a word is.
    #[inline(always)]
    pub fn of_word(w: HeapWord) -> Self {
        // SAFETY: a bloblet's word is made only by `word`, of its object,
        // alive for `'r`, its trailer just before where the word points.
        let h = unsafe { w.header_ptr() };
        TBlobletAt(
            // SAFETY: as above, its head's address.
            unsafe { NonNull::new_unchecked(h.cast::<TBlobHead<Fs, S>>().cast_mut()) },
            std::marker::PhantomData,
        )
    }
}

/// A typed-suffix bloblet's object, owned: a region's arena holds one made
/// in it; one of the heap's is kept.
pub struct TBloblet<Fs, S> {
    at: NonNull<TBlobHead<Fs, S>>,
}

impl<Fs, S: Flat> TBloblet<Fs, S> {
    /// One of `fields` and a zeroed suffix, its header saying `bytes`.
    pub(crate) fn new(bytes: usize, fields: Fs) -> Self {
        let lay = std::alloc::Layout::new::<TBlobHead<Fs, S>>();
        let to_trailer = std::mem::offset_of!(TBlobHead<Fs, S>, trailer) / 8;
        // SAFETY: a layout of nonzero size (two words at least), zeroed, so
        // its suffix a valid `S` (`Flat`); its header, fields and trailer
        // written before it is read.
        unsafe {
            let p = std::alloc::alloc_zeroed(lay).cast::<TBlobHead<Fs, S>>();
            let Some(at) = NonNull::new(p) else {
                std::alloc::handle_alloc_error(lay)
            };
            std::ptr::addr_of_mut!((*p).header).write(layout::header(
                layout::KIND_BLOBLET,
                to_trailer,
                bytes,
                false,
                false,
            ));
            std::ptr::addr_of_mut!((*p).fields).write(fields);
            std::ptr::addr_of_mut!((*p).trailer).write(layout::trailer(to_trailer));
            TBloblet { at }
        }
    }
}

impl<Fs, S> TBloblet<Fs, S> {
    /// It, as a value, alive as long as it is.
    #[inline]
    pub fn at(&self) -> TBlobletAt<'_, Fs, S> {
        TBlobletAt(self.at, std::marker::PhantomData)
    }

    /// One of `fields` and a zeroed suffix, its header saying `bytes`, in
    /// the heap's space (C507), kept for the run.
    pub(crate) fn kept_new<'a>(bytes: usize, fields: Fs) -> TBlobletAt<'a, Fs, S>
    where
        Fs: crate::Visit + 'static,
        S: Flat + 'static,
    {
        let to_trailer = std::mem::offset_of!(TBlobHead<Fs, S>, trailer) / 8;
        let size = std::mem::size_of::<TBlobHead<Fs, S>>();
        const {
            assert!(
                std::mem::align_of::<TBlobHead<Fs, S>>() <= 8,
                "a bloblet's suffix aligned past a word"
            )
        };
        let p = space::words(size.div_ceil(8)).cast::<TBlobHead<Fs, S>>();
        // SAFETY: new words of the space, zeroed, so its suffix a valid
        // `S` (`Flat`); its header, fields and trailer written before it is
        // read.
        unsafe {
            std::ptr::addr_of_mut!((*p).header).write(layout::header(
                layout::KIND_BLOBLET,
                to_trailer,
                bytes,
                false,
                false,
            ));
            std::ptr::addr_of_mut!((*p).fields).write(fields);
            std::ptr::addr_of_mut!((*p).trailer).write(layout::trailer(to_trailer));
        }
        #[cfg(feature = "heap-stats")]
        crate::census::object(p as usize, size, trace_tbloblet::<Fs, S>);
        crate::stats::note::<TBloblet<Fs, S>>(size);
        // SAFETY: not null, a word of the space.
        TBlobletAt(
            unsafe { NonNull::new_unchecked(p) },
            std::marker::PhantomData,
        )
    }
}

impl<Fs, S> Drop for TBloblet<Fs, S> {
    fn drop(&mut self) {
        // SAFETY: its block, made by `new` with this layout; its fields
        // dropped once, here; its suffix needs no drop (`Flat`).
        unsafe {
            std::ptr::drop_in_place(std::ptr::addr_of_mut!((*self.at.as_ptr()).fields));
            std::alloc::dealloc(
                self.at.as_ptr().cast(),
                std::alloc::Layout::new::<TBlobHead<Fs, S>>(),
            );
        }
    }
}

/// A typed-suffix bloblet's head: its fields visited, its suffix plain
/// bytes (C487).
/// A typed-suffix bloblet a region's arena owns (C492): its object reached.
impl<Fs: crate::Visit, S> crate::Visit for TBloblet<Fs, S> {
    fn visit<'a>(&'a self, t: &mut crate::Tracer<'a>) {
        t.reach(self.at().head());
    }
    fn visit_size(&self) -> usize {
        0
    }
}

impl<Fs: crate::Visit, S> crate::Visit for TBlobHead<Fs, S> {
    fn visit<'a>(&'a self, t: &mut crate::Tracer<'a>) {
        self.fields.visit(t);
    }
    fn visit_name(&self) -> &'static str {
        std::any::type_name::<TBloblet<Fs, S>>()
    }
}

/// A typed-suffix bloblet: its object reached (C487); held as its word
/// (C503).
impl<Fs: crate::Visit, S> crate::Trace for TBlobletAt<'_, Fs, S> {
    type Stored = HeapWord;
    const VALUE_WORD: bool = true;
    #[inline(always)]
    fn store(self) -> HeapWord {
        self.word()
    }
    #[inline(always)]
    fn load(w: HeapWord) -> Self {
        TBlobletAt::of_word(w)
    }
    #[inline]
    fn trace<'a>(self, t: &mut crate::Tracer<'a>)
    where
        Self: 'a,
    {
        t.reach(self.head());
    }
}

/// A typed-suffix bloblet's object at `addr`, traced (a census's).
///
/// # Safety
/// `addr` is such an object of fields `Fs` and suffix `S`, kept for the run.
#[cfg(feature = "heap-stats")]
unsafe fn trace_tbloblet<Fs: crate::Visit + 'static, S: 'static>(
    addr: usize,
    t: &mut crate::Tracer<'static>,
) {
    // SAFETY: the caller's.
    let b = TBlobletAt::<'static, Fs, S>(
        unsafe { NonNull::new_unchecked(addr as *mut TBlobHead<Fs, S>) },
        std::marker::PhantomData,
    );
    crate::Trace::trace(b, t);
}

/// `(flatarrayof T R)` (C489): one word, to its first element, as Fixpt's
/// bloblet tag points at a suffix: Fixpt's flat array's header (kind 42,
/// its elements' bytes the suffix's), its element code and its trailer in
/// the three words before it (C126), its elements each a cell. Native code
/// is given it as itself, shared (`native.rs`).
pub struct FlatArray<T: 'static>(
    NonNull<Cell<T>>,
    std::marker::PhantomData<&'static [Cell<T>]>,
);

impl<T> Clone for FlatArray<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for FlatArray<T> {}

/// A flat array's words before its first element: header, element code,
/// trailer.
pub(crate) const FLAT_PREFIX: usize = 3;

impl FlatArray<i64> {
    /// One of the cells `all`, its first `FLAT_PREFIX` the header, element
    /// code and trailer, the rest its elements; kept for the run.
    pub(crate) fn of_cells(all: &'static [Cell<i64>]) -> Self {
        assert!(all.len() >= FLAT_PREFIX, "a flat array's prefix");
        // The elements' start, its provenance the whole block's.
        let p = all.as_ptr().wrapping_add(FLAT_PREFIX);
        #[cfg(feature = "heap-stats")]
        crate::census::object(all.as_ptr() as usize, 8 * all.len(), trace_nothing);
        FlatArray(
            NonNull::new(p.cast_mut()).expect("a flat array"),
            std::marker::PhantomData,
        )
    }
}

impl<T> FlatArray<T> {
    /// Its header, as Fixpt's.
    #[inline]
    fn header(self) -> u64 {
        // SAFETY: made only by `of_cells`, its header `FLAT_PREFIX` words
        // before its first element, in the block it points into, kept.
        unsafe { *self.0.as_ptr().cast::<u64>().sub(FLAT_PREFIX) }
    }

    /// Its elements, how many: its suffix's bytes, as its header says.
    #[inline]
    pub fn len(self) -> usize {
        (self.header() >> 32) as usize / std::mem::size_of::<T>()
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    /// Its elements, each a cell.
    #[inline]
    pub fn cells(self) -> &'static [Cell<T>] {
        // SAFETY: its `len` elements from its first, in its block, kept;
        // cells, written only through `Cell`.
        unsafe { std::slice::from_raw_parts(self.0.as_ptr(), self.len()) }
    }

    /// Its first element's address: its word to native code, untagged.
    #[inline]
    pub fn addr(self) -> usize {
        self.0.as_ptr() as usize
    }

    /// Its object's bytes.
    pub fn object_bytes(self) -> usize {
        8 * FLAT_PREFIX + (self.header() >> 32) as usize
    }

    /// Its word (C502): Fixpt's, tagged `100`, to its suffix's start, its
    /// first element, past its trailer.
    #[inline(always)]
    pub fn word(self) -> HeapWord {
        HeapWord::object_at(self.0.as_ptr().cast::<u64>())
    }

    /// The flat array a word is.
    #[inline(always)]
    pub fn of_word(w: HeapWord) -> Self {
        match NonNull::new(w.object_ptr().cast::<Cell<T>>().cast_mut()) {
            Some(p) => FlatArray(p, std::marker::PhantomData),
            None => crate::fail("a heap's word read as a flat array, which it is not"),
        }
    }
}

/// `(arrayof T R)` (C485): one word, to an object of Fixpt's layout: its
/// header (kind 3, vector; its fields its elements' words and its trailer,
/// as Fixpt counts them, C498), and, past what the header's 18 bits count,
/// Fixpt's extension word before it (kind 255, the count above it, the
/// main header `large`); its elements, each a cell; its trailer. In the
/// heap, kept.
pub struct Array<T: crate::Trace + 'static>(
    NonNull<u64>,
    std::marker::PhantomData<&'static [crate::Slot<T>]>,
);

impl<T: crate::Trace> Clone for Array<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: crate::Trace> Copy for Array<T> {}

/// The header's field count, 18 bits; and its `large` bit.
const FIELDS_MAX: usize = (1 << 18) - 1;
const LARGE: u64 = 1 << 11;

/// Fixpt's kind of an extension word.
const KIND_EXTENSION: u64 = kayak_layout::KIND_EXTENSION as u64;

/// An array of `n` `T`s's fields, as Fixpt counts them: its elements'
/// words and its trailer.
const fn field_words<T: crate::Trace>(n: usize) -> usize {
    (n * std::mem::size_of::<crate::Slot<T>>()).div_ceil(8) + 1
}

/// Where an array's elements begin, in words: after one header word, or
/// two for a large one.
const fn elements_at<T: crate::Trace>(n: usize) -> usize {
    if field_words::<T>(n) > FIELDS_MAX {
        2
    } else {
        1
    }
}

impl<T: crate::Trace> Array<T> {
    /// Its fields as Fixpt counts them (its elements' words and its
    /// trailer): its header's, or, `large`, its extension word's.
    #[inline(always)]
    fn fields(self) -> usize {
        // SAFETY: an `Array` is made only by `array_made` or `of_word`, of
        // its main header, the extension word before it if `large`.
        unsafe {
            let h = *self.0.as_ptr();
            if h & LARGE != 0 {
                (*self.0.as_ptr().sub(1) >> 11) as usize
            } else {
                ((h >> 14) & FIELDS_MAX as u64) as usize
            }
        }
    }

    /// Its elements, how many.
    #[inline]
    pub fn len(self) -> usize {
        let size = std::mem::size_of::<crate::Slot<T>>();
        if size == 0 {
            return 0;
        }
        (self.fields() - 1) * 8 / size
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    /// Its elements, each a cell, its value held (C490: a product boxed).
    #[inline]
    pub fn cells(self) -> &'static [crate::Slot<T>] {
        let n = self.len();
        // SAFETY: its elements follow its main header, `n` of them, in the
        // block it was made in (whose pointer it keeps), kept for the run.
        unsafe { std::slice::from_raw_parts(self.0.as_ptr().add(1).cast::<crate::Slot<T>>(), n) }
    }

    /// Where its object is: its identity, its main header's address.
    #[inline]
    pub fn addr(self) -> usize {
        self.0.as_ptr() as usize
    }

    /// Its object's bytes: its header words, its elements, its trailer.
    pub fn object_bytes(self) -> usize {
        let n = self.len();
        (elements_at::<T>(n) * 8 + n * std::mem::size_of::<crate::Slot<T>>()).next_multiple_of(8)
            + 8
    }

    /// Its word (C502): Fixpt's, tagged `100`, to its suffix's start, one
    /// past its trailer (a vector has no suffix), as Fixpt's word to an
    /// object is.
    #[inline(always)]
    pub fn word(self) -> HeapWord {
        // Its trailer `fields` words past its main header.
        HeapWord::object_at(self.0.as_ptr().wrapping_add(self.fields() + 1))
    }

    /// The array a word is: its main header, as its trailer says how far
    /// back it is.
    #[inline(always)]
    pub fn of_word(w: HeapWord) -> Self {
        let at = w.object_ptr();
        // SAFETY: an array's word is made only by `word`, of an object kept
        // for the run, whose trailer, just before where the word points,
        // says how far back its main header is (Fixpt's distance, 61 bits).
        unsafe {
            let back = (*at.sub(1) >> 3) as usize;
            Array(
                NonNull::new_unchecked(at.sub(1 + back).cast_mut()),
                std::marker::PhantomData,
            )
        }
    }
}

/// An array's object at `addr`, traced (a census's, C486).
///
/// # Safety
/// `addr` is an array of `T`'s object, `array_made`'s, kept for the run.
#[cfg(feature = "heap-stats")]
unsafe fn trace_array<T: crate::Trace + 'static>(addr: usize, t: &mut crate::Tracer<'static>) {
    // SAFETY: the caller's: its block's first word its main header, or,
    // large, the extension word before it.
    let main = unsafe {
        let p = addr as *mut u64;
        if (*p >> 3) & 0xff == KIND_EXTENSION {
            p.add(1)
        } else {
            p
        }
    };
    // SAFETY: as above, `main` its main header.
    let a = Array::<T>(
        unsafe { NonNull::new_unchecked(main) },
        std::marker::PhantomData,
    );
    crate::Trace::trace(a, t);
}

/// An array of `items`, kept for the run: Fixpt's plain bloblet (kind 32,
/// its elements its fields, no suffix), as Fixpt's `make-array` makes one
/// (`%make-bloblet-filled`; C506, where it was a vector).
pub(crate) fn array_made<T: crate::Trace + 'static>(
    items: impl ExactSizeIterator<Item = T>,
) -> Array<T> {
    array_made_of(layout::KIND_BLOBLET, items)
}

/// An array of `items` whose object is of kind `kind`: a plain bloblet's
/// (32), or a vector's (3, a datum's, C505).
pub(crate) fn array_made_of<T: crate::Trace + 'static>(
    kind: u8,
    items: impl ExactSizeIterator<Item = T>,
) -> Array<T> {
    const {
        assert!(
            std::mem::align_of::<crate::Slot<T>>() <= 8,
            "an array's element aligned past a word"
        );
        assert!(
            std::mem::size_of::<crate::Slot<T>>().is_multiple_of(8),
            "an array's element not whole words"
        )
    };
    let n = items.len();
    let at = elements_at::<T>(n) * 8;
    let trailer_at = (at + n * std::mem::size_of::<crate::Slot<T>>()).next_multiple_of(8);
    let layout = std::alloc::Layout::from_size_align(trailer_at + 8, 8).expect("an array's size");
    crate::stats::note::<Array<T>>(layout.size());
    // SAFETY: a layout of nonzero size, aligned to a word; each word of
    // the header, each element and the trailer written before it is read.
    unsafe {
        let p = space::words(layout.size() / 8);
        let fields = field_words::<T>(n);
        if fields > FIELDS_MAX {
            // Fixpt's: the extension word, then the main header, `large`.
            p.write(layout::TAG_HEADER | KIND_EXTENSION << 3 | (fields as u64) << 11);
            p.add(1)
                .write(layout::header(kind, 0, 0, false, false) | LARGE);
        } else {
            p.write(layout::header(kind, fields, 0, false, false));
        }
        let cells = p.cast::<u8>().add(at).cast::<crate::Slot<T>>();
        let mut k = 0;
        // An item made may allocate, and so collect: the array old then,
        // what is written after it perhaps a young object's word, no
        // barrier passed. So each cell written once one has run noted
        // (`space::epoch`), as it is written: the next item's allocation
        // may collect again.
        let born = space::epoch();
        for x in items.take(n) {
            cells.add(k).write(crate::Slot::new(x));
            if space::epoch() != born {
                (*cells.add(k)).note();
            }
            k += 1;
        }
        assert!(k == n, "an array's items fewer than it said");
        // Its main header, after the extension word if there is one; its
        // trailer's distance back to it, as Fixpt's (the fields).
        let main = p.add(at / 8 - 1);
        debug_assert!(trailer_at / 8 == at / 8 - 1 + fields);
        p.cast::<u8>()
            .add(trailer_at)
            .cast::<u64>()
            .write(layout::trailer(fields));
        #[cfg(feature = "heap-stats")]
        crate::census::object(p as usize, layout.size(), trace_array::<T>);
        Array(NonNull::new_unchecked(main), std::marker::PhantomData)
    }
}

#[cfg(test)]
mod array_tests {
    use super::*;

    #[test]
    fn a_flat_array_one_word_its_header_before_it() {
        let a = crate::make_flatarray_i64(5, 7);
        assert_eq!(std::mem::size_of::<FlatArray<i64>>(), 8);
        assert_eq!(crate::flatarray_length_i64(a), 5);
        crate::flatarray_set_i64_x(a, 4, 9);
        assert_eq!(crate::flatarray_ref_i64(a, 4), 9);
        assert_eq!(crate::flatarray_ref_i64(a, 0), 7);
        assert_eq!(a.header() & layout::TAG_MASK, layout::TAG_HEADER);
        assert_eq!(a.object_bytes(), 8 * (3 + 5));
        // A big one of zeros, reserved (C139): the same.
        let big = crate::make_flatarray_i64(1 << 17, 0);
        assert_eq!(big.len(), 1 << 17);
        assert_eq!(crate::flatarray_ref_i64(big, (1 << 17) - 1), 0);
        let (rows, _) = crate::traced((a, a, big)).census();
        assert_eq!(rows.iter().map(|r| r.1).sum::<u64>(), 2);
    }

    #[test]
    fn an_array_small_and_large() {
        let a = array_made([1i64, 2, 3].into_iter());
        assert_eq!(a.len(), 3);
        a.cells()[1].set(20);
        assert_eq!(
            a.cells().iter().map(crate::Slot::get).collect::<Vec<_>>(),
            vec![1, 20, 3]
        );
        let big = array_made((0..300_000).map(|k| k as u8));
        assert_eq!(big.len(), 300_000);
        assert_eq!(big.cells()[299_999].get(), (299_999 % 256) as u8);
        // Its word Fixpt's, past its trailer, back to the same array.
        use crate::Trace;
        for x in [a, array_made([7i64].into_iter())] {
            let w = x.store();
            assert_eq!(w.raw() & 7, 4);
            assert_eq!(Array::<i64>::load(w).addr(), x.addr());
            assert_eq!(Array::<i64>::load(w).len(), x.len());
        }
        let wb = big.store();
        assert_eq!(Array::<u8>::load(wb).addr(), big.addr());
        // A large one's trailer counts back to its main header, as Fixpt's.
        // SAFETY: its word points one past its trailer, in its block.
        let back = unsafe { *wb.object_ptr().sub(1) } >> 3;
        assert_eq!(back as usize, big.fields());
        let units = array_made(std::iter::repeat_n((), 5));
        assert_eq!(units.len(), 5);
        assert_eq!(std::mem::size_of::<Array<i64>>(), 8);
    }
}

/// A product's kind (Fixpt's `product`, 37): its fields traced, never set.
const KIND_PRODUCT: u8 = kayak_layout::kind("product");

/// A product's object (C490, C496), as Fixpt's product (kind 37) is: its
/// parts' words its fields, its trailer, no suffix; frozen (a product is
/// never written).
#[repr(C)]
pub struct ProdObj<P: crate::Product> {
    header: u64,
    words: P::Words,
    trailer: u64,
}

/// A product as a heap object holds it (C490): one word, to its object.
/// On the stack a product is its parts (kc's calling convention); boxed
/// where a pair, a sum, a closure holds one, its `Trace::Stored`.
pub struct Prod<P: crate::Product>(NonNull<ProdObj<P>>, std::marker::PhantomData<P>);

impl<P: crate::Product> Clone for Prod<P> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<P: crate::Product> Copy for Prod<P> {}

// SAFETY: its object is never written after it is made, and holds what `P`
// does: shared or sent as `P` may be.
unsafe impl<P: crate::Product + Send> Send for Prod<P> {}
// SAFETY: as above.
unsafe impl<P: crate::Product + Sync> Sync for Prod<P> {}

impl<P: crate::Product> Prod<P> {
    /// `p`'s object, kept for the run (kayak-rt has no collector yet).
    pub fn new(p: P) -> Self {
        let words = p.words();
        let to_trailer = std::mem::offset_of!(ProdObj<P>, trailer) / 8;
        let o: &ProdObj<P> = space::put(ProdObj {
            header: layout::header(KIND_PRODUCT, to_trailer, 0, true, true),
            words,
            trailer: layout::trailer(to_trailer),
        });
        crate::stats::note::<ProdObj<P>>(std::mem::size_of::<ProdObj<P>>());
        #[cfg(feature = "heap-stats")]
        crate::census::one(o);
        Prod(NonNull::from(o), std::marker::PhantomData)
    }

    #[inline]
    fn obj<'a>(self) -> &'a ProdObj<P> {
        // SAFETY: made only by `new`, of a block never freed.
        unsafe { self.0.as_ref() }
    }

    /// The product, its parts joined again.
    #[inline]
    pub fn get(self) -> P {
        P::of_words(self.obj().words)
    }

    /// Its object's header word.
    pub fn header(self) -> u64 {
        self.obj().header
    }

    /// Its word (C506): Fixpt's, tagged `100`, one past its trailer (a
    /// product has no suffix).
    #[inline(always)]
    pub fn word(self) -> HeapWord {
        HeapWord::object_at(
            self.0
                .as_ptr()
                .cast::<u8>()
                .wrapping_add(std::mem::size_of::<ProdObj<P>>())
                .cast(),
        )
    }

    /// The product a word is to.
    #[inline(always)]
    pub fn of_word(w: HeapWord) -> Self {
        // SAFETY: a product's word is made only by `word`, of its object,
        // kept for the run, its trailer just before where the word points.
        let h = unsafe { w.header_ptr() };
        // SAFETY: as above, its object's start.
        Prod(
            unsafe { NonNull::new_unchecked(h.cast::<ProdObj<P>>().cast_mut()) },
            std::marker::PhantomData,
        )
    }
}

impl<P: crate::Product> ProdObj<P> {
    /// The product it is, its parts joined again.
    #[inline]
    pub fn get(&self) -> P {
        P::of_words(self.words)
    }
}

/// A product's object: its words visited, each its part's.
impl<P: crate::Product> crate::Visit for ProdObj<P> {
    fn visit<'a>(&'a self, t: &mut crate::Tracer<'a>) {
        crate::Trace::trace(P::of_words(self.words), t);
    }
    fn visit_name(&self) -> &'static str {
        std::any::type_name::<Prod<P>>()
    }
}

/// A held product: its object reached; held as its word (C506).
impl<P: crate::Product> crate::Trace for Prod<P> {
    type Stored = HeapWord;
    const VALUE_WORD: bool = true;
    #[inline(always)]
    fn store(self) -> HeapWord {
        self.word()
    }
    #[inline(always)]
    fn load(w: HeapWord) -> Self {
        Prod::of_word(w)
    }
    #[inline]
    fn trace<'a>(self, t: &mut crate::Tracer<'a>)
    where
        Self: 'a,
    {
        t.reach(self.obj());
    }
}

#[cfg(test)]
mod product_tests {
    use crate::{Cons, Held, Str};

    #[derive(Clone, Copy)]
    struct P2<A0, A1>(A0, A1);
    crate::trace_product!(P2, A0 0, A1 1);

    #[test]
    fn a_product_held_one_word_its_object_kind_37() {
        let s: Str = str_lit!("x");
        let inner = P2(3i64, s);
        let p = P2(inner, 'c');
        let h = Held::new(p);
        assert_eq!(std::mem::size_of::<Held<P2<P2<i64, Str>, char>>>(), 8);
        let back = h.get();
        assert_eq!((back.0.0, back.0.1.as_str(), back.1), (3, "x", 'c'));
        // Kind 37, two words (the inner product's, `'c'`'s) and its
        // trailer, Fixpt's count (C498), no suffix (C496).
        let w = crate::Prod::new(p).header();
        assert_eq!(
            (w & 7, (w >> 3) & 0xff, (w >> 14) & 0x3ffff, w >> 32),
            (6, 37, 3, 0)
        );
        assert_eq!(w & (3 << 12), 3 << 12);
        // A pair of products: its element one word, as its tail is.
        assert_eq!(
            std::mem::size_of::<Cons<'static, P2<P2<i64, Str>, char>>>(),
            16
        );
        let xs = crate::heap_cons(p, crate::nil());
        assert_eq!(crate::car(xs).0.0, 3);
        // A ref's cell and an array's element: one word each.
        assert_eq!(
            std::mem::size_of::<crate::Slot<P2<P2<i64, Str>, char>>>(),
            8
        );
        let r = crate::heap_new(p);
        r.set(P2(P2(4, s), 'd'));
        assert_eq!((r.get().0.0, r.get().1), (4, 'd'));
        let a = crate::make_array(crate::int_lit(3), p);
        crate::array_set_x(a, crate::int_lit(1), P2(P2(5, s), 'e'));
        assert_eq!(crate::array_ref(a, crate::int_lit(1)).0.0, 5);
        assert_eq!(a.object_bytes(), 8 * 5);
        // A sum's payload, held.
        let su = crate::sum_of(crate::tag_lit!("pair"), p);
        assert_eq!(crate::sum_payload::<P2<P2<i64, Str>, char>>(su).1, 'c');
    }
}

/// A Fixpt word as a heap object holds a value (C495): a fixnum, an
/// immediate, or a tagged pointer, as Fixpt's heap holds every value
/// (`fixpt-heap`'s `value.rs`); kayak's an address where Fixpt's is an
/// index, its tag in the same three bits. A pointer, not a `u64`, so that
/// a word to an object keeps the object's provenance (as `Int` does).
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct HeapWord(*const u64);

// SAFETY: a word is a value's, and what it points to is never written
// through it (a flonum's, a bignum's, a string's object are frozen).
unsafe impl Send for HeapWord {}
// SAFETY: as above.
unsafe impl Sync for HeapWord {}

// Fixpt's tags, immediates and kinds, from kayak-layout's table (TODO
// §56, step 1).
const TAG_MASK: usize = kayak_layout::TAG_MASK as usize;
const TAG_IMMEDIATE: usize = kayak_layout::tag("immediate") as usize;
const TAG_OBJECT: usize = kayak_layout::tag("bloblet") as usize;
const TAG_PAIR: usize = kayak_layout::tag("pair") as usize;
/// An immediate's subtag, as Fixpt's `Value` makes its word.
const fn subtag(v: kayak_layout::value::Value) -> usize {
    ((v.raw() >> kayak_layout::TAG_BITS) & 0b1_1111) as usize
}
const IMM_NULL: usize = subtag(kayak_layout::value::Value::NULL);
const IMM_FALSE: usize = subtag(kayak_layout::value::Value::FALSE);
const IMM_TRUE: usize = subtag(kayak_layout::value::Value::TRUE);
const IMM_UNIT: usize = subtag(kayak_layout::value::Value::UNIT);
pub(crate) const IMM_CHAR: usize = subtag(kayak_layout::value::Value::char('\0'));
pub(crate) const IMM_F32: usize = subtag(kayak_layout::value::Value::f32(0.0));
const KIND_FLONUM: u8 = kayak_layout::kind("flonum");

impl HeapWord {
    /// Bits that are no pointer: a fixnum's, an immediate's.
    #[inline(always)]
    const fn bits(b: usize) -> HeapWord {
        HeapWord(std::ptr::without_provenance(b))
    }

    /// Its bits.
    #[inline(always)]
    pub fn raw(self) -> usize {
        self.0.addr()
    }

    /// The fixnum `n`, which fits 61 bits.
    #[inline(always)]
    pub const fn fixnum(n: i64) -> HeapWord {
        HeapWord::bits((n << 3) as usize)
    }

    /// Its fixnum, if it is one.
    #[inline(always)]
    pub fn small(self) -> Option<i64> {
        (self.raw() & TAG_MASK == 0).then(|| (self.raw() as i64) >> 3)
    }

    #[inline(always)]
    const fn imm(subtag: usize, payload: usize) -> HeapWord {
        HeapWord::bits(payload << 8 | subtag << 3 | TAG_IMMEDIATE)
    }

    pub const UNIT: HeapWord = HeapWord::imm(IMM_UNIT, 0);
    /// Fixpt's empty list (C497).
    pub const NULL: HeapWord = HeapWord::imm(IMM_NULL, 0);

    #[inline(always)]
    pub const fn boolean(b: bool) -> HeapWord {
        HeapWord::imm(if b { IMM_TRUE } else { IMM_FALSE }, 0)
    }

    #[inline(always)]
    pub fn of_char(c: char) -> HeapWord {
        HeapWord::imm(IMM_CHAR, c as usize)
    }

    #[inline(always)]
    pub fn of_f32(x: f32) -> HeapWord {
        // Its bits in the upper half, as Fixpt's (`Value::f32`).
        HeapWord::bits((x.to_bits() as usize) << 32 | IMM_F32 << 3 | TAG_IMMEDIATE)
    }

    #[inline(always)]
    fn payload(self, subtag: usize, what: &str) -> usize {
        if self.raw() & 0xff != subtag << 3 | TAG_IMMEDIATE {
            crate::fail(&format!("a heap's word read as {what}, which it is not"));
        }
        self.raw() >> 8
    }

    #[inline(always)]
    pub fn to_bool(self) -> bool {
        match self.raw() >> 3 {
            IMM_TRUE => true,
            IMM_FALSE => false,
            _ => crate::fail("a heap's word read as a boolean, which it is not"),
        }
    }

    #[inline(always)]
    pub fn to_char(self) -> char {
        match char::from_u32(self.payload(IMM_CHAR, "a character") as u32) {
            Some(c) => c,
            None => crate::fail("a heap's word read as a character, which it is not"),
        }
    }

    #[inline(always)]
    pub fn to_f32(self) -> f32 {
        f32::from_bits((self.payload(IMM_F32, "an f32") >> 24) as u32)
    }

    /// An `f64`: a flonum, Fixpt's boxed float (kind 5), its bits its
    /// suffix; the word its suffix's address, tagged.
    pub fn flonum(x: f64) -> HeapWord {
        let ws = words(2);
        ws[0] = layout::header(KIND_FLONUM, 0, 8, true, true);
        ws[1] = x.to_bits();
        crate::stats::note::<f64>(16);
        #[cfg(feature = "heap-stats")]
        crate::census::object(ws.as_ptr() as usize, 16, trace_nothing);
        HeapWord(ws[1..].as_ptr().map_addr(|a| a | TAG_OBJECT))
    }

    /// A flonum's `f64`.
    #[inline(always)]
    pub fn to_f64(self) -> f64 {
        if self.raw() & TAG_MASK != TAG_OBJECT {
            crate::fail("a heap's word read as an f64, which it is not");
        }
        // SAFETY: a flonum's word, made by `flonum`: its suffix's one word,
        // kept for the run, never written again.
        f64::from_bits(unsafe { *self.0.map_addr(|a| a & !TAG_MASK) })
    }

    /// An integer's word: a fixnum's own, a bignum's tagged address.
    #[inline(always)]
    pub fn of_int(n: Int) -> HeapWord {
        HeapWord(n.0)
    }

    /// `of_int`, for a `const`.
    #[inline(always)]
    pub const fn const_of_int(n: Int) -> HeapWord {
        HeapWord(n.0)
    }

    /// The integer a word is.
    #[inline(always)]
    pub fn to_int(self) -> Int {
        Int(self.0)
    }

    /// An `i64`: the integer it stands for, a fixnum or (past 60 bits) a
    /// bignum, as Fixpt holds one.
    #[inline(always)]
    pub fn of_i64(n: i64) -> HeapWord {
        HeapWord::of_int(Int::of_i64(n))
    }

    /// The `i64` a word is: an integer of its range.
    #[inline(always)]
    pub fn to_i64(self) -> i64 {
        match self.to_int().view() {
            IntView::Small(n) => n,
            IntView::Big(b) if b.limbs().len() == 1 => {
                let m = b.limbs()[0];
                if b.negative() {
                    0u64.wrapping_sub(m) as i64
                } else {
                    m as i64
                }
            }
            IntView::Big(_) => crate::fail("a heap's word read as an i64, past its range"),
        }
    }

    /// A `u64`: the integer it stands for, as Fixpt holds one.
    #[inline(always)]
    pub fn of_u64(n: u64) -> HeapWord {
        if n <= FIXNUM_MAX as u64 {
            HeapWord::fixnum(n as i64)
        } else {
            HeapWord::of_int(Int::bignum(false, &[n]))
        }
    }

    /// The `u64` a word is.
    #[inline(always)]
    pub fn to_u64(self) -> u64 {
        match self.to_int().view() {
            IntView::Small(n) if n >= 0 => n as u64,
            IntView::Big(b) if !b.negative() && b.limbs().len() == 1 => b.limbs()[0],
            _ => crate::fail("a heap's word read as a u64, past its range"),
        }
    }

    /// A string's word: its text's address (its suffix's), tagged.
    #[inline(always)]
    pub fn of_str(s: Str) -> HeapWord {
        HeapWord(
            s.0.as_ptr()
                .cast_const()
                .wrapping_add(2)
                .map_addr(|a| a | TAG_OBJECT),
        )
    }

    /// A string literal's word, made as rustc builds (a tag's name, C499):
    /// its suffix's address, two words past its start, tagged.
    pub const fn of_str_lit<const W: usize>(s: &'static StrLit<W>) -> HeapWord {
        HeapWord::object_at(std::ptr::from_ref(s).cast::<u64>().wrapping_add(2))
    }

    /// The word of a reference to what is kayak-rt's own, outside the
    /// heap's space (an I-cell, a mailbox, a knot, a place; C508): its
    /// address, of tag `000`, a fixnum's to a collector, which neither
    /// traces nor moves it.
    #[inline(always)]
    pub(crate) fn outside<T>(r: &T) -> HeapWord {
        // One of no bytes (an empty region's) any other: fixnum 0.
        if std::mem::size_of::<T>() == 0 {
            return HeapWord::fixnum(0);
        }
        const {
            assert!(
                std::mem::size_of::<T>() == 0 || std::mem::align_of::<T>() >= 8,
                "a reference held as its address, aligned less than a word"
            )
        };
        let p = std::ptr::from_ref(r).cast::<u64>();
        debug_assert!(
            space::object_of(p.addr()).is_none(),
            "a reference into the heap's space held as an address"
        );
        HeapWord(p)
    }

    /// The reference whose word is `outside`'s.
    #[inline(always)]
    pub(crate) fn to_outside<'r, T>(self) -> &'r T {
        if std::mem::size_of::<T>() == 0 {
            // SAFETY: a reference to a value of no bytes, aligned, not
            // null, is valid wherever it points.
            return unsafe { &*NonNull::<T>::dangling().as_ptr() };
        }
        // SAFETY: a word `outside` made of a `&'r T`, held where a `&'r T`
        // is (`Held<&'r T>`, its type its maker's), so it is that
        // reference again, its referent kayak-rt's, alive for `'r`.
        unsafe { &*self.0.cast::<T>() }
    }

    /// The word to an object whose suffix starts at `at` (8-aligned):
    /// `at`, tagged `100`, its provenance kept.
    #[inline(always)]
    pub(crate) const fn object_at(at: *const u64) -> HeapWord {
        // An 8-aligned address's low three bits are clear: adding the tag
        // sets them, as `|` would, and rustc can do it as it builds.
        HeapWord(at.wrapping_byte_add(TAG_OBJECT))
    }

    /// The header of the object a word points to (C503), as Fixpt finds
    /// it: the word just before where it points, if a header (an object of
    /// no fields: a flonum); else its trailer, which says how far back it is
    /// (Fixpt's distance, in words; a string's trailer's bit 62, whether
    /// its text is ASCII, not of it, C482).
    ///
    /// # Safety
    /// `self` is a word to an object kayak-rt made, of a reference to it
    /// whose provenance covers its header, alive as long as what is read.
    #[inline(always)]
    pub(crate) unsafe fn header_ptr(self) -> *const u64 {
        let at = self.object_ptr();
        // SAFETY: the caller's.
        unsafe {
            let before = *at.sub(1);
            if before & TAG_MASK as u64 == layout::TAG_HEADER {
                return at.sub(1);
            }
            let back = ((before >> 3) & ((1 << 59) - 1)) as usize;
            at.sub(1 + back)
        }
    }

    /// Its tag, Fixpt's three bits (C506).
    #[inline(always)]
    pub(crate) fn tag(self) -> usize {
        self.raw() & TAG_MASK
    }

    /// Whether it is a pair's word.
    #[inline(always)]
    pub(crate) fn is_pair(self) -> bool {
        self.tag() == TAG_PAIR
    }

    /// Whether it is an object's word.
    #[inline(always)]
    pub(crate) fn is_object(self) -> bool {
        self.tag() == TAG_OBJECT
    }

    /// An immediate's subtag (C506), if it is one.
    #[inline(always)]
    pub(crate) fn immediate(self) -> Option<usize> {
        (self.tag() == TAG_IMMEDIATE).then_some((self.raw() >> 3) & 0x1f)
    }

    /// The header of the object a word is to (C506), if it is to one: a
    /// dynamic value's (`Value`), each of whose object words kayak-rt made.
    #[inline(always)]
    pub(crate) fn object_header(self) -> Option<u64> {
        if !self.is_object() {
            return None;
        }
        // SAFETY: an object's word, kayak-rt's (a `Value` holds no other),
        // its object kept for the run.
        Some(unsafe { *self.header_ptr() })
    }

    /// Its object's kind (Fixpt's, its header's bits 3 to 10), if it is an
    /// object's word.
    #[inline(always)]
    pub(crate) fn kind(self) -> Option<u8> {
        self.object_header().map(|h| ((h >> 3) & 0xff) as u8)
    }

    /// The address of the header of the object it is to: a census's key.
    #[inline(always)]
    pub(crate) fn header_addr(self) -> usize {
        // SAFETY: as `object_header`'s.
        unsafe { self.header_ptr() as usize }
    }

    /// A pair's head and tail words (C506).
    #[inline(always)]
    pub(crate) fn pair_words(self) -> (HeapWord, HeapWord) {
        debug_assert!(self.is_pair());
        let p = self.0.map_addr(|a| a & !TAG_MASK).cast::<HeapWord>();
        // SAFETY: a pair's word, made only of a heap's pair cell (`ListAt`'s
        // `of_pair`, `value_cons`), two words, kept for the run; its head
        // written only by `set-car!`'s cell, read as its word.
        unsafe { (*p, *p.add(1)) }
    }

    /// Where the suffix of the object a word points to starts: its tag
    /// cleared, its provenance kept.
    #[inline(always)]
    pub(crate) fn object_ptr(self) -> *const u64 {
        self.0.map_addr(|a| a & !TAG_MASK)
    }

    /// The string a word is.
    #[inline(always)]
    pub fn to_str(self) -> Str {
        let p = self.0.map_addr(|a| a & !TAG_MASK).wrapping_sub(2);
        match NonNull::new(p.cast_mut()) {
            Some(p) => Str(p),
            None => crate::fail("a heap's word read as a string, which it is not"),
        }
    }
}

#[cfg(test)]
mod word_tests {
    use super::*;

    #[test]
    fn values_as_fixpt_words() {
        assert_eq!(HeapWord::fixnum(-3).small(), Some(-3));
        assert_eq!(HeapWord::fixnum(5).raw(), 40);
        assert_eq!(
            HeapWord::boolean(true).raw(),
            kayak_jit::layout::TRUE as usize
        );
        assert_eq!(
            HeapWord::boolean(false).raw(),
            kayak_jit::layout::FALSE as usize
        );
        assert_eq!(HeapWord::UNIT.raw(), kayak_jit::layout::UNIT as usize);
        assert!(HeapWord::boolean(true).to_bool() && !HeapWord::boolean(false).to_bool());
        let c = HeapWord::of_char('λ');
        assert_eq!(c.raw() & 0xff, (8 << 3) | 3);
        assert_eq!(c.to_char(), 'λ');
        assert_eq!(HeapWord::of_f32(1.5).to_f32(), 1.5);
        let f = HeapWord::flonum(2.25);
        assert_eq!(f.raw() & 7, 4);
        assert_eq!(f.to_f64(), 2.25);
        let s: Str = crate::str_lit("abc");
        assert_eq!(HeapWord::of_str(s).raw() & 7, 4);
        assert_eq!(HeapWord::of_str(s).to_str().as_str(), "abc");
        let big = Int::of_i64(i64::MAX);
        assert_eq!(HeapWord::of_int(big).to_int().small(), None);
        assert_eq!(HeapWord::of_int(Int::of_i64(7)).raw(), 56);
    }
}

/// `(listof T R)` (C497): one Fixpt word, on the stack as in the heap:
/// Fixpt's empty list (an immediate), or a pair's address tagged `001`, its
/// pair Fixpt's headerless two-word cell (`Cons`: its car's word, its cdr's).
/// In a `letrena`'s region `ListAt<'r, T>`, in the heap `List<T>`. Its word
/// made only here, of a pair's handle or the empty list.
#[repr(transparent)]
pub struct ListAt<'r, T: crate::Trace>(
    HeapWord,
    std::marker::PhantomData<crate::Handle<'r, crate::Cons<'r, T>>>,
);

/// A pair of a dynamic value (C506), its head `car`'s word, its tail any
/// value's word (a list's, or not: `(1 . 2)`): Fixpt's pair cell, as a
/// typed list's is, in the heap's arena of them.
pub(crate) fn value_cons(car: crate::value::Value, cdr: HeapWord) -> HeapWord {
    crate::heap_cons(car, ListAt(cdr, std::marker::PhantomData)).0
}

impl<T: crate::Trace> Clone for ListAt<'_, T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: crate::Trace> Copy for ListAt<'_, T> {}

impl<'r, T: crate::Trace> ListAt<'r, T> {
    /// The empty list.
    pub const NIL: Self = ListAt(HeapWord::NULL, std::marker::PhantomData);

    /// The list whose first pair is `p`.
    #[inline(always)]
    pub fn of_pair(p: crate::Handle<'r, crate::Cons<'r, T>>) -> Self {
        // A pair is word-aligned (its cdr a word), so its tag's bits free.
        let a = std::ptr::from_ref(p.to_ref()).cast::<u64>();
        ListAt(
            HeapWord(a.map_addr(|x| x | TAG_PAIR)),
            std::marker::PhantomData,
        )
    }

    /// The empty list, or the list whose first pair is `p`.
    #[inline(always)]
    pub fn of(p: Option<crate::Handle<'r, crate::Cons<'r, T>>>) -> Self {
        p.map_or(Self::NIL, Self::of_pair)
    }

    /// Its first pair, if it is not empty.
    #[inline(always)]
    pub fn pair(self) -> Option<crate::Handle<'r, crate::Cons<'r, T>>> {
        if self.0.raw() & TAG_MASK == TAG_PAIR {
            let a = self.0.0.map_addr(|x| x & !TAG_MASK);
            // SAFETY: a word tagged a pair is made only by `of_pair`, of a
            // pair's handle good for `'r`.
            Some(crate::Handle::made(unsafe {
                &*a.cast::<crate::Cons<'r, T>>()
            }))
        } else {
            None
        }
    }

    /// Whether it is empty.
    #[inline(always)]
    pub fn is_nil(self) -> bool {
        self.0.raw() == HeapWord::NULL.raw()
    }

    /// Its word.
    #[inline(always)]
    pub fn word(self) -> HeapWord {
        self.0
    }
}

/// A list: its first pair reached; held as its word.
impl<T: crate::Trace> crate::Trace for ListAt<'_, T> {
    #[inline]
    fn trace<'a>(self, t: &mut crate::Tracer<'a>)
    where
        Self: 'a,
    {
        if let Some(p) = self.pair() {
            t.reach(p.to_ref());
        }
    }
    type Stored = HeapWord;
    const VALUE_WORD: bool = T::VALUE_WORD;
    #[inline(always)]
    fn store(self) -> HeapWord {
        self.0
    }
    #[inline(always)]
    fn load(w: HeapWord) -> Self {
        ListAt(w, std::marker::PhantomData)
    }
}

#[cfg(test)]
mod list_tests {
    use crate::{Cons, HeapWord, List, Trace};

    #[test]
    fn a_list_one_fixpt_word() {
        // The empty list Fixpt's immediate (subtag 2); a pair's word its
        // address tagged `001`; the pair two words, its car's and cdr's.
        let nil: List<i64> = crate::nil();
        assert_eq!(nil.word().raw(), 2 << 3 | 0b011);
        assert!(crate::null_p(nil) && nil.pair().is_none());
        let xs = crate::heap_cons(5i64, crate::heap_cons(6i64, nil));
        assert_eq!(xs.word().raw() & 7, 0b001);
        assert_eq!(std::mem::size_of::<List<i64>>(), 8);
        assert_eq!(std::mem::size_of::<Cons<'static, i64>>(), 16);
        assert_eq!((crate::car(xs), crate::car(crate::cdr(xs))), (5, 6));
        assert!(crate::null_p(crate::cdr(crate::cdr(xs))));
        // Held as its word, as a pair's cdr is.
        let w: HeapWord = xs.store();
        assert_eq!(crate::car(List::<i64>::load(w)), 5);
    }
}

/// A box's kind (Fixpt's `box`, 10): its one field traced and settable.
const KIND_BOX: u8 = kayak_layout::kind("box");

/// A reference's object (C498), Fixpt's box: its header, its value's cell
/// (one word, but what is held as itself as yet), its trailer, no suffix.
#[repr(C)]
pub struct RefObj<T: crate::Trace> {
    header: u64,
    cell: crate::Slot<T>,
    trailer: u64,
}

impl<T: crate::Trace> RefObj<T> {
    /// Its fields as Fixpt counts them: its cell's words and its trailer.
    const TO_TRAILER: usize = std::mem::offset_of!(RefObj<T>, trailer) / 8;
    /// From its start to its suffix's, where its word points.
    const TO_SUFFIX: usize = std::mem::offset_of!(RefObj<T>, trailer) + 8;

    /// A box holding `v`.
    #[inline]
    pub fn new(v: T) -> Self {
        RefObj {
            header: layout::header(KIND_BOX, Self::TO_TRAILER, 0, false, false),
            cell: crate::Slot::new(v),
            trailer: layout::trailer(Self::TO_TRAILER),
        }
    }

    /// Its header word.
    pub fn header(&self) -> u64 {
        self.header
    }
}

/// A box: its value visited.
impl<T: crate::Trace> crate::Visit for RefObj<T> {
    fn visit<'a>(&'a self, t: &mut crate::Tracer<'a>) {
        crate::Visit::visit(&self.cell, t);
    }
}

/// `(ref T R)` (C498): one Fixpt word, to a box (`RefObj`), tagged `100`
/// at its suffix's start, as Fixpt's word to an object is; which any
/// alias may read and write, as FX-26's may. In a `letrena`'s region
/// `RefAt<'r, T>`, borrowed from its arena; in the heap, never freed,
/// `Ref<T>`. Its word made only here, of a box's reference.
#[repr(transparent)]
pub struct RefAt<'r, T: crate::Trace>(HeapWord, std::marker::PhantomData<&'r RefObj<T>>);

impl<T: crate::Trace> Clone for RefAt<'_, T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: crate::Trace> Copy for RefAt<'_, T> {}

impl<'r, T: crate::Trace> RefAt<'r, T> {
    /// The reference to box `o`.
    #[inline(always)]
    pub fn of_obj(o: crate::Handle<'r, RefObj<T>>) -> Self {
        let a = std::ptr::from_ref(o.to_ref()).cast::<u8>();
        // SAFETY: its suffix's start, one past its trailer, the box's end.
        let at = unsafe { a.add(RefObj::<T>::TO_SUFFIX) }.cast::<u64>();
        RefAt(
            HeapWord(at.map_addr(|x| x | TAG_OBJECT)),
            std::marker::PhantomData,
        )
    }

    /// Its box.
    #[inline(always)]
    pub fn obj(self) -> &'r RefObj<T> {
        let at = self.0.0.map_addr(|x| x & !TAG_MASK).cast::<u8>();
        // SAFETY: its word is made only by `of_obj`, of a box good for `'r`,
        // `TO_SUFFIX` bytes before where it points.
        unsafe { &*at.sub(RefObj::<T>::TO_SUFFIX).cast::<RefObj<T>>() }
    }

    #[inline(always)]
    pub fn get(self) -> T {
        self.obj().cell.get()
    }

    #[inline(always)]
    pub fn set(self, v: T) {
        self.obj().cell.set(v)
    }

    /// Whether it is the same reference as `other` (`eq?`).
    #[inline(always)]
    pub fn same(self, other: Self) -> bool {
        self.0.raw() == other.0.raw()
    }

    /// Its word.
    #[inline(always)]
    pub fn word(self) -> HeapWord {
        self.0
    }
}

/// A reference: its box reached; held as its word.
impl<T: crate::Trace> crate::Trace for RefAt<'_, T> {
    #[inline]
    fn trace<'a>(self, t: &mut crate::Tracer<'a>)
    where
        Self: 'a,
    {
        t.reach(self.obj());
    }
    type Stored = HeapWord;
    const VALUE_WORD: bool = true;
    #[inline(always)]
    fn store(self) -> HeapWord {
        self.0
    }
    #[inline(always)]
    fn load(w: HeapWord) -> Self {
        RefAt(w, std::marker::PhantomData)
    }
}

/// A knot (`letrec`'s, `knot`): a box of the space (Fixpt's, kind 10),
/// its field the procedure's word once tied, fixnum 0 before (C511).
pub struct Knot<'a, T: crate::Trace<Stored = HeapWord>>(RefAt<'a, T>);

impl<T: crate::Trace<Stored = HeapWord>> Clone for Knot<'_, T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: crate::Trace<Stored = HeapWord>> Copy for Knot<'_, T> {}

// SAFETY: tied once, by the thread that made it, before any closure that
// holds it is made, so before it may cross (C102); read only after.
unsafe impl<T: crate::Trace<Stored = HeapWord> + Send + Sync> Send for Knot<'_, T> {}
// SAFETY: as above.
unsafe impl<T: crate::Trace<Stored = HeapWord> + Send + Sync> Sync for Knot<'_, T> {}

impl<T: crate::Trace<Stored = HeapWord>> Knot<'_, T> {
    #[inline]
    pub fn get(&self) -> T {
        if self.0.obj().cell.stored().raw() == 0 {
            crate::fail("letrec: a procedure used before it is made")
        }
        self.0.get()
    }

    #[inline]
    pub fn set(&self, v: T) {
        if self.0.obj().cell.stored().raw() != 0 {
            crate::fail("letrec: a procedure made twice")
        }
        self.0.set(v)
    }
}

/// A knot: its box reached; held as its word.
impl<T: crate::Trace<Stored = HeapWord>> crate::Trace for Knot<'_, T> {
    #[inline]
    fn trace<'a>(self, t: &mut crate::Tracer<'a>)
    where
        Self: 'a,
    {
        self.0.trace(t)
    }
    type Stored = HeapWord;
    const VALUE_WORD: bool = true;
    #[inline(always)]
    fn store(self) -> HeapWord {
        self.0.word()
    }
    #[inline(always)]
    fn load(w: HeapWord) -> Self {
        Knot(RefAt::load(w))
    }
}

/// A knot, untied.
#[inline]
pub fn knot<'a, T: crate::Trace<Stored = HeapWord> + 'a>() -> Knot<'a, T> {
    let o = space::put(RefObj {
        header: layout::header(KIND_BOX, RefObj::<T>::TO_TRAILER, 0, false, false),
        cell: crate::Slot::of_stored(HeapWord::fixnum(0)),
        trailer: layout::trailer(RefObj::<T>::TO_TRAILER),
    });
    Knot(RefAt::of_obj(crate::Handle::made(o)))
}

// A reference as a union's member (C491): its box's word, kind 10 (C506).
crate::word_member!([T: crate::Trace + 'static] RefAt<'static, T>; 10);

#[cfg(test)]
mod ref_tests {
    use crate::{HeapWord, Ref, Trace};

    #[test]
    fn a_ref_one_word_to_fixpt_box() {
        let r: Ref<i64> = crate::heap_new(5);
        assert_eq!(std::mem::size_of::<Ref<i64>>(), 8);
        // Tagged `100`, at its box's suffix: its header three words back,
        // kind 10, two fields (its value's, its trailer), not frozen.
        assert_eq!(r.word().raw() & 7, 0b100);
        let h = r.obj().header();
        assert_eq!(
            (
                h & 7,
                (h >> 3) & 0xff,
                (h >> 12) & 3,
                (h >> 14) & 0x3ffff,
                h >> 32
            ),
            (6, 10, 0, 2, 0)
        );
        r.set(6);
        assert_eq!(crate::get(r), 6);
        let w: HeapWord = r.store();
        let s = Ref::<i64>::load(w);
        assert!(s.same(r) && !s.same(crate::heap_new(6)));
        // In a pair, as its car: its word.
        let xs = crate::heap_cons(r, crate::nil());
        assert_eq!(crate::car(xs).get(), 6);
    }
}

/// The `n` words before where `w` points (its object's, back from its
/// suffix), first the farthest: for a test to read what it made.
#[cfg(test)]
pub(crate) fn words_before(w: HeapWord, n: usize) -> &'static [u64] {
    // SAFETY: a test's word, to an object kept for the run, of at least
    // `n` words before its suffix; read through the word's provenance.
    unsafe { std::slice::from_raw_parts(w.object_ptr().sub(n), n) }
}

/// Procedure values (C501): each one Fixpt word, to Fixpt's native
/// closure (kind 41, frozen): `[free…][code][trailer]`, free value `i` at
/// slot `CLOSURE_FREE0 + i` (counted back from the trailer, which is slot
/// 1), its code at `CLOSURE_WORD`, a word to its code's object. Fixpt's
/// is a code bloblet of machine code; kayak's, a frozen bloblet of its
/// own, one of each closure's type, made as rustc builds, its suffix the
/// type's functions: its call and its trace ([`ProcCode`]). Its Rust
/// closure, of no bytes (it captures nothing, checked when built), its
/// type's.
pub mod procs {
    use super::{HeapWord, layout};
    use crate::trace::{Free, Trace, Tracer, Visit};

    /// Fixpt's native closure's kind.
    const KIND_CLOSURE: u8 = layout::KIND_NATIVE_CLOSURE;

    /// A closure's type's code (C501): a frozen bloblet, no fields, its
    /// suffix two functions, raw: its call, of its arity (erased, given
    /// its type again by the procedure's type, `ProcN`, the only reader),
    /// and what it reaches.
    #[repr(C)]
    pub struct ProcCode {
        header: u64,
        trailer: u64,
        call: *const (),
        reach: unsafe fn(*const u64) -> *const (dyn Visit + 'static),
    }

    impl ProcCode {
        const fn new(
            call: *const (),
            reach: unsafe fn(*const u64) -> *const (dyn Visit + 'static),
        ) -> ProcCode {
            ProcCode {
                header: layout::header(layout::KIND_BLOBLET, 1, 16, true, true),
                trailer: layout::trailer(1),
                call,
                reach,
            }
        }
    }

    /// A closure's object: Fixpt's native closure.
    #[repr(C)]
    pub struct ClosureObj<E: Free, C> {
        header: u64,
        words: E::Words,
        code: HeapWord,
        trailer: u64,
        /// Its code, of no bytes, its type the object's.
        run: C,
    }

    impl<E: Free, C> ClosureObj<E, C> {
        /// Its fields as Fixpt counts them, its trailer one.
        const TO_TRAILER: usize = std::mem::offset_of!(ClosureObj<E, C>, trailer) / 8;
        /// From its start to its suffix's, where its word points.
        const TO_SUFFIX: usize = std::mem::offset_of!(ClosureObj<E, C>, trailer) + 8;

        /// The closure whose suffix starts at `p`.
        ///
        /// # Safety
        /// `p` is a word to a `ClosureObj<E, C>` (its code is this type's,
        /// which only this type's objects are made with), alive for `'r`.
        #[inline(always)]
        unsafe fn at<'r>(p: *const u64) -> &'r Self {
            // SAFETY: the caller's.
            unsafe { &*p.cast::<u8>().sub(Self::TO_SUFFIX).cast::<Self>() }
        }
    }

    /// A closure: its free values traced.
    impl<E: Free, C> Visit for ClosureObj<E, C> {
        fn visit<'a>(&'a self, t: &mut Tracer<'a>) {
            E::of_words(self.words).trace(t);
        }
    }

    /// What a closure of type `ClosureObj<E, C>` reaches: it, as a
    /// `Visit`, its lifetime the caller's to know.
    ///
    /// # Safety
    /// `p` is a word to a `ClosureObj<E, C>`.
    unsafe fn reach_of<'r, E: Free + 'r, C: 'r>(p: *const u64) -> *const (dyn Visit + 'static) {
        // SAFETY: the caller's.
        let o: &'r (dyn Visit + 'r) = unsafe { ClosureObj::<E, C>::at(p) };
        // SAFETY: only the bound of its lifetime changed; the procedure
        // that reads it (`trace`) is of `'r`, and gives it a tracer's.
        unsafe { std::mem::transmute::<*const (dyn Visit + 'r), *const (dyn Visit + 'static)>(o) }
    }

    /// What the closure a word is to reaches (C506), by its code's `reach`,
    /// whatever its type: a procedure's, or a dynamic value's.
    #[inline]
    pub(crate) fn reach_closure<'a>(w: HeapWord, t: &mut Tracer<'a>) {
        let p = w.object_ptr();
        // SAFETY: a closure's word (`made`'s), its code at slot 2, a word to
        // its type's `ProcCode`, whose `reach` is its type's, of a closure
        // kept for the run.
        let o: &'a (dyn Visit + 'a) = unsafe {
            let code = *p.sub(2).cast::<HeapWord>();
            let reach = *code
                .object_ptr()
                .add(1)
                .cast::<unsafe fn(*const u64) -> *const (dyn Visit + 'static)>();
            &*reach(p)
        };
        t.reach(o);
    }

    /// A closure of no bytes of its own (it captures nothing, so any one of
    /// its type is any other), kept for the run, noted for a census.
    fn kept<'r, T: Visit + 'r>(v: T) -> &'r T {
        crate::stats::note::<T>(std::mem::size_of::<T>());
        let r: &'r T = super::space::put(v);
        #[cfg(feature = "heap-stats")]
        crate::census::one(r);
        r
    }

    /// The word to a closure just made, of free values `env`, its code
    /// `run` and its type's `code`.
    #[inline]
    fn made<'r, E: Free + 'r, C: 'r>(env: E, run: C, code: &'static ProcCode) -> HeapWord {
        const { assert!(std::mem::size_of::<C>() == 0, "a closure's code captures") };
        let o: &'r ClosureObj<E, C> = kept(ClosureObj {
            header: layout::header(KIND_CLOSURE, ClosureObj::<E, C>::TO_TRAILER, 0, true, true),
            words: env.words(),
            code: HeapWord::object_at(std::ptr::from_ref(code).cast::<u64>().wrapping_add(2)),
            trailer: layout::trailer(ClosureObj::<E, C>::TO_TRAILER),
            run,
        });
        HeapWord::object_at(
            std::ptr::from_ref(o)
                .cast::<u8>()
                .wrapping_add(ClosureObj::<E, C>::TO_SUFFIX)
                .cast(),
        )
    }

    macro_rules! arities {
        ($($proc:ident $procs:ident $make:ident $makes:ident $code:ident $run:ident ($($a:ident)*);)*) => {$(
            /// The call of a closure of type `ClosureObj<E, Code>`, of this
            /// arity.
            ///
            /// # Safety
            /// `p` is a word to a `ClosureObj<E, Code>`.
            #[allow(non_snake_case, clippy::too_many_arguments)]
            unsafe fn $run<E: Free, Code: Fn(E, $($a),*) -> R, $($a,)* R>(p: *const u64, $($a: $a),*) -> R {
                // SAFETY: the caller's.
                let o = unsafe { ClosureObj::<E, Code>::at(p) };
                (o.run)(E::of_words(o.words), $($a),*)
            }

            /// A closure's type's code, of this arity.
            trait $code<$($a,)* R> {
                const CODE: &'static ProcCode;
            }

            impl<E: Free, Code: Fn(E, $($a),*) -> R, $($a,)* R> $code<$($a,)* R> for ClosureObj<E, Code> {
                const CODE: &'static ProcCode = &ProcCode::new(
                    $run::<E, Code, $($a,)* R> as *const (),
                    reach_of::<E, Code>,
                );
            }

            /// A procedure value of this arity: one Fixpt word, to its
            /// closure, tagged `100` at its suffix.
            #[repr(transparent)]
            pub struct $proc<'r, $($a,)* R>(HeapWord, std::marker::PhantomData<(&'r (), fn($($a),*) -> R, *const ())>);

            /// One of an effect a process may have (C102): `Send + Sync`,
            /// its free values all.
            #[repr(transparent)]
            pub struct $procs<'r, $($a,)* R>(HeapWord, std::marker::PhantomData<(&'r (), fn($($a),*) -> R)>);

            impl<$($a,)* R> Clone for $proc<'_, $($a,)* R> {
                #[inline(always)]
                fn clone(&self) -> Self {
                    *self
                }
            }
            impl<$($a,)* R> Copy for $proc<'_, $($a,)* R> {}
            impl<$($a,)* R> Clone for $procs<'_, $($a,)* R> {
                #[inline(always)]
                fn clone(&self) -> Self {
                    *self
                }
            }
            impl<$($a,)* R> Copy for $procs<'_, $($a,)* R> {}

            impl<'r, $($a,)* R> $proc<'r, $($a,)* R> {
                /// Called: its code's call, given its closure.
                #[allow(non_snake_case, clippy::too_many_arguments)]
                #[inline]
                pub fn call(&self, $($a: $a),*) -> R {
                    let p = self.0.object_ptr();
                    // SAFETY: its word is made only of a closure (`$make`,
                    // `$makes`), its code at slot 2, a word to its type's
                    // `ProcCode`, whose call is `$run` of that type, of
                    // this arity and these arguments' types (the
                    // procedure's own, `$make`'s), erased to be kept.
                    unsafe {
                        let code = *p.sub(2).cast::<HeapWord>();
                        let call = *code.object_ptr().cast::<*const ()>();
                        let call: unsafe fn(*const u64, $($a),*) -> R = std::mem::transmute(call);
                        call(p, $($a),*)
                    }
                }
            }

            impl<'r, $($a,)* R> $procs<'r, $($a,)* R> {
                /// Called.
                #[allow(non_snake_case, clippy::too_many_arguments)]
                #[inline]
                pub fn call(&self, $($a: $a),*) -> R {
                    self.unsent().call($($a),*)
                }

                /// The same procedure, as one that may not be `Send`.
                #[inline(always)]
                pub fn unsent(self) -> $proc<'r, $($a,)* R> {
                    $proc(self.0, std::marker::PhantomData)
                }
            }

            /// A procedure: its closure reached.
            impl<'r, $($a,)* R> Trace for $proc<'r, $($a,)* R> {
                type Stored = HeapWord;
    const VALUE_WORD: bool = true;
                #[inline(always)]
                fn store(self) -> HeapWord {
                    self.0
                }
                #[inline(always)]
                fn load(w: HeapWord) -> Self {
                    $proc(w, std::marker::PhantomData)
                }
                #[inline]
                fn trace<'a>(self, t: &mut Tracer<'a>) where Self: 'a {
                    reach_closure(self.0, t);
                }
            }

            impl<'r, $($a,)* R> Trace for $procs<'r, $($a,)* R> {
                type Stored = HeapWord;
    const VALUE_WORD: bool = true;
                #[inline(always)]
                fn store(self) -> HeapWord {
                    self.0
                }
                #[inline(always)]
                fn load(w: HeapWord) -> Self {
                    $procs(w, std::marker::PhantomData)
                }
                #[inline]
                fn trace<'a>(self, t: &mut Tracer<'a>) where Self: 'a {
                    self.unsent().trace(t);
                }
            }

            // A procedure value as a union's member (C491): its closure's
            // word, kind 41 (C506).
            crate::word_member!([$($a: 'static,)* R: 'static] $proc<'static, $($a,)* R>; 41);
            crate::word_member!([$($a: 'static,)* R: 'static] $procs<'static, $($a,)* R>; 41);

            /// A closure of free values `env` and code `code`: a Rust
            /// closure that captures nothing (C493, checked when built).
            #[inline]
            pub fn $make<'r, E: Free + 'r, Code: Fn(E, $($a),*) -> R + 'r, $($a: 'r,)* R: 'r>(env: E, code: Code) -> $proc<'r, $($a,)* R> {
                $proc(made(env, code, <ClosureObj<E, Code> as $code<$($a,)* R>>::CODE), std::marker::PhantomData)
            }

            /// A closure of free values `env`, all `Send + Sync`, and code
            /// `code`.
            #[inline]
            pub fn $makes<'r, E: Free + Send + Sync + 'r, Code: Fn(E, $($a),*) -> R + Send + Sync + 'r, $($a: 'r,)* R: 'r>(env: E, code: Code) -> $procs<'r, $($a,)* R> {
                $procs(made(env, code, <ClosureObj<E, Code> as $code<$($a,)* R>>::CODE), std::marker::PhantomData)
            }
        )*};
    }

    arities! {
        Proc0 Proc0s closure0 closure0s Coded0 run0 ();
        Proc1 Proc1s closure1 closure1s Coded1 run1 (A);
        Proc2 Proc2s closure2 closure2s Coded2 run2 (A B);
        Proc3 Proc3s closure3 closure3s Coded3 run3 (A B C);
        Proc4 Proc4s closure4 closure4s Coded4 run4 (A B C D);
        Proc5 Proc5s closure5 closure5s Coded5 run5 (A B C D F);
        Proc6 Proc6s closure6 closure6s Coded6 run6 (A B C D F G);
    }
}

/// An object read by its layout alone (C508; `TODO.md` §54, a collector's
/// trace): what a precise trace of the heap needs, no Rust type. Every
/// value a heap object holds is a Fixpt word (`Trace::WHOLE`), so its
/// fields are found by its header, as Fixpt's collector finds them.
#[cfg_attr(not(any(test, feature = "heap-stats")), allow(dead_code))]
pub(crate) mod laid {
    use super::*;

    /// An object at its first word: its identity (its main header's
    /// address, or a pair's first word), the words that may hold values,
    /// and its end.
    pub(crate) struct Laid {
        pub pair: bool,
        /// What a collector forwards (`TODO.md` §54's step 4).
        #[allow(dead_code)]
        pub id: usize,
        pub fields: &'static [HeapWord],
        /// Where its suffix starts, where an object's word points (0 for
        /// a pair).
        pub suffix: usize,
        pub end: usize,
    }

    /// The kinds whose fields hold no value (Fixpt's untraced kinds).
    fn untraced(kind: u64) -> bool {
        matches!(kind, 1 | 4 | 5 | 6)
    }

    /// The object whose first word is `at`.
    ///
    /// # Safety
    /// `at` is the first word of an object kayak-rt made, kept for the run.
    pub(crate) unsafe fn at(at: *const u64) -> Laid {
        // SAFETY: the caller's: its words are the object's, as its header
        // says them.
        unsafe {
            let w = *at;
            if w & TAG_MASK as u64 != layout::TAG_HEADER {
                // A pair: two words, each a value.
                return Laid {
                    pair: true,
                    id: at.addr(),
                    fields: std::slice::from_raw_parts(at.cast::<HeapWord>(), 2),
                    suffix: 0,
                    end: at.addr() + 16,
                };
            }
            let (main, fields) = if (w >> 3) & 0xff == KIND_EXTENSION {
                (at.add(1), (w >> 11) as usize)
            } else {
                (at, ((w >> 14) & ((1 << 18) - 1)) as usize)
            };
            let h = *main;
            let kind = (h >> 3) & 0xff;
            let bytes = (h >> 32) as usize;
            // A string's and a bytevector's trailer, which their header's
            // count leaves out (C482, C483; Fixpt counts it).
            let extra = usize::from(
                fields == 0 && matches!(kind, 1 | 4) && *main.add(1) & TAG_MASK as u64 == 0b101,
            );
            let n = if untraced(kind) { 0 } else { fields };
            Laid {
                pair: false,
                id: main.addr(),
                fields: std::slice::from_raw_parts(main.add(1).cast::<HeapWord>(), n),
                suffix: main.addr() + 8 * (1 + fields + extra),
                end: main.addr() + 8 * (1 + fields + extra + bytes.div_ceil(8)),
            }
        }
    }

    /// The first word of what `w` points to, if it is a pair's or an
    /// object's word into the heap's space: a pair's first word, an
    /// object's main header (its extension word before it not counted).
    pub(crate) fn target(w: HeapWord) -> Option<usize> {
        let t = if w.is_pair() {
            w.raw() & !TAG_MASK
        } else if w.is_object() {
            let at = w.raw() & !TAG_MASK;
            // Only a word into the space is read: another's object (a
            // literal's, kayak-rt's own) is no collector's.
            space::object_of(at.wrapping_sub(8))?;
            // SAFETY: an object's word into the space, kayak-rt's, its
            // object kept for the run.
            unsafe { w.header_ptr() }.addr()
        } else {
            return None;
        };
        space::object_of(t).map(|_| t)
    }

    /// The objects of the allocation whose first word is `start`: one, or
    /// a datatype's value's two (its product, then its sum, C500), each
    /// after the last while a header follows.
    ///
    /// # Safety
    /// `start` is an allocation's first word in the space (`object_of`'s).
    pub(crate) unsafe fn allocation(start: usize) -> Vec<Laid> {
        let mut out = Vec::new();
        let mut at = start;
        loop {
            // SAFETY: the caller's, and then the next object of the same
            // allocation, which a header begins.
            let o = unsafe { self::at(at as *const u64) };
            let (end, pair) = (o.end, o.pair);
            out.push(o);
            if pair || space::object_of(end) != Some(start) {
                break;
            }
            // SAFETY: a word of the same allocation, past its last object.
            if unsafe { *(end as *const u64) } & TAG_MASK as u64 != layout::TAG_HEADER {
                break;
            }
            at = end;
        }
        out
    }
}

#[cfg(test)]
mod laid_tests {
    use super::*;
    use std::collections::BTreeSet;

    /// The allocations a word reaches, by layout alone.
    fn reached(w: HeapWord) -> BTreeSet<usize> {
        let mut seen = BTreeSet::new();
        let mut work: Vec<usize> = laid::target(w)
            .and_then(space::object_of)
            .into_iter()
            .collect();
        while let Some(a) = work.pop() {
            if seen.insert(a) {
                // SAFETY: an allocation's first word in the space.
                for o in unsafe { laid::allocation(a) } {
                    work.extend(
                        o.fields
                            .iter()
                            .filter_map(|&f| laid::target(f).and_then(space::object_of)),
                    );
                }
            }
        }
        seen
    }

    #[test]
    fn a_heap_traced_by_its_layout_alone() {
        // A list of an array (of a string and a bignum's list), a pair of
        // strings, a box, and a string; each object's allocation reached.
        let s = crate::leak_string("a string".to_string());
        let t = crate::leak_string("another".to_string());
        let big = Int::of_i64(i64::MAX);
        let bigs = crate::heap_cons(big, crate::nil());
        let a = crate::list_to_array(crate::heap_cons(s, crate::nil()));
        let b = crate::list_to_array(crate::heap_cons(bigs, crate::nil()));
        let r = crate::heap_new(t);
        let outer = crate::heap_cons(a, crate::nil());
        let rb = crate::heap_new(b);
        use crate::Trace;
        // Each word's allocation: its object's, found as a trace finds it
        // (an array's word is one past its trailer, its object no suffix).
        let at = |w: HeapWord| {
            laid::target(w)
                .and_then(space::object_of)
                .expect("in the space")
        };
        let set = |ws: &[HeapWord]| ws.iter().map(|&w| at(w)).collect::<BTreeSet<usize>>();
        assert_eq!(
            reached(outer.store()),
            set(&[outer.store(), a.word(), HeapWord::of_str(s)])
        );
        assert_eq!(
            reached(rb.store()),
            set(&[rb.store(), b.word(), bigs.store(), big.store()])
        );
        // A box and its string: kind 10, then kind 1.
        assert_eq!(reached(r.store()).len(), 2);
    }
}

#[cfg(test)]
mod layout_tests {
    use super::*;

    /// The constants drawn from Fixpt's table (TODO §56, step 1) are those
    /// this crate wrote out before.
    #[test]
    fn the_table_gives_the_values_written_before() {
        assert_eq!(
            (TAG_MASK, TAG_IMMEDIATE, TAG_OBJECT, TAG_PAIR),
            (7, 3, 4, 1)
        );
        assert_eq!(
            (IMM_FALSE, IMM_TRUE, IMM_NULL, IMM_UNIT, IMM_CHAR, IMM_F32),
            (0, 1, 2, 3, 8, 9)
        );
        assert_eq!(
            (
                KIND_FLONUM,
                KIND_BIGNUM,
                KIND_PRODUCT,
                KIND_BOX,
                KIND_EXTENSION
            ),
            (5, 6, 37, 10, 255)
        );
        assert_eq!(kayak_layout::kind("bytevector"), 4);
        assert_eq!(
            crate::value::tests_kinds(),
            (1, 2, 3, 4, 5, 6, 10, 32, 36, 37, 41, 42)
        );
    }
}
