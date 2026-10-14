# The engine's REPL: Fixpt's features, Coopt's foundation, entries that run while you type

Status: design, 2026-10-13; stages 1 to 3 done (C94, C95), checking as
you type (C96, C97); the user's: "spend some time working on
improving the REPL experience. Fixpt's REPL is the most sophisticated …
model the high-level feature set off of what it offers. But Coopt's REPL
… has the stronger foundation, … everything being async-pilled on that
runtime." And: "take in an expression, emit a live-updating line that has
a pending animation …, and then immediately issue the next prompt", with
shades of FrTime (Cooper and Krishnamurthi, ESOP 2006 / the 2004 work).

## What there is

- **Fixpt's** (`fixpt/crates/fixpt-cli/src/fx26.rs`, `lineedit.rs`,
  `help.rs`): a line editor of its own (raw mode by `stty`, history in a
  file, completion of names, a form edited whole until it is complete),
  `value : type ! effect`, and commands: `,help NAME`, `,apropos`,
  `,fits`, `,returns`, `,time E`, `,load FILE`, `,step-limit`,
  `,list-outdated`, `,rerun-outdated`, `,redefine b|r`, `,pending`,
  `,disassemble`, `,code`, `,quit`; `,help` as a hole inside a form.
  Synchronous: one form read, checked, run, written, then the next.
- **Coopt's** (`coopt/crates/coopt-repl`): every part a task on Coopt's
  deterministic scheduler, by priority (input first): keys read by a
  console task; each draft checked as it is typed; an entry run as a task
  in fuel-sized slices, ^C delivered at the end of a slice; output written
  while input waits; record and replay of a session. One entry at a time;
  an Enter during a run queues the next.
- **kayak's** (`fx26/repl.kfx`, `crates/kayak-fx26/src/main.rs`): a form
  read by `read_line` until balanced, checked by the compiler's checker,
  lowered, run to its end on the engine's machine, `value : type !
  effect`. No commands, no editing, no ^C, no `load-module`.

## The foundation: Coopt's, as kayak has it

kayak took Coopt's design and not its crates (PLAN §6.1; C48–C52): one
scheduler of its own, deterministic, the world's answers the only input,
so that a run records and replays. The REPL is built the same way, not on
Coopt's library (which would put an executor in Rust whose policy is
Rust, against K7, and give `kayak-rt` dependencies):

- **The REPL is a function of what arrives.** Its state is the engine's
  (in KFX); one step takes the keys that arrived and the time, runs the
  entries in slices, and gives the bytes to write. The Rust around it
  only shuttles bytes and time: raw mode, a reader thread, a tick. So a
  session is recorded as its arrivals and replayed exactly, and a test is
  a script of keys whose screen is compared (Coopt's sim runner).
- **An entry is a task: the machine's registers.** The engine is a CEK
  machine whose continuation is in its heap, so pausing an entry is
  keeping its seven registers, rooted for the collector; resuming is
  `run` from them. A slice is a count of steps (fuel), as Coopt's.
- **Priorities as Coopt's**: input first (each step handles every key
  that arrived before running anything); then the entries, round robin, a
  slice each; then the screen.

## The screen: a live transcript

An entry, once complete, is given a number and a line, and the prompt
comes back at once:

```
fx26> (fib 30)
[1] ⠋ running … 0.4 s
fx26> (+ 1 2)
[2] 3 : int ! pure
fx26> █
```

- A running entry's line shows a spinner, its time so far, and the last
  line it wrote, if any (its progress). When it finishes, its line is
  rewritten in place: `[1] 832040 : int ! pure (1.3 s)`; or `[1] !
  evaluation failed: …`; or `[1] interrupted`.
- The lines above the prompt that are on the screen are rewritten in
  place (ANSI: up, clear, down); a line scrolled off the screen is not,
  and its result is written as a new line above the prompt instead. Only
  the screen's text, never the draft being typed, is disturbed: the
  prompt line is redrawn after.
- What an entry writes (`display`, `write-string`) goes to its own block,
  shown under its line when it finishes; its last line shows while it
  runs.

## Which entries run together: effects

The checker gives each entry's effect before it runs, so the REPL knows
what each may touch (Coopt's tasks declare theirs too). In order to mean
what typing them one at a time would mean:

- An entry whose effect writes no global and does no I/O (pure, alloc,
  reads of globals) runs at once, beside the others.
- One that writes a global, defines one, or does I/O waits for those
  before it that write or do I/O, and those after it that read what it
  writes wait for it: the order of conflicting entries is the order typed.
- A definition is an entry too: its global is pending until it is made;
  an entry that reads it waits.

## Results as names, and waiting on them (FrTime's first step)

`%1`, `%2`, … name entries' results (KFX's names have no `$`). An entry that names a pending result
waits for it, its own line saying so (`[3] waiting for [1]`); this is
dataflow, the REPL's entries a graph. FrTime goes further: a *behavior*
is a value that changes over time, and expressions over behaviors are
recomputed as they change; the REPL shows a behavior's current value,
updated in place. Kept for after the stages below, with a design of its
own: in FX-26 it wants a type of signals and an effect for time, and the
scheduler's clock is the clock.

## Stages

1. **Entries as tasks, the step as a function**: the machine's fuel and
   pausing; an entry's state (registers, rooted); the REPL's step,
   `(repl-step! keys now) -> bytes`; the screen as above, with spinners,
   results in place, `%k`; ^C interrupts the newest running entry (or
   clears the draft), ^D leaves when nothing runs. Scripted-key tests.
2. **Editing as Fixpt's**: the form as one buffer (arrows, ^A ^E ^K ^U
   ^W), history in a file, completion of names; in KFX.
3. **Commands as Fixpt's**: `,help`, `,type`, `,time`, `,load`, `,quit`,
   `,step-limit`, `,apropos`, `,pending` (the entries running and
   waiting), `,cancel K`, `,wait K`.
4. **Effects decide concurrency**, as above; `load-module` in the REPL.
5. **Behaviors** (FrTime), designed first.

## Checking as you type (C96)

The draft is closed off and checked on each change, as Fixpt's REPL
does: a refusal said under the draft when it is about what has been
finished, else a hint for the argument at the end; a finished
expression that only reads and allocates runs early, as a task beside
the entries, its value under the draft. Details: `CLAUDES_CHOICES.md`
C96. The hints are the checker's, at a hole where the next item goes,
as Fixpt's: the argument's type as the arguments before it solve it, a
`tagcase`'s arms not taken, a `sum`'s tags; a refused subform is
underlined (C97); an inference variable said by its binder's name,
`t`, a second of the name primed, `t'`, as Fixpt says them (C98). Left:
speculating a command's forms (`,type E`).

## The front and the machine (C103)

Two threads, each with its own copy of the session: the front reads the
keys, edits, checks and draws; the machine lowers and runs. The front
sends each entry, the draft to run early, ^C and `,cancel`; the machine
sends back each entry as it changes and the draft's run's outcome. So a
key is drawn whatever a run is doing, one step of it however long. The
draft's own check is still the front's, before the next key.

## Coopt's skimmer, and what kayak might take from it

Coopt splits input into forms, and finds the names each defines, before
anything is read in full (`coopt-read/src/skim.rs`; its `PLAN.md` §3.2).
It is lexical only: brackets, strings, characters, comments, and for each
top-level list a sketch of its elements (a name, a list's head, other) to
depth two, which a table of definition heads (`define`, `define-rec`,
`define-type`, `define-datatype`, `define-generative` …) turns into the
names it provides in each namespace; one it cannot read is *opaque*, and
a lookup that misses waits on the opaque forms before saying a name is
unbound. It is fed bytes in chunks of any size (so a keystroke at a
time), allocates only for what goes into a sketch, and runs at 300 to 480
MB/s. Its index, (namespace, name) → form, lets each form's checking find
what it depends on without reading the rest; Coopt's REPL looks up each
name in the draft there and asks for that form's check at typing
priority. Where it and the parser disagree, the parser wins.

kayak's closing off (C96) is already such a lexical pass, over one draft.
Where a skimmer in KFX would pay:
- **The incremental compiler** (`incr.kfx`, C54): memos are keyed by the
  text of the items read whole; a skim would find the forms and their
  names without reading them, so that a changed file reads only the forms
  that changed. Measure first: whether reading is a cost in a warm compile.
- **The REPL's names before checking**: a `,load`ed file's names known (to
  Tab, to `,help`) at once; a draft of several forms, the names its
  earlier forms define offered while its later ones are typed.
- **Pending definitions** (Fixpt's entries waiting on types not yet
  defined): an index says which pending form would provide a name.
