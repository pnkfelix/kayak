# Types as reps, in kayak

Status: design (2026-11-01, C299); step 1 done (C300); step 2 begun
(C301-C303: 1334 of 1365 definitions round-trip, the rest declined). The user's direction: adopt Fixpt's
"heap types" (its `docs/research/heap-types.md`, steps 1 to 6 done at
the pin 63b95ef) before `stage` and sessions, so that those are built on
reps from the start rather than on text and then moved, as Fixpt's were
(its step 6).

## What Fixpt did

A **rep** is a type as a graph of nodes: a node is a head (a name:
`subr`, `poly`, `ref`, …), the names of the variables it binds, and its
operands. Operands are other nodes by index, bound variables by their
binder's node and place (alpha-equivalent types have one rep), holes
(free variables, what evidence fills), constants (abstract and generative
types, by key), symbols, integers and flags. A node may be a **set** (an
effect's atoms), its operands ordered canonically. A rep made by
`Builder::finish` is canonical: the type's minimal graph, numbered by a
walk from the root, so equal types have equal reps and one digest
(FNV-1a). Reps have a binary form, a readable form, a validator, and a
filling of holes by evidence.

The crate `fixpt-reps` knows nothing of the type language. What heads
mean is each checker's: Rust's `reps.rs` (export, import, and the table of
heads), and FX-26's `check-reps.fx`. The FX-26 checker does not
canonicalize. It hands a type's nodes to the standard procedure
`rep-of-nodes`, which the runtime implements with `fixpt-reps`, and gets
the binary form back as a bytevector; `rep-nodes` is the reverse. Both
are standard FX-26 procedures (`standard.rs`), as is `rep-cell` (a rep as
compiled code holds it).

Fixpt replaced text at four boundaries: a `stage`'s `M`, `E` and `R`;
run-time evidence; a `.fasl`'s types; and, for its tests, the two
checkers' agreement, by reps rather than by printed lines. The checkers
keep their own stores for inference (unification needs mutable
variables) and convert only at the boundaries.

## kayak's boundaries today

kayak has no `stage`, evidence or `.fasl` yet. Types leave its checker as
printed text in **facts** (`note-fact`, `FILE:POS what …`), read back by:

- KFX's emitter, which re-reads fact text with `read-all` at about 20
  places (`emit.kfx`, `emit-tops.kfx`): `sum-type`, `rdata-type`,
  `head-unbox`, `native-types`, `letrena-type`, `letrena-locals`,
  `blob-at`, `blob-make`, `blob-raw`, `rheld`, `spawn`, `wrap`, and the
  types of globals;
- the engine's lowering and tier 1's code generator (`fact-at`, C215);
- the incremental compile's memos (`kfx --cache`, C54), which keep facts
  across runs as text;
- the checkers' agreement test (`crates/kayak-checks/tests/checkers.rs`),
  by printed lines against `fixpt check`.

What text costs kayak now: every label had to be stripped of `#:` again
on the way back once types printed labels as keywords (C291, `label-of`);
the emitter parses types it was handed already elaborated; and the
agreement test compares printings, so a difference in how the two print
a type (a datatype as its `mu`, a `nat`'s size, an abbreviation's name;
88 programs) cannot be told from a difference in the type.

## The proposal

### The crate: Fixpt's, taken whole

`crates/kayak-reps`: Fixpt's `fixpt-reps` `lib.rs`, `binary.rs`,
`fill.rs` and `text.rs` copied as they are, by `scripts/built-in.py`
(its `COPIES`, checked at the pin as the conventions' files are). Fixpt's
`heap.rs` is not copied, as it is Fixpt's heap's: kayak's `heap.rs` is
its own, with the same names, over kayak-rt's bytevectors (a rep crosses
as its binary form, as the standard procedures have it). Taking the
crate whole keeps the canonical form, the digest and the binary form
byte for byte Fixpt's, so that a rep kayak's checker makes and one
Fixpt's makes can be compared as bytes.

### The standard procedures

`rep-of-nodes`, `rep-nodes` and `rep-cell`, typed as Fixpt's
`standard.rs` types them, in all three of kayak's runtimes:

- **kayak-rt** (KFX): by `kayak-reps`, a node list in, a bytevector out;
- **the engine**: ops, as C293's are;
- **tier 1**: cells/ primitives (kayak-rt's own, cells/ being KFX).

KFX may call them: they are FX-26's (KFX ⊂ FX-26). The checker, in KFX,
uses them through kayak-rt, as Fixpt's FX-26 checker uses them through
its runtime.

### kayak's export and import

`kfx/reps.kfx` (after `check.kfx`): kayak's `ty` as rep nodes by Fixpt's
table of heads (`reps.rs`), so that the reps agree:

| kayak | rep |
| --- | --- |
| `ty-int`, `ty-i64`, `ty-f64`, `ty-bool`, `ty-char`, `ty-string`, `ty-unit`; `ty-data` `symbol`, `keyword`, `datum`, `i8` … | `(base NAME)` |
| `ty-nat` | `(nat (finite))`; `%natof` `(nat N)` |
| `ty-void`; `nil`, `false` | `(void)`, `(nil)`, `(false)` |
| `ty-list E R` | `(pairof E node R #t)` whose tail is itself: a cycle |
| `%pairof A B R` | `(pairof A B R #f)` |
| `ty-ref T R`, `%icell`, `%place`, `%mark-key` | `(ref T R)`, `(icell T R)`, `(place R)`, `(mark-key T R)` |
| `ty-proc ES PS R` (and `%vsubr`, `convof`) | `(subr C E (params P …) R)`, `E` an `&effect` set |
| `%poly` | `(poly {x …} (binder k distinct) … T)`, its variables bound |
| `ty-var` under a binder | `#n.p`; free (a stage's) a hole |
| `ty-shape #t/#f LS TS` | `(productof l T …)`, `(sumof l T …)` |
| `%union` | `(union T …)`, in the order written |
| `%mu` | a cycle in the node table, no node of its own |
| a datatype or generative type by name | `(named <g type key> d …)`, a constant |
| `%mtype` | `(moduleof {t …} (binder k) … (descs …) (vals x T …))` |
| `%app`, `%dlambda` | `(app f d …)`, `(dlambda {x …} …)` |
| `%array`, `%narray`, `%bloblet`, `%stringof`, `%bytes` … | as `reps.rs` writes each |
| `ty-meta` unsolved | declined (no rep), as an unresolved `select` is |

Effects: kayak keeps atoms as strings (`read:@a`, `g:x`, `spin`); each
becomes an atom node of the set. Regions: `ty-rconst` `(rconst a)` or
`(heap)`, `(frozen …)`, `(global g)`, `(globals)`, a variable bound or a
hole. `%named` (an abbreviation kept for printing, C164) is seen through:
a rep is the type, not how it was written, as Fixpt's is.

Import is the reverse: nodes to `ty`, a cycle to a `%mu`, constants found
among the program's datatypes and generative types by key. Keys follow
Fixpt's for now: a generative type's name and place.

### The steps

1. **The crate and the procedures**: `kayak-reps` copied and checked at
   the pin; `rep-of-nodes` and `rep-nodes` in kayak-rt, the engine and
   tier 1. Test: Fixpt's own reps tests' round trips (binary, readable),
   run on kayak's copy.
2. **Export and import**: `kfx/reps.kfx`. Test: every type of the test
   programs kayak's checker takes, exported, imported and exported again,
   is the same rep (Fixpt's step 1 and 2 test, on kayak's checker).
3. **The emitter and lowering by rep**: the facts that carry a type carry
   its rep instead (as text, its binary form in hex, as Fixpt's
   `%fx26-rep-of-hex` does), and the emitter, the lowering and tier 1's
   code generator import it rather than parsing printed text. The memos
   keep reps. Measure: the bootstrap's time (`regen.sh`), whose compiles
   parse those types today.
4. **`stage`**, evidence and sessions, on reps from the start, with
   Fixpt's `fixpt-cli/tests/programs/stage/` brought into kayak's tests.

## Open questions, the user's

- **Comparing reps with Fixpt's.** The agreement test could compare reps
  rather than printed lines, which would tell a type differing from its
  printing differing. But Fixpt's CLI prints no reps: its tests compare
  its two checkers in-process (`fixpt-fx26/tests/reps.rs`). A `fixpt
  check --reps` (each definition's rep, readable or in hex) would be a
  change to Fixpt, which is Fixpt's to make first.
- **Taking `fixpt-reps` as a dependency instead of a copy.** It depends
  only on `fixpt-heap`, and only in `heap.rs`. A copy keeps kayak
  building with no Fixpt clone, as kayak's checks do now; a path
  dependency would not.
