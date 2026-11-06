//! Values read back from their stable encoding (`TODO.md` §48: a store of
//! job values).
//!
//! A value's [`StableEncode`] stream is all of it, so the stream an
//! [`Encoder::recording`] keeps is what the value is stored as, and its
//! fingerprint is the fingerprint of that stream. [`Load`] reads the
//! stream back, each type's way mirroring its `encode`: so a value loaded
//! and encoded again gives the same bytes, and the same fingerprint, which
//! [`round_trips`] checks.
// coopt-source: crates/coopt-base/src/stored.rs@9283bba

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::rc::Rc;
use std::sync::Arc;

use crate::fingerprint::{Encoder, Fingerprint, StableEncode};

/// Bytes that are no value of the type asked for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bad(pub String);

impl fmt::Display for Bad {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Reads a stable encoding, as [`Encoder`] writes one.
pub struct Decoder<'a> {
    bytes: &'a [u8],
}

impl<'a> Decoder<'a> {
    pub fn new(bytes: &'a [u8]) -> Decoder<'a> {
        Decoder { bytes }
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], Bad> {
        if self.bytes.len() < n {
            return Err(Bad(format!("{n} bytes wanted, {} left", self.bytes.len())));
        }
        let (a, b) = self.bytes.split_at(n);
        self.bytes = b;
        Ok(a)
    }

    pub fn u8(&mut self) -> Result<u8, Bad> {
        Ok(self.take(1)?[0])
    }

    pub fn u64(&mut self) -> Result<u64, Bad> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().expect("eight")))
    }

    /// A variant's tag.
    pub fn tag(&mut self) -> Result<u8, Bad> {
        self.u8()
    }

    pub fn bytes(&mut self) -> Result<&'a [u8], Bad> {
        let n = self.length()?;
        self.take(n)
    }

    /// A length, as `u64`: no longer than what is left could hold.
    pub fn length(&mut self) -> Result<usize, Bad> {
        let n = self.u64()?;
        if n > self.bytes.len() as u64 {
            return Err(Bad(format!(
                "a length of {n}, {} bytes left",
                self.bytes.len()
            )));
        }
        Ok(n as usize)
    }

    pub fn string(&mut self) -> Result<String, Bad> {
        String::from_utf8(self.bytes()?.to_vec()).map_err(|e| Bad(e.to_string()))
    }

    /// A tag that names no variant.
    pub fn bad_tag<T>(what: &str, tag: u8) -> Result<T, Bad> {
        Err(Bad(format!("{tag} is no {what}'s tag")))
    }
}

/// A type read back from its stable encoding: `load` mirrors `encode`.
pub trait Load: Sized {
    fn load(d: &mut Decoder<'_>) -> Result<Self, Bad>;
}

/// A value and its fingerprint, as stored: what `encode` writes.
pub fn store<T: StableEncode + ?Sized>(x: &T) -> (Fingerprint, Vec<u8>) {
    let mut e = Encoder::recording();
    x.encode(&mut e);
    let (f, b) = e.into_parts();
    (f, b.expect("recorded"))
}

/// The value `bytes` store, all of them.
pub fn load<T: Load>(bytes: &[u8]) -> Result<T, Bad> {
    let mut d = Decoder::new(bytes);
    let x = T::load(&mut d)?;
    if !d.bytes.is_empty() {
        return Err(Bad(format!("{} bytes left over", d.bytes.len())));
    }
    Ok(x)
}

/// Whether `x`, stored and loaded, stores as it did: its `load` mirrors
/// its `encode`.
pub fn round_trips<T: StableEncode + Load>(x: &T) -> bool {
    let (f, b) = store(x);
    match load::<T>(&b) {
        Ok(y) => store(&y) == (f, b),
        Err(_) => false,
    }
}

macro_rules! unsigned {
    ($($t:ty)*) => {$(
        impl Load for $t {
            fn load(d: &mut Decoder<'_>) -> Result<$t, Bad> {
                let n = d.u64()?;
                <$t>::try_from(n).map_err(|_| Bad(format!("{n} is out of range")))
            }
        }
    )*};
}
unsigned!(u8 u16 u32 u64 usize);

macro_rules! signed {
    ($($t:ty)*) => {$(
        impl Load for $t {
            fn load(d: &mut Decoder<'_>) -> Result<$t, Bad> {
                let n = d.u64()? as i64;
                <$t>::try_from(n).map_err(|_| Bad(format!("{n} is out of range")))
            }
        }
    )*};
}
signed!(i8 i16 i32 i64);

impl Load for bool {
    fn load(d: &mut Decoder<'_>) -> Result<bool, Bad> {
        match d.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            t => Decoder::bad_tag("bool", t),
        }
    }
}

impl Load for () {
    fn load(_: &mut Decoder<'_>) -> Result<(), Bad> {
        Ok(())
    }
}

impl Load for String {
    fn load(d: &mut Decoder<'_>) -> Result<String, Bad> {
        d.string()
    }
}

impl<T: Load> Load for Vec<T> {
    fn load(d: &mut Decoder<'_>) -> Result<Vec<T>, Bad> {
        let n = d.u64()?;
        // Each item takes a byte at least, but for `()`: no allocation
        // ahead of what the bytes could hold.
        let mut v = Vec::with_capacity((n as usize).min(d.bytes.len()));
        for _ in 0..n {
            v.push(T::load(d)?);
        }
        Ok(v)
    }
}

impl<T: Load> Load for Option<T> {
    fn load(d: &mut Decoder<'_>) -> Result<Option<T>, Bad> {
        match d.tag()? {
            0 => Ok(None),
            1 => Ok(Some(T::load(d)?)),
            t => Decoder::bad_tag("option", t),
        }
    }
}

impl<T: Load> Load for Rc<T> {
    fn load(d: &mut Decoder<'_>) -> Result<Rc<T>, Bad> {
        Ok(Rc::new(T::load(d)?))
    }
}

impl<T: Load> Load for Arc<T> {
    fn load(d: &mut Decoder<'_>) -> Result<Arc<T>, Bad> {
        Ok(Arc::new(T::load(d)?))
    }
}

impl Load for Arc<str> {
    fn load(d: &mut Decoder<'_>) -> Result<Arc<str>, Bad> {
        Ok(d.string()?.into())
    }
}

impl Load for Rc<str> {
    fn load(d: &mut Decoder<'_>) -> Result<Rc<str>, Bad> {
        Ok(d.string()?.into())
    }
}

impl<T: Load> Load for Box<T> {
    fn load(d: &mut Decoder<'_>) -> Result<Box<T>, Bad> {
        Ok(Box::new(T::load(d)?))
    }
}

impl<A: Load, B: Load> Load for (A, B) {
    fn load(d: &mut Decoder<'_>) -> Result<(A, B), Bad> {
        Ok((A::load(d)?, B::load(d)?))
    }
}

impl<A: Load, B: Load, C: Load> Load for (A, B, C) {
    fn load(d: &mut Decoder<'_>) -> Result<(A, B, C), Bad> {
        Ok((A::load(d)?, B::load(d)?, C::load(d)?))
    }
}

impl<K: Load + Ord, V: Load> Load for BTreeMap<K, V> {
    fn load(d: &mut Decoder<'_>) -> Result<BTreeMap<K, V>, Bad> {
        let n = d.u64()?;
        let mut m = BTreeMap::new();
        for _ in 0..n {
            let k = K::load(d)?;
            m.insert(k, V::load(d)?);
        }
        Ok(m)
    }
}

impl<T: Load + Ord> Load for BTreeSet<T> {
    fn load(d: &mut Decoder<'_>) -> Result<BTreeSet<T>, Bad> {
        let n = d.u64()?;
        let mut s = BTreeSet::new();
        for _ in 0..n {
            s.insert(T::load(d)?);
        }
        Ok(s)
    }
}

impl Load for Fingerprint {
    fn load(d: &mut Decoder<'_>) -> Result<Fingerprint, Bad> {
        Ok(Fingerprint(d.u64()?, d.u64()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fingerprint::fingerprint;

    type Mixed = (Vec<Option<String>>, BTreeMap<u32, (i64, bool)>);

    #[test]
    fn values_round_trip_with_their_fingerprints() {
        let v: Mixed = (
            vec![Some("a".into()), None, Some("héllo".into())],
            [(1, (-5, true)), (9, (i64::MIN, false))].into(),
        );
        let (f, b) = store(&v);
        assert_eq!(f, fingerprint(&v));
        assert_eq!(
            load::<(Vec<Option<String>>, BTreeMap<u32, (i64, bool)>)>(&b),
            Ok(v.clone())
        );
        assert!(round_trips(&v));
        let a: Arc<str> = "shared".into();
        assert!(round_trips(&a));
    }

    #[test]
    fn bad_bytes_are_refused() {
        let (_, b) = store(&vec![1u64, 2, 3]);
        assert!(load::<Vec<u64>>(&b[..b.len() - 1]).is_err());
        assert!(load::<u8>(&store(&300u64).1).is_err());
        assert!(load::<bool>(&[2]).is_err());
        assert!(load::<u64>(&[0; 9]).is_err(), "a byte left over");
        // A length longer than the bytes left is refused, not allocated.
        let mut huge = u64::MAX.to_le_bytes().to_vec();
        huge.push(0);
        assert!(load::<String>(&huge).is_err());
    }
}
