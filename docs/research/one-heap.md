# One heap: the engine on tier 1's heap (Fixpt's layout)

Status: plan (2026-11-03, C317). The user's decision, the same day: unify
the heaps before `stage` comes to the compiled tiers
(`docs/research/compiled-stage.md`, question 4).

## Why

kayak has two heaps, by history (C316). The engine's (`fx26/heap.kfx`,
C64) is a layout of its own: a word's low two bits its tag (fixnum,
pointer, immediate, IR node), an object a header word and its payload, a
copying collector. Tier 1's (`cells/heap.kfx`, C122) is Fixpt's layout,
taken whole (C121): values `vbase + 8 index + tag`, pairs of two words,
everything else a bloblet, and Coopt's collector's baseline.

This was always the intent, though it wasn't said in so many words: the
user (2026-11-03), "my intention had been that the engine would likewise
be adapted to Fixpt's tagging scheme". C121 took Fixpt's formats for
compiled code; the engine's own encoding, a few days older (C64), was
never revisited. So this closes a gap rather than making a new choice.

With one heap, the engine and the compiled tiers share every value but
procedures, which need an adapter each way. That makes option (b) of the
compiled-stage note possible, and it puts kayak on Fixpt's one format
everywhere.

## What the engine's heap is used through

About 700 uses across `heap.kfx`, `words.kfx`, `vwords.kfx`, `vprims.kfx`,
`vm.kfx`, `vshow.kfx`, `vsched.kfx`, `vstage.kfx` and `table.kfx`. Nearly
all go through a small API:
- making and reading objects: `alloc!`, `obj-ref`, `obj-set!`, `obj-size`,
  `obj-kind`, `is-obj?`;
- values: `fixnum`, `fixnum-value`, `pair!`, `pair-car`, `pair-cdr`, the
  immediates (`w-nil`, `w-unit`, `w-true`, `w-false`, chars, symbols);
- strings, reals, bignums: `string-word!`, `word-string` and the rest.

Arithmetic on the encoding itself is confined to `heap.kfx` and four
places outside it. The collector takes its roots as arrays (`collect!`:
the roots, the globals, the constants).

## Where each engine object goes

Fixpt's kinds already include an interpreter's own objects; its engines
share the heap:

| engine's | Fixpt's kind |
| --- | --- |
| pair | a pair |
| string, bytevector, flonum, bignum | `string`, `bytevector`, `flonum`, `bignum` |
| sum, product | `sum`, `product` (FX-26's frozen data) |
| ref (a cell) | `box` |
| array | `vector` |
| flat array | `flat-array` |
| bloblet, blobref | `bloblet`, a suffix reference |
| eqtable | `eqtable` |
| procedure (IR code and its environment) | `closure`, its code an IR node's number as a fixnum |
| environment | `env-frame` ("the AST engine's environment frames") |
| continuation | `continuation` |
| CEK frame, constructor, the rest | `record` or `vector`, by need |
| icell, mailbox | `icell`, `mailbox` |
| symbols, chars, `#t`, `#f`, `nil`, `#u` | Fixpt's immediates (`v-true`, `v-null`, …) |

## Steps

Done: 1a, values in Fixpt's encoding (C318); 1b, objects in Fixpt's
layout and the collector over them (C319), 11 to 17% slower. 1c so far:
strings and bytevectors (C320), symbols and keywords (C321), sums (C322),
products (C323, now 0 to 4% faster than before 1a), datatype values
(C324), bignums (C325), arrays, bloblets and suffix references (C326),
flat arrays (C327), module values (C328), eqtables (C329, tier 1's too),
prompt tags and mark keys (C330). Every value the engine makes is now in
Fixpt's form but its procedures (primitives, an immediate of its own;
closures, constructors and continuations, objects of its own layout) and
its frames: those reach compiled code through adapters (step 4). Step
1's last part done (C331, C332): the engine is written over tier 1's heap
(`cells/layout.kfx`, `cells/heap.kfx`, in its unit), sharing its spaces,
its allocation's state, its collector, its symbols and its accessors; the
engine's file of it is `fx26/vheap.kfx`, its region `@c` as tier 1's,
and tier 1's heap takes a heap's effects (`cread`, `cwrite`, `calloc`), not
`cm`, which has I/O. Step 2 done (C333): the engine's numbers and
printing are tier 1's (`numbers.kfx`, `show.kfx`). Step 3 done (C334):
one scheduler's state, tier 1's `sched.kfx`; each machine keeps its own
dispatcher. Next: step 4.

1. **The engine's API over tier 1's heap, in the engine's own binary.**
   - The engine's unit loads `cells/layout.kfx` and `cells/heap.kfx`
     (renaming where the two clash: both define `alloc!`, `obj-ref`,
     `obj-set!`, `pair-car`), and `fx26/heap.kfx` becomes the engine's
     API written over them.
   - IR node words become fixnums in a closure's code field.
   - The collector is tier 1's, given the engine's roots.
   - Test: every engine test as it is, and the engine's benchmarks
     measured before and after.
2. **Printing and numbers**: the engine's `vshow.kfx` and its bignums
   (`words.kfx`) through tier 1's `show.kfx` and `numbers.kfx`, where they
   agree, so that a value prints one way everywhere.
3. **The scheduler**: one of the two (`fx26/vsched.kfx`, `cells/sched.kfx`,
   both ports of Fixpt's `sched.rs`), once their objects are the same.
4. **One program**: the engine and the cells machine in one binary, a
   procedure of either kind callable from the other through an adapter
   (as Fixpt's conventions are).
5. Then `docs/research/compiled-stage.md`, (a) or (b), on the one heap.

## Questions

- **Step 4, against Fixpt** (2026-11-04, after C334). Fixpt has no calls
  between its evaluator and its compiled code: its REPL and `fixpt eval`
  compile each form (its FX-26 compiler) and run the words on one chosen
  machine (`--machine`); its evaluator, `fx-ast-eval`, is a machine of
  its own, "the reference, not a way to run things"
  (`docs/performance.md`); its adapters are between the cellular and the
  native conventions only. So step 4 as written, the engine's procedures
  and tier 1's calling each other, would be kayak's alone. With the heap
  one (steps 1 to 3), `docs/research/compiled-stage.md`'s (a), the front
  end linked into the compiled program as Fixpt's conductor is, needs no
  adapter: staged code becomes cells. The user's to choose: step 4 as
  written, or (a) next, step 4 dropped.

- **Speed.** The engine's words are tagged in 2 bits, Fixpt's in 3, and
  Fixpt's objects are read below a suffix, not after a header. Step 1
  measures what that costs the engine. If it costs much, the API's hot
  paths (`obj-ref`, `pair-car`) are where to look.
- **Order.** Steps 2 and 3 might come after step 4, if a combined program
  is wanted sooner.
