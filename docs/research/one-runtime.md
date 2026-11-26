# One runtime: kayak-rt's and kcrt's, unified (TODO §56)

The user (2026-11-24): "I didn't intend for us to end with two wildly
diverging runtimes"; the KFX-to-Rust path "was intended as a way to get a
near trivial bootstrap"; "see how much of the runtime we can unify between
kayak-rt and kcrt ... potentially via Rust's trait abstraction"; then
(2026-11-25) "make a plan to unify the runtimes so that the KFX code and
FX26 code can share as much runtime machine as possible", and carry it
out without waiting for review.

## Where things stand

| | kayak-rt (KFX's Rust) | kcrt-core (kc's native code) |
|---|---|---|
| size | ~21k lines, 7.5k of it `obj/` (the collector) | 1.5k lines |
| layout constants | its own (`obj.rs`, `value.rs`, `sum.rs`), some from `kayak_jit::layout` | its own (`lib.rs`) |
| `string` | UTF-8 after a trailer (C482), its own format | Fixpt's: UTF-32, two code points a word |
| `utf8` | the string's object (kind 1): C525's gap | none yet |
| closures | Fixpt's kind 41 (native-closure) | Fixpt's kind 8 (closure) |
| collector | a heap per thread, generations, fragments, Bartlett's mostly-copying (C478–C524) | Cheney's, one heap, two spaces (C453) |
| roots | conservative: Rust stacks scanned, ambiguous roots pin | exact enough: kc's stack, every word a value or code |
| scheduler | futures (`suspend.rs`, `process.rs`) | none: one native stack |

Shared today: Fixpt's tags and most kinds (each written out separately),
kcrt-core linked into kayak-rt for kc's code run in process, kayak-jit's
code area and its state block.

## The principle

Fixpt's formats everywhere (the memory "Fixpt's formats everywhere"): the
two runtimes share one heap format because they must share one heap. kc's
code runs in process beside KFX's Rust, and the engine's values cross
between them. So a layout is not a trait parameter. Where the two differ in
format, the fix is to adopt Fixpt's in both, not to abstract over the
difference. Traits are for what does differ in kind: how roots are found
(conservative Rust stacks against kc's exact native stacks) and what a
task is (a Rust future against a native stack and its state block).

## The steps

1. **One layout crate, Fixpt's.** `crates/kayak-layout`: Fixpt's
   `fixpt-heap/src/layout.rs` copied whole by `scripts/built-in.py`, as
   kayak-reps takes `fixpt-reps` (checked at the pin). kayak-jit's
   `layout` module, kayak-rt's constants and kcrt-core's are each made from
   it, the names each crate uses kept as aliases, so that no constant is
   written twice. A test per crate that its constants are the table's.
   Small, no behaviour changes. *Serves both runtimes.*
2. **kayak-rt's formats made Fixpt's.**
   (a) `string` as Fixpt's UTF-32 object, kcrt-core's format: `Str`'s
   accessors, `str_lit!`, printing and the string primitives rewritten over
   code points. (b) `utf8` its own object, kind 49 (50 for ASCII): what
   kayak-rt's string is now, so C482's UTF-8 machinery moves there almost
   whole. (c) Then `utf8` and `bytevector` in KFX's `datum` (the checker's
   `datum-members`, `Value`'s kinds), the expander's dropped arms restored
   (C527's stopgap). Each step, every program's answers unchanged.
   *Deepens kayak-rt, to converge it: the user's.*
3. **One collector, over a `Roots` trait.** `crates/kayak-heap`: kayak-rt's
   `obj/space.rs`, `collect.rs`, `roots.rs` and `frames.rs` moved there,
   over raw Fixpt words, not kayak-rt's types. `Roots` gives a collection
   its exact roots (rewritten) and its ambiguous ones (pinning their pages).
   kayak-rt's is its conservative stack scan; kcrt's is kc's stack, all
   exact, nothing pinned, so it behaves as Cheney's does now. kcrt-core's
   `gc.rs` retired; kc gets the heap per thread, generations and fragments
   for free. Allocation: kc's bump pointer and limit are a page's (the
   `%r15` state block's `TOP` and `ALLOC_LIMIT`), kayak-rt's the same
   thread page. *Moves kayak-rt's work into the shared runtime.*
4. **The primitives over words, once.** Strings, numbers (bignums),
   printing, hashing and symbols, written over Fixpt words in a shared
   crate (`kayak-heap` or `kayak-prims`), called by kcrt's routines and by
   kayak-rt's `Str`, `Int` and `Show`. kcrt-core's routines are the start:
   they are already of Fixpt's format.
5. **One scheduler over a `Task` trait.** A task is a Rust future
   (kayak-rt's) or a native stack and its state block (kc's), and one
   run queue, one per thread, with work stealing later, runs both. Coopt's
   scheduler (`coopt-adoption.md`) is the model.
6. **Incremental collection**, designed once, in `kayak-heap` (TODO §54
   step 6; the memory "shared heap may move").

Each step leaves every check passing and is a commit of its own. The order
is by dependency: 1 is a prerequisite of 3 and 4; 2(a) of 4; 3 of 5's
native tasks sharing one heap; 6 last.

## Measures

How much runtime code is shared: lines in shared crates against lines in
kayak-rt and kcrt-core alone, recorded at each step here.

| step | shared | kayak-rt alone | kcrt-core alone |
|---|---|---|---|
| before | kayak-jit's `layout`, `state` | ~21k | 1.5k |
| 1 (C528) | + kayak-layout: Fixpt's `layout.rs` and `value.rs` (1.5k), every tag, immediate and kind of the three crates drawn from it | ~21k | 1.5k |
| 2 (C529–C534) | formats: every object either makes is Fixpt's; kcrt reads kayak-rt's strings, `utf8`s and bytevectors as its own | ~21k | 1.5k |
| 3, the walk (C535) | + kayak-heap: one object walk (an object's extent, its traced fields, its header from its suffix), both collectors' | ~21k | 1.5k |
| 4, strings (C536) | + kayak-heap's `text`: a string's words, `string-hash`, a symbol's hash, search, both runtimes' | ~21k | 1.5k |
| 3, kc in kayak-rt's heap (C538) | kc's code in process allocates in kayak-rt's thread pages, collected by its collector, its stores through its barrier; kcrt's Cheney a built program's alone | ~21k | 1.5k |
| 3, the space (C539) | + kayak-heap's `space` (1.6k lines, kayak-rt's `obj/space.rs` moved whole): pages, a thread's heap, start bits, the bump, the store buffer | ~19.5k | 1.5k |
| 3, the collector (C540) | + kayak-heap's `collect` (marking, Cheney's copy, minor and major, the verifier), `frames`, `laid`, the kept index: kayak-rt gives its roots (`Host`) | ~18k | 1.5k |
| 3, a built kc program (C541) | kc's heap code one copy, kcrt-core's `heap` (kayak-rt's `obj::kc_heap` moved); a built program's heap the shared one, its roots its own `Host`; kcrt's Cheney (`gc.rs`) gone; a stack's base, its spill and native code's cells, kayak-heap's | ~18k | 1.6k |

## Done

- **Step 1 (C528)**: `crates/kayak-layout`, Fixpt's `layout.rs` and
  `value.rs` taken whole (`built-in.py`), their twelve tests passing.
  kayak-jit's `layout` module, kcrt-core's constants and kayak-rt's
  (`obj.rs`, `obj/collect.rs`, `sum.rs`, `value.rs`) each named from it
  (`tag("pair")`, `kind("string")`, the immediates' words from
  `Value::FALSE` and kin), a test in each that the values are those
  written before. No format changed: they were already Fixpt's numbers;
  now they cannot drift. Left: the native convention's offsets (kayak-jit's
  `state`), which are the declaration's, already checked against it.
- **Step 2(c), its `bytevector` (C529)**: KFX's `datum` takes the frozen
  bytevector (Fixpt's `bytevector?`); kayak-rt's `Bytes` a `Value`'s
  member; kc's native code reads one's length and bytes (kcrt routines);
  the expander's Fixpt text unpatched. Its `utf8` waits for 2(b).
- **Step 2(b), `utf8` its own object (C530)**: kayak-rt's `Utf8`
  Fixpt's, as `make_utf8` makes it: kind 49 (50 all ASCII), no fields, no
  trailer, the header's count of bytes exact; its literal a static
  (`utf8_lit!`); `string->utf8` a copy, as Fixpt's. A union can now tell a
  `utf8` from a string, so the checker's `datum` can take it next.
- **Step 2(c), its `utf8` (C531)**: KFX's `datum` Fixpt's union whole;
  `utf8?` and `typecase`'s `utf8` arm in the checker, KFX's Rust, the
  engine and tier 1. Left: kc's `utf8?`, so the expander's shim stays.
- **Step 2(c), the expander's (C532)**: kc's native code tests a `utf8`
  (kinds 49, 50 and Fixpt's slice, 51) and makes its string (kcrt
  routines), so the expander's shim is gone; the engine's bridge gives the
  expander `#utf8"…"` and `#u8(…)` as Fixpt's reader does, atoms, not the
  lists kayak's reader makes of them, and takes them back. Step 2(c) done.
- **Step 2(a), `string` Fixpt's (C533)**: kayak-rt's `Str` is Fixpt's
  object, kcrt's: kind 1, no fields, no trailer, the suffix the count and
  the code points, UTF-32, two a word. `Str` gives its points as one
  `&[u32]` (little-endian), so comparing, hashing, `string-ref` (O(1) now,
  the cursor of C14 gone), `substring`, `string-append` and joining are
  over points, nothing decoded; `text()` a copy where Rust wants a `str`
  (printing, paths, syscalls). A string crosses to native code word for
  word. Every answer unchanged. Its cost, measured on kfx compiling
  itself: 2.96 s to 3.11 s, peak memory 197 MB to 294 MB, the compiler's
  text, mostly ASCII, four bytes a character now. The formats are one but
  for the bytevector (kayak's a trailer, Fixpt's a length word first).
- **Step 2(d), `bytevector` Fixpt's (C534)**: kind 4, no fields, no
  trailer, the suffix its length then its bytes, the header counting the
  suffix's words, as Fixpt's `make_bytevector`; kcrt reads it so. The
  collector's trailer guess (`laid::at`) gone with the last object that
  had one without fields. **Step 2 done: every object kayak-rt makes is of
  Fixpt's format.**
- **Step 3, the walk (C535)**: `crates/kayak-heap`, its first piece:
  `object_at` (where an object ends, which of its fields hold values, by
  Fixpt's table's `traced`) and `header_of_suffix`, over a word-reading
  function, so no `unsafe` there. kayak-rt's `laid::at` and
  `HeapWord::header_ptr`, and kcrt's Cheney copy and scan, each call it;
  their own readings of a header (kayak-rt's untraced kinds written
  `1 | 4 | 5 | 6`) gone. Next in step 3: the space (pages, a thread's
  heap) and the roots behind a `Roots` trait.
- **Step 4, strings (C536)**: `kayak_heap::text`, over a string's code
  points as one `&[u32]`: its words (`string_words`, `string_header`,
  `fill_string`), `string_hash`, `symbol_hash` (kayak-rt's tags' and
  kcrt's interning's, the same word written two ways before), `search`.
  kcrt's string routines read a string's points in place
  (`string_points`) where they decoded each to a Rust `String`: `string=?`,
  `string<?`, `string-hash`, search, `string-append`, `substring`.
- **The barrier, step 3's next decision**: kayak-rt's generations
  keep a store buffer: each written slot that now holds a heap pointer,
  outside the thread's current (young) page, pushed (`space::remember`,
  C523), which a minor collection takes as roots. Fixpt's native code
  marks cards instead (`fixpt-native`'s `x86_64::stored`: a generation
  table by 1 GiB, a card table by 512 bytes, in line, a slow call to the
  heap). For kc's code to allocate in kayak-rt's pages in process, its
  stores need kayak-rt's barrier: a routine `kc_remember(slot)` that
  kayak-rt registers in kcrt's routine table (kcrt-core cannot name
  kayak-rt, which links it), called after each store of a value into a
  heap object (`set`, an array's element, a vector's). The store buffer
  first, as it is what kayak-rt's collector reads; Fixpt's cards if the
  measures favour them, a change to both runtimes then, not to kc alone.
- **Step 3, the roots (C537)**: `kayak_heap::roots::Roots`, a
  collection's roots as each runtime finds them: exact slots (read and
  rewritten) and ambiguous words (pinning). kcrt's Cheney takes its roots
  through it (`Runs`: kc's stack from its pointer to its base, the
  globals' cells, all exact). kayak-rt's are next: its `roots_from`'s
  words the ambiguous ones, its kept blocks' slots the exact.

- **Step 3, kc's code in kayak-rt's heap (C538)**: in process, kc's
  objects are made in this thread's pages and collected with kayak-rt's
  (`obj::kc_heap`), as designed below: the page lent (`space::lend`), each
  object's start noted as it is given back, walked by its header
  (`lent_back`); its frames on the scanned stack, its globals' cells
  roots while it runs; its two stores through `kc_remember`, kayak-rt's
  barrier; a run's room it never gave back zeroed again
  (`kayak:kc-done/1`). kcrt-core's `HeapHost` is the seam: kayak-rt
  installs it, a built program has none and keeps Cheney's copy.
  Measured right under stress (a collection every allocation, freed pages
  poisoned, every collection verified, the barrier's slots too); with the
  barrier off, the verifier stops the run.

- **Step 3, the space (C539)**: kayak-rt's `obj/space.rs` moved whole
  into `kayak_heap::space` (pages and chunks, a thread's bump, the start
  bits, the store buffer, large objects, adoption, kc's room lent): the
  shared heap's, and its one module with `unsafe`. The collection it
  starts is a runtime's, given once (`set_collector`): kayak-rt's
  `obj::space` re-exports the space and gives it its collector before its
  first allocation. Its small functions `#[inline]`, as rustc inlined
  them within kayak-rt; without, kfx's self-compile was 3.5% slower. Next:
  the collector (`obj::collect`, its marking and copying) over the
  `Roots` trait, leaving kayak-rt its roots and its types' traces.

- **Step 3, the collector (C540)**: kayak-rt's `obj/collect.rs` (but its
  tests) and `obj/frames.rs` moved to `kayak_heap::collect` and `frames`;
  the kept-block index (`KeptAt`, `KeptIndex`), the gathered roots
  (`Gathered`) and `scan_words` to `kayak_heap::roots`; a `laid` of plain
  words for the collector. What the runtime gives is `collect::Host`:
  its registers spilled, its roots gathered, what it is told after (what
  it reaches of others' kept blocks, which of its own to retire). The
  space starts the collector itself once a runtime is its host. kayak-rt's
  `obj::collect` is its `Host` (`Rt`) and the collector's tests. **Step 3
  is done**: the heap is the shared crate's; what is kayak-rt's is its
  roots and its types. A built kc program could have it next, its `Host`
  kc's exact stack and cells, retiring Cheney's copy (`gc.rs`).

- **Step 3, a built kc program (C541)**: kc's code has one heap, the
  shared one, built or in process. kayak-rt's `obj::kc_heap` (lending
  kc's code room, giving it back noted) is kcrt-core's `heap`, over
  `kayak_heap::space` directly; kcrt's Cheney (`gc.rs`, 336 lines) gone.
  A built program's `main` gives the collector its `Host` (`kayak-kcrt`'s
  `Built`): the registers spilled, its thread's stack and kc's cells,
  every word ambiguous, as kayak-rt's are (the Rust frames between kc's
  are not exact), nothing kept apart, no other thread. Its heap starts
  empty, its first object asking for room; `kc_remember` is the heap's
  barrier there too. In process kayak-rt's `Rt` is the host
  (`kayak:kc-routine/1` makes it so first). The stack's base, the spill
  and the native cells' run moved from kayak-rt's `obj::roots` to
  `kayak_heap::roots`, both hosts'. Not yet: kc's frames as exact roots,
  so that what only they reach can move.

## Step 3's rest: kc's code in kayak-rt's heap, in process (done, C538)

What it took for kc's code run in process (`kc run`, a transformer) to
allocate in kayak-rt's thread pages instead of kcrt's semispaces (a
built program keeps kcrt's Cheney: it has no kayak-rt):
1. **Where objects start.** kayak-rt's collector pins a page's objects by
   their start bits, which `space::words` sets; kc's code bumps `%r12` in
   line and sets none. A page kc fills is all kc's objects, one after
   another, each saying its size, so its starts are found by walking it
   (`kayak_heap::object_at`) when kc gives it back: at `kc_gc_call`, and
   before any collection while kc's frames are on the stack.
2. **The refill.** In process, `kc_collect`'s routine is kayak-rt's
   (registered in kcrt's routine table, which kcrt-core holds and
   kayak-rt fills: kcrt-core cannot name kayak-rt, which links it): it
   walks the page kc filled, takes a new one (`%r12` and `%r13` its start
   and end), and collects when kayak-rt's policy says.
3. **The roots.** kc's frames are on the thread's own stack, below the
   Rust frame that called `kc_call2`, so kayak-rt's conservative scan
   sees them; the globals' cells (`kc_cells_lo` to `kc_cells_hi`) are a
   kept block, registered at `kc_gc_init`. Exact roots (`Runs`) would let
   kc's objects move; ambiguous ones pin them, which is correct first.
4. **The barrier.** kc's two stores into the heap (`set`, `array-set!`)
   call `kc_remember` after the store: kayak-rt's `space::remember` in
   process, nothing in a built program.
