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

/// `(native-code?)`.
pub fn native_code_p() -> bool {
    kayak_jit::native_code()
}

/// `(seal-code b)`: a copy of `b` in the code area, its suffix frozen and
/// runnable. A bloblet of no fields (KFX's checker sees to it).
pub fn seal_code<'a>(b: &Bloblet<()>) -> &'a Bloblet<()> {
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
        Err(kayak_jit::Trap::Routine(r)) => fail(&format!(
            "native code called the `{r}` routine, which kayak's runtime does not have yet (C116)"
        )),
    }
}

/// What `native-procedure` makes a procedure of: a sealed copy's code,
/// and its native closure.
fn sealed(c: &Bloblet<()>) -> (u64, usize) {
    if !kayak_jit::native_code() {
        fail("native-procedure: this machine runs no native code");
    }
    match c.code {
        Some(code) => (kayak_jit::closure(code), code.entry()),
        None => fail("native-procedure: not code that `seal-code` gave"),
    }
}

/// `(native-procedure c)` at `(subr (conv native) E () R)`.
pub fn native_procedure0<'a, R: Native>(c: &Bloblet<()>) -> &'a (dyn Fn() -> R + Send + Sync) {
    let (clo, entry) = sealed(c);
    crate::lambda0s(move || R::of_word(run(clo, entry, &[])))
}

/// At `(subr (conv native) E (A) R)`.
pub fn native_procedure1<'a, A: Native, R: Native>(
    c: &Bloblet<()>,
) -> &'a (dyn Fn(A) -> R + Send + Sync) {
    let (clo, entry) = sealed(c);
    crate::lambda1s(move |a: A| R::of_word(run(clo, entry, &[a.word()])))
}

/// At `(subr (conv native) E (A B) R)`.
pub fn native_procedure2<'a, A: Native, B: Native, R: Native>(
    c: &Bloblet<()>,
) -> &'a (dyn Fn(A, B) -> R + Send + Sync) {
    let (clo, entry) = sealed(c);
    crate::lambda2s(move |a: A, b: B| R::of_word(run(clo, entry, &[a.word(), b.word()])))
}

/// At `(subr (conv native) E (A B C) R)`.
pub fn native_procedure3<'a, A: Native, B: Native, C: Native, R: Native>(
    c: &Bloblet<()>,
) -> &'a (dyn Fn(A, B, C) -> R + Send + Sync) {
    let (clo, entry) = sealed(c);
    crate::lambda3s(move |a: A, b: B, d: C| {
        R::of_word(run(clo, entry, &[a.word(), b.word(), d.word()]))
    })
}
