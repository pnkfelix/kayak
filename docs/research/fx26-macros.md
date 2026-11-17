# FX-26 as Fixpt has it: macros first, then the rest, then the runtime

Status: plan, 2026-11-12, at Fixpt's pin 4e13016. **Its order is
superseded the same day by `kc.md`** (a compiler from scratch, of FX-26
and KFX, its passes queries): the measurements below, the reading of
Fixpt's macros and stage 0 (`#%toplevel`, C397, in the checker kc keeps)
stand; the expander's stages become kc's passes, in kc's order. The user: "start
implementing FX26 to match Fixpt. It's had a lot of new features. And its
runtime got a lot better". The order, the user's choice the same day:
features first (track A), then performance through the planned Rust
runtime (track B, `coopt-adoption.md` steps 4-6), not by tuning `cells/`.

How it serves G1 (`docs/direction.md`): kayak's FX-26 front end is the one
KFX source meant to replace Fixpt's two (its Rust `fixpt-fx26` and its
front end written in FX-26). Every feature Fixpt's has and kayak's lacks
is a program the one source cannot yet take; macros are most of them.

## Where kayak stands (measured at 4e13016)

- **The checker** (`kayak-fx26 --check` against `fixpt check`, the
  `checkers` tests): of 726 programs Fixpt accepts, kayak's says the same
  of 470, differs on 105, refuses 151; of 537 Fixpt refuses, kayak
  refuses every one. Of the 151: `define-syntax` (macros/ 81,
  procedural/ 11, and macro-made programs elsewhere), `include` 8,
  `module-parameters` 4, `hide` 3, `define-syntax-rec` 3, `#%toplevel`
  (modules/toplevel-qualified-* 10), keyword constructors (`#:ok`),
  `modnames`, `type-at`, `allof` calls fitting by conversion, and a tail.
- **The generator** (`kayak-fx26 --cells`): 474 programs agree on tier 1,
  127 it cannot compile yet, mostly for the same reasons.
- **Speed** (`fixpt bench`, the run alone, against kayak-cells' tiers on
  kayak's words, the whole process less a trivial program's): kayak's
  tier 6 is 1.6-3.7x slower than Fixpt's fastest machine
  (`fx-regs-rsjit-nativecc`) on its eight benchmarks; tier 1 2-3x slower
  than Fixpt's Rust interpreter.

## Fixpt's macros, as they are

Typed hygienic macros, Herman's, adapted (`typed-macros.md`,
`binding-shapes.md`, `macro-provenance.md`, `docs/fx26.md` from
"Macros, the prototype"): `define-syntax NAME SIGNATURE (syntax-rules
...)`, the signature a syntax type with binding annotations (or inferred
from `syntax-rules`), the definition checked against it, each use parsed
by it and expanded by transcription, the template's binders made fresh
(`%tmp:3`), its free names the top level's through `#%toplevel`. Then:
ellipsis, macros that make definitions and descriptions, binders of
descriptions, `defn`, `import-with-syntax`, binding shapes (what a module
gives, known to the expander), hygiene's later steps, macro provenance
(a macro's code placed at one id), and procedural macros, whose
transformers are FX-26 code run while the program is read.

Two expanders, Rust's (`macros.rs`, about 9,000 lines) and FX-26's
(`macros*.fx`, 22 files, 9,808 lines), agreeing on every program
(`macros_both.rs`). The FX-26 one runs in `parse-tops`, on each top-level
form before anything else sees it; the checkers and compilers see only
expansions.

## Where kayak's expander goes

kayak's FX-26 front end reads a program with kfx's reader (datums with
places), checks it with kfx's own checker in its FX-26 dialect
(`kfx/check*.kfx`), and compiles it with `fx26/cg.kfx`, a port of Fixpt's
FX-26-written parser and compiler, decision for decision. The expander is
one stage between the reader and both: datums in, datums out, so that the
checker and `cg.kfx` both see the expansion, as Fixpt's both see its.

**Fixpt's FX-26-written expander itself, made KFX by a script** (C429,
`scripts/macros-port.py`): KFX already reads nearly all the FX-26 the
expander is written in (tuples, `->*`, symbols, `datum`), so its 25 files,
and what they reach of Fixpt's parser's (its syntax, its failure), are
taken as Fixpt writes them, each file's names made one module's
(`fx26/macros/expander.kfx`), what KFX lacks given by a shim written by
hand (`fx26/macros-shim.kfx`: keywords, `eq?` of booleans, `datum-dotted`,
`parse-nat`, the procedural primitives as not yet). The pin's moving
remakes it; `check.sh` holds it to the pin. This replaces the plan below
of a port by hand, decision for decision, as `cg.kfx` is of its
compiler: a port would be a second text of the expander to keep, which
is what kayak means to end. Not the
Rust one: the FX-26 one is a front end's, as kayak's is, and is the one
Fixpt means to keep. Its names (`%tmp:3`), its places (a key past every
text's for each part a template makes) and its `#%toplevel` forms are
what make kayak's checker lines and words Fixpt's, so they are taken as
they are.

## Cacheable and incremental (the user, 2026-11-16: "Do your best to keep it cacheable and incremental")

Fixpt's FX-26 expander keeps its state in globals each form updates
(`macros.fx`'s `defined`, `loads`, `globals`, `fresh-count`, `marks`; the
`mu` table; `by-state`, which snapshots them per form for a session). The
port keeps every decision and makes the state a value:

- **One form at a time, a function.** The port keeps Fixpt's state where
  Fixpt keeps it, refs (at `@m`), so that each file reads as Fixpt's;
  but `expand-form` takes the form and the state the forms before it
  left as a value (`mstate`: the macros defined, the trailmarks, the
  globals defined, the fresh names' count, the `mu` table's length),
  sets the refs from it, and gives the expansion and the state after, as
  Fixpt's `by-state` snapshots a form's state for a session. Nothing else
  is read; the `mu` table only grows, so a state's length says which of
  it is in force.
- **Fresh names numbered from the form's start.** Fixpt numbers fresh
  binders program-wide (`%x:7`, the seventh since the program began), so
  each form's names depend on how many every form before made. The
  expansion is made with its own count from 0 and a count of how many it
  made; a renumbering step adds the count before it (Fixpt's names, as
  `fixpt check` prints them). The expansion so made is a function of the
  form and the macros it uses alone, and is what is cached; renumbering
  is linear and runs again whenever an earlier form's count changes.
- **Dependencies by name.** A use's expansion depends on the macros it
  names (the use's head, and every macro its expansion's uses name, to a
  fixed point), each by its definition, not on the whole environment: a
  form whose expansion names no macro depends on no `define-syntax`. As a
  query (`lib/db.kfx`, kc's step after 14), `expanded(form)` reads
  `macro(name, before form)` for each name it meets, so editing one
  macro re-expands only its users, and a form that defines no macro
  leaves the state as it was (early cutoff).
- **A macro's value is its own.** A `define-syntax`'s checked clauses,
  signature and `mu`s are one value, compared by its text and the
  macros it uses, so that defining it again the same way changes nothing
  downstream.

**As built (C445), in kc**: no state carried at all. Each form is
expanded in an instance of the expander of its own, given its text, the
definitions of the macros it names (to a fixed point, each by the scope
query `macro-in`), the shapes of the globals it names (by `global-in` and
`entry`, each name alone), and its hash's count (Fixpt's fresh names are
per form since 0585488, C441). A macro's definition is defined again in
each instance that needs it, cheap; its value is the definition's text.
What a form needs that no written definition gives (a macro or a shaped
global a use makes) is refused, not yet.

**Procedural macros (C446), in kc**: a form's expansion reads each
transformer's answer it needs as an input, `phase1 REQUEST`; one not
set yet, the query says `(needs REQUEST)`, the driver answers it (by
kc's own native code, C447: the request's phase-1 program a file of the
database, its forms queries, built and run) and asks again. A procedural macro's definition
needs the `begin-for-syntax` forms before it too (`wforsyntax`), which
its replay replays first.

## Stages (track A)

Each stage ends with the full check; each moves programs from refused to
agreeing, and the passing lists only grow.

0. **`#%toplevel`** (done, C397), which every expansion uses: `(with #%toplevel
   name)`, `(select #%toplevel T)`, `#%toplevel:name` as a value, a type,
   an effect, a description's head, read and checked as Fixpt reads them,
   and compiled as the plain name where nothing shadows it
   (`modules/toplevel-qualified-*.fx`).
1. **The expander's core** (as the pin has it: Fixpt's 2026-11-07 state
   was the plan's first, and by 120b70e (C428) a signature is written
   `(macro SHAPE MADE FACT …)`, the shape as a `syntax-rules` pattern,
   what a use makes said, `desc` for a description's part, `type`
   refused; one signature a macro, `clauses` gone; an expansion's depth
   and uses limited, `FIXPT_MACRO_DEPTH`, `FIXPT_MACRO_USES`; a
   `define-syntax` a template makes defines a macro; trailmarks; facts
   under an ellipsis. The port is of the pin's expander): the syntax types
   (`macros-types.fx`), the kernel's signatures (`macros-kernel.fx`),
   alignment and definition (`macros-align.fx`), `define-syntax` with a
   written signature, uses transcribed, fresh binders, free names through
   `#%toplevel` (`macros.fx`); errors where the template wrote them.
   Programs: `macros/` with written signatures.
2. **Signatures inferred** (`macros-infer.fx`, `macros-solve.fx`) and
   ellipsis: `syntax-rules` with no signature, the commonest form.
3. **Macros that make definitions and descriptions**, binders of
   descriptions, `defn`, `define-syntax-rec`; `import-with-syntax` and a
   file's macros (`macros-files.fx`).
4. **Binding shapes and hygiene's later steps** (`macros-shapes.fx`,
   `macros-shaper.fx`, `macros-scopes.fx`, `modnames`), macro provenance.
5. **Procedural macros** (`macros-procedural.fx`): a transformer is FX-26
   code run while reading, so kayak's front end must run FX-26 code it
   has just checked. Done (C446). Fixpt's expander asks its runtime by
   text (`%forms`, `%define`, `%apply`: Fixpt's `procedural::asked`) and
   is answered by text, `=` and what was given or `!` and why: a
   function of the request's text alone. kayak's shim answers from what
   the instance was given and keeps the first request it was not; the
   host answers that and expands again. The answer is the engine's
   (`kayak-fx26 --phase1`): Fixpt's prelude, the `begin-for-syntax` forms,
   the transformer at its style's type, the call, checked and run as a
   program, in a process of its own (the engine's heap and machine are
   one, the program's). kc keeps each answer as an input, `phase1
   REQUEST`, so a request is run once per database. kc answers it with
   its own native code (C447): the phase-1 program a file of its
   database, a transformer compiled items like any other, a use's call the
   one new item at each use.
6. **The rest of the refusals**: `include`, `module-parameters`, `hide`,
   keyword constructors, `type-at`, `allof` by conversion, the tail; then
   the 105 differing lines.

## Track B, after A

`coopt-adoption.md` steps 4-6: `kayak-heap` (Coopt's collector over
Fixpt's format), `kayak-machine` (Fixpt's cellular machine in Rust, a
task on it), native code on it; `cells/` and the IR engine retired once
the new machine passes the corpus. Measured against `fixpt bench` as
above.

**And the compiler, Twobit's way** (the user, 2026-11-12): "Fixpt always
had to deal with the oddity that it had two different compiler source
codes that I tried to keep in sync. This made it hard to implement Twobit
style optimizations. But Kayak should be able to implement Twobit style
optimizations in KFX since it has one source of truth that it then
transpiles." Clinger's Twobit: after expansion, a pass of analysis
(references, assignments, escapes), then whole-program-ish rewriting
(inlining, lambda lifting, constant and copy propagation, closure
conversion choosing what each lambda needs), representation inference,
then code generation. kayak can do this once, in KFX, where Fixpt would
have to do it twice. A consequence to settle when it starts: where the
optimizer goes past Fixpt's compiler, kayak's words stop being Fixpt's
listing exactly (the `same words` count falls), while every program's
answer is still Fixpt's (tier 1's test compares answers). The listing's
exactness then holds for an unoptimized mode only, if it is kept.
