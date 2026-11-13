# kayak as a compilation service: queries with a dynamic dependence graph

Status: proposal, 2026-11-12, for the user's decisions (§6). The user,
the same day, as track A began (`fx26-macros.md`): "since we are making a
compiler 'from scratch', I want us to think harder about making this as
incremental as possible. I want the system to eg store the dynamic
dependence graph, salsa style", and "think of Kayak as being a
compilation service rather than a batch compiler. It's quite fitting
given that it's meant to be transpiling to async Rust."

## 1. What exists

- **kayak's queries** (`lib/query.kfx`, C384; `docs/research/queries.md`):
  content-addressed memos kept in a file between runs (`--cache FILE`).
  A form's check is keyed by its text and the bindings of each name the
  text mentions, found by walking the text *before* checking
  (`names-walk`); its Rust by its text, facts and the emitter's entries
  for those names. Static dependence: the key is what the form *could*
  read, by its words, not what checking it *did* read. No revisions, no
  graph, no early cutoff above a form.
- **Coopt's engine** (`coopt-query`, 883 lines of Rust; Coopt's
  `docs/research/queries.md`): salsa's design. Queries and inputs; each
  memo records the dependencies it read, in the order read; revisions
  and durabilities (draft, session, library, fixed); verification walks
  the dependencies in order and re-executes on a change; early cutoff by
  fingerprint; every run a scheduler task (a future that waits on other
  queries and spends fuel); cycles found before a wait; validate on
  resume when an input changes mid-run; demand-driven priority; jobs for
  pure work. In memory only: one long-running process, its REPL.
  Persistence was left as future work, its fingerprints chosen for it.
- **rustc's incremental** (not read for this note; from memory, to check
  before building): the dependency graph of the previous session
  serialized, and "try mark green" on the next: a node whose recorded
  dependencies are all green is green without running.

## 2. Why the macros make this pressing

A use of a macro reads the macro's definition, and its expansion reads
whatever the template names: `#%toplevel` globals, other macros, a
file's exports. None of that is in the use's text. A static scan of the
form (C384's) cannot key an expansion; a recorded read can. So the
expander, built now, should be built as queries reading through a
context, not as a pass over forms with a session's state (Fixpt's shape,
which Fixpt's REPL then keys "by the session's state").

## 3. The proposal

**One query engine, salsa's design, Coopt's interface, in KFX**, so that
the single source uses it in both of its builds: as Rust on kayak-rt (the
service) and as FX-26 on Fixpt (stage 0, the fixpoint check), where it
runs to completion synchronously. kayak-rt may later give the same names
in Rust, forked from `coopt-query` (coopt-adoption step 3's second
stage), as it gives the prelude's.

- **Inputs**: a file's text (durability by where it is: draft, the
  program, the library, the built-in modules); the options.
- **Queries**, each a function of what it reads through the context:
  `forms(file)`; `form-text(file, i)`; `macro-of(name, scope)`;
  `expand(form)`; `interface(name)` (a definition's type, as checked or
  declared); `check(form)`; `emit(form)`, its Rust; `unit(file)`, the
  crate. Keys by identity, not position: a form keyed by its file and its
  name (a definition) or its index among the unnamed ones, so an insertion
  does not shift its neighbours' keys.
- **The dependence graph recorded as each runs**, in read order, with
  each dependency's fingerprint when read; verification as Coopt's;
  early cutoff by fingerprint of the value (a body edited, its interface
  the same, stops at `interface`).
- **Persisted**: the graph and the memos written between runs, rustc's
  way, so a batch `kfx` run is a session of the service that starts warm.
  Content addressing (C384) stays where it helps (a memo's value), the
  graph where it is needed (what to re-verify).
- **The service**: a long-running `kayak serve` on kayak-sched, holding
  the database; `kfx` and `kayak-fx26` clients of it, or running it in
  process for one request when none is up. Files watched or told of; a
  request a root query at a priority (an editor's keystroke above a
  build). The REPL becomes one more client, as Coopt's is.

## 4. What changes in the plan

Track A's stages (`fx26-macros.md`) stay, but stage 1 is preceded by the
engine and the front end's queries:

- **Q1.** The engine in KFX (`lib/query.kfx` grown): inputs, queries,
  recorded reads, revisions, durabilities, verification, early cutoff;
  synchronous first (a query body a procedure), tested on its own
  against a from-scratch run (the incremental verifier's rule).
- **Q2.** The compiler's check and emit as queries on it, replacing
  C384's static keys; `--cache` persisting the graph.
- **Q3.** The service: `kayak serve`, async on kayak-sched, the CLI its
  client.
- Then track A's stage 1, the expander, written as queries from its first
  line.

Q1 and Q2 move the work that exists; Q3 can follow the expander if the
user prefers features sooner.

## 5. What this costs and risks

- The Fixpt build: KFX's query engine needs mutable tables (a region of
  its own) and runs on Fixpt as FX-26; the fixpoint check (both builds
  making the same Rust) must still hold, so a query's result may not
  depend on what was memoized, only on what it read (salsa's rule).
- Parity: Fixpt's expander is a pass; a port "decision for decision" keeps
  its decisions (what a template's names mean, fresh names, places) but
  not its control: each form's expansion a query.
- Fresh names (`%tmp:3`): Fixpt numbers them by a session counter, which
  a query cannot share without depending on every form before it. Keyed
  per form (`%tmp:FORM:3`) they are stable under edits elsewhere, but no
  longer Fixpt's spelling, which the checker's lines show only inside
  types of locals. To settle when stage 1 meets it.

## 6. Questions for the user

1. The engine's home: in KFX (both builds, the fixpoint check kept), or
   Rust only (`coopt-query` forked; the FX-26 build stays batch)?
2. Persist the dependence graph between runs (rustc's way), or the
   service's memory only (Coopt's), with C384's file for batch runs?
3. Order: Q1-Q3 before the expander, Q1-Q2 before it and the service
   after, or the expander first and queries after?

## 7. The user's answers (2026-11-12)

1. The engine in KFX, both builds.
2. The dependence graph persisted between runs.
3. Order: open. The user asked what the options differ in, and: "part of
   me thinks you should plan to start from scratch, maybe in the style of
   Abdulaziz Ghuloum's micro pass compiler tutorial" (Ghuloum, "An
   Incremental Approach to Compiler Construction", 2006: a compiler grown
   in small steps, each a working compiler for a larger language, tested
   at every step). To be answered before building.
