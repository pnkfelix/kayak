//! Job values kept beside a log (`TODO.md` §48, decided with the user): a
//! run recorded with them keeps each job's value, by its fingerprint, as
//! its stable encoding; a replay given them may go on with a recorded
//! value where a job, run again by another build, gives another, or may
//! take every job's value from them rather than run it.
//!
//! The file: `coopt-values 1\n`, then for each value its fingerprint (two
//! `u64`s), its length (a `u64`) and its bytes, little-endian. Reading it
//! checks each value's bytes against its fingerprint.
// coopt-source: crates/coopt-runner/src/values.rs@9283bba

use std::collections::BTreeMap;

use kayak_base::Fingerprint;
use kayak_base::fingerprint::Encoder;

const MAGIC: &[u8] = b"coopt-values 1\n";

/// Job values, by fingerprint.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Values {
    by: BTreeMap<Fingerprint, Vec<u8>>,
}

impl Values {
    pub fn insert(&mut self, fp: Fingerprint, bytes: Vec<u8>) {
        self.by.entry(fp).or_insert(bytes);
    }

    pub fn get(&self, fp: &Fingerprint) -> Option<&[u8]> {
        self.by.get(fp).map(Vec::as_slice)
    }

    pub fn len(&self) -> usize {
        self.by.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by.is_empty()
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = MAGIC.to_vec();
        for (Fingerprint(a, b), bytes) in &self.by {
            out.extend_from_slice(&a.to_le_bytes());
            out.extend_from_slice(&b.to_le_bytes());
            out.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
            out.extend_from_slice(bytes);
        }
        out
    }

    pub fn read(file: &[u8]) -> Result<Values, String> {
        let mut rest = file
            .strip_prefix(MAGIC)
            .ok_or("not a file of coopt's job values (version 1)")?;
        let u64_ = |rest: &mut &[u8]| -> Result<u64, String> {
            let (h, t) = rest
                .split_first_chunk::<8>()
                .ok_or("job values cut short")?;
            *rest = t;
            Ok(u64::from_le_bytes(*h))
        };
        let mut v = Values::default();
        while !rest.is_empty() {
            let fp = Fingerprint(u64_(&mut rest)?, u64_(&mut rest)?);
            let n = u64_(&mut rest)?;
            if n > rest.len() as u64 {
                return Err("job values cut short".into());
            }
            let (bytes, t) = rest.split_at(n as usize);
            rest = t;
            if Encoder::of_stream(bytes) != fp {
                return Err(format!(
                    "a job value whose bytes are not its fingerprint's, {fp:?}"
                ));
            }
            v.insert(fp, bytes.to_vec());
        }
        Ok(v)
    }
}

/// What a replay does with recorded job values.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UseValues {
    /// Runs each job again; where it gives another value than the one
    /// recorded, and that value is kept, warns and goes on with it.
    #[default]
    Fallback,
    /// Runs each job again; another value is a divergence, as with no
    /// values at all.
    Strict,
    /// Takes each job's value from the values kept, without running it;
    /// runs only a job whose value is not kept.
    Trust,
}

#[cfg(test)]
mod tests {
    use super::*;
    use kayak_base::stored::store;

    #[test]
    fn values_read_back_and_are_checked() {
        let mut v = Values::default();
        for x in [1u64, 2, 3] {
            let (fp, bytes) = store(&("value", x));
            v.insert(fp, bytes);
        }
        let file = v.encode();
        assert_eq!(Values::read(&file), Ok(v));
        let mut bad = file.clone();
        *bad.last_mut().unwrap() ^= 1;
        assert!(
            Values::read(&bad)
                .unwrap_err()
                .contains("not its fingerprint")
        );
        assert!(Values::read(&file[..file.len() - 1]).is_err());
        assert!(Values::read(b"something else").is_err());
    }
}
