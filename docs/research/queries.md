# Queries: the compiler made incremental

Status: research and a first step, 2026-10-12. `PLAN.md` §6.5, step 9;
Q5 (the user: the compiler itself incremental, by machinery KFX can use).
Decisions: `CLAUDES_CHOICES.md` C54.

Read: Coopt's `docs/research/queries.md`, `crates/coopt-query/src/lib.rs`,
its front end's and checker's queries (`coopt-front`, `coopt-fx26`), its
`DONE.md` on them, at `8d4fb96`.

## Findings first

1. **Coopt's queries live in one long-running process**, its REPL: one
   global revision, durabilities, memos with their dependencies in read
   order, early cutoff by fingerprint (SipHash of a canonical encoding),
   cycles poisoned, every query run a scheduler task whose body is a
   future. Nothing is kept on disk; a persisted database is future work
   there, which its fingerprints were chosen to allow.
2. **kfx is run once per compile.** A memo that dies with the process saves
   nothing in a batch compiler. So for kayak, the first thing to build is
   what Coopt has not: memos kept between runs. A memo kept on disk cannot
   be checked against revisions it never saw; it is checked against what
   it was made from, by fingerprint. Content addressing does exactly that:
   the memo's key *is* the fingerprint of all it was made from.
3. **The granularity is the top-level form**, as Coopt's (`FormId`).
   Coopt's lesson, absolute positions in values invalidate everything
   after an edit: kfx's facts are keyed by offset in the file. So a
   form's facts are fingerprinted at offsets from the form's start, and
   the Rust kfx writes for a form was checked not to depend on where the
   form is (an insertion before it leaves it the same).
4. **Where the time goes** (the compiler on itself, 0.6 s): reading
   15 ms; checking 170 ms; emitting most of the rest. Emitting is per form
   already, given the checker's facts and the context: the first query.
5. **Coopt's verifier** (incremental equals from scratch, under random
   edits) caught every bug planted in its machinery but one. kayak takes
   it first: `crates/kayak-checks/tests/incremental.rs` (was `scripts/incremental.py`).

## The first step (C54)

`Emit(form)`, memoized across runs: `kfx --cache FILE PROGRAM.unit DIR`.
Its key is the FNV-1a fingerprint of the form's text, its facts (relative
offsets), and the context's fingerprint (datatypes, tags, constants,
families, types found, modules: what the emitter reads besides the
facts). The memos given are taken where they fit; those this compile made
or took are written back, so the file does not grow. The memo file is
read and written as runner operations: a compile with memos is recorded
and replayed as any other.

Measured, the compiler on itself: 0.82 s cold, 0.32 s warm (0.61 s with
no memos at all). What is left is checking, all of it each time.

## The second step (C55)

`Check(form)`, as a change to the checker's environment, keyed by a chain
of fingerprints (§Next, 1, done coarsely: everything before the form, not
the names it reads). Warm, 0.12 s; a body edited, 0.12 s; a definition
added early, 0.28 s.

## Next

1. **`Check(form)`**: the checker's environment threaded form by form is
   the dependency: a definition's check reads the interfaces of the names
   it refers to (their types, declared or found) and the program's
   datatypes and effects. Its result is facts (text already) and, for an
   untyped definition, its type. Keyed as `Emit` is, with the
   fingerprints of the interfaces read in place of the whole context: so
   a body's edit re-checks that body alone, and an interface's edit what
   reads it.
2. **Early cutoff**, which content addressing gives for free one level
   down: a form whose text changes but whose interface does not leaves
   its dependents' keys the same.
3. **Queries in the library**, for KFX programs (Q5's (b)): a memo table
   generic in its value, fingerprints, and a store kept by a runner;
   `lib/query.kfx`. Then the compiler's queries as tasks (Coopt's I4),
   when the compiler has a reason to wait: a REPL, or work in parallel.
