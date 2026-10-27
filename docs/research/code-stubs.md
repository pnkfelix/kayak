# Code stubs: every procedure's code runs, so native calls need no test

Status: kayak's request to the Fixpt team (the user, 2026-10-26: "We can
try delivering the request to the fixpt team"), from the user's design
intent for bloblets, raised in kayak's review of its departures from
Fixpt (`docs/research/departures.md`, departure 4; C230). The details are
this note's proposal, for the Fixpt team to adjust as they implement it,
and to say so in their copy.

## The intent

A bloblet's layout (fields, then a binary suffix) is meant to let a
procedure's code be *uniformly callable*: whatever a closure holds as its
code is a code bloblet, and its suffix is machine code. Native code then
calls any closure the same way, with no test of what it is. A cellular
word, which an interpreter runs, carries in its suffix a tiny stub that
dispatches to the interpreter.

## What the convention does today

The declared convention's `closures` clause (`declared-conventions.md`,
step 5):

    (closures (kind 41) (code-field 2) (first-free-field 3)
              (native-when-code-at-least code-lo))

A native call of a closure it was given (made code's, e.g.
`made-code-x86_64/frames.fx`, instructions 28 to 33):

    mov r9, [rbx-20]        ; field 2, the closure's code
    cmp r9, [r15+216]       ; at least the state's code-lo?
    jb  foreign-path        ; no: not native code; by `foreign`
    add r9, [r15+208]       ; yes: plus the distance, where it runs
    call r9

So every call of an unknown procedure pays a compare and a branch, and
a runtime must keep every heap object's address below every code
bloblet's: Fixpt by its layout (the code area at the reservation's top),
kayak's tier 1 by an order checked when its reservations are made (C230),
since its code area is a reservation of its own.

## The request

1. **Every procedure's code field holds a code bloblet in the code area,
   whose suffix runs.** For native code, as today. For a cellular word,
   a stub: the closure is already in the closure register and the count
   in the count register, so the stub is the `foreign` routine's entry,
   a tail jump:

       x86_64:  jmp [r15 + foreign]              ; 41 ff a7 20 01 00 00
       arm64:   ldr x16, [x24, #foreign]; br x16

   (as the native adapter, `fixpt-native`'s `adapter()`, less its load of
   the free value). Likewise every other kind `foreign` takes today
   (continuations, primitives as values, …): its field 2 a code bloblet
   whose stub enters `foreign`, or its own entry.
2. **Words in the code area.** A stub runs only where its bloblet is
   executable, at its reference plus the distance: so cellular words,
   which Fixpt publishes with their fields frozen already, are made in
   the code area, which never moves and which the collector marks and
   sweeps (`object-model.md`, "A collected code area").
3. **The convention says so.** The clause becomes, say,
   `(closures (kind 41) (code-field 2) (first-free-field 3) (code-always-runs))`,
   and native calls drop the test:

       mov r9, [rbx-20]
       add r9, [r15+208]
       call r9

4. **Code written for the old clause still works**, if a runtime keeps
   `code-lo` at its code area's lowest: a cellular word, now in the code
   area, passes the old test, and its stub enters `foreign`, as the old
   path did. So the change need not break `made-code*/*.fx`; new code may
   drop the test.

## What it buys

- One compare and branch fewer on every call of an unknown procedure.
- No address-order requirement between a runtime's heap and its code
  area: kayak's tier 1 drops its check (C230), and a runtime may lay its
  spaces out as it likes.
- **A patch point for tiering.** A word's stub is where its code is
  entered from native code: when a word is compiled to native code
  later, its stub can be made a jump to the compiled code (or the
  word's code field pointed at it), and every native caller then runs
  the compiled code with no change of its own.
- Native code and cellular code callable alike, which the adapters
  between conventions (`%fx26-convert`) then need for fewer cases.

## What it costs

- Every word lives in the code area: the area holds more, is swept more,
  and making a word (at load, by `eval`, by a REPL) writes through the
  area's writable view and flushes the instruction cache for its stub's
  bytes.
- Each kind of procedure needs a stub (shared, one per kind or per
  routine, not one per object, where its entry is the same).
- The stub's bytes are the target's: one per architecture, as the
  adapter's are.

## What each side does

**Fixpt** (first):
- every kind a closure or a call can hold as code given a code bloblet
  that runs, words made in the code area;
- the convention's clause, and native calls without the test (keeping
  `code-lo` meaningful for code written for the old clause);
- its machines and compilers so; tests: a made-code call of a cellular
  closure, a continuation, a primitive, with no test; the old
  `made-code*/*.fx` unchanged and passing.

**Kayak** (after the pin moves): the same in kayak-jit's routines and
tier 1 (words in tier 1's code area; the stubs; the address-order check
of C230 dropped), and in kayak's compiler to cells where it writes the
convention.

## Open questions

- Which kinds `foreign` takes today, each's stub or entry.
- Whether a word's stub should be its own bloblet (shared) or each word's
  suffix (per word, so that it can be patched when the word is compiled):
  the patch point argues for per word.
- Memory: how much the code area grows when every word is there, and
  whether words made by a REPL or `eval` should be swept as eagerly.
- Whether macOS's per-thread `MAP_JIT` toggling, kept as a hardening
  option (kayak's `TODO.md`), changes how words are written there.
