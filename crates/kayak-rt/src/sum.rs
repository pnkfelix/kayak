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
    /// Never zero: a heap arena's cell of one no wider for it (C504).
    header: std::num::NonZeroU64,
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
            header: match std::num::NonZeroU64::new(layout::header(
                KIND_SYMBOL,
                Self::TO_TRAILER,
                0,
                true,
                true,
            )) {
                Some(h) => h,
                None => panic!("a symbol's header zero"),
            },
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

/// A symbol's object: its name's text reached.
impl crate::Visit for SymObj {
    fn visit<'a>(&'a self, t: &mut crate::Tracer<'a>) {
        crate::Trace::trace(self.name(), t);
    }
}

/// `symbol` (C504): one word, to Fixpt's symbol (kind 2, frozen: its
/// name, its hash, its place, −1), as a tag's (C499). A quoted one made as
/// rustc builds (`sym_lit!`), one of each in each crate; `string->symbol`'s
/// made in the heap, one each time. So not one of each name, as Fixpt's
/// interning has it: `eq?` of two compares their objects, then their
/// hashes and names, which is what interning makes it (and what kayak's
/// was, a name's text, C467).
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Symbol(NonNull<SymObj>);

// SAFETY: its object is frozen, never written once made, kept for the run.
unsafe impl Send for Symbol {}
// SAFETY: as above.
unsafe impl Sync for Symbol {}

impl Symbol {
    /// The symbol whose object is `o` (a quoted one's, `sym_lit!`).
    #[inline(always)]
    pub const fn of_obj(o: &'static SymObj) -> Symbol {
        Symbol(NonNull::from_ref(o))
    }

    /// One named `name`, made in the heap.
    pub fn new(name: Str) -> Symbol {
        let o = crate::heap_alloc(SymObj::of(HeapWord::of_str(name), name.as_str()));
        Symbol::of_obj(o.to_ref())
    }

    #[inline(always)]
    fn obj(self) -> &'static SymObj {
        // SAFETY: made only of a `&'static SymObj`.
        unsafe { self.0.as_ref() }
    }

    /// Its name.
    #[inline]
    pub fn name(self) -> Str {
        self.obj().name()
    }

    /// Its hash, Fixpt's (`symbol_hash_of`), a fixnum's 61 bits.
    #[inline]
    pub fn hash(self) -> u64 {
        self.obj().hash.to_i64() as u64
    }

    /// Its word: Fixpt's, tagged `100`, past its trailer.
    #[inline(always)]
    pub fn word(self) -> HeapWord {
        HeapWord::object_at(self.0.as_ptr().cast::<u64>().wrapping_add(5))
    }

    /// The symbol a word is.
    #[inline(always)]
    pub fn of_word(w: HeapWord) -> Symbol {
        match NonNull::new(w.object_ptr().wrapping_sub(5).cast::<SymObj>().cast_mut()) {
            Some(p) => Symbol(p),
            None => crate::fail("a heap's word read as a symbol, which it is not"),
        }
    }
}

/// `eq?` of symbols: the same object, or the same hash and name.
impl PartialEq for Symbol {
    #[inline]
    fn eq(&self, other: &Symbol) -> bool {
        self.0 == other.0
            || (self.obj().hash.raw() == other.obj().hash.raw()
                && self.name().as_str() == other.name().as_str())
    }
}

impl Eq for Symbol {}

/// A symbol: its object reached (a quoted one's too, rustc's); held as its
/// word.
impl crate::Trace for Symbol {
    type Stored = HeapWord;
    #[inline(always)]
    fn store(self) -> HeapWord {
        self.word()
    }
    #[inline(always)]
    fn load(w: HeapWord) -> Self {
        Symbol::of_word(w)
    }
    #[inline]
    fn trace<'a>(self, t: &mut crate::Tracer<'a>)
    where
        Self: 'a,
    {
        t.reach(self.obj());
    }
}

/// A quoted symbol, its object made as rustc builds: `sym_lit!("name")`.
#[macro_export]
macro_rules! sym_lit {
    ($s:expr) => {
        $crate::Symbol::of_obj($crate::tag_lit!($s))
    };
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

/// FX-26's datatypes (C500): each value Fixpt's sum of a product. A
/// module of its own, so that a program's glob of kayak-rt's names meets
/// none of a program's own (a datatype `variant`'s enum, `Variant`).
pub mod data {
    use super::{HeapWord, KIND_SUM, SymObj, layout};

    /// A datatype's value of a tag none of its variants have: its word
    /// not of its type, which no checked program makes.
    #[cold]
    pub fn no_variant() -> ! {
        crate::fail("a datatype's value of a tag not its own")
    }

    #[cold]
    fn not_its_variant() -> ! {
        crate::fail("a datatype's value read as a variant not its own")
    }

    /// Fixpt's product's kind (37): a datatype's value's fields.
    const KIND_PRODUCT: u8 = 37;

    /// A datatype's variant `I` (C500): the types of its fields, a tuple, and
    /// its tag, a symbol of its own (`tag_lit!` in the one `tag` of the
    /// impl), so that an object made of a variant is read again only as that
    /// variant: what makes [`DataAt::fields`] sound is that the tag it finds
    /// is the very symbol the variant's impl made it with, and so the fields
    /// are of the types they were made of. kfx writes one impl per variant.
    pub trait Variant<const I: usize>: Sized {
        type Fields: crate::Captures;
        fn tag() -> &'static SymObj;
    }

    /// A datatype (C500): its enum, the value taken apart (what a `tagcase`
    /// matches); in the heap, the objects [`DataAt`] points to. kfx writes it,
    /// each method by the variants' tags.
    pub trait Datatype: Sized + Copy {
        /// The value a word is, its variant's fields read.
        fn read(w: DataAt<'_, Self>) -> Self;
        /// What its variant's object reaches, the object reached.
        fn reach<'a>(w: DataAt<'a, Self>, t: &mut crate::Tracer<'a>)
        where
            Self: 'a;
        /// Its variant's object, as a union holds it.
        fn object(w: DataAt<'static, Self>) -> &'static dyn crate::value::Obj
        where
            Self: 'static;
        /// The value whose variant's object `o` is, if it is one of this
        /// datatype's.
        fn of_object(o: &'static dyn crate::value::Obj) -> Option<DataAt<'static, Self>>
        where
            Self: 'static;
    }

    /// A datatype's value's objects (C500), as Fixpt's `define-datatype`
    /// makes one (`expand_datatype`: a sum of products): Fixpt's product
    /// (kind 37, frozen), its variant's fields' words; then Fixpt's sum (kind
    /// 36, frozen), its tag and a word to the product. Made as one, so one
    /// allocation of kayak's makes the two.
    #[repr(C)]
    pub struct DataObj<E: crate::Captures> {
        /// Never zero (its tag `110`): so that an arena's `OnceCell` of
        /// one needs no word of its own to say whether it is filled.
        pub(super) p_header: std::num::NonZeroU64,
        words: E::Words,
        p_trailer: u64,
        pub(super) s_header: u64,
        tag: HeapWord,
        /// The product's word, its suffix's start (the sum's header): written
        /// once the object is where it stays.
        pub(super) payload: std::cell::Cell<HeapWord>,
        s_trailer: u64,
    }

    // SAFETY: its one cell is written once, as it is made, before any word to
    // it is; then the whole object is frozen.
    unsafe impl<E: crate::Captures> Sync for DataObj<E> where E::Words: Sync {}

    impl<E: crate::Captures> DataObj<E> {
        /// The product's fields as Fixpt counts them, its trailer one.
        const P_TO_TRAILER: usize = std::mem::offset_of!(DataObj<E>, p_trailer) / 8;
        /// From the object's start to the sum's suffix, where its word points.
        const TO_END: usize = std::mem::size_of::<DataObj<E>>();

        fn new(tag: &'static SymObj, e: E) -> Self {
            DataObj {
                p_header: match std::num::NonZeroU64::new(layout::header(
                    KIND_PRODUCT,
                    Self::P_TO_TRAILER,
                    0,
                    true,
                    true,
                )) {
                    Some(h) => h,
                    None => unreachable!(),
                },
                words: e.words(),
                p_trailer: layout::trailer(Self::P_TO_TRAILER),
                s_header: layout::header(KIND_SUM, 3, 0, true, true),
                tag: tag.word(),
                payload: std::cell::Cell::new(HeapWord::UNIT),
                s_trailer: layout::trailer(3),
            }
        }

        /// The word to it, its payload written, once it is where it stays.
        fn placed<'r, D>(&'r self) -> DataAt<'r, D> {
            let start = std::ptr::from_ref(self).cast::<u8>();
            let at = |n: usize| start.wrapping_add(n).cast::<u64>();
            self.payload
                .set(HeapWord::object_at(at(std::mem::offset_of!(
                    DataObj<E>,
                    s_header
                ))));
            DataAt(
                HeapWord::object_at(at(Self::TO_END)),
                std::marker::PhantomData,
            )
        }

        /// Its fields.
        #[inline(always)]
        pub fn fields(&self) -> E {
            E::of_words(self.words)
        }
    }

    /// A datatype's variant's object: its fields traced.
    impl<E: crate::Captures> crate::Visit for DataObj<E> {
        fn visit<'a>(&'a self, t: &mut crate::Tracer<'a>) {
            crate::Trace::trace(self.fields(), t);
        }
    }

    /// A datatype's value (C500): one Fixpt word, to its sum's suffix,
    /// tagged `100`; in a `letrena`'s place `DataAt<'r, D>`, in the heap
    /// `DataAt<'static, D>`. Its word made only of its objects (`placed`).
    #[repr(transparent)]
    pub struct DataAt<'r, D>(HeapWord, std::marker::PhantomData<&'r D>);

    impl<D> Clone for DataAt<'_, D> {
        #[inline(always)]
        fn clone(&self) -> Self {
            *self
        }
    }

    impl<D> Copy for DataAt<'_, D> {}

    impl<'r, D> DataAt<'r, D> {
        /// The sum's `i`th word back from its suffix.
        #[inline(always)]
        pub(super) fn back(self, i: usize) -> u64 {
            // SAFETY: its word is made only by `placed`, one past the end of a
            // datatype's object that lives for `'r`, whose last words are its
            // sum's: its header, tag, payload and trailer.
            unsafe { *self.0.object_ptr().sub(i) }
        }

        /// Its tag's word's bits.
        #[inline(always)]
        fn tag_word(self) -> usize {
            self.back(3) as usize
        }

        /// Its tag's discriminant: its symbol's hash.
        #[inline]
        pub fn tag_hash(self) -> u64 {
            // SAFETY: as `back`; its tag a symbol's word (`DataObj::new`), of
            // a `static`, its hash `HASH_BACK` words before its suffix.
            let h = unsafe {
                let tag = *self.0.object_ptr().sub(3).cast::<HeapWord>();
                *tag.object_ptr().sub(SymObj::HASH_BACK)
            };
            ((h as i64) >> 3) as u64
        }

        /// Whether it is of variant `I`: its tag that variant's symbol.
        #[inline(always)]
        pub fn is<const I: usize>(self) -> bool
        where
            D: Variant<I>,
        {
            self.tag_word() == <D as Variant<I>>::tag().word().raw()
        }

        /// Its object, as variant `I`'s: that variant's, or a failure.
        #[inline(always)]
        pub fn obj<const I: usize>(self) -> &'r DataObj<<D as Variant<I>>::Fields>
        where
            D: Variant<I>,
        {
            if !self.is::<I>() {
                not_its_variant();
            }
            let end = self.0.object_ptr().cast::<u8>();
            // SAFETY: its tag is variant `I`'s own symbol, made only into an
            // object of `I`'s fields (`DataObj::new`, by `data_heap`, ...);
            // its word one past that object's end, of its provenance.
            unsafe {
                &*end
                    .sub(DataObj::<<D as Variant<I>>::Fields>::TO_END)
                    .cast::<DataObj<<D as Variant<I>>::Fields>>()
            }
        }

        /// Its fields, as variant `I`'s.
        #[inline(always)]
        pub fn fields<const I: usize>(self) -> <D as Variant<I>>::Fields
        where
            D: Variant<I>,
        {
            self.obj::<I>().fields()
        }

        /// Its word.
        #[inline(always)]
        pub fn word(self) -> HeapWord {
            self.0
        }

        /// Whether it is the same value as `other` (`eq?`).
        #[inline(always)]
        pub fn same(self, other: Self) -> bool {
            self.0.raw() == other.0.raw()
        }
    }

    impl<D: Datatype> DataAt<'_, D> {
        /// The value, taken apart.
        #[inline]
        pub fn get(self) -> D {
            D::read(self)
        }
    }

    /// A datatype's value: its object reached; held as its word.
    impl<'r, D: Datatype + 'r> crate::Trace for DataAt<'r, D> {
        type Stored = HeapWord;
        #[inline(always)]
        fn store(self) -> HeapWord {
            self.0
        }
        #[inline(always)]
        fn load(w: HeapWord) -> Self {
            DataAt(w, std::marker::PhantomData)
        }
        #[inline]
        fn trace<'a>(self, t: &mut crate::Tracer<'a>)
        where
            Self: 'a,
        {
            D::reach(self, t);
        }
    }

    /// A datatype's value as a union's member: its variant's object.
    impl<D: Datatype + 'static> crate::value::Member for DataAt<'static, D> {
        #[inline]
        fn object(self) -> &'static dyn crate::value::Obj {
            D::object(self)
        }
        #[inline]
        fn of_object(o: &'static dyn crate::value::Obj) -> Option<Self> {
            D::of_object(o)
        }
    }

    /// `o` as variant `I`'s object of `D`, if it is one: its type that
    /// variant's, its tag that variant's symbol (two datatypes' variants may
    /// have one type of fields).
    pub fn data_of_object<D: Variant<I> + 'static, const I: usize>(
        o: &'static dyn crate::value::Obj,
    ) -> Option<DataAt<'static, D>>
    where
        <D as Variant<I>>::Fields: 'static,
    {
        let x = (o as &dyn std::any::Any).downcast_ref::<DataObj<<D as Variant<I>>::Fields>>()?;
        let w = DataAt::<D>(
            HeapWord::object_at(
                std::ptr::from_ref(x)
                    .cast::<u8>()
                    .wrapping_add(DataObj::<<D as Variant<I>>::Fields>::TO_END)
                    .cast(),
            ),
            std::marker::PhantomData,
        );
        w.is::<I>().then_some(w)
    }

    /// Variant `I` of `D`, of `fields`, in the heap.
    #[inline]
    pub fn data_heap<D: Variant<I> + 'static, const I: usize>(
        fields: <D as Variant<I>>::Fields,
    ) -> DataAt<'static, D>
    where
        <D as Variant<I>>::Fields: 'static,
    {
        crate::heap_alloc(DataObj::new(D::tag(), fields))
            .to_ref()
            .placed()
    }

    /// The same, at a region's lifetime, kept for the run (`alloc`'s way).
    #[inline]
    pub fn data_alloc<'r, D: Variant<I> + 'r, const I: usize>(
        fields: <D as Variant<I>>::Fields,
    ) -> DataAt<'r, D>
    where
        <D as Variant<I>>::Fields: 'r,
    {
        crate::alloc(DataObj::new(D::tag(), fields))
            .to_ref()
            .placed()
    }

    /// What a place keeps a datatype's value in (C500): its variant's
    /// object, boxed, any variant's of any datatype; freed with the
    /// place. Its type no lifetime's, so that a place's arenas need not
    /// outlive the place (a box of a lifetime's object might, Rust's drop
    /// check supposes, read it as it is freed; this one's never does).
    pub struct DataBox(Box<dyn crate::Visit + 'static>);

    /// A place's datatype's value: its object visited.
    impl crate::Visit for DataBox {
        fn visit<'a>(&'a self, t: &mut crate::Tracer<'a>) {
            self.0.visit(t);
        }
        fn visit_size(&self) -> usize {
            self.0.visit_size()
        }
        fn visit_name(&self) -> &'static str {
            self.0.visit_name()
        }
    }

    /// Variant `I` of `D`, of `fields`, in the place `p`.
    pub fn data_in<'r, D: Variant<I> + 'r, const I: usize, P: crate::Holds<DataBox>>(
        p: &'r P,
        fields: <D as Variant<I>>::Fields,
    ) -> DataAt<'r, D>
    where
        <D as Variant<I>>::Fields: 'r,
    {
        type Obj<D, const I: usize> = DataObj<<D as Variant<I>>::Fields>;
        const { assert!(!std::mem::needs_drop::<Obj<D, I>>()) };
        let b = Box::new(DataObj::new(D::tag(), fields));
        let at: *const Obj<D, I> = &*b;
        let b: Box<dyn crate::Visit + 'r> = b;
        // SAFETY: only its lifetime's bound changed, not its layout. The
        // box is freed with the place's arena, borrowed for `'r`, and its
        // object visited only while the arena is (a trace of `'a`, within
        // `'r`); freeing it runs no code (asserted), so reads nothing.
        let b: Box<dyn crate::Visit + 'static> = unsafe { std::mem::transmute(b) };
        p.arena().alloc(DataBox(b));
        // SAFETY: the box's contents never move, and live while the arena
        // is borrowed, `'r`.
        unsafe { &*at }.placed()
    }
}

#[cfg(test)]
mod tests {
    use super::data::*;
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
        assert_eq!((sym.header.get() >> 3) & 0xff, 2);
        assert_eq!(s.at(1) as usize, sym.word().raw());
        assert_eq!(sym.name().as_str(), "circle");
        // In a pair, its word.
        let xs = crate::heap_cons(s, crate::nil());
        assert_eq!(sum_name(crate::car(xs)), "circle");
    }

    /// A datatype as kfx writes one (C500): `(define-datatype tree
    /// (leaf) (node tree i64 tree))`.
    #[derive(Clone, Copy)]
    enum Tree {
        Leaf(),
        Node(DataAt<'static, Tree>, i64, DataAt<'static, Tree>),
    }

    impl Variant<0> for Tree {
        type Fields = ();
        fn tag() -> &'static SymObj {
            crate::tag_lit!("leaf")
        }
    }

    impl Variant<1> for Tree {
        type Fields = (DataAt<'static, Tree>, i64, DataAt<'static, Tree>);
        fn tag() -> &'static SymObj {
            crate::tag_lit!("node")
        }
    }

    impl Datatype for Tree {
        fn read(w: DataAt<'_, Self>) -> Self {
            let h = w.tag_hash();
            if h == const { tag_hash("leaf") } {
                let () = w.fields::<0>();
                Self::Leaf()
            } else if h == const { tag_hash("node") } {
                let (f0, f1, f2) = w.fields::<1>();
                Self::Node(f0, f1, f2)
            } else {
                crate::fail("a datatype's value of a tag not its own")
            }
        }
        fn reach<'a>(w: DataAt<'a, Self>, t: &mut crate::Tracer<'a>)
        where
            Self: 'a,
        {
            if w.is::<0>() {
                t.reach(w.obj::<0>())
            } else {
                t.reach(w.obj::<1>())
            }
        }
        fn object(w: DataAt<'static, Self>) -> &'static dyn crate::value::Obj {
            if w.is::<0>() {
                w.obj::<0>()
            } else {
                w.obj::<1>()
            }
        }
        fn of_object(o: &'static dyn crate::value::Obj) -> Option<DataAt<'static, Self>> {
            data_of_object::<Self, 0>(o).or_else(|| data_of_object::<Self, 1>(o))
        }
    }

    fn sum_tree(t: DataAt<'static, Tree>) -> i64 {
        match t.get() {
            Tree::Leaf() => 0,
            Tree::Node(l, n, r) => sum_tree(l) + n + sum_tree(r),
        }
    }

    /// Fixpt's datatype value (C500): a word to Fixpt's sum (kind 36, its
    /// tag, its payload a word to Fixpt's product, kind 37, of its fields).
    #[test]
    fn a_datatype_value_fixpt_sum_of_product() {
        let leaf = || data_heap::<Tree, 0>(());
        let t = data_heap::<Tree, 1>((leaf(), 5, data_heap::<Tree, 1>((leaf(), 7, leaf()))));
        assert_eq!(std::mem::size_of::<DataAt<'static, Tree>>(), 8);
        assert_eq!(sum_tree(t), 12);
        assert_eq!(t.word().raw() & 7, 0b100);
        let sh = t.back(4);
        assert_eq!(((sh >> 3) & 0xff, (sh >> 14) & 0x3ffff), (36, 3));
        assert_eq!(t.tag_hash(), tag_hash("node"));
        // Its payload, a word to the product, whose header is kind 37,
        // three fields and its trailer.
        let o = t.obj::<1>();
        assert_eq!(o.payload.get().raw() & 7, 0b100);
        assert_eq!(
            o.payload.get().raw(),
            std::ptr::from_ref(&o.s_header) as usize | 4
        );
        let ph = o.p_header.get();
        assert_eq!(((ph >> 3) & 0xff, (ph >> 14) & 0x3ffff), (37, 4));
        // An arena's cell of one no larger than it (its header a niche).
        assert_eq!(
            std::mem::size_of::<std::cell::OnceCell<DataObj<((), i64)>>>(),
            std::mem::size_of::<DataObj<((), i64)>>()
        );
        assert!(!t.same(leaf()));
        // In a union, its object; found again.
        let v = crate::value::other(t, crate::value::Shape::Product);
        assert_eq!(
            sum_tree(crate::value::unbox_other::<DataAt<'static, Tree>>(v)),
            12
        );
        // Traced: three nodes, four leaves.
        let mut tr = crate::traced(t);
        assert!(tr.take_opaque().is_empty());
    }

    #[test]
    #[should_panic(expected = "not its own")]
    fn a_datatype_value_read_as_another_variant_fails() {
        let t = data_heap::<Tree, 0>(());
        let _ = t.fields::<1>();
    }

    #[test]
    #[should_panic(expected = "not its own")]
    fn a_payload_at_another_type_fails() {
        let s = sum_of(crate::tag_lit!("circle"), 3i64);
        let _ = sum_payload::<u8>(s);
    }
}

#[cfg(test)]
mod symbol_tests {
    use super::*;

    #[test]
    fn a_symbol_one_word_to_fixpt_symbol() {
        assert_eq!(std::mem::size_of::<Symbol>(), 8);
        assert_eq!(std::mem::size_of::<Option<Symbol>>(), 8);
        let quoted = crate::sym_lit!("lambda");
        let made = crate::string_to_symbol(crate::str_lit!("lambda"));
        let other = crate::string_to_symbol(crate::str_lit!("lambdb"));
        // `eq?` as interning makes it: by name, whoever made it.
        assert!(quoted == made && made == quoted && quoted != other);
        assert_eq!(made.hash(), tag_hash("lambda"));
        assert_eq!(crate::symbol_to_string(made).as_str(), "lambda");
        // Its word Fixpt's, to its object's suffix (none), kind 2.
        use crate::Trace;
        let w = made.store();
        assert_eq!(w.raw() & 7, 4);
        // SAFETY: its header, 5 words before where its word points.
        let h = unsafe { *w.object_ptr().sub(5) };
        assert_eq!((h >> 3) & 0xff, 2);
        assert!(Symbol::load(w) == made);
        assert!(Symbol::load(quoted.store()) == quoted);
    }
}
