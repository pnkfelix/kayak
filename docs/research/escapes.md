# Escapes: FX-26's prompts and aborts, as Rust

Status: a design note (2026-10-11), before building; `TODO.md` §16. The
decision it asks for is in §5.

## 1. What FX-26 has

From `fixpt/docs/fx26.md`, "Control, typed":

- A **prompt tag** lives in a region and fixes what crosses its prompts:
  `(prompt-tag A H D R)`, its prompts delivering an `A`, its aborts
  carrying an `H`, its delimited computations of effect at most `D` apart
  from control on `R`.
- `(prompt tag body handler)` is syntax: it runs `body`; an abort to `tag`
  inside it unwinds to here, and `handler` gets the payload. It removes
  `(goto R)` and `(comefrom R)` from the body's effect, when the body can
  reach no other tag of `R`.
- `abort-current-continuation` has the effect `(goto R)` and returns
  `void`. `call-with-composable-continuation` has `(comefrom R)`, and the
  continuation it captures, a `(composable T A D R)`, may be called again.
- Marks (`with-mark`, `first-mark`, ...) are locations in the dynamic
  context.

## 2. What maps, and what does not

**Escapes map.** An abort that only leaves, to a prompt that is still
running, is an early return: `Result` and `?`, or an unwinding panic.
**Composable continuations do not**: one called after its prompt has
returned resumes a computation whose Rust frames are gone. Rust has no
one-shot or multi-shot continuations short of a CPS transform or an
explicit stack, neither of them a trivial mapping. So KFX's escapes are:
tags, prompts, aborts; not `call-with-composable-continuation`, not
`cwcc` (its continuation may be kept, `comefrom`), not marks for now.

The checker's line between them is FX-26's own: an expression whose
effect has `(comefrom R)` keeps a continuation, and KFX refuses it; one
with only `(goto R)` escapes.

## 3. Two Rust forms

### 3.1 `Result`, by effect

A procedure whose latent effect has `(goto R)` returns
`Result<T, Unwind>`; a call of one is followed by `?`; an abort is
`return Err(Unwind::new(tag, payload))`; a prompt is

```rust
match (|| -> Result<A, Unwind> { Ok(body) })() {
    Ok(v) => v,
    Err(u) if u.is(tag) => handler(u.payload::<H>()),
    Err(u) => return Err(u),     // to another tag, if the effect says one can
}
```

`Unwind` carries the tag's identity and the payload, erased
(`&'static dyn Any`, which every KFX value is, being `'static`), so that
aborts to tags of different payload types share a `Result`.

This is the mapping the plan has meant since draft 1: **effects decide
code generation**. It is exact (an abort is a return, nothing else is
unwound), costs nothing where nothing escapes, and is what Rust programs
write by hand. What it needs:

- **The emitter knows effects.** Which procedures return a `Result`, which
  calls take a `?`, and which procedure *types* have a `Result` (a
  `(subr (goto @k) (i64) i64)` is a `&dyn Fn(i64) -> Result<i64, Unwind>`).
  Effects can come through abbreviations (`define-effect`), so the type's
  text does not say; the checker must. The facts the checker records by
  place (`CLAUDES_CHOICES.md` C15) carry it: a fact at each `subr` type's
  place, at each call's, and at each definition's. Every list has a place
  already.
- **Tail positions return `Ok(...)`**, in a looping procedure too
  (`return Ok(v)`), and a self tail call in a `Result` procedure is still
  a loop.
- **A lambda of a `goto` effect** is a closure returning a `Result`; the
  `lambdaN` helpers are generic in the result already.

### 3.2 Unwinding panics

An abort is `std::panic::panic_any(Unwind{..})`; a prompt is
`catch_unwind`, rethrowing what is not its tag's. Nothing else changes:
no `Result`, no `?`, no facts.

It is a trivial mapping, but it is not the plan's: effects decide
nothing; a panic is slow, and needs `panic = "unwind"`; `catch_unwind`
wants its closure `UnwindSafe`, which closures over cells are not
(`AssertUnwindSafe` would say what is not checked).

## 4. Tests that exist

Fixpt's `tests/programs/control/` (`own-abort.fx`, `other-tag.fx`,
`fresh-tag.fx`) and `pldi89/` (C4, C6, C7, all with `cwcc`, so KFX would
refuse them: `comefrom`). `docs/survey.md` counts 14 programs whose first
refusal is an effect, most of them `goto`.

## 5. To decide

1. **3.1, `Result` by effect** (recommended, and the plan's), or 3.2,
   unwinding panics, quicker but against the grain of the project.
2. Whether a procedure with a `goto` effect named as a value
   (`&dyn Fn ... -> Result`) is in the first step, or only direct calls.
3. Whether `Unwind`'s payload is erased (`dyn Any`, one `Result` type for
   every escape) or typed per tag region (a `Result<T, E_R>` per region,
   exact, but a procedure aborting to two regions needs a sum).
