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
- **Seen:** 2026-11-16, C429.

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
- **Seen:** 2026-11-16, C429. Not yet felt in kayak, since the driver isn't
  written; to confirm when it is.
