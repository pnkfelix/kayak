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
- **No executor**: the scheduler is KFX (C45). Instead, in order:
  `abort-with-continuation` in the prelude and kayak-rt; the checker's
  rule for it, `(comefrom r)`, prompts that may capture, none inside a
  `letrena`; the emitter's `async fn` by effect; Fixpt's `ping-pong.fx`
  and `promise.fx` as KFX in the corpus; a library of tasks in KFX.
- **Record and replay** (the user, 2026-10-12), as Coopt's: runners
  (sim, replay; live later) behind one interface, a log of arrivals by
  step and clock moves, outputs checked against it on replay, one event
  stream, and a determinism check in `scripts/check.sh`. The compiler's
  own I/O moved behind the runner.
- **Corpus programs**, compared with Fixpt.
- **Incremental compiling** (Q5, answered): the compiler's passes as
  queries, memoized with their dependencies, as `coopt-query`'s; the
  machinery in `kayak-rt`, given to KFX as primitives. After the
  library of tasks, since a query run is a task (Coopt's arrangement).

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
