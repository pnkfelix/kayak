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
