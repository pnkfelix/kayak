# kc: kayak's compiler, from scratch, incremental, of two languages

Status: the plan, 2026-11-12, the user's direction the same day. It
supersedes the ordering of `fx26-macros.md` (whose measurements, and its
reading of Fixpt's macros, stand) and builds on
`compilation-service.md` (its engine and service, the user's answers
there: the engine in KFX, the graph persisted).

## 1. The user's direction

- "think harder about making this as incremental as possible. I want the
  system to eg store the dynamic dependence graph, salsa style."
- "think of Kayak as being a compilation service rather than a batch
  compiler. It's quite fitting given that it's meant to be transpiling to
  async Rust."
- "part of me thinks you should plan to start from scratch, maybe in the
  style of Abdulaziz Ghuloum's micro pass compiler tutorial."
- "your new compiler should start off treating both FX26 and KFX as its
  source languages. And it should handle transpiling KFX to async Rust as
  one backend, and then also compile both FX26 and KFX to whatever other
  backends you choose to make (obviously one native one with our own
  async and parallel runtime and incremental GC, that's part of the whole
  point; but we might want to also have a wasm backend, maybe)."
- Earlier the same day: Twobit-style optimizations, which one source makes
  practical (Fixpt's two kept them out).

## 2. What kc is

One compiler, written in KFX (so the single source still runs on Fixpt as
FX-26, and both of kayak's builds compile it), whose:
- **sources** are FX-26 and KFX, one front end: KFX is FX-26 with the
  restrictions that give it a Rust meaning, a mode of the same checker,
  not a second front end;
- **back ends** are: async Rust, of KFX (what `kfx` does today); native
  code, of both, on kayak's own runtime (async, parallel, incremental
  GC: track B of `fx26-macros.md`, now grown with kc rather than after
  it); and later, perhaps, wasm;
- **passes** are many and small (nanopass: Sarkar, Waddell and Dybvig;
  Ghuloum's tutorial grows a compiler that way), each a function from one
  small language to the next, each run per definition as a **query** on
  the engine (`compilation-service.md`), its reads recorded, its graph
  persisted; Twobit's analyses and rewrites are passes among them;
- **service** is the long-running form: `kayak serve` holds the database;
  the command lines are its clients, or run it in process for one
  request.

## 3. Grown Ghuloum's way

Each step is a working compiler for a larger language, tested end to end
before the next: programs of that language compiled by each back end that
takes them, run, and their answers compared with Fixpt's (the corpus and
its kept answers, `tests/fixpt-answers`), the set of programs that pass
only growing. Ghuloum's order, adapted:

1. integers; 2. immediates (booleans, characters, unit, the empty list);
3. unary primitives; 4. binary primitives; 5. `let`; 6. `if` and the
derived conditionals; 7. pairs and the heap (allocation first, no
collection, as Ghuloum's; the collector when programs need it);
8. procedures; 9. closures; 10. tail calls; 11. strings, vectors,
symbols; 12. assignment (`ref`, `set`); 13. modules; 14. the effects that
need the runtime (regions, prompts and continuations, processes);
15. macros (the expander, `fx26-macros.md` stages 1-5, as passes); and
FX-26's remainder, until kc takes the corpus.

From step 1 the front end is the checker (types are what the Rust back
end is written from, and what KFX is told apart by), and every pass a
query. The native back end's runtime grows with it: step 1's is a
function that prints a value.

## 4. What is kept, what is retired

- **Kept**: the reader; kfx's checker (`kfx/check*.kfx`), wrapped as the
  front end's passes, converging on Fixpt's as it has been; the corpus,
  Fixpt's answers, the comparisons; `lib/query.kfx`, grown into the
  engine; the lang items (`lang/items.sexp`), the vocabulary kc's runtime
  and back ends share; Coopt's runtime crates (kayak-sched, kayak-runner)
  under the native runtime.
- **Retired once kc covers what each does**: `kfx`'s emitter
  (`kfx/emit*.kfx`), when kc's Rust back end makes the compiler's own Rust
  and the fixpoint check moves to it; `fx26/cg*.kfx`, its register code
  and listings, and the cellular machine (`cells/`), when kc's native
  back end takes the corpus; the IR engine (`fx26/v*`).
- **Parity**: with Fixpt's answers, not its listings: kc's code is its
  own. The listing checks stay with what they check until it is retired.

## 5. Choices made here, to revisit

- **Native code, first form**: x86-64 assembly text, assembled and linked
  with the system's `cc` against a small runtime crate (Ghuloum's way):
  readable and easy to check. kayak's own encoder (`kfx/regs-isa.kfx`'s
  encodings) when the service compiles in memory.
- **Where it lives**: `kc/`, a unit of its own (`kc/kc.unit`), beside
  `kfx/` and `fx26/`; its tests in `tests/kc/`.
- **First step's harness**: a program compiled by kc, run, its printed
  value compared with Fixpt's for the corpus programs in the language so
  far (`scripts/kc-corpus.py`), and with kayak's own small programs per
  step (`tests/kc/stepN/`).

## 6. Order of work

- **K0. The engine** (`compilation-service.md` Q1): inputs, queries,
  recorded reads in order, revisions, durabilities, verification, early
  cutoff, the graph and memos persisted; synchronous first; tested
  against a from-scratch run.
- **K0** done (C398). **K1** done (C399). Step 2, the immediates, done (C401). Step 3, unary primitives, done (C402). Step 4, binary primitives, done (C403). Step 5, `let`, done (C404). Step 6, `if` and the derived conditionals, done (C405). Step 7, pairs and the heap, done (C406). Step 8, procedures, and a query for each form, done (C407). kc's files as modules, each hiding its helpers (C411). Step 9, closures, done (C412). Step 10, tail calls, done (C413). Each step from here measured against Fixpt, edit by edit (`scripts/kc-bench.py`, `bench/kc.tsv`; C414). The check memoized, a refusal a value, and `kc serve` (C415). Items depend on the names they look up (C416). The database's keys data, its files a graph and a payload (C417). The graph read as needed (C418). `kc build`, objects by fingerprint (C419). Step 11: strings and symbols (C421), arrays (C422). Types in kc's core: abbreviations, products, sums, datatypes (C423). Step 12, refs (C424). Step 13, modules (C425). `define*`, `define-rec`, `letrec`, `plambda`, `proj` (C426). Step 15, macros, begun before step 14: each form's expansion a query of the names it reads (C445). Procedural macros and `begin-for-syntax` (C446): each transformer's request an input, answered by the engine. Answered by kc's own native code (C447): a request's phase-1 program a file of the database, compiled, built and run; `datum` data, `quasiquote`, `typecase`. A transformer's program built once, each use a run of it given the use (C448). Each request asked as the expansion goes (C449): `phase1 REQUEST` a query whose body runs the transformer, no expansion run again. kc's own assembler (C450): x64.kfx's text an ELF object, as `as` makes it, object for object; `kc build` links kc's objects. kc's own convention and `kc run` (C451): the state in `%r15`, a stack's limit and fuel checked at each entry, a failure back to the trampoline; the program's image sealed in kayak-jit's code area and called, each step program answering in process as its executable does. Its runtime kcrt's, one copy (C452): each routine's address by a syscall of kayak-rt's, in a table of the image. Its collector (C453): Cheney's, kcrt-core's, built and in process alike, its roots every word of kc's stack (each frame zeroed when made) and the globals' cells. A transformer in process (C454): its image sealed once, each use a call, no process. What kcrt makes for kc's code made in kc's heap and collected (C455). kc's sealed code freed, the code area's room given back first fit (C456).
- **K1. Step 1 end to end**: kc's driver; the front end (read, check) as
  queries; the passes from checked integers to each back end (Rust:
  a `main` that prints; native: assembly and a runtime that prints);
  the harness.
- **K2 onward**: Ghuloum's steps, each with its tests.
- **The service** (`kayak serve`) once the engine has a client worth
  keeping warm: after K2's first steps.

## Sums in the Rust back end (the user, 2026-11-13; C408)

FX-26's sums and unions are structural: a tag is the sum's, the same tag
in every sum that has it. kc's Rust back end will write each sum type's
enum with explicit discriminants, `#[repr(u64)]`, each tag's a fixed hash
of its name (SipHash, a fixed key): one tag one discriminant everywhere,
so that widening a sum to one with more tags keeps each value's
discriminant (a plain match, or a reinterpretation where the payloads'
layouts agree), and a keyword is a plain `u64`. A collision within one
enum is a duplicate discriminant, which rustc refuses: never silent. Which
enum an arm or a construction is of comes from the term's type, as the
checker found it, never from a tag alone (as kfx's emitter now does, C408).
The native back end keeps Fixpt's own representation of sums.

Since then: kfx's sums are one erased `Sum`, its tag hashed (C409), and
KFX's datatypes stay nominal, Rust's types of their own, the cost of
expanding them into sums measured too high for the checker (C410).

