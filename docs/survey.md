# What KFX lacks, by Fixpt's test programs

**Rerun 2026-10-11, at the end of the day** (Fixpt `95c5605`): `kfx`
accepts 73 of the 286; the 72 of them `scripts/run-fixpt-programs.py`
runs (it skips those meant to run for ever) all agree with Fixpt, value
for value. (The table is of an earlier run that day, at 71.) The first
refusals now:

| programs | first refusal                                    | what it would take                                     |
| -------: | ------------------------------------------------ | ------------------------------------------------------ |
|       21 | no type named: `datum` (8), `symbol`, `i32`, `u64`, `f32` | dynamic data and its quoted syntax; fixed widths  |
|       19 | nothing named: `extend`, `set-car!`, `apply`, …  | `extend`; pairs that may be written; the rest small    |
|       13 | bloblets                                         | flat data beyond `i64` arrays (`TODO.md` §15)          |
|       10 | `letrena`                                        | places (`TODO.md` §14, C23: not yet)                   |
|       10 | `poly` over effects, sizes, places               | effect polymorphism, and the rest                      |
|        8 | `load-module`                                    | files as modules                                       |
|        7 | `nat` facts from tests                           | FX-26's size facts (refinement by tests)               |
|        5 | unions                                           | `(union ...)` and `typecase`                           |
|        5 | `proj`                                           | explicit instantiation                                 |
| the rest | modules as values, `pairof`, higher kinds, …     |                                                        |

The table below is the first survey's, the same day, when `kfx`
accepted 1.

Measured 2026-10-11, at Fixpt `95c5605`, by `scripts/survey-fixpt.py`:
each of the FX-26 programs in Fixpt's `crates/fixpt-fx26/tests/programs`
that Fixpt checks is given to `kfx`, and its first refusal kept.

Fixpt's tests are written in FX-26 at large, so KFX, a kernel, accepts one
of the 286 (`module-files/answer-types.fx`). The first refusals rank what
FX-26 at large asks for first (`TODO.md` §16):

| programs | first refusal                                         | what it would take                                          |
| -------: | ----------------------------------------------------- | ----------------------------------------------------------- |
|       70 | a source not ASCII (comments, mostly)                 | strings of characters, not bytes (`CLAUDES_CHOICES.md` C4)  |
|       48 | `int`                                                 | a bignum in `kayak-rt`, and `int` back in the language      |
|       43 | `define*`, whose globals read are found               | the checker finding them; a declaration form                |
|       36 | `(define x e)`, with no type                          | the emitter knowing the checker's types                     |
|       14 | a type with parameters (`define-type`, datatypes)     | type families, as Rust generic types                        |
|        8 | bloblets                                              | flat data beyond flat arrays (`TODO.md` §15)                |
|        8 | an effect KFX does not have (effect variables, …)     | effect polymorphism                                         |
|        7 | a `poly` over a kind KFX does not have                | sizes, effects, places, `distinct`                          |
|        6 | a list at a region but `acyclic`                      | lists that may be written                                   |
|        5 | `define-generative`                                   | generative types, as Rust newtypes                          |
|        4 | prompt tags                                           | delimited control (`TODO.md` §16)                           |
| the rest | modules, sizes, unions, sums and products, I-cells, … |                                                             |

A first refusal hides the rest of a program, so the counts after the
first rows say less than they seem; and the ranking is of Fixpt's tests,
which exercise features, not of programs written to be useful.

Before this survey, `kfx` crashed on 70 of these programs, on a source's
first non-ASCII byte; it now refuses them, saying where. And a type with
parameters was refused with no place given; it now has one.
