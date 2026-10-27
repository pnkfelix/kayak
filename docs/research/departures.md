# Where kayak's runtime departs from Fixpt's

Status: for review with the user (`TODO.md`, C230: "I'm sure some are
totally reasonable, but I'd like to have better awareness of them
regardless"). Each item: what kayak does, what Fixpt does or means to, the
reason kayak gave, and whether a program can tell. Gathered from all of
`CLAUDES_CHOICES.md`, `native.md` and `fixpt-tiers.md` against Fixpt's
`docs/object-model.md` and `docs/research/declared-conventions.md`, at
C231; the first item checked in the code. Out of scope: where kayak's
checker accepts or refuses differently, and KFX's own Rust object model
(listed last, as a question).

## Observable: a program can tell

### 1. Made code's `poll` never stops it (C116)
- **Kayak**: kayak-jit's `poll` refills the fuel from the reserve and
  returns (`kayak_jit_poll`, `crates/kayak-jit/src/lib.rs`): no step limit,
  no preemption, no collection. `overflow` ends the call with a
  stack-overflow trap.
- **Fixpt**: `common_poll` gives the next slice, traps when the budget is
  spent, preempts when the scheduler wants; `overflow` flushes the stack
  cache into the heap and goes on.
- **Why**: a first version, not returned to; for `overflow`, kayak's native
  stack is the thread's, with no cache to flush.
- **So**: a loop in made code is never stopped or preempted; very deep
  made-code recursion traps where Fixpt's goes on.
- **Decided** (review, 2026-10-26): to be made Fixpt's, the step limit, a
  collection at `poll` and preemption with made code's frames live, as
  one piece of work (`TODO.md`).

### 2. Steps counted one way on every tier (C125, C128, C130, C138, C229)
- **Kayak**: every tier spends tier 1's one count: a step a cell, a step a
  register instruction (tier 6 a block's at once, the same total); the
  limit is the whole program's, tier 1 running the listing's one `program`
  word; a callback from made code counts too.
- **Fixpt**: the register tiers poll at entries and back edges only; the
  limit is per top-level form.
- **Why**: limits and preemption at the same points on every tier, as
  Fixpt's `rsinterp` has them.
- **So**: where a step limit fails and where tasks interleave can differ
  from Fixpt's native machine (C125: `process-fails.fx`,
  `allocating-short.fx`); C229's test had to shrink its churn.
- **Reviewed** (2026-10-26): measured, the counting is Fixpt's cellular
  machine's (one form of 1.5M iterations passes on both, of 3M fails on
  both); the scope is not: two forms of 1.5M each pass on Fixpt and the
  engine, and fail on tier 1, whose one `program` word holds every form.
  **Decided**: the rule (a budget per top-level form) is FX-26's, written
  down as kayak's; kayak's compiler marks each form (`%form`), tier 1
  obeys; Fixpt's listings, unmarked, a cross-check's limit (`TODO.md`).

### 3. The engine is not Fixpt's evaluator (C64, C134, C219, C220)
- **Kayak**: a CEK machine over its own heap; frames never written,
  continuations copied when resumed; it never recurses on the Rust stack;
  its scheduler shares messages rather than copying them.
- **So**: `native/crossing-overflow.fx` runs on it where Fixpt fails;
  `eq?` on a message sent differs (checked: Fixpt and tier 1 `#f`, the
  engine `#t`), as FX-26's `eq?` allows on immutable data.
- **Reviewed** (2026-10-26): both kept. A machine's resource limits are
  limits only within a space-efficiency class kayak's definition is to
  state (Clinger's, the user's ask); `TODO.md`.

## Internal: memory, speed, code bytes

### 4. Tier 1's code area is its own reservation (C226, C229, C230)
- **Kayak**: a reservation apart from the two spaces, an execute view by
  `code:exec-view/1`; a word's index past the heap's read from it. Marked
  and swept as Fixpt's (C229). It must lie above both spaces, for the
  convention's `code-lo` test (C230): reserved first (top-down, Linux),
  else after (bottom-up, macOS, not run), else no code area.
- **Fixpt**: one reservation: the spaces, the arenas, the reaps, the code
  area at the top (object-model.md, "A collected code area"). `native.md`
  decision 3 said kayak would do so too.
- **Why**: two-space copying swaps `mem` at each collection, so a range of
  it does not stay; one reservation "would have moved every space's
  indexing".
- **So**: none, but `native-code?` is `#f` where no area fits.
- **The user's proposal**: a dispatch stub in every code bloblet's suffix,
  so native calls need no `code-lo` test at all; for Fixpt first.
- **Reviewed** (2026-10-26): kept until the stub lands; its ordering check
  then dropped. Code mapped twice (read+write, read+execute) stays the
  baseline, `MAP_JIT` toggling a macOS hardening option; the macOS path
  to be run on the user's Mac (`TODO.md`).

### 5. Collection under made code: values copied out and back (C227–C229)
- **Kayak**: a callback through `foreign` runs `native-callback`: the
  machine's registers kept on tier 1's return stack, the live native
  frames' values (found by their stack maps, `traced_slots`) on its data
  stack, where a collection moves them, written back before native code
  resumes; the called closure kept as a root.
- **Fixpt**: its collector traces the native frames in place, or flushes
  them into the heap.
- **Why**: tier 1's collector knows nothing outside its own spaces.
- **Reviewed** (2026-10-26): Fixpt does not trace them in place either:
  before a collection in a call-out it flushes the run's frames into the
  heap as `native-frame` objects and restores the innermost after
  (`callout`, Larceny's way), the mechanism its `overflow` uses too.
  Folded into departure 1's work: frames flushed into the heap, which
  retires this copy (`TODO.md`).

### 6. The convention's routines: Rust, on a heap handed over per call (C226, C227, C230)
- **Kayak**: tier 1 calls made code through `kayak:call-native/1`, words in
  a mailbox; kayak-jit's Rust `cons` and `bloblet` bump tier 1's top,
  handed over and back (`m[10..11]`); the card range and the code area
  swapped in per call (`m[12..15]`); `foreign` alone reaches KFX.
- **Fixpt**: callouts from its state, on its one heap.
- **Also against kayak's own plan**: C116 and `native.md` decision 6 meant
  the routines to be KFX procedures reached through fields.
- **Reviewed** (2026-10-26): Fixpt's routines are Rust too, but collect
  (`may-collect`); kayak's `cons` and `bloblet` never do, bumping through
  the whole reservation and panicking when it is full: observable, as a
  failure of space safety. Folded into departure 1's work (`TODO.md`).

### 7. The collector is Cheney's, without a barrier that is read (C122, C212, C227)
- **Kayak**: Coopt's baseline: two spaces, no nursery, collecting only at
  safepoints; no write barrier in tier 1; made code's card marks never
  read.
- **Fixpt**: generational, a nursery of 2^20 words, a card-marking barrier
  after every store on every machine.
- **Why**: Coopt's steps to be ported one by one; not yet.
- **Reviewed** (2026-10-26): left; the runtime to be revised whole when
  Coopt's runtime and code generation are brought in.

### 8. Regions and reaps in the one heap (C122, C124, C125)
- **Kayak**: `%region-enter`, `%reap-enter`, `%region-new` count;
  `%region-exit` frees nothing; `%region-cons` is `cons`.
- **Fixpt**: arenas and reaps areas, freed at exit; a prompt's height
  counts regions live.
- **Reviewed** (2026-10-26): left for the revision from Coopt's runtime;
  unobservable, even in space.

### 9. Tier 6's frames are kayak's (C138, C147, C210–C212)
- **Kayak**: register code as native procedures by `call`/`ret`; slots on
  tier 1's data stack, a small record per call on the machine stack; no
  stack maps; anything tier 5 would hand back leaves tier 6, its records
  written out as return entries.
- **Fixpt**: native frames as bloblets with stack maps on a native stack,
  flushed into the heap before a collection; its tier 6 allocates and
  calls out.
- **Why**: Fixpt's tier 6 files assume its heap and `DState`, and tier 1,
  an interpreter in KFX, must be returned to.
- **Reviewed** (2026-10-26): left for the revision from Coopt's code
  generation; overlaps the frame-flushing work (`TODO.md`).

### 10. Register code's registers on x86_64 (C137, C139, C140)
- **Kayak**: `REG1` `rdi`, `REG2` `r14`, `REG3` `r10`, `REG4`–`REG8` in the
  state; the link popped into the state at entry. `--fixpt-calls` gives
  Fixpt's (`rdi`, `rsi`, `r8`, the link on the stack). arm64 as Fixpt's.
- **Why**: the routines' scratch registers; `r10` once the heap's base.
- **Reviewed** (2026-10-26): left for the code-generation revision.

### 11. Tiers 2–5 run on tier 1's state (C126, C128, C129, C131, C136)
- **Kayak**: tier 1's heap, stacks (growing up) and registers, KFX flat
  arrays; a cell without a routine handed back to tier 1; tier 4 compiles
  `hand.kfx`'s routines, not Fixpt's `native.fx`; `prim` numbers kayak's.
- **Fixpt**: its own state, stacks growing down, Rust callouts.
- **Why**: one routine at a time, always agreeing with tier 1.
- **Reviewed** (2026-10-26): measured 1.2-5x behind Fixpt's machine of
  each kind, worst at tier 6 on calls (`tak` 5x) and allocation (`lists`,
  `closures` 4-5x); not hand-backs (rare) but the call protocol and the
  collector's policy (hundreds of times Fixpt's collections). To be
  addressed before the Coopt adoption; the table and the order of work in
  `TODO.md`.

### 12. The native tiers' code is never freed (C126, C128, C141)
- **Kayak**: the hand machine and every compiled word in one bloblet of
  kayak-jit's area, made at start, kept.
- **Fixpt**: a code space collected by copying, words kept by weak
  references.
- **Reviewed** (2026-10-26): harmless while tier 1 runs one listing, its
  words fixed and compiled once; no action (`TODO.md`).

### 13. Values are addresses, each space its own reservation (C139)
- **Kayak**: addresses by default, as Fixpt's; `--indexes` kept (no native
  code there); each space 2^28 words reserved apart; arm64 keeps a base
  register (`x19`, 0).
- **Fixpt**: addresses, one reservation.
- **Done** (C244): `--indexes` removed; arm64's base register left.
- **Reviewed** (2026-10-26): `--indexes` to be removed (the user);
  addresses the one mode. Found here: tier 1's word accessors test every
  index for the code area, a cost of departure 4 on its hottest path
  (`TODO.md`, performance).

### 14. Tier 1's runtime frees nothing (C3, C43, C142, C148–C151)
- kayak-rt, which KFX compiles to, never frees; hot-path leaks were taken
  out by hand.
- **Reviewed** (2026-10-26): KFX's business, but for where it keeps an
  FX-26 machine from FX-26's promises: measured, tier 1's `write-out`
  leaks ~32 bytes a call (KFX's built-ins' results, and the string's
  copy), not safe for space; to be fixed first (`TODO.md`). Collections
  measured flat (10,000 in 5 MB, as 1,000).

### 15. Interned data's weak pairs carry a trailer (C231)
- **Kayak**: tier 1 finds a bloblet's header through its trailer when it
  has fields, so its weak pair has one: the car field 3, the cdr field 2.
- **Fixpt**: two fields, the car 2, the cdr 1.
- **Reviewed** (2026-10-26): the trailer was a workaround: Fixpt finds the
  header of a trailer-less bloblet with fields by scanning back
  (`main_of`), tier 1 does not, a latent bug. The scan to be added, weak
  pairs then Fixpt's layout (`TODO.md`).

## A question rather than a departure: KFX's own object model
KFX compiles to Rust over kayak-rt (C3, C30, C43, C45, C102, C116): values
in leaked typed arenas, `letrena` a Rust arena, composable continuations
Rust futures resumed once, processes OS threads with shared messages,
values crossing into made code copied. A different object model by
design, partly observable (thread order, copies at the native boundary);
whether it counts as an FX-26 runtime's is for the review.

**Reviewed** (2026-10-26): KFX may differ by refusing (K1). One possible
hole, the copy of values crossing into made code, to be probed; if real,
crossing data to be FX-26's flat data and blobrefs, shared in place
(`TODO.md`). The review is done.

## Missing, not different
Tier 3 (stencils); the standard input's readiness waited for, and the
scheduler's seed (C125); tier 1 without Fixpt's cross-machine stack
checks (C134).
