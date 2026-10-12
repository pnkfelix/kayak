# A uniform value beside unboxed generics: unions and first-class `poly` for KFX?

Status: research, 2026-10-12, the user's proposal: "a uniform boxed value,
but also the target procedures generic, so that they can be instantiated
at unboxed (`i64`, `f64`, `char`, `bool`, ...) when that much is known."
It reopens C57 (unions not in KFX) and TODO's first-class polymorphism.

## Findings first

1. **C57's reason was meaning, and in KFX the meaning cannot differ.** A
   coercion (a member boxed into its union, a list copied to a list of a
   union, a procedure wrapped) makes a new object where FX-26 has the old
   one. That is observable only by identity (`eq?`) or by writing through
   one alias and reading through the other. KFX has neither on what a
   coercion would touch: `eq?` and `set-car!` are refused for Rust (C69),
   and a reference is invariant (no coercion under `ref`). So a coerced
   value is the same value as far as any KFX program can tell. What
   differs is cost, not answers: K1 holds.
2. **The hybrid is the standard arrangement** (GHC's specialisation,
   .NET's generics, OCaml with flambda): code generic in its type
   variables, which rustc already monomorphises at `i64` and the rest
   wherever KFX's types are known; a uniform `Value` only where they are
   not, which is (a) a position of a union's type and (b) a first-class
   `poly` value, instantiated at `Value` (KFX's type variables are `Copy +
   'static` only, which `Value` is).
3. **The coercions, at the subsumption points the checker already finds**
   (`fits`), told to the emitter as facts:
   - a member to its union: boxed (`Value::Int(n)`), O(1);
   - a union narrowed to a member (a test, a `typecase`): unboxed by a
     `match`, O(1);
   - a list of members to a list of the union (covariance): O(n) if
     copied; O(1) with a *view*, a `Value` variant that holds the typed
     list and boxes each element as it is read (a gradual-typing runtime's
     proxy; its identity differs, which KFX cannot see);
   - a procedure to one of wider parameters (contravariance): a wrapper
     closure, O(1), as the view;
   - a first-class `poly` value: made from a generic procedure by
     instantiating it at `Value`, and a `proj` of it at `int` a wrapper
     that boxes going in and unboxes coming out; where the `poly` value is
     a known global, `proj` calls the generic at `i64` directly, unboxed.
4. **The cost** is allocation (views and wrappers, never freed under C43),
   a box per scalar in a union, and dispatch on `Value`'s tag; none where
   types are known, as now.

## What it would take

- `kayak-rt`: `Value` (an enum over the shapes, a view variant), the
  boxing and unboxing of each base type and datatype, list views.
- The checker: subsumption points recorded as facts (member to union, list
  to list of union, procedure to procedure, `poly` value to its instance).
- The emitter: the coercions written at those facts; `Value` for union
  types and for `poly` types; generic procedures as now.
- The restriction pass: unions, `typecase`, `datum`, first-class `poly`
  no longer refused (C57 superseded); `eq?`, `set-car!`, `set-cdr!` still
  refused, since the argument above rests on their absence.

## Recommendation

Worth doing, after the engine's current growth, for `datum` above all: a
compiler written in KFX reads S-expressions, and `datum` with `quote` is
FX-26's own way to hold them (kfx has its own `sexp` datatype, which
`datum` could replace when KFX has it). A first step, small and measured:
unions of scalars and `datum` with views, in the corpus beside Fixpt's
answers; first-class `poly` after. The user's call: it reverses C57.

## Decided (the user, 2026-10-12)

Planned after the checker's current growth for the engine (C68–C72 and
what follows), not before.

## Step 1, done (C75)

Unions with scalars, lists, procedures, references, arrays, sums and
products as members; `typecase`; the shape tests; `false`; a `pairof`
whose tail is a list. kayak-rt's `Value` (`crates/kayak-rt/src/value.rs`);
facts from the checker at `check`, `check-by-synth`, narrowed locals,
`typecase` arms, calls of a narrowed procedure; the emitter's coercions at
them. No views yet: a list of members given as a list of the union is
still a type error for rustc, not refused by kfx. 114 of Fixpt's programs
compile and agree (108 before).

## Step 2, done (C77)

`datum`; a pair whose tail is no list as a `Value` (a `PairLike`: a pair
of `Value`s, or a typed list's own pair seen as one, a view); `car` and
`cdr` of a union; `quote` of any datum; `typecase` as FX-26 writes it. 117
of Fixpt's programs agree through kfx, 109 through the engine.
