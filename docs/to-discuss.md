# To discuss with the user

Things noticed along the way that do not seem right, kept for a later
conversation (the user, 2026-11-12: "if you happen to notice something
that doesn't seem right, take a note of it"). Newest last; each says
where it was seen and what it would take.

1. **`docs/direction.md` calls `fx26/cg*.kfx` drift; track A extends it.**
   §2 lists "a third copy of Fixpt's cellular code generator
   (`fx26/cg*.kfx`)" under what drifted, while §7 keeps "the compiler
   half". Track A (`docs/research/fx26-macros.md`) grows the front end and
   `cg.kfx` both. §7's reading (keep) is the one followed; §2's sentence
   should probably be revised so the two agree.
2. **The passing lists are not blessed by the check.** At b41109e,
   `checks-summary cells --bless` added 193 lines to `tests/cells-passing`
   and 72 to `tests/cellgen-passing`: programs agreeing since earlier pins
   that no list protected. "Passing lists only grow" holds only as far as
   someone blesses them. Should `check-snapshot.sh --commit` bless (adding
   only), or report the programs that newly agree?
3. **kayak's speed numbers are not Fixpt's kind.** `fixpt bench` times the
   run alone; kayak-cells has no such mode, so kayak's include process
   start and, on tiers 4-6, compiling words to machine code. The 1.6-3.7x
   gap is an upper bound. A run-alone timer in kayak-cells (or the new
   machine) would make track B's measure fair.
4. **Twobit-style optimization against listing-exactness.** Most of
   kayak's checks of its generator compare words with Fixpt's listing
   (`same words`, the register words' counts). An optimizer past Fixpt's
   breaks that by design. Keep an unoptimized mode that stays exact, or
   compare answers only once optimizing?
5. **The checker crashes rather than refuses.** `(listof int)`, its region
   left out, stopped kayak's checker with `car: nil` (KFX's `car` of an
   empty list, from `third` of a two-element form), not a message; fixed
   for that form (C397), but any form shorter than a case expects does
   the same. A checker should refuse, in words, what it cannot read. A
   sweep for `second`/`third` of unchecked forms, or a reader of forms
   that checks their shape first, would close the class.
6. **kayak writes an abbreviation's name where Fixpt writes what it stands
   for.** `(define-type t int)` then `(subr pure (t) t)`: Fixpt prints
   `(subr pure (int) int)`, kayak `(subr pure (t) t)`. Likely much of the
   105 programs whose lines differ (`checks-summary checkers`). In
   `toplevel-qualified-type.fx` it makes kayak's line read `idx` where
   `idx` is a type variable in scope, which misleads. Worth deciding
   whether kayak follows Fixpt here.
7. **A changed answer goes to a file of this machine's, silently.**
   `corpus.py --bless` writes a corpus program's answer to `NAME.out` only
   on the home platform (arm64 macOS) or when there is none; elsewhere, an
   answer that differs from `NAME.out` goes to `NAME.x86_64-linux.out`
   (C134's design, for answers that really are a machine's). Twice today
   it kept a stale shared answer while the platform file held the right
   one (`shapes.out`, then `db-verify.out`, where it hid for a while that
   a bug had been fixed). Perhaps `--bless` should say when it writes a
   platform file, or refuse to unless asked.

8. **FX-26's "no datum labels" has outlived its reason (to raise with
   Fixpt).** `docs/fx26-grammar.md` (2026-10-02): "No datum labels (`#0=`):
   nothing in FX-26 is a cyclic datum." Since then FX-26 has `set-cdr!`,
   lists at a writable region "may be cyclic", and `docs/fx26.md`'s
   `datum` section anticipates "a reader of datum labels (`#1=(a . #1#)`)"
   for frozen data that may have a cycle; Fixpt's printer writes cycles
   with labels and its listings label every shared object. The user
   (2026-11-13): "Surely that statement is out of date, or at least the
   justification?" kayak reads labels in data alone (`read-data`, C417);
   in program text it would be an FX-26 extension, Fixpt's first.
