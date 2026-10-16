//! FX-26's bloblets and blobrefs (CLAUDES_CHOICES.md C31): a bloblet is
//! fields, each a cell, and a suffix of raw bytes; a blobref is a pointer
//! to a flat (a scalar, or a product or array of flats) at a byte offset
//! in an owner's suffix, which it keeps alive. FX-26's suffix is untyped:
//! one may be read as several flats, at any offset. So here the suffix is
//! words, `Cell<u64>`s, and a blobref is its owner and an offset; a
//! scalar is read or written within its word when aligned to its size
//! (C's layout, which FX-26's flats have, makes every field so), else
//! byte by byte. In safe Rust, little-endian as Fixpt's hosts are.

use crate::{Holds, Int, fail, i64_to_int, int_to_i64};
use std::cell::Cell;

/// `(bloblet (fields T …) R)`: `fields` a tuple of cells, one a field.
pub struct Bloblet<Fs> {
    pub fields: Fs,
    suffix: Box<[Cell<u64>]>,
    bytes: usize,
    /// Its copy in the code area, if it is one `seal-code` gave (C116): its
    /// suffix frozen, its code runnable.
    pub(crate) code: Option<kayak_jit::Code>,
    /// For a sealed copy, how a field written is written beside its code
    /// too, as Fixpt's code bloblet's fields are its code's (C116).
    resync: Option<fn(&Bloblet<Fs>, usize)>,
}

/// An `int` as a size or an index: one past an `i64` is never small
/// (`Int`'s results are made small whenever they fit), so it is too big.
fn index_of(n: Int, what: &str) -> usize {
    match n {
        Int::Small(i) if i >= 0 => i as usize,
        _ => fail(what),
    }
}

impl<Fs> Bloblet<Fs> {
    fn new(bytes: Int, fields: Fs) -> Self {
        let bytes = index_of(bytes, "make-bloblet: a negative size");
        Bloblet {
            fields,
            suffix: (0..bytes.div_ceil(8)).map(|_| Cell::new(0)).collect(),
            bytes,
            code: None,
            resync: None,
        }
    }

    fn byte(&self, i: usize) -> u64 {
        (self.suffix[i / 8].get() >> ((i % 8) * 8)) & 0xff
    }

    fn set_byte(&self, i: usize, v: u64) {
        if self.code.is_some() {
            fail("the bloblet's suffix is frozen");
        }
        let w = &self.suffix[i / 8];
        let sh = (i % 8) * 8;
        w.set((w.get() & !(0xff << sh)) | ((v & 0xff) << sh));
    }

    /// `n` bytes at `at`, as the low bytes of a word: in its word if
    /// aligned to `n`, else byte by byte; within the suffix, or an error.
    #[inline]
    fn read(&self, at: usize, n: usize) -> u64 {
        if at.checked_add(n).is_none_or(|end| end > self.bytes) {
            fail("blobref-get: past the suffix");
        }
        if n == 8 && at.is_multiple_of(8) {
            self.suffix[at / 8].get()
        } else if at.is_multiple_of(n) {
            (self.suffix[at / 8].get() >> ((at % 8) * 8)) & (u64::MAX >> (64 - n * 8))
        } else {
            (0..n).fold(0, |v, i| v | (self.byte(at + i) << (i * 8)))
        }
    }

    #[inline]
    fn write(&self, at: usize, n: usize, v: u64) {
        if at.checked_add(n).is_none_or(|end| end > self.bytes) {
            fail("blobref-set!: past the suffix");
        }
        if self.code.is_some() {
            fail("the bloblet's suffix is frozen");
        }
        if n == 8 && at.is_multiple_of(8) {
            self.suffix[at / 8].set(v)
        } else if at.is_multiple_of(n) {
            let w = &self.suffix[at / 8];
            let sh = (at % 8) * 8;
            let mask = (u64::MAX >> (64 - n * 8)) << sh;
            w.set((w.get() & !mask) | ((v << sh) & mask))
        } else {
            (0..n).for_each(|i| self.set_byte(at + i, v >> (i * 8)))
        }
    }
}

impl<Fs> Bloblet<Fs> {
    /// One of fields `fields` and suffix `bytes`, as native code made it
    /// (C116), copied.
    pub(crate) fn of_native(fields: Fs, bytes: &[u8]) -> Self {
        let b = Bloblet::new(i64_to_int(bytes.len() as i64), fields);
        for (i, &x) in bytes.iter().enumerate() {
            b.set_byte(i, x as u64);
        }
        b
    }

    /// Its suffix's bytes; and its code, if it is a sealed copy.
    pub(crate) fn native_parts(&self) -> (Vec<u8>, Option<kayak_jit::Code>) {
        (
            (0..self.bytes).map(|i| self.byte(i) as u8).collect(),
            self.code,
        )
    }
}

impl<Fs: crate::NativeFields + Clone> Bloblet<Fs> {
    /// Its copy sealed into the code area (`seal-code`, C116): its suffix
    /// the code, frozen; its fields beside it, after a trailer if it has
    /// any, as Fixpt's; a field written later written there too.
    pub(crate) fn sealed_copy(&self) -> Self {
        let bytes: Vec<u8> = (0..self.bytes).map(|i| self.byte(i) as u8).collect();
        let fields = self.fields.field_words();
        let mut slots = Vec::new();
        if !fields.is_empty() {
            slots.push(kayak_jit::layout::trailer(fields.len() + 1));
            slots.extend(fields);
        }
        let code = kayak_jit::seal(kayak_jit::layout::KIND_BLOBLET, &slots, false, &bytes);
        Bloblet {
            fields: self.fields.clone(),
            suffix: self.suffix.clone(),
            bytes: self.bytes,
            code: Some(code),
            resync: Some(|b, k| {
                if let Some(c) = b.code {
                    c.set_slot(k + 2, b.fields.field_word(k))
                }
            }),
        }
    }
}

/// After `(bloblet-set! b k v)`: a sealed copy's field `k` written beside
/// its code too (C116); nothing for any other bloblet.
pub trait Written {
    fn written(self, k: usize);
}

impl<Fs> Written for &Bloblet<Fs> {
    #[inline]
    fn written(self, k: usize) {
        if let Some(f) = self.resync {
            f(self, k)
        }
    }
}

impl<Fs, S> Written for &TBloblet<Fs, S> {
    #[inline]
    fn written(self, _: usize) {}
}

pub fn bloblet_written<W: Written>(b: W, k: usize) {
    b.written(k)
}

/// `(make-bloblet bytes e …)`: in the heap, never freed.
pub fn make_bloblet<'a, Fs: 'a>(bytes: Int, fields: Fs) -> &'a Bloblet<Fs> {
    crate::alloc(Bloblet::new(bytes, fields))
}

/// `(rmake-bloblet r bytes e …)`: in the region `r`.
pub fn rmake_bloblet<Fs, R: Holds<Bloblet<Fs>>>(r: &R, bytes: Int, fields: Fs) -> &Bloblet<Fs> {
    r.arena().alloc(Bloblet::new(bytes, fields))
}

pub fn bloblet_bytes<Fs>(b: &Bloblet<Fs>) -> Int {
    i64_to_int(b.bytes as i64)
}

pub fn bloblet_byte<Fs>(b: &Bloblet<Fs>, i: Int) -> Int {
    let i = index_of(i, "bloblet-byte: no such byte");
    if i >= b.bytes {
        fail("bloblet-byte: no such byte");
    }
    i64_to_int(b.byte(i) as i64)
}

pub fn bloblet_set_byte_x<Fs>(b: &Bloblet<Fs>, i: Int, v: Int) {
    let i = index_of(i, "bloblet-set-byte!: no such byte");
    if i >= b.bytes {
        fail("bloblet-set-byte!: no such byte");
    }
    b.set_byte(i, int_to_i64(v) as u64)
}

/// What a blobref points into: a bloblet, as a reference of any lifetime
/// (the heap's, or a `letrena`'s), whose suffix is read and written.
pub trait Owner: Copy {
    /// The suffix's length, in bytes.
    fn bytes(self) -> usize;
    fn read(self, at: usize, n: usize) -> u64;
    fn write(self, at: usize, n: usize, v: u64);
}

impl<Fs> Owner for &Bloblet<Fs> {
    #[inline]
    fn bytes(self) -> usize {
        self.bytes
    }

    #[inline]
    fn read(self, at: usize, n: usize) -> u64 {
        Bloblet::read(self, at, n)
    }

    #[inline]
    fn write(self, at: usize, n: usize, v: u64) {
        Bloblet::write(self, at, n, v)
    }
}

/// `(blobref B F)`: a flat in `owner`'s suffix, `at` bytes in; `B` the
/// owner's Rust type, so that a blobref's type is written as its owner's
/// is. Its flat is the checker's, which tells the emitter each read's and
/// step's layout.
#[derive(Clone, Copy)]
pub struct BlobRef<B> {
    pub owner: B,
    at: usize,
}

/// `(blobref-at b off)`, of a flat of `size` bytes.
pub fn blobref_at<B: Owner>(b: B, off: Int, size: usize) -> BlobRef<B> {
    let at = index_of(off, "blobref-at: a negative offset");
    whole(BlobRef { owner: b, at }, size)
}

/// A new blobref, its flat of `size` bytes lying whole in the suffix, as
/// FX-26 checks it where one is made (Fixpt's TODO §77).
fn whole<B: Owner>(r: BlobRef<B>, size: usize) -> BlobRef<B> {
    if r.at
        .checked_add(size)
        .is_none_or(|end| end > r.owner.bytes())
    {
        fail(&format!(
            "suffix ref: a flat of {size} bytes at byte {} runs past the end of the suffix",
            r.at
        ));
    }
    r
}

macro_rules! scalar {
    ($get:ident, $set:ident, $t:ty, $n:expr, $from:expr, $to:expr) => {
        #[inline]
        pub fn $get(self) -> $t {
            let from: fn(u64) -> $t = $from;
            from(self.owner.read(self.at, $n))
        }

        #[inline]
        pub fn $set(self, v: $t) {
            let to: fn($t) -> u64 = $to;
            self.owner.write(self.at, $n, to(v))
        }
    };
}

impl<B: Owner> BlobRef<B> {
    /// `(blobref+ r d)`.
    pub fn plus(self, d: Int) -> Self {
        let at = int_to_i64(d)
            .checked_add(self.at as i64)
            .and_then(|at| usize::try_from(at).ok())
            .unwrap_or_else(|| fail("blobref+: a negative offset"));
        BlobRef { at, ..self }
    }

    /// An `extract` of a flat product's field, `n` bytes in.
    #[inline]
    pub fn field(self, n: usize) -> Self {
        BlobRef {
            at: self.at + n,
            ..self
        }
    }

    /// `(blobref-index r i)` of a flat array of `len` elements of `size`
    /// bytes.
    #[inline]
    pub fn index(self, i: Int, size: usize, len: usize) -> Self {
        let i = int_to_i64(i);
        if i < 0 || i >= len as i64 {
            fail("blobref-index: out of range");
        }
        BlobRef {
            at: self.at + i as usize * size,
            ..self
        }
    }

    /// A flat sum's tag, `k` bytes, the label as it is (C33).
    #[inline]
    pub fn tag(self, k: usize) -> u64 {
        self.owner.read(self.at, k)
    }

    /// `(blobref-variant! r label)`: the tag written, and a blobref to the
    /// variant, `off` bytes in.
    #[inline]
    pub fn variant(self, k: usize, label: u64, off: usize) -> Self {
        self.owner.write(self.at, k, label);
        self.field(off)
    }

    pub fn offset(self) -> Int {
        i64_to_int(self.at as i64)
    }

    scalar!(get_i64, set_i64, i64, 8, |w| w as i64, |v| v as u64);
    scalar!(get_u64, set_u64, u64, 8, |w| w, |v| v);
    scalar!(get_f64, set_f64, f64, 8, f64::from_bits, f64::to_bits);
    scalar!(get_i32, set_i32, i32, 4, |w| w as u32 as i32, |v| v as u32
        as u64);
    scalar!(get_u32, set_u32, u32, 4, |w| w as u32, |v| v as u64);
    scalar!(
        get_f32,
        set_f32,
        f32,
        4,
        |w| f32::from_bits(w as u32),
        |v| v.to_bits() as u64
    );
}

/// A `tagcase` on a flat sum whose tag no arm has, and no `else`.
pub fn no_flat_tag() -> ! {
    fail("tagcase: no arm for this flat sum's tag")
}

/// `(blobref-owner r)`.
pub fn blobref_owner<B: Owner>(r: BlobRef<B>) -> B {
    r.owner
}

/// `(blobref-offset r)`.
pub fn blobref_offset<B: Owner>(r: BlobRef<B>) -> Int {
    r.offset()
}

/// `(blobref+ r d)`, to a flat of `size` bytes.
pub fn blobref_plus<B: Owner>(r: BlobRef<B>, d: Int, size: usize) -> BlobRef<B> {
    whole(r.plus(d), size)
}

// The typed fast path (C32): a bloblet whose suffix the program reads
// only as one flat, from offset 0, by `extract` and `blobref-index`, never
// byte by byte or at an offset of its own: its suffix is that flat, cells
// and arrays of them, and a blobref a reference into it. No word is
// shifted, no offset checked but an array's index.

/// A flat as a suffix holds it, all zero bytes to begin with, as FX-26's
/// suffix is.
pub trait Flat {
    fn zeroed() -> Self;
}

macro_rules! flat_scalar {
    ($($t:ty),*) => {
        $(impl Flat for Cell<$t> {
            fn zeroed() -> Self {
                Cell::new(0 as $t)
            }
        })*
    };
}

flat_scalar!(i64, u64, f64, i32, u32, f32);

/// `(arrayof F n)` in a typed suffix: `n` `F`s, on the heap with the
/// bloblet (a large one does not fit a stack).
pub struct FlatArr<T, const N: usize>(Box<[T]>);

impl<T: Flat, const N: usize> Flat for FlatArr<T, N> {
    fn zeroed() -> Self {
        FlatArr((0..N).map(|_| T::zeroed()).collect())
    }
}

impl<T, const N: usize> FlatArr<T, N> {
    /// `(blobref-index r i)`.
    #[inline]
    pub fn at(&self, i: Int) -> &T {
        match i {
            Int::Small(i) if (0..N as i64).contains(&i) => &self.0[i as usize],
            _ => fail("blobref-index: out of range"),
        }
    }
}

/// A bloblet of a typed suffix, `S`.
pub struct TBloblet<Fs, S> {
    pub fields: Fs,
    pub suffix: S,
    bytes: usize,
}

impl<Fs, S: Flat> TBloblet<Fs, S> {
    fn new(bytes: Int, fields: Fs) -> Self {
        TBloblet {
            fields,
            suffix: S::zeroed(),
            bytes: index_of(bytes, "make-bloblet: a negative size"),
        }
    }
}

/// `(make-bloblet bytes e …)` of a typed suffix.
pub fn make_tbloblet<'a, Fs: 'a, S: Flat + 'a>(bytes: Int, fields: Fs) -> &'a TBloblet<Fs, S> {
    crate::alloc(TBloblet::new(bytes, fields))
}

/// `(rmake-bloblet r bytes e …)` of a typed suffix.
pub fn rmake_tbloblet<Fs, S: Flat, R: Holds<TBloblet<Fs, S>>>(
    r: &R,
    bytes: Int,
    fields: Fs,
) -> &TBloblet<Fs, S> {
    r.arena().alloc(TBloblet::new(bytes, fields))
}

impl<Fs, S> crate::Show for TBloblet<Fs, S> {
    fn show(&self, out: &mut String) {
        out.push_str("#<bloblet>");
    }
}

impl<Fs, S> TBloblet<Fs, S> {
    pub fn byte_size(&self) -> usize {
        self.bytes
    }
}

impl<Fs> crate::Show for Bloblet<Fs> {
    fn show(&self, out: &mut String) {
        out.push_str("#<bloblet>");
    }
}

impl<B> crate::Show for BlobRef<B> {
    fn show(&self, out: &mut String) {
        out.push_str("#<suffix-ref>");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::int_lit;

    #[test]
    fn scalars_at_their_offsets_and_not() {
        let b = make_bloblet(int_lit(72), (Cell::new(int_lit(3)),));
        let r = blobref_at(b, int_lit(0), 72);
        r.field(16).set_f64(2.5);
        r.field(8).set_i32(-7);
        r.field(12).set_u32(4_000_000_000);
        assert_eq!(r.field(16).get_f64(), 2.5);
        assert_eq!(r.field(8).get_i32(), -7);
        assert_eq!(r.field(12).get_u32(), 4_000_000_000);
        let odd = r.plus(int_lit(41));
        odd.set_i32(-7);
        assert_eq!(odd.get_i32(), -7);
        assert_eq!(r.field(8).get_i32(), -7);
        odd.set_i64(i64::MIN + 5);
        assert_eq!(odd.get_i64(), i64::MIN + 5);
        assert_eq!(int_to_i64(r.index(int_lit(2), 24, 3).offset()), 48);
        assert_eq!(int_to_i64(bloblet_byte(b, int_lit(8))), 0xf9);
    }
}
