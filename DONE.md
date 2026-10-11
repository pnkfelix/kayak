# Done

What [`TODO.md`](TODO.md) listed, once it was finished, under the same
section number. Tasks tried and abandoned are here too, each with a short
note on what was explored and why it was dropped. Sections are in number
order, and each says when it was done.

---

## 1. The plan, drafts 1 and 2 (the user, 2026-10-11)

The user's brief: a sublanguage of FX-26, KFX, each of whose constructs
maps trivially to something in Rust; a transpiler from KFX to Rust; the
transpiler written in KFX, its Rust generated from that source; and from
there, a self-hosted compiler for the broader FX-26.

Draft 1 planned broadly: a mapping for most of FX-26, eight milestones,
and a hand-written stage 0 in Rust. The user's direction replaced it:
*close the local loop as fast as possible*, the shortest path to KFX
source that transpiles to the Rust that builds it. Draft 2 is that path,
with everything else deferred.

Also from the user: Fixpt and coopt move fast on their own, so analysis
is done on clones, in `~/Dev/Rust/kayak-refs` (`CLAUDES_CHOICES.md`, C1).

**Abandoned: a hand-written stage 0.** Draft 1's `kfx0`, a transpiler
written in Rust, would have meant writing the transpiler twice. A probe
showed that Fixpt can run a KFX transpiler directly, since KFX is
FX-26, so stage 0 is Fixpt (C2).

## 2. The closed loop (2026-10-11)

- `crates/kayak-rt`: `List`, `Cons`, `Str`, allocation by `Box::leak`,
  and the primitives the compiler's source uses, each named by the
  mangling (`PLAN.md` §2).
- `kfx/kfx.kfx`: the transpiler, in KFX-0, about 600 lines:
  - a reader, from characters to a `sexp` datatype;
  - a first pass, noting datatypes' tags and constants;
  - an emitter, writing Rust onto a list of characters newest first,
    reversed once at the end.
- `scripts/stage0.py`: Fixpt runs the compiler on its own source, and the
  result is `boot/src/kfx.rs`. FX-26 has no file I/O, so the source goes
  in as a string literal and the Rust comes out as the last form's value.
- `boot/`: the generated `kfx.rs`, and a hand-written `main` of ten lines.
- `scripts/fixpoint.sh`: the built `kfx` makes `boot/src/kfx.rs` again,
  byte for byte, and so does Fixpt running the source.

Fixpt accepted the compiler as FX-26 with one change: its `length` took
only an `nlist`, so the compiler counts with its own `size` (§6).
Building the generated Rust gave no errors the first time, and the
fixed point held at once. Stage 0 takes 0.13 s; compiling the compiler
peaks at 10 MB.

## 3. Comments that survive, and an emitter that indents (2026-10-11)

The user noticed that comments were lost: the reader drops `;` comments.

- **Comments are strings**, where FX-26 accepts one and a body's value is
  not taken. FX-26 checks a string there as pure, its value dropped.
  - A string at the top level becomes a `//` comment.
  - A string first in a procedure's body, with more after it, is its
    docstring: `///` on the `fn`.
  - Any other string in a body, not last, becomes a `//` comment.
- **The emitter indents**, in rustfmt's style. A body's last `let`,
  `let*`, `begin` or `tagcase` is written into the enclosing block. The
  generated `kfx.rs` went from one line per function to 1074 lines.

Done through the bootstrap in two steps: first the compiler learnt to
write comments, and was regenerated; then its own comments became
strings (a one-off script), and it was regenerated again.

**Abandoned: a string before a definition as its doc comment.** A section
header such as `"Reading"`, followed directly by `define-datatype sexp`,
would have become `sexp`'s documentation. The docstring convention
avoids that.

**Abandoned: running `rustfmt` on the output.** The fixed point would then
depend on `rustfmt`'s version as well as on the compiler.

## 4. The Fixpt pin: `FIXPT_REV` (the user, 2026-10-11)

The user asked how kayak tracked its dependency on a version of Fixpt.
It did not: the pin was a sentence in `PLAN.md`, and stage 0 ran
whatever binary was in the clone.

- `FIXPT_REV` holds the full hash of the commit stage 0 runs.
- `scripts/fixpt.sh` checks that the clone's `HEAD` is that commit, with
  no changes of its own, builds it, and prints the binary's path. It never
  checks out by itself. With no clone it exits 3, and `fixpoint.sh` then
  skips stage 0, saying so; at another commit it fails, and says how to
  check out or move the pin.
- The `KAYAK_FIXPT` override, which could name any binary, is gone;
  `KAYAK_FIXPT_CLONE` names the clone instead.

Tested: at the pin, both checks; with no clone, stage 0 skipped; at
another commit, a failure naming both commits.

## 5. No `int`: `i64`, and the prelude (the user, 2026-10-11)

KFX-0 first mapped `int` to an `i64` whose overflow panicked: K1's one
licence, since FX-26's `int` is a bignum. The user's ruling: `+` on `int`
implies a meaning Rust's does not have, and KFX is the kernel whose
meaning is known because its mapping is. So KFX writes `i64+` and the
rest, and has no `int`.

- FX-26's `i64` operations, probed at their edges, are Rust's `wrapping_*`
  exactly: wrapping `+ - *`; truncating `quotient` and `remainder`, with
  `MIN / -1` giving `MIN` and remainder 0; an error on division by zero;
  shifts counted modulo 64, `shr` arithmetic.
- `kfx/prelude.fx`, FX-26 and not KFX, defines `string-length-i64`,
  `string-ref-i64`, `char->i64` and `i64->string` from the standard
  environment's `int` primitives. Fixpt reads it before a KFX program; the
  transpiler never sees it, since each name is a `kayak-rt` function.
- Done through the bootstrap in three steps: the compiler learnt `i64`
  (and that a `-` ending a name is `_minus`, so `i64-` is `i64_minus`);
  its source moved to `i64`; then `int` became an error, and the `int`
  primitives left `kayak-rt`.

**Abandoned: `int` as an `i64` that panics on overflow** (above).

**Not asked for: `i64` twins of Fixpt's primitives.** FX-26's whole string
and character interface is at `int`. Twins of all of it would duplicate a
large part of the standard environment for one client, so the conversions
are kayak's prelude.

## 6. Requests for Fixpt (2026-10-11)

Written as feature requests, which the user passed to Fixpt's workers.
None was blocking. Each, once landed, was pinned and checked:

1. **A literal checks as a fixed-width type that is expected**: `fb86975`.
   `(i64+ 1 2)` is `3 : i64`, and `(define y u32 -1)` is refused with the
   range. A real literal against `f32` is still to come (Fixpt's own
   note).
2. **`fixpt eval --raw`**: `e287634`. Stage 0 uses it, and the decoder of
   printed strings in `stage0.py` is gone.
3. **`length` of a list**: `b5ae910`. `length` takes a list at `acyclic`,
   and of a list at a region the error names `list-length`. The compiler
   keeps its own `size`, since `length` gives an `int`.

The pin is now `35b071f`.

## 8. A corpus of KFX programs (2026-10-11)

`tests/kfx/NAME.kfx`, each a program whose `main` gives its answer as a
string, and beside each, kept by `scripts/corpus.py`:
- `NAME.rs`, the Rust `kfx` makes of it, golden: a change to the compiler
  shows in every program it changes;
- `NAME.out`, its answer as Fixpt gives it, running the prelude and the
  program as FX-26.

`crates/kfx-corpus` builds every `NAME.rs`, and its test requires each
one's answer to be `NAME.out`: K1, program by program. `--bless` rewrites
the files instead of checking them. Eight programs, one for each part of
the mapping: `i64` arithmetic at its edges, control, `let` and `let*`,
datatypes, lists, strings and characters (escapes included), `define-rec`
and a 200000-deep recursion, and the mangling. All eight gave Fixpt's
answers at once. A planted wrong answer is caught by both the test and the
script.

What writing them taught is in `TODO.md` §17: FX-26's `case` takes no
`i64` key, a literal with nothing expected is an `int`, FX-26's keywords
(`sum`) cannot name procedures, and shifts take an `int` count.

## 9. Guardrails (2026-10-11)

- `scripts/check.sh`: `cargo fmt --check`, clippy with `-D warnings`, the
  tests, `scripts/corpus.py`, and `scripts/fixpoint.sh`. Run before every
  commit.
- `unsafe_code = "deny"` across the workspace. Generated code is compiled
  in `boot` and `kfx-corpus`, so K4 is rustc's to enforce.
- `kfx-corpus`'s `tidy` test: KFX sources are ASCII (C4).
- Generated modules are `#[rustfmt::skip]`, since their layout is the
  compiler's, and allow every warning and clippy lint: generated code
  says what its source says (`true && true`, for `(and #t #t)`). Which
  warnings it should be free of is still open (`TODO.md` §17).

**Abandoned: a test that generated Rust has no `unsafe` in it, by text.**
The compiler's list of Rust's keywords has the string `"unsafe"`; the lint
is the check.

## 10. Self tail calls as loops (2026-10-11)

A procedure that calls itself in tail position is a Rust `loop`: the call
assigns the parameters it changes, all at once (`(a, b) = (b, a)`), and
`continue`s; whatever else it could end with is `return`ed. Tail positions
are a body's last expression and the arms of `if`, `cond`, `case`,
`tagcase`, `let`, `let*` and `begin`. Under a binding that hides a
parameter, a self call stays a call, since an assignment would reach the
binding. A pre-scan (`tail-calls?`) decides whether a procedure loops, so
one that does not is written as before.

In the emitter, a `tail` datatype says where an expression stands:
`value`, or `returns`, with the procedure's name and parameters and
whether they are hidden.

26 of the compiler's own procedures loop. `tests/kfx/tails.kfx` has each
case (unchanged arguments, a swap, tail calls in `cond`, `case` and
`tagcase`, a hidden parameter), and `kfx-corpus` runs it on a stack of
256 KiB, which three million frames would overflow.

The bootstrap caught a bug that Fixpt's check could not, since Fixpt only
checks the compiler and does not run it on itself: the pre-scan gave a
`case`'s key to the walk of its arms. Generation 1 failed on its own
source. And the regenerating loop had copied that failure's empty output
over the seed; restored from git, and generations are now kept apart.

## 11. First-class procedures, and 12. `poly` (2026-10-11)

- A `(subr E (T…) R)` that is a value's type is `&'static dyn Fn(T…) -> R`:
  `Copy`, as every KFX value is.
- A `lambda` that is a value is `lambdaN(move |x, …| { body })`. The
  helpers in `kayak-rt` take an `impl Fn` and give the `&'static dyn Fn`,
  so that rustc finds the closure's parameter and result types from where
  it is used, as FX-26's checking does: probed by hand first, `nil()` in a
  closure's body included. A parameter written `(x T)` is `x: T`.
- A global procedure named as a value is `&f`: a `fn` item promoted to
  `'static`, coerced to `dyn Fn`.
- A call whose operator is an expression, `((adder 3) 4)`, is
  `(adder(3i64))(4i64)`.
- A definition's type `(poly ((a type) …) T)` makes the `fn` generic,
  `<A: Copy + 'static, …>`; an effect binder is erased, a region one
  refused. Its value may be a `lambda` (FX-26 puts the binders in scope)
  or a `plambda`. Instantiation is inferred, by FX-26 and rustc alike.

**Scope.** That a name is a global procedure, not a local, is something
the emitter had not known. A procedure the program defines could be
listed, but a primitive (`i64->string`, named as `map`'s argument) cannot,
since `kayak-rt` is not read. So the emitter now keeps the names bound
where it writes (parameters, `let`, `let*`, `tagcase`'s fields): a name is
a local, a constant, or else a global procedure. This also mends a hazard
`TODO.md` §17 listed, a local named like a constant.

`tests/kfx/procs.kfx`: `map` and `fold` over several types, a global
procedure, a primitive and lambdas as arguments, a capturing closure, one
that makes another, a procedure in a datatype, and an expression as an
operator. Its first run found the primitive case.

`scripts/regen.sh` regenerates through the bootstrap safely: only a
successful run's output replaces the seed, and on failure the seed is
restored.

Then the second step of the bootstrap: the compiler's own source uses
them. A generic `map`, `any?`, `size` and `last` replace six typed copies
of the same walks (`sym-names`, `param-names`, `strings-size`, …), and one
higher-order `put-sep` writes every comma-separated list (names, types,
expressions). FX-26 infers the lambdas' parameter types from `put-sep`'s
signature, as rustc does from the generated `lambda2`.

## 13. A KFX checker, in KFX (2026-10-11)

`kfx` checks a program's types and effects before it writes any Rust, and
refuses, naming the definition and saying why, what is not KFX: so it
needs neither Fixpt nor rustc's messages to refuse. About 700 lines of
the compiler's 1900; bidirectional, as FX-26's checking is. A `poly`
procedure's call is instantiated with inference variables, solved as the
arguments are checked; effects are sets of atoms (`spin`, `@globals`, a
global read), and a procedure's body's effect must be within its type's.
KFX's types are those of PLAN.md §2, its lists at `acyclic`, the
primitives typed in a table at KFX's types (where making and reading
lists at `acyclic` are pure).

K1 is that it accepts nothing FX-26 refuses. Where FX-26 is clever, it is
plain: every recursion says `spin` (FX-26 proves termination by
size-change); a call of a procedure value, and a global procedure named
as a value, are `spin`; `poly` is over types. And two rules come from
Rust, not FX-26:
- **A call's arguments are checked in order.** FX-26 checks lambdas last;
  rustc finds a closure's parameter types only from the arguments before
  it, since the generated closure is inside a `lambdaN(…)` call, checked
  in order. So a lambda parameter whose type no earlier argument gives
  must be written `(x T)`. Found when generation 1 did not build: four
  of the compiler's own lambdas were so.
- **A keyword may name a local but not be called** (`(sum 1 2)` is FX-26's
  sum form, whatever `sum` is bound to), nor name a global. The first
  version refused keywords as locals altogether; the compiler's own
  `put-define` has a parameter named `type`, which FX-26 accepts.

Also: a number literal past an `i64` is refused as it is read; digits are
accumulated negated, so that `-9223372036854775808` reads.

Tests, in `scripts/corpus.py`:
- the corpus and the compiler itself, which it accepts;
- `tests/kfx-refused`, 14 programs FX-26 refuses, which it must refuse
  too: wrong types, unbound and forward names, effects, `spin`, `cond`
  without `else`, `tagcase` arms missing, mistaken and of the wrong shape,
  arity, a literal past `i64`, branches, a lambda's effect, a keyword
  called. Each refused by both, for the same reason;
- `tests/kfx-stricter`, 4 programs FX-26 accepts and KFX refuses on
  purpose: `int`, a literal with nothing expected, a recursion FX-26
  proves ends, and a lambda before the argument that types it.

The bootstrap's second step needed none: the checker accepted the
compiler as soon as it built. Stage 0 now runs it too, under Fixpt, in
0.75 s; the Rust compiler checks and translates its own source in 10 ms.

## Programs of several files, and the compiler in six (2026-10-11)

Done at the user's question of how to break up the compiler, without a
TODO number of its own.
- A manifest, `NAME.unit`, lists a program's files in order; `kfx
  PROGRAM.unit DIR` writes a module per file and a `mod.rs`
  (`compile-units`). Each module imports the others' names by glob, so
  the checker refuses a name defined twice, a primitive defined again,
  and `kayak-rt`'s own names (`tests/kfx-stricter`).
- Stage 0 gets one string from Fixpt, so the outputs are bundled, each
  after a line beginning `@@kfx-file`. The first splitter split anywhere
  the marker appeared, and the compiler's own docstring and string
  literal contain it: it now splits only at line starts, which no line of
  generated Rust begins with.
- The compiler is `kfx/base.kfx`, `read.kfx`, `names.kfx`, `check.kfx`,
  `emit.kfx` and `compile.kfx`, and `boot/src/kfx/` a module of each. The
  checker had borrowed `param-name` and `procedure?` from the emitter;
  they moved to `read.kfx`, so the checker comes before the emitter.
- In two steps, as ever: the compiler learnt manifests as one file (and
  `boot/src/kfx.rs` became `boot/src/kfx/`, one module), then was split.
  The corpus gains `tests/kfx/two`, a program of two files whose second
  takes apart the first's datatype.

## 7. Errors that point at KFX source (2026-10-11)

`kfx`'s messages now say where: `kfx: tests/kfx-refused/branches.kfx:3: in
f: a string is here, where a i64 is expected`.
- A list remembers where it began, `(lst sexps file line)`: the file's
  name (a shared reference, nothing allocated) and the line of its `(`.
- The reader counts lines as it reads: `skip` counts the newlines it
  passes, `string-body` those in a string, and each result carries the
  line it leaves off at. Its own errors (an unclosed string or list, a
  literal past `i64`, an unknown character name) say the line.
- The checker's `cv` carries the position of the innermost list being
  checked, set as `synth`, `check` and `check-top` enter one; `refuse`
  says it, then the definition.
- `compile-named` names the file; `kfx FILE.kfx` gives it the path, a
  manifest's files their own names.
- `scripts/corpus.py` requires every refusal, of the 20, to say its file
  and line.

What is left is in `TODO.md` §7: positions of atoms, columns, and
positions in generated Rust.

## 15. Mutable data: flat arrays of i64, and references (2026-10-11)

KFX's first mutable data, and its first regions.
- `(ref T R)` with `new`, `get` and `set`, and `(flatarrayof i64 R)`
  through the prelude's `make-flatarray-i64`, `flatarray-ref-i64`,
  `flatarray-set-i64!` and `flatarray-length-i64` (FX-26's take `int`s).
  In Rust, cells (C9): `&'static Cell<T>`, `&'static [Cell<i64>]`, bounds
  checked as FX-26 checks them.
- Regions: constants `@a`, and `(r region)` binders of a `poly`, erased in
  Rust (C10). The checker has region types and effect atoms `read:R`,
  `write:R`, `alloc:R`; a call instantiates its region binders with
  inference variables, which the expected type, fitted first, and the
  arguments solve; its latent effect then names the regions found, and
  one still unsolved is refused (C13).
- A constant may only read globals (C11).
- **A K1 hole, closed:** the prelude's names had been bound as primitives,
  with no effect; to FX-26 they are globals, and calling one reads it. A
  `pure` procedure calling `string-length-i64` was accepted by `kfx` and
  refused by Fixpt (`tests/kfx-refused/prelude-global`).
- `tests/kfx/mutable.kfx`: arrays filled, summed and sorted in place by
  region-polymorphic procedures, copied across two regions, a cell
  written through one name and read through another, a cell in a
  datatype, and closures sharing it. Rust's answer is Fixpt's.
  Refused by both: a write and a read the effect does not say, and the
  prelude case; by KFX alone: a constant making an array, a `distinct`
  binder, a region no one names.

Also: the pin moved to Fixpt `95c5605`; `onto` and `concat` moved to
`base.kfx`, the checker needing them.

## A survey of Fixpt's programs; refusals that do not crash (2026-10-11)

`scripts/survey-fixpt.py` gives `kfx` each FX-26 test program Fixpt
checks, and counts first refusals: `docs/survey.md`. KFX accepts 1 of 286,
being a kernel; the ranking says what FX-26 at large asks for first. It
found `kfx` crashing on 70, at a source's first non-ASCII byte: the driver
now refuses such a source, naming its line. Also: a name expected where
there is none, and a type with parameters, are refused with a place;
FX-26's forms that KFX lacks (`define*`, `proj`, `module`, …) are named as
such, not as calls of keywords.

## Strings of characters (2026-10-11)

C14. In three steps through the bootstrap: the emitter wrote literals as
`str_lit("…")`, and the reader took its source with FX-26's `string->list`
(a primitive the checker had to know before the compiler could use it:
the first try failed at generation 1, as it should); then `Str` became a
struct that knows whether it is ASCII; then the driver's ASCII guard went,
and the tidy test with it. `tests/kfx/unicode.kfx` takes lengths, indices
and `char->i64` of strings that are not ASCII, with Rust's answers
Fixpt's.

## Definitions with no type: elaboration (2026-10-11)

`(define x e)` is KFX now. The checker finds `e`'s type, as FX-26 does;
the emitter, which knows no types, is told it, as though it had been
written: `check-program` gives the elaborated types (`typed`), `ctx`
carries them, and `put-top` writes such a definition with its type. The
type is never `poly`, must be known whole (no inference variable left),
and the name is not in scope in `e`, a lambda's body included, as in
FX-26. A constant's value may only read globals, as a typed constant's
(C11). `tests/kfx/untyped.kfx`, and `kfx-refused/untyped-recursive`.

## `define*` (2026-10-11)

`(define* name type lambda)` is KFX: the body is checked as though its
type read every global, and the global is then of the type written, its
effect with the globals found (`check-define*`, after FX-26's "checked
twice"). In Rust, a `fn`, as `define`. `tests/kfx/define-star.kfx`: pure
procedures reading globals, one calling the other; and
`kfx-refused/define-star-write`, a write no effect covers.

## `int`, and the checker's facts (2026-10-11)

C15. `int` is `kayak-rt`'s bignum (`crates/kayak-rt/src/int.rs`, with unit
tests), with `+`, `-`, `*`, `quotient`, `remainder`, `modulo`, `=`, `<`,
`>`, `<=`, `>=`, `int->string`, `int->i64` and `i64->int`. A literal with
nothing expected is an `int`, as in FX-26 (so `tests/kfx-stricter/int` and
`let-literal` are gone: KFX accepts them). The reader gives places
(file, offset) to lists and numbers; the checker's facts about places go
to the emitter (`ctx-fact?`), the first being which literals are `int`s.
`tests/kfx/bignum.kfx`: factorial 30, Fibonacci 150, powers, division of
every sign, wrapping to `i64`, comparisons across the `i64` boundary,
with Rust's answers Fixpt's. The first run lost the facts of a list's
elements: `elements` dropped its state, harmless until the state carried
facts; it threads it now.

## Fixpt's programs, run both ways (2026-10-11)

`kfx --toplevel FILE` adds a `Show` for each datatype and `kfx_toplevel`,
which writes the values of the program's top-level expressions as `fixpt
eval` does (`kayak-rt`'s `Show`, `#<sum TAG>`, `#<box V>`, FX-26's string
and character syntax). `scripts/run-fixpt-programs.py` compiles each of
Fixpt's test programs that `kfx` accepts, builds them into one runner,
and compares each one's values with Fixpt's, by aligning `fixpt eval`'s
lines with the program's forms (a definition prints a line for each name
it binds, `define-effect` and `define-type` none). A procedure is any
`#<...closure...>`. Fixpt's `diverge/` programs, meant to loop, are
skipped. 17 programs, all agreeing: K1 on programs not written for
kayak. `scripts/check.sh` runs it when there is a Fixpt clone.

## Type families, and lists at any region (2026-10-11)

C17. Datatypes and type abbreviations with type and region parameters:
in the checker, `ty-data` has arguments, a `tagcase` instantiates its
variants' fields, an abbreviation applied is expanded; in Rust, generic
enums, constructors, aliases and `Show`s, regions erased. `replace-var`
moved ahead of the type parser, whose abbreviations need it.

With them, lists at any region, which a rose tree over a region needs:
`cons` and `list` allocate at the list's region and `car` and `cdr` read
it, as in FX-26; effects on `acyclic` are pure, and dropped wherever
effects meet; mutable data is refused at `acyclic`. In Rust a list is a
`List` at any region, since KFX writes no list. A region no one names is
`acyclic` (C16, revising C13).

`run-fixpt-programs.py` found a K1 hole: `kfx` accepted `@globals` as a
region in a type, which Fixpt refuses; now refused
(`kfx-refused/globals-region`). The harness had called that program
skipped; a program `kfx` accepts and Fixpt refuses is now a failure.
24 of Fixpt's programs accepted, all agreeing. `tests/kfx/families.kfx`.

## `letrec` (2026-10-11)

`(letrec ((f T lambda) ...) body ...)`: in the checker, `let` with written
types, every procedure in scope in each lambda and in the body. In Rust,
closures cannot name themselves, so each is reached through a `Knot`
(`kayak-rt`), a cell made empty, then tied once the closures are made,
as FX-26's I-cells tie a knot; FX-26's `letrec` binds only lambdas, so no
knot is read untied. The emitter knows a group's names as knots inside
its lambdas (`local-kind`), the innermost binding counting, so a
parameter named as the procedure is the parameter. Also: a call of a
constant whose value is a procedure (a definition made by a `letrec`,
not a lambda) calls its `fn` first, `fib()(…)`, which Fixpt's
`redefine/local-fib.fx` found. `tests/kfx/letrec.kfx`, and
`kfx-refused/letrec-value`. 28 of Fixpt's programs accepted, all agreeing.

## `letregion`, `nat`, and type variables in scope (2026-10-11)

- `cv` has the type and region variables in scope, so `(the a e)`, a
  lambda's written parameter types and a `letrec`'s types may name a
  `poly`'s binders (a gap `TODO.md` §13 listed).
- `(letregion r body ...)`: `r` in scope for the body's types; what the
  body does to `r` masked, and a value whose type mentions `r` refused, as
  in FX-26. A `letregion` naming a region already in scope is refused, so
  that masking, by name, cannot mask an outer region's effects. In Rust,
  a block: the region's data is the heap's.
- `nat`: below `int` (a `nat` fits where an `int` is expected, not the
  other way); a literal checks as one if it is not negative; `Int` in
  Rust. `string-length`, `length` (of a list at `acyclic`, as FX-26's)
  and `list-length` give `nat`s; arithmetic gives `int`s, where FX-26 may
  keep a `nat` (stricter).
- `zero?`, `max`, `min`, `append`, `reverse`.

`tests/kfx/letregion-nat.kfx` (a pure procedure with a private counter),
`kfx-refused/letregion-escape`. 30 of Fixpt's programs, all agreeing.

## `define-generative`; `nil` heard last; type definitions checked where they stand (2026-10-11)

- `(define-generative t T)`, and with type parameters: a new type,
  distinct from its representation, with `up-t` and `down-t`; in Rust a
  `Copy` newtype and two functions, and a `Show` that writes the
  representation, as Fixpt does. Variance marks (`(a type +)`) are
  refused: KFX's families are invariant, and FX-26 checks marks KFX would
  not (`kfx-refused/variance`, found by `run-fixpt-programs`:
  `generative/bad-variance.fx`).
- A call's `nil` whose parameter is an inference variable is heard after
  the other arguments, as FX-26 hears it. Literals were deferred too, at
  first, which made `kfx` accept `tests/kfx/generative.kfx` where Fixpt
  refuses it: the corpus caught it, and only `nil` is deferred.
- A type abbreviation is read where it is defined, used or not, and one
  that mentions itself is refused: a Rust type alias cannot
  (`generative/unguarded-cycle.fx`, which Fixpt refuses too, `kfx` had
  accepted, never reading the unused abbreviation).
- `run-fixpt-programs.py` counts a `define-generative`'s two lines.

32 of Fixpt's programs accepted, all agreeing.

## `f64` (2026-10-11)

Real literals (a point, no exponent: FX-26's reading of an exponent was
not what Rust's is), the type `f64`, and FX-26's double operations: the
arithmetic, comparisons, `f64-min`, `f64-max`, `f64-abs`, `f64-neg`,
`f64-sqrt`, the roundings (`f64-round` ties to even), the predicates, and
`int->f64`, `f64->int` (exact, an error for a value not integral) and
`f64->string`. In Rust, `f64` and its own operations, written as Fixpt
writes a double (`fixpt-runtime`'s `format_flonum`, its writer's rule for
values), read from Fixpt's source rather than guessed: an integral value
below 1e21 with one decimal, any other as Rust writes it, `+inf.0`,
`+nan.0`. `tests/kfx/floats.kfx`, every edge Fixpt's printer has, with
Rust's answers Fixpt's.

## Escapes (2026-10-11)

FX-26's typed control, as far as Rust has it (C18,
`docs/research/escapes.md`): `(prompt-tag A H D R)`, which is `Tag<A, H>`;
`make-continuation-prompt-tag`; `prompt`; `abort-current-continuation`,
whose effect is `(goto R)`. A procedure whose effect may `goto` returns
`Result<T, Unwind>`, its tail positions `Ok(..)` (a self tail call still a
loop); a call the checker found may abort takes `?`; an abort is
`return Err(unwind(tag, v))`; a prompt is a closure returning a `Result`,
its tag's abort handled, any other passed on, or, where the checker found
the prompt contains every abort its body may make, unreachable. A
procedure that may abort is a value too, a `&dyn Fn(..) -> Result<..>`,
and a lambda of such a type a closure returning a `Result`. In toplevel
mode each expression is in such a closure. Kept continuations
(`comefrom`) are refused, with a message (`kfx-stricter/comefrom`).

`tests/kfx/escapes.kfx`: a loop that may leave, a procedure value that
may, and an abort through a prompt of another tag to an outer one.

Found on the way, both K1 holes, now closed:
- `global-atom?` looked at an atom's first letter only, so every `goto:`
  was taken as a read of a global, within any effect that reads them:
  `kfx` accepted a prompt whose body aborts to a tag the prompt's own does
  not allow (`kfx-refused/prompt-effect`).
- A latent effect that names a region not yet solved (`(goto ?4)`, a
  `poly`'s region at a call) was compared as text; now the region is
  solved from the actual's, as regions in types are.

## Modules, the first step (2026-10-11)

FX-26's modules where they map (C19): `(define m (module item ...))` at
the top level, a Rust `pub mod m`, its items a program's definitions;
`(with m body ...)`, the names `m` gives written as paths (`m::x`), since
FX-26's `with` hides locals and Rust's `use` would not; `(select m t)`,
`m::T`. A generative type in a module is abstract: a newtype with a
private field and private conversions. The checker reads a module's items
in their own scope, names its types `m..t` (as FX-26 writes them), and
gives the program its values as `m x`, which `with` opens; naming an
item inside its module reads no global, as FX-26's `letrec*` has it, and
`with m` reads the global `m`. Modules as values, `load-module`,
`include`, `extend`, `hide`, datatypes in modules and items naming later
ones are refused (`tests/kfx-stricter/module-*`).

`tests/kfx/modules.kfx`; `tests/kfx-refused/module-{abstract,opaque,global}`.
45 of Fixpt's programs accepted (32 before), all agreeing.
