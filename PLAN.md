# `kayak`: KFX, a sublanguage of FX-26 that maps onto Rust, and bootstraps

**Status: draft 4, 2026-10-12: §6 is the next goal, the user's.** The loop closed early (§4), and KFX has
grown since, through the bootstrap, each step a fixed point: the
compiler, six files of KFX, checks and translates its own 2700 lines.
What KFX is and how it maps is §2; how it is checked, §3; what is left,
`TODO.md`, ranked by `docs/survey.md`; the decisions made along the way,
`CLAUDES_CHOICES.md`. Draft 1 planned broadly; the user's direction since
was *close the local loop as fast as possible* (draft 2), then to go on
autonomously.

kayak prototypes a bootstrappable system:

1. **KFX** (KFX26 where it could be confused with PLDI '89's KFX) is a
   sublanguage of FX-26. Every construct in it maps in an obvious way to
   *something* in Rust.
2. **A transpiler** turns KFX into Rust.
3. **The transpiler is written in KFX.** The Rust that builds it is
   generated from that source and committed. Running the built compiler
   on its own source gives the same Rust back, byte for byte.
4. That compiler is then the basis for a better one, for the broader
   FX-26, still self-hosted.

Companion files, as in the sibling repositories:
- [`TODO.md`](TODO.md): every task not yet done, numbered;
- [`DONE.md`](DONE.md): what is done, and what was tried and abandoned,
  under the same numbers;
- [`CLAUDES_CHOICES.md`](CLAUDES_CHOICES.md): decisions made without
  asking, each with what else was possible.

**References.** Fixpt and coopt move fast on their own, so kayak works
from clones: `~/Dev/Rust/kayak-refs/fixpt` and `~/Dev/Rust/kayak-refs/coopt`
(at `8d4fb96`, read only, for its design). Citations (`fixpt/docs/fx26.md`)
are to the clones.

**The Fixpt pin is `FIXPT_REV`**, the full hash of the commit stage 0
runs. `scripts/fixpt.sh` checks that the clone's `HEAD` is that commit,
with no changes of its own, builds it (quick when nothing changed), and
gives the binary; `stage0.py` and `fixpoint.sh` use nothing else. It
never checks out by itself. To move the pin: fetch and check out the
clone at the new commit, write it into `FIXPT_REV`, run
`scripts/fixpoint.sh`, and commit, saying what changed in Fixpt.

---

## 1. The loop

```
                 stage 0                         stage 1
kfx/kfx.unit ──fixpt eval──▶ boot/src/kfx/ ──cargo──▶ kfx ──kfx/kfx.unit──▶ the same files
```

- **Stage 0 is Fixpt.** A KFX program is an FX-26 program, so Fixpt can
  *run* the transpiler. `scripts/stage0.py` makes one FX-26 file of the
  prelude (`kfx/prelude.fx`, below), the compiler, and `(compile "…the
  source, escaped…")`, and runs `fixpt eval --raw` on it (with
  `--step-limit none`), which prints the string it gives exactly: that is
  `boot/src/kfx/`. No transpiler is written by hand in Rust: the
  transpiler is written once, in KFX.
- **Stage 1** builds `boot/`, a Rust crate whose `main` (hand-written,
  ten lines) reads a file, calls the generated `compile`, and writes the
  result.
- **The fixed point** (`scripts/fixpoint.sh`): the built `kfx` on
  `kfx/kfx.unit` gives `boot/src/kfx/`, byte for byte. Fixpt and the Rust
  build run the same program on the same input, so they agree unless the
  mapping is wrong. A disagreement is a bug in the mapping, found on the
  compiler itself.
- **Once it closes, Fixpt is no longer needed to build.** `boot/` is the
  seed: change `kfx/`, run the current `kfx` on it, rebuild, run
  again, and commit once the output stops changing. Stage 0 stays as a
  check that KFX is still FX-26 (§3, K1).

Verified by probe (2026-10-11), against the clone's Fixpt:
- `fixpt eval FILE` prints each top-level form's value with its type, and
  the last line is the result: a string printed with `\\`, `\"`, `\n` and
  `\t` escapes;
- a 60 KB string through `string->list`, a 60K-deep non-tail recursion
  and `list->string` is instant;
- top-level names are seen only by forms after them; mutual recursion is
  a `define-rec`, which must say `spin`, so KFX code uses one effect
  everywhere, `(define-effect kfx (maxeff (read @globals) spin))`;
- `i64+`, `i64-` and `i64*` wrap; `i64-quotient` and `i64-remainder`
  truncate, `MIN / -1` is `MIN` with remainder 0; division by zero is an
  error; `i64-shl` counts modulo 64 and `i64-shr` is arithmetic. That is
  Rust's `wrapping_*`, operation for operation.

---

## 2. KFX: the language, and its mapping

KFX is what the transpiler is written in and what it translates: the
part of FX-26 each of whose constructs maps exactly to something in Rust.
It began as KFX-0, what the loop needed (§4), and grows a construct at a
time, through the bootstrap.

**The memory model.** Every KFX value is a `Copy` Rust value. Scalars are
themselves. Everything else is a `&'static` reference to memory that is
never freed: a compiler run is short, and rustc works the same way with
its arenas. Allocation is `Box::leak` for now, and an arena later. There
is no `Drop`, no `Rc` and no `.clone()`.

| KFX-0 (FX-26)                                         | Rust                                                        |
| ----------------------------------------------------- | ----------------------------------------------------------- |
| `i64`; `i64+`, `i64-`, `i64*`, `i64<`, `i64=`, …       | `i64`; `wrapping_add`, `wrapping_sub`, …, `<`, `==`, …      |
| `int`; `+`, `-`, `*`, `quotient`, `=`, `<`, …          | `Int`, `kayak-rt`'s bignum; `_plus`, …: exactly FX-26's     |
| a literal                                             | `5i64` where an `i64` is expected, else `int_lit(5)` (a fact) |
| a literal past an `i64`                               | `int_text("…")`, an `int` (a `nat` up to 2^60 - 1, as FX-26) |
| `bool`, `char`, `unit`                                | `bool`, `char`, `()`                                        |
| `string`, a literal `"…"`                             | `Str`, a `&'static str` that knows if it is ASCII; `str_lit("…")` |
| `(listof T R)`, at any region                         | `List<T>` = `Option<&'static Cons<T>>`: KFX writes no list  |
| a `define-datatype` name `t`                          | `&'static T`, `enum T` holding its variants                 |
| `(define f (subr kfx (T…) T) (lambda (x…) body))`     | `fn f(x: T, …) -> T { body }`                               |
| `(define-rec (f type lambda) …)`                      | the same, one `fn` each                                     |
| a self call in tail position                          | the `fn`'s body in a `loop`: assignments, then `continue`   |
| `(define x T e)`, not a lambda                        | `fn x() -> T { e }`; each use `x()` (`e` only reads globals, C11) |
| … whose `e` allocates                                 | made at its first use and kept: `thread_local!`, `once` (C21) |
| `(define x e)`, with no type                          | as with the type the checker finds (elaboration)            |
| `(define* f T lambda)`                                | as `define`; the globals read are found, for the checker    |
| `(define-datatype t (tag T…) …)`                      | `enum T { Tag(T…), … }` and `fn tag(…) -> &'static T`       |
| `(define-datatype (t (a type) (r region)) …)`         | `enum T<A: Copy + 'static>`, generic constructors; `r` erased |
| `(define-type (t (a type)) T)`; `(t i64)`             | `type T<A> = …;` `T<i64>`; never one that mentions itself   |
| `(define-generative t T)`; `up-t`, `down-t`           | `#[derive(Clone, Copy)] struct T(pub T')`; `up_t`, `down_t` |
| `(define-effect …)`                                   | nothing                                                     |
| `(f a …)`, `f` a global or a primitive                | `f(a, …)`; the primitives are `kayak_rt`'s                  |
| `(e a …)`, `e` an expression                          | `(e)(a, …)`                                                 |
| a global procedure named as a value                   | `&f`: a reference to its `fn`, promoted to `'static`        |
| `(subr E (T…) R)` as a value's type                   | `&'static dyn Fn(T…) -> R`                                  |
| `(ref T R)`; `new`, `get`, `set`                      | `Ref<T>` = `&'static Cell<T>`; `Cell::new`, `get`, `set`    |
| `(flatarrayof i64 R)`; the prelude's `…-i64` forms    | `FlatArray<i64>` = `&'static [Cell<i64>]`, bounds checked   |
| a region, `@a` or a `poly`'s `(r region)`             | erased: every region's place is the heap, so `'static`      |
| `(read R)`, `(write R)`, `(alloc R)`                  | erased, as every effect is; checked by `kfx` and Fixpt      |
| `(lambda (x …) body)` as a value                      | `lambdaN(move \|x, …\| { body })`, `N` its arity          |
| `(poly ((a type) …) T)`, a definition's type          | `fn f<A: Copy + 'static, …>`; an effect binder is erased    |
| a call of a `poly` procedure                          | inferred, by FX-26 and by rustc alike: no `proj`            |
| `if`, `cond`, `and`, `or`                             | `if`/`else`, `&&`, `\|\|`                                   |
| `let`, `let*`, `begin`                                | blocks of `let`s and statements                             |
| `(letrec ((f T lambda) …) body)`                      | closures tied through `Knot`s, as FX-26's I-cells tie them  |
| `(letregion r body)`                                  | a block: `r`'s data is the heap's; its effects masked       |
| `nat`                                                 | `Int`; below `int` in the checker                           |
| `f64`; `f64+`, …, `f64-round`, `f64->string`, …       | `f64`; Rust's operations, written as Fixpt writes doubles   |
| a real literal, `1.5`                                 | `1.5f64` (no exponent: KFX refuses `1e5`)                   |
| `(case k ((d …) e) … (else e))` on chars, bools       | `match k { d \| … => e, …, _ => e }`                        |
| … on an `int` or a string                             | `match k { case_ if _eq(case_, d) \|\| … => e, … }`         |
| `(tagcase e (tag (x …) body) … [(else y body)])`      | `match *e { T::Tag(x, …) => body, … }`                      |
| `(productof (l T) …)`; `(product (l e) …)`; `extract` | `Product_1l…<T…>`, a tuple struct per list of labels; `.i` (C22) |
| `(sumof (tag T) …)`; `(sum tag e)`                    | `&'static Sum_3tag…<T…>`, an enum per list of tags (C22)    |
| `i32`, `u32`, `u64`; `u32+`, `u32-shl`, `int->u32`, …  | Rust's; `wrapping_add`, …; literals `5u32`                  |
| `(prompt-tag A H D R)`; `make-continuation-prompt-tag` | `Tag<A, H>` (C18)                                           |
| a procedure whose effect may `goto`                   | `fn … -> Result<R, Unwind>`; its calls `?` (C18)            |
| `(abort-current-continuation t v)`                    | `return Err(unwind(t, v))`                                  |
| `(prompt t body handler)`                             | a closure returning a `Result`, matched (C18)               |
| `(define m (module item …))`, top level               | `pub mod m { use super::*; … }` (C19)                       |
| `(with m e)`; `(select m t)`                          | paths: `m::x`, `m::T`; a generative type's field private    |
| `(include n)`, `(hide item …)` in a module            | `pub use super::n::{x, T};`, plain `use` when hidden        |
| `(define-type s (moduleof (abs t type) (val x T) …))` | `pub trait S { type T; fn x() -> …; }` (C24)                |
| a top-level procedure's parameter of type `s`         | a type parameter `<M: S>`; a module given, `m::Module` (C24) |
| `nil`                                                 | `nil()`                                                     |
| `(list a …)`                                          | nested `cons`                                               |
| `(error "…")`                                         | `error("…")`, which returns `!`                             |
| literals                                              | Rust literals: `1i64`, `'a'`, `str_lit("a\n")`, `true`, `()` |
| a string at the top level                             | a `//` comment                                              |
| a string first in a procedure's body, before more     | its docstring: `///` on the `fn`                            |
| any other string in a body, not last                  | a `//` comment                                              |

**Comments are strings** (2026-10-11). `;` comments do not survive
reading, so KFX writes its comments as string literals where a body's
value is not taken, as Lisp docstrings are. FX-26 accepts a string
there: it is pure, and its value is dropped. Where FX-26 has no
expression (between a `define-rec`'s bindings, a datatype's variants, or
arms), a comment goes into the nearest body or the top level.

The emitter indents as it writes, in rustfmt's style: blocks on lines of
their own, and a body's last `let`, `let*`, `begin` or `tagcase`
written into the enclosing block. `rustfmt` is not run, so the fixed
point stays the compiler's alone.

**`int` is a bignum, in Rust too** (C15). The user's ruling (2026-10-11)
was that `+` on `int` may not mean a bignum in FX-26 and an `i64` in
Rust: so `int` was refused, and KFX wrote `i64+`. Now `int` is
`kayak-rt`'s `Int`, a bignum, and `+` on it means in Rust what it means
in FX-26, so the ruling holds with `int` back. A literal is an `i64`
where one is expected (since Fixpt `fb86975`), and an `int` elsewhere,
as in FX-26: the checker says which, by a fact about its place.

**The prelude** (`kfx/prelude.fx`) is FX-26, not KFX: where FX-26's
standard environment has a primitive only at `int` (`string-length`,
`string-ref`, `char->integer`, `int->string`), it defines one at `i64`
(`string-length-i64`, `string-ref-i64`, `char->i64`, `i64->string`) by
converting. Fixpt reads it before a KFX program; the transpiler never
sees it, since each of its names is a `kayak-rt` function by the same
mangling, and the prelude is that function's meaning in FX-26.

Names are mangled reversibly, so a primitive and the Rust function that
implements it share a name: `-` becomes `_`, `?` becomes `_p`, `!` becomes
`_x`, `->` becomes `_to_`, and `+ * < > = /` become `_plus _star _lt _gt
_eq _slash`; a `-` alone or last is `_minus`. So `string->list` is
`string_to_list`, `char=?` is `char_eq_p`, `i64-` is `i64_minus`, and
`i64<=` is `i64_lt_eq`.

**Programs of several files** (2026-10-11). A program is files in order,
listed by a manifest (`NAME.unit`): to FX-26, the files one after
another, each seeing the definitions of those before it; to Rust, a
module each, which imports every other's names and `kayak-rt`'s (`use
super::*`), with a generated `mod.rs` declaring and re-exporting them.
Since the modules import by glob, KFX defines each name once, and never a
primitive's or `kayak-rt`'s. FX-26's own modules are KFX's where they are
defined at the top level (C19) or given for a module parameter (C24);
modules as values otherwise, and loaded files, are not yet (`TODO.md` §19).

**What the checker proves, as FX-26's does** (C20, C21). Recursion that
ends needs no `spin`, by FX-26's size-change check, a subset of it
measure for measure; a procedure kept in storage at a region it reads
must say `spin` (FX-26's `no_knot`); a global whose latent effect reads
itself too. None of it changes the Rust: it is what K1 needs.

**Mutable data is cells** (2026-10-11; `CLAUDES_CHOICES.md` C9, C10). A
reference and a flat array are shared and aliasable, in FX-26 as in a
Rust `Cell`: any copy may write, every copy sees it. Not `&mut`, which
needs the one live reference that a `Copy` value cannot promise. Regions
are checked (effects on them, by `kfx` and Fixpt) and erased in Rust.

**Checked first.** `kfx` checks types and effects before it writes Rust
(`DONE.md` §13). Two of its rules are Rust's, not FX-26's: a call's
arguments are checked in order, so a lambda's parameter whose type no
earlier argument gives is written `(x T)`; and a keyword may name a local
but not be called.

A name as a value is a local, a constant (`x()`), or else a global
procedure (`&f`), the program's or `kayak-rt`'s: the emitter keeps the
names bound where it is, from parameters, `let`, `let*` and `tagcase`.

What KFX does not have, ranked by how often Fixpt's programs first need
it (`docs/survey.md`, rerun 2026-10-11: 67 of 286 accepted, all agreeing
when run): `datum` and unions; bloblets and flat data beyond `i64` arrays
(§15); modules as values, `include`, `extend`, `hide`, `load-module`
(§19); places, `letrena` (§14, C23); `poly` over effects or sizes, higher
kinds; fixed-width integers but `i64`, and `f32`; kept continuations and
marks (§16); `proj` (instantiation is inferred).

---

## 3. Invariants

- **K1. KFX ⊂ FX-26.** Every KFX program is an FX-26 program with the
  same answer. KFX may refuse FX-26 programs, never reinterpret them,
  and takes no licence: every construct means in Rust what it means in
  FX-26 (`int` included, a bignum in both, C15). Checked four ways:
  stage 0 (Fixpt runs the compiler); the corpus (each program's answer,
  by Fixpt and by its Rust); `kfx`'s own checker, which refuses what Fixpt
  refuses (`tests/kfx-refused`) and, on purpose, a little more
  (`tests/kfx-stricter`); and Fixpt's own test programs that `kfx`
  accepts, run as Rust and by Fixpt (`scripts/run-fixpt-programs.py`),
  any that `kfx` accepts and Fixpt refuses being a failure.
- **K2. The bootstrap is a fixed point:** `scripts/fixpoint.sh`.
- **K3. Output is deterministic:** no hash-map order, no addresses, no
  clocks. Without this, K2 cannot hold.
- **K4. Generated code is safe Rust**, with no `unsafe`.
- **K5. The generated Rust is readable.** It keeps KFX's names and
  structure, and is the seed of the bootstrap, so it is reviewed as
  source.

---

## 4. Steps to the closed loop (done, 2026-10-11)

1. `kayak-rt`: `List`, `Cons`, `Str`, and the primitives KFX-0's source
   uses, each named by the mangling.
2. `kfx/` (then one file): a reader, a first pass noting datatypes and constants,
   and an emitter.
3. `scripts/stage0.py`, then `boot/` building.
4. `scripts/fixpoint.sh` passing.

All done: `DONE.md` §2, and what followed, §3 to §6.

---

## 5. After the loop

The tasks are `TODO.md`'s. Done since the loop closed (`DONE.md`): a
corpus and guardrails, tail calls as loops, procedures as values, `poly`,
a checker, messages with places, several files, mutable data and
regions, strings of characters, untyped definitions, `define*`, `int`,
type families, `letrec`, `letregion`, `nat`, `define-generative`, `f64`,
escapes (the first place effects decide code generation: C18), modules
defined at the top level (C19) and given as parameters (C24),
termination (C20), constants made once (C21), products and sums (C22),
`case` beyond characters, literals past an `i64`. Each was then probed
by programs written to find where `kfx` and Fixpt part: four K1 holes
found that way, and fixed (`DONE.md`). Next, by the survey: `datum` and
unions, flat data, more of modules.

Then FX-26 at large, written in KFX. Two directions were open:
- **keep targeting Rust**, with a runtime that grows (a collector, as
  coopt's; continuations, through an explicit stack), each FX-26 feature
  leaving KFX's trivial mapping as it arrives, so that KFX stays the
  subset the compiler is *written* in while the language it *compiles*
  grows;
- **target something else** for FX-26 at large (Fixpt's cellular words,
  or native code), and keep Rust for KFX alone.

The first is the one taken, for now: §6 grows the runtime, in Rust.

---

## 6. Next: an asynchronous runtime, in Coopt's mindset (2026-10-12)

**The user's direction (2026-10-12).** Coopt's async-first approach has
proved interesting. kayak need not copy all of its choices, but its
runtime should move to Coopt's mindset, above all its deep support for
*multiplexed tasks* and *incremental computation*. The first goal: work
out how to revise `kayak-rt` around asynchronous computation, in part
through async Rust, most likely with an executor of our own; doing that
properly will change the emitter too, so that it writes `async fn`,
`Pin` and so on.

### 6.1 What we take from Coopt, and where we differ

Taken (Coopt's `PLAN.md` §1, §3.5):
- **One scheduler of our own** (Coopt's I4): no tokio, no mio; and so
  `kayak-rt` keeps having no dependencies.
- **Determinism** (I1): a seeded scheduler, a virtual clock first, steps
  as the clock. Here it is also what K1 needs: an async program's answer
  must be Fixpt's, so the order tasks run in must be the reference's.
- **The scheduler sees effects** (I3): a task's effect is known when it
  is spawned.
- **Structured concurrency**: nurseries, cancellation delivered at
  suspension points (Fixpt's `docs/research/async.md` §5, which Coopt
  implements).

Where kayak differs, and why. Coopt interprets: its engine's stack is in
its heap, and engine state is kept out of Rust futures (I4, I5), so that
its continuations are composable and copyable and its collector can move
everything. kayak compiles, so a Rust `async fn` *is* the
continuation-passing transform of a frame, made by rustc: suspension
compiled, not interpreted. The price is that a Rust future resumes once.
So kayak gets **one-shot suspension**, and a continuation resumed twice
(`call-with-composable-continuation` at large) stays refused. That is
enough: in Fixpt's async design every suspension resumes once, with
unit, through a one-shot waker (§6.1 there), which is Rust's `Waker`.

Not taken, yet: a moving collector, the query compiler, record and
replay, real I/O (a reactor needs `libc` or `unsafe`, which `kayak-rt`
has neither of).

### 6.2 The mapping, sketched

- **Suspension is an effect.** In FX-26 a task suspends by capturing up
  to its loop's prompt: `(comefrom c)` on the loop's region `c`, the note's
  `(suspends c)` (§3.1 there). As escapes are `Result` by effect (C18), a
  procedure whose effect may suspend becomes an `async fn`; the rest stay
  plain `fn`s, at no cost. The colour of a function is its effect, known
  statically: nothing is compiled twice.
- **The operations** (`with-loop`, `spawn`, `join`, `suspend`, `wake!`,
  `yield`, `sleep`, nurseries, cancel scopes, channels: Fixpt's note,
  §6.1) are `kayak-rt` primitives on our executor; and for Fixpt, the
  same operations in FX-26 over continuations in `kfx/prelude.fx` (ported
  from Coopt's library), so K1 still compares answers with Fixpt.
- **The executor**: one thread, a FIFO run queue, a virtual clock; futures
  need not be `Send`, so the heap's thread-local arenas (C43) stay.
- **Cancellation** is dropping a task's future, which happens only at a
  suspension point.
- **Escapes** inside an `async fn` compose as they are (`.await?`).

### 6.3 Problems to solve first (a research note)

1. **Recursion.** An `async fn` that calls itself needs a `Box::pin`
   per call. Only suspending procedures pay, and tail calls are loops;
   later, frames could go in arenas.
2. **Procedures as values.** A suspending closure returns a
   `Pin<Box<dyn Future>>`, or is an `AsyncFn`; which one, measured.
3. **Regions across suspension.** Fixpt's note (finding 4) refuses a
   suspension inside a task's own `letrena`; a `letrena` around the loop
   is shared by its tasks. In Rust, a future borrowing `'r` across an
   `.await`: self-reference, which is what `Pin` is for.
4. **Preemption.** Cooperative, at suspension points, first. Coopt's
   fuel would make every `spin` procedure suspend, the compiler's nearly
   all: measured before it is chosen.
5. **Memory.** A long-running task needs its data freed, and the heap is
   never freed (C43); nurseries as regions are the likely route.
6. **Incremental computation**: what it is for kayak is §7, Q5.

### 6.4 Steps

1. The research note, `docs/research/async-rust.md`: Coopt's scheduler and
   its library in FX-26, Fixpt's note, and §6.2 checked by writing, by
   hand, the Rust for two or three of Coopt's async programs
   (`cancel.fx`, a ping-pong) over a prototype executor.
2. The executor in `kayak-rt`: deterministic, a virtual clock, our own
   `Waker`, no dependencies.
3. The operations in `kfx/prelude.fx` for Fixpt.
4. The checker: the suspension effect, and its rules (§6.3).
5. The emitter: `async fn` by effect.
6. Corpus programs, their answers compared with Fixpt's.

---

## 7. Questions for you

- **Q1. The name.** PLDI '89's kernel is also KFX, and `fixpt/docs/fx26.md`
  calls it that. I write KFX, and KFX26 where the two could be confused.
  Files are `.kfx`.
- **Q2. `int`.** Answered (the user, 2026-10-11): no `int` whose meaning
  differs between FX-26 and Rust. Since C15, `int` is a bignum in both.
- **Q3. Escapes**: `Result` by effect, or unwinding panics? Decided:
  `Result` by effect, the recommendation (C18).
- **Q4. Places**: needed now? (`docs/research/places.md` §4.) Not yet,
  I decided (C23): nothing needs memory reclaimed; yours to overrule.
- **Q5. Incremental computation** (§6). Answered (the user,
  2026-10-12): (a), the compiler itself incremental, as Coopt's query
  compiler (`coopt-query`: revisions, memos, dependencies, early cutoff),
  so that a change recompiles only what it reaches. And (b), a facility
  KFX programs can use, insofar as it gets us (a): the compiler is
  written in KFX, so its queries are KFX procedures, and the machinery
  under them (the memo table, the dependency graph, revisions) is
  `kayak-rt`'s, given to KFX as primitives, as the executor is.
- **Q6. The reference for async programs.** Fixpt has no scheduler, so
  K1 needs the operations written in FX-26 in `kfx/prelude.fx` (from
  Coopt's library); or Coopt is made the reference for them. My default:
  the prelude, so Fixpt stays the one reference.

## 8. Requests for Fixpt

Three, all done and pinned: `DONE.md` §6.
