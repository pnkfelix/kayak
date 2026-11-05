# A compiled word's code in its own bloblet (Fixpt's 516abc8e, bb635fe8)

Status: plan (2026-11-04, after C341). The user's "go ahead" (the port, after
the pin move to 8616f2f). Step 6 of `docs/research/compiled-stage.md`:
staged words compiled to machine code need it, as Fixpt's sessions did.

## What kayak's tiers 4 to 6 do now

kayak's hand machines (`cells/hand.kfx` arm64, `cells/hand-x64.kfx`
x86_64) are kayak's own compilers, in KFX, of Fixpt's design before
516abc8e:
- **One block, made once** (`hand-code-x64!`, `hand-code!`): the machine's
  routines, then, with `--compiled`, every word of the listing loaded
  (`words-compiled!`, `registers-compiled!`), each a run of its cells'
  code; encoded, then sealed by kayak-rt (`seal-code`), outside tier 1's
  code area.
- **A word's entry a slot number**, `rt-count + s`; a table at the block's
  start gives each slot its code and its table of where to resume.
  `NEXT`, `execute` and the entry dispatch: below `rt-count` a routine,
  else the slot's code.
- **A compiled word's code branches to the machine's own labels** (`NEXT`
  as `l-next`, giving back as `l-give`, `l-give-pre`), by relative
  branches inside the one block.
So a word read after the machine is made (a stage's, a session's) is never
compiled: it runs as cells (C338).

## What Fixpt does since 516abc8e (x86_64) and bb635fe8 (arm64)

- A compiled word's entry is a **code bloblet** in the heap's code area,
  native part (`make_word_code`): its suffix the code from a 32-byte
  boundary, then its table of where each cell resumes, as offsets; field
  `COMPILED_RESUME` the table's byte. It runs at its reference plus
  `State::code_delta`.
- `NEXT`, `execute` and the entry test the entry's tag: a fixnum is a
  routine's number (`docol`'s for a word of cells), else code.
- The word keeps its code alive; the collector frees it with the word. The
  machine keeps nothing of a word: no slots, no table of words, no list of
  what it compiled. Its code space holds its own code alone.
- A word's code reaches outside itself only by address, through the
  state (the machine's trap, exit and routines it jumps to); other words
  through their entries; register code calls a twin's entry.

## The port, in steps

1. **The machine's code apart from the words' code.** The machine made
   with no words (as `--compiled` off makes it); the state gives each
   routine a compiled word jumps to an address (`NEXT`, the ways to give
   back, `docol`'s entry, the trap, the exit). Test: every tier as now
   (words still compiled into the block meanwhile).
2. **A word's code in a code bloblet** (x86_64): `yword-compiled!` emits a
   word alone, placed and encoded on its own, its branches out by the
   state's addresses; `make-word-code!` in tier 1's heap (cells/heap.kfx),
   the code area's native part, `code-alloc!`, its suffix 32-aligned, its
   resume table after; written through `code-set!`, made seen
   (`code:flush/1`), run at its reference plus `code-distance`.
3. **The dispatch by tag** (x86_64): `NEXT`, `execute`, the entry, a
   resume (the word's own table), register code's call of a twin (tier 5),
   tier 6's adapter. The block's table of words goes.
4. **Compiled as read**: `listing-added!` compiles each word it makes
   (C337, C340), so a stage's and a session's words are compiled to machine
   code too; the collector frees a dead word's code (it marks and sweeps
   the code area already, C229, C234).
5. **arm64** (`hand.kfx`): the same, as bb635fe8.
6. Measure: `engine-bench.py`'s programs on `--compiled`, `--regs
   --compiled`, `--regs --direct`, before and after (Fixpt measured
   compiled cells' calls some 10% dearer, the tag's test, and register code
   faster, `fib 32` 38 to 34 ms).

## Questions for the user

1. **arm64 cannot run here** (this machine is x86_64 Linux; the hand
   machine runs on arm64 macOS only). Fixpt's own interim, at 516abc8e,
   was for arm64 to compile nothing until it was ported, running every
   word's cells, correct but slower. Port arm64 blind (encoded, never run
   until someone runs it on arm64), or take Fixpt's interim until it can
   be run? I lean to the interim: code nobody has run is a guess.
2. **The block's words meanwhile**: step 1 keeps them so that every step
   tests as it goes; step 3 removes them. Fine as one sequence, or would
   you rather have the switch in one go?
