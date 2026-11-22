//! What of the heap a value reaches (C479, C481), precisely, as a collector
//! will need it (`TODO.md` §54; Bartlett's mostly-copying: the Rust stack's
//! words conservative, the heap's values precise).
//!
//! Two traits, as FX-26's heap grammar has two things (C481):
//! - **`Trace`**, of a value (every KFX value is `Copy`): traced by value,
//!   so a cell's copy traces what it reaches; and held in the heap as its
//!   type's `Stored`, a Fixpt word (C495, C496), as Fixpt's heap holds
//!   every value: a closure's captures, a product's parts, a pair's
//!   element, each a word.
//! - **`Visit`**, of what a handle points to (a datatype's value, a pair, a
//!   cell, a sum's cell, a closure): visited by reference, once, and
//!   object-safe, so a closure behind `dyn` is visited too.
//!
//! A lifetime is the method's, not the trait's: a handle's `'r` outlives
//! the trace's `'a`, as the value it is in does.
//!
//! A tracer keeps its own list of what is still to visit, so that a long
//! list is traced in a loop, not a recursion as deep as the list.

use crate::{Array, BigAt, Cons, Handle, Int, Str, Void};
use std::cell::Cell;
use std::collections::{HashMap, HashSet};

/// A value: what of the heap it reaches, told to the tracer; and how a
/// heap object holds it.
pub trait Trace: Copy {
    fn trace<'a>(self, t: &mut Tracer<'a>)
    where
        Self: 'a;
    /// How a heap object holds it (C495): a Fixpt word (`Word`: a fixnum,
    /// an immediate, a tagged pointer), as Fixpt's heap holds every value;
    /// what has no object of Fixpt's layout yet, itself.
    type Stored: Copy;
    fn store(self) -> Self::Stored;
    fn load(s: Self::Stored) -> Self;
    /// Whether its stored word is one a dynamic value (`Value`, C506) holds
    /// as it is: Fixpt's own word for it, its shape its tag's or its
    /// object's kind's. So a list of it is a list of values as it is.
    const VALUE_WORD: bool = false;
    /// Whether it is held as itself, no Fixpt word (`trace_whole!`'s,
    /// `trace_bytes!`'s): what no heap object may hold (C508), so that an
    /// object's fields are each a word its header says, traced by its
    /// layout alone. A heap object's maker asserts it, as rustc builds.
    const WHOLE: bool = false;
}

/// What a handle points to: visited by reference, once.
pub trait Visit {
    fn visit<'a>(&'a self, t: &mut Tracer<'a>);
    /// Its type's name, for a census.
    fn visit_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    /// Its bytes, for a census.
    fn visit_size(&self) -> usize {
        std::mem::size_of_val(self)
    }
    /// It visited where it is a one-word value, kept by the tracer only
    /// while visited (an array): what it reaches lives as long as the
    /// trace, not as the value. Visited by reference unless it says so.
    fn visit_owned(&self, _: &mut Tracer<'_>) {}
}

/// What a trace has visited, and what it still must, each alive for `'a`.
#[derive(Default)]
pub struct Tracer<'a> {
    seen: HashSet<usize>,
    pending: Vec<&'a (dyn Visit + 'a)>,
    /// What is still to visit of one-word values (an array, C486): each
    /// the value itself, which a reference would not outlive.
    pending_values: Vec<Box<dyn Visit + 'a>>,
    /// Of each type a handle reached: how many values, and their bytes.
    counts: HashMap<&'static str, (u64, u64)>,
    strings: (u64, u64),
    /// The plain references met (a bloblet, a flat array): what a census
    /// reads as opaque blocks (C480).
    opaque: Vec<usize>,
}

impl<'a> Tracer<'a> {
    /// A handle's value, `v`: counted and kept to visit, if not seen
    /// before.
    pub fn reach(&mut self, v: &'a (dyn Visit + 'a)) {
        if self
            .seen
            .insert(std::ptr::from_ref(v).cast::<()>() as usize)
        {
            let e = self.counts.entry(v.visit_name()).or_insert((0, 0));
            e.0 += 1;
            e.1 += v.visit_size() as u64;
            self.pending.push(v);
        }
    }

    /// An object a one-word value points to (an array), at `addr`: counted
    /// and kept to visit, the value with it, if not seen before.
    pub fn reach_value<V: Visit + 'a>(&mut self, addr: usize, v: V) {
        if self.seen.insert(addr) {
            let e = self.counts.entry(v.visit_name()).or_insert((0, 0));
            e.0 += 1;
            e.1 += v.visit_size() as u64;
            self.pending_values.push(Box::new(v));
        }
    }

    /// A string's bytes, counted once.
    pub fn string(&mut self, s: &str) {
        if !s.is_empty() && self.seen.insert(s.as_ptr() as usize) {
            self.strings.0 += 1;
            self.strings.1 += s.len() as u64;
        }
    }

    /// A plain reference met, to what no `Trace` says the parts of.
    pub fn opaque(&mut self, p: usize) {
        self.opaque.push(p);
    }

    /// The addresses of what it has reached (a census's, C508).
    #[cfg(feature = "heap-stats")]
    pub(crate) fn seen(&self) -> impl Iterator<Item = usize> + '_ {
        self.seen.iter().copied()
    }

    /// The plain references met since asked last.
    pub fn take_opaque(&mut self) -> Vec<usize> {
        std::mem::take(&mut self.opaque)
    }

    /// Everything kept to visit, visited, and what that reaches.
    pub fn finish(&mut self) {
        loop {
            if let Some(v) = self.pending.pop() {
                v.visit(self);
            } else if let Some(v) = self.pending_values.pop() {
                // Visited as owned: what it reaches the heap's, not its.
                v.visit_owned(self);
            } else {
                break;
            }
        }
    }

    /// What was reached: by type, how many and how many bytes, the most
    /// bytes first; and the strings'.
    pub fn census(&self) -> (Vec<CensusRow>, (u64, u64)) {
        let mut rows: Vec<_> = self.counts.iter().map(|(k, (n, b))| (*k, *n, *b)).collect();
        rows.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(b.0)));
        (rows, self.strings)
    }
}

/// A census's row: a type's name, how many of its values, their bytes.
pub type CensusRow = (&'static str, u64, u64);

/// What `root` reaches: traced to the end.
pub fn traced<'a, T: Trace + 'a>(root: T) -> Tracer<'a> {
    let mut t = Tracer::default();
    root.trace(&mut t);
    t.finish();
    t
}

/// A value all plain bytes: none of it traced.
#[macro_export]
macro_rules! trace_bytes {
    ($($t:ty),*) => {
        $(impl $crate::Trace for $t {
            const WHOLE: bool = true;
            #[inline(always)]
            fn trace<'a>(self, _: &mut $crate::Tracer<'a>) where Self: 'a {}
            type Stored = Self;
            #[inline(always)]
            fn store(self) -> Self { self }
            #[inline(always)]
            fn load(s: Self) -> Self { s }
        })*
    };
}

/// A scalar, held as a Fixpt word (C495): its bytes on the stack, its
/// word in the heap.
macro_rules! trace_word {
    ($($t:ty, $store:expr, $load:expr;)*) => {
        $(impl Trace for $t {
            #[inline(always)]
            fn trace<'a>(self, _: &mut Tracer<'a>) where Self: 'a {}
            type Stored = crate::HeapWord;
            const VALUE_WORD: bool = true;
            #[inline(always)]
            fn store(self) -> crate::HeapWord { ($store)(self) }
            #[inline(always)]
            fn load(w: crate::HeapWord) -> Self { ($load)(w) }
        })*
    };
}

/// A value held as itself (no object of Fixpt's layout yet, C495): in an
/// impl of `Trace`, before its `trace`.
#[macro_export]
macro_rules! trace_whole {
    () => {
        // Held as it is, for now (C495): no object of Fixpt's layout yet.
        const WHOLE: bool = true;
        type Stored = Self;
        #[inline(always)]
        fn store(self) -> Self {
            self
        }
        #[inline(always)]
        fn load(s: Self) -> Self {
            s
        }
    };
}

/// A product (a tuple struct, C490): held boxed, one word to its object
/// (`Prod`, kind 37), where a heap object holds it, as FX-26's heap grammar
/// has it; its object its parts' words (`Product`, C496), as Fixpt's
/// product's fields. On the stack, its parts traced each.
#[macro_export]
macro_rules! trace_product {
    ($name:ident, $($a:ident $i:tt),*) => {
        impl<$($a: $crate::Trace),*> $crate::Trace for $name<$($a),*> {
            #[inline]
            fn trace<'a>(self, t: &mut $crate::Tracer<'a>) where Self: 'a {
                $(self.$i.trace(t);)*
            }
            // Held boxed, one word to its object (C490), Fixpt's (C506).
            type Stored = $crate::HeapWord;
            const VALUE_WORD: bool = true;
            #[inline]
            fn store(self) -> $crate::HeapWord {
                $crate::Prod::new(self).word()
            }
            #[inline(always)]
            fn load(w: $crate::HeapWord) -> Self {
                $crate::Prod::<Self>::of_word(w).get()
            }
        }
        impl<$($a: $crate::Trace),*> $crate::Product for $name<$($a),*> {
            type Words = $name<$($a::Stored),*>;
            #[inline(always)]
            fn words(self) -> Self::Words {
                const { assert!(true $(&& !$a::WHOLE)*, "a product's field held as itself, no Fixpt word (C508)") };
                $name($($crate::Trace::store(self.$i)),*)
            }
            #[inline(always)]
            fn of_words(w: Self::Words) -> Self {
                $name($($a::load(w.$i)),*)
            }
        }
        // A union's member (C491): its object's word, kind 37 (C506).
        $crate::word_member!([$($a: $crate::Trace + 'static),*] $name<$($a),*>; 37);
    };
}

/// A product's parts as its object (`Prod`) keeps them (C496): each a
/// Fixpt word, its `Stored`.
pub trait Product: Trace {
    type Words: Copy;
    fn words(self) -> Self::Words;
    fn of_words(w: Self::Words) -> Self;
}

/// A value as a heap object holds it (C490, C495): its type's `Stored`, a
/// Fixpt word (a product boxed, one word to its object). A pair's
/// element, a sum's payload, a cell's value.
pub struct Held<T: Trace>(T::Stored);

impl<T: Trace> Clone for Held<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Trace> Copy for Held<T> {}

impl<T: Trace> Held<T> {
    #[inline(always)]
    pub fn new(v: T) -> Self {
        const {
            assert!(
                !T::WHOLE,
                "a value held as itself in a heap object, no Fixpt word (C508)"
            )
        };
        Held(v.store())
    }
    #[inline(always)]
    pub fn get(self) -> T {
        T::load(self.0)
    }
    /// What it reaches: its value's.
    #[inline]
    pub fn reach<'a>(self, t: &mut Tracer<'a>)
    where
        Self: 'a,
    {
        self.get().trace(t);
    }
}

/// For a `static`'s (a `const`): a byte, an integer and unit, held as
/// their words.
impl Held<u8> {
    pub const fn byte(b: u8) -> Self {
        Held(crate::HeapWord::fixnum(b as i64))
    }
}

impl Held<Int> {
    pub const fn int(n: Int) -> Self {
        Held(crate::HeapWord::const_of_int(n))
    }
}

impl Held<()> {
    pub const UNIT: Self = Held(crate::HeapWord::UNIT);
}

trace_word! {
    i8, |x: i8| crate::HeapWord::fixnum(x as i64), |w: crate::HeapWord| crate::HeapWord::to_i64(w) as i8;
    i16, |x: i16| crate::HeapWord::fixnum(x as i64), |w: crate::HeapWord| crate::HeapWord::to_i64(w) as i16;
    i32, |x: i32| crate::HeapWord::fixnum(x as i64), |w: crate::HeapWord| crate::HeapWord::to_i64(w) as i32;
    u8, |x: u8| crate::HeapWord::fixnum(x as i64), |w: crate::HeapWord| crate::HeapWord::to_i64(w) as u8;
    u16, |x: u16| crate::HeapWord::fixnum(x as i64), |w: crate::HeapWord| crate::HeapWord::to_i64(w) as u16;
    u32, |x: u32| crate::HeapWord::fixnum(x as i64), |w: crate::HeapWord| crate::HeapWord::to_i64(w) as u32;
    i64, crate::HeapWord::of_i64, crate::HeapWord::to_i64;
    u64, crate::HeapWord::of_u64, crate::HeapWord::to_u64;
    usize, |x: usize| crate::HeapWord::of_u64(x as u64), |w: crate::HeapWord| crate::HeapWord::to_u64(w) as usize;
    f32, crate::HeapWord::of_f32, crate::HeapWord::to_f32;
    f64, crate::HeapWord::flonum, crate::HeapWord::to_f64;
    bool, crate::HeapWord::boolean, crate::HeapWord::to_bool;
    char, crate::HeapWord::of_char, crate::HeapWord::to_char;
}

trace_bytes!(Void);

/// Nothing: not traced, held as Fixpt's unit.
impl Trace for () {
    #[inline(always)]
    fn trace<'a>(self, _: &mut Tracer<'a>)
    where
        Self: 'a,
    {
    }
    type Stored = crate::HeapWord;
    const VALUE_WORD: bool = true;
    #[inline(always)]
    fn store(self) -> crate::HeapWord {
        crate::HeapWord::UNIT
    }
    #[inline(always)]
    fn load(_: crate::HeapWord) {}
}

/// A closure's captures (`closure.rs`, C496): a tuple, its parts' words;
/// none, no words (not unit's word: a closure of no captures no bytes,
/// C494), as Fixpt's closure of no free values has none.
pub trait Captures: Trace {
    type Words: Copy;
    fn words(self) -> Self::Words;
    fn of_words(w: Self::Words) -> Self;
}

impl Captures for () {
    type Words = ();
    #[inline(always)]
    fn words(self) {}
    #[inline(always)]
    fn of_words(_: ()) {}
}

/// Tuples: each part traced, and held, as itself (a closure's captures
/// each a word).
macro_rules! trace_tuples {
    ($(($($a:ident $i:tt),+)),*) => {
        $(impl<$($a: Trace),+> Trace for ($($a,)+) {
            #[inline]
            fn trace<'a>(self, t: &mut Tracer<'a>) where Self: 'a {
                $(self.$i.trace(t);)+
            }
            type Stored = ($($a::Stored,)+);
            const WHOLE: bool = false $(|| $a::WHOLE)+;
            #[inline(always)]
            fn store(self) -> Self::Stored {
                ($(self.$i.store(),)+)
            }
            #[inline(always)]
            fn load(s: Self::Stored) -> Self {
                ($($a::load(s.$i),)+)
            }
        }
        impl<$($a: Trace),+> Captures for ($($a,)+) {
            type Words = ($($a::Stored,)+);
            #[inline(always)]
            fn words(self) -> Self::Words {
                const { assert!(!<Self as Trace>::WHOLE, "a closure's capture held as itself, no Fixpt word (C508)") };
                self.store()
            }
            #[inline(always)]
            fn of_words(w: Self::Words) -> Self {
                Self::load(w)
            }
        })*
    };
}
trace_tuples!(
    (A 0),
    (A 0, B 1),
    (A 0, B 1, C 2),
    (A 0, B 1, C 2, D 3),
    (A 0, B 1, C 2, D 3, E 4),
    (A 0, B 1, C 2, D 3, E 4, F 5),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13, O 14),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13, O 14, P 15)
);

/// A closure's free values (C501), as Fixpt's native closure holds them:
/// each a word, free value 0 nearest its code (Fixpt's `CLOSURE_FREE0`,
/// its slots counted back from the trailer), so in memory the last first.
pub trait Free: Trace {
    type Words: Copy;
    fn words(self) -> Self::Words;
    fn of_words(w: Self::Words) -> Self;
}

impl Free for () {
    type Words = ();
    #[inline(always)]
    fn words(self) {}
    #[inline(always)]
    fn of_words(_: ()) {}
}

impl<A: Trace> Free for (A,) {
    type Words = (A::Stored,);
    #[inline(always)]
    fn words(self) -> Self::Words {
        (self.0.store(),)
    }
    #[inline(always)]
    fn of_words(w: Self::Words) -> Self {
        (A::load(w.0),)
    }
}

impl<A: Trace, B: Trace> Free for (A, B) {
    type Words = (B::Stored, A::Stored);
    #[inline(always)]
    fn words(self) -> Self::Words {
        (self.1.store(), self.0.store())
    }
    #[inline(always)]
    fn of_words(w: Self::Words) -> Self {
        (A::load(w.1), B::load(w.0))
    }
}

impl<A: Trace, B: Trace, C: Trace> Free for (A, B, C) {
    type Words = (C::Stored, B::Stored, A::Stored);
    #[inline(always)]
    fn words(self) -> Self::Words {
        (self.2.store(), self.1.store(), self.0.store())
    }
    #[inline(always)]
    fn of_words(w: Self::Words) -> Self {
        (A::load(w.2), B::load(w.1), C::load(w.0))
    }
}

impl<A: Trace, B: Trace, C: Trace, D: Trace> Free for (A, B, C, D) {
    type Words = (D::Stored, C::Stored, B::Stored, A::Stored);
    #[inline(always)]
    fn words(self) -> Self::Words {
        (
            self.3.store(),
            self.2.store(),
            self.1.store(),
            self.0.store(),
        )
    }
    #[inline(always)]
    fn of_words(w: Self::Words) -> Self {
        (A::load(w.3), B::load(w.2), C::load(w.1), D::load(w.0))
    }
}

impl<A: Trace, B: Trace, C: Trace, D: Trace, E: Trace> Free for (A, B, C, D, E) {
    type Words = (E::Stored, D::Stored, C::Stored, B::Stored, A::Stored);
    #[inline(always)]
    fn words(self) -> Self::Words {
        (
            self.4.store(),
            self.3.store(),
            self.2.store(),
            self.1.store(),
            self.0.store(),
        )
    }
    #[inline(always)]
    fn of_words(w: Self::Words) -> Self {
        (
            A::load(w.4),
            B::load(w.3),
            C::load(w.2),
            D::load(w.1),
            E::load(w.0),
        )
    }
}

impl<A: Trace, B: Trace, C: Trace, D: Trace, E: Trace, F: Trace> Free for (A, B, C, D, E, F) {
    type Words = (
        F::Stored,
        E::Stored,
        D::Stored,
        C::Stored,
        B::Stored,
        A::Stored,
    );
    #[inline(always)]
    fn words(self) -> Self::Words {
        (
            self.5.store(),
            self.4.store(),
            self.3.store(),
            self.2.store(),
            self.1.store(),
            self.0.store(),
        )
    }
    #[inline(always)]
    fn of_words(w: Self::Words) -> Self {
        (
            A::load(w.5),
            B::load(w.4),
            C::load(w.3),
            D::load(w.2),
            E::load(w.1),
            F::load(w.0),
        )
    }
}

impl<A: Trace, B: Trace, C: Trace, D: Trace, E: Trace, F: Trace, G: Trace> Free
    for (A, B, C, D, E, F, G)
{
    type Words = (
        G::Stored,
        F::Stored,
        E::Stored,
        D::Stored,
        C::Stored,
        B::Stored,
        A::Stored,
    );
    #[inline(always)]
    fn words(self) -> Self::Words {
        (
            self.6.store(),
            self.5.store(),
            self.4.store(),
            self.3.store(),
            self.2.store(),
            self.1.store(),
            self.0.store(),
        )
    }
    #[inline(always)]
    fn of_words(w: Self::Words) -> Self {
        (
            A::load(w.6),
            B::load(w.5),
            C::load(w.4),
            D::load(w.3),
            E::load(w.2),
            F::load(w.1),
            G::load(w.0),
        )
    }
}

impl<A: Trace, B: Trace, C: Trace, D: Trace, E: Trace, F: Trace, G: Trace, H: Trace> Free
    for (A, B, C, D, E, F, G, H)
{
    type Words = (
        H::Stored,
        G::Stored,
        F::Stored,
        E::Stored,
        D::Stored,
        C::Stored,
        B::Stored,
        A::Stored,
    );
    #[inline(always)]
    fn words(self) -> Self::Words {
        (
            self.7.store(),
            self.6.store(),
            self.5.store(),
            self.4.store(),
            self.3.store(),
            self.2.store(),
            self.1.store(),
            self.0.store(),
        )
    }
    #[inline(always)]
    fn of_words(w: Self::Words) -> Self {
        (
            A::load(w.7),
            B::load(w.6),
            C::load(w.5),
            D::load(w.4),
            E::load(w.3),
            F::load(w.2),
            G::load(w.1),
            H::load(w.0),
        )
    }
}

impl<A: Trace, B: Trace, C: Trace, D: Trace, E: Trace, F: Trace, G: Trace, H: Trace, I: Trace> Free
    for (A, B, C, D, E, F, G, H, I)
{
    type Words = (
        I::Stored,
        H::Stored,
        G::Stored,
        F::Stored,
        E::Stored,
        D::Stored,
        C::Stored,
        B::Stored,
        A::Stored,
    );
    #[inline(always)]
    fn words(self) -> Self::Words {
        (
            self.8.store(),
            self.7.store(),
            self.6.store(),
            self.5.store(),
            self.4.store(),
            self.3.store(),
            self.2.store(),
            self.1.store(),
            self.0.store(),
        )
    }
    #[inline(always)]
    fn of_words(w: Self::Words) -> Self {
        (
            A::load(w.8),
            B::load(w.7),
            C::load(w.6),
            D::load(w.5),
            E::load(w.4),
            F::load(w.3),
            G::load(w.2),
            H::load(w.1),
            I::load(w.0),
        )
    }
}

impl<
    A: Trace,
    B: Trace,
    C: Trace,
    D: Trace,
    E: Trace,
    F: Trace,
    G: Trace,
    H: Trace,
    I: Trace,
    J: Trace,
> Free for (A, B, C, D, E, F, G, H, I, J)
{
    type Words = (
        J::Stored,
        I::Stored,
        H::Stored,
        G::Stored,
        F::Stored,
        E::Stored,
        D::Stored,
        C::Stored,
        B::Stored,
        A::Stored,
    );
    #[inline(always)]
    fn words(self) -> Self::Words {
        (
            self.9.store(),
            self.8.store(),
            self.7.store(),
            self.6.store(),
            self.5.store(),
            self.4.store(),
            self.3.store(),
            self.2.store(),
            self.1.store(),
            self.0.store(),
        )
    }
    #[inline(always)]
    fn of_words(w: Self::Words) -> Self {
        (
            A::load(w.9),
            B::load(w.8),
            C::load(w.7),
            D::load(w.6),
            E::load(w.5),
            F::load(w.4),
            G::load(w.3),
            H::load(w.2),
            I::load(w.1),
            J::load(w.0),
        )
    }
}

impl<
    A: Trace,
    B: Trace,
    C: Trace,
    D: Trace,
    E: Trace,
    F: Trace,
    G: Trace,
    H: Trace,
    I: Trace,
    J: Trace,
    K: Trace,
> Free for (A, B, C, D, E, F, G, H, I, J, K)
{
    type Words = (
        K::Stored,
        J::Stored,
        I::Stored,
        H::Stored,
        G::Stored,
        F::Stored,
        E::Stored,
        D::Stored,
        C::Stored,
        B::Stored,
        A::Stored,
    );
    #[inline(always)]
    fn words(self) -> Self::Words {
        (
            self.10.store(),
            self.9.store(),
            self.8.store(),
            self.7.store(),
            self.6.store(),
            self.5.store(),
            self.4.store(),
            self.3.store(),
            self.2.store(),
            self.1.store(),
            self.0.store(),
        )
    }
    #[inline(always)]
    fn of_words(w: Self::Words) -> Self {
        (
            A::load(w.10),
            B::load(w.9),
            C::load(w.8),
            D::load(w.7),
            E::load(w.6),
            F::load(w.5),
            G::load(w.4),
            H::load(w.3),
            I::load(w.2),
            J::load(w.1),
            K::load(w.0),
        )
    }
}

impl<
    A: Trace,
    B: Trace,
    C: Trace,
    D: Trace,
    E: Trace,
    F: Trace,
    G: Trace,
    H: Trace,
    I: Trace,
    J: Trace,
    K: Trace,
    L: Trace,
> Free for (A, B, C, D, E, F, G, H, I, J, K, L)
{
    type Words = (
        L::Stored,
        K::Stored,
        J::Stored,
        I::Stored,
        H::Stored,
        G::Stored,
        F::Stored,
        E::Stored,
        D::Stored,
        C::Stored,
        B::Stored,
        A::Stored,
    );
    #[inline(always)]
    fn words(self) -> Self::Words {
        (
            self.11.store(),
            self.10.store(),
            self.9.store(),
            self.8.store(),
            self.7.store(),
            self.6.store(),
            self.5.store(),
            self.4.store(),
            self.3.store(),
            self.2.store(),
            self.1.store(),
            self.0.store(),
        )
    }
    #[inline(always)]
    fn of_words(w: Self::Words) -> Self {
        (
            A::load(w.11),
            B::load(w.10),
            C::load(w.9),
            D::load(w.8),
            E::load(w.7),
            F::load(w.6),
            G::load(w.5),
            H::load(w.4),
            I::load(w.3),
            J::load(w.2),
            K::load(w.1),
            L::load(w.0),
        )
    }
}

impl<
    A: Trace,
    B: Trace,
    C: Trace,
    D: Trace,
    E: Trace,
    F: Trace,
    G: Trace,
    H: Trace,
    I: Trace,
    J: Trace,
    K: Trace,
    L: Trace,
    M: Trace,
> Free for (A, B, C, D, E, F, G, H, I, J, K, L, M)
{
    type Words = (
        M::Stored,
        L::Stored,
        K::Stored,
        J::Stored,
        I::Stored,
        H::Stored,
        G::Stored,
        F::Stored,
        E::Stored,
        D::Stored,
        C::Stored,
        B::Stored,
        A::Stored,
    );
    #[inline(always)]
    fn words(self) -> Self::Words {
        (
            self.12.store(),
            self.11.store(),
            self.10.store(),
            self.9.store(),
            self.8.store(),
            self.7.store(),
            self.6.store(),
            self.5.store(),
            self.4.store(),
            self.3.store(),
            self.2.store(),
            self.1.store(),
            self.0.store(),
        )
    }
    #[inline(always)]
    fn of_words(w: Self::Words) -> Self {
        (
            A::load(w.12),
            B::load(w.11),
            C::load(w.10),
            D::load(w.9),
            E::load(w.8),
            F::load(w.7),
            G::load(w.6),
            H::load(w.5),
            I::load(w.4),
            J::load(w.3),
            K::load(w.2),
            L::load(w.1),
            M::load(w.0),
        )
    }
}

impl<
    A: Trace,
    B: Trace,
    C: Trace,
    D: Trace,
    E: Trace,
    F: Trace,
    G: Trace,
    H: Trace,
    I: Trace,
    J: Trace,
    K: Trace,
    L: Trace,
    M: Trace,
    N: Trace,
> Free for (A, B, C, D, E, F, G, H, I, J, K, L, M, N)
{
    type Words = (
        N::Stored,
        M::Stored,
        L::Stored,
        K::Stored,
        J::Stored,
        I::Stored,
        H::Stored,
        G::Stored,
        F::Stored,
        E::Stored,
        D::Stored,
        C::Stored,
        B::Stored,
        A::Stored,
    );
    #[inline(always)]
    fn words(self) -> Self::Words {
        (
            self.13.store(),
            self.12.store(),
            self.11.store(),
            self.10.store(),
            self.9.store(),
            self.8.store(),
            self.7.store(),
            self.6.store(),
            self.5.store(),
            self.4.store(),
            self.3.store(),
            self.2.store(),
            self.1.store(),
            self.0.store(),
        )
    }
    #[inline(always)]
    fn of_words(w: Self::Words) -> Self {
        (
            A::load(w.13),
            B::load(w.12),
            C::load(w.11),
            D::load(w.10),
            E::load(w.9),
            F::load(w.8),
            G::load(w.7),
            H::load(w.6),
            I::load(w.5),
            J::load(w.4),
            K::load(w.3),
            L::load(w.2),
            M::load(w.1),
            N::load(w.0),
        )
    }
}

impl<
    A: Trace,
    B: Trace,
    C: Trace,
    D: Trace,
    E: Trace,
    F: Trace,
    G: Trace,
    H: Trace,
    I: Trace,
    J: Trace,
    K: Trace,
    L: Trace,
    M: Trace,
    N: Trace,
    O: Trace,
> Free for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O)
{
    type Words = (
        O::Stored,
        N::Stored,
        M::Stored,
        L::Stored,
        K::Stored,
        J::Stored,
        I::Stored,
        H::Stored,
        G::Stored,
        F::Stored,
        E::Stored,
        D::Stored,
        C::Stored,
        B::Stored,
        A::Stored,
    );
    #[inline(always)]
    fn words(self) -> Self::Words {
        (
            self.14.store(),
            self.13.store(),
            self.12.store(),
            self.11.store(),
            self.10.store(),
            self.9.store(),
            self.8.store(),
            self.7.store(),
            self.6.store(),
            self.5.store(),
            self.4.store(),
            self.3.store(),
            self.2.store(),
            self.1.store(),
            self.0.store(),
        )
    }
    #[inline(always)]
    fn of_words(w: Self::Words) -> Self {
        (
            A::load(w.14),
            B::load(w.13),
            C::load(w.12),
            D::load(w.11),
            E::load(w.10),
            F::load(w.9),
            G::load(w.8),
            H::load(w.7),
            I::load(w.6),
            J::load(w.5),
            K::load(w.4),
            L::load(w.3),
            M::load(w.2),
            N::load(w.1),
            O::load(w.0),
        )
    }
}

impl<
    A: Trace,
    B: Trace,
    C: Trace,
    D: Trace,
    E: Trace,
    F: Trace,
    G: Trace,
    H: Trace,
    I: Trace,
    J: Trace,
    K: Trace,
    L: Trace,
    M: Trace,
    N: Trace,
    O: Trace,
    P: Trace,
> Free for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P)
{
    type Words = (
        P::Stored,
        O::Stored,
        N::Stored,
        M::Stored,
        L::Stored,
        K::Stored,
        J::Stored,
        I::Stored,
        H::Stored,
        G::Stored,
        F::Stored,
        E::Stored,
        D::Stored,
        C::Stored,
        B::Stored,
        A::Stored,
    );
    #[inline(always)]
    fn words(self) -> Self::Words {
        (
            self.15.store(),
            self.14.store(),
            self.13.store(),
            self.12.store(),
            self.11.store(),
            self.10.store(),
            self.9.store(),
            self.8.store(),
            self.7.store(),
            self.6.store(),
            self.5.store(),
            self.4.store(),
            self.3.store(),
            self.2.store(),
            self.1.store(),
            self.0.store(),
        )
    }
    #[inline(always)]
    fn of_words(w: Self::Words) -> Self {
        (
            A::load(w.15),
            B::load(w.14),
            C::load(w.13),
            D::load(w.12),
            E::load(w.11),
            F::load(w.10),
            G::load(w.9),
            H::load(w.8),
            I::load(w.7),
            J::load(w.6),
            K::load(w.5),
            L::load(w.4),
            M::load(w.3),
            N::load(w.2),
            O::load(w.1),
            P::load(w.0),
        )
    }
}

/// A bloblet's fields, each a cell (C486): each visited.
macro_rules! visit_tuples {
    ($(($($a:ident $i:tt),+)),*) => {
        $(impl<$($a: Visit),+> Visit for ($($a,)+) {
            fn visit<'a>(&'a self, t: &mut Tracer<'a>) {
                $(self.$i.visit(t);)+
            }
        })*
    };
}
visit_tuples!(
    (A 0),
    (A 0, B 1),
    (A 0, B 1, C 2),
    (A 0, B 1, C 2, D 3),
    (A 0, B 1, C 2, D 3, E 4),
    (A 0, B 1, C 2, D 3, E 4, F 5),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13, O 14),
    (A 0, B 1, C 2, D 3, E 4, F 5, G 6, H 7, I 8, J 9, K 10, L 11, M 12, N 13, O 14, P 15)
);

/// No fields.
impl Visit for () {
    fn visit<'a>(&'a self, _: &mut Tracer<'a>) {}
}

/// A bytevector's bytes, the heap's but holding none of it: held as its
/// word (C508), Fixpt's (kind 4).
impl Trace for crate::BytesAt<'_> {
    type Stored = crate::HeapWord;
    #[inline(always)]
    fn store(self) -> crate::HeapWord {
        self.word()
    }
    #[inline(always)]
    fn load(w: crate::HeapWord) -> Self {
        crate::BytesAt::of_word(w)
    }
    fn trace<'a>(self, _: &mut Tracer<'a>)
    where
        Self: 'a,
    {
    }
}

impl Trace for Str {
    type Stored = crate::HeapWord;
    const VALUE_WORD: bool = true;
    #[inline(always)]
    fn store(self) -> crate::HeapWord {
        crate::HeapWord::of_str(self)
    }
    #[inline(always)]
    fn load(w: crate::HeapWord) -> Self {
        w.to_str()
    }
    fn trace<'a>(self, t: &mut Tracer<'a>)
    where
        Self: 'a,
    {
        t.string(self.as_str());
    }
}

/// An integer (C488): a fixnum, nothing; a bignum, its object reached,
/// nothing in it traced.
impl Trace for Int {
    type Stored = crate::HeapWord;
    const VALUE_WORD: bool = true;
    #[inline(always)]
    fn store(self) -> crate::HeapWord {
        crate::HeapWord::of_int(self)
    }
    #[inline(always)]
    fn load(w: crate::HeapWord) -> Self {
        w.to_int()
    }
    #[inline]
    fn trace<'a>(self, t: &mut Tracer<'a>)
    where
        Self: 'a,
    {
        if let crate::IntView::Big(b) = self.view() {
            t.reach_value(self.word() & !0b111, b);
        }
    }
}

/// A flat array (C489): its object reached once; its elements numbers,
/// nothing traced.
impl<T: 'static> Trace for crate::FlatArray<T> {
    type Stored = crate::HeapWord;
    const VALUE_WORD: bool = true;
    #[inline(always)]
    fn store(self) -> crate::HeapWord {
        self.word()
    }
    #[inline(always)]
    fn load(w: crate::HeapWord) -> Self {
        crate::FlatArray::of_word(w)
    }
    #[inline]
    fn trace<'a>(self, t: &mut Tracer<'a>)
    where
        Self: 'a,
    {
        t.reach_value(self.addr(), self);
    }
}

impl<T> Visit for crate::FlatArray<T> {
    fn visit<'a>(&'a self, _: &mut Tracer<'a>) {}
    fn visit_size(&self) -> usize {
        self.object_bytes()
    }
}

impl Visit for BigAt {
    fn visit<'a>(&'a self, _: &mut Tracer<'a>) {}
    fn visit_size(&self) -> usize {
        self.object_bytes()
    }
}

impl<'r, T: Visit + 'r> Trace for Handle<'r, T> {
    trace_whole!();
    #[inline]
    fn trace<'a>(self, t: &mut Tracer<'a>)
    where
        Self: 'a,
    {
        t.reach(self.to_ref());
    }
}

impl<T: Trace> Trace for Option<T> {
    trace_whole!();
    #[inline]
    fn trace<'a>(self, t: &mut Tracer<'a>)
    where
        Self: 'a,
    {
        if let Some(x) = self {
            x.trace(t);
        }
    }
}

impl<T: Trace> Visit for Cons<'_, T> {
    fn visit<'a>(&'a self, t: &mut Tracer<'a>) {
        self.car_held().reach(t);
        self.cdr.trace(t);
    }
}

impl<T: Trace> Visit for Cell<T> {
    fn visit<'a>(&'a self, t: &mut Tracer<'a>) {
        self.get().trace(t);
    }
}

/// A cell of KFX's (C490): a box's (a reference's, `RefObj`, C498) or an
/// array's element, its value held (a Fixpt word, as Fixpt's box holds
/// one). Read and written as a `Cell` is.
pub struct Slot<T: Trace>(Cell<Held<T>>);

impl<T: Trace> Slot<T> {
    #[inline(always)]
    pub fn new(v: T) -> Self {
        Slot(Cell::new(Held::new(v)))
    }
    #[inline(always)]
    pub fn get(&self) -> T {
        self.0.get().get()
    }
    #[inline(always)]
    pub fn set(&self, v: T) {
        self.0.set(Held::new(v))
    }
}

/// A copy of its value in a cell of its own, as `Cell`'s (a sealed
/// bloblet's fields, C116).
impl<T: Trace> Clone for Slot<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Slot(Cell::new(self.0.get()))
    }
}

impl<T: Trace> Visit for Slot<T> {
    fn visit<'a>(&'a self, t: &mut Tracer<'a>) {
        self.0.get().reach(t);
    }
}

/// An array (C486): its object reached once, its elements visited then;
/// held as its word (C502).
impl<T: Trace> Trace for Array<T> {
    type Stored = crate::HeapWord;
    const VALUE_WORD: bool = true;
    #[inline(always)]
    fn store(self) -> crate::HeapWord {
        self.word()
    }
    #[inline(always)]
    fn load(w: crate::HeapWord) -> Self {
        Array::of_word(w)
    }
    #[inline]
    fn trace<'a>(self, t: &mut Tracer<'a>)
    where
        Self: 'a,
    {
        t.reach_value(self.addr(), self);
    }
}

impl<T: Trace> Visit for Array<T> {
    fn visit<'a>(&'a self, t: &mut Tracer<'a>) {
        self.visit_owned(t);
    }
    fn visit_owned(&self, t: &mut Tracer<'_>) {
        for c in self.cells() {
            c.get().trace(t);
        }
    }
    fn visit_size(&self) -> usize {
        self.object_bytes()
    }
}

/// A plain reference, what KFX's Rust has that is no handle (C492): what
/// it points to visited, precisely: an I-cell, a mailbox, a knot, a
/// place, a module's state. (A continuation and a blobref into a suffix
/// are opaque, each by its own impl.)
///
/// Held as its address (C508), a word of tag `000`, which a collector reads
/// as a fixnum, neither tracing nor moving it: what it points to is
/// kayak-rt's own, never the heap's space (checked in a debug build), and
/// what that holds its roots' (`TODO.md` §54's step 3).
impl<'r, T: Visit + 'r> Trace for &'r T {
    type Stored = crate::HeapWord;
    #[inline(always)]
    fn store(self) -> crate::HeapWord {
        crate::HeapWord::outside(self)
    }
    #[inline(always)]
    fn load(w: crate::HeapWord) -> Self {
        w.to_outside()
    }
    #[inline]
    fn trace<'a>(self, t: &mut Tracer<'a>)
    where
        Self: 'a,
    {
        t.reach(self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_list_reached_once_each() {
        let shared = crate::heap_cons(crate::str_lit("x"), crate::nil());
        let xs = crate::heap_cons(crate::str_lit("y"), shared);
        let ys = crate::heap_cons(crate::str_lit("z"), shared);
        let both = crate::heap_cons(xs, crate::heap_cons(ys, crate::nil()));
        let t = traced(both);
        let (rows, strings) = t.census();
        let pairs = |name: &str| {
            rows.iter()
                .filter(|r| r.0.contains(name))
                .map(|r| r.1)
                .sum::<u64>()
        };
        // Two pairs of the outer list, three of strings (the shared tail once).
        assert_eq!(pairs("Cons"), 5);
        assert_eq!(strings.0, 3);
    }

    #[test]
    fn arrays_and_bloblets_reached_once_each() {
        // An array of a string twice and a pair; a bloblet holding the
        // array twice and another string.
        let p = crate::heap_cons(crate::str_lit("p"), crate::nil());
        let a = crate::list_to_array(crate::heap_cons(
            p,
            crate::heap_cons(crate::nil(), crate::heap_cons(p, crate::nil())),
        ));
        let b = crate::make_bloblet(
            crate::int_lit(12),
            (Cell::new(a), Cell::new(a), Cell::new(crate::str_lit("q"))),
        );
        let t = traced((b, b, a));
        let (rows, strings) = t.census();
        let of = |name: &str| {
            rows.iter()
                .filter(|r| r.0.starts_with(name))
                .collect::<Vec<_>>()
        };
        assert_eq!(of("kayak_rt::obj::Array<")[0].1, 1);
        // Its header, three elements of a word, its trailer.
        assert_eq!(of("kayak_rt::obj::Array<")[0].2, 5 * 8);
        assert_eq!(of("kayak_rt::obj::Bloblet<")[0].1, 1);
        // Its header, three fields, its trailer, two words of suffix.
        assert_eq!(of("kayak_rt::obj::Bloblet<")[0].2, 7 * 8);
        assert_eq!(of("kayak_rt::Cons<")[0].1, 1);
        assert_eq!(strings.0, 2);
    }

    /// A chain of arrays, each holding the next: a type of its own.
    #[derive(Clone, Copy)]
    struct Link(Option<crate::Array<Link>>);

    /// Held as a word (C508): its array's, or `#f`.
    impl Trace for Link {
        type Stored = crate::HeapWord;
        fn store(self) -> crate::HeapWord {
            match self.0 {
                Some(a) => a.word(),
                None => crate::HeapWord::boolean(false),
            }
        }
        fn load(w: crate::HeapWord) -> Self {
            Link(
                (w.raw() != crate::HeapWord::boolean(false).raw())
                    .then(|| crate::Array::of_word(w)),
            )
        }
        fn trace<'a>(self, t: &mut Tracer<'a>)
        where
            Self: 'a,
        {
            self.0.trace(t);
        }
    }

    #[test]
    fn plain_references_visited_precisely() {
        // An I-cell, a mailbox and a knot, each holding a string; and a
        // region's arena of pairs, reached through a reference to it.
        let c: crate::ICell<Str> = crate::leak(crate::process::ICellBox {
            value: std::sync::Mutex::new(Some(crate::str_lit("i"))),
        });
        let m: crate::Addr<Str> = crate::leak(crate::process::MailboxBox {
            queue: std::sync::Mutex::new([crate::str_lit("m")].into_iter().collect()),
        });
        let k: &'static crate::Knot<Str> = crate::leak(crate::Knot(std::sync::OnceLock::new()));
        let _ = k.0.set(crate::str_lit("k"));
        let arena: crate::Arena<Cons<'_, Str>> = crate::Arena::default();
        let xs = crate::ListAt::of_pair(arena.alloc(Cons::new(crate::str_lit("r"), crate::nil())));
        let ys = crate::ListAt::of_pair(arena.alloc(Cons::new(crate::str_lit("s"), xs)));
        let mut t = traced((c, m, k, &arena, ys));
        assert!(t.take_opaque().is_empty());
        let (rows, strings) = t.census();
        let of = |name: &str| {
            rows.iter()
                .filter(|r| r.0.contains(name))
                .map(|r| r.1)
                .sum::<u64>()
        };
        assert_eq!(of("ICellBox"), 1);
        assert_eq!(of("MailboxBox"), 1);
        assert_eq!(of("Knot"), 1);
        assert_eq!(of("Arena"), 1);
        // The arena's two pairs, each once, though `ys` reaches them too.
        let pairs = rows
            .iter()
            .filter(|r| r.0.starts_with("kayak_rt::Cons<"))
            .map(|r| r.1)
            .sum::<u64>();
        assert_eq!(pairs, 2);
        assert_eq!(strings.0, 5);
    }

    #[test]
    fn a_long_chain_of_arrays_in_a_loop() {
        let mut a = crate::list_to_array(crate::heap_cons(Link(None), crate::nil()));
        for _ in 0..200_000 {
            a = crate::list_to_array(crate::heap_cons(Link(Some(a)), crate::nil()));
        }
        let (rows, _) = traced(a).census();
        assert_eq!(rows.iter().map(|r| r.1).sum::<u64>(), 200_001);
    }

    #[test]
    fn a_long_list_in_a_loop() {
        let mut xs: crate::List<i64> = crate::nil();
        for k in 0..1_000_000 {
            xs = crate::heap_cons(k, xs);
        }
        let (rows, _) = traced(xs).census();
        assert_eq!(rows.iter().map(|r| r.1).sum::<u64>(), 1_000_000);
    }

    #[test]
    fn a_tuple_held_as_words() {
        let s = crate::str_lit("s");
        let w = (3i64, s, 2.5f64).store();
        let _: (crate::HeapWord, crate::HeapWord, crate::HeapWord) = w;
        assert_eq!(w.0.raw(), 3 << 3);
        let back = <(i64, Str, f64)>::load(w);
        assert_eq!((back.0, back.1.as_str(), back.2), (3, "s", 2.5));
    }
}
