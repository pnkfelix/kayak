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
9. **An array's index out of range is said differently by each of Fixpt's
   machines.** `(array-ref xs 3)` of three elements: `rs-scheme-vm` "bloblet
   field: the bloblet has no field -5"; `fx-ast-eval` and the native-calls
   cellular machine "field@: no field 5 of #<bloblet 4 fields 0 bytes>"; the
   other cellular machines "`field@` named a field that is not there". The
   words are each machine's internals, not FX-26's; strings, by contrast,
   say "string index out of range (length N)" everywhere. kc says "array
   index out of range (length N)" (C422) and its tests state no answer for
   it. Worth one FX-26 message, Fixpt's first.
10. **Fixpt fcb33f0 refuses a `load-module` of a built-in module after
   about 50 definitions (a regression, Fixpt's first).** Since 6864363b
   (heap syntax, its first stage; fd18a6ec, its parent, is fine):
   `(define c (load-module "fx26:conditions.fx"))` after 50 one-line
   definitions `(define fK (subr pure (int) int) (lambda (x) (+ x (* x
   K))))`, by `fixpt eval` on any machine, says "the checkers disagree:
   the Rust checker takes `c`'s definition, and the one written in FX-26
   refuses it: cannot read `fx26:conditions.fx`: it was not read"
   (`parser-exps.fx`, `parse-loaded`, finding no loaded file at the form's
   start). After 25 it is taken. kayak's corpus meets it in four programs
   (`conditions`, `made-*-x86_64`, `fx26:convention-types.fx`), KFX's
   prelude being their 61 lines first. **Fixed** in Fixpt 95f92e6f;
   the pin is past it (196dd2a, C427).
11. **For Fixpt: a place a procedure's own `poly` binds, transmissible
    (INCR_IMPED.md §4, C439).** A process's body may hold only what is
    transmissible, globals too, and `untransmissible_in` takes no place,
    so a process may call no module maker, even to make an instance in its
    own place: `tests/kfx-refused/process-maker.kfx`, refused by Fixpt
    ("its body holds `counter-in`: a (place p)"). This parks the REPL's
    macros: its front, a process, was to own an expander instance (C437's
    plan). The proposal: a place at a region the procedure's `poly` binds
    is transmissible, as a mailbox at one already is. The user, 2026-11-16:
    ask Fixpt first, the REPL parked till then.
