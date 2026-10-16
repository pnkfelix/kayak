//! Code a program makes and runs (`CLAUDES_CHOICES.md` C116; Fixpt's TODO
//! §97): FX-26's `seal-code`, `native-procedure` and `native-code?`, over
//! kayak-jit's code area, in Fixpt's native convention.
//!
//! A native procedure is called with FX-26's values in Fixpt's layout, and
//! gives one; kayak's values are Rust's. So what crosses is what both lay
//! out alike, and KFX's checker lets nothing else cross: an integer that is
//! a fixnum, a boolean, unit, a character. An integer past a fixnum, which
//! Fixpt passes as a bignum, fails here rather than be made one.

use crate::{Bloblet, Int, fail, i64_to_int};

/// A value that crosses into native code and back, as the same word Fixpt
/// would pass.
pub trait Native: Sized {
    fn word(self) -> u64;
    fn of_word(w: u64) -> Self;
}

const SHIFT: u32 = kayak_jit::layout::FIXNUM_SHIFT;
const LIMIT: i64 = 1 << (kayak_jit::layout::FIXNUM_BITS - 1);

fn fixnum(n: i64) -> u64 {
    if !(-LIMIT..LIMIT).contains(&n) {
        fail("native code: an integer past a fixnum, which kayak cannot yet give native code");
    }
    (n << SHIFT) as u64
}

fn of_fixnum(w: u64) -> i64 {
    if w & kayak_jit::layout::TAG_MASK != kayak_jit::layout::TAG_FIXNUM {
        fail(
            "native code: gave what is not a fixnum where an integer was wanted, which kayak cannot yet read",
        );
    }
    (w as i64) >> SHIFT
}

impl Native for Int {
    fn word(self) -> u64 {
        match self {
            Int::Small(n) => fixnum(n),
            Int::Big(_) => fixnum(i64::MAX),
        }
    }
    fn of_word(w: u64) -> Self {
        i64_to_int(of_fixnum(w))
    }
}

impl Native for i64 {
    fn word(self) -> u64 {
        fixnum(self)
    }
    fn of_word(w: u64) -> Self {
        of_fixnum(w)
    }
}

impl Native for bool {
    fn word(self) -> u64 {
        if self {
            kayak_jit::layout::TRUE
        } else {
            kayak_jit::layout::FALSE
        }
    }
    fn of_word(w: u64) -> Self {
        match w {
            kayak_jit::layout::TRUE => true,
            kayak_jit::layout::FALSE => false,
            _ => fail("native code: gave what is not a boolean where one was wanted"),
        }
    }
}

impl Native for () {
    fn word(self) -> u64 {
        kayak_jit::layout::UNIT
    }
    fn of_word(w: u64) -> Self {
        if w != kayak_jit::layout::UNIT {
            fail("native code: gave what is not unit where it was wanted")
        }
    }
}

impl Native for char {
    fn word(self) -> u64 {
        (self as u64) << kayak_jit::layout::CHAR_SHIFT
            | kayak_jit::layout::CHAR_SUBTAG << 3
            | kayak_jit::layout::TAG_IMMEDIATE
    }
    fn of_word(w: u64) -> Self {
        let low = kayak_jit::layout::CHAR_SUBTAG << 3 | kayak_jit::layout::TAG_IMMEDIATE;
        match (w & 0xff == low).then(|| char::from_u32((w >> kayak_jit::layout::CHAR_SHIFT) as u32))
        {
            Some(Some(c)) => c,
            _ => fail("native code: gave what is not a character where one was wanted"),
        }
    }
}

/// A list, copied each way, as pairs of Fixpt's layout made by kayak-jit
/// and as kayak's. No KFX program can tell the copy from the list, since
/// none tells pairs apart nor writes through one (C116).
impl<T: Native + Clone + 'static> Native for crate::List<T> {
    fn word(self) -> u64 {
        let mut items = Vec::new();
        let mut at = self;
        while let Some(c) = at {
            items.push(c.car.clone());
            at = c.cdr;
        }
        items
            .into_iter()
            .rev()
            .fold(kayak_jit::layout::NULL, |l, x| kayak_jit::cons(x.word(), l))
    }
    fn of_word(w: u64) -> Self {
        let mut items = Vec::new();
        let mut at = w;
        while let Some((car, cdr)) = kayak_jit::pair(at) {
            items.push(T::of_word(car));
            at = cdr;
        }
        if at != kayak_jit::layout::NULL {
            fail("native code: gave what is not a list where one was wanted");
        }
        items
            .into_iter()
            .rev()
            .fold(crate::nil(), |l, x| crate::heap_cons(x, l))
    }
}

/// An `i64` flat array, shared, not copied (C126): the word of its first
/// element, as Fixpt's flat array's suffix, its header, element code and
/// trailer before it (`make_flatarray_i64`). What native code writes there
/// KFX reads, and the other way. Native code gives none back.
impl Native for crate::FlatArray<i64> {
    fn word(self) -> u64 {
        self.as_ptr() as u64 | kayak_jit::layout::TAG_BLOBLET
    }
    fn of_word(_: u64) -> Self {
        fail("native code: gave a flat array, which kayak cannot yet take back")
    }
}

/// A string, copied each way as one of Fixpt's layout (its code points two
/// to a word): FX-26's strings are never written.
impl Native for crate::Str {
    fn word(self) -> u64 {
        kayak_jit::string(self.as_str())
    }
    fn of_word(w: u64) -> Self {
        match kayak_jit::string_of(w) {
            Some(t) => crate::leak_string(t),
            None => fail("native code: gave what is not a string where one was wanted"),
        }
    }
}

/// A bloblet, its fields of what crosses: copied each way as a plain
/// bloblet of Fixpt's layout, its fields after a trailer (C116). A copy
/// that native code keeps does not see what KFX writes after, nor KFX
/// what native code writes after; native code's own, made and given back,
/// is KFX's alone from then on. A sealed copy is given as itself.
macro_rules! crossing_bloblet {
    ($n:expr; $($a:ident $i:tt),*) => {
        impl<$($a: Native + Copy + 'static),*> Native for &'static Bloblet<($(std::cell::Cell<$a>,)*)> {
            fn word(self) -> u64 {
                let (bytes, code) = self.native_parts();
                if let Some(c) = code {
                    return c.reference();
                }
                let fields: Vec<u64> = vec![$(self.fields.$i.get().word()),*];
                let suffix: Vec<u64> = bytes
                    .chunks(8)
                    .map(|w| w.iter().enumerate().fold(0, |x, (k, &b)| x | (b as u64) << (8 * k)))
                    .collect();
                kayak_jit::bloblet(&fields, &suffix, bytes.len())
            }
            #[allow(unused_variables)]
            fn of_word(w: u64) -> Self {
                let (bytes, slots) = match kayak_jit::header_of(w) {
                    Some((kayak_jit::layout::KIND_BLOBLET, f, b)) if f == $n + 1 || (f == 0 && $n == 0) => (b, f),
                    _ => fail("native code: gave what is not a bloblet of the fields wanted"),
                };
                let suffix: Vec<u8> = (0..bytes).map(|i| (kayak_jit::suffix_word(w, i / 8) >> (8 * (i % 8))) as u8).collect();
                crate::alloc(Bloblet::of_native(($(std::cell::Cell::new($a::of_word(kayak_jit::slot_of(w, $i + 2))),)*), &suffix))
            }
        }
    };
}
crossing_bloblet!(0;);
crossing_bloblet!(1; A 0);
crossing_bloblet!(2; A 0, B 1);
crossing_bloblet!(3; A 0, B 1, C 2);
crossing_bloblet!(4; A 0, B 1, C 2, D 3);

/// A procedure of KFX's, given to native code, which calls it by the
/// `foreign` routine: a closure of Fixpt's layout over it. Native code
/// gives none back to KFX (its checker sees to it).
macro_rules! given {
    ($($a:ident $i:tt),*) => {
        impl<$($a: Native + 'static,)* R: Native + 'static> Native for &'static dyn Fn($($a),*) -> R {
            fn word(self) -> u64 {
                kayak_jit::foreign_procedure(Box::new(move |args: &[u64]| {
                    if args.len() != given!(@count $($a)*) {
                        fail("native code: called a procedure with the wrong number of arguments");
                    }
                    self($($a::of_word(args[$i])),*).word()
                }))
            }
            fn of_word(_: u64) -> Self {
                fail("native code: gave a procedure, which kayak cannot yet take back")
            }
        }
        impl<$($a: Native + 'static,)* R: Native + 'static> Native for &'static (dyn Fn($($a),*) -> R + Send + Sync) {
            fn word(self) -> u64 {
                let f: &'static dyn Fn($($a),*) -> R = self;
                f.word()
            }
            fn of_word(_: u64) -> Self {
                fail("native code: gave a procedure, which kayak cannot yet take back")
            }
        }
    };
    (@count) => { 0 };
    (@count $x:ident $($y:ident)*) => { 1 + given!(@count $($y)*) };
}
given!();
given!(A 0);
given!(A 0, B 1);
given!(A 0, B 1, C 2);

/// `(native-convention)`: the clauses of the convention made code keeps
/// here, one a string, as Fixpt gives its own (kayak-jit's, which is
/// Fixpt's at the pin, C116).
pub fn native_convention() -> crate::List<crate::Str> {
    kayak_jit::DECLARATION
        .lines()
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .fold(crate::nil(), |acc, l| {
            crate::heap_cons(crate::leak_string(l.to_string()), acc)
        })
}

/// `(native-code?)`.
pub fn native_code_p() -> bool {
    kayak_jit::native_code()
}

/// A bloblet's fields as words of Fixpt's layout, beside a sealed copy's
/// code (C116): each of what crosses.
pub trait NativeFields {
    fn field_words(&self) -> Vec<u64>;
    fn field_word(&self, k: usize) -> u64;
}

macro_rules! native_fields {
    ($($a:ident $i:tt),*) => {
        impl<$($a: Native + Copy),*> NativeFields for ($(std::cell::Cell<$a>,)*) {
            fn field_words(&self) -> Vec<u64> {
                vec![$(self.$i.get().word()),*]
            }
            #[allow(unused_variables)]
            fn field_word(&self, k: usize) -> u64 {
                match k {
                    $($i => self.$i.get().word(),)*
                    _ => fail("a bloblet's field: no such field"),
                }
            }
        }
    };
}
native_fields!();
native_fields!(A 0);
native_fields!(A 0, B 1);
native_fields!(A 0, B 1, C 2);
native_fields!(A 0, B 1, C 2, D 3);

/// `(seal-code b)`: a copy of `b` in the code area, its suffix frozen and
/// runnable, its fields beside it (KFX's checker sees they cross).
pub fn seal_code<'a, Fs: NativeFields + Clone + 'a>(b: &Bloblet<Fs>) -> &'a Bloblet<Fs> {
    if !kayak_jit::native_code() {
        fail("seal-code: this machine runs no native code");
    }
    crate::alloc(b.sealed_copy())
}

/// A native procedure's call: its answer, or a failure, as Fixpt's machines
/// say its traps; or a routine kayak does not have yet.
fn run(clo: u64, entry: usize, args: &[u64]) -> u64 {
    match kayak_jit::call(clo, entry, args) {
        Ok(w) => w,
        Err(kayak_jit::Trap::Kind(k)) => fail(match k {
            1 => "evaluation step limit exceeded",
            2 => "stack overflow",
            3 => "a primitive failed",
            4 => "car or cdr of a non-pair",
            5 => "division by zero",
            _ => "native code trapped, of a kind the convention does not declare",
        }),
        Err(kayak_jit::Trap::Failed(m)) => fail(&m),
        Err(kayak_jit::Trap::Routine(r)) => fail(&format!(
            "native code called the `{r}` routine, which kayak's runtime does not have yet (C116)"
        )),
    }
}

/// What `native-procedure` is given: a bloblet, perhaps a sealed copy.
pub trait Sealed {
    fn sealed_code(&self) -> Option<kayak_jit::Code>;
}

impl<Fs> Sealed for Bloblet<Fs> {
    fn sealed_code(&self) -> Option<kayak_jit::Code> {
        self.code
    }
}

/// What `native-procedure` makes a procedure of: a sealed copy's code,
/// and its native closure.
fn sealed(c: &dyn Sealed) -> (u64, usize) {
    if !kayak_jit::native_code() {
        fail("native-procedure: this machine runs no native code");
    }
    match c.sealed_code() {
        Some(code) => (kayak_jit::closure(code), code.entry()),
        None => fail("native-procedure: not code that `seal-code` gave"),
    }
}

/// `(native-procedure c)` at `(subr (conv native) E () R)`.
pub fn native_procedure0<'a, R: Native>(c: &dyn Sealed) -> &'a (dyn Fn() -> R + Send + Sync) {
    let (clo, entry) = sealed(c);
    crate::lambda0s(move || R::of_word(run(clo, entry, &[])))
}

/// At `(subr (conv native) E (A) R)`.
pub fn native_procedure1<'a, A: Native, R: Native>(
    c: &dyn Sealed,
) -> &'a (dyn Fn(A) -> R + Send + Sync) {
    let (clo, entry) = sealed(c);
    crate::lambda1s(move |a: A| R::of_word(run(clo, entry, &[a.word()])))
}

/// At `(subr (conv native) E (A B) R)`.
pub fn native_procedure2<'a, A: Native, B: Native, R: Native>(
    c: &dyn Sealed,
) -> &'a (dyn Fn(A, B) -> R + Send + Sync) {
    let (clo, entry) = sealed(c);
    crate::lambda2s(move |a: A, b: B| R::of_word(run(clo, entry, &[a.word(), b.word()])))
}

/// At `(subr (conv native) E (A B C) R)`.
pub fn native_procedure3<'a, A: Native, B: Native, C: Native, R: Native>(
    c: &dyn Sealed,
) -> &'a (dyn Fn(A, B, C) -> R + Send + Sync) {
    let (clo, entry) = sealed(c);
    crate::lambda3s(move |a: A, b: B, d: C| {
        R::of_word(run(clo, entry, &[a.word(), b.word(), d.word()]))
    })
}

/// At `(subr (conv native) E (A B C D) R)`: tier 2's machine is given its
/// registers, heap and two stacks (C126).
pub fn native_procedure4<'a, A: Native, B: Native, C: Native, D: Native, R: Native>(
    c: &dyn Sealed,
) -> &'a (dyn Fn(A, B, C, D) -> R + Send + Sync) {
    let (clo, entry) = sealed(c);
    crate::lambda4s(move |a: A, b: B, d: C, e: D| {
        R::of_word(run(clo, entry, &[a.word(), b.word(), d.word(), e.word()]))
    })
}
