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
}

/// `(arrayof T R)` (C485): one word, to an object of Fixpt's layout: its
/// header (kind 3, vector; its elements the fields, as many as it has),
/// and, past what the header's 18 bits count, Fixpt's second header word
/// (`large`, kind 255, the count above it); its elements, each a cell;
/// its trailer. In the heap, kept.
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

/// Where an array's elements begin, in words: after one header word, or
/// two for a large one.
const fn elements_at(n: usize) -> usize {
    if n > FIELDS_MAX { 2 } else { 1 }
}

impl<T: crate::Trace> Array<T> {
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

    /// Its elements, each a cell, its value held (C490: a product boxed).
    #[inline]
    pub fn cells(self) -> &'static [crate::Slot<T>] {
        let n = self.len();
        // SAFETY: its elements follow its header words, `n` of them, in the
        // block it was made in (whose pointer it keeps), kept for the run.
        unsafe {
            std::slice::from_raw_parts(
                self.0.as_ptr().add(elements_at(n)).cast::<crate::Slot<T>>(),
                n,
            )
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
        (elements_at(n) * 8 + n * std::mem::size_of::<crate::Slot<T>>()).next_multiple_of(8) + 8
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
            std::mem::align_of::<crate::Slot<T>>() <= 8,
            "an array's element aligned past a word"
        )
    };
    let n = items.len();
    let at = elements_at(n) * 8;
    let trailer_at = (at + n * std::mem::size_of::<crate::Slot<T>>()).next_multiple_of(8);
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
        let cells = p.cast::<u8>().add(at).cast::<crate::Slot<T>>();
        let mut k = 0;
        for x in items.take(n) {
            cells.add(k).write(crate::Slot::new(x));
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
        let units = array_made(std::iter::repeat_n((), 5));
        assert_eq!(units.len(), 5);
        assert_eq!(std::mem::size_of::<Array<i64>>(), 8);
    }
}

/// A product's kind (Fixpt's `product`, 37): its fields traced, never set.
const KIND_PRODUCT: u8 = 37;

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
        let o: &ProdObj<P> = Box::leak(Box::new(ProdObj {
            header: layout::header(KIND_PRODUCT, to_trailer - 1, 0, true, true),
            words,
            trailer: layout::trailer(to_trailer),
        }));
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

    /// Its object, kept for the run: what a union holds (C491).
    #[inline]
    pub fn object(self) -> &'static ProdObj<P>
    where
        P: 'static,
    {
        // SAFETY: made only by `new`, of a block never freed.
        unsafe { self.0.as_ref() }
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

/// A held product: its object reached.
impl<P: crate::Product> crate::Trace for Prod<P> {
    crate::trace_whole!();
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
        // Kind 37, two fields (the inner product's word, `'c'`'s), no
        // suffix (C496).
        let w = crate::Prod::new(p).header();
        assert_eq!(
            (w & 7, (w >> 3) & 0xff, (w >> 14) & 0x3ffff, w >> 32),
            (6, 37, 2, 0)
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
        let su = crate::sum_of("pair", p);
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

const TAG_MASK: usize = 0b111;
const TAG_IMMEDIATE: usize = 0b011;
const TAG_OBJECT: usize = 0b100;
const TAG_PAIR: usize = 0b001;
const IMM_NULL: usize = 2;
const IMM_FALSE: usize = 0;
const IMM_TRUE: usize = 1;
const IMM_UNIT: usize = 3;
const IMM_CHAR: usize = 8;
const IMM_F32: usize = 9;
const KIND_FLONUM: u8 = 5;

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
