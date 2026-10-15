# Fixpt's FX-26 execution tiers: a report for re-implementation

Source: `~/Dev/Rust/kayak-refs/fixpt` at `09acf16` (2026-10-14), read only.
Paths below are relative to that clone. "file:N" is a line I read; anything
marked **(inference)** I concluded rather than read. Where Fixpt's own docs
disagree with its code, I say so.

## 0. The shape in one paragraph

Fixpt has one portable code form, **cellular words** (Forth-style
token-threaded code stored as heap objects, called "threaded" elsewhere), and
a second, faster form, **register code** (Larceny's MacScheme machine: an
accumulator, 8 argument registers, explicit frames). Both are made from the
checked FX-26 tree by two compilers that each exist twice, in Rust and in
FX-26, and the two copies must produce identical output. The machines are:

- interpreters of cells: Rust, hand-encoded arm64, stencils;
- compilers of cells to arm64: Rust, FX-26;
- compilers of register code to arm64: one inside the cellular machine
  ("cellcc"), and one in a separate calling convention with native frames
  ("nativecc", `direct.rs`).

Every tier is checked against a slower one, and the slowest are the
lowering to Scheme and the FX-26 evaluator.

## 1. The machines

The machine names come from `crates/fixpt-cli/src/main.rs:510-526`
(`FX26_MACHINES`) and README.md:465-542. "Stage A" means source to
words/register code. "Stage B" means words or register code to arm64.

| machine | code it runs | stage A | stage B | what executes | `--fx26-run` / `--cellular-machine` / conv |
|---|---|---|---|---|---|
| `rs-scheme-vm` | FX-26 lowered to annotated Scheme (`crates/fixpt-fx26/src/lower.rs`, 963 lines) | none (Rust lowering) | none | Scheme bytecode VM (`fixpt-engine/src/vm.rs` 1263, `compile.rs` 581) | lower |
| `fx-ast-eval` | parser trees | none | none | the metacircular evaluator in FX-26 (`eval-core.fx` 537, `eval-prims.fx` 780, `eval-values.fx` 309, `eval-types.fx` 135) | evaluate |
| `fx-cells-rsinterp` | cellular words | FX-26 (`compile*.fx`) | none | Rust `match` interpreter (`fixpt-engine/src/cellular.rs`, 1700) | cellular / rust |
| `fx-cells-asminterp` | cellular words | FX-26 | none | "hand machine": arm64 routines emitted by Rust through its own encoder (`fixpt-native/src/cellular.rs`, 2321) | cellular / native |
| `fx-cells-stencils-cellcc` | cellular words | FX-26 | Rust stencils (nightly) | the same interpreter, routines written in Rust with `become`, copied in (`fixpt-native/src/stencil.rs` 272, `stencils/cellular.rs` 597, `build.rs` 255) | cellular / stencils |
| `fx-cells-rsjit-cellcc` | cellular words → arm64 | FX-26 | Rust `assemble_word` (`fixpt-native/src/cellular.rs:1027-1120`) | hand machine, word bodies inlined | cellular / native-compiled |
| `fx-cells-fxjit-cellcc` | cellular words → arm64 | FX-26 | FX-26 `native.fx` (815) over `arm64.fx` (178) | hand machine; only placing is Rust (`place_word`, cellular.rs:2305) | cellular / fx-compiled |
| `fx-regs-rsjit-cellcc` | register code → arm64 | FX-26 (`regcode*.fx`) | Rust `assemble_register_word` (`fixpt-native/src/cellular/regcode.rs`, 1484) | hand machine's state and stacks; register code calls by `blr` | cellular / registers |
| `fx-regs-rsjit-nativecc` (default, `fast`) | register code → arm64, native frames | FX-26 | Rust `direct.rs` (5552) + `direct/reps.rs` (643) | native frames on a stack inside the nursery; `bl`/`ret` | cellular / registers / native |

There is no `fx-regs-fxjit-*`: no FX-26 compiler takes register code to arm64
(README.md:507-508; TODO.md §81, :2214-2229).

Caveats on stage A:

- `fixpt bench` and the front end itself use the **Rust** stage A
  (`fixpt_fx26::cellular::Compiler`), and its columns are named `rs-…`
  (README.md:527-537).
- The front end (reader, checker, compilers written in FX-26) always runs as
  Rust-compiled register code on the hand machine. It is cached in
  `~/Library/Caches/fixpt/`.
- So under `fx-cells-fxjit-cellcc` the FX-26 arm64 compiler was itself
  compiled by Rust.

### How the tiers are checked against each other

- **Cell for cell, Rust stage A against FX-26 stage A:**
  `crates/fixpt-fx26/tests/rust_compiler.rs` (196 lines).
  `every_test_program_compiles_the_same` (:110),
  `every_test_program_has_the_same_register_code` (:130),
  `the_bootstrap_program_compiles_the_same` (:190), and
  `the_rust_compilers_words_run_as_lowered` (:152). `fixpt compile INPUT`
  compares both compilers too, and `fixpt check` compares both checkers.
- **Instruction for instruction, `native.fx` against `assemble_word`:**
  `crates/fixpt-fx26/tests/native.rs` (101 lines). It compares every word
  reachable from the bootstrap word (`reachable`, :16), the instructions and
  the per-cell start offsets alike (`compare`, :38;
  `every_word_of_the_front_end_as_the_rust_compiler_compiles_it`, :99).
  PLAN.md 11c records 817 words and 1.1 M instructions identical.
- **Encoder against encoder:** `crates/fixpt-fx26/tests/arm64.rs` compares
  `arm64.fx` with the Rust encoder `fixpt-native/src/arm64.rs`, refusals
  included (-1 in FX); PLAN.md 11b records 6,956 encodings. The Rust encoder
  is in turn checked against the system `as` (`fixpt-native/tests/arm64.rs`
  header).
- **Interpreters against the Rust machine on hand-built words:**
  `fixpt-native/tests/cellular.rs` (hand-encoded machine, and stencils at
  every opt level: same results and same traps) and
  `fixpt-native/tests/frames.rs` (frames, closures, calls, globals, prims).
- **Every program on every machine:** `crates/fixpt-fx26/tests/compiler.rs`,
  `every_program_on_every_machine` (:105-150), runs every test program
  through five machines and compares each with the lowered run:

  - native
  - native, words compiled
  - native, words compiled by `native.fx`
  - register code
  - stencils

  `compiler.rs:12` (`three`) also compares the FX evaluator, the lowering and
  cellular code.
- **Native convention against the Rust machine:**
  `crates/fixpt-fx26/tests/direct.rs` (1218 lines; for example
  `fib_and_tak_as_the_rust_machine_runs_them` :181, the GC-stress variants,
  `control_on_native_frames`).
- **Bootstrap fixpoints** (`crates/fixpt-fx26/tests/bootstrap.rs`):

  | test | what runs stage 2 |
  |---|---|
  | `fixpoint` (:100) | stage 1 (the FX compiler run lowered) compiles the front end; stage 2 runs on the hand machine; the stage-1 and stage-2 words must be `same_code` |
  | `fixpoint_with_words_compiled` (:108) | the same, words compiled by `assemble_word` |
  | `fixpoint_as_register_code` (:117) | Rust stage A with register code, run as register code |
  | `fixpoint_with_words_compiled_by_fx26` (:477) | words compiled by `native.fx`, itself compiled and running natively |

- **Generated tables must match the Rust tables:**
  - `layout.fx` (208 lines) is generated from `fixpt-heap/src/layout.rs`;
  - `native-layout.fx` (106 lines) is generated by
    `fixpt_native::cellular::fx26_module` (cellular.rs:1122-1190);
  - `tests/layout.rs` keeps both equal, and `FIXPT_BLESS=1` regenerates them.

### Measured speeds

The latest run table is in commit `b6120de` (2026-10-14). These columns use
Rust stage A and the old column names; ms, best of 3.

| program | scheme-vm | cells-rsinterp | asminterp | stencils | cells-rsjit | regs-rsjit-cellcc | regs-rsjit-nativecc |
|---|---:|---:|---:|---:|---:|---:|---:|
| fib (832040) | 122.9 | 135.3 | 17.1 | 23.4 | 13.8 | 6.2 | 2.8 |
| tak (22 16 8 → 9) | 41.5 | 49.9 | 6.8 | 8.9 | 4.8 | 1.8 | 1.0 |
| loop | 415.7 | 477.3 | 70.2 | 65.4 | 49.7 | 10.6 | 4.0 |
| closures | 249.9 | 387.5 | 74.8 | 74.4 | 60.4 | 15.6 | 20.7 |
| lists | 212.7 | 290.3 | 54.4 | 56.9 | 48.4 | 7.4 | 6.4 |
| captures | 95.7 | 28.0 | 10.7 | 11.2 | 9.7 | 7.4 | 11.7 |
| helpers | 433.0 | 649.2 | 96.3 | 92.8 | 63.6 | 5.9 | 2.0 |

Other figures from `docs/performance.md`:

- **FX-26 evaluator, whole process:** fib 1.4 s, tak 0.6 s (:258-268, after
  the 2026-10-08 rewrite).
- **Per cell (A′3/A′4, :176-233):** the Rust interpreter takes about 3 ns
  per cell, the hand machine about 0.55 ns. Stencils at -O1..-Os match the
  hand machine (fib27 0.57 hand against 0.62-0.65 stencils); at -O0 they
  take about 4.2 ns.
- **Cell code compiled to arm64 (C11a, :390-420):** no gain on the
  micro-benchmarks, because `NEXT`'s indirect branch predicts well.
  Bootstrap stage 2 went from 1.9 s to 1.4 s.
- **Compile cost (:2348-2410):** `native.fx` takes about 140× the time of
  `assemble_word` (979 ms against 7 ms on the front end). The FX-26 checker
  is about 3.5× the Rust one, and the FX-26 compiler about 11×.
- **Notable regressions:**
  - `closures` is slower under nativecc than cellcc, because deep recursion
    overflows the return-address predictor with `ret`. Returning by
    `br x30` fixes `closures` but costs fib/peek about 10% (:2810-2836).
  - `captures` is slower natively because a capture copies frames
    (TODO.md §52).

## 2. Cellular words

### Object format

A word is a bloblet of kind `cellular-code` (33). Its fields, addressed
backward from the suffix, are `[cell…][twin][name][entry][trailer]`
(`fixpt-heap/src/layout.rs:287-310`):

- `WORD_ENTRY` = 2: a fixnum routine number. `docol` = 0 means "run my
  cells". A compiled word gets a new number above the primitives, which
  indexes a native slot.
- `WORD_NAME` = 3: a symbol.
- `WORD_TWIN` = 4: the register-code word, or `#f`.
- Cell *i* is at field `WORD_CELL0 + i` = 5 + *i*. Cells run toward higher
  *k*, which is toward lower addresses, so the ip post-decrements by 8
  (cellular.rs:18-29).

The entry is a *number*, not an address, so heap images hold no machine
addresses (layout.rs:290-293).

A **cell** is either:

- a fixnum *n*, naming primitive routine *n* directly (token threading); or
- a pointer to another word, which runs that word's entry routine.

Operands follow their routine as raw cells (layout.rs:295-298). The operand
counts are in `operands()` (layout.rs:392-412):

- 1 operand: `lit`, `branch`, `0branch`, `slot`, `slot!`, `free`, `global`,
  `global!`, `call`, `tailcall`, `tcall`, `ttailcall`, `field`;
- 2 operands: `closure`, `prim`.

Words are made and validated in exactly one place,
`Heap::make_cellular_word` (`fixpt-heap/src/cellular.rs:27-102`), because the
native machines trust cells blindly. It checks:

- routine numbers are in range;
- branch targets land on an instruction start;
- `global` operands are global cells;
- `closure` takes a word and a count;
- `field` k ≥ 2;
- the last instruction does not fall through (one of `exit`, `halt`,
  `branch`, `tailcall`, `ttailcall`, `return`, `withmark-tail`, `resume`,
  `undefined`).

`Value::DEFAULT` in a cell means "this word itself", which is how a word
refers to itself. The word is frozen after it is built.

A branch offset counts from the cell after the operand:
target = i + 2 + off (cellular.rs:59).

### Instruction set

The routine table is `ROUTINES` (layout.rs:313-389), 63 entries (inference
from the list; `ROUTINE_SLOTS` = 64, state.rs:7). By group:

| group | routines |
|---|---|
| Forth core | `docol`, `exit`, `halt`, `lit`, `branch`, `0branch` (branch if `#f`), `execute`, `dup`, `drop`, `swap`, `over`, `+`, `-`, `<` (checked fixnums), `eq`, `field@`, `field!`, `cons`, `car`, `cdr` |
| FX-26 / MacScheme-on-a-stack | `slot i`, `slot! i`, `free i`, `global g`, `global! g`, `closure w n`, `call n`, `tailcall n`, `return`, `prim p n` (runtime primitive *p* on *n* stack values) |
| control | `prompt`, `abort`, `callcomp`, `callcc`, `withmark`, `firstmark`, `currentmarks`, `marksof`, `withmark-tail`, `resume`, `undefined` |
| typed calls | `tcall n`, `ttailcall n`: the checker proved the callee a closure, so there is no kind check and no stack-limit check in tail position |
| typed primitives | `int-add`, `int-sub`, `int-less`, `int-eq` (fixnum fast path, bignum otherwise), `pair-car`, `pair-cdr`, `field k`, `rest` (vargs) |

### Machine model

The machine model (`fixpt-engine/src/cellular.rs:1-21` and :382-412) has:

- a **data stack** of Values (`DS_LIMIT` = 2^17);
- a **return stack** of 4-Value entries `(word, fixnum 8k, fixnum fp,
  closure)` (`RS_LIMIT` = 2^16 entries);
- registers `cur`, `k`, `fp` (where the running frame's slot 0 is), and
  `clo` (the running closure, MacScheme's REG0).

Every stack entry is a Value, so both stacks are GC roots as they lie. The
fp is saved as `ds_base - 8 - fp`, which has the bits of a fixnum index
(state.rs:41-43).

How the language features map onto it:

- **Frames:** a call's arguments stay on the data stack and become the
  callee's slots 0..n-1. A `let` pushes onto the frame.
- **Calls:** `call n` pops the closure and pushes a return entry, setting
  FP = DSP + 8n - 8 (hand machine cellular.rs:967-972). `tailcall` slides
  the n arguments down over the current frame (:973-988). `return` pops the
  value, sets DSP = FP, stores the value, and pops a return entry
  (:1006-1015).
- **Closures** are flat: kind `cellular-closure` (38), laid out
  `[free…][word][trailer]`, with `CLOSURE_WORD` = 2 and `CLOSURE_FREE0` = 3
  (layout.rs:415-418). `closure w n` copies n stack values in.
- **Globals** are plain bloblets with `[value=2][name=3][writes=4]`
  (layout.rs:424-432). `global!` increments `writes`, which is what
  `global-guard` tests.
- **`letrec`** uses boxes, or since 13e no box: a pending slot patched once
  every sibling exists. Lambda lifting and loops are compiler-side.
- **Prompts** are two return entries: the resume entry, and a marker
  `(PROMPT_MARK, tag, handler, height)`. `PROMPT_MARK = Value::DEFAULT`,
  and the height packs the data-stack height (low 32 bits) with the count
  of live regions (high 32) (engine cellular.rs:345-375).
- **Marks** are one entry, `(MARK_MARK = UNSPECIFIED, key, value, 0)`
  (:377-379). Returns skip prompt and mark entries (hand machine
  `pop_return_of`, cellular.rs:448-480).
- **Continuations:** kind `cellular-continuation` (39), with fields `CONT_DS`,
  `CONT_RS` (vectors copied from the stacks), `CUR`, `K`, `FP`, `CLO`,
  `BASE`, `WHOLE`, `REGIONS`, `RUN`, `INNER` (layout.rs:434-461).
  - `callcc` copies everything; `callcomp` copies the stacks above the
    prompt.
  - Reinstating a delimited continuation pushes the copied stacks with frame
    pointers and prompt heights rebased by a delta; a whole one replaces the
    stacks (engine cellular.rs `reinstate`, about :690-730).
  - As a callable value, a continuation is wrapped in a closure whose word
    is `slot 0; free 0; resume` (`fixpt-heap/src/cellular.rs:112-122`).
- **Primitives:** `prim p n` calls runtime primitive *p* (the
  `fixpt_runtime::PRIMITIVES` table, `prim.rs`, 2666 lines) with *n* stack
  values.

### Traps

`Trap` (engine cellular.rs:49-130) has codes:

| code | trap |
|---|---|
| 1 | Type |
| 2 | Overflow |
| 3 | Underflow |
| 4 | Field |
| 5 | NoRoutine |
| 6 | NotAWord |
| 7 | OutOfFuel |
| 8 | StackOverflow |
| 9 | TooDeep |
| 10 | Prim |

The detail is the routine number. Fuel and limits are checked at word entry
and at taken branches (only backward ones in compiled words, cellular.rs:543).

## 3. Register code

### Format

Register code is a bloblet of kind `register-code` (40), laid out like a word,
`[cell…][twin][name][entry][trailer]`. Its twin is the cellular word it stands
for, and its entry is 0 until compiled (`fixpt-heap/src/regcode.rs:17-27`;
layout.rs:463-473). Cells are an op number followed by its operands. The
validator `check` (regcode.rs:34-112) requires:

- the code starts with `args n` or `vargs`;
- register operands are in 1..8;
- branch targets land on instruction starts;
- the last instruction is `return`, `tailinvoke` or `branch`.

The machine (layout.rs:466-473):

- `RESULT`, an accumulator;
- `REG0`, the running closure;
- `REG1`…`REG8` (`REGS` = 8), holding the arguments on entry; past 8, REG8
  holds a list of the rest (Larceny's convention);
- a frame of slots made by `save`.

Anything that may collect is an op marked "may collect", and only the frame
and RESULT survive it.

The ops (`OPS`, layout.rs:489-521), 35 in all:

- **Entry:** `args n`, `vargs`.
- **Moves:** `const x`, `global g`, `setglbl g`, `reg k`, `setreg k`,
  `movereg k1 k2`, `lexical i`.
- **Frames:** `save n`, `pop n`, `stack n`, `setstk n`, `load k n`,
  `store k n`.
- **Inline primitives:**
  - `op1 r`, `op2 r k`, `op2imm r x`, where *r* is a *cellular routine
    number* (reusing that table for `int-add`, `pair-car` and so on);
  - `field k`, `setfield k j`.
- **Call-outs:** `prim p n` (may collect), and `prim1 p`, `prim2 p k`,
  `prim2imm p x`, `prim3imm p x k` for primitives that never collect, so no
  spill is needed.
- **Closures:** `lambda w n`, which closes over a *cellular word*; the twin
  is found through the word.
- **Calls:** `invoke n`, `tailinvoke n`, `invokeself n`, `return`.
- **Branches:** `branch`, `branchf`, `brancht`.
- **Fallback:** `cellular r n` runs any cellular routine with REG1..n as its
  stack operands. This is how control (prompts, captures, marks) is
  reached.
- **Guard:** `global-guard g n off`, "unless global *g* has been written *n*
  times, skip *off*". It guards a fast version that inlined, specialized or
  folded a global (layout.rs:516).

### Relation to stack words: the "register twin"

Each lambda's cellular word is made first. Its register code, the *twin*, is
compiled again **from the tree, not from the cells**, and stored in
`WORD_TWIN`. A closure always points to the stack word; a machine that runs
register code takes the twin if there is one
(docs/research/compiler-middle-phase.md:9-27). Since the "twins as a phase"
step (same doc :214-238), a form's words are all made first, then all twins
(`Twin` enum, `fixpt-fx26/src/cellular.rs:96-100`; `compile-twins.fx`, 90
lines).

### Register allocation

There is no general register allocator. It is a fixed policy
(`fixpt-fx26/src/cellular/regcode.rs:1-20`; performance.md:619-640,
1431-1446):

- **A leaf** (no calls or call-outs, loops aside; tail calls done by
  shuffling the arguments, `r_leaf_tail_call`) keeps parameters, `let`s and
  temporaries in REG1..8.
- **A non-leaf** keeps parameters and `let`s in frame slots made on entry.
  Registers hold only temporaries and outgoing arguments.
- **Exception, `r_in_regs`:** a value bound where nothing after it calls
  stays in a register, using at most half the registers.
- **Join points** (`r_join_ok`): their parameters go in registers in a leaf,
  else in slots.
- **Constants** bound to a name occupy no place at all (`RLoc::Const`).
- The place kinds are `RLoc` (regcode.rs:35-55): `Reg`, `Slot`, `Free`,
  `Global`, `Loop`, `Pending`, `Const`, `Join`, `Lifted`, `Test`.
- A peephole pass over frame traffic was proposed and not built (TODO §24).

### cellcc: register code inside the cellular machine

`fixpt-native/src/cellular/regcode.rs` (1484 lines) compiles register code
onto the hand machine's state (header :1-21):

- `RESULT` = x0, REG1..8 = x1..x8, REG0 = CLO (x27).
- `CUR` is the register word, and `IP` points at the word's last field so
  that constants and global cells are one load away.
- A call to a callee with compiled register code is a `blr`, with a
  *marked* return entry pushed (`8k` negated) so that captures still see the
  frame. The callee's `return` sees the mark and `ret`s with the value in x0
  (performance.md:864-917).
- Stack-code callers enter through the word's own entry, which moves stack
  arguments into registers.
- Frames live on the data stack, so the stacks remain the GC roots. After a
  call-out the registers are reloaded from `State`.
- Fuel polls in register code refill time slices inline (cellular.rs:521-533).

### nativecc: `direct.rs`

`direct.rs` (header :1-44 is partly stale, see §8) compiles each procedure's
register code to code in its own convention (direct.rs:7-25;
docs/research/native-conventions.md):

| what | where |
|---|---|
| arguments | `x1`–`x8` (REG1..8) |
| result | `x0` |
| `vargs` count | `x9` |
| closure called | `x10` |
| return address | `x30`, saved by procedures with frames |
| frame link | `x29` |
| state (`DState`) | `x24`, pinned |
| fuel | `x28`, pinned |
| stack limit | `x27`, pinned |
| code-area delta (RW→RX) | `x26`, pinned |

How it differs from cellcc:

- There is no ip, no data stack, no return entries and no resume tables.
  Calls are `bl`/`blr` and returns are `ret`.
- **Frames** are laid out as heap bloblets, so they can be flushed into the
  heap in place (direct.rs:563-586):
  - `[x29,#-8]` is room for a header;
  - `[x29]` is the link;
  - `[x29,#8]` is the return address;
  - `[x29,#16]` is the **stack map**, a fixnum holding a 48-bit mask of live
    value slots plus the field count above bit 48 (`MAP_BITS`; negative
    means "all values");
  - then the value slots, then the code bloblet and closure;
  - then raw `f64` slots, which form the bloblet's untraced suffix.

  The mask comes from a backward liveness pass and is stored before each
  call or call-out where it changes (performance.md:2002-2036).
- **Leaves** check nothing: `(lambda ((x int)) x)` is `mov x0,x1; ret`. A
  procedure with a frame checks the stack limit and fuel on entry; loops
  check fuel on back edges.
- **Each procedure is a code bloblet in the heap's code area.** Its fields
  (`Field` enum, :413-436) hold, read PC-relatively:
  - itself;
  - the source word;
  - callees' code;
  - heap constants;
  - global cells;
  - closures of globals' procedures.

  An unknown call loads the code from the closure's field 2, adds the delta,
  and traps if the result is not in the code area. A native closure is kind
  `native-closure` (41), `[free…][code][trailer]`.
- **Raw representations** come from `reps.rs`: a forward pass keeps `i64`,
  `u64` and `f64` raw in registers, boxing at any value use. There is also
  a **fixnum version** of each procedure's code with fallover to the general
  version at the same instruction (performance.md:2269-2290).
- **Inline paths:** pairs, sums, products and mutable bloblets are
  bump-allocated inline (direct.rs:2580-2640). Fixnum arithmetic and
  `field@` are inline too.
- **Declines.** When a procedure declines, its callers decline too, and the
  code runs as cellular words. The reasons (`decline`, direct.rs:1439 and
  following):
  - the procedure has no register code (:1336);
  - a frame or closure is too large (frame > 4080 bytes, :1501);
  - an unsupported `op1`/`op2` routine (:1934, :1979);
  - a field too far (:2073);
  - a call-out outside a frame (:2102);
  - an op on a primitive that may collect in an unsupported place
    (:2246, :2428);
  - it calls `%stay-cellular` (:2486), which is deliberately always
    declined so tests can force mixing;
  - unknown primitives or routines (:2490-2493);
  - any other op (:2914).
- **Calls between conventions:**
  - Native code calls non-native closures through `common_foreign`, then
    `Callout::Foreign`, then `call_foreign` (direct.rs:4621-4660). That runs
    the closure on a **fresh Rust cellular interpreter**,
    `fixpt_engine::cellular::call_value` (engine cellular.rs:1525-1555), with
    the native frames' values copied to a rooted vector meanwhile.
  - Cellular code calls a native closure through `call_native`
    (direct.rs:536-557).
  - Convention adapters are four instructions into `common_foreign`
    (direct.rs:505-534), or the cellular word `free 0; tailcall n`.

  The checker infers conventions as a kind `conv`
  (`cellular | native | fx`, `cellular ≤ fx`, `native ≤ fx`) and inserts
  conversions (native-conventions.md:52-115).

## 4. The hand machine (`fx-cells-asminterp`)

The hand machine is **not a hand-written `.s` file**. It is arm64 emitted at
start-up by Rust code (`generate()`, `fixpt-native/src/cellular.rs:606-660`;
`routine_body`, :663-1020) through Fixpt's own encoder
(`fixpt-native/src/arm64.rs`, 580 lines, checked against `as`). The CLI calls
it "hand-written arm64" (main.rs:76) and the docs call it "hand-encoded".

Register roles (cellular.rs:69-89). All machine registers are callee-saved,
so they survive calls into Rust.

| register | role |
|---|---|
| x19 BASE | heap base; 0 since "Values as addresses" |
| x20 IP | the ip, as an address |
| x21 CUR | current word |
| x22 DSP | data stack pointer |
| x23 RSP | return stack pointer |
| x24 ST | `*State` |
| x25 TABLE | routine address table |
| x26 FP | frame pointer |
| x27 CLO | running closure |
| x28 FUEL | fuel |
| x30 | link for register code |
| x9-x16 | scratch |

**Dispatch** is token threading, with each routine ending in its own inlined
`NEXT` (cellular.rs:18-29, :344-351):

```
ldr  x9, [ip], #-8      ; cell
tst  x9, #7             ; fixnum => primitive
b.ne 1f
ldr  x10, [table, x9]   ; n<<3 is the byte offset
br   x10
1: add x11, base, x9 ; ldur x10, [x11, #-20] ; ldr x10, [table, x10] ; br x10
```

Routines start 16-aligned. The entry stub saves x19-x30 and loads the
registers from `State` (:606-635). The common trap stores status and aux, and
the common exit saves the registers and returns (:637-650).

**`State`** (`fixpt-native/src/state.rs`, 82 lines) is `#[repr(C)]` and is
shared by offset with both the generated code and the stencils, which
`include!` the same file. It holds:

- the saved registers, with the ip saved as `d` = 8k;
- the stack bases and limits;
- the `callout` function pointer;
- `resume` (per-word resume tables);
- `top`, `alloc_limit` and `words` for inline allocation;
- `cards` (card table) and `regions`;
- `code_lo`;
- `routines[64]`;
- `trap` and `exit` addresses;
- `leaf_link`;
- `pure`;
- `reserve` (fuel not yet given out).

Routines without machine code (control, closures, `cons`, and others) go to
`callout(n)`: save, `blr st.callout`, then reload. Those in
`CONTROL_CALLOUTS` (cellular.rs:1112-1117) re-enter wherever `CUR`/`IP` now
point (`enter_cur`, :329-342). The Rust side (:1198-1560) either runs the
routine directly (`prim`, `cons`, `field@`) or does a "round trip": it lifts
both stacks into a Rust `Machine` for one routine and puts them back
(PLAN.md 9c).

**Compiled words** (`assemble_word`) are a word's routines inlined in cell
order, with the ip kept in step:

- branches become jumps, and a taken branch rebuilds the ip from the word
  (`branch_ip`);
- the word's code is position-independent, reaching the trap and exit only
  through `State`;
- returns into it go through a per-word resume table indexed by `8k`
  (cellular.rs:1-16, `enter_cur`).

Word code lives in a `CodeSpace` that is collected by copying
(object-model.md, "The cellular machines' code, collected by copying").

## 5. Stencils

Stencils are produced by `fixpt-native/build.rs` (255 lines):

1. It compiles `stencils/cellular.rs` with `rustup run nightly rustc --emit
   obj --target aarch64-apple-darwin`, with `-C panic=abort`,
   `debuginfo=0`, `-enable-machine-outliner=never`,
   `debug-assertions=off` and `overflow-checks=off`, once per opt level
   0/1/2/3/s (:20, :118-145).
2. It generates `consts.rs` from the layout table (tags, kinds, routine
   numbers, trap codes; :38-74).
3. It parses the Mach-O by hand: header, `LC_SEGMENT_64` sections, symtab
   (:170-215).
4. It takes each `_st_*` symbol's bytes up to the next symbol.
5. It emits `stencils.rs` as `STENCIL_SETS`: per opt level,
   `(name, &[u8])`. With no nightly there are no stencils, only a warning.

**There are no holes and no patching in the built machine.** The build
**fails on any relocation at all** (:216-223), and on any branch that leaves
the stencil (`check_branches`, :233-255). Placing a stencil is a copy
(`StencilMachine::new`, stencil.rs). This works because:

- every routine has the same signature,
  `extern "C" fn(base, ip, cur, dsp, rsp, *mut State, fp, cell) -> u64`,
  so the machine lives in the 8 argument registers;
- every routine ends in `become` (nightly `explicit_tail_calls`), which is a
  guaranteed tail call even at -O0;
- `NEXT` is inlined, as a macro;
- the routine table is in `State.routines`;
- constants are immediates from `consts.rs`;
- constant stores are made volatile, so LLVM does not merge them into a
  constant-pool vector load (stencils/cellular.rs:1-17, :100-115).

A routine with no stencil uses `st_other`, the call-out round trip.

The Xu–Kjolstad style, with holes for operands, continuations and `GOT`
relaxation, was prototyped only in a scratch experiment
(docs/research/copy-and-patch.md §4). The README and CLI call this machine
"copy-and-patch", but in the built machine the "patch" step is empty: it is
a copied *interpreter*, not a template JIT.

## 6. The runtime contract for native code

### Value representation

64-bit words with a low 3-bit tag (`fixpt-heap/src/value.rs:8-17`,
layout.rs:26-35):

| tag | meaning |
|---|---|
| 000 | fixnum, 61-bit |
| 001 | pair (a 2-word car/cdr cell with no header) |
| 010 | suffix ref (34-bit owner index + 27-bit byte offset) |
| 011 | immediate (subtag in bits 3..8: `#f`, `#t`, `()`, unit, eof, unspecified, default, unbound, char, f32) |
| 100 | bloblet, pointing at the *start of the suffix* |
| 101 | trailer |
| 110 | header, never a value |
| 111 | forward |

A bloblet is `[header][F tagged fields][B bytes of suffix]`, and the fields
are addressed negatively from the suffix (field k at `suffix - 4 - 8k`
relative to the tagged value; cellular.rs:97-101). The header bits are tag:3,
kind:8, large:1, fields-frozen:1, suffix-frozen:1, F:18, B:32
(layout.rs:77-86). A trailer word stores F so that a value can find its
header (`ldur [v,#-12]`, then subtract).

Values are now **addresses** (performance.md:835-862). The `value.rs`
module comment still says "word offset into the active semispace", which is
stale. Fixnum add with overflow check is `adds` on tagged values.

### Heap allocation from native code

Allocation is an inline bump of `heap.top` (a word index, read through
`st.top`) against `st.alloc_limit`. The address is `st.words + 8*top`. On
failure, the code calls out.

- `direct.rs` does this for pairs, `%make-bloblet`, `%make-frozen` and
  flonums (:2580-2640, :3239, :3390).
- Cellcc register code does it with `bump_words` (cellular/regcode.rs:324),
  including region (`rcons`) allocation through `st.regions`.

Every call-out refreshes `words` and `alloc_limit` (direct.rs:4609-4612).

### GC root finding

- **Cellular machines:** the data stack and return stack are all Values,
  and the collector scans them as slices at call-outs. The hand machine
  saves its registers into `State` first and reloads them after, since a
  collection moves things. Register code (cellcc) keeps live values in the
  data-stack frame across anything that may collect.
- **nativecc:** **safepoints only at calls and call-outs**; registers hold
  no Value across one. Frames carry the mask stack map described in §3.
  - When a collection is due, the native stack is **flushed in place into
    the nursery** as `native-frame` bloblets (kind 45). The native stack *is*
    the top of the nursery range. The innermost frame is restored after. So
    the collector reads no stack (Larceny's stack cache: direct.rs:4466-4481;
    performance.md:2650-2715).
  - Overflow likewise flushes the stack to a heap chain (`common_overflow`).
    Underflow restores up to 256 words (`common_underflow`).
  - `frame_shape` (direct.rs:4955-4972) and `native_frames` (:5490-5503)
    read the maps.
- **Code liveness:** each frame stores its own code bloblet in a slot, which
  keeps the code alive (no return-address index). The code area is
  mark-swept inside the copying collection (`fixpt-heap/src/heap/code.rs`;
  object-model.md:388-462).

### Write barrier

Card marking, unconditional: 6 instructions, a byte per 512-byte card
(`card_mark`, `fixpt-native/src/arm64.rs:299-309`: `add/sub t1,obj,off;
lsr t1,t1,#9; ldr t2,[st,#cards]; add t2,t2,t1; mov t1,#1; strb t1,[t2]`). It
is emitted after `field!`, `global!`, `setfield` and `setglbl` in every
machine. The nursery is 2^20 words (performance.md:2037-2050).

### Executable memory

The PLAN said `MAP_JIT` (A1). What was **built** is **double mapping, no
`MAP_JIT`, no W^X toggling**:

1. `mmap` a `MAP_SHARED|MAP_ANON` RW region;
2. `mach_vm_remap` a second view;
3. `mprotect` that view RX;
4. flush with `sys_icache_invalidate`.

This is in two places:

- `fixpt-native/src/codespace.rs` (the `CodeSpace` for cellular word code,
  :56-110, :189);
- `fixpt-memmgmt/src/exec.rs` (`ExecView` over the heap's code area,
  :1-11, :84-134).

The measured rationale (object-model.md:306-345): `MAP_JIT` toggling costs
32 ns per field write, while double mapping lets code bloblet fields be
written at full speed. Native code reaches the RX address through `DELTA`
(x26).

### Calling back into the runtime

- **Hand machine:** `State.callout`, an `extern "C" fn callout(st, n)`
  (cellular.rs:1202). Its nonzero return means a trap. Separately,
  `State.pure`/`pure_call` (:1235) is for never-collecting primitives with
  no safepoint.
- **nativecc:** `DState.callout` with an index into a per-machine table of
  `Callout` (direct.rs:328-396):
  - `Cons`, `Prim{p,n}`, `Pure`, `Closure`, `ClosureAny`, `RegionClosure`,
    `FieldRef`;
  - `Foreign`, `Abort`, `Capture{whole}`, `Reinstate`, `FirstMark`,
    `CurrentMarks`, `MarksOf`, `Rest`;
  - `Box`, `Unbox`, `BoxF64`;
  - `Underflow`, `Overflow`, `Poll`, `PollDone`, `Failed`.

  Arguments are in `st.args[8]`. The handler is direct.rs:4443-4613. It
  collects first if a collection is due and the call-out is not "quiet".
- **Errors:** traps branch to stubs, then the common trap. The trap code is
  stored (`TRAPS` = step limit, stack overflow, a primitive failed, car of
  a non-pair, division by zero; direct.rs:216). A primitive's message goes
  through the thread-local `LAST_MESSAGE`. `Failed` lets a monitored
  process's failure become an abort to the scheduler's prompt.
- **Entry from Rust:** a trampoline (direct.rs:1031-1070) saves the
  callee-saved registers, switches SP to the native stack, loads the pinned
  registers and arguments from `DState`, and `blr`s.

### Preemption and fuel

Fuel is a register (x28) decremented at polls:

- cellular machines: word entry and taken branches;
- nativecc: procedure entry and back edges, leaves never.

Fuel is handed out in time slices (`sched::slice`), with the remainder in
`reserve`. At slice end the cellular machines leave to Rust and resume from
`State`. Register code and native code take the next slice inline, calling
`Callout::Poll`, and call the scheduler's `preempt` closure as an ordinary
call if another task should run. A native poll's frame (208 bytes) maps the
live registers (direct.rs:216-318). Machines run out of fuel only "within a
bounded distance" of one another (native-conventions.md:179-187).

### Where `unsafe` lives

The workspace sets `unsafe_code = "deny"` (Cargo.toml:27). The exceptions:

- `crates/fixpt-native` (`#![allow(unsafe_code)]`, src/lib.rs:13): the
  encoder output, code spaces, machine entry, and stack walking;
- `crates/fixpt-memmgmt` (347 lines): reservation, words, the exec view;
- `crates/fixpt-runtime/src/tty.rs:5`: terminal modes;
- the stencil source, which is a separate nightly `no_std` crate compiled
  by `build.rs`, and is all `unsafe extern "C"`;
- some `fixpt-native` tests.

The heap, runtime and engine crates are otherwise safe.

## 7. The compiler pipeline, checked FX-26 to cells and register code

The same passes exist in Rust (`fixpt-fx26/src/cellular.rs` 2589,
`cellular/procs.rs` 511, `cellular/regcode.rs` 3134) and in FX-26 (loaded in
this order by `conductor.fx:255-337`). The order:

1. **Checker facts** (types, effects, constants, conversions, `extract`
   field orders) are recorded as tables keyed by span (`c-set-facts!`).
   Compilation is of the checker's tree; there is no separate IR.
2. **`compile.fx`** (795 + 376 types) is the stack compiler's core: resolve
   each variable to slot, free, global, or a `letrec` box/pending slot.
   `prompt` bodies become thunks.
3. **`compile-lift.fx`** (558 + 151): lambda lifting of `letrec` groups whose
   members are only called (Twobit pass2's bound: fewer than 6 added names,
   at most 8 arguments), and the standard operations.
4. **`compile-state.fx`** (450 + 179), **`compile-exps.fx`** (570 + 107):
   expressions. `depth` tracks frame slots. Tail position ends in
   `tailcall`/`return`, and self tail calls become loops (13c).
5. **`compile-plan.fx`** (591 + 118) / Rust `procs.rs` is the "middle phase"
   (docs/research/compiler-middle-phase.md): before each top-level form, a
   walk decides each lambda's captures, each `letrec`'s lifting, and each
   call's inline or specialize decision, including nested contexts for
   inlined bodies and specialized copies.
6. **The register compiler:**
   - `regcode.fx` (828 + 313 types) is the entry and drivers;
   - `regcode-exps.fx` (691 + 234) handles expressions and the may-collect
     analysis;
   - `regcode-places.fx` (249 + 208) handles places and environments;
   - `regcode-helpers.fx` (547 + 235) holds helpers;
   - `regcode-modules.fx` (126 + 81) handles first-class modules;
   - `regcode-core.fx` (1130 + 25) is one recursive group of 50 members,
     over the 1000-line limit by decision;
   - `regcode-entry.fx` (303 + 45) handles specialized temporaries and the
     entry.
7. **`compile-twins.fx`** (90 + 17): all of a form's twins after all its
   words.
8. **`compile-inline.fx`** (358 + 22): records which globals may be inlined
   or specialized after each definition.
9. **`compile-programs.fx`** (551 + 26): the program and REPL driver.
10. **`native.fx`** (815; with `native-types.fx` 30, `native-layout.fx` 106,
    `arm64.fx` 178, `arm64-types.fx` 55): stage B for cells only.

The optimizations built (all in both compilers, word-identical), from
performance.md:

- typed calls and typed primitives (13c′/13d);
- self tail calls as loops, and `letrec` without boxes (13e);
- guarded inlining of small global procedures, at most 20 tree nodes
  (:1295);
- specialization of a recursive higher-order procedure at a lambda
  argument, `map1@lambda@N` (:1351);
- constant propagation and folding through inlining (:1462);
- self-calls through the global, guarded, as loops or `invokeself` (:1499);
- join points (:1578);
- "versions": all guards at body start, then a fast body and a plain body
  (:1654);
- a single `global-guard` on the write count (:2492);
- lambda lifting (:1855);
- an applied lambda as a `let` (:1896);
- tests as jumps (:1765);
- `case` by halving (:2600);
- temporaries in registers (:1431);
- a leaf that only makes a closure (:1913).

**Twobit (docs/research/twobit.md) comparison:**

| Twobit pass | in Fixpt |
|---|---|
| pass2 lambda lifting (with its 6-name bound) | taken |
| let-conversion and known procedures | taken (applied lambda as `let`, join points, known self-calls) |
| assignments to cells | moot, FX-26 has no variable assignment |
| pass3 inlining | taken, guarded |
| constant propagation and folding | taken |
| representation specialization of primitives | taken (typed primitives, `reps.rs`, fixnum versions) |
| pass4-style lazy frames / frame elision | partly (leaves have no frame; no frame elision on non-calling paths) |
| known calls | taken (`invokeself`, `bl` to known code bloblets) |
| A-normal form, commoning (CSE), copy propagation, dead code | **not** taken; CSE and loop lifting were "measured, not built" (performance.md:1403, :1534) |
| parallel argument assignment | not taken (leaf tail-call shuffles only) |
| peephole superinstructions (13g) | not built |
| frame-traffic peephole | proposed only (TODO §24) |
| integrated `.check!` style | not used; checks are in the ops |

## 8. What is still Rust-only, and known weaknesses

**Rust-only pieces:**

- register code to arm64, both `cellular/regcode.rs` (1484) and
  `direct.rs` + `reps.rs` (6195) (TODO §81);
- the hand machine's routines (`generate`);
- the stencil build;
- code placement and code spaces;
- the whole runtime (primitives, collector, scheduler, control crossing);
- the lowering to Scheme and the Scheme VM;
- the front end's own compilation (always Rust stage A, and Rust register
  code to arm64).

The FX-26 side covers stage A completely (cells and register code) and stage
B for cells only.

**Weaknesses and TODOs:**

- **Return prediction:** `ret` mispredicts after deep recursion; `br x30`
  hurts shallow calls (TODO §52; performance.md:2810).
- **Captures copy frames eagerly**, so natively they are slower than cellcc
  (TODO §52). The proposed fix is lazy or one-shot capture (§91).
- **Native frames as bloblets** made deep recursion with GC slower (`merge`
  2.91 → 3.34 s; TODO §91). Raw `i64`/`u64` slots are not done; `f64`s pass
  raw only to known callees; there is no FP register allocation.
- **No real register allocator.** Non-leaf parameters live in slots, and
  nothing survives a call in a register (TODO §91, §24).
- **Images** refuse heaps containing native code. Return addresses are raw
  (TODO §52).
- **Cross-machine control gaps** (TODO §93): a `callcc` continuation thrown
  past native code is an error; polls in nested cellular code are limited.
- **Nested native runs** get little stack room (TODO §52). About 680 control
  frames in a row can overflow.
- **`native.fx` is about 140× slower than `assemble_word`.**

Stale or contradicting docs, worth knowing before trusting a doc line:

- **The default machine:** TODO §81 calls `fx-regs-rsjit-cellcc` the
  default, while README.md:483 and main.rs:509 say `-nativecc`. TODO §81
  also estimates direct.rs at "about 4,900 lines"; it is 6,195 with reps.
- **`direct.rs`'s module doc** (:33-44) says only first-order procedures,
  no closures, no stack maps, and globals bound at compile time. All of that
  is superseded by steps 3-5 (native-conventions.md:245-346).
- **The convention table:** native-conventions.md:131 says arguments in
  x0–x7; the build uses x1–x8 with the result in x0.
- **`value.rs`'s module comment** says values are word offsets; values are
  addresses now.
- **The hand machine** is called "hand-written arm64" but is
  Rust-generated.
- **The stencil machine** is called copy-and-patch but has no patching.
- **PLAN A1** says `MAP_JIT`; the build is double-mapped.

## 9. Sizes (lines, `wc -l`)

Rust:

| component | files | lines |
|---|---|---:|
| stage A, stack compiler | fixpt-fx26/src/cellular.rs | 2589 |
| stage A, middle phase | cellular/procs.rs | 511 |
| stage A, register compiler | cellular/regcode.rs | 3134 |
| lowering to Scheme | lower.rs | 963 |
| Rust cellular interpreter | fixpt-engine/src/cellular.rs | 1700 |
| Scheme VM (+ its compiler) | vm.rs + compile.rs | 1263 + 581 |
| hand machine + `assemble_word` | fixpt-native/src/cellular.rs | 2321 |
| register code → arm64 (cellcc) | fixpt-native/src/cellular/regcode.rs | 1484 |
| nativecc | direct.rs + direct/reps.rs | 5552 + 643 |
| arm64 encoder + disassembler | arm64.rs + arm64/disasm.rs | 580 + 133 |
| stencils | stencils/cellular.rs + stencil.rs + build.rs | 597 + 272 + 255 |
| code space; state; control round trip | codespace.rs; state.rs; control.rs | 252; 82; 440 |
| faults, symbols, lib | faults.rs, symbols.rs, lib.rs | 61 + 49 + 24 |
| **fixpt-native total, tests excluded** | src 11,893 + stencils 597 + build.rs 255 | **12,745** |
| memmgmt (exec views, reservation) | fixpt-memmgmt/src | 347 |
| heap: words, register-code checker, layout table | heap/cellular.rs, heap/regcode.rs, layout.rs | 175, 112, 686 |
| heap core, code area | heap.rs, heap/code.rs, young.rs, regions.rs | 2572, 181, 301, 406 |
| runtime primitives, scheduler, crossing | prim.rs, sched.rs, crossing.rs | 2666, 970, 108 |

FX-26 (all in `crates/fixpt-fx26/src`):

| component | lines (all files incl. `-types.fx`) | without `-types` |
|---|---:|---:|
| stack compiler, `compile*.fx` (9 files + types) | 4959 | 3963 |
| register compiler, `regcode*.fx` (7 files + types) | 5015 | 3874 |
| stage B: `native.fx`, `native-types.fx`, `native-layout.fx` (generated), `arm64.fx`, `arm64-types.fx` | 1250 | 815 + 178 core |
| evaluator `eval-*.fx` | 1761 | |
| `layout.fx` (generated) | 208 | |

Main tests: `fixpt-fx26/tests/`:

- native.rs 101
- rust_compiler.rs 196
- compiler.rs 150
- register_code.rs 535
- direct.rs 1218
- arm64.rs 152
- bootstrap.rs 801

`fixpt-native/tests`: about 780 lines.

**Notes for kayak (inference):** Fixpt's cheapest parity guarantees came from
(a) one table, `layout.rs`, generating the FX-side constants, (b) one word
validator that every producer goes through, and (c) oracle tests that compare
instruction streams, not just results. The stencil tier has no holes, so it
adds little over the hand machine besides proving that Rust-with-`become` can
express the interpreter, and it needs nightly. The performance tiers that
mattered are register code and nativecc. Their unsafe surface is concentrated
in `direct.rs`'s runtime side (frame walking, flush/restore, trampolines)
and the double-mapped code area.
