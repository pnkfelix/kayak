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

const KIND_BIGNUM: u8 = 6;

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

/// A bloblet: its object reached (C486).
impl<Fs: crate::Visit> crate::Trace for BlobletAt<'_, Fs> {
    crate::trace_whole!();
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
                header: layout::header(layout::KIND_BLOBLET, to_trailer - 1, bytes, false, frozen),
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

    /// It, kept for the run (the heap's: never freed).
    pub(crate) fn kept<'a>(self) -> BlobletAt<'a, Fs>
    where
        Fs: crate::Visit + 'static,
    {
        let at = self.at;
        #[cfg(feature = "heap-stats")]
        crate::census::object(
            at.as_ptr() as usize,
            self.layout.size(),
            trace_bloblet::<Fs>,
        );
        crate::stats::note::<Bloblet<Fs>>(self.layout.size());
        std::mem::forget(self);
        BlobletAt(at, std::marker::PhantomData)
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
        let b = Bloblet::new((Cell::new(7i64), Cell::new('x')), 12, false);
        let at = b.at();
        assert_eq!(at.fields().0.get(), 7);
        at.fields().1.set('y');
        assert_eq!(at.bytes(), 12);
        assert_eq!(at.suffix().len(), 2);
        at.suffix()[1].set(5);
        assert_eq!(at.suffix()[1].get(), 5);
        assert_eq!(at.fields().1.get(), 'y');
        assert!(!at.suffix_frozen());
        assert_eq!(std::mem::size_of::<BlobletAt<'_, (Cell<i64>,)>>(), 8);
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
                to_trailer - 1,
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

    /// It, kept for the run (the heap's: never freed).
    pub(crate) fn kept<'a>(self) -> TBlobletAt<'a, Fs, S>
    where
        Fs: crate::Visit + 'static,
        S: 'static,
    {
        let at = self.at;
        let size = std::mem::size_of::<TBlobHead<Fs, S>>();
        #[cfg(feature = "heap-stats")]
        crate::census::object(at.as_ptr() as usize, size, trace_tbloblet::<Fs, S>);
        crate::stats::note::<TBloblet<Fs, S>>(size);
        std::mem::forget(self);
        TBlobletAt(at, std::marker::PhantomData)
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
impl<Fs: crate::Visit, S> crate::Visit for TBlobHead<Fs, S> {
    fn visit<'a>(&'a self, t: &mut crate::Tracer<'a>) {
        self.fields.visit(t);
    }
    fn visit_name(&self) -> &'static str {
        std::any::type_name::<TBloblet<Fs, S>>()
    }
}

/// A typed-suffix bloblet: its object reached (C487).
impl<Fs: crate::Visit, S> crate::Trace for TBlobletAt<'_, Fs, S> {
    crate::trace_whole!();
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

/// `(arrayof T R)` (C485): one word, to an object of Fixpt's layout: its
/// header (kind 3, vector; its elements the fields, as many as it has),
/// and, past what the header's 18 bits count, Fixpt's second header word
/// (`large`, kind 255, the count above it); its elements, each a cell;
/// its trailer. In the heap, kept.
pub struct Array<T: 'static>(NonNull<u64>, std::marker::PhantomData<&'static [Cell<T>]>);

impl<T> Clone for Array<T> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Array<T> {}

/// The header's field count, 18 bits; and its `large` bit.
const FIELDS_MAX: usize = (1 << 18) - 1;
const LARGE: u64 = 1 << 11;

/// Where an array's elements begin, in words: after one header word, or
/// two for a large one.
const fn elements_at(n: usize) -> usize {
    if n > FIELDS_MAX { 2 } else { 1 }
}

impl<T> Array<T> {
    /// Its elements, how many.
    #[inline]
    pub fn len(self) -> usize {
        // SAFETY: an `Array` is made only by `array_made`, of a block whose
        // first word is its header and, if large, its second the count.
        unsafe {
            let h = *self.0.as_ptr();
            if h & LARGE != 0 {
                (*self.0.as_ptr().add(1) >> 11) as usize
            } else {
                ((h >> 14) & FIELDS_MAX as u64) as usize
            }
        }
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    /// Its elements, each a cell.
    #[inline]
    pub fn cells(self) -> &'static [Cell<T>] {
        let n = self.len();
        // SAFETY: its elements follow its header words, `n` of them, in the
        // block it was made in (whose pointer it keeps), kept for the run.
        unsafe {
            std::slice::from_raw_parts(self.0.as_ptr().add(elements_at(n)).cast::<Cell<T>>(), n)
        }
    }

    /// Where its object is: its identity.
    #[inline]
    pub fn addr(self) -> usize {
        self.0.as_ptr() as usize
    }

    /// Its object's bytes: its header words, its elements, its trailer.
    pub fn object_bytes(self) -> usize {
        let n = self.len();
        (elements_at(n) * 8 + n * std::mem::size_of::<T>()).next_multiple_of(8) + 8
    }
}

/// An array's object at `addr`, traced (a census's, C486).
///
/// # Safety
/// `addr` is an array of `T`'s object, `array_made`'s, kept for the run.
#[cfg(feature = "heap-stats")]
unsafe fn trace_array<T: crate::Trace + 'static>(addr: usize, t: &mut crate::Tracer<'static>) {
    // SAFETY: the caller's.
    let a = Array::<T>(
        unsafe { NonNull::new_unchecked(addr as *mut u64) },
        std::marker::PhantomData,
    );
    crate::Trace::trace(a, t);
}

/// An array of `items`, kept for the run.
pub(crate) fn array_made<T: crate::Trace + 'static>(
    items: impl ExactSizeIterator<Item = T>,
) -> Array<T> {
    const {
        assert!(
            std::mem::align_of::<T>() <= 8,
            "an array's element aligned past a word"
        )
    };
    let n = items.len();
    let at = elements_at(n) * 8;
    let trailer_at = (at + n * std::mem::size_of::<T>()).next_multiple_of(8);
    let layout = std::alloc::Layout::from_size_align(trailer_at + 8, 8).expect("an array's size");
    crate::stats::note::<Array<T>>(layout.size());
    // SAFETY: a layout of nonzero size, aligned to a word; each word of
    // the header, each element and the trailer written before it is read.
    unsafe {
        let p = std::alloc::alloc(layout).cast::<u64>();
        let Some(base) = NonNull::new(p) else {
            std::alloc::handle_alloc_error(layout)
        };
        if n > FIELDS_MAX {
            p.write(layout::header(3, 0, 0, false, false) | LARGE);
            p.add(1)
                .write(layout::TAG_HEADER | 255 << 3 | (n as u64) << 11);
        } else {
            p.write(layout::header(3, n, 0, false, false));
        }
        let cells = p.cast::<u8>().add(at).cast::<Cell<T>>();
        let mut k = 0;
        for x in items.take(n) {
            cells.add(k).write(Cell::new(x));
            k += 1;
        }
        assert!(k == n, "an array's items fewer than it said");
        p.cast::<u8>()
            .add(trailer_at)
            .cast::<u64>()
            .write(layout::trailer(trailer_at / 8));
        #[cfg(feature = "heap-stats")]
        crate::census::object(p as usize, layout.size(), trace_array::<T>);
        Array(base, std::marker::PhantomData)
    }
}

#[cfg(test)]
mod array_tests {
    use super::*;

    #[test]
    fn an_array_small_and_large() {
        let a = array_made([1i64, 2, 3].into_iter());
        assert_eq!(a.len(), 3);
        a.cells()[1].set(20);
        assert_eq!(
            a.cells().iter().map(Cell::get).collect::<Vec<_>>(),
            vec![1, 20, 3]
        );
        let big = array_made((0..300_000).map(|k| k as u8));
        assert_eq!(big.len(), 300_000);
        assert_eq!(big.cells()[299_999].get(), (299_999 % 256) as u8);
        let units = array_made(std::iter::repeat_n((), 5));
        assert_eq!(units.len(), 5);
        assert_eq!(std::mem::size_of::<Array<i64>>(), 8);
    }
}
