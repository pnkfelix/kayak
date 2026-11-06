//! Fingerprints: 128-bit hashes of values, for the query engine's early
//! cutoff (two results with equal fingerprints are taken to be equal), and
//! later for anything persisted.
//!
//! - **The hash** is SipHash-1-3 with 128-bit output, rustc's choice for
//!   its fingerprints (`docs/research/queries.md` §4), with a fixed key.
//!   [`Sip128`] is written out here, for any number of rounds, and checked
//!   against `siphasher` 1.0.3's outputs and SipHash's published vector.
//! - **What is hashed** is what [`StableEncode`] writes: a byte stream we
//!   define, not whatever the standard library's `Hash` impls write today.
//!   Integers go little-endian at a fixed width (`usize` as `u64`), and
//!   sequences are preceded by their length. An enum writes a tag byte for
//!   its variant. Nothing that depends on a run may be written: no heap
//!   address, no interned index, no pointer.
//!
//! This is not [`crate::hash`]'s FxHash, which is for tables: 64 bits, and
//! the zero word hashes to zero.
// coopt-source: crates/coopt-base/src/fingerprint.rs@9283bba

use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

/// SipHash with 128-bit output, `C` compression and `D` finalisation
/// rounds.
#[derive(Clone, Debug)]
pub struct Sip128<const C: usize, const D: usize> {
    v: [u64; 4],
    /// Bytes not yet making a whole word, and how many.
    tail: u64,
    ntail: usize,
    len: u64,
}

#[inline]
fn round(v: &mut [u64; 4]) {
    v[0] = v[0].wrapping_add(v[1]);
    v[1] = v[1].rotate_left(13) ^ v[0];
    v[0] = v[0].rotate_left(32);
    v[2] = v[2].wrapping_add(v[3]);
    v[3] = v[3].rotate_left(16) ^ v[2];
    v[0] = v[0].wrapping_add(v[3]);
    v[3] = v[3].rotate_left(21) ^ v[0];
    v[2] = v[2].wrapping_add(v[1]);
    v[1] = v[1].rotate_left(17) ^ v[2];
    v[2] = v[2].rotate_left(32);
}

impl<const C: usize, const D: usize> Sip128<C, D> {
    pub fn new(k0: u64, k1: u64) -> Self {
        Sip128 {
            v: [
                k0 ^ 0x736f_6d65_7073_6575,
                k1 ^ 0x646f_7261_6e64_6f6d ^ 0xee,
                k0 ^ 0x6c79_6765_6e65_7261,
                k1 ^ 0x7465_6462_7974_6573,
            ],
            tail: 0,
            ntail: 0,
            len: 0,
        }
    }

    #[inline]
    fn compress(&mut self, m: u64) {
        self.v[3] ^= m;
        for _ in 0..C {
            round(&mut self.v);
        }
        self.v[0] ^= m;
    }

    pub fn write(&mut self, mut bytes: &[u8]) {
        self.len += bytes.len() as u64;
        if self.ntail > 0 {
            while self.ntail < 8 && !bytes.is_empty() {
                self.tail |= u64::from(bytes[0]) << (8 * self.ntail);
                self.ntail += 1;
                bytes = &bytes[1..];
            }
            if self.ntail < 8 {
                return;
            }
            let m = self.tail;
            self.compress(m);
            self.tail = 0;
            self.ntail = 0;
        }
        let (words, rest) = bytes.as_chunks::<8>();
        for &w in words {
            self.compress(u64::from_le_bytes(w));
        }
        for (i, &b) in rest.iter().enumerate() {
            self.tail |= u64::from(b) << (8 * i);
        }
        self.ntail = rest.len();
    }

    /// The two 64-bit halves, first and second as SipHash defines them.
    pub fn finish(&self) -> (u64, u64) {
        let mut s = self.clone();
        let b = ((s.len & 0xff) << 56) | s.tail;
        s.compress(b);
        s.v[2] ^= 0xee;
        for _ in 0..D {
            round(&mut s.v);
        }
        let h1 = s.v[0] ^ s.v[1] ^ s.v[2] ^ s.v[3];
        s.v[1] ^= 0xdd;
        for _ in 0..D {
            round(&mut s.v);
        }
        let h2 = s.v[0] ^ s.v[1] ^ s.v[2] ^ s.v[3];
        (h1, h2)
    }
}

/// The key every fingerprint is made with: "coopt fp" and "rint v1\0".
/// Changing it changes every fingerprint.
const KEY: (u64, u64) = (0x7066_2074_706f_6f63, 0x0031_7620_746e_6972);

/// A 128-bit fingerprint.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Fingerprint(pub u64, pub u64);

impl Fingerprint {
    /// The fingerprint of a sequence of two.
    pub fn combine(self, other: Fingerprint) -> Fingerprint {
        fingerprint(&(self, other))
    }
}

/// Writes values as the canonical byte stream fingerprints are made of;
/// if made by [`Encoder::recording`], keeps the stream too, which
/// [`crate::stored::Load`] reads back.
pub struct Encoder {
    sip: Sip128<1, 3>,
    record: Option<Vec<u8>>,
}

impl Default for Encoder {
    fn default() -> Encoder {
        Encoder {
            sip: Sip128::new(KEY.0, KEY.1),
            record: None,
        }
    }
}

impl Encoder {
    /// An encoder that keeps the bytes it hashes.
    pub fn recording() -> Encoder {
        Encoder {
            record: Some(Vec::new()),
            ..Encoder::default()
        }
    }

    /// The fingerprint, and the bytes if they were kept.
    pub fn into_parts(self) -> (Fingerprint, Option<Vec<u8>>) {
        (self.finish(), self.record)
    }

    /// The fingerprint of a stream an [`Encoder::recording`] kept: the
    /// fingerprint of the value it is of.
    pub fn of_stream(bytes: &[u8]) -> Fingerprint {
        let mut e = Encoder::default();
        e.write(bytes);
        e.finish()
    }

    fn write(&mut self, b: &[u8]) {
        self.sip.write(b);
        if let Some(r) = &mut self.record {
            r.extend_from_slice(b);
        }
    }

    pub fn bytes(&mut self, b: &[u8]) {
        self.u64(b.len() as u64);
        self.write(b);
    }

    pub fn u8(&mut self, x: u8) {
        self.write(&[x]);
    }

    pub fn u64(&mut self, x: u64) {
        self.write(&x.to_le_bytes());
    }

    /// A variant's tag, before its fields.
    pub fn tag(&mut self, t: u8) {
        self.u8(t);
    }

    pub fn finish(&self) -> Fingerprint {
        let (a, b) = self.sip.finish();
        Fingerprint(a, b)
    }
}

/// A value with a canonical encoding: equal values encode alike, and the
/// encoding depends on nothing but the value.
pub trait StableEncode {
    fn encode(&self, e: &mut Encoder);
}

/// The fingerprint of `x`'s stable encoding.
pub fn fingerprint<T: StableEncode + ?Sized>(x: &T) -> Fingerprint {
    let mut e = Encoder::default();
    x.encode(&mut e);
    e.finish()
}

macro_rules! unsigned {
    ($($t:ty)*) => {$(
        impl StableEncode for $t {
            fn encode(&self, e: &mut Encoder) {
                e.u64(*self as u64);
            }
        }
    )*};
}
unsigned!(u8 u16 u32 u64 usize);

macro_rules! signed {
    ($($t:ty)*) => {$(
        impl StableEncode for $t {
            fn encode(&self, e: &mut Encoder) {
                e.u64(i64::from(*self) as u64);
            }
        }
    )*};
}
signed!(i8 i16 i32 i64);

impl StableEncode for bool {
    fn encode(&self, e: &mut Encoder) {
        e.u8(u8::from(*self));
    }
}

impl StableEncode for () {
    fn encode(&self, _: &mut Encoder) {}
}

impl StableEncode for str {
    fn encode(&self, e: &mut Encoder) {
        e.bytes(self.as_bytes());
    }
}

impl StableEncode for String {
    fn encode(&self, e: &mut Encoder) {
        self.as_str().encode(e);
    }
}

impl<T: StableEncode> StableEncode for [T] {
    fn encode(&self, e: &mut Encoder) {
        e.u64(self.len() as u64);
        for x in self {
            x.encode(e);
        }
    }
}

impl<T: StableEncode> StableEncode for Vec<T> {
    fn encode(&self, e: &mut Encoder) {
        self.as_slice().encode(e);
    }
}

impl<T: StableEncode> StableEncode for Option<T> {
    fn encode(&self, e: &mut Encoder) {
        match self {
            None => e.tag(0),
            Some(x) => {
                e.tag(1);
                x.encode(e);
            }
        }
    }
}

impl<T: StableEncode + ?Sized> StableEncode for Rc<T> {
    fn encode(&self, e: &mut Encoder) {
        (**self).encode(e);
    }
}

impl<T: StableEncode + ?Sized> StableEncode for std::sync::Arc<T> {
    fn encode(&self, e: &mut Encoder) {
        (**self).encode(e);
    }
}

impl<T: StableEncode + ?Sized> StableEncode for Box<T> {
    fn encode(&self, e: &mut Encoder) {
        (**self).encode(e);
    }
}

impl<T: StableEncode + ?Sized> StableEncode for &T {
    fn encode(&self, e: &mut Encoder) {
        (**self).encode(e);
    }
}

impl<A: StableEncode, B: StableEncode> StableEncode for (A, B) {
    fn encode(&self, e: &mut Encoder) {
        self.0.encode(e);
        self.1.encode(e);
    }
}

impl<A: StableEncode, B: StableEncode, C: StableEncode> StableEncode for (A, B, C) {
    fn encode(&self, e: &mut Encoder) {
        self.0.encode(e);
        self.1.encode(e);
        self.2.encode(e);
    }
}

/// In key order, so the encoding does not depend on insertion order.
impl<K: StableEncode, V: StableEncode> StableEncode for BTreeMap<K, V> {
    fn encode(&self, e: &mut Encoder) {
        e.u64(self.len() as u64);
        for (k, v) in self {
            k.encode(e);
            v.encode(e);
        }
    }
}

impl<T: StableEncode> StableEncode for BTreeSet<T> {
    fn encode(&self, e: &mut Encoder) {
        e.u64(self.len() as u64);
        for x in self {
            x.encode(e);
        }
    }
}

impl StableEncode for Fingerprint {
    fn encode(&self, e: &mut Encoder) {
        e.u64(self.0);
        e.u64(self.1);
    }
}
