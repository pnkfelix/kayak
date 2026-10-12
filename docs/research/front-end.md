# The front end, shared: the checker grown to FX-26's

Status: research and a plan, 2026-10-12. `PLAN.md` §6.5, step 8.

Read: Fixpt's `docs/fx26.md` at `17ac0c5` (the kernel; unions of shapes
and what tests prove; sizes; higher kinds; variadic procedures; modules;
immutable data and symbols; datums and quote); the survey
(`scripts/survey-fixpt.py`, `docs/survey.md`), run again today.

## Findings first

1. **kfx accepts 97 of the 308 programs Fixpt's checker accepts** (91
   two days ago; the I/O and bytes, C49, gave the rest). By Fixpt's test
   directory, the refused are: modules 53, run 34, native 21, sizes 19,
   higher kinds 12, unions 11, datum 9, recursive types 8, bloblets 7,
   conventions 6, the rest fewer than 5 each.
2. **The refusals are of three kinds**, and only one is the checker's:
   - *what FX-26 has and KFX's checker does not know*: unions of shapes
     and what tests prove; `symbol` and `datum`, quote; sizes (`nlist`,
     `(nat k)`, lemmas, `proves`); higher kinds; variadic procedures
     (`vsubr`); dependent lambdas (`dlambda`, `dletrec`); modules' forms
     KFX lacks (`load-module`, parameters, `moduleof` items); a dozen
     primitives (`arithmetic-shift`, `set-car!`, `apply`, `odd?`, …);
   - *what has no Rust mapping yet, or never*: continuations resumed
     twice; continuation marks (`mark-key`); procedures in bloblets;
     flat arrays of more than `i64`; arrays of values;
   - *KFX's own stricter rules*, on purpose (`tests/kfx-stricter`): names
     Rust would confuse, a type named as Rust's (`Cell`), redefinition.
3. **The split PLAN §6 asks for follows the kinds**: a checker for FX-26
   that takes the first, and a KFX pass after it that refuses the second
   and third, by the facts the checker leaves. Today one checker does
   all three, and a refusal of the first kind is indistinguishable from
   the second in its message.
4. **Most of the first kind is also what kayak's own FX-26 runtime
   (phase C) needs**, and nothing of it is cheap: unions need a run-time
   shape test and narrowing (Rust: an enum per union, a `match` per
   test); sizes are a solver; higher kinds, type application in the
   checker's core. Each is a project as large as C45 (suspension).

## The plan

Stages, each a commit or a few, each moving programs from refused to
agreed (`scripts/run-fixpt-programs.py` counts them):

1. **The cheap names**: the primitives missing (`arithmetic-shift`,
   `odd?`, `bool=?`, `apply` on a list, …), each with its Rust; `symbol`
   as a base type (an interned string in kayak-rt), `quote` of a symbol,
   `string->symbol`, `symbol->string`, `eq?` on symbols.
2. **The restriction pass, split off**: the checker's refusals of the
   second and third kind moved to a pass over the checked program, with
   their messages; the checker then says only what FX-26 says. Measured
   by the survey: the same programs refused, by the pass.
3. **Unions of shapes and narrowing** (`unions/`, `datum/`, much of
   `run/`): `(union T …)` as a Rust enum over its members' shapes, the
   shape tests as `match`es, a variable narrowed where a test proves it.
   Then `datum` (a union with pairs and symbols) and `quote` of any datum.
4. **Modules' remaining forms** (`modules/`, 53): the largest group;
   `load-module` and module files need the driver's I/O (C50), which is
   now KFX's.
5. **Higher kinds, variadic procedures, sizes**: each its own note first.

What stays refused for Rust, for good, goes to phase C's engine: resumed
continuations, marks. The pass's refusals are then the list of what the
engine is for.

## Not decided here

Whether the checker grown to FX-26 is a new one, written after Fixpt's
own front end in FX-26 (`fx26:parser`, its checker), or kfx's grown in
place. In place keeps K2 (the fixed point) at every step and the
incremental queries (C54, C55) working; Fixpt's front end in FX-26 is
where to learn each feature's rules. The plan above is in place.
