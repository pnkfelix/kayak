# Things to do

Every task thought of and not yet finished is here: what is left of
[`PLAN.md`](PLAN.md), and what was noticed while building something
else. Each says what it is, and for anything deferred, why it was
deferred and where to start, because the reason is the part that goes
stale first.

When a task is finished it moves to [`DONE.md`](DONE.md) under the same
section number, so "§7" means the same thing in both files. Numbers are
never reused. A section partly done appears in both files. A task tried
and abandoned also moves to `DONE.md`, with a short note on what was
explored and why it was dropped.

A change to the language the compiler is written in goes through the
bootstrap in two steps: teach the compiler (`kfx/`) the feature and
regenerate `boot/`, and only then use it in the compiler (`README.md`).

---

## 7. Errors that point at KFX source: what is left

Done is `DONE.md` §7. Left:
- **Atoms have no position**: a message gives the line of the innermost
  list around what is wrong, which may be a line or two before it.
  Positions on atoms would need `sym` and the rest to carry them too.
- **No column**, only a line.
- **Generated Rust carries no `// file:line`.** The generated functions
  keep their KFX names (the mangling is reversible), and `kfx` checks
  before rustc sees anything, so a rustc error in generated code should
  be the compiler's bug. If that changes, a comment per `fn` is cheap.
- **Run-time errors** (`error`, `car` of `nil`) name nothing but their
  message: a Rust backtrace names the generated functions, so the KFX
  procedure, but not the line.

## 13. The checker: what it does not do yet

What it does is `DONE.md` §13. What is left:
- **Effect polymorphism**: a `poly` over effects is refused by the
  checker, though the emitter would erase it.
- **Messages name the definition, not the line**, as §7 says.
- **`spin` for every recursion**: where FX-26 proves termination
  (size-change), KFX cannot. Porting a termination check would let
  KFX procedures be `pure`.
- **What it does not check that rustc does**: nothing known. Each such
  case, when found, belongs in `tests/kfx-stricter` or `kfx-refused`.

## 14. Places as lifetimes

The cost, and what to decide: `docs/research/places.md`.

A Rust lifetime says how long memory stays valid, and in FX-26 what has
a memory lifetime is a *place*, not a region: a region is "a name in
types and effects, erased before anything runs", a place "memory that
data is put in" (`fixpt/docs/fx26.md`, "Regions and places"). The rule:

> a region's Rust lifetime is the lifetime of the place its data lives in;
> a region variable is a lifetime parameter, ranging over places.

- **`letrena r`**: an arena freed when the body ends, `r` its lifetime
  `'r`. The place value `(place r)` is `&'r Arena`, and `rcons p x y` is
  `p.alloc(…)`. FX-26's escape check on `letrena` is what makes `'r`
  well-formed.
- **The heap** is a place too (FX-26's `heap`): `'static` while it is
  leaked; a branded `'gc` if a collector comes (§16), as coopt's
  `Value<'gc>` is.
- **`letregion r`** makes no place, so `r` is the heap's lifetime. Rust
  cannot make a fresh lifetime in a block anyway, short of a closure's
  brand.
- **`(r region p)`**, a region in a place, is `'p`: its data is in `p`'s
  memory.
- **Frozen data**: at `(const p)` or `(acyclic p)`, `'p`; at `const`,
  `'static`.
- **`letreap`** is excluded: its objects move, and a `&'p T` cannot
  point at what moves.
- **Region polymorphism** is lifetime polymorphism: a procedure in `(r
  region)` may be called at a region in an arena, so it is `fn f<'r>`,
  and `List<'r, T> = Option<&'r Cons<'r, T>>`. Two regions in one place
  are one lifetime, which is sound: Rust needs the memory to outlive the
  reference, and that two regions are distinct is FX's concern.

Effects stay FX's: lifetimes check memory, not `(read r)` or `(write r)`
(§18).

This is what makes memory reclaimable without a collector. Today's
`'static` everywhere is the rule's degenerate case: every region's place
is the heap.

## 15. Flat data: what is left

Flat arrays of `i64` and references are done (`DONE.md` §15). Left:
- other element types (`f64` first, which needs real literals and
  arithmetic in KFX), through a `(flatlayout T)` as a Rust trait;
- flat products and arrays reached through `blobref`, as `#[repr(C)]`
  structs of cells: `extract` a field's `&Cell`, `blobref-index` an
  element's; not `blobref-at` or `blobref+`, which step by bytes;
- flat sums: a variant written in place (`blobref-variant!`) has no safe
  Rust form over a `Cell`, short of a tag cell and a union;
- `distinct` region binders, checked as Fixpt does;
- masking, with `letregion` (§14).

## 16. Escapes, then FX-26 at large

Escapes are done (`DONE.md`, C18). Left:

- marks (`with-mark`, `first-mark`), locations in the dynamic context;
- `proj` of `abort-current-continuation`, as Fixpt's `control/` tests
  write it;
- a `poly` over effects, so that a procedure taking a procedure that may
  abort need not say on which region.
- Then the broader FX-26: a runtime with a collector, bignums, unions,
  delimited control, modules. Whether that keeps targeting Rust or
  something else is open (`PLAN.md` §5).

## 17. Small things noticed

- **K1 gaps a probe found, left** (2026-10-11):
  - a constant whose value fails and is never named does not fail in Rust
    (FX-26 makes it where it is defined; KFX at its first use, C21): a
    program FX-26 stops, KFX's Rust runs on;
  - an `i64` expected of a call reaches its arguments' calls in KFX, not
    in FX-26: `(define b i64 (car (cons 1 nil)))` and `(get (new 5))` are
    KFX's, not FX-26's (the values are the same). FX-26 checks a call
    argument against its parameter when an earlier argument fixed its
    type (`tests/kfx/families.kfx`), and finds it alone when only the
    result expected did; KFX's rule is a `list` alone;
  - a `list` made under `with` or `letregion` is at `acyclic` to KFX and
    at a fresh region to FX-26, which then refuses it where `acyclic` is
    expected; KFX would need FX-26's masking of fresh regions;
  - a list made where nothing says its region is at `%fresh`, one region
    for them all (FX-26 makes one for each): two such lists are joined
    where FX-26's may not be;

- **K1 differences left** (C36): a pair whose tail is a pair, found by
  itself, given where a list is; a literal in such a pair's tail.
- **A `cons`'s or `rcons`'s value is a pair to FX-26**, `(pairof T (listof
  T r) r)`, which it will not give where a procedure value wants a list
  (`(f (cons 1 (cons 2 nil)))`, `f` a local lambda): KFX, whose `cons` is
  a list's, accepts it (K1). Nor does FX-26 type a literal in a pair
  constructor at what the list's type says (`(the i64 (car (cons 1
  nil)))`, `(rcons r k (rcons r 1 nil))`); KFX does.
- **Bloblets' rest** (C31, C32):
  frozen bloblets; width; flat sums without a key (they need symbols);
  families over `flat`.
- **`letrena`'s rest** (C30, C34, C40): an alias body's `(place r)` of a
  plain region parameter (accepted; FX-26 refuses it);
  `rmake-array`, `rmake-icell` (KFX has no arrays of values, nor I-cells);
  datatypes over regions used in a `letrena` (their regions erased); 
- **`null?` of an argument whose element nothing says**, other than a name
  or an `error` (`(null? (car (cons nil nil)))`), is refused; FX-26, whose
  `null?`'s binder is its argument's whole type, accepts it (C29).
- **`(define x e)` with no type** is refused; FX-26 allows it.
- **A looping procedure's parameters are all `mut`**, even those no self
  tail call changes (`count-up`'s `n`). Only the changed ones need be.
- **Non-tail recursion is still as deep as its data**: a walk that builds
  a list on the way back (`string-chars-from`) needs `with_big_stack`.
- **Tail calls between procedures** of a `define-rec` or a `letrec` are
  calls, and grow the stack: only a procedure's calls of itself loop (a
  top-level one's, and a `letrec`'s closure's).
- **`proj`** (C40) is checked, its arguments given in its binders' order;
  in Rust, inference finds the generics (a turbofish where it cannot,
  C29). Not yet of a primitive (`abort-current-continuation`, above).
- **A `lambda` of more than four parameters** has no `lambdaN` in
  `kayak-rt`.
- **Warnings in generated code** are all allowed. Unused bindings (a
  `tagcase` arm ignoring fields) are the common ones; whether the emitter
  should avoid them is open.
- **`(the T e)`** is written as `e`. Rust could check `T` too:
  `{ let v: T = e; v }`.
- **Allocation** is `Box::leak`, one allocation per object; an arena
  later, and §14.
- **`case` on an `i64`** is refused by FX-26, whose `case` compares integer
  keys as `int`s; KFX's `case` is on characters, and an `i64` dispatch is a
  `cond`. A request for Fixpt, if it matters: `case` on fixed-width keys.
- **Shifts**: `i64-shl` and `i64-shr` take their count as an `int`; KFX
  has none, so they are not in the corpus. A prelude `i64-shl-i64`, when
  wanted.

## 19. Modules

The first step is done (`DONE.md`, C19): top-level modules, `with`,
`select`. Next, by what Fixpt's `modules/` programs use:
- a module's datatype whose tag another has: the checker could leave the
  datatype as a fact at each `tagcase`, for the emitter;
- a module's types with parameters;
- items naming later items, as FX-26's `letrec*` allows;
- `the` sealing a module defined at the top level to fewer values or
  abstract types: a second `mod` re-exporting;
- `load-module` of a file, the file a Rust module;
- modules as values beyond C24's parameters: made where given, sealed
  with `the`, passed on; dependent parameters.

Files are units in order, with no interfaces: every name is visible
everywhere after its definition, and every Rust item is `pub`. FX-26's
modules (`module`, `load-module`, `moduleof`, `with`, `select`) would give
interfaces and abstraction; they are first-class values, so their mapping
needs designing, perhaps a `moduleof` as a Rust module's `pub` surface,
or a trait.

## 20. Performance: what the benchmarks showed (2026-10-11)

`scripts/bench.py` times Fixpt's benchmarks and their ports
(`bench/ports/`) as kfx's Rust and on Fixpt's machines. What they showed:

- ~~**Procedures over regions take no lifetimes**~~: done, C42. Left:
  a type variable at a type over such a region is refused, its Rust type
  parameter being `'static`; dropping the bound needs to know which
  procedures need it (a datatype's constructor, `alloc`, and so whatever
  calls them at the type), a fixed point over the call graph.
- ~~**Heap allocation is a `malloc` each**~~: the user's ruling, safe
  typed arenas only; done, C43, where every type is `'static`. Left: a
  closure is still a `Box` each (its type is no `'static` one to find an
  arena by), and so is the heap's data inside a `letrena` or a procedure
  over a region or place.
- **A closure that does not escape is leaked all the same**: `lambdaN` for
  every lambda, one each round in `closures`. One the checker finds stays
  within its scope could be a local, borrowed as `&dyn Fn`.
- **`rlambda`'s closure is the heap's**, not its region's: a `Box<dyn Fn>`
  in the region's arena would be freed with it.
- **The arena's cells are `OnceCell`s**: a flag tested and set at each
  allocation; the rest of `lists-region`'s gap to a bump. Safe, or the
  same decision as the heap's.
- **`int` is twice `i64`** (fib 4.2 ms to 1.2): `Int`'s small case could be
  cheaper (a smaller representation, its overflow a branch rustc
  predicts).
- ~~**Rust's keywords as names**~~: done, C41 (`r#loop`).
- **Benchmarks whose inputs rustc sees are computed at compile time**
  (`helpers`, `loop`: 0.0 ms): a bench mode whose literals the emitter
  gives `std::hint::black_box`.
- **Tail calls between procedures grow the stack** (section 17).
- Fixpt runs the `i64` ports slower than the `int` originals on every
  machine: Fixpt's, if the user would tell its team.

## 21. FX-26 as well, on an asynchronous runtime (`PLAN.md` §6; the user, 2026-10-12)

The next goal. Phase A, in order (`PLAN.md` §6.5):
- ~~**The research note**~~: done, `docs/research/async-rust.md`, C45,
  with the prototype `kayak-rt/src/suspend.rs`.
- ~~**Suspension in the checker and emitter**~~: done (C45): `(comefrom r)`,
  `abort-with-continuation`, `async fn` by effect, suspending prompts and
  procedure values; ping-pong and promises in the corpus.
- ~~**A library of tasks in KFX**~~: done, `lib/tasks.kfx` (C47). Next
  in it: channels; nurseries and cancel scopes (continuation marks, which
  KFX takes from FX-26 when they are needed); Fixpt's `cancel.fx` and
  `context.fx` as corpus programs.
- **Effect polymorphism's cost** (C46): a box per call of an effect-poly
  procedure and per closure given one; monomorphise by colour if it is
  measured to matter. A datatype over an effect is refused.
- **Boxing only what recursion needs**: every suspending call is boxed
  now; a call outside its callee's recursive group need not be.
- **Record and replay**: done in KFX (C48), live and joined to the loop
  (C51, `lib/world.kfx`, `lib/live.kfx`), the log as text. Seeded
  delays and faults, and the determinism check: done (C53,
  `lib/sim.kfx`). Left: I/O that does not block, answered as it
  arrives; the arguments in the log's header; more kinds of fault.
- ~~**I/O and bytes**~~: done (C49), as Fixpt 17ac0c5 has them. Left: a
  bytevector at a letrena's region; `rmake-bytevector` when Fixpt has it.
- **Corpus programs**, compared with Fixpt.
- **Incremental compiling** (Q5, answered; `docs/research/queries.md`):
  `Emit(form)` and `Check(form)` kept between compiles (C54, C55). Next:
  checks keyed by the names they read, not the whole chain; the
  compiler's identity in the memo file, so that the driver, not its
  caller, keeps one compiler's memos from another's; queries as a library
  for KFX programs (`lib/query.kfx`); query runs as tasks when the
  compiler has a reason to wait.

Phase B, the compiler for two languages:
- **The front end shared**: the checker grown to FX-26's, KFX's
  restrictions a pass of their own.

Phase C, FX-26's runtime:
- **An IR** for FX-26 at large.
- **The engine, in KFX** (K7): a heap with a collector, its stack in the
  heap, composable continuations; an interpreter of the IR, kept as the
  reference (Q8); its tasks on the one scheduler.
- **K6**: Fixpt's programs on it, compared with Fixpt.
- **The second self-hosting**: the compiler on kayak's runtime, built
  into an image, its output compared with the Rust build's.

Phase D: more code generators (compiled words, native code), `unsafe`
in a few runtime crates alone (Q7).

Throughout: where KFX or FX-26 lacks what runtime code in KFX needs,
raise it with the user first (`PLAN.md` K7, §8).

## 18. Research: regions as exclusive borrows (GhostCell tokens)

Places are lifetimes (§14), but regions are something else: permission
to touch data, which must not alias. Lucassen's anti-aliasing rule in
FX-87 (the user's observation, 2026-10-11) is the `&mut` discipline. FX-87's
checker (`GiffordHistory/mit-psrg-fx/fx87/old-impl/type-check.lisp`,
`no-aliasing?`, from `desc-of-proj`) refused:
1. a `proj` whose region arguments are not pairwise distinct, regions
   free in effect arguments included, the immutable region excepted
   ("Region conflict on arguments to PROJ"): Rust's refusal of
   `f(&mut t, &mut t)`;
2. a `proj` whose region argument is free in the `poly` instantiated
   ("Region conflict between arguments and POLY type"): Rust's refusal
   to pass `&mut t` to a closure that has captured `&mut t`.

**FX-26 does not have the rule.** `fixpt/docs/fx26.md` follows PLDI '89,
which assumes "the anti-aliasing rule of FX-87 is eliminated": "So do
we, until something needs it". Probed at `35b071f`: `(proj swap @c @c)`,
`swap` reading and writing both its regions, is accepted, and runs.

The mapping this would allow, after GhostCell (Yanovski, Dang, Jung and
Dreyer, ICFP 2021): each region a branded `GhostToken<'r>` (`'r` a brand,
invariant, not a memory lifetime), `(ref T r)` a `GhostCell<'r, T>`, a
procedure with `(read r)` taking `&GhostToken<'r>` and with `(write r)`
taking `&mut GhostToken<'r>`. rustc would then check FX's region effects
as borrows. It needs the rule: without it `swap` at `@c @c` borrows
`@c`'s token mutably twice. Rust's rule is finer than Lucassen's (shared
borrows may alias; only `&mut` must be unique), so with Lucassen's KFX
refuses a little more than Rust would, which K1 allows.

To decide (the user's): KFX reinstates the rule itself, in its checker
(§13), or FX-26 does, perhaps as an opt-in, so that stage 0 checks it.

**Deferred** (the user, 2026-10-11): regions, and `&mut`, wait until
this is settled; KFX avoids both meanwhile.

**Fixpt's answer** (2026-10-11, `fixpt/docs/research/region-anti-aliasing.md`,
on Fixpt's `origin/main`, past the pin). A survey found the full rule
costly (274 places would break, mostly `append` and `reverse`'s two-region
types) and Rust's own (aliasing XOR mutation) nearly free (9). Built, in
`31eebfb`: an opt-in `distinct` on a region binder, `(poly ((r1 region
distinct) (r2 region)) …)`, refused at any instantiation that does not give
`r1` a region provably apart from the other binders' and from the
procedure's type's. Proposed next there: `blobref-split`, flat data's
`split_at_mut`, whose pieces refuse `blobref+`.

**What kayak concluded** (2026-10-11, `CLAUDES_CHOICES.md` C9): since
every KFX value is `Copy`, a reference can always be duplicated, so even
at a `distinct` region two copies may alias, and `&mut` is out of reach
without an affine discipline. Mutable data is cells. What `distinct`
would still give is the checker's: effects on distinct regions commute.
What Fixpt's note says it gives a Rust transpiler: `&mut` between
parameters at `distinct` regions, and for flat data (§15) down to
`split_at_mut`; not `&mut` into heap data,
which may share within a region and would need a substructural
discipline. KFX's heap data is all frozen (`acyclic`), so `&` covers it,
as today. So the first `&mut` is flat data: a written `distinct` region
`&'r mut`, a read one `&'r`, `blobref-split` `split_at_mut`. That needs
no GhostCell tokens: Fixpt checks the distinctness, in stage 0.

The hard part either way is effect polymorphism: a latent effect that is
a variable, or a `maxeff` over regions, needs a procedure generic over
sets of tokens. A research note first: which of FX's rules carry over,
and what the generated code looks like for the compiler's own source.

## 22. The front end, shared (`docs/research/front-end.md`)

- ~~Stage 1, the cheap names~~ (C56): standard operations, symbols,
  arrays of values. Left of it: `set-car!` and `set-cdr!` (KFX's pairs are
  immutable in Rust), `apply` and variadic procedures (stage 5),
  `string-hash` (Fixpt's hash, to match).
- Stage 2: the restriction pass split off the checker.
- Stage 3: unions of shapes, `datum`: not for KFX (C57); the checker
  takes them with phase C's engine, the restriction pass refusing them.
- Stage 4: modules' remaining forms.
- **First-class polymorphism** (the user asked why KFX has none,
  2026-10-12): the checker's polymorphism is ML's, on named definitions
  only; a `poly` type of its own (a local's, an argument's, a recursive
  type's) touches every function over types, with FX-26's bidirectional
  rules. For the engine; refused for Rust (Rust has no type-polymorphic
  closure; region-only polymorphism could be `for<'a>` closures, if KFX
  ever wants it).
- Stage 5: higher kinds, variadic procedures, sizes.

## 23. FX-26's engine (`docs/research/engine.md`, C58)

- ~~The core~~, ~~the corpus's forms but bloblets~~, ~~Fixpt's
  programs compared~~ (C58, C59): 49 of 53 corpus programs, 97 of the 105
  Fixpt programs kfx accepts, agree; none differs.
- Bloblets (byte layouts, blobrefs), the last of what kfx accepts.
- FX-26 beyond KFX, the engine's reason: the checker given an FX-26 mode
  (the engine's), in which KFX's restrictions are off and FX-26's typing
  of the control primitives (`composable`, `call-with-composable-
  continuation` as a poly primitive with `proj`) and then unions are its
  own; the restrictions a pass for Rust after it. A design note first.
- ~~The collector~~ (C64, in KFX); ~~the second self-hosting~~ (C66: the
  compiler on the engine makes `boot/src/kfx/`, 278 s); ~~a REPL~~ (C65).
  Next: the engine's speed (thread-local lookups, dispatch); its tasks on
  the scheduler; bloblets; FX-26 beyond KFX (pairof, datum, sizes,
  variadics, marks); a GC for kayak-rt itself, a research note.

## 24. A uniform value beside unboxed generics (`docs/research/uniform-values.md`)

The user's proposal (2026-10-12), agreed to follow the checker's current
growth: unions, `datum` and first-class `poly` for KFX, as a `Value` where
types are not known and Rust's generics, unboxed, where they are;
coercions at the checker's subsumption points, views for lists. Reverses
C57 when done.
- **Done, step 1 (C75)**: unions, `typecase`, the shape tests, `false`,
  `pairof` of a list: `Value`, boxed and unboxed at the checker's facts.
- **Done, step 2 (C77)**: `datum`; pairs of a union's as `Value`s; `car`
  and `cdr` of a union; `quote` of any datum KFX reads; a typed list given
  as a datum by a view; `typecase` as FX-26 writes it.
- **Done (C79)**: quasiquote; quotes interned in the engine.
- **Done (C80)**: the reader's `#(...)` and dotted lists, in quotes.
- **Done (C82)**: a procedure given where one of a union's parameters or
  result is wanted, wrapped.
- **Left of step 2**: coercions more than one level down (C78, C82: a
  list of lists; a wrapper's procedure argument); rustc refuses them, not
  kfx; `datum-int?`;
  `bytevector` in `datum`; a local named `nil`, which FX-26 allows and the
  checker's naming rule refuses (`datum/quote-shadowed.fx`); `no-pair`.
- **Step 3**: first-class `poly`, a generic instantiated at `Value`.
- **Done (C84)**: datatypes as union members; the first union in the
  compiler's own code (`find-family`). More lookups as unions, one at a
  time (C83, C84).

## 25. FX-26 beyond KFX: what is left, by subsystem (2026-10-12, after C68–C74)

The round of the checker's growth for the engine done: pairs and `nil`,
narrowing through `not`/`and`/`or`, `eq?`, `set-car!`, variadics, marks,
`case` on symbols (KFX's too), lambdas heard last (KFX's too). Beyond KFX
the engine agrees on 31 of Fixpt's programs, none differs; its checker
refuses 165. What is left are subsystems, each its own design note first:
- **Modules, FX-26's** (46 programs): first-class modules, parameterised
  module types, `extend`, `load-module`; KFX's are a Rust-shaped subset.
- **Sizes** (19, and lemmas' 3): `(nat k)`, `nlist`, a test's
  propositions in its type, `proves`; a solver.
- **Higher kinds** (12).
- **First-class `poly`** (9, and recursive types through it): TODO §22.
- **`datum`** (9): with C75's uniform values for KFX, and `letfreeze`,
  `const`, quasiquote, vectors.
- **Smaller**: bloblets in the engine.
- **A K1 gap, reduced (C81, C88)**: `(define f (subr pure (string strs)
  strs) (lambda (p c) (reverse (cons p c))))`, `strs` a list at `acyclic`:
  FX-26 refuses it, KFX accepts. FX-26's `cons` is `(subr (alloc r) (t1
  t2) (pairof t1 t2 r))`, its `r` free, made fresh where the call is found
  by itself (`default_regions`), a pair whose tail is a `(listof T
  acyclic)` being that list wherever one is wanted; KFX's is `(subr (t
  (listof t r)) (listof t r))`, the pair's region its tail's. Take
  FX-26's typing of `cons` (the Rust unchanged: such a pair is the list),
  checked against the compiler's every `cons`; the fixpoint check (Fixpt
  checking the compiler's own text) catches any in the compiler meanwhile.

## 26. The compiler in modules (the user's, 2026-10-12; C91, C92)

Done: `termination`, `restrictions`, `incremental`, `expressions`,
`emitter`, `writing`; memos per item inside a module, for checking and
emitting; a warm compile 0.02 s dearer than with no modules.
- Next by what each would hide: check-defs.kfx (98 of 127 names), check.kfx
  (82 of 215: a large interface), read.kfx (22), names.kfx (7). base.kfx
  and compile.kfx are mostly interface.
- The 0.02 s left: each module's scope (`module-prep`) made each compile;
  keep it by the module's text and the chain before it, as items are.
- Comments between a module's items: done (C93), strings again, as FX-26
  now takes them, and comments in the Rust.

## 27. The REPL (the user's, 2026-10-13; `docs/research/repl.md`, C94)

Stage 1 done: entries run while the next is typed, each a task on the
machine (fuel, parked registers), a live line rewritten in place, `%k`,
waiting on a pending `%k`, ^C, effects deciding which run together;
scripted sessions (`tests/repl/`, `scripts/repl-sessions.py`).
Stages 2 and 3 done (C95): editing, history, Tab; Fixpt's commands.
- Fixpt's commands not yet: `,fits TYPE`, `,returns TYPE`, `,help` as a
  hole inside a form, `,disassemble`, `,list-outdated` and
  `,rerun-outdated` (redefinition at another type), `,redefine b|r`.
- Checking as you type (C96), hints at a hole and the underline (C97)
  done; type variables said by their binders' names (C98). Left:
  speculating `,type E`'s and `,time E`'s forms.
- The machine on a thread of its own: done (C103). Left on the front's
  thread: the check of a large draft, which runs to its end before the
  next key is drawn (a third thread for speculation, or the check in
  slices).
- Bignums in the engine leak (C99): KFX's `int`s are never freed, so each
  step on a bignum leaks its operands' conversions (500 MB for `(fact
  3000)`). Arithmetic on the limbs in the heap, or a free in `kayak-rt`.
- **Processes, the rest (C101, C102):** the engine's (it says "cannot
  yet"). The REPL's front and machine are two threads the Rust makes
  (C103), not KFX processes, since the checker's refusals must be caught;
  once KFX has FX-26's conditions, a `spawn`. Entries as processes of
  their own would need the machine's state out of globals and its heap
  split by region. Measure `par`'s
  threads against running in turn, and a pool if a thread a branch costs
  too much.
- **Conditions, what is left (C105–C107):** `current-marks` and
  `marks-of` (a `with-mark` in tail position replacing its key's mark);
  a procedure deeper in a mark's value than the key's own, lifted; a poly
  type with type or effect binders (rank 2 beyond regions); a `let`'s
  `nil` generalized, as FX-26's is; `load-module` in the REPL; Fixpt's
  other built-in modules (`fx26:processes.fx`, ...).
- **The REPL in processes (C111), what is left**: the draft's check
  still holds up keys, the front waiting for its worker (spawning the
  front's checks off, their outcomes as messages, would free it); the
  machine's failures not yet values (`exit`, a stack overflow, the
  heap's invariant) end the REPL, as nothing catches them in the program's
  own computation (the engine in a place, a monitored machine, would);
  whether to drop C103's REPL (`rust-repl`) once compared enough, the
  user's call.
- **KFX: a procedure polymorphic in a place alone** (its place only its
  mailbox's) leaves the Rust type parameter for the place unconstrained
  (rustc: "cannot infer type"); found by C111's relay.
- **From Fixpt 0a0e277 (C108), not yet in KFX:** `race` (cancellable
  branches; a failed one lost), the `sendable` kind; the engine's
  processes and monitors.
- **The emitter's memos gave wrong Rust** (2026-10-14; again the same day,
  `target/memo-bug2/`, while C107 was built: generation 2 of the bootstrap
  without `void-call`'s `match … {}`, a fresh cache right): a memo file made
  while the checker changed (today's intermediate compilers) held, under a
  key today's sources make, Rust without the facts of `synth` (no `match
  … {}` for a `void` call, no turbofish). A fresh cache is right. Kept:
  `target/memo-bug/` (the memo file, the commit, the sources' diff).
  Again four times on 2026-10-16 while C119 was built (generation 1 or 2
  failing, a fresh cache right each time): common now when the checker
  changes; worth finding before it costs more.
- Inside a global's value, Fixpt does not count a local lambda's reads of
  globals against its declared effect; KFX does, so refuses
  `pselect-stop.fx` and the like (C101).
- A skimmer in KFX, after Coopt's (repl.md, Coopt's skimmer): forms and
  the names they define, found lexically; for `incr.kfx` (measure first
  whether reading costs in a warm compile), and the REPL's names before
  checking.
- Stage 4: `load-module` in the REPL; an entry's output beside its line.
- Stage 5: behaviors (FrTime), a design first.
- A refusal is caught by the Rust as a failure, so the checker's state
  after one is whatever the failed check left (as before C94).
- kfx: a file `f.kfx` whose module is named `f` makes ambiguous Rust
  (`pub use f::*` beside `f::f`); refuse it, or name the file's Rust
  module apart.
- kfx's checker fails (`car: nil`), not refuses with words, on
  `(listof int)`, a `listof` with no region.

- The KFX REPL (C111) starts some 7 ms slower than C103's (17 against 10 ms for an empty plain session), cause not found (not the per-thread constants); and some 55 µs more per entry, its worker and relay threads.

## 28. A quick check (the user, 2026-10-14: first of three)

Done (C112): `scripts/check.sh --quick`, 49 s. Left:
- A cheaper K1 for the compiler (`fixpt check` of its source rather than
  running it on Fixpt), timed against stage 0.
- Why stage 0 is so slow on Fixpt's fastest machine (200 s, against 1 to
  1.6 s as kfx's Rust; its output says many procedures are "not in the
  native convention yet"): worth a note to Fixpt, if it is theirs.

## 29. KFX-level backtraces on the engine (the user, 2026-10-14: second)

Done (C113): a failure on the engine says its procedures, innermost
first, in the REPL and in `kayak-fx26 --run` (status 70 now, as Fixpt's).
Left:
- The compiler loaded into the engine's REPL (`load-module` in the REPL,
  §27), to call its procedures on small inputs.
- A panic of the compiled Rust naming the KFX definition it came from.
- A test of `kayak-fx26 --run`'s failure (its stderr and status): the
  REPL's session (`tests/repl/backtrace.keys`) is the only one now.

## 30. The checker for FX-26 at large (the user, 2026-10-14: third; PLAN §9)

PLAN §6.5 step 8, now first among the steps to kayak being the reference.
The measure (C114): `scripts/compare-checkers.py`; the plan,
`docs/research/fx26-checker.md`. That day, of 357 programs Fixpt
accepts: 22 the same, 52 alike, 110 different, 173 refused by kayak.
1. Printing as Fixpt prints, in `show-ty`: effects sorted, abbreviations
   by name, `poly` as written, module types (and `--check` writing
   module definitions).
2. The restriction pass split off the checker (`front-end.md` stage 2).
3. FX-26's subsystems: modules, sizes, processes, higher kinds, bloblets,
   recursive types, unions; each a note first.
4. Refusals' messages and places compared, for what both refuse.
- `higher-kinds/proj-await.fx`: refused by `--check` alone, its constant's
  value checked again by the printer (a `proj` there).
- Of Fixpt's 143 front-end files, kfx accepts 7 as they are; 121 are
  refused first at a module file's form (`(module ...)`, its `hide`).

## 31. FX-26 to native code, as Fixpt's ladder (the user, 2026-10-14; PLAN §9, the main line)

`docs/research/native.md`: tiers 0 to 6 (the engine; cells interpreted;
the hand machine; stencils from kayak-rt; cells compiled; register code;
the native convention), Fixpt's formats taken whole, code bloblets in a
double-mapped code area as Fixpt's (code beside its fields), `unsafe` in one crate (`kayak-jit`).
1. Module files in KFX; Fixpt's front end's 143 files surveyed again,
   to choose where stage A comes from (A: Fixpt's front end through KFX;
   B: Fixpt's compilers ported onto kayak's checker).
   **At Fixpt `7641962` (C119)**: 54 of 150 accepted. Next, by files:
   `wcell`, `tword`, `wglobal`, the cellular machine's types Fixpt's
   compilers in FX-26 use (14; how kayak's tiers stand to Fixpt's cells
   decides it); `atom is made from atom` in `parser.fx`'s module order
   (4); `table.fx`'s `table-has?`, a `v` where a list is expected (3, a
   KFX checker gap, Fixpt accepts it); `load-input` of a file that is one
   `lambda` (`regcode*`, `compile*`, `eval*`, `native.fx`); the eager
   reader's own states (2, expected); a nested load's path joined to an
   absolute one (2, a bug). And `call-with-oneshot-continuation` whose
   procedure ends in an abort to its tag, as C45's form (C118).
2. The layout table for KFX; the heap in Fixpt's format; tier 1.
3. `kayak-jit`; the arm64 encoder in KFX; tier 2. **Begun (C116,
   2026-10-15)**: Fixpt's §97 operations in KFX, the code area, the
   trampoline, `tests/kfx/made-code.kfx`. Next, toward Fixpt's
   `made-code/frames.fx` and `generated.fx` on kayak:
   - ~~the routines `poll`, `overflow`, `trap`, `cons`, `foreign`;
     lists and fx procedures crossing; `define-convention`,
     `native-convention`~~ (C116: Fixpt's `frames.fx`, `declared.fx`);
   - ~~`load-input` of a built-in file; `fx26:arm64.fx`,
     `fx26:convention.fx` as they are~~ (`made-encoded`,
     `made-generated`); ~~`allocated.fx`, `linked.fx`~~ (`made-allocated`,
     `made-linked`); `load-input` of a program's file; Fixpt's masking of
     reads of what a procedure made itself (`allocated.fx`'s `pack-code`);
   - ~~fields beside code, written through; the `bloblet` routine; the
     write barrier's cards~~;
   - a bytevector, shared rather than copied (a header of Fixpt's before
     its bytes), so that native code and KFX see one heap's data: the way
     to the heap a compiled program runs in (`native.md` decision 6);
   - more than three parameters; integers past a fixnum (Fixpt's bignum);
     a procedure native code gives back.
4. Tiers 4, 5, 6; then 3.
5. Twobit's passes; the compiler compiled into its own heap.
