# Where kayak is going, and how to tell drift from progress

Status: 2026-11-06. Written after the user said kayak had "forgotten
that element of the plot" and become "some strange other beast"; the
user's answers to its questions are in §6, and change §5.

## 1. The goals, in the user's words

- **G1. One source.** Fixpt implements FX-26's compiler twice, in Rust
  (`fixpt-fx26`) and in FX-26 (its front end, `compile-plan.fx`), and
  the two drift: `fixpt check --reps` prints `NAME differs REP` because
  Fixpt's two checkers disagree. kayak's answer is one source, in KFX,
  a syntactic and semantic subset of FX-26 *so that* it can be fed into
  itself: transpiled to Rust, and run as FX-26. The single source is
  meant to replace all of Fixpt, "*eventually*. But we're clearly nowhere
  near that point today."
- **G2. A runtime in Coopt's style.** kayak's core runtime rewritten in
  the async style of `~/Dev/LangPlay/coopt` (§3). Already in PLAN.md
  item 5 and §6.1, and in the departures review (TODO.md, 2026-10-26:
  "we will revise the runtime entirely when we start pulling in Coopt's
  runtime and codegen design").
- **Self-hosting is driven, not deferred.** The source fed into itself is
  the point of G1. Interpretive overhead is answered by native code
  generation, not by staying on the Rust path.

**The test for any piece of work**: say how it serves G1 or G2. Runtime
work is welcome where it is likely to serve them (the user,
2026-11-06); work that deepens what G2 will replace probably does not.

## 2. Where kayak is today (measured 2026-11-06)

**What serves G1 and holds:**
- The compiler is one KFX source with two builds, both checked by every
  full check (`scripts/fixpoint.sh`): the Rust-built `kfx` makes
  `boot/src/kfx/` again, and Fixpt, running the same source as FX-26
  with `kfx/prelude.fx` (stage 0), makes the same files byte for byte.
- KFX ⊂ FX-26 is enforced: Fixpt answers every corpus program
  (`tests/fixpt-answers`), and kayak may accept nothing Fixpt refuses.
- The checker is converging on Fixpt's: 446 programs whose every
  definition's type rep is Fixpt's byte for byte, 63 that differ
  (`checks-summary reps`); Fixpt's 28 logical-type programs all behave
  as Fixpt's checker does (U0-U3).

**What serves G1 but is broken:**
- **The compiler on kayak's own runtime** (the second self-hosting, C66,
  2026-10-12) no longer runs. On the IR engine (`kayak-fx26 --run`):
  `nothing is named rep-nodes`, a primitive the compiler has used since
  the type reps (C301). On the cellular machine: the compiler compiles
  to words (a 19.7 MB listing, 3 s, the prelude first), but the listing
  names two words `map@lambda@24085`. A compiled expression's position
  has no file, so two files' lambdas collide, and so can their copies
  (`r-copy-of`, keyed by name and position), which would be wrong code,
  not only a wrong name. Fixpt names a lambda outside the first file
  `NAME@lambda@FILE:START`.

**What drifted** (by the test above):
- Tuning the Rust-built compiler (C361-C365, today): it is the bootstrap
  path, not the goal.
- A third copy of Fixpt's cellular code generator (`fx26/cg*.kfx`), its
  machine's tiers 1-6, word code, the arm64 port: built to match Fixpt's
  machines rather than to replace a duplicated piece of Fixpt, and on
  the runtime G2 means to revise.

## 3. Coopt's runtime, as its PLAN.md has it

(`~/Dev/LangPlay/coopt` at 9283bba; M0-M8, M10, M11 done.) Coopt is a
from-scratch fork of Fixpt, hand-written Rust:
- **I1 determinism**: preemption by fuel (a counter charged per step),
  a seeded scheduler, steps as the clock, fixed hashers, allocation
  deterministic.
- **I2 rooting by types**: `Value<'gc>` branded, live only inside a
  `mutate` scope; across an `.await`, an owned `Root<T>`; stores through
  the mutation context, so no barrier can be skipped. `unsafe` in two
  crates only (`coopt-mem`, `coopt-sys`).
- **I3 one scheduler for everything** (user tasks, GC slices, compile
  tasks, I/O), prioritised, and it sees each task's effect: tasks whose
  effects do not interfere may run when fuel runs out.
- **I4 no external async runtime**: Rust futures only as the shell
  around a budgeted engine slice; engine state in the traced heap.
- **I5 no Rust frame between a capture and its prompt**: the
  interpreter's stack in the heap.
- A moving collector from the first commit (Cheney, then generations,
  then heap regions); a skimmer and a query compiler (salsa-style, a
  form the unit, each query run a task); runners (sim, replay, live) for
  all I/O; one event stream.

**Where kayak's async work differs today.** kayak's October design (C45,
PLAN.md §6.1) compiles KFX to Rust so that a Rust `async fn` *is* a
frame's continuation: suspension compiled, one-shot. Coopt keeps engine
state out of futures (I4). kayak's FX-26 engine and cellular machine
are a third arrangement, closer to Fixpt's than to Coopt's.

## 4. How the goals meet

The single source (G1) self-hosts on kayak's own runtime (G2), compiled
natively: the compiler, as FX-26, compiled by itself onto a Coopt-style
runtime, Rust then only the bootstrap. The departures review already
ordered it so: "self-hosting, after the Coopt revision", because today's
native code runs on tier 1's state, and compiling tier 1 by it would be
circular.

## 5. Proposed order

1. **Restore the compiler on kayak's runtime, as a check**, with the
   smallest fixes: positions that carry their file (the first file's
   unchanged, so single-file listings stay Fixpt's). Serves G1 now, and
   the fix carries over to any runtime. Not further work on the machine.
2. **A design note before code: Coopt's runtime for kayak**
   (`docs/research/coopt-adoption.md`): which of Coopt's invariants and
   crates kayak takes; which layers are KFX (the single source) and which
   hand-written Rust (`unsafe` crates only); what of kayak-rt, the engine
   and the cellular machine survives; how one-shot compiled suspension
   (C45) and Coopt's heap-stack engine (I4, I5) meet. Read from Coopt's
   crates and research notes, not from memory.
3. **The runtime rewritten** per that note, in steps each checked, and
   the compiler self-hosted on it.
4. **Toward replacing Fixpt, one duplicated piece at a time**, the
   checker first (the reps' 63 differences, then what Fixpt would need
   to build its checker from kayak's source). "Eventually", and planned
   in steps.

## 6. The user's answers (2026-11-06)

1. **Which layers are async**: "All of them; the whole thing should aim
   to be async and incremental-computation pilled": kayak-rt, the FX-26
   runtime, and the compiler as tasks and queries.
2. **Rust or KFX for the runtime**: "Runtime can be as much rust as makes
   sense." Putting so much of it in KFX "probably isn't needed"; what
   might be is **lang items**, as rustc has them: one vocabulary of names
   that the runtime, the standard library and the compiler share, so that
   each refers to the others' pieces by the same names.
3. **Order**: restore self-hosting first (§5, step 1).
4. **The machine work**: the user asked whether it is salvageable or
   better started over. The assessment, §7.

## 7. The machine work: what to keep, what to start over

The FX-26 side of kayak is two halves, and they answer differently.

**The compiler half: keep.** `fx26/ir`, `lower`, `cg`, `cg-top`,
`listing`, the register code (`regs-*`, `kfx/regs-isa`), about 7,500
lines of KFX: FX-26 to Fixpt's cellular words and register code, in
Fixpt's formats, its listings compared with Fixpt's over the corpus. It
is a third copy of a piece Fixpt has twice (`cellular.rs` in Rust,
`compile-plan.fx` in FX-26): exactly what G1 means to be the one copy of,
in time. Its instruction encodings are the native code generator's
start.

**The runtime half: start over, in Rust, in Coopt's style.** The
cellular machine (`cells/`: its heap, Cheney collector, interpreter,
hand machine, compiled words, about 10,400 lines) and the IR engine
(`fx26/v*`, `vm`, `words`, `table`, about 2,400) are KFX ports of Fixpt's
Rust runtime, routine by routine. They have what Coopt treats as
foundations late or not at all: no prompts, continuations or marks on
the machine ("Not yet", `machine.kfx`); no fuel-driven, effect-aware
scheduler shared with GC and compiling; a collector with no barrier and
no branded rooting; native code running on tier 1's state, so circular
to self-host on. Two runtimes, neither Coopt's shape. What carries over:
Fixpt's object formats (kayak's memory: Fixpt's tagging and layout
everywhere), the corpus and its comparisons with Fixpt's answers, which
are the new runtime's oracle, and what was learned (`departures.md`,
`performance.md`).

**The REPL** (`live-repl`, `repl-tasks`, `start`, about 5,000 lines) is
a front end over the runtime; it moves to the new runtime when there is
one, as Coopt's REPL is a client of its queries.

So the order becomes: restore the self-hosting check on today's machine
(step 1, small, and the compiler half's fix carries over); then the
Coopt-adoption note, now with lang items in it and the runtime in Rust;
then the new runtime, the compiler half kept and retargeted, the old
runtime retired once the new one passes the corpus and self-hosts.
