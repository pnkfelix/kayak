# Impediments to incrementality, caching and parallelism

Choices of Fixpt's that look incidental, but that make kayak's work harder to
cache, recompute incrementally or run in parallel. Each is written up as
feedback for Fixpt: what the choice is, why it costs kayak, and an
alternative that might still work for Fixpt. The user asked for this file on
2026-11-16:

> "If you encounter deltas that seem incidental (eg naming choice for freshly
> constructed names) but for some reason interfere with incrementality or
> cacheabilty or parallelizability, make a note of it in a file, call it
> INCR_IMPED.md. The idea is to collect all such cases and provide feedback to
> Fixpt if there's an alternative they can identify that still works for them
> but also unlocks doors for you."

Each entry has the same parts:
- **Where:** the place in Fixpt's code.
- **What:** what Fixpt does.
- **Cost:** what it costs kayak.
- **Alternative:** a suggestion, for Fixpt to weigh.
- **Seen:** the date, and the kayak choice (C-number) where it came up.

## 1. Fresh names are numbered program-wide

- **Where:** `macros.fx`, `fresh-name` and `fresh-count`;
  `macros-templates.fx`, `fresh-symbol`.
- **What:**
  - A template's binders become `%x:N`, where N is one more than the number
    of fresh names made since the program, or the session's state, began.
  - `by-state` saves the count with each form's state, so that a form read
    again gets the same names.
- **Cost:**
  - A form's expansion depends on how many fresh names every form before it
    made.
  - So editing one early form renames the binders in every later expansion,
    even though nothing in those forms changed. Their checked results and
    compiled words then differ by name alone, and caches keyed on the
    expansion's text miss.
  - Forms can't be expanded in parallel either: each needs the count the one
    before it leaves.
  - Kayak's plan works around this (`docs/research/fx26-macros.md`,
    "Cacheable and incremental"). Each form's expansion is cached with
    names counted from 0, and a renumbering pass then adds the earlier
    forms' count. The renumbering still changes the output downstream.
- **Alternative:** number fresh names within the form, keyed by something
  stable, for example `%x:FORM.N`. FORM could be the form's own key, a hash
  of its text, or the name it defines, and N its own count. Two forms'
  names would still be distinct, and a form's names would depend only on
  itself and the macros it uses. If it matters to Fixpt that the names be
  short, the printer could abbreviate them.
- **Seen:** 2026-11-16, C429.
- **Fixpt's answer (f9001a3d, 05854885; kayak's pin since C441):** done
  as suggested. A form's fresh names are `%x:K.N`, K the form's key (a
  hash of what was written, and how many forms of that hash came before),
  N counted within it. A procedural macro's aliases are `%~K.N:x` alike.
  kayak's driver begins each form so (`form-begin!`).

## 2. The expander's state is globals updated form by form

- **Where:** `macros.fx`: `defined`, `loads`, `globals`, `fresh-count`, the
  trailmarks in `marks`; `macros-types.fx`: the `mu` table and
  `syntax-sigs`; many more refs (`@ms`) across the files.
- **What:**
  - Each form's expansion reads and writes module-level refs.
  - `by-state` snapshots part of them per form, for sessions.
- **Cost:**
  - To cache a form's expansion, kayak has to know everything it read, and
    restore everything it wrote, at the form's boundary.
  - The refs are spread over about 25 files. With nothing that says which
    of them carry over between forms, kayak has to treat them all as an
    input, so every form depends on every earlier one.
  - The `mu` table is the worst case. It is a program-wide array, and
    signatures name its entries by index, so a macro's value refers to
    numbers that depend on how many types were read before it.
- **Alternative:**
  - One explicit state value per form, the macros, trailmarks, globals and
    count, passed in and returned. Fixpt's `by-state` already holds most of
    it.
  - Per-expansion scratch refs that are reset when the form ends, and
    declared to be.
  - A macro's `mu` bodies kept with the macro, rather than in a
    program-wide table.
- **Parallelism too:** forms can't be expanded on several threads while
  each reads and writes refs the one before it wrote.
- **kayak's way round (C437):** the expander is a module maker, so each
  program, and each REPL session, has an instance of its own, in a place
  of its own. That separates programs, not forms: within a program, the
  forms still meet in the instance's refs. And a process may make no
  instance, as §4 says.
- **Seen:** 2026-11-16, C429; C437.
- **Fixpt's answer, in part (f9001a3d, be35fbd8; C441):**
  - The `mu` table is gone: a recursive syntax type is a value, `(s-mu
    body …)`, its variables by de Bruijn index, so a signature is of what
    was written alone.
  - What one form keeps is as at any form's start (`form-begun!`).
  - Still program-wide, and in a form's output, though only provenance
    reads them: expansion numbers and source keys. Fixpt's TODO §290 asks
    which a cache of kayak's needs: a form's share of each table,
    renumbered where it replays, or a per-form provenance designed.
  - Still open: the state in refs, which a form reads and writes; and a
    process may make no instance (§4).
- **kayak's answer to §290 (C445):** a cache of kayak's needs neither a
  form's share of the expansion numbers nor of the source keys. kc
  expands each form in an instance of its own, whose numbers start over,
  given the definitions of the macros it names and the shapes of the
  globals it names, by name. The driver writes places, not keys
  (`drive-placed`), so no number leaves the instance. The state in refs is
  no impediment so used: an instance's refs are its own, and live for one
  form. Nothing for Fixpt to change.

## 3. Positions are offsets into one program-wide text

- **Where:** the parser's `load-base` and `in-loaded-as`; the syntax's
  start and end fields; a template's parts placed at "a key past every
  text's".
- **What:** a loaded file's positions are offsets into one combined text,
  with a base per file, so a loaded file's offsets depend on what was
  loaded before it.
- **Cost:** a file's syntax, and anything keyed by its positions (facts,
  error places, the identity of a node), changes when a file loaded before
  it changes size. That holds even when the file itself is unchanged. It
  also orders the reading of loads.
- **Alternative:** positions as a file's identity, its path or content
  hash, plus an offset within that file. A made part's key would be the
  use's position plus the template part's own position, not a global
  counter.
- **Fixpt's note (TODO §290):** a session's texts are placed one after
  another by length too (`Session::eval_base`); left for when a cache of
  kayak's needs it.
- **Seen:** 2026-11-16, C429. Not felt in kc (C445): it reads each form
  from its own text, at line 1, offset 0, and moves the places back, so
  a form's positions in the instance are its own.

## 4. A procedure of a place may not be held by a process

- **Where:** `processes.rs`, `untransmissible_in`: a `Ty::Named` place
  falls to the last arm ("a place"), though a `poly` around it binds its
  region, which the arm before allows for a mailbox.
- **What:** a process's body may hold, from outside it, only what is
  transmissible, globals too. A procedure is transmissible only if its
  parameters are, and a place never is. So a process may call no global
  procedure that takes a place, and no module maker, `(poly ((p place))
  (subr E ((place p)) (i p)))`, even to make an instance in its own place.
- **Cost:**
  - kayak's REPL front, a process, can't own an expander instance. Neither
    can a worker, so expansion can't move to the workers, one instance
    each, to run in parallel.
  - The one party that may hold an instance is the machine's thread, which
    isn't a process. As FX-26 stands, the front would have to ask it to
    expand each entry and draft, and wait for the answer.
  - Test: `tests/kfx-refused/process-maker.kfx`, refused by both: "its body
    holds `counter-in`: a (place p)".
- **Alternative:** a place at a region the procedure's own `poly` binds is
  transmissible, as Fixpt already allows a mailbox at one ("A procedure's
  parameter, at a region it binds, the caller's"). The caller gives the
  place, so nothing outside the process's own place can be written through
  it. The procedure's latent effect is already checked by
  `crossing_effect`, where an atom on a bound binder is the caller's.
- **Seen:** 2026-11-16, C439.
- **Fixpt's answer (`docs/research/closed-code.md`, their working copy,
  uncommitted, 2026-11-16):** the alternative declined: the maker's result
  is a module, whose abstract types may hide a captured value, so judging
  it by its type would need another type-based approximation. Instead:
  - Today: a module file of `(module-parameters ((p place)))`, loaded in
    the process's body, is accepted, since a load sees only the standard
    environment and the spawn check exempts a load's hidden globals.
  - Proposed, not built: *closed* globals, a `lambda` naming only standard
    bindings, loads' globals, closed globals or transmissible ones; a spawn
    would not judge one by its type. It rests on a global's binding never
    changing after it is defined.
  - What kayak would need for the first: its loads are static, one module
    per path and arguments (C116, C117), so a load given a process's place
    has no top-level module to be. It would have to be an instance per
    load, made as a module maker's is (C433-C437), and the spawn check
    taught the exemption.
  - On the second: FX-26 defines a top-level global again
    (`redefine/rerun-chain.fx`, `knot*.fx`; kayak's C202), so the premise
    needs checking against redefinition, not only against assignment.
- **Taken (C442-C444):** the expander is a file of a place, loaded by
  the engine and by the REPL's front, each an instance of its own. Each
  worker could load one too, for expansion in parallel.

## 5. A form's key counts the forms of its hash before it

- **Where:** `macros-templates.fx`, `form-begin!`, `syn-hash`: K in
  `%x:K.N` is a hash of the form's symbols, strings and shape, and how
  many forms of that hash came before it. Any other atom is its kind
  alone, so `(check 1)` and `(check 2)` hash alike.
- **What:** two forms that differ only in their numbers (or characters,
  booleans) share a hash, so the later one's K counts the earlier.
- **Cost:** putting such a form before others, say one more `(check-eq?
  (f 3) 9)` among many, changes K for every later form of that hash. Each
  is then expanded again, and its fresh names change, so its checked
  result and code differ by name alone. kc reads the count as an input
  (`wseen`), so it re-runs exactly those forms. That is correct, but it is
  work an edit elsewhere causes.
- **Alternative:** hash every atom, numbers too, so that only forms
  written alike share a hash. Or count only forms whose text is the
  same. Forms that are the same text would still count each other, which
  seems unavoidable if two of them must get different names.
- **Seen:** 2026-11-16, C445.
