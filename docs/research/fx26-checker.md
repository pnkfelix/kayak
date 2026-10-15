# The checker for FX-26 at large (TODO §30)

Status: started, 2026-10-14. PLAN §9, step 1: kayak is to become FX-26's
reference, so its checker must accept what Fixpt's accepts, refuse what
it refuses, and say the same types and effects.

## The measure

`kayak-fx26 --check FILE` writes what `fixpt check` writes: each
definition's `define NAME : TYPE ! EFFECT` (a datatype's constructors and
a generative type's conversions too), each expression's `TYPE ! EFFECT`.
`scripts/compare-checkers.py` gives every one of Fixpt's 653 programs to
both and counts each program once: *same* (every line), *alike* (the same
once effects are sorted, as Fixpt sorts them), *differ*, *refused* (Fixpt
accepts, kayak refuses), *ACCEPTED* (a K1 breach: `scripts/check.sh`
fails on one), *both* (both refuse; messages not compared yet).
`--why` counts kayak's refusals by message; `--list KIND` names them;
`--dir DIR` takes one of Fixpt's test directories.

## Where it stood, 2026-10-14

Of the 357 programs Fixpt accepts: the same lines for 22, alike for 52,
different for 110, refused by kayak for 173. Of the 296 it refuses,
kayak refuses all.

What the 110 differ in, mostly printing, not checking:
- module definitions, which `--check` does not write yet (51 programs in
  `modules/`), where Fixpt writes each with its module type;
- type abbreviations: Fixpt keeps the name an abbreviation was given
  (`(subr pure (v) int)`), kayak's checker expands it at declaration (43
  lines);
- `poly`: kayak merges nested binders (`(poly ((t type) (e effect)) …)`
  for Fixpt's `(poly ((t type)) (poly ((e effect)) …))`), and writes a
  region binder `'r` inside;
- and a few that are the checker's: Fixpt masks the effects on a region
  an expression makes and drops (`int ! pure`, kayak `(maxeff (alloc @m)
  (write @m) (read @m))`).

What the 173 refusals are, by `--why`: a long tail, its head KFX's own
rules inside the checker (`a type KFX does not have` 16, `a region KFX-0
does not have` 9, `KFX's poly is over …` 8, poly binders not all regions
8, `proj` 7, effects 5), then FX-26's subsystems (sizes 20, modules 24,
higher kinds 12, processes 14, bloblets 9, recursive types 8, unions 8).

## The plan

Grown in place (the question `front-end.md` left open): K2's fixed point
and the incremental compile (C54) hold at every step, and the comparison
says each step's gain. Fixpt's checker in FX-26
(`crates/fixpt-fx26/src/check-*.fx`) is read for each feature's rules,
rule for rule, its messages too.

1. **Printing as Fixpt prints**, in the checker's own `show-ty` (the REPL
   writes the same): effects sorted, abbreviations by name, `poly` as
   written, module types. Moves *alike* and much of *differ* to *same*.
2. **The restriction pass split off** (`front-end.md` stage 2): KFX's
   refusals of what FX-26 has moved out of the checker into a pass that
   kfx runs after it and the engine does not. The engine's checker is
   then FX-26's, short only of what it does not know.
3. **FX-26's subsystems**, each its own note first, largest first:
   modules, sizes, processes, higher kinds, bloblets, recursive types,
   unions.
4. **Refusals' messages** compared too: the same place and words as
   Fixpt's, for the 296 both refuse.
