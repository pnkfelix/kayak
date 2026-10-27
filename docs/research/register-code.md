# Register code from kayak's own compiler

Status: stage 1 done (C248); the user: "Yes I've been waiting days (weeks?) for
register code in Kayak"; and "If you could try to make your own
architecture more modular compared to Fixpt, it would probably yield big
dividends".

## Why

Tiers 5 and 6 run register code: a lambda's word's *twin*, MacScheme
machine instructions over registers and a frame. Only Fixpt's listings
have it; kayak's compiler to cells (`fx26/cg.kfx`) makes stack code
alone. So a program kayak compiles stops at tier 4, every tier 6 number
so far is of Fixpt's listings, and tier 6's code has no source lines
(C246). Register code from kayak's compiler is what makes kayak's
pipeline whole, and what the call-protocol work should be measured on.

## What Fixpt does

`crates/fixpt-fx26/src/regcode*.fx` (5,100 lines of FX-26, as its Rust
compiler's `cellular/regcode.rs`), called per top-level form after its
stack words (`compile-twins.fx`):

- **Entry** (`regcode-entry.fx`): a lambda is a *leaf* (its parameters,
  `let`s and temporaries in registers, no frame) if nothing in it calls
  or calls out, plain tail calls aside; else its parameters are stored to
  a frame made on entry. Where sound (no global can change during a run
  of the body; it makes no closure) and worth it (a leaf, or it loops), a
  *fast* version assumes every global it inlines, folds or calls itself
  through still holds what it held: one `global-guard` per assumption at
  the start, falling to the *plain* version where one fails. `tak`: the
  fast version's self tail call a loop (`branch → 17`), its other self
  calls `invokeself`.
- **Expressions** (`regcode-core.fx`): values into RESULT; `if` as
  branches on tests (`and`/`or`/`not` without booleans; a test decided
  by an `if` around is known in its arms); `let` values in registers
  where no call follows, else frame slots, at most half the registers;
  standard operations in line (`op1`, `op2`, `op2imm`, `prim1/2`, …);
  calls with arguments into REG1…REGn, hard ones kept in the frame first;
  a leaf's tail call by parallel register moves; `letrec` closures with
  patches, or join points; `tagcase`; closures (`lambda`) of the words
  the stack code made.
- **Declines**: whatever it does not do, it declines, and the lambda keeps
  its stack code alone. Every path that cannot be made says so.
- **Depends on** Fixpt's middle phase, which kayak's compiler has not:
  lambda lifting (`rl-lifted`), plans, inlining (`r-inline`),
  specialization (`r-specialize`, copies), unrolling, constant globals.

## How kayak does it

**Fixpt's output is the oracle, not its structure.** Each register word
kayak makes is compared with Fixpt's for the same program; where kayak's
stack words are Fixpt's exactly (292 of 402 programs), its register code
should be too. What depends on what kayak's compiler has not yet is
declined, so the lambda keeps its stack code: never wrong, only slower.

**Layers**, each depending only on those above it:

1. `kfx/regs-isa.kfx`, shared by the engine and tier 1: the instruction
   set, numbered by Fixpt's `fx26:layout.fx`, with kayak's names, operand
   counts, conventions and text form. Tier 1's reader (`cells/listing.kfx`,
   `rop-names` in `cells/layout.kfx`) and the compiler's writer use the
   one definition.
2. `fx26/regs-asm.kfx`: an assembler. Items (instructions with operands,
   labels, branches, guards, the frame's size), labels resolved, the
   listing's lines written. Knows nothing of FX-26.
3. `fx26/regs-moves.kfx`: places and parallel register moves (the order
   that clobbers nothing, a cycle broken through RESULT). Pure.
4. `fx26/regs-know.kfx`: analyses of an expression: whether it may
   collect (calls or calls out), what is known of it (constants, folding,
   tests an `if` decided), the standard operations' table (what each is in
   register code). Pure, given an explicit context.
5. `fx26/regs-gen.kfx`: the expressions' compiler. Its state an explicit
   context (whether a fast version, what it assumes, the procedure's own
   global, whether it looped, whether declined), passed along, not
   globals set and restored around calls as Fixpt's are. Each feature not
   ported (inlining, specialization, lifting, unrolling, modules) one
   decline, in one place.
6. `fx26/regs-entry.kfx`: a lambda's register code: the leaf decision,
   the plain and fast versions and their guards. Its interface to the
   stack compiler: a *twin* (the word, its parameters, body and scope,
   what it knows of itself, its definition's global) in; its listing
   text, or none, out.

The stack compiler (`cg.kfx`) notes each lambda's twin as it makes its
word, and counts each global's writes (`global!`), which a guard expects
(`written N`); its form loop, split off with the listing writer into
`fx26/cg-top.kfx` after the layers, makes each form's twins after its
words, as Fixpt does; the listing writes each word's register code after
its cells.

## Stages

1. **The core.** Layers 1-6 for: variables, constants, `if` and tests,
   `begin`, `let`, standard operations, calls, self calls (`invokeself`),
   loops, a top-level procedure's self calls through its global (guarded),
   fast and plain versions, leaves and their tail calls. Declined:
   closures made, `letrec`, `tagcase`, modules, regions, prompts, marks,
   `apply`, `list`, bloblets, conversions, more than `register-regs`
   arguments. Benchmarks first: `tak`, `fib`, `loop`.
2. Closures, `letrec` (patches, join points), `tagcase`, `apply`, `list`,
   the call-outs.
3. The rest kayak's stack compiler has: modules, regions, prompts and
   marks, bloblets, conversions.
4. What needs Fixpt's middle phase, as kayak's compiler gains it.

## How it is checked

- `cells-programs --ours`: kayak's listings run with `--regs`,
  `--compiled --regs` and `--direct` too (new ways, so a change to the
  check, agreed with this plan), each answer Fixpt's.
- Each register word compared with Fixpt's: of those kayak makes, how many
  are Fixpt's exactly, how many differ (each difference understood: a
  decline of kayak's is no difference, a word kayak makes that Fixpt does
  not, or makes otherwise, is), as `292 of them Fixpt's words exactly`
  counts stack words.
- Tier 6 measured on kayak's listings (`tak 30 21 12`, `lists`) against
  the same on Fixpt's.
- Register code's line tables, as stack code's (C245), for tiers 5-6.

## Stage 1, as done (C248)

- **Where it is**: `kfx/regs-isa.kfx` (shared, tier 1's reader on it too),
  `fx26/regs-asm.kfx`, `regs-moves.kfx`, `regs-know.kfx`, `regs-gen.kfx`,
  `regs-entry.kfx`; `cg.kfx` notes each lambda's twin and where the
  checker's facts of it are; `cg-top.kfx` (split from `cg.kfx`), the form
  loop, makes each form's twins after its words and counts its globals'
  writes, and the listing writes each word's register code after its
  cells.
- **What it makes**: leaves and framed bodies; constants and folding;
  tests an `if` decided; `let`s in registers or the frame; standard
  operations in line; calls, call-outs, more arguments than registers;
  the procedure's own calls (`invokeself`), loops, and a top-level
  procedure's calls of itself through its global, guarded; the fast and
  plain versions; a leaf's tail calls by parallel moves; quoted data, the
  stack code's interned nodes.
- **Measured**: of the 292 programs whose stack words are Fixpt's, 523
  register words are Fixpt's exactly, 72 differ, 241 are declined, none
  is kayak's alone. `tak`'s 241 cells are Fixpt's, and tier 6 runs kayak's
  listing as fast as Fixpt's (`tak 30 21 12`, 0.07 s each). Every program
  agrees on every `--regs` way.
- **The differences**: 68 are what kayak's compiler does not yet do:
  small global procedures inlined behind guards, constant globals folded,
  constant lists unrolled. 2 are Fixpt's `the` kept as a node (an
  ascription makes an argument not simple: the same code, made in
  another order). 2 are Fixpt's specialization's registry (a procedure
  taking procedures it only calls is no plain callee, so a call of it
  keeps its caller from being a leaf).
- **Next**: inlining and constant globals (most of the differences, and
  `bench/helpers` and `bench/lists`' tier 6); then stage 2.

## Inlining, unrolling, constant globals (C252)

Small global procedures inlined behind guards, small self-calling ones
unrolled where called with a constant list (`nil` only, yet), constant
globals folded in fast versions. What Fixpt keeps in globals set and
restored around an inlined body is said in the register environment
(`%genv`, `%inlining:G`, `%const:G`). 573 register words Fixpt's
exactly, 11 differ. Next: stage 2.
