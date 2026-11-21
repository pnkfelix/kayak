//! FX-26's sums, structural (C409; the user: "Given that FX sums and
//! unions are structural ... can't you map keywords to unique u64, eg by
//! SipHashing them, and then manually assign those uid's as
//! discriminants?").
//!
//! A tag is the sum's, the same in every sum that has it, so every sum has
//! one representation (C499): Fixpt's sum object, kind 36, frozen, its
//! fields its tag, a word to Fixpt's symbol (kind 2: its name, its hash,
//! its global's place), and its payload, a word; then, before its trailer,
//! one field Fixpt's has not: its payload's type ([`SumDesc`]), what a
//! word of kayak's does not yet say of itself. A tag's discriminant is its
//! symbol's hash, Fixpt's (`symbol_hash_of`, [`tag_hash`]). A sum of fewer
//! tags is a sum of more as it is, and a sum is a union's member as it is:
//! no conversion, no new enum for each set of tags. A `tagcase` reads the
//! tag and, in the arm it chooses, the payload at the type the checker
//! found for that tag ([`sum_payload`]), which is the one place here that
//! is not safe Rust: what makes it sound is the checker, which gives each
//! tag of a sum one payload type, and the type the object keeps, which
//! every read checks.
//!
//! Two tags hashing alike would be two tags taken for one: kfx asserts, as
//! the program is built, that its tags and kayak-rt's hash apart
//! ([`tags_distinct`]), so that a collision is a build's failure, never a
//! wrong arm taken.

#![allow(unsafe_code)]

use crate::{HeapWord, Show, Str, StrLit};
use kayak_jit::layout;
use std::any::TypeId;
use std::ptr::NonNull;

/// Fixpt's kinds: a symbol's, a sum's.
const KIND_SYMBOL: u8 = 2;
const KIND_SUM: u8 = 36;

/// A tag (C499): Fixpt's symbol, kind 2, frozen: its name, its hash
/// (Fixpt's, [`tag_hash`]), its global's place, which kayak has none of
/// (−1). Its name a string's word, where Fixpt's is a text of its heap's
/// (`Value::text_numbered`). Made as rustc builds ([`tag_lit!`]), one of
/// each tag in each crate: a `tagcase` compares hashes, not symbols.
#[repr(C)]
pub struct SymObj {
    header: u64,
    name: HeapWord,
    hash: HeapWord,
    place: HeapWord,
    trailer: u64,
}

impl SymObj {
    /// Its fields as Fixpt counts them: three, and its trailer.
    const TO_TRAILER: usize = 4;
    /// From its suffix, where its word points, back to its hash: past
    /// its trailer and its place.
    const HASH_BACK: usize = 3;

    const fn of(name: HeapWord, text: &str) -> SymObj {
        SymObj {
            header: layout::header(KIND_SYMBOL, Self::TO_TRAILER, 0, true, true),
            name,
            hash: HeapWord::fixnum(tag_hash(text) as i64),
            place: HeapWord::fixnum(-1),
            trailer: layout::trailer(Self::TO_TRAILER),
        }
    }

    /// A tag named `text`, its name the literal `name` of it.
    pub const fn new<const W: usize>(name: &'static StrLit<W>, text: &str) -> SymObj {
        SymObj::of(HeapWord::of_str_lit(name), text)
    }

    /// Its word: its suffix's start, past its trailer, tagged.
    pub const fn word(&'static self) -> HeapWord {
        HeapWord::object_at(std::ptr::from_ref(self).cast::<u64>().wrapping_add(5))
    }

    /// Its name.
    pub fn name(&self) -> Str {
        self.name.to_str()
    }
}

/// A tag, its symbol made as rustc builds: `tag_lit!("circle")`.
#[macro_export]
macro_rules! tag_lit {
    ($s:expr) => {{
        const S: &str = $s;
        static N: $crate::StrLit<{ $crate::string_words(S.len()) }> = $crate::StrLit::new(S);
        static T: $crate::SymObj = $crate::SymObj::new(&N, S);
        &T
    }};
}

/// What a sum's object keeps of its payload's type (C499), its last
/// field: what it is, which every read of the payload checks (kfx's idea
/// of a primitive's sum and kayak-rt's are two, which rustc no longer
/// compares once sums are erased, so a disagreement fails loudly rather
/// than reads a payload at the wrong type), and its object as its type
/// has it, to trace (C491) and to be a union's member. A `static`'s
/// address, 8-aligned, so read as a word it is a fixnum: nothing a walk
/// of Fixpt's would follow. Gone when every word says what it is.
pub struct SumDesc {
    payload_type: fn() -> TypeId,
    cell: unsafe fn(NonNull<u64>) -> &'static dyn crate::value::Obj,
}

/// A head's cell, a `SumOf<T>`.
///
/// # Safety
/// `p` is the start of a `SumOf<T>` that lives for the run, and has its
/// provenance.
unsafe fn cell_of<T: crate::Trace + 'static>(p: NonNull<u64>) -> &'static dyn crate::value::Obj {
    // SAFETY: the caller's.
    unsafe { p.cast::<SumOf<T>>().as_ref() }
}

/// A payload type's [`SumDesc`]: one of each, a `static` rustc makes.
trait Described {
    const DESC: &'static SumDesc;
}

impl<T: crate::Trace + 'static> Described for T {
    const DESC: &'static SumDesc = &SumDesc {
        payload_type: TypeId::of::<T>,
        cell: cell_of::<T>,
    };
}

/// A sum's object: Fixpt's sum, its tag and its payload, then what it
/// keeps of its payload's type.
#[repr(C)]
pub struct SumOf<T: crate::Trace> {
    header: u64,
    tag: HeapWord,
    /// Held (C490): its type's word, as FX-26's sum holds one.
    payload: crate::Held<T>,
    desc: &'static SumDesc,
    trailer: u64,
}

/// A sum's payload traced (C480): its tag is a symbol of the image.
impl<T: crate::Trace + 'static> crate::Visit for SumOf<T> {
    fn visit<'a>(&'a self, t: &mut crate::Tracer<'a>) {
        self.payload.reach(t);
    }
}

impl<T: crate::Trace + 'static> SumOf<T> {
    /// Its fields as Fixpt counts them, its trailer one.
    const TO_TRAILER: usize = std::mem::offset_of!(SumOf<T>, trailer) / 8;

    /// An object, for a `static`: `SumOf::new(tag_lit!("eof"),
    /// Held::UNIT)`, then [`SumOf::sum`].
    pub const fn new(tag: &'static SymObj, payload: crate::Held<T>) -> Self {
        SumOf {
            header: layout::header(KIND_SUM, Self::TO_TRAILER, 0, true, true),
            tag: tag.word(),
            payload,
            desc: <T as Described>::DESC,
            trailer: layout::trailer(Self::TO_TRAILER),
        }
    }

    /// The sum an object is.
    pub fn sum(&'static self) -> Sum {
        Sum(NonNull::from(self).cast())
    }
}

/// A sum: where its object starts, whatever its payload's type; held as
/// Fixpt's word to it, tagged `100` at its suffix's start.
#[derive(Clone, Copy)]
pub struct Sum(NonNull<u64>);

// SAFETY: a sum's object is frozen, never written once made, nor freed.
unsafe impl Send for Sum {}
// SAFETY: as above.
unsafe impl Sync for Sum {}

impl Sum {
    /// Its object's `i`th word.
    #[inline(always)]
    fn at(self, i: usize) -> u64 {
        // SAFETY: a sum is made only of an object that lives for the run
        // (`SumOf::sum`), its provenance its whole object's;
        // `i` is within it, its field count said by its header.
        unsafe { *self.0.as_ptr().add(i) }
    }

    /// Its fields, its trailer one, as its header says.
    #[inline(always)]
    fn to_trailer(self) -> usize {
        ((self.at(0) >> 14) & 0x3ffff) as usize
    }

    /// Its tag's symbol.
    #[inline(always)]
    fn tag_obj(self) -> &'static SymObj {
        // SAFETY: field 1 a symbol's word (`SumOf::new`), of a `static`
        // (`tag_lit!`), five words past its start.
        unsafe {
            let w = *self.0.as_ptr().add(1).cast::<HeapWord>();
            &*w.object_ptr().sub(5).cast::<SymObj>()
        }
    }

    /// What it keeps of its payload's type: its last field.
    #[inline(always)]
    fn desc(self) -> &'static SumDesc {
        // SAFETY: as `at`; the field before its trailer its desc.
        unsafe {
            *self
                .0
                .as_ptr()
                .add(self.to_trailer() - 1)
                .cast::<&'static SumDesc>()
        }
    }

    /// Its object, as its type has it: what a trace reaches (C491).
    #[inline]
    pub fn object(self) -> &'static dyn crate::value::Obj {
        // SAFETY: its desc is its object's type's (`SumOf::new`).
        unsafe { (self.desc().cell)(self.0) }
    }

    /// Where its object is: its identity.
    #[inline]
    pub fn addr(self) -> usize {
        self.0.as_ptr() as usize
    }

    /// Its word: its suffix's start, past its trailer, tagged.
    #[inline(always)]
    pub fn word(self) -> HeapWord {
        HeapWord::object_at(self.0.as_ptr().wrapping_add(self.to_trailer() + 1))
    }

    /// The sum a word is.
    #[inline(always)]
    pub fn of_word(w: HeapWord) -> Sum {
        let at = w.object_ptr();
        // SAFETY: a sum's word is made only by `word`, of an object that
        // lives for the run, whose trailer, just before where the word
        // points, says how far back its header is.
        unsafe {
            let back = ((*at.sub(1) >> 3) & 0x3ffff) as usize;
            Sum(NonNull::new_unchecked(at.sub(1 + back).cast_mut()))
        }
    }
}

/// A sum: its object reached, precisely (C491); held as its word (C499).
impl crate::Trace for Sum {
    type Stored = HeapWord;
    #[inline(always)]
    fn store(self) -> HeapWord {
        self.word()
    }
    #[inline(always)]
    fn load(w: HeapWord) -> Self {
        Sum::of_word(w)
    }
    #[inline]
    fn trace<'a>(self, t: &mut crate::Tracer<'a>)
    where
        Self: 'a,
    {
        let o: &'static dyn crate::Visit = self.object();
        t.reach(o);
    }
}

/// Fixpt's `symbol_hash_of`'s FNV-1a, its offset basis and prime.
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// A tag's discriminant (C499): its symbol's hash, Fixpt's
/// (`symbol_hash_of`), FNV-1a of its name's bytes, its sign bit cleared,
/// kept to a fixnum's 61 bits. A `const fn`, so that kfx's code writes
/// `const { tag_hash("circle") }` and rustc makes it as it builds.
pub const fn tag_hash(name: &str) -> u64 {
    let m = name.as_bytes();
    let mut h = FNV_OFFSET;
    let mut i = 0;
    while i < m.len() {
        h = (h ^ m[i] as u64).wrapping_mul(FNV_PRIME);
        i += 1;
    }
    let n = (h as i64) & i64::MAX;
    ((n << 3) >> 3) as u64
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

/// A sum of tag `tag` and `payload`, on the heap: what kfx writes, its
/// tag made as rustc builds (`tag_lit!`).
#[inline]
pub fn sum_tagged<T: 'static + crate::Trace>(tag: &'static SymObj, payload: T) -> Sum {
    crate::heap_alloc(SumOf::new(tag, crate::Held::new(payload)))
        .to_ref()
        .sum()
}

/// The same, for kayak-rt's own sums: `sum_of(tag_lit!("eof"), ...)`.
#[inline]
pub fn sum_of<T: 'static + crate::Trace>(tag: &'static SymObj, payload: T) -> Sum {
    sum_tagged(tag, payload)
}

/// A sum's tag's discriminant: its symbol's hash.
#[inline]
pub fn sum_tag(s: Sum) -> u64 {
    // SAFETY: a symbol's hash, `HASH_BACK` words before its suffix; a
    // fixnum, its value the word shifted right three.
    let w = unsafe {
        *s.tag_obj()
            .word()
            .object_ptr()
            .sub(SymObj::HASH_BACK)
            .cast::<HeapWord>()
    };
    ((w.raw() as i64) >> 3) as u64
}

/// A sum's tag's name.
#[inline]
pub fn sum_name(s: Sum) -> &'static str {
    s.tag_obj().name().as_str()
}

/// A sum's payload, at the type `T` its tag's is. The caller says `T`: kfx
/// writes it from the type the checker found for the arm's tag, and a
/// sum's tag has one payload type in a checked program.
#[inline]
pub fn sum_payload<T: crate::Trace + 'static>(s: Sum) -> T {
    let d = s.desc();
    if !std::ptr::eq(d, <T as Described>::DESC) && (d.payload_type)() != TypeId::of::<T>() {
        crate::fail(&format!(
            "a sum's payload read at a type not its own: {}'s, as {}",
            sum_name(s),
            std::any::type_name::<T>()
        ));
    }
    // SAFETY: `s` is the start of a `SumOf<T'>` (`SumOf::sum`), and `T'`
    // is `T`: its desc says so, just checked.
    unsafe { s.0.cast::<SumOf<T>>().as_ref().payload.get() }
}

/// A sum as Fixpt writes one: `#<sum TAG>`.
impl Show for Sum {
    fn show(&self, out: &mut String) {
        out.push_str("#<sum ");
        out.push_str(sum_name(*self));
        out.push('>');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fixpt's hash of a symbol (`symbol_hash_of`), worked by hand: FNV-1a
    /// of "a", its sign bit cleared, as a fixnum.
    #[test]
    fn a_tag_hashes_as_fixpt_symbol() {
        let h = (0xcbf2_9ce4_8422_2325u64 ^ b'a' as u64).wrapping_mul(0x0000_0100_0000_01b3);
        let n = (h as i64) & i64::MAX;
        assert_eq!(tag_hash("a"), ((n << 3) >> 3) as u64);
        assert_eq!(
            crate::int_to_i64(crate::symbol_name_hash(crate::string_to_symbol(
                crate::str_lit!("circle")
            ))),
            tag_hash("circle") as i64
        );
    }

    #[test]
    fn a_sum_widened_is_the_same_sum() {
        let s = sum_of(crate::tag_lit!("circle"), 3i64);
        assert_eq!(sum_tag(s), tag_hash("circle"));
        assert_eq!(sum_payload::<i64>(s), 3);
        assert_eq!(sum_name(s), "circle");
        static EOF: SumOf<()> = SumOf::new(crate::tag_lit!("eof"), crate::Held::UNIT);
        assert_eq!(sum_tag(EOF.sum()), tag_hash("eof"));
        // A tag made in another place is the same tag.
        let r = sum_of(crate::tag_lit!("circle"), 4i64);
        assert_eq!(sum_tag(r), sum_tag(s));
        assert_eq!(sum_name(r), "circle");
    }

    /// Fixpt's sum (C499): a word tagged `100` at its object's suffix,
    /// its header kind 36, frozen, three fields and its trailer; its tag a
    /// word to Fixpt's symbol, kind 2; its payload a word.
    #[test]
    fn a_sum_fixpt_sum_object() {
        let s = sum_of(crate::tag_lit!("circle"), 3i64);
        assert_eq!(std::mem::size_of::<Sum>(), 8);
        let w = s.word();
        assert_eq!(w.raw() & 7, 0b100);
        let h = s.at(0);
        assert_eq!(
            (
                h & 7,
                (h >> 3) & 0xff,
                (h >> 12) & 3,
                (h >> 14) & 0x3ffff,
                h >> 32
            ),
            (0b110, 36, 0b11, 4, 0)
        );
        assert_eq!(s.at(2), HeapWord::fixnum(3).raw() as u64);
        assert_eq!(s.at(4) & 7, 0b101);
        assert_eq!(Sum::of_word(w).addr(), s.addr());
        let sym = s.tag_obj();
        assert_eq!((sym.header >> 3) & 0xff, 2);
        assert_eq!(s.at(1) as usize, sym.word().raw());
        assert_eq!(sym.name().as_str(), "circle");
        // In a pair, its word.
        let xs = crate::heap_cons(s, crate::nil());
        assert_eq!(sum_name(crate::car(xs)), "circle");
    }

    #[test]
    #[should_panic(expected = "not its own")]
    fn a_payload_at_another_type_fails() {
        let s = sum_of(crate::tag_lit!("circle"), 3i64);
        let _ = sum_payload::<u8>(s);
    }
}
