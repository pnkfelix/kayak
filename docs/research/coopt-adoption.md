# Coopt's runtime for kayak: what to take, what to keep, in what order

Status: approved by the user, 2026-11-06 ("Everything here sounds
right"), with §8's defaults as its answers (§9). Step 2 of
`docs/direction.md` §5.

Sources: Coopt at `~/Dev/LangPlay/coopt` (9283bba; read for this note,
its code as well as its PLAN), kayak at 680e53eab, Fixpt's pin (1311497),
rustc's lang-item sources (rust-lang/rust `main`, read 2026-11-06). Where
a claim rests on memory rather than a source, it says so.

## 1. What this note is for

The user's goals (direction.md §1, §6):
- **G1**: one KFX source that eventually replaces all of Fixpt.
- **G2**: kayak's whole system async and incremental, in Coopt's style:
  the runtime, kayak-rt, and the compiler as tasks and queries.
- The runtime as much Rust as makes sense; **lang items**, as rustc has
  them, so that runtime, standard library and compiler share one
  vocabulary.

This note says what Coopt has (§2), what kayak has (§3), where they meet
and conflict (§4), a design (§5, §6), an order of work (§7), and the
questions only the user can answer (§8).

## 2. Coopt as implemented (not only as planned)

Coopt is about 51,000 lines of Rust in 16 crates, single-threaded but for
worker "jobs", with about 500 tests. M0-M12 are done but M9 (stages 1-5
of 6); M13 (heap images) in part; M14 in part; no native code at all.
Its code is further along than its PLAN reads:

- **Heap** (`coopt-heap`, `coopt-mem`). Fixpt's bloblet object model,
  its header bits identical to Fixpt's; pointer payloads are word
  offsets, so identity is the bits and determinism holds. Not Cheney
  alone, as its crate docs still say: a nursery, an old generation in
  regions (summaries, Yuasa snapshot marking, paced by promotion debt),
  large objects in spans, a card-and-crossing-map barrier through the
  mutation context, places and arenas (recent). `Value<'gc>` branded,
  `Mutation<'gc>` scopes, `Root<K>` handles across awaits, eight
  compile-fail tests of the escapes. Heap images: dump, load, byte
  identical. No eq?-tables yet. Differs from Fixpt in kind numbers, in
  keeping tag `010` free (Fixpt's suffix refs), and in UTF-8 strings
  (Fixpt's are UTF-32).
- **Scheduler** (`coopt-sched`). Tasks are Rust futures polled a slice at
  a time; fuel paid by `cx.charge(n).await`; seven priority classes,
  priority inheritance, demotion; a virtual clock advanced by fuel; each
  task's effect as region sets, and a preempted task's section "open" so
  that interfering tasks wait; an inbox delivered only at steps; seeded
  stress of quanta, choice and delivery. The collector is a task.
- **Engine** (`coopt-engine`). Tier 0 a CEK machine whose code,
  environments and continuation frames are heap objects; nothing on the
  Rust stack between steps (I5); delimited control as primitives over
  heap frames; a run is a task, sliced by fuel. Tier 1 a tree rewrite
  (about 14x fewer steps), swapped in at safepoints behind stamps.
  In-language tasks an FX library (`async.fx`, 1,147 lines).
- **Runners and trace**. One `Runner` trait (sim, replay, live on
  epoll/kqueue), one `Op` enum, one event stream numbered by step, a
  binary log with stream-hash checkpoints.
- **Queries** (`coopt-query`). Salsa-style: typed queries and inputs,
  durability, early cutoff on 128-bit fingerprints, cycle detection,
  every query run a scheduler task at its demander's priority. Nothing
  persisted between runs.
- **Shared vocabulary**: none as a registry. Heap kinds in one table;
  primitive names and arities in one macro table whose numbers are enum
  positions (so code in a heap image is tied to its build); standard
  types in another; names also as string literals in lowering; tests
  check that the tables agree.

## 3. kayak as it is (680e53eab)

- **kayak-rt** (7,900 lines of safe Rust): KFX's values for its Rust, all
  `Copy`, never freed; one-shot suspension as Rust `async fn` (C45,
  `suspend.rs`); FX-26 processes as OS threads (`process.rs`); I/O
  (`world.rs`); a syscall registry. Its executor and its runners are KFX
  (`lib/tasks.kfx`, `lib/runner.kfx`: sim, record, replay); the compiler's
  own I/O goes through the runner. Its only query is `kfx/incr.kfx`'s
  memo file.
- **kayak-jit** (1,900 lines, `unsafe`): the code area, the trampoline
  into Fixpt's native convention, the routines. **kayak-sync** (82,
  `unsafe`): `par`'s shared branches.
- **Tier 1** (`cells/`, 10,500 lines of KFX): Fixpt's cellular machine,
  its heap in Fixpt's format, a Cheney collector at safepoints, prompts,
  marks and continuations (but five routines), Fixpt's scheduler's
  runtime half; tiers 2, 4, 5, 6 (x86_64 and arm64 machine code), no
  tier 3. It self-hosts the compiler (C366): 97 s, 7.5 GB.
- **The IR engine** (`fx26/v*` and others, 3,200 lines; and the REPL on
  it, 5,000): a CEK machine over tier 1's heap, its own primitive table
  by surface name; cannot run the compiler today.
- **The compiler half** (`fx26/cg*`, `regs-*`, `listing`, 6,600 lines):
  FX-26 to Fixpt's cellular words and register code, byte for byte
  Fixpt's listings. Kept (direction.md §7).

## 4. Where Coopt and kayak meet, and where they conflict

1. **Both use Rust futures as tasks.** Coopt's scheduler polls futures
   charged fuel; kayak's KFX already compiles a suspending procedure to
   an `async fn` (C45). So KFX's Rust can be scheduled by Coopt's
   scheduler with no change to how it suspends: what it lacks is fuel
   (Coopt's Rust charges it by hand; kayak's emitter would insert it) and
   the scheduler itself (now a KFX loop).
2. **Machines differ.** Coopt's tier 0 interprets a heap-resident tree;
   kayak's compiler half makes Fixpt's cellular words and machine code.
   The new runtime runs kayak's compiler's output, so its machine is
   Fixpt's cellular machine, taken into Coopt's structure: a task sliced
   by fuel, its stacks where the collector sees them, no Rust frame
   between a capture and its prompt (Coopt's I5, which Fixpt's machine
   keeps too).
3. **Object formats conflict.** Coopt diverged from Fixpt in kind
   numbers, tag `010` and string encoding. kayak's rule is Fixpt's
   formats everywhere (the user, earlier). So Coopt's heap is taken for
   its mechanisms (generations, regions, barriers, branding, images),
   not its format: §8, Q1.
4. **Native code is kayak's to bring.** Coopt has none; kayak's compiler
   half has x86_64 and arm64 code, on tier 1's state today (departures
   1, 5, 6, 9, 10, 11). In the new runtime, native frames must be ones
   the collector and a capture can walk.
5. **Determinism.** Coopt is deterministic by construction (offsets for
   identity, fuel for time, runners for I/O). kayak has the runners in
   KFX, a wall clock, and processes as OS threads; the new runtime takes
   Coopt's single-threaded, seeded model (§8, Q5).

## 5. The proposed runtime

Rust crates, Coopt's arrangement, Fixpt's formats:

| Crate | From | Holds |
| --- | --- | --- |
| `kayak-mem`, `kayak-sys` | Coopt's, as they are | the only `unsafe` (with kayak-jit): reservations, epoll/kqueue, signals |
| `kayak-trace` | Coopt's | one event stream, by step |
| `kayak-sched` | Coopt's | futures as tasks, fuel, priorities, effects, virtual clock, inbox, stress |
| `kayak-runner` | Coopt's | sim, replay, live; replaces `lib/runner.kfx` and `world.rs`'s I/O |
| `kayak-heap` | Coopt's mechanisms, Fixpt's format | nursery, regions, barrier, `Value<'gc>`, `Root`, images; Fixpt's kinds, suffix refs, UTF-32 |
| `kayak-machine` | Fixpt's cellular machine, in Rust | tier 1 as a task over `kayak-heap`; the routines; the words of kayak's compiler |
| `kayak-jit` | kayak's | native code from the compiler half, its frames walkable |
| `kayak-query` | Coopt's | queries as tasks, fingerprints, cutoff; persisted (Coopt does not yet) |
| `kayak-rt` | kayak's | KFX's Rust values; its tasks on `kayak-sched`, its I/O on `kayak-runner` |

The compiler stays KFX (the single source). It becomes async and
incremental by running as queries on `kayak-query` (its forms' checks and
emits, now `incr.kfx`'s memos), in both of its builds: as Rust on
kayak-rt, and as FX-26 on `kayak-machine`.

What is retired as each replacement passes the corpus and the
self-hosting check: `cells/` (tier 1 in KFX), the IR engine
(`fx26/v*`, `vm`, `table`, `words`), `lib/tasks.kfx` and
`lib/runner.kfx`, `process.rs`'s threads. The REPL moves onto the new
runtime.

## 6. Lang items

**rustc's design** (`compiler/rustc_attr_ir/src/lang_items.rs`, about
227 rows; `rustc_resolve/src/late/lang_items.rs`): the compiler owns a
closed list of names, each with a kind (trait, fn, struct, foreign fn)
and a generic count; a library marks its definition `#[lang = "..."]`;
collection over the crate graph makes each unique (a duplicate is fatal)
and its absence an error where it is needed (`require_lang_item`), or for
the two "weak" items (`panic_impl`, `eh_personality`) at link time, when a
downstream crate must supply them. The compiler knows names and kinds;
libraries supply definitions.

**kayak's vocabularies today** (each copy, and whether generated):

| Vocabulary | Copies | Kept by |
| --- | --- | --- |
| standard name to runtime primitive (`+` to `%fx26-add`) | Fixpt's `STANDARD`, generated to `standard.fx`, copied whole | generated, checked |
| object tags, kinds, routines | Fixpt's `layout.rs` to `layout.fx`, copied; kayak-jit's Rust constants tested against it | generated, tested |
| immediates (`#f` is 3, `()` 19, …) | `cells/layout.kfx`, `fx26/vheap.kfx` (no stamp), kayak-jit | by hand, three times |
| primitive numbers and arities | tier 1's `o-*` (kayak's own numbering), the engine's `prim-list`, Fixpt's by position | by hand; a misspelt name fails at run time |
| standard types | Fixpt's `ENTRIES`; kayak's `primitives` string (77 of 350 shared names typed differently, most deliberately), `pair-op-text`, `prelude` | by hand; checked only indirectly (`fixpt-accepts.py`) |
| KFX's Rust functions | `check-defs`' types, `prelude.fx`'s meanings, kayak-rt's `pub fn`s, joined by mangling | by hand |
| the emitter's own calls (`int_lit`, `str_lit`, `rcons`, `make_bloblet`, …) | string literals in `emit.kfx` | nothing: kayak's nearest thing to rustc's compiler-called lang items |
| `%sched` operations, syscalls | Fixpt's `processes.fx`; tier 1 and the engine each implement them; syscall names declared in six files | by hand |

**Proposed**: one table, `lang/items` (its form a question, §8 Q4), of
every item the compiler, the runtime and the standard library refer to
across their boundary. A row: the item's name; its kind (primitive,
runtime function the emitter calls, object kind, immediate, scheduler
operation, syscall, standard type); its FX-26 type or arity; its number
where it has one; where it is defined (Fixpt, so taken from Fixpt's
generated tables; or kayak); and whether it is required. From it,
generated: the machine's primitive table and numbers, kayak-rt's
registry, the immediates, the emitter's helper names. And checked, as
rustc does: every required item defined exactly once, every use of an
item one the table has, each implementation's arity the table's.
Items FX-26 defines come from Fixpt's tables (Fixpt first); kayak's own
(the prelude's `i64` helpers, syscalls, emitter helpers) are kayak's
rows. No new syntax in KFX: the table is data the build reads, so KFX
stays a subset of FX-26. Coopt has no such table, so this is kayak's.

## 7. Order of work

Each step ends with the full check, the self-hosting check among it.

1. **Lang items over what exists** (§6): the table, generated from and
   checked against today's vocabularies, with no runtime change. Small;
   catches drift now; every later step builds on it.
   Done (C367): `lang/items.sexp`, `scripts/lang-items.py`.
2. **Scheduler, trace, runners in Rust** (`kayak-sched`, `kayak-trace`,
   `kayak-runner`), Coopt's taken over with marks as Fixpt's code is
   (`coopt-source: PATH@REV`, a pinned clone, a drift script). KFX's
   tasks and the compiler's I/O moved onto them; the emitter inserting
   fuel. Done for the emitter's async mode (C378-C382): fuel, processes
   as tasks, the standard streams, files, the terminal and programs run
   beside the program all through kayak-sched and kayak-runner.
   `lib/runner.kfx` and `lib/live.kfx` are kept, not retired (revised,
   C383, the user agreeing): they are the compiler's record and replay,
   which run on Fixpt too, and in the async mode they sit above
   kayak-runner, whose I/O their live answers use; they go only if FX-26
   comes to have runners of its own (Fixpt first). `lib/tasks.kfx` stays
   while the FX-26 REPL's loop is built on it; FX-26's processes could
   replace it.
3. **The compiler as queries** (`kayak-query`), replacing `incr.kfx`'s
   memos, persisted between runs; the Salsa-like layer the user asked
   about, now on the shared scheduler.
   Revised (C383, C384, the user choosing): in two stages, so that the
   compiler still runs on Fixpt. First the queries in KFX
   (`lib/query.kfx`), the compiler's memos keyed by what each form reads,
   by name: done (C384). Then a Rust implementation of the same names in
   kayak-rt, forked from Coopt's `coopt-query`, as the prelude's names
   are given in Rust.
4. **`kayak-heap`**: Coopt's collector over Fixpt's format.
5. **`kayak-machine`**: Fixpt's cellular machine in Rust, as a task on
   `kayak-heap`; the corpus against Fixpt; the self-hosting check moved
   to it; `cells/` and the IR engine retired.
6. **Native code on it**: the compiler half's machine code with frames
   the collector and captures walk (departures 1, 5, 6, 9-12); then the
   compiler self-hosted natively, rustc only the bootstrap.

Steps 2-3 make the compiler async and incremental early, in its Rust
build; 4-6 bring the same to the FX-26 build.

## 8. Questions for the user

1. **Formats**: where Coopt and Fixpt differ (kind numbers, tag `010`,
   UTF-8 or UTF-32 strings), Fixpt's, as kayak's rule has been? This note
   assumes so.
2. **Taking Coopt's code**: copied into kayak with `coopt-source:` marks
   and a pinned clone (as Fixpt's code is taken), depended on as crates
   from a pinned clone, or written again from its design?
3. **Order**: lang items first (step 1), or the scheduler first?
4. **The lang-item table's form**: an FX-26 data file the build reads, a
   Rust table (a macro, as rustc's), or KFX? And does it eventually
   replace Fixpt's own `STANDARD`, `ENTRIES` and `PRIMITIVES` (G1), or
   only mirror them until then?
5. **Threads**: Coopt is single-threaded but for jobs; kayak runs FX-26
   processes as OS threads and `par` on scoped threads. Take Coopt's
   model (processes as tasks, parallelism later, its R10 design)?

## 9. The answers (2026-11-06)

The user approved the note as written; its defaults stand:
1. Fixpt's formats where Coopt's differ.
2. Coopt's code copied into kayak with `coopt-source: PATH@REV` marks, from
   a pinned clone. Then (the user, later the same day): forked at will,
   "rather than treat them as 3rd deps to preserve and maintain"; the
   marks say where each came from, and upstream's changes are read, not
   owed (C369). `libc` comes with `coopt-sys`: the platforms' struct
   layouts are what it is for, and kayak may replace it, owning the fork.
3. Lang items first (§7, step 1).
4. The table an s-expression data file the build reads (kayak's reader
   and the Rust side's tests alike); it mirrors Fixpt's tables until the
   single source can replace them (G1).
5. Coopt's single-threaded model: processes as tasks, parallelism later.
