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
- ~~Why stage 0 is so slow on Fixpt's fastest machine~~ Done in Fixpt
  (`4ab77e92`, C141): about 610 s to 25 s a run.

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
   by name (**done**, C164, as written; C165, base types by the latest),
   `poly` as written, module types (**done** for most, C165: datatypes,
   nested and loaded modules, a module's own effect left), a fresh
   region's name (**done**, C166).
2. The restriction pass split off the checker (`front-end.md` stage 2).
   **Begun** (C154): `kfx-restricted` notes, kfx's `restrict` refuses;
   moved so far: a constant's effects (C154), reals written past KFX's
   way (C155); FX-26's names the checker had not (C156); sizes: facts
   from tests (C157), `(nat s)`, `nlist`, size binders (C158), poly types
   of types and sizes, rank 2 (C159), the confirm forms (C160); `const`
   regions and `letfreeze` (C161); specifications (C162); what a test
   proves (C163); a bloblet's width (C167); `cwcc` (C168); `race`
   (C169); first-class modules (C170, C171, C172); higher kinds (C173);
   recursive types, `mu` (C174); flat arrays of any flat, flat sums with
   no key, arrays to the end of the suffix (C175).
   10 of the 433 programs Fixpt accepts still refused (2026-10-25, C201);
   62 differ; 361 the same.
   The user (2026-10-25), on C183: Fixpt taking fixed-width integers as
   `int`s was probably a mistake, an ABI matter (an `i64` boxed where an
   `int` is wanted); perhaps `(raw i32)` apart from `i32`. Kayak follows
   Fixpt's checker for now, KFX refusing each use; if Fixpt changes it,
   the one rule in `fits` goes. Paths narrowed: **done** (C196);
   `cons` as a value at Fixpt's type: **done** (C197).
   Flaky: `processes/pselect-deadline.fx` under `cells-programs.py`'s
   `--slice 2` reported a deadlock Fixpt's run has not, twice in the full
   check (C184, C187); never in 300 runs alone, idle or with every core
   busy. Next among them: redefinition at another
   type, re-run and broken (2; C180); `conv` binders (C181). Of flats left: the kind `flat`
   (`flat-kind.fx`), `narrayof` (`narrayof-lengths.fx`). Of modules left: `load-layers.fx` (an effect
   read from an item made before it, in a loaded file); printing, where
   Fixpt names a shared type by the latest abbreviation or description of
   it (`include-types.fx`), a module's own making's effect, a family's
   description `(dlambda ...)`, loads' names `%shared:PATH` (Fixpt's
   absolute).
   Left: `nat?` (kfx defines its own), `certify-nat`; sizes' rest
   (`narrayof`, fixed widths); `(const p)` into a place
   not the heap; then `poly` over other kinds, `const` regions with
   `letfreeze`, termination, conventions, `cwcc`.
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
1. ~~Module files in KFX; Fixpt's front end's files surveyed again, to
   choose where stage A comes from~~: **B** (C120, C121), Fixpt's
   compilers ported onto kayak's checker; Fixpt followed upstream by the
   pin (`built-in.py`, `fixpt-drift.py`, words compared). What A left
   open, below, is for whatever else of Fixpt's we take as written.
   **At Fixpt `7641962` (C119)**: 54 of 150 accepted. Fixpt's compilers
   in FX-26, which use `wcell`, `tword`, `wglobal` (16 files), are
   strategy B (the user, 2026-10-16; C120): ported onto kayak's checker,
   not taken through KFX. Next, by files: `atom is made from atom` in `parser.fx`'s module order
   (4); `table.fx` (3, now at `rehash-array`: KFX's `spin` for a call of
   a procedure over a type variable, C25, more cautious than FX-26's;
   TODO §30's kind); **the functor files** (62, looked at 2026-10-16,
   not taken on): each a `let*` of loads around a `lambda` taking modules
   by signature and giving a module that includes them, applied once each
   in `conductor.fx`'s one `let*`. KFX could expand each application
   statically (known modules, applied once), as C117 does a file's
   regions; a hand-made probe of `check-types.fx` so stopped first in
   `table.fx`. The cost is less the form than what follows it: the
   bodies are Fixpt's checker, and every place KFX's checker is more
   cautious than Fixpt's (TODO §30) stops one. Strategy B is the likelier
   road for much of it (the user: "strategy B is inevitable in some
   cases"); the eager
   reader's own states (2, expected); a nested load's path joined to an
   absolute one (2, fixed: C121, `input-nested`). (kfx's stack overflow on
   `check-holds-types.fx` and `check-modules-types.fx`: fixed, C119; 56
   accepted then.) And `call-with-oneshot-continuation` whose
   procedure ends in an abort to its tag, as C45's form (C118).
2. The layout table for KFX (**done**, C121: Fixpt's `layout.fx` built
   in, `kayak-jit`'s constants tested against it); the heap in Fixpt's
   format and tier 1, cells interpreted (**begun**, C122: `cells/`, on
   the words Fixpt's compiler lists; at Fixpt 1925bca, 194 of Fixpt's
   programs agree, 0 differ). Done for tier 1: Coopt's Cheney baseline;
   prompts, marks and continuations (but routines 51–55). Done: the
   listings Fixpt now writes in full (`--full-literals`, C123), read; three
   gaps in them worked around, to be raised with Fixpt (C123). Done: the primitives, Fixpt's
   runtime's (C124; 292 agree). Done: the scheduler's runtime, Fixpt's
   (C125; 326 agree): processes, `par`, `race`, `pselect`, monitors,
   preemption, the clock. Next: eqtables, `%fx26-intern-datum`, I/O (the
   standard input waited for, too); native code's primitives with tier 2;
   Coopt's collector's steps past Cheney (generational, regions,
   summaries, marking, pacing, each against the one before); then stage A
   ported (`compile*.fx`, each marked `fixpt-source:`), its words
   compared with the listing.
   - `->*` in KFX takes a `define-rec` as one component (C123), Fixpt
     its components callees first: a group Fixpt accepts KFX may refuse.
     Fixpt's order, if one does. `->*` in a module's items and `letrec`:
     refused yet.
   - Tier 1's step limit is the whole program's, Fixpt's each form's
     (C125): the listing has one `program` word. Marks between forms in
     Fixpt's listing would let tier 1 count as Fixpt does.
   - Tier 1's clock is the wall clock, as Fixpt's (C125); kayak's own
     runtime, later, Coopt's virtual clock of steps.
   - `scripts/gen-vprims.py` is stale: `fx26/vprims.kfx` was edited by
     hand since (C110's refusals), so regenerating drops them. Bring the
     generator up to the file, or retire it.
3. `kayak-jit`; the arm64 encoder in KFX; tier 2. **Tier 2 under way
   (C126, 2026-10-16)**: the hand machine emitted by KFX with
   `fx26:arm64.fx`, on tier 1's memory shared; steps (1) flat arrays
   shared with native code (**done**), (2) entry, `NEXT`, exit, stack and
   control routines (**done**: 23 routines, 326 agree, `--hand`), (3) the
   rest, by how often programs give back: ~~calls (`call`, `tailcall`,
   `tcall`, `ttailcall`), `free`, `field`~~, ~~allocation (`cons`,
   `closure`)~~, ~~`prim`'s `%region-cons`, `global!`, `field@`~~ (C126): every
   routine Fixpt's hand machine has in arm64 but `halt` (once a run) and
   `execute` (no listing of Fixpt's programs has it). Given back still, as Fixpt's calls out for them: the other
   primitives, prompts, marks, continuations, `rest`, `field!`; each
   moved when a program gives it back often (`--gc-stats`). **Tier 4
   (C128, 2026-10-16)**: the listing's words compiled with tier 2's
   routines (`--compiled`), entered through tables of where to resume;
   326 agree. **Tier 5 begun (C129, 2026-10-16)**: register code, which
   Fixpt's listings already carry, read into register words and
   interpreted by tier 1 (`--regs`, the reference; 326 agree), and
   compiled into the hand machine (`--regs --compiled`, C130; 326 agree;
   `helpers` 99 ms to 21). Next: what register code gives back most
   (`cellular`'s control routines, calls of stack code, the other
   primitives); its call and return without the data stack (Fixpt's
   link, `blr`); then tier 6, native frames. Tier 3,
   stencils, last (`native.md` step 5). **Begun (C116,
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
5. Twobit's passes; the compiler compiled into its own heap. **Begun
   (C133, 2026-10-17)**: kayak's own code generator, Fixpt's compiler
   written in FX-26 ported to KFX (`fx26/cg.kfx`, `kayak-fx26 --cells`),
   its words run every way Fixpt's are (`cells-programs.py --ours`): 77
   programs agree, 68 of them Fixpt's words exactly. Done since:
   modules (C144), regions (C145), prompts (C146), bloblets and floats
   (C152), `case` and `typecase` (C153). Next: a standard operation
   whose arity the generator does not know, as a value (3); `vlambda`
   (2); `quasiquote`; a `case` of many data, a search tree; a `tagcase`
   of a blobref to a flat sum; literals shared (`#0=`). Then Fixpt's middle phase (inlining, specialization),
   lambda lifting, register code (twins), so that the words are
   Fixpt's; then the REPL's forms through it (§33's `,disassemble`).
   Most programs not compiled wait on the checker (§30).
- Tier 1 on a long run (C142, the user: "support long running
  programs/services"): what it makes of the runtime's own, never let go
  (C43), on a path a program takes again and again. Done: the
  scheduler's lists and closures (C148), a message's copy (C149), the
  primitives of any number of arguments and the strings' code points
  (C150), UTF-8 and `list->bytevector` (C151): a run's own memory is
  now the listing read, whatever the run's length. What a failure's
  message and the value shown make (`string-text`, KFX's strings) is
  made once each, as it must be. Left, as found: whatever `heap-stats`
  shows growing with a run's length.
- Tier 1 at `--slice 2` (C143): a resumed task has a step or two
  before its next preemption, so each turn of a loop costs some 6,700
  steps (C142 put that at each preemption; one is some 160). Fixpt's
  machine runs the same rule; tier 1's step is what costs, its slice's
  end now asked lazily. Left: `--compiled --slice 2` of
  `preempt/allocating.fx`, 139 s, gives tier 1 back nearly every cell
  (21.6 million entries in the short one), the hand-back itself
  (`hand!`'s copies of the registers) its cost; and fat LTO, 18% more
  of tier 1, not taken: every release build of the check would be under
  it (C143).

## 32. `wcell`, `tword`, `wglobal` out of FX-26 (the user, 2026-10-16; C120)

Fixpt's grammar has them as base types beside `int` and `string`, and its
checker makes them at its start (`check.rs`, "Cellular code, for the
compiler written in FX-26"), with standard constants (`wcell-unit`,
`wcell-global`, `wcell-sum`, ...): handles on Fixpt's cellular machine,
added with its compiler in FX-26 (`61369385`, 2026-09-26). They belong to
an implementation, not the language: they should leave FX-26 proper, as
a built-in module, say, as native code's operations came with `(io
native)`. **Not raised with Fixpt yet** (the user: not to bother them
about it for now). Until then KFX refuses them, which K1 allows.


## 33. The REPL's commands, as Fixpt's has them (the user, 2026-10-16)

Fixpt's FX-26 REPL (`fixpt-cli/src/fx26.rs`, `help.rs`) has commands ours
(`fx26/live-repl.kfx`'s `command!`, and `repl-tasks.kfx`'s) lacks; fill the
gap where it makes sense. Ours already has `,help`, `,type`, `,time`,
`,load`, `,pending`, `,cancel`, `,step-limit`, `,apropos`, `,quit`.
- Now, on what we have:
  - `,fits TYPE`, `,returns TYPE`: what accepts, and what produces, a
    value of a type, by subtyping over the environment's types;
  - `,help` inside a form: a hole, answered from the types (what fits
    there) and by running the form up to it. In FX-26 Fixpt checks the
    hole and does not hold it, so no `,resume` or `,where`;
  - `,list-outdated`, `,rerun-outdated`, `,redefine b|r`: definitions
    using a global since defined again at another type;
  - `,code`: each form's generated code shown, on or off;
  - `,at FILE LINE COL` under an `--emacs` option: where the next form
    came from;
  - `,apropos KIND TEXT`, narrowed to one namespace (value, type, family,
    generative, effect, region, base);
  - the short forms `,h`, `,?`, `,a`, `,f`, `,r`.
- Once the REPL's forms reach the cells machine (our FX-26 to cells code
  generator, §31 item 5; kayak-fx26 linking kayak-cells):
  `,disassemble E`, `,disassemble-asm E` (`cells/show.kfx`, `hand.kfx`);
  `,native NAME [ARG…]` with tier 6's convention; `,inliners NAME` once
  register code inlines.
- Not to copy as is: Fixpt's `,pending [clear]` is about entries waiting
  on types not defined yet, while ours lists the entries not ended; if
  we come to have the former, it wants another name.

## 34. Tiers 2 and up on x86_64 Linux (the user, 2026-10-18; C134)

Fixpt has the whole ladder on x86_64 already (`cellular/x64.rs`, its
stencils, `x86_64.fx`, `native-x64.fx`, `native-x64-regs.fx`,
`direct/x64.rs`), on its branch `x86_64-fxjit-nativecc`, which the pin
now is (`5ef9fe8`). Done: the pin there; kayak's checks pass on
Linux, stage 0 included (C134). ~~1. An x86_64 convention for §97 in
Fixpt first~~ (done by Fixpt, `native-x86_64.conv`, and the pin on it:
C135). ~~2. `kayak-jit` on Linux x86_64~~ (done, C135: the code area's
two views, the trampoline and routines, the stack's limit; the corpus's
made-code programs twinned for x86_64 and run there). ~~3. Tier 2~~
(done, C136: `cells/hand-x64.kfx`, every routine of `hand.kfx`'s machine
over `fx26:x86_64.fx`; the 338 programs agree under `--hand` on Linux).
~~4. Tiers 4 and 5~~ (done, C137: hand.kfx's word and register
compilers, on x86_64, at the same places; every `--compiled` way of
`cells-programs.py` on Linux; `RESULT` in `rax`, `REGk` in the state,
the link there too). Left:
~~5. Tier 6~~ (begun, C138: register code as native procedures, which
leave for tier 5 rather than give back; `--direct`). Next, by what keeps
words out or makes them leave: allocation as tier 5 allocates (done,
C147: where it would collect, tier 6 leaves first). Looked at
2026-10-24, not taken: making the runtime primitives register code
calls most (`%fx26-sched`, `%make-box`, `%make-frozen`, `setfield`) in
place. Every hot word of the benchmarks and of `preempt/allocating.fx`
is tier 6's already; what those primitives keep out is the scheduler's
library and words run once a spawn, a send or a preemption. Left of
tier 6's: the control routines (`prompt`, `callcomp`, `abort`, in
`captures`), and the same on arm64, measured on macOS.
- ~~`REG1`… in registers~~ (`REG1`, `REG2`, C137; `REG3` in `r10` with
  addresses, C139).
- Values as tagged addresses (C139): done, and on x86_64 without the
  base register; on arm64, `x19` to go the same way, and the arm64 side
  tried on macOS.
- `--fixpt-calls` (C140): Fixpt's tier 5 convention on x86_64, a toggle;
  the default if it ever measures better, or dropped if it never does.
- On arm64, measured on macOS: register code's return by `ret` (x30), not
  `br x30`, so that the processor predicts it, as x86_64's `ret` (C137);
  `bl`/`blr` already push to its predictor.
- Noticed on the way: Fixpt's `native/crossing-overflow.fx` (its §93, new
  at this pin) fails on Fixpt, a Rust stack too deep for calls between
  machines, and runs on the engine, which has no such calls:
  `run-fixpt-programs.py`'s one "beyond KFX" difference.
- ~~`kayak-cells --hand` on x86_64 would run tier 2's arm64 bytes~~:
  `hand-code!` now chooses by the convention's `target`, and refuses on
  another machine (C136).
- The corpus's answers now differ by machine (C134): libm's last bit in
  `floats`, and `made-code`/`made-flat` say whether made code ran
  natively. If FX-26 is to say what `f64-tan` gives, as a reference
  must, that is for its definition (§9 of PLAN).

## 35. An index of the checker's bindings (the Fixpt team, 2026-10-25)

**Done** (C176): both parts; stage 0 here 102G cycles to 65G, L2 misses
1.04G to 0.56G (pinned Fixpt 8f76d65); at the pin ae397dc (C177), 21.7G
and 0.27G. For the team to measure on their snapshot.

Measured on Fixpt ae397dc8, kayak at abecd81, one core: stage 0 (kfx
compiling itself under `fixpt run`, fx-regs-rsjit-nativecc) is about
16.7G cycles; `find-bind` (kfx/check.kfx) is 12.2% of them and 52% of
the run's L2 data misses (140M of 268M); 10.5% at C143. What is left is
the walk: `cv-binds` from the innermost, `string=?` on each, a chain of
dependent loads per element, and a miss walks it all. 63 calls at
78c0bc2 (check-exprs 40, check-defs 13, check 9, terminate 1). The worst
are lookups that find nothing: the markers `%module`, `%loaded`,
`%toplambda`, `%instance`, the `%type NAME` lookups since C143, and the
form-head checks (whether `vlambda`, `rlambda`, a blob form's name is
bound), asked of every such form.
1. **The markers as state**: whether a module, a load, a top-level
   lambda is being checked, and the instance context, as fields of `cv`
   (or one field of flags) set where their bindings are pushed now; each
   lookup a field read. The smaller change, most of the full-length misses.
2. **`cv-binds` indexed by name** beside the list: a persistent balanced
   tree keyed by the name (as `fidx` is for facts, C143), each name to its
   bindings innermost first; adding a binding conses and inserts;
   `find-bind` a tree lookup giving the same answer. `cv` is passed
   functionally, so scoping needs no undo. Callers that walk or filter
   the list (`conversion-bind`, `opened-binds`, check-defs' filters over
   `bs`) keep it until moved over, one at a time. Interned names would
   be cheaper again; not needed for the main gain.
Expected: most of `find-bind`'s 12% and about half of the L2 misses.
Fixpt will measure what lands, on a new pinned snapshot.

## 36. `letfreeze` and eq-tables in KFX (the Fixpt team, 2026-10-25)

**Done** (C193, C194): step 1, and step 2 as choice (a); eq-tables keyed
by references, arrays and I-cells, in the checker, kayak-rt and the
emitter; `pair-identity` and `eq?` still refused for KFX. Left:
`(letfreeze (r p) ...)` into another place (on `letreap`'s emitting), pair
keys. The corpus under `unsafe-par` in the check since C195.

For the compiler on kayak's own FX-26 runtime, tasks on several threads:
frozen data tasks share without copying (a form's checked interface,
read by many checkers), and tables keyed by identity (memo tables, what
was visited, facts about a node). Also now, in the Rust path: frozen data
crossing processes and `par` branches under C102's `Send`/`Sync`.
Today KFX refuses `letfreeze` and any `const` region (C156, C161), and
`eq?` (C75: a union's value in Rust is a box, a new object). The checker
and built-in.kfx know the eq-table operations; the engine and tier 1
have them (kind 43, `%fx26-eqtable` primitives); only Rust is missing.
1. **`letfreeze` into the heap**, the value's type holding no mutable
   kind (no ref, array, flat array, bytevector, I-cell: lists, products,
   sums, datatypes, strings, scalars): the body with `r` as `'static`,
   what it allocates at `r` in the heap's arenas, the value the Rust type
   it would have at the heap. `Cons` has no `Cell`, so such data is
   `Sync` where its elements are, rustc checking it, no `unsafe`. A type
   at `const`, or at `acyclic` made so, written as at the heap.
2. **Frozen data holding a mutable kind**: a frozen ref is still a `Cell`,
   not `Sync`. (a) frozen to the checker, crossing no thread in Rust; (b)
   `kayak-sync` asserting `Sync` at `const` (a SAFETY comment on the
   checker's rule that nothing writes `const`); (c) a second, Cell-less
   representation, a copy at the freeze. The team recommends (a).
   `(letfreeze (r p) ...)` into another place later, on `letreap`'s.
3. **Eq-tables by key kind**: Fixpt keys them only by mutable objects
   (`ref-identity`, `array-identity`, `icell-identity`, `pair-identity`),
   its operations writing the keys' region, so no table is keyed by frozen
   data (fx26.md, "Identity"). In kayak-rt a ref is a `&Cell`, an array a
   slice of `Cell`s, an I-cell a reference, none copied boxed into a
   `Value` (`value::other` boxes the pointer): a `HashMap` by address
   (`std::ptr::eq`, the fixed hasher), its type carrying the keys'
   lifetime, so rustc refuses a table outliving a `letrena` of its keys.
   No iteration (`ref`, `has?`, `set!`, `delete!`, `count`), so the address
   order is never seen: answers deterministic, Fixpt's. Pair keys wait:
   §24's list-as-union view might rebuild a list, until the emitter shows
   no conversion copies a pair at a writable region. `eq?` may stay
   refused; later, perhaps, at those kinds' types.
Checks: corpus programs with Fixpt's answers: a frozen list shared by
`par` branches on threads (default and `unsafe-par`); a frozen ref sent
to a process refused (choice a); an eq-table keyed by refs, with deletes
and counts, inside and outside a `letrena`; refused, a table outliving its
keys' `letrena`, and `pair-identity` until allowed.

