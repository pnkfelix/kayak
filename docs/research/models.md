# Models of kayak's machinery

Status: begun 2026-11-12 (C400). The user: "We may need to make some kind
of model of the incremental computation mechanism", then: "I think you
should look at Quint. And maybe Kani, though I'm less convinced of that
latter bit for this purpose."

## What is here

- `models/db.qnt`: the query database (`lib/db.kfx`) as Quint, abstractly:
  inputs with values and durabilities; three levels of queries whose reads
  depend on what they read; verification in read order; early cutoff; the
  durability shortcut. Its property, `sound`: what the database gives is
  what a run from scratch gives, every node the shortcut would take as
  current is, and every node verified this revision is. Two instances:
  `fixed`, the engine as it is, and `unfixed`, as it was before C399's
  and C400's fixes, which the model must find wanting.
- `scripts/models.py`, in the full check: Quint's simulator on each,
  10,000 random runs of 30 steps; the fixed must hold, the unfixed fail.
- Quint pinned in `tools/quint` (0.33.1; `npm ci` there).

## What it found

At once, a bug the random test of the KFX engine (`tests/kfx/db-verify`)
had missed: a node verified by its reads kept the least durability it had
when it last ran, though a read's had since been lowered, so the
durability shortcut later took it as current when it was not (C400).

## Not yet

- **Exhaustive checking** (`quint verify`, through Apalache, or its TLC
  backend, which compiles through Apalache too) needs Java 17 or more;
  the system's is 11. A snap's OpenJDK 27 (`openjdk.java`) runs it, given
  `QUINT_HOME` and a temporary directory outside the snap's private
  `/tmp` and outside hidden directories, but the first bounded run took
  more than ten minutes without an answer: set aside, the user agreeing,
  until the engine is asynchronous, where exhaustive checking pays most.
- **Kani**, for the Rust kfx makes of the engine: the user less convinced
  for this; not begun.
- The model and the engine kept in step by hand; a trace of the model
  replayed against the KFX engine would tie them.
