# `kayak`: KFX, a sublanguage of FX-26 that maps onto Rust, and bootstraps

**Status: draft 2, 2026-10-11. The loop is closed (§4): `scripts/fixpoint.sh`
passes; what is left is `TODO.md`.** Draft 1 planned broadly. The user's
direction since: *close the local loop as fast as possible*, meaning the
shortest path to KFX source that transpiles to the very Rust that builds
it. Everything not on that path is in §5, for after.

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
kfx/kfx.kfx ──fixpt eval──▶ boot/src/kfx.rs ──cargo──▶ kfx ──kfx/kfx.kfx──▶ same kfx.rs
```

- **Stage 0 is Fixpt.** A KFX program is an FX-26 program, so Fixpt can
  *run* the transpiler. `scripts/stage0.py` makes one FX-26 file of the
  prelude (`kfx/prelude.fx`, below), the compiler, and `(compile "…the
  source, escaped…")`, and runs `fixpt eval --raw` on it (with
  `--step-limit none`), which prints the string it gives exactly: that is
  `boot/src/kfx.rs`. No transpiler is written by hand in Rust: the
  transpiler is written once, in KFX.
- **Stage 1** builds `boot/`, a Rust crate whose `main` (hand-written,
  ten lines) reads a file, calls the generated `compile`, and writes the
  result.
- **The fixed point** (`scripts/fixpoint.sh`): the built `kfx` on
  `kfx/kfx.kfx` gives `boot/src/kfx.rs`, byte for byte. Fixpt and the Rust
  build run the same program on the same input, so they agree unless the
  mapping is wrong. A disagreement is a bug in the mapping, found on the
  compiler itself.
- **Once it closes, Fixpt is no longer needed to build.** `boot/` is the
  seed: change `kfx.kfx`, run the current `kfx` on it, rebuild, run
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

## 2. KFX-0: the language the loop needs

KFX-0 is exactly what the transpiler is written in, and so what it must
translate. Anything more waits until the loop is closed.

**The memory model.** Every KFX value is a `Copy` Rust value. Scalars are
themselves. Everything else is a `&'static` reference to memory that is
never freed: a compiler run is short, and rustc works the same way with
its arenas. Allocation is `Box::leak` for now, and an arena later. There
is no `Drop`, no `Rc` and no `.clone()`.

| KFX-0 (FX-26)                                         | Rust                                                        |
| ----------------------------------------------------- | ----------------------------------------------------------- |
| `i64`; `i64+`, `i64-`, `i64*`, `i64<`, `i64=`, …       | `i64`; `wrapping_add`, `wrapping_sub`, …, `<`, `==`, …      |
| `int`                                                 | **refused**: its `+` makes a bignum, which Rust's does not  |
| `bool`, `char`, `unit`                                | `bool`, `char`, `()`                                        |
| `string`                                              | `Str` = `&'static str`; ASCII only, so `string-ref` is O(1) |
| `(listof T acyclic)`                                  | `List<T>` = `Option<&'static Cons<T>>`                      |
| a `define-datatype` name `t`                          | `&'static T`, `enum T` holding its variants                 |
| `(define f (subr kfx (T…) T) (lambda (x…) body))`     | `fn f(x: T, …) -> T { body }`                               |
| `(define-rec (f type lambda) …)`                      | the same, one `fn` each                                     |
| `(define x T e)`, not a lambda                        | `fn x() -> T { e }`; each use `x()` (KFX-0: `e` pure)       |
| `(define-datatype t (tag T…) …)`                      | `enum T { Tag(T…), … }` and `fn tag(…) -> &'static T`       |
| `(define-effect …)`                                   | nothing                                                     |
| `(f a …)`, `f` a global or a primitive                | `f(a, …)`; the primitives are `kayak_rt`'s                  |
| `if`, `cond`, `and`, `or`                             | `if`/`else`, `&&`, `\|\|`                                   |
| `let`, `let*`, `begin`                                | blocks of `let`s and statements                             |
| `(case k ((d …) e) … (else e))` on chars             | `match k { d \| … => e, …, _ => e }`                        |
| `(tagcase e (tag (x …) body) … [(else y body)])`      | `match *e { T::Tag(x, …) => body, … }`                      |
| `nil`                                                 | `nil()`                                                     |
| `(list a …)`                                          | nested `cons`                                               |
| `(error "…")`                                         | `error("…")`, which returns `!`                             |
| literals                                              | Rust literals: `1i64`, `'a'`, `"a\n"`, `true`, `()`         |
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

**No `int`** (the user's, 2026-10-11). `+`, `-` and `<` on `int` mean
bignum arithmetic, which is not what Rust's mean; KFX is the kernel whose
meaning is known because its mapping is, so it writes `i64+` and the rest,
whose meaning in FX-26 is Rust's exactly. Literals are plain: since Fixpt
`fb86975` a literal checks as an `i64` where one is expected.

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

KFX-0 has no first-class procedures (a call's operator is always a
global's name), no `poly` of its own (the primitives' polymorphism is
Rust generics in `kayak_rt`), and no regions but `acyclic`. These are the
first things to add after the loop closes (§5).

---

## 3. Invariants

- **K1. KFX ⊂ FX-26.** Every KFX program is an FX-26 program with the
  same answer. KFX may refuse FX-26 programs, never reinterpret them,
  and takes no licence: with `int` gone, every construct means in Rust
  what it means in FX-26. The check is stage 0 itself, run before each
  commit.
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
2. `kfx/kfx.kfx`: a reader, a first pass noting datatypes and constants,
   and an emitter.
3. `scripts/stage0.py`, then `boot/` building.
4. `scripts/fixpoint.sh` passing.

All done: `DONE.md` §2, and what followed, §3 to §6.

---

## 5. After the loop

The tasks are `TODO.md` §7 on. Their direction, in rough order: errors
that point at KFX source, a corpus of KFX programs, guardrails; self tail
calls as loops, first-class procedures, `poly` as generics; a KFX checker
in KFX; places as lifetimes, flat data; escapes as `Result`, the first
place effects decide code generation.

Then FX-26 at large, written in KFX. Two directions, not chosen yet:
- **keep targeting Rust**, with a runtime that grows (a collector, as
  coopt's; continuations, through an explicit stack), each FX-26 feature
  leaving KFX's trivial mapping as it arrives, so that KFX stays the
  subset the compiler is *written* in while the language it *compiles*
  grows;
- **target something else** for FX-26 at large (Fixpt's cellular words,
  or native code), and keep Rust for KFX alone.

---

## 6. Questions for you

- **Q1. The name.** PLDI '89's kernel is also KFX, and `fixpt/docs/fx26.md`
  calls it that. I write KFX, and KFX26 where the two could be confused.
  Files are `.kfx`.
- **Q2. `int`.** Answered (the user, 2026-10-11): KFX writes `i64`, and
  has no `int` (§2).

## 7. Requests for Fixpt

Three, all done and pinned: `DONE.md` §6.
