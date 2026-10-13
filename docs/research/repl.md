# The engine's REPL: Fixpt's features, Coopt's foundation, entries that run while you type

Status: design, 2026-10-13; stages 1 to 3 done (C94, C95), the user's: "spend some time working on
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
