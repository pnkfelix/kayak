//! FX-26's sums, structural (C409; the user: "Given that FX sums and
//! unions are structural ... can't you map keywords to unique u64, eg by
//! SipHashing them, and then manually assign those uid's as
//! discriminants?").
//!
//! A tag is the sum's, the same in every sum that has it, so every sum has
//! one representation: a reference to a cell, its tag's discriminant, a
//! fixed hash of the tag's name ([`tag_hash`]), and its name, for writing
//! it, then its payload. A sum of fewer tags is a sum of more as it is, and
//! a sum is a union's member as it is: no conversion, no new enum for each
//! set of tags. A `tagcase` reads the tag and, in the arm it chooses, the
//! payload at the type the checker found for that tag ([`sum_payload`]),
//! which is the one place here that is not safe Rust: what makes it sound is
//! the checker, which gives each tag of a sum one payload type.
//!
//! Two tags hashing alike would be two tags taken for one: kfx asserts, as
//! the program is built, that its tags and kayak-rt's hash apart
//! ([`tags_distinct`]), so that a collision is a build's failure, never a
//! wrong arm taken.

#![allow(unsafe_code)]

use crate::Show;
use std::any::TypeId;

/// What every sum's cell begins with: its tag's discriminant and name, and
/// its payload's type, which every read of the payload checks: kfx's idea
/// of a primitive's sum and kayak-rt's are two, which rustc no longer
/// compares once sums are erased, so a disagreement fails loudly rather
/// than reads a payload at the wrong type. A function, not a `TypeId`, so
/// that a cell can be made in a `const` (kayak-rt's tables).
#[repr(C)]
pub struct SumHead {
    tag: u64,
    name: &'static str,
    payload_type: fn() -> TypeId,
    /// Its whole cell, to trace (C491): what a header's kind tells a
    /// collector, here the cell's type.
    cell: for<'s> unsafe fn(&'s SumHead) -> &'s (dyn crate::Visit + 's),
}

/// A head's cell, a `SumOf<T>`.
///
/// # Safety
/// `h` is the head of a `SumOf<T>`.
unsafe fn cell_of<'s, T: crate::Trace + 'static>(h: &'s SumHead) -> &'s (dyn crate::Visit + 's) {
    // SAFETY: the caller's; the head is the cell's first field (`repr(C)`).
    unsafe { &*(h as *const SumHead as *const SumOf<T>) }
}

/// A sum's head: its cell visited (C491), as its cell's type has it.
impl crate::Visit for SumHead {
    fn visit<'a>(&'a self, t: &mut crate::Tracer<'a>) {
        // SAFETY: a head is made only in its cell, `cell` of the cell's type.
        unsafe { (self.cell)(self) }.visit(t);
    }
    fn visit_size(&self) -> usize {
        // SAFETY: as above.
        unsafe { (self.cell)(self) }.visit_size()
    }
    fn visit_name(&self) -> &'static str {
        // SAFETY: as above.
        unsafe { (self.cell)(self) }.visit_name()
    }
}

/// A sum: its cell reached, precisely (C491; before, a plain reference).
impl crate::Trace for Sum {
    crate::trace_whole!();
    #[inline]
    fn trace<'a>(self, t: &mut crate::Tracer<'a>)
    where
        Self: 'a,
    {
        t.reach(self.0);
    }
}

/// A sum's cell: its head, then its payload.
#[repr(C)]
pub struct SumOf<T: crate::Trace> {
    head: SumHead,
    /// Held (C490): a product boxed, one word, as FX-26's sum holds one.
    payload: crate::Held<T>,
}

/// A sum's payload traced (C480): its head is kayak-rt's, holding none.
impl<T: crate::Trace> crate::Visit for SumOf<T> {
    fn visit<'a>(&'a self, t: &mut crate::Tracer<'a>) {
        self.payload.reach(t);
    }
}

impl<T: crate::Trace + 'static> SumOf<T> {
    /// A cell, for a `static`: `SumOf::new("eof", Held::UNIT)`, then
    /// [`SumOf::sum`].
    pub const fn new(name: &'static str, payload: crate::Held<T>) -> Self {
        SumOf {
            head: SumHead {
                tag: tag_hash(name),
                name,
                payload_type: TypeId::of::<T>,
                cell: cell_of::<T>,
            },
            payload,
        }
    }

    /// The sum a cell is.
    pub const fn sum(&'static self) -> Sum {
        Sum(&self.head)
    }
}

/// A sum: a reference to its cell, whatever its payload's type.
#[derive(Clone, Copy)]
pub struct Sum(&'static SumHead);

impl std::ops::Deref for Sum {
    type Target = SumHead;
    #[inline(always)]
    fn deref(&self) -> &SumHead {
        self.0
    }
}

impl Sum {
    /// Its head, its cell's first field: what a union holds (C491).
    #[inline]
    pub fn head(self) -> &'static SumHead {
        self.0
    }
    /// The sum whose head `h` is: made only in a cell, so a sum's.
    #[inline]
    pub fn of_head(h: &'static SumHead) -> Sum {
        Sum(h)
    }
    /// Where its cell is: its identity.
    #[inline]
    pub fn addr(self) -> usize {
        std::ptr::from_ref(self.0) as usize
    }
}

/// The key of the tags' hash: fixed, so that one tag has one discriminant in
/// every program and every build.
const K0: u64 = 0x6b61_7961_6b5f_7461; // "kayak_ta"
const K1: u64 = 0x6773_5f73_756d_7321; // "gs_sums!"

const fn rotl(x: u64, b: u32) -> u64 {
    x.rotate_left(b)
}

const fn sipround(v: [u64; 4]) -> [u64; 4] {
    let [mut v0, mut v1, mut v2, mut v3] = v;
    v0 = v0.wrapping_add(v1);
    v1 = rotl(v1, 13);
    v1 ^= v0;
    v0 = rotl(v0, 32);
    v2 = v2.wrapping_add(v3);
    v3 = rotl(v3, 16);
    v3 ^= v2;
    v0 = v0.wrapping_add(v3);
    v3 = rotl(v3, 21);
    v3 ^= v0;
    v2 = v2.wrapping_add(v1);
    v1 = rotl(v1, 17);
    v1 ^= v2;
    v2 = rotl(v2, 32);
    [v0, v1, v2, v3]
}

/// A tag's discriminant: SipHash-2-4 of its name's bytes, keyed by
/// [`K0`] and [`K1`]. A `const fn`, so that kfx's code writes
/// `const { tag_hash("circle") }` and rustc makes it as it builds.
pub const fn tag_hash(name: &str) -> u64 {
    let m = name.as_bytes();
    let n = m.len();
    let mut v = [
        K0 ^ 0x736f_6d65_7073_6575,
        K1 ^ 0x646f_7261_6e64_6f6d,
        K0 ^ 0x6c79_6765_6e65_7261,
        K1 ^ 0x7465_6462_7974_6573,
    ];
    let mut i = 0;
    while i + 8 <= n {
        let mut w = 0u64;
        let mut j = 0;
        while j < 8 {
            w |= (m[i + j] as u64) << (8 * j);
            j += 1;
        }
        v[3] ^= w;
        v = sipround(sipround(v));
        v[0] ^= w;
        i += 8;
    }
    let mut last = (n as u64 & 0xff) << 56;
    let mut j = 0;
    while i + j < n {
        last |= (m[i + j] as u64) << (8 * j);
        j += 1;
    }
    v[3] ^= last;
    v = sipround(sipround(v));
    v[0] ^= last;
    v[2] ^= 0xff;
    v = sipround(sipround(sipround(sipround(v))));
    v[0] ^ v[1] ^ v[2] ^ v[3]
}

const fn same_str(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// That no two of `names`, each a tag, are two names hashing alike: a
/// `const` assertion kfx writes of a program's tags and kayak-rt's, so that
/// a collision fails the build.
pub const fn tags_distinct(names: &[&str]) {
    let mut i = 0;
    while i < names.len() {
        let mut j = i + 1;
        while j < names.len() {
            if !same_str(names[i], names[j]) && tag_hash(names[i]) == tag_hash(names[j]) {
                panic!("two tags hash alike: rename one");
            }
            j += 1;
        }
        i += 1;
    }
}

/// That `names`, a program's tags, hash apart from each other and from
/// kayak-rt's own ([`RUNTIME_TAGS`]): the assertion kfx writes in each
/// program, `const _: () = tags_apart_from_runtime(&[...]);`.
pub const fn tags_apart_from_runtime(names: &[&str]) {
    tags_distinct(names);
    let mut i = 0;
    while i < names.len() {
        let mut j = 0;
        while j < RUNTIME_TAGS.len() {
            if !same_str(names[i], RUNTIME_TAGS[j])
                && tag_hash(names[i]) == tag_hash(RUNTIME_TAGS[j])
            {
                panic!("a tag hashes as kayak-rt's: rename it");
            }
            j += 1;
        }
        i += 1;
    }
}

/// The tags kayak-rt's own sums have: what FX-26's I/O and bytes primitives
/// answer with, which a program's tags must hash apart from.
pub const RUNTIME_TAGS: &[&str] = &[
    "text",
    "bad",
    "done",
    "failed",
    "line",
    "eof",
    "byte",
    "count",
    "not-a-terminal",
    "size",
    "data",
    "names",
    "info",
    "target",
    "file",
    "directory",
    "symlink",
    "other",
    "not-found",
    "permission-denied",
    "already-exists",
    "not-a-directory",
    "is-a-directory",
    "broken-pipe",
    "invalid-data",
    "started",
    "running",
    "exited",
    "signalled",
    "unknown",
    "node",
    "bound",
    "hole",
    "const",
    "sym",
    "int",
    "flag",
];

const _: () = tags_distinct(RUNTIME_TAGS);

/// A sum of tag `name`, its discriminant `tag` (`tag_hash(name)`), and
/// `payload`, on the heap.
pub fn sum_new<T: 'static + crate::Trace>(tag: u64, name: &'static str, payload: T) -> Sum {
    let cell: &'static SumOf<T> = crate::heap_alloc(SumOf {
        head: SumHead {
            tag,
            name,
            payload_type: TypeId::of::<T>,
            cell: cell_of::<T>,
        },
        payload: crate::Held::new(payload),
    })
    .to_ref();
    Sum(&cell.head)
}

/// The same, its discriminant found from its name: for kayak-rt's own sums.
pub fn sum_of<T: 'static + crate::Trace>(name: &'static str, payload: T) -> Sum {
    sum_new(tag_hash(name), name, payload)
}

/// A sum's tag's discriminant.
#[inline]
pub fn sum_tag(s: Sum) -> u64 {
    s.tag
}

/// A sum's tag's name.
#[inline]
pub fn sum_name(s: Sum) -> &'static str {
    s.name
}

/// A sum's payload, at the type `T` its tag's is. The caller says `T`: kfx
/// writes it from the type the checker found for the arm's tag, and a
/// sum's tag has one payload type in a checked program.
#[inline]
pub fn sum_payload<T: crate::Trace + 'static>(s: Sum) -> T {
    if (s.payload_type)() != TypeId::of::<T>() {
        crate::fail(&format!(
            "a sum's payload read at a type not its own: {}'s, as {}",
            s.name,
            std::any::type_name::<T>()
        ));
    }
    // SAFETY: `s` is the head of a `SumOf<T'>` (`sum_new`, `SumOf::new`),
    // and `T'` is `T`: its head says so, just checked. The head is the
    // cell's first field (`repr(C)`), so the cell's address is the head's.
    unsafe { (*(s.0 as *const SumHead as *const SumOf<T>)).payload.get() }
}

/// A sum as Fixpt writes one: `#<sum TAG>`.
impl Show for SumHead {
    fn show(&self, out: &mut String) {
        out.push_str("#<sum ");
        out.push_str(self.name);
        out.push('>');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SipHash-2-4's reference vector (the paper's appendix): key 00..0f,
    /// message 00..0e, as a check that the rounds are right; kayak's key
    /// is its own.
    #[test]
    fn siphash_reference() {
        const fn keyed(k0: u64, k1: u64, m: &[u8]) -> u64 {
            let n = m.len();
            let mut v = [
                k0 ^ 0x736f_6d65_7073_6575,
                k1 ^ 0x646f_7261_6e64_6f6d,
                k0 ^ 0x6c79_6765_6e65_7261,
                k1 ^ 0x7465_6462_7974_6573,
            ];
            let mut i = 0;
            while i + 8 <= n {
                let mut w = 0u64;
                let mut j = 0;
                while j < 8 {
                    w |= (m[i + j] as u64) << (8 * j);
                    j += 1;
                }
                v[3] ^= w;
                v = sipround(sipround(v));
                v[0] ^= w;
                i += 8;
            }
            let mut last = (n as u64 & 0xff) << 56;
            let mut j = 0;
            while i + j < n {
                last |= (m[i + j] as u64) << (8 * j);
                j += 1;
            }
            v[3] ^= last;
            v = sipround(sipround(v));
            v[0] ^= last;
            v[2] ^= 0xff;
            v = sipround(sipround(sipround(sipround(v))));
            v[0] ^ v[1] ^ v[2] ^ v[3]
        }
        let m: Vec<u8> = (0u8..15).collect();
        assert_eq!(
            keyed(0x0706050403020100, 0x0f0e0d0c0b0a0908, &m),
            0xa129ca6149be45e5
        );
    }

    #[test]
    fn a_sum_widened_is_the_same_sum() {
        let s = sum_of("circle", 3i64);
        assert_eq!(sum_tag(s), tag_hash("circle"));
        assert_eq!(sum_payload::<i64>(s), 3);
        assert_eq!(sum_name(s), "circle");
        static EOF: SumOf<()> = SumOf::new("eof", crate::Held::UNIT);
        assert_eq!(sum_tag(EOF.sum()), tag_hash("eof"));
    }

    #[test]
    #[should_panic(expected = "not its own")]
    fn a_payload_at_another_type_fails() {
        let s = sum_of("circle", 3i64);
        let _ = sum_payload::<u8>(s);
    }
}
