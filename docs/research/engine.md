# FX-26's engine, in KFX: the first code generator, an interpreter

Status: design, 2026-10-12. `PLAN.md` §6.5, steps 10–12; Q8 (the user: an
interpreter, written in KFX, the start and kept after as the reference);
C57 (FX-26's features with no Rust form, unions first, come with it).

## What it is

A program in KFX, `fx26/`, that reads an FX-26 program, checks it with
the compiler's own checker (`kfx/check*.kfx`), lowers it to an IR, and
runs the IR on a machine whose values are uniform and whose continuation
is data. Compiled by kfx to Rust, it is a binary, `kayak-fx26`; run by
Fixpt as FX-26, it is the same program (K1 holds for the engine as for
the compiler). Its answers are compared with Fixpt's (K6).

## Decisions

1. **The machine is a CEK machine**: a control (an IR node or a value), an
   environment, and a continuation that is a list of frames, data in the
   engine's heap. So:
   - FX-26's continuations come for free and as FX-26 has them:
     `call-with-composable-continuation` captures the frames up to a
     prompt, and resuming copies nothing (the frames are immutable and
     shared), any number of times. What KFX refuses for Rust (C45: a
     continuation resumed twice) the engine runs.
   - Tail calls take no space: a call in tail position pushes no frame.
   - The machine's loop is one KFX procedure calling itself in tail
     position, which kfx compiles to a Rust `loop`.
2. **Values are uniform**: one KFX datatype, `val`, with a variant per
   shape (an integer, `i64` and the fixed widths' values held as `int`
   and wrapped by their operations, `f64`, a character, a string, a
   symbol, `#t`/`#f`, unit, `nil`, a pair, a reference, a sum, a
   product, a datatype's value, an array, a bytevector, a procedure, a
   primitive, a prompt tag, a continuation). So unions (C57) cost
   nothing: a member is its union's value already.
3. **The IR resolves names**: a local is a frame's depth and slot, a
   global an index into the globals' array; a primitive call names its
   primitive. Forms FX-26 derives (`let*`, `cond`, `case`, `and`, …) are
   lowered to the core. The IR is a KFX datatype, so the lowering is a
   function from the checked program to it.
4. **The heap is KFX's, for now**: a pair, a reference, an environment's
   frame are KFX data at a region of the engine's, never freed (C43's
   heap). A collector of its own, a heap as an array of words, values as
   indices (Fixpt's and Coopt's arrangement), is step 11's second half:
   needed for long runs, not for answers.
5. **Types are not needed to run**, but where FX-26's meaning depends on
   them: a literal's type (an `i64` or a `u8` literal is an integer
   either way; the operation wraps it), `the` (dropped), a `nil`'s list
   (any). The checker's facts are there if a form needs one.
6. **Primitives are KFX procedures** of the engine, a table by name: the
   standard environment's (`kfx/check-defs.kfx`'s table names them), and
   the prelude's (`kfx/prelude.fx`'s names at `i64`), so a KFX program's
   `main` runs as it is.
7. **Printing is Fixpt's**: a value as `fixpt eval` writes it, so that
   `scripts/run-fixpt-programs.py`'s comparison serves.

## Steps

1. The IR, the lowering and the machine for the core: literals, `lambda`,
   calls, `define`, `if`, `begin`, `let` forms, `set`/`get`/`new`, lists,
   strings, characters, integers; the corpus's simplest programs run.
2. Sums, products, datatypes, `tagcase`; prompts, aborts, captures.
3. The corpus (`tests/kfx/`), each program's `main` run by the engine and
   its answer compared with Fixpt's (`scripts/engine-programs.py`); then
   Fixpt's own programs.
4. Features KFX refuses for Rust, the engine first: continuations
   resumed twice, then unions (the checker grown, the restriction pass
   refusing them for Rust).
5. The collector; the engine's tasks on the scheduler (`lib/tasks.kfx`).
6. The second self-hosting: the compiler run on the engine.
