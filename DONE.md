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
