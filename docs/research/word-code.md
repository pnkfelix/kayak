# A compiled word's code in its own bloblet (Fixpt's 516abc8e, bb635fe8)

Status: done (C342 to C344, 2026-11-05), arm64 on Fixpt's interim
(compiling nothing) until it can be run; bb635fe8's port to arm64 open. The user's "go ahead" (the port, after the pin move to 8616f2f) and
"go for it" (this plan: arm64 on Fixpt's interim, the port in steps). Step 6 of `docs/research/compiled-stage.md`:
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

## Done

- **Step 1** (C342): the machine's places a word's code goes to, in the
  state (`st-exports`); a word's code reaches them by it alone.
- **Steps 2 and 3** (C343), as one: steps 2's code bloblets could not run
  before step 3's dispatch by tag, so neither tests alone. Each word,
  emitted alone after the machine is sealed, is a code bloblet of the
  code area's native part (`word-code!`): field 0 its table's place past
  its suffix, the suffix its code then its table, every offset from the
  field's word, so that the reference plus `st-code-delta` (the code area's
  distance less 12) is where they are from. Kind `bloblet`, frozen; the
  collector marks it from the word's entry and frees it with the word, as
  any code bloblet (C229). Its suffix 8-aligned only, as the area's words
  are (Fixpt's starts at 32 bytes; a filler before it would need the sweep
  to know of it): to measure at step 6. The dispatch by tag, as Fixpt's:
  `NEXT`'s word, a resume, register code's call of a twin, tier 6's invoke;
  tier 1's `enter!` too. The block's table of words and its export gone.
  Where the code area is not (no `code:exec-view/1`), nothing is compiled.
- **Step 4** (C344): `compiled-read!` (stage.kfx), which reads a stage's
  and a session's listing into the running heap, then compiles the words
  it added (`words-read-compiled!`: the first of the lists of words, past
  the counts they had), as the listing's were. `cli-stage/stage.fx`:
  77 words compiled on `--compiled` (the program's and its stages'), 150
  on `--regs --direct`, 69 of them tier 6.
- **Step 5** (C344): arm64 compiles no word (Fixpt's interim at
  516abc8e); its machine keeps its empty table of words, which its
  routines read. Its words run as cells until bb635fe8 is ported, which
  needs an arm64 to run on.
- **Step 6** (C344): best of 5, whole process, ms, Fixpt's benchmarks
  scaled up (`fib 34`, `tak 26 18 10`, `count 100000000`, `main 30000`,
  `rounds 30000`, `rounds 200000`), C342's build against C344's:

  | program  | `--compiled` | `--regs --compiled` | `--regs --direct` |
  |----------|--------------|---------------------|-------------------|
  | fib      | 368 → 375    | 137 → 143           | 85 → 83           |
  | tak      | 56 → 56      | 29 → 29             | 18 → 19           |
  | loop     | 1207 → 1235  | 345 → 346           | 178 → 179         |
  | closures | 1389 → 1424  | 550 → 530           | 334 → 333         |
  | lists    | 915 → 922    | 351 → 346           | 197 → 199         |
  | captures | 621 → 621    | 595 → 600           | 596 → 603         |

  Compiled cells some 2% dearer (the entry's tag tested, the table found
  from it; Fixpt measured some 10%); register code as it was (Fixpt's got
  faster, `fib 32` 38 to 34 ms, from what its port changed besides). The
  8-aligned suffix costs nothing measurable here.
