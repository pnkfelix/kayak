# `stage` and sessions on the compiled tiers

Status: design (2026-11-03, C316), for the user's decisions at the end.
`stage`, evidence and sessions work on the engine and in both REPLs
(C304-C315). None of the compiled tiers takes them yet:
`kayak-fx26 --cells` stops at a staging program ("cells: not yet compiled
as a value: @heap").

## How a program reaches the compiled tiers today

`kayak-fx26 --cells FILE` checks the program, compiles it to cellular
words (`fx26/cg.kfx`, `cg-top.kfx`, kayak's port of Fixpt's `compile.fx`),
and writes them as a text listing, in the format `fixpt compile` writes.
`kayak-cells LISTING` reads that listing into a heap of its own
(`cells/listing.kfx`, `listing-words!`) and runs its `program` word:

- tier 1, the cells interpreted;
- tier 2 (`--hand`), arm64 routines for the cells;
- tier 4 (`--compiled`), each word compiled to machine code;
- tiers 5 and 6 (`--regs`, `--direct`), register code.

Tiers 4 to 6 compile every word once, at load (`words-compiled!` over all
the words). The tier 2 machine hands back to tier 1 any cell it has no
routine for (`hand.kfx`).

So at run time `kayak-cells` holds no checker and no compiler: only the
words it was given.

## What Fixpt does

On Fixpt's compiled machines, the front end is itself FX-26, compiled to
cellular words and present in the same machine as the program (the
"conductor").

- **`stage`** (`stage.rs`): `%fx26-stage` writes the code out as text. A
  front end of the stage's own, made at the first `stage` and kept
  (`make-stage-check`), checks it against the module type, effect and
  result type the program's checker found. It then compiles the code to a
  word that makes `(lambda ((m M)) (lambda () (with m text)))`. That word
  runs on the program's machine, and the procedure it makes is given the
  module: `(sum ok p)`.
- **Sessions** (`sessions.rs`, compiled mode): a compiler of the session's
  own (`make-session-compiler`). Its globals are the session's, and it is
  given what the checker found. A commit compiles the definition, and
  whatever runs again, into the procedure the program calls.
- **516abc8e** (C314, not ported): a compiled word's machine code is kept
  in a code object of its own, which the heap collects. Before it, the
  machine kept every compiled word, so sessions filled it.

## Two ways for kayak

**(a) The front end linked into the compiled program's runtime.** This is
Fixpt's design, except that kayak's front end is KFX compiled to Rust
rather than FX-26 compiled to words.

`kayak-cells` would also contain kayak's checker and cell generator. A
staged piece of code is:
1. checked, by a worker, as the engine does (C305);
2. compiled to a listing;
3. read into the running heap by the listing reader `kayak-cells` already
   has;
4. run, its procedure given the module.

Values never leave the cells heap.

**(b) Staged code given to the engine, as the heaps are now.** This
reuses `vstage.kfx` as it is. But today the engine's heap
(`fx26/heap.kfx`) and the cells heap (`cells/heap.kfx`) have two
different layouts. Every value crossing would need converting both ways:
the module, the evidence, and the procedure the code makes, which would
close over the other heap. Procedures can't be converted at all.

**(c) One heap, then (b).** The two heaps are separate by history, not by
need (the user's question, 2026-11-03). The engine was written first
(C58, C64) with a layout of its own. Tier 1 came after the decision to
take Fixpt's formats whole (C121), on Fixpt's layout (C122). Both are KFX
programs over a flat array of `i64`, so the engine could move onto tier
1's heap.
- Data would then cross freely: modules, evidence, lists, strings.
- Procedures would still be of two kinds, engine closures over IR code
  and cellular closures over words. A call from one to the other needs an
  adapter each way, much as Fixpt's conventions adapt one kind of
  procedure to another.
- It costs a port of the engine's heap, words, primitives, machine and
  collector. It's worth doing for its own sake (one format, Fixpt's,
  across kayak), but it's a project of its own.

**Recommendation: (a)**, without waiting for (c). Under (a) staged code
becomes cells, the tiers' own form, so no procedure ever crosses between
machines. (b) can't pass procedures until (c) is done, and even then it
needs the two adapters. Whether to unify the heaps is worth deciding
apart from this (question 4).

## The design, under (a)

- **One program**: `cells/cells.unit` plus the front end's files (`kfx/`'s
  checker and reps, `fx26/`'s lowering helpers, `cg.kfx`, `cg-top.kfx`,
  the listing writer).
  - The two share 25 top-level names (`st`, `code`, `top`, `hex`,
    `name-of`, `words-of`, …), since KFX has one namespace. Rename them on
    the cells side.
  - Either `kayak-cells` grows by the front end, or a second binary does
    (question 1).
- **Compiling `%stage`**: `cg-top.kfx` compiles `%stage`, `%reify`, the
  bounds and `%session-check` to calls of new cells primitives. Each gets
  its rep and evidence names as literals, as the engine's lowering does
  (C305, C307).
- **Running one** (a new `cells/stage.kfx`):
  1. The code, a datum in the cells heap, is read back as the checker's
     `sexp`, placed by its offsets (C312).
  2. The rep is filled from the evidence (C307's filling, shared).
  3. A worker checks the code; a refusal comes back as a `stage-failure`
     with its places.
  4. The thunk program is compiled by `compile-cells` to a listing, its
     words named with a prefix of the stage's own.
  5. `listing-words!` reads it into the running heap.
  6. Its `program` word runs, and the result is given the module.
  - The engine-independent parts of `vstage.kfx`, from the reading-back
    to the session's checker state, move to a file both use, which is
    question 3.
- **The tiers**: new words run as cells under every tier. Tier 2 hands
  back what it lacks, and tiers 4 to 6 compiled only what they loaded.
  Staged code therefore runs at tier 1 speed at first; that needs
  checking on each tier (step 3).
  - Compiling staged words to machine code is a later step. It needs
    516abc8e first, so the code can be collected.
- **Sessions**: the session's checker state as on the engine (C310,
  C311). Its globals are cells globals named `fx26-session-K:NAME`, and
  each definition, and each rerun, is a listing read in at commit.
- **Tests**: the stage test runs each of Fixpt's stage programs on tier 1,
  and on the other tiers once they run them, against `fixpt run`, as it
  runs the engine now.

## Steps

Status (2026-11-04): (a) chosen, the user's ("Start on (a)"), questions 1
to 3 taken as leaned; step 1 done (C335); the shared half of staging
(C336); step 2 done (C337), 13 of Fixpt's 23 stage programs on tier 1;
step 3 done (C338), the same under every tier; step 4 done (C339), 16 of
23, the seven left sessions'.

1. **The front end in a cells program**: rename the clashes, build the
   combined unit, and measure its build time and binary size. No behaviour
   changes.
2. **`stage` on tier 1**: compile `%stage`; run it through `cells/stage.kfx`.
   Test: Fixpt's stage programs without evidence or sessions.
3. **The other tiers**: staged words run as cells under `--hand`,
   `--compiled`, `--regs` and `--direct`. Find where they don't.
4. **Evidence and bounds on tier 1** (`%reify`, filling).
5. **Sessions on tier 1**, then the other tiers.
6. **Later**: 516abc8e, then staged words compiled to machine code.

## Questions for the user

1. **Which binary**: the front end inside `kayak-cells`, or a second binary
   (`kayak-cells` stays small, and only programs that stage need the large
   one)? I lean towards one binary, as Fixpt has: a program shouldn't need
   a different runner depending on whether it stages.
2. **Speed of staged code**: is tier 1 speed for staged code enough for now,
   with machine-compiling it (and 516abc8e) after sessions? I lean yes.
3. **Shared code**: move the engine-independent half of `vstage.kfx`
   (reading code back, refusals and their places, reps' filling, a
   session's checker state and redefinition rules) to a file the engine and
   the cells program both load, rather than copying it? I lean yes. The
   engine's half (its globals and its lowering) and the cells half stay
   apart.
4. **One heap**: move the engine onto tier 1's heap (Fixpt's layout), as a
   project of its own, before, after, or instead of (a)? I leaned after.
   **Decided, the user's (2026-11-03): before.** The plan is
   `docs/research/one-heap.md` (C317). Questions 1 to 3 wait for it.
