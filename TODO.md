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

## Next, in order (the user, 2026-10-27)

"File all of these as todo's. Then do tier 1's missing primitives. After
that, register code stage 3. After that, specialization." Each in its own
section below where it has one; here the order, and what each is.

1. **Tier 1's missing primitives.** Done (C258): `string->list`,
   `symbol->string` (the 64-bit integers' tier 1 had). `peval` runs on
   tiers 1-6 (its 2000 iterations 281 s on tier 6: a benchmark to measure
   against Fixpt's). Found by it: **the KFX compiler on tier 6 grows its
   heap without bound** (killed at 248 GB; Fixpt runs it in 0.6 GB):
   a leak, or what tier 1 keeps reachable; first to find, with `--gc-stats`
   on a shorter run, what the heap holds.
2. **Register code stage 3** (`docs/research/register-code.md`). Done
   (C259): bloblets, regions, modules and `with`, flat sums' `tagcase`;
   830 register words Fixpt's exactly, 9 differ, none declined. Found:
   Fixpt's `r-collects` takes a `case`'s comparison for a call where a
   local of its name shadows it (the checker leaves its `(with #%fx …)`);
   kayak matches it, a question for Fixpt's team whether to keep.
3. **Specialization**. Done (C260): a procedure taking a lambda it only
   calls copied for the lambda given (`map-add@lambda@730`); the 4
   programs with copies' stack words Fixpt's (299 in all), 910 register
   words Fixpt's exactly, 7 differ (a module member re-exported, 2; a
   global holding a constant list unrolled, 5: Fixpt's `c-reexport-inline!`
   and its constant lists, both C252's remainder). `bench/closures` on
   tier 6 220 million cycles, Fixpt's listing 218 (from 253). Found:
   kayak keeps a global's inlining after it is defined again, where
   Fixpt drops it (`c-push-global`); its specializations now follow
   Fixpt's, its inlinings to do.
4. **Compile memory and time**: kayak compiles 10-30x slower than Fixpt,
   mostly page faults (an 11 KB program 256 MB at peak; the KFX compiler
   compiled to cells by kayak's FX-26 compiler, 55 GB). Measured (C261),
   at that size it is not regions kept but work done: the listing built
   by appending each piece onto the rest (86 of the heap's 187 MB, now
   joined at once, `string-list-append`: 179 MB, 240 ms), then the reader
   reading kayak-rt's 198 KB primitives' table again for each new name
   `c-arity` was asked of (C262, kept once read: 55 MB, 80 ms). Left: the
   checker's mangling, lists of strings. Then the 55 GB case measured the
   same way (`heap-stats`); options to the user before building anything
   larger than a fix of this kind.
   Fixpt has made the change (5731f52, the pin since C263): syscalls of no
   required effect, and `kayak:string-utf8-length/1`, which kayak-rt, the
   engine and tier 1 now give too. Next: `string-list-append` as one buffer
   sized once (`make-bytevector`, `bytevector-copy!`, `utf8->string`), its
   pieces' sizes by the syscall, in the engine's own files (only a
   runtime's code declares one).
4b. **FX-26's features since the pin moved to 5731f52 (C263)**, which
   kayak's checker refuses for now (so their programs are not in the
   passing lists): `attempt` in KFX (KFX's since C415; the checker, the engine and tier 1
   have it, C297; the processes' primitives are C295's and C298's); `type-at`;
   `load-moduleof`, `load-input-moduleof`, `load-typeof`,
   `load-input-typeof`; `restrict`; a top-level `include`; a file's
   abstract types one for every load of it. Fixpt's `gdb`/`lldb` support
   (code named and its frames told, source lines) is not taken either;
   kayak has perf's (C246). Then KFX's own `load-module` of files, so that
   kayak's units say their layers' interfaces (`load-moduleof`,
   `restrict`), as the user suggested.
   Since 6447f9b (C275): strings at a region, `(stringof r)`, kayak's
   FX-26 has since C285 (KFX's waits for plan B, which needs it); a module's expressions as its items; `module-parameters`
   of modules. Since 0c2e4dd: `(module-parameters ())`, a maker of none;
   arm64's native makers and in-line allocation, and `lldb` (Fixpt's own).
   Since 7038269: a list nothing places is `acyclic` (Fixpt's e4af8997,
   Heist's request): a `poly`'s region binder a call leaves unsolved, used
   only as a pair's region and only made and read there, is `(acyclic
   heap)`, not a region of its own: kayak's since C284, with frozen
   covariance and masking's reads and writes. Module parameters of every
   kind, expressions as items, `load-moduleof`, `restrict`: kayak's since
   C286-C288. Not yet: a `load-input` maker whose loads are given its own
   binders (`modules/load-moduleof.fx`, `restrict-parameter.fx`, the
   `made-*` files), which a static splice cannot make; a family imported
   into another module (`import-list-and-family.fx`). Also `.fasl`s, heap images, a native front end, and a
   place's limits: Fixpt's own, nothing of kayak's ported.
4c. **FX-26's features since the pin moved to 63b95ef (C290)**, which
   kayak's FX-26 has not yet; the ported sources' marks moved with none
   of them carried over (the runtime's, tier 1's and the code generator's
   parts of each to come with it):
   - **Keywords** in KFX: the checker, the engine and tier 1 have them
     (C291, C294); kayak-rt has not (KFX refuses keyword values); nor the
     reader `#:|a b|`.
   - **Joins**, beyond `list`'s elements (C292): any call's arguments
     instantiating one type binder bounded only from below by a sum
     (Fixpt's `sum_lower_only`, 39318012).
   - **`stage`**, evidence and **sessions** on the compiled tiers: done
     (C335-C344, `docs/research/compiled-stage.md`), 22 of Fixpt's 23 stage
     programs on all seven tiers, staged words compiled to machine code on
     x86_64; `session.fx` differs in a refusal's type printed (below,
     "Refusals in Fixpt's words"). Not KFX (not needed there).
   - **The engine's primitives and constructors within data** (C333): a
     list of them prints as tier 1 sees them (`#<immediate>`, `#<? 2
     fields 0 bytes>`), not `#<procedure>`, until they are Fixpt's objects
     (one heap, step 4).
   - **The live REPL's early run of a draft** (`live-repl.kfx`, the
     in-process speculation) lowers it with no checker facts: a module
     given where its type orders it otherwise is not reshaped there (C328),
     nor a `with` of a module value opened. The machine's speculation
     (`repl-tasks.kfx`) has them.
   - **Refusals in Fixpt's words** (C312; 122 of 406 so far): Fixpt's
     termination reasons after a `spin` mismatch ("; it may not end:
     …"); the storage knot's "and it is a T" (no type printer before
     `show-ty` in check.kfx); types printed alike (`car`'s parameter,
     inference variables `?` against kayak's `?1`); the tail
     (`target/kayak-checks/checkers`, `both`).
   - **From Fixpt 19b31f1 and 8616f2f** (C314, C341): Fixpt's 516abc8e and
     bb635fe8, a compiled word's code kept in its own bloblet, on x86_64 and
     arm64 (`cells/hand.kfx`'s mark at 8616f2f): x86_64's done, C342 to
     C344 (`docs/research/word-code.md`); arm64 compiles nothing, Fixpt's
     interim, until bb635fe8 is ported, on an arm64 to run it on;
     arm64's register code (6c46d762). (`attempt-within`
     and `dlambda` descriptions done, C315.)
   - **Exporting a new item from check-exprs.kfx's `expressions` module**
     (`mtype-opened`, at C315) broke the bootstrap's second generation:
     the Rust it emitted did not build (`expected &Res, found Void`, 382
     errors). Not looked into.
   - **The compiler no longer runs on the engine** (found 2026-11-03,
     C318): the engine has no `rep-nodes` or `rep-of-nodes` (C300's
     plan, "the engine: ops", not done), which the checker uses since
     C301, so `kayak-fx26 --run kfx/… -- arith.kfx …` (C63) fails, and no
     check runs it. The ops, then a check that runs it, and
     `scripts/engine-bench.py`'s row for it.
   - **The compile's remaining cost** (C361-C365): the compiler on
     itself, warm with memos 1.36 s (0.92 GB), plain 2.38 s (1.26 GB);
     of either, about a third is still the kernel faulting in memory the
     heap never frees. What makes most now (plain, 1.19 GB in all): list
     cells of characters (290 MB: the emitter's output, a module's body
     copied at each module around it in `put-module-around`, the output
     made a string by `rev->string`), the checker's `bix` index
     (180 MB: a path of the tree copied for each binding, and batching
     the bindings did not help, C365), lists of strings (112 MB), the
     read forms. The time left in kfx's own code is spread thin: no
     function has more than 2% of it (C365's profile); checking is half
     of a plain compile, emitting a seventh. Huge pages for the heap
     (`GLIBC_TUNABLES=glibc.malloc.hugetlb=1` took a warm run 2.96 s to
     2.29 s, C362's) would want `unsafe` in kayak-rt, which has none. A
     form's facts from the emitter's tree (`fidx`) instead of a scan of
     the whole list was tried, and was slower (0.94 s against 0.64 s):
     the tree mixes files' positions. A Salsa-like query layer (the
     user's idea, 2026-11-06): keys by the names a form's check reads,
     not the chain of every form before it; worth most in a compiler
     that stays running.
   - **Reps byte for byte Fixpt's** (C346-C349, C352): checked against
     `fixpt check --reps` (`tests/reps-passing`, `checks-summary reps`):
     431 programs the same, 63 differ (mostly modules' reps: abstract
     types, descriptions and their order; nested `poly`s; `param-sel`),
     four whose module type as a value kayak cannot read (C352).
   - **Staged code and sessions never give memory back** (the user's
     concern, 2026-11-03; corrected 2026-11-05): kayak's own doing, not
     FX-26's nor Fixpt's compiled machines', whose session is all in the
     heap (`sessions26.rs`, 6ddd924d: its front end in the session's
     third field, its globals cells of that front end's compiler, its code
     its words', which the machine keeps nothing of since 516abc8e), so
     that an unreachable session is collected, code and all. Only Fixpt's
     path on its Rust checker (`sessions.rs`'s thread-local `SESSIONS`,
     never shrunk) grows, as kayak does. kayak's: every word a listing
     reads is named, so a root forever (`named!`, `named-forwarded!`),
     a plain `stage`'s too; a session's globals are tier 1's (and the
     engine's) flat table, `fx26-session-K:NAME` (C311, from Fixpt's
     lowered path); `sessions` and `pendings` keep every session's
     checker state and every definition checked. The plan:
     `docs/research/session-memory.md`. (The user's tree of environment
     frames, each session a leaf, is what Fixpt's heap session amounts to.)
   - **A race in Fixpt's `process/basic.fx`** (seen once, 2026-11-01,
     under `--regs --compiled --gc-stress` with the check's load): its
     `wait-for` returns on `exited` without a last `process-output`, so
     what the child wrote between the two is lost (`got hi`). kayak's
     `process-status` is Fixpt's rule; the program is Fixpt's to fix.
   - **I/O**: a child in a session of its own, not only a group (needs
     `setsid`, so `unsafe` or a helper); `dump-heap!` (59114584).
   - (Fixpt's e2f3f0f6, a bloblet's values by their index, §205, is its
     Rust heap's accessors and a lint: nothing of FX-26's, nothing kayak
     ports.)
   - Fixpt's re-architecture, "heap types": types as reps
     (`docs/research/type-reps.md`, C299): the crate taken whole, the
     standard `rep-of-nodes`/`rep-nodes`, kayak's export and import, the
     facts by rep, then `stage` on reps.
5. **The call protocol on tier 6** (performance item 3 below): the
   entry's stack bounds, the frame's header word, values kept in
   registers across calls; each part measured by the line tables (C250).
6. **arm64**: nothing of C241-C253 has run there (the Mac).
6a. **A memo the incremental compile reuses wrongly (2026-10-30, C54)**:
   `scripts/regen.sh`, with its memos (`kfx --cache`), made Rust that does
   not build, where the same compile without them is the fixed point. The
   compiler of 6fc82507, on the sources of the `stringof` change (C285),
   with the memos it had kept: `check_exprs.rs` lacks `nil`'s turbofish
   (`nil::<Str>()`) and `refuse`'s `match … {}` in 669 lines, facts the
   forms have now that a memo made before did not. The memo file and the
   sources' diff are in target/memo-repro/. The incremental tests' edits
   do not make this case: find what the memo's key leaves out (a fact
   outside the form's span the emission reads? the module's context?).
6b. **Tiny slices, measured (2026-10-28)**: `preempt/allocating.fx`'s
   `--slice 2` ways are the check's longest tests (`--compiled --slice 2`
   116 s alone). At 25,000 iterations, slice 2: tier 1 runs 325M cells,
   about 6,500 a loop iteration, `%sched` 28% of them, and collects 226
   times (8 at slice 16). That is the scheduler's design, ported from
   Fixpt: its interpreter has the same steep curve (13.8 s at slice 2,
   0.8 s at 16; kayak's tier 1, 18 s). Tiers 2 and 4 are slower there
   than tier 1 (27-28 s): each slice's fuel is so small that the hand
   machine is entered 180M times (1.7M at slice 16), each entry a cell
   or two. Not worth fixing for real slices (65,536 polls). Done instead:
   the check's slices four or five, jittered (C280).
7. **Checker completeness**: the 12 of `scheme-bench/` kayak refuses
   (`checks-summary checkers --list refused`, after the `checkers` tests run); the knot rule at a poly's
   instantiation; `set-car!`/`set-cdr!` of a union's pair; the 69 programs
   whose types print otherwise.
8. **Register code's remaining differences from Fixpt's**: globals
   holding constant lists (`unrolled-lists.fx`), a module's member
   re-exported and inlined (`reexport-inlined.fx`).

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

- **Another machine's programs in tier 1's tests, to look at together**
  (the user, 2026-10-28, not high priority): C283 keeps a program that
  declares another machine's convention as it was there, unrun, as C135
  does for the corpus. The user is not certain that pinning such a
  program to its convention's machine is best. To look at: whether tier 1,
  which runs no native code of its own, could check such a program's
  words on any machine (Fixpt refuses the convention elsewhere, so its
  listing would have to be the other machine's, kept); or whether the
  program should say which machines it is for some other way.

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
- ~~**A `lambda` of more than four parameters** has no `lambdaN` in
  `kayak-rt`.~~ (C493: `lambdaN` gone; a closure has six, `closure0` to
  `closure6`.)
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
- **A closure that does not escape is leaked all the same**: a closure
  object for every lambda (C481; `lambdaN` before), one each round in
  `closures`. One the checker finds stays within its scope could be a
  local, borrowed as `&dyn CallN`.
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
  `Emit(form)` and `Check(form)` kept between compiles (C54, C55), each
  keyed by what the form reads, by name (C384), the machinery a library,
  `lib/query.kfx`; the compiler's identity in the memo file (C387). Next: the library's names given in Rust (kayak-rt, from
  Coopt's `coopt-query`: coopt-adoption.md step 3's second stage), when a
  long-running compiler wants them; query runs as tasks
  when the compiler has a reason to wait. A warm compile's cost now is spread out
  (C386): emitting and checking from memos, the reader, the keys.

Phase B, the compiler for two languages:
- **The front end shared**: the checker grown to FX-26's, KFX's
  restrictions a pass of their own.

Phase C, FX-26's runtime:
- **An IR** for FX-26 at large.
- **The engine, in KFX** (K7): a heap with a collector, its stack in the
  heap, composable continuations; an interpreter of the IR, kept as the
  reference (Q8); its tasks on the one scheduler.
- **K6**: Fixpt's programs on it, compared with Fixpt.
- **Reclaiming kayak-rt's heap: tabled** (the user, 2026-11-07: "until
  it seems like it is unacceptable overhead"). Nothing is freed but a
  `letrena`'s arena. When it is taken up: a heap census first (how much
  is still reachable while the compiler compiles itself); then a
  non-moving mark-sweep in kayak-rt and kayak-sys, collecting at tasks'
  yields so that no Rust frame holds a KFX reference (roots: suspended
  futures, globals, thread-locals), opaque closures and futures scanned
  conservatively; Coopt's precise `Value<'gc>` heap the end state.
- **The emitter's async mode** (C371-C382): every procedure an `async
  fn`; the compiler built so makes itself, and each corpus program gives
  Fixpt's answer (`scripts/async-check.sh`, in the full check). Leaves
  are plain `fn`s (C377: 494 of the compiler's; 3.65 s to 3.43 s, the
  synchronous build 2.1 s). Next: the rest of the overhead, futures'
  boxes and moves in recursive walks and in higher-order procedures
  (`any?`, `map`) given procedure values. Fuel is charged and `main` is a
  `kayak-sched` task (C378), and calls boxed and charged only within
  recursion groups, the call graph's strongly connected components
  (C379: 21.9 G instructions, the synchronous build 12.1 G), and
  processes are tasks of the same scheduler (C380), and the standard
  streams and `sleep-until` go through kayak-runner (C381), and so do
  files, the terminal's modes and programs run beside this one (C382).
  Waits where nothing may wait: refused by effect where kayak's code runs
  to its end (C388); Fixpt's markers in progress (masking a wait leaves
  `spin` and the `io` split are in at 0d933ff, C393; markers, marker
  bounds and `effect-case` to come), then `waits?` replaced by their
  table; `docs/requests/fixpt-responsive-effects.md` asks
  Fixpt for sleeping apart from the clock, inspectable evidence and
  bounded effect variables. Next:
  `--async` with `--cache` (the mode in the memo key); a maker's poly or failing items; native code calling back a
  procedure that may abort.
- **The second self-hosting** (C366): restored on tier 1, checked in the
  full check (`scripts/selfhost-cells.sh`, 97 s, 7.5 GB); the IR engine's
  path (`scripts/selfhost-engine.sh`, C66) is broken (no `rep-nodes`) and
  left, as docs/direction.md §7 retires that engine. Tier 1's start-up
  never frees its listing's text or its compiled words (4 GB).
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
- ~~Fixpt, to raise: `native/mixed.fx`'s `#<continuation>` on its default
  machine~~ (raised; fixed by Fixpt `19973abf`, at the pin since C224).

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
The measure (C114): `crates/kayak-checks/tests/checkers.rs` (was `scripts/compare-checkers.py`); the plan,
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
   None of the 433 programs Fixpt accepts refused (2026-10-25, C208); 64
   differ in printing; 369 the same; the 336 it refuses all refused. What each needs (looked at 2026-10-25):
   - Redefinition at another type: **done** under `--check` (C202); the
     REPL and the engine's run still refuse it.
   - `modules/load-layers.fx`: **accepted** (C203); its printing differs.
   - `regions/distinct-swap.fx`: **accepted** (C204).
   - `conventions/poly.fx`: **accepted** (C205); converting between a
     binder's convention and another, not yet.
   - `run/acyclic-in-a-place.fx`, `run/letfreeze-into-place.fx`:
     **accepted** (C206).
   - `sizes/narrayof-lengths.fx`: **accepted** (C207).
   - `made-code/leaves.fx`, `made-code-x86_64/leaves.fx`: **accepted**
     (C208).
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
   preemption, the clock. Processes on kayak's own words and the engine
   too (C219): the forms made as Fixpt's parser makes them, the engine's
   scheduler `vsched.kfx`; left, `%sched`'s standard input (op 43) with
   `read-byte`, on both. Then (C220) `cwcc`, `f32`, flat arrays,
   eqtables, `native-convention` on the engine; eqtables, `assume`,
   `assert`, `list->string`, `string->symbol` on tier 1; left on tier 1,
   `%fx26-intern-datum` (Fixpt's weak table; done, C231). Kayak's compiler to cells
   (C221), adapters too (C230); words not Fixpt's exactly
   in `bloblet/flat-inlined`, `native/flat-keyed`, `native/variadic`,
   `run/variadic`, `run/case-tree`. Code a program makes, on tier 1
   (C226, by syscalls, C225): a code area, `seal-code`, native closures
   called; the convention's routines on tier 1's heap (C227); callbacks
   on every tier (C228); collections under native frames, by their stack
   maps, and code no longer reached freed (C229); adapters between the
   conventions (C230). **Before building further on them: review with the
   user kayak's departures from Fixpt's object model and intended design**
   (the user, 2026-10-26: "I'm sure some are totally reasonable, but I'd
   like to have better awareness of them regardless"). A note first, each
   departure what kayak does, what Fixpt does or means to, and why:
   tier 1's code area a reservation of its own, not a range of its heap
   (C226; `native.md` decision 3), and so the address order checked
   (C230); the convention's routines' objects in a host heap (C227); the
   machine's registers kept across a callback on the return stack, the
   frames' values on the data stack (C228, C229); and any other place the
   implementation was taken over the intent. The note is written:
   `docs/research/departures.md` (C231); the review is under way, its
   decisions recorded there and here:
   - **Departure 7, Cheney's collector, no barrier read** (the user: "just
     leave this for now. We will revise the runtime entirely when we start
     pulling in Coopt's runtime and codegen design"): left; the runtime's
     collector and code generation to be revised whole then, from Coopt's.
   - **Departure 8, regions and reaps in the one heap** (the user,
     "Sure"): left for the same revision; unobservable, even in space
     (what a region held is unreachable once it exits, and collected).
   - **Departure 9, tier 6's frames kayak's own** (the user, "Yeah
     okay"): left for the revision from Coopt's code generation; the
     native-frame flushing work (departures 1, 5, 6) overlaps it, and
     would give tier 6 real frames.
   - **Departure 10, register code's registers on x86_64** (the user,
     "Okay"): left for the code-generation revision; `--fixpt-calls`
     (C140) already gives Fixpt's, checked every way.
   - **Departure 11, tiers 2-5 on tier 1's state**, and **performance
     before the Coopt adoption** (the user: "Frankly I was shocked by the
     overheads you measured, so hopefully we can address the bulk of that
     *before* undertaking the Coopt adoption"). Measured 2026-10-26, x86_64
     Linux, best of 3, ms, kayak's ~6 ms start-up taken off, kayak's tier
     against Fixpt's machine of the same kind:

     | program  | t1/rsinterp | t2/asminterp | t4/rsjit-cellcc | t5/regs-rsjit-cellcc | t6/regs-rsjit-nativecc |
     | fib      | 753/292     | 91/38        | 54/27           | 21/12.5              | 16/6.3  |
     | tak      | 342/104     | 35/16        | 20/10           | 11/3.7               | 10/2.0  |
     | loop     | 3062/868    | 297/117      | 122/77          | 36/24                | 22/9.5  |
     | lists    | 1912/634    | 201/119      | 118/102         | 58/15                | 48/11   |
     | closures | 2604/962    | 280/154      | 171/123         | 89/25                | 81/18   |
     | helpers  | 4128/1637   | 336/159      | 240/122         | 32/14                | 23/6.4  |

     Not hand-backs to tier 1 (184 to ~4400 machine entries a run):
     `tak`, no collection, 5x at tier 6: the call protocol (frames on tier
     1's flat-array stacks, a return entry each call); `lists` 2999
     collections, `closures` 750, Fixpt's 6 each: tier 1's spaces start
     small and grow only past half live. In order:
     0. **The I/O leak, first** (the user: "Yes definitely fix the IO
        leak"; departure 14): tier 1's `write-out`, 2M calls 71 MB peak
        against 13 MB for 200k, ~32 bytes a call, linear: KFX's built-ins
        leak their results (`heap_alloc` in kayak-rt) and tier 1 copies
        each string to KFX's first (`string-text`). The engine uses the
        same built-ins (not measured). Fix: the built-ins' constant
        results made once (`done`, `eof`), the machines' per-operation
        temporaries in a `letrena` or a reused buffer; then each machine's
        peak memory flat over a 10x longer loop.
        **Done** (C233) for the results, the standard streams' text and
        the input's read-ahead; left: `read-file`, `list-dir`, `args`,
        `write-file` (KFX data per call).
     0a. **Done** (C236). **The engine's tail calls through `tagcase`** (found by C233):
        `vm.kfx`'s `run` and `tagcase-step` call each other, a Rust call
        each time (KFX makes no mutual tail call a jump): a loop through a
        `tagcase` arm, 10M iterations 930 MB and 35 s, 40M an abort; Fixpt
        and tier 1 flat. `tagcase-step` to give `run`'s own loop the arm's
        body and environment; every mutual call of `vm.kfx`'s swept for the
        same; a test per kind of continuation, in bounded space.
     0b. **Done** (C237). **Fixpt's backward scan for a bloblet's header** (the user,
        "Yeah"; departure 15): a bloblet with fields and no trailer has
        its header found by scanning back from its suffix to the first
        header-tagged word, which no field can hold (Fixpt's `main_of`,
        `bloblet_main_cold`; `object-model.md`'s invariants). Tier 1
        assumes the header just before the suffix whenever there is no
        trailer, right only with no fields: a latent bug for any
        trailer-less bloblet with fields (copied as garbage, as C231's
        first weak pair was). One lookup, `header-index`, called by
        `forward-object!`, `weak-after` and `code-marked!` rather than its
        logic copied into each; weak pairs then Fixpt's layout (two
        fields, no trailer, car field 2, cdr field 1); a test of such a
        bloblet across collections.
     1. **Done** (C238; collections a quarter of the gap, not most). Heap sizing as Fixpt's (a nursery's worth to start, growth by what
        lives), measured against the table: cheap, likely most of
        `lists`' and `closures`' gap.
     2. **Done** (C239): `perf` of `tak` and `lists` on tier 6: at a
        measurable size 2.8x and 1.7x behind (the short runs held kayak's
        start-up), 93-95% in tier 6's own code. Next: a jitdump (code
        bytes, then FX-26 lines) for `perf annotate` of it; and kayak's
        start-up (tier 2's machine encoded at each start) on its own.
     3. The call protocol (values in registers across calls, frames on the
        machine stack), with the frame-flushing work (departures 1, 5, 6).
        What tier 6's code does, annotated (C240): a bounds check before
        each data-stack slot's access; arguments through the frame's
        slots in memory; slots made a store and an `add` each; comparisons
        made booleans before a branch; fuel counted at several blocks of a
        call. Each its own improvement, measured against C239's runs
        (`tak 30 21 12`, `lists` 60000 rounds).
     Start-up (C243): the stacks reserved, not written, 1.3 ms a start;
        tier 2's encoding 0.3 ms, not the cost guessed; the rest the
        kernel's loading of the binary.
     Debug info for FX-26 itself: **done** for tier 1 (C245): kayak's
        listings carry each word's line table (`; at FILE CELL:POS …`),
        and `kayak-cells --profile` counts tier 1's cells by source line,
        expression and word. **Done** for tier 4 (C246): the jitdump's
        `JIT_CODE_DEBUG_INFO` records, each compiled word's cells' code
        given their lines, so that `perf annotate -l` and `perf report
        --sort srcline` show FX-26 lines. Left: tiers 5-6 run register
        code, which only Fixpt's listings have, and those have no line
        tables: lines there when kayak's compiler makes register code
        (with a table of its own, instruction to position). Columns:
        `KAYAK_JITDUMP=expr` (C247), each expression a line of a side
        file, read back by `scripts/perf-expr.py`. Later (the user): that
        script ported to KFX, and all of this a REPL `,profile` command. A word from a module file of
        its own is given the program's file; positions are characters,
        not bytes; a listing's relative path read from where kayak-cells
        runs (no lines if not found there).
     Register code from kayak's compiler (C248, `docs/research/
        register-code.md`): stage 1 done; inlining, unrolling and constant
        globals done (C252: 11 differences left, `loop` and `closures`
        behind on tier 6 for stage 2; not yet: globals holding constant
        lists, a module's member re-exported and inlined); stage 2 done
        (C253: 733 Fixpt's exactly, 9 differ, 94 declined; `closures`
        behind for Fixpt's specialization, stage 4); next, stage 3
        (closures, `letrec`, `tagcase`, `apply`, `list`), stage 3 (modules,
        regions, prompts, bloblets). Register code's line tables, tiers
        5-6's debug info: done (C249); `tak` on tier 6, 35% in its entry
        (fuel, the guard, stack bounds, six slots written `#f`, the
        parameters stored), the call protocol's cost. C251: fewer fills, the
        guard's count an immediate, the entry's fuel once: -14.6% cycles;
        the entry 26.9%, its three stack bounds the most of what is left.
     4. Tier 1's own 2.5-3.5x (KFX's representation: bounds-checked flat
        arrays, words through calls), if it still matters.
   - **Departure 12, the native tiers' code never freed** (the user,
     "Yep"): no action; tier 1 compiles a listing's fixed words once at
     start, so the code is bounded by the program. Relevant when tier 1
     makes words at run time (a REPL, `eval`): the code-stubs design
     (words in the collected code area) covers it. Compiling every word
     up front lengthens start-up for big listings: lazily, in the
     code-generation revision.
   - **Departure 13, values as addresses** (the user: "Let's kill
     `--indexes`"): addresses the one mode. **Done** (C244), but arm64's
     base register `x19`, left until arm64 runs. Was to do: `--indexes` removed
     from tier 1 and tiers 2-6 (the `addressed` branches, arm64's base
     register `x19`, the index mode's paths) and from `cells-programs`'
     ways (`--indexes`, `--hand --indexes`, `--indexes --gc-stress`), the
     check so changing by the user's decision; C139's note of why it was
     kept answered. Heap images do not need indices (the user): a dump
     with addresses normalizes as it writes, each pointer word an offset
     from its space's base (the code area's from its own), its tag kept,
     and a load adds the new base, a linear pass each way; the code's
     bytes need none while made code addresses through the state and
     reference plus `delta`, as the convention has it (a rule to state
     when images come). And for performance item 4: tier 1's `word-at` and
     `word-set!` test every index for the code area (`code-on`, two
     compares; C226), on its hottest path: accessors of their own for the
     code area, or one reservation.
   - **KFX a sublanguage of FX-26** (the user, C254): Fixpt accepts each
     of kayak's KFX units, checked (`fixpt-accepts.py`). Done: the knot rule
     through conventions (C255), `car`/`cdr` of a union's pair (C256),
     `cons` at a region of its own (C257); `scheme-bench/` and kayak's own
     programs in the checkers' comparison. Left: the 12 of `scheme-bench/`
     kayak refuses (`checks-summary checkers --list refused`, after the `checkers` tests run); the knot rule at
     a poly's instantiation, as Fixpt checks it there too; `set-car!` and
     `set-cdr!` of a union's pair.
   - **KFX's own object model** (the review's last item; the user: "these
     differences are reasonably motivated"): KFX may differ from FX-26 by
     refusing, as K1 allows (its continuations Rust futures, resumed once,
     C45). One possible hole to probe: values crossing into made code are
     copied (C116, "a copy native code keeps does not see KFX's later
     writes"), which would be reinterpretation, not refusal. The probe: a
     KFX program that gives made code a bloblet, writes it after, and has
     the made code read it, on kayak and on Fixpt. If they part, the
     remedy the user points to: FX-26's flat data and blobrefs, added for
     a value model laid out as Rust's, shared in place across the
     boundary (as syscalls pass `flatarrayof` and `bytevectorof`), copying
     left only for KFX's Rust-shaped values, which have no FX-26 layout.
     **The review is done** (2026-10-26).
   - **Self-hosting, after the Coopt revision** (agreed with the user):
     kayak's runtimes (tier 1, the engine, KFX's compiler) compiled by
     kayak's own FX-26 native compiler, standalone with their own
     runtime routines; KFX to Rust then only the bootstrap. Not possible
     before: today's native code runs on tier 1's state, so compiling
     tier 1 by it would be circular.
   - **Convention changes go to Fixpt first** (the user, departure 4: "We
     can try delivering the request to the fixpt team"): the suffix stub
     written as a request, `docs/research/code-stubs.md`, its prompt
     given to the user; kayak follows when the pin moves.
   - **Departure 1, made code's `poll`** (the user, 2026-10-26: "we will
     want all of 1,2,3 at once"): one piece of work, not split. `poll`
     as Fixpt's `common_poll`: (1) the step limit, the slice charged to
     tier 1's count, trap kind 1 when spent; (2) a collection at a `poll`
     (`may-collect`); (3) preemption with made code's frames live, those
     frames captured as Fixpt flushes its native stack into heap
     `native-frame` objects, so that a task switch with them on the
     machine stack is whole. Reaching tier 1 as `foreign` does (C228,
     C229). With departure 5 folded in (the user, "Yep"): the mechanism
     is Fixpt's, made code's frames flushed into the heap Larceny's way
     (`flush_all`, `restore`, a stack cache returned into by underflow;
     `fixpt-native/src/direct.rs`, `callout`, `overflow`), which gives
     preemption with them live, `overflow` going on instead of trapping,
     and a collection that reads no stack, retiring C229's copy of their
     values onto tier 1's data stack and back (correct, kept until then). With
     departure 6 folded in too (the user, "Yep"): every `may-collect`
     routine can collect, as Fixpt's `callout` does. Today made code's
     `cons` and `bloblet` bump through all of tier 1's reservation
     (`box[11] = heap-capacity`), never collect, and `panic!` when it is
     full ("the heap it was given is full"): a program bounded in space
     on Fixpt crashes kayak (read in the code, not run). To be: the bump
     against the state's `alloc-limit`, tier 1's threshold; past it, the
     slow path to tier 1 as `foreign` goes, which collects (frames
     flushed) and retries; `poll` alike; the slow path's logic tier 1's,
     in KFX, as C116 and `native.md` decision 6 meant.
   - **Departure 2, the step limit's scope** (the user: kayak to be the
     reference, "we will stop depending on details of Fixpt's listing
     format"): the rule, each top-level form its own budget, a step a cell
     run, is FX-26's, to be written down as kayak's definition says it.
     Kayak's compiler marks each form's start (a primitive cell, `%form`,
     beginning a budget); tier 1 obeys it, and so matches Fixpt and the
     engine on kayak's words. Fixpt's listings, run on tier 1 as a
     cross-check, have no marks: their program one form there, a limit of
     the cross-check, not kayak's. `cells-programs` leaves the marks out
     when it compares kayak's words with Fixpt's. Counting (a step a cell
     on every tier) stays: it is Fixpt's cellular machine's, the one
     compared with.
   - **Departure 3, the engine not Fixpt's evaluator** (reviewed): both
     its effects kept. `native/crossing-overflow.fx`, 3 million crossings
     deep: Fixpt overflows its Rust stack, the engine answers. Messages
     shared, not copied: tier 1 copies (`eq?` of a sent string `#f`, as
     Fixpt), the engine shares (`#t`), both as FX-26's `eq?` allows on
     immutable data (`fx26.md` "Identity": `#f` says nothing). To do:
     C219's note corrected (the `eq?` rule permits the sharing, not the
     transmissible rule); `run-fixpt-programs` to report
     `crossing-overflow` as expected, not as a difference.
   - **Space efficiency** (the user: "some kind of formal safe-for-space
     property, a la Will Clinger's asymptotic space efficiency classes";
     Clinger, "Proper Tail Recursion and Space Efficiency", PLDI 1998):
     kayak's definition of FX-26 to state the class every machine must
     meet (proper tail calls at least; which of Clinger's, `S_sfs` the
     aim, to be settled), and that running out of a resource is an
     implementation limit only where the program's needs under that
     class pass what the machine was given. Then tests of the class, each
     machine's, among them a bounded-memory loop per class of operation
     (I/O, allocation, calls, scheduling), each machine's peak memory
     flat over a 10x longer run (departure 14): Fixpt's crossing (Rust frames per crossing) would not
     meet it. These are FX-26's guarantees, not KFX's (the user: "it's
     okay for KFX to not enjoy the same guarantees that FX26 does"): KFX,
     kayak's language for writing its runtimes, may promise less; what it
     lacks matters only where it keeps one of kayak's FX-26 machines from
     FX-26's promises (so departure 14, kayak-rt freeing nothing, and KFX's
     own object model are KFX's, reviewed as such).
   - **Departure 4, tier 1's code area its own reservation** (the user:
     option 1): kept until the suffix stub lands, then its ordering check
     dropped. To do, on the user's arm64 Mac: kayak's made-code programs
     (`made-code/*.fx`, and kayak's own) every way, which tries both the
     code area reserved above the heap after the spaces (bottom-up) and
     made code run from a `mach_vm_remap` view. The user asked whether
     macOS needs code pages flipped between writable and executable: no,
     not as Fixpt and kayak map them, twice (read+write and read+execute
     views of shared pages, a fixed distance apart, nothing re-protected;
     Fixpt's `exec.rs`); flipping (`MAP_JIT`, per-thread
     `pthread_jit_write_protect_np`) is Apple's blessed way, kept per
     address by dual mapping only in letter. Not known: whether a signed,
     hardened kayak needs `allow-unsigned-executable-memory` for dual
     mapping, as `allow-jit` for `MAP_JIT`.
     **Decided** (the user, "sounds fine"): dual mapping the baseline on
     both systems (no kernel entry when code is written: tier 1 writes
     code bloblets at every seal and every collection's forwarding and
     sweep); `MAP_JIT` with per-thread toggling on macOS a hardening
     option, for W^X in substance or a signed, hardened kayak: the
     state's `delta` then cancels only the tag, tier 1's writes into the
     code area wrapped in toggles, made code unchanged. The instruction
     cache flushed after code is sealed (only instructions; fields are
     data), by user-mode cache maintenance (`dc cvau`, `ic ivau`) on
     arm64 Linux; on the Mac, `sys_icache_invalidate` disassembled to see
     that it enters no kernel.
   - **Through the review**: each rule that is FX-26's meaning, not an
     implementation's choice, flagged to be written down as kayak's
     definition of FX-26. So far: the step limit per top-level form
     (departure 2); `eq?` on immutable data (departure 3, as Fixpt's
     `fx26.md` has it); the space-efficiency class and resource limits
     (above). Then the
   suffix stub note for Fixpt's team, out of the same review: written,
   `docs/research/code-stubs.md`.
   Weak pairs and `%fx26-intern-datum` on tier 1 (C231). FX-26's I/O on
   every machine, the input waited for by one kayak-rt syscall (C232).
   Next: a note for Fixpt's team: code bloblets dispatching by a stub
   in their suffix, so that native calls need no `code-lo` test (C230);
   native code's primitives with tier 2;
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
   (C152), `case` and `typecase` (C153), calls in the fx convention
   and `proj` (C209). Next: a standard operation
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

- KFX bug (C215): a local `lambda` whose captures include a list of data
  holding `Cell`s (the engine's IR) is emitted `Send + Sync`
  (`lambda3s`), which rustc refuses; the checker's crossing analysis of a
  closure should look at its captures' types. Worked around in
  `fx26/lower.kfx`.

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
leave for tier 5 rather than give back; `--direct`; on arm64 too, C210,
measured on macOS). Next, by what keeps words out or makes them leave:
~~allocation as tier 5 allocates~~ (C147 on x86_64, C211 on arm64:
`cellular cons 2`, `prim %region-cons 3`, `lambda` of a closure; where it
would collect, tier 6 leaves first); ~~then the rest of `prim` and
`cellular`, by how often they keep a word out~~ (C212, arm64: `setfield`,
`%make-box`, `%make-frozen`, `%fx26-procedure?`, `%fx26-mul` on fixnums;
52% of the 338 programs' register words tier 6's, 85% but the
scheduler's; `direct-able?`'s `fuller`). Looked at 2026-10-24, on
x86_64, not taken: making the runtime primitives register code
calls most (`%fx26-sched`, `%make-box`, `%make-frozen`, `setfield`) in
place. Every hot word of the benchmarks and of `preempt/allocating.fx`
is tier 6's already; what those primitives keep out is the scheduler's
library and words run once a spawn, a send or a preemption. x86_64's
tier 5 to have them before `fuller` is turned on there. Left, by words
kept out on arm64: `%fx26-sched` (1564, the scheduler's own, tier 1's by
nature); `%fx26-make-icell` (37), `prompt` (13), `%stay-cellular` (13),
`modulo` (13), `abort` (12), `%fx26-error` (11), `exact-integer?` (11);
the control routines (`prompt`, `callcomp`, `abort`, in `captures`).
- Noticed on the way (C210): x86_64's tables never send a block's first
  to its way in to tier 6, as C138 says they do: `yregister-compiled!`
  writes them after `h-6-on` is reset, so `t6-to` gives tier 5's label.
  Tier 5's calls and branches still go in. arm64's tables do (its calls
  are found by them); x86_64's to be made so and measured on Linux.
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

## 50. Taken from Fixpt 0d933ff in part (C393)

- Procedural macros (`%fx26-expand-procedural`, Fixpt's TODO §228) and
  hygiene, binding shapes: done (C431-C446, §51).
- A quoted list or vector compiled as one constant (Fixpt's TODO §229), in
  `fx26/cg.kfx`'s port: an optimization, not taken yet.
- Bounded effect variables, `(e effect (within E))` (TODO §226), in Fixpt's
  checkers: kayak's checker refuses them, stricter, as yet.
- `allof` calls fitting a member by conversion or a module's reshape, and
  a `lambda` argument of no types (Fixpt d667f281): kayak refuses them.
- Macro provenance (`%provenance-id`, the heap's provenances, Fixpt
  9e75efab): with procedural macros, if kayak's front end takes them.

## 51. FX-26 as Fixpt has it (the user, 2026-11-12; `docs/research/fx26-macros.md`)

Track A, by stage: 0 `#%toplevel` (done, C397); 1 the expander's core;
2 inferred signatures and ellipsis; 3 macros that make definitions and
descriptions, `import-with-syntax`; 4 binding shapes and hygiene; 5
procedural macros; 6 the rest of the refusals and the differing lines.
Track B after it: the Rust runtime (`coopt-adoption.md` steps 4-6), and
Twobit-style optimizations in KFX.
- `listof/nested.fx`: a list of lists against `(mu t (oneof (listof t)
  symbol))` refused ("a symbol is expected here").
- A family through `#%toplevel` reads its arguments at the top level too
  (C397): to read them in place.
- Stage 1, the expander (C429-C431): Fixpt's, made KFX by
  `scripts/macros-port.py` (`fx26/macros/expander.kfx`), driven by the
  engine (`fx26/macros-driver.kfx`): 379 of Fixpt's programs beyond KFX
  agree (296 before), the engine refuses each macro program Fixpt does.
  Next: the driver's not-yets (a file's macros, `import-with-syntax`, in
  a top-level `include`, which the engine has not either; `begin-for-syntax`,
  procedural macros', with stage 5; a shape's definition done, C440, and
  `modnames` and `#:relied` with it: a `modnames` inside one not yet; the
  `#:relied` check's names all the body's symbols, stricter than Fixpt's); the REPL: done (C444), its front loading an instance of
  the expander, a file of a place (C442, C443, Fixpt's `closed-code.md`);
  the legacy `rust-repl` (live-repl.kfx) not yet; a live session's drafts
  and hints with macros untested; the `expands` effect, which C437's
  instance, made in a `letrena`, masks (still carried by `engine-main`
  and start.kfx's `main`); each form's expansion cached, by the names it
  reads, folded into kc (the user, 2026-11-16): done, C445, kc's step 15
  (`kc/expand.kfx`, `scripts/kc-expanded.py`: 261 of Fixpt's 269 macro
  programs as the engine expands them), but what no written definition
  gives a later form: a macro a use makes, or a `hide` holds (6
  programs); a global a use defines, its shape known (2): kc refuses
  them, not yet (each wants a form's query to read what an earlier
  form's expansion made, by name); two `define-syntax`es of one name
  each given, untested beyond a trailmark's; procedural macros (stage 5):
  done, C446, each transformer's request (Fixpt's `%forms`, `%define`,
  `%apply`) answered by the engine's `--phase1` (prelude, phase-1 forms,
  the transformer checked at its style's type and run, a process of its
  own), in kc an input `phase1 REQUEST`, the expansion asking again; in kc
  answered by kc's own native code (C447), the request's phase-1 program
  a file of the database, its forms queries, built and run: a
  transformer's program built once, each use a run of it given the use
  (C448, kc's reader, `--apply`), a use it does not read (a vector, `#u`)
  the call compiled; in the REPL answered by the engine's `--phase1`
  (C448), drafts of the answers known alone; not yet a transformer's
  answer a vector, nor, in kc, a use or a quoted datum that is one; a
  typecase of an `f64` or `f32` in kc not yet (kc has no floats); an
  expansion run once, its requests asked as it goes (C449); a
  transformer's image kept sealed, in process, once no use names it
  (C454; no longer built into `DB.phase1/`); the driver's lines (each list's, met; an atom's,
  the last list's before it).
- An expansion run again for each new transformer request (C446): done,
  C449 (the user, 2026-11-17: "remove the quadratic case"): a request is
  asked as the expansion goes, of the host's asker (kc's `phase1` query;
  the engine's `--phase1`), the expansion going on with the answer, as
  Fixpt's does. An explicit renaming chain of 200 uses, cold, in kc: 18.6
  s, now 0.57 s. Left: the engine's asker runs the engine again for each
  request (5 s at 200, kc's being a run of a transformer built once), and
  in-memory code (the user's steps 1-5, C449: one byte emitter, kc's own
  convention) would make each a call; `models/db.qnt` has no input set
  fresh (C449's `db-set!` of a key no node is of, no new revision); the
  expander written twice (expander.fx, expander-kc.fx), one asker's
  effect each, as KFX's Rust writes no module type of an effect
  parameter (a file of a place and an effect, FX-26's, kfx's to take).
- A loaded file's type named as one of the program's (the expander's
  `part`, lib/db.kfx's datatype `part`, C445): one type in the Rust kfx
  makes, which rustc refused. The port names the expander's apart; kfx
  should refuse it, as a module's ("a module's type named as the
  program's"), or keep a load's types apart.
- Found by macros' expansions, the checker's (C432): a keyword's
  construction, `(#:circle 2)` (`macros/define-data*.fx`, and
  `run/keyword-construct.fx` with none); a `moduleof` a template makes,
  its labels keywords (`macros/desc-moduleof-given.fx`), which fails, "car
  or cdr of a non-pair", where it should refuse (TODO §53); `->*` where a
  definition's type is not (`macros/defn-rec.fx`); a definition of no type
  whose type is not known (`macros/defn-hidden.fx`); mutable data at no
  region (`macros/swap.fx`, `many.fx`).
- A datatype's constructor named as a global already is (`place-named`,
  C442): kfx took it and its synchronous Rust built, the async mode's not
  (a call of the function then a future no one awaits). To refuse it, as a
  name defined twice.
- `(with #%toplevel get)`, as an expansion writes it (C444): the effect
  says the region unsolved, `(read ?0)`, where `get` says `(read @g)`.
- A unit's loads (C443) are in no memo's key: `kfx --cache` would keep
  a form's Rust though a file it loads changed.
- `kfx --cache` on the engine's unit fails, "substring range out of
  bounds", with or without the expander (C432, found measuring): the
  memos are used for kfx's own unit alone, so far.
- A `datum` shared, in Rust: it is data made and never written, but it is
  a `Value`, which holds a union's procedures and cells too, so no
  procedure another thread may run holds one (C102). Three of Fixpt's
  expander's procedure types are given `(read rs)` for it
  (`macros-port.py`'s `PATCHES`, C430); a `Datum` of its own in kayak-rt,
  `Sync`, would end them.

## 53. Taken from Fixpt 120b70e in part (C428)

- A `moduleof`'s label and binder written apart, `(abs #:t u type)`,
  `(desc #:d u D)`, and a `val`'s label a keyword, `(val #:x T)` (Fixpt
  687accf1): kayak's checker refuses them ("a module type's item KFX does
  not have"), stricter, as yet (`modules/moduleof-*-apart.fx`,
  `moduleof-label-with.fx`). Where the label is named nowhere free after,
  the binder renamed to it is the old form; where it is (an outer `t`
  beside the label `t`), module specs need a label and a binder apart.
  Program text is read at many places (kfx, the engine, kc), so the
  rewrite wants one pass they share, not the reader (quoted data).
- `get-environment-variable`, `(subr (io env-read) (string) (sumof (value
  string) (unset unit) (not-text unit)))`, and the I/O kind `env-read`
  (Fixpt be3b4291): kayak's checker does not bind it (stricter); every
  layer would take it as `read-symlink` was taken (C293): the checker's
  standard list, the emitter's I/O names, kayak-rt, the engine's
  primitives, tier 1's `cprims.kfx` (primitive 526).
- Fixpt's macros since 2026-11-07 (typed tier, `(macro σ made)`
  signatures, `desc`, trailmarks, `begin-for-syntax`, procedural macros in
  a phase-1 namespace, expansion limits): §51 ported the pin's expander
  (C429-C446, `docs/research/fx26-macros.md`).

## 52. kc (the user, 2026-11-12; `docs/research/kc.md`)

The compiler from scratch, of FX-26 and KFX, its passes queries. K0 the
engine (`lib/db.kfx`, done, C398); K1 step 1 end to end (done, C399) (the driver, the
front end as queries, integers to async Rust and to native code, the
harness); K2 onward Ghuloum's steps; the service after K2's first steps.
- The engine: generic over effects and regions; values kept decoded in
  memory beside their text (each read decodes it now); a body that fails
  leaves its frame (an error ends the run today); asynchronous bodies on
  kayak-sched (Coopt's: a waited-on query a task); nodes no key reaches
  any longer collected; durability's shortcut measured.
- kc: a refusal is an error that ends the run, so it is no query's value:
  a checked program's refusal to be one, said, and kept, as any value.
- kc: each query reads its file's forms again from the text (`kc-forms`);
  the forms as a query's value once values are kept decoded.
- kc's step 15, macros (C445): each form's expansion a query, by the
  names it reads (TODO §51); a macro's checks (a typed one's hidden
  definitions) checked with the program; a file's macros
  (`import-with-syntax`), loads, not yet; procedural macros and
  `begin-for-syntax` (C446), their transformers compiled by kc and run
  (C447): a request's phase-1 program a file of the database, each
  transformer's items queries, a use's call the one new item. Step 14
  (the effects that need the runtime) not done before it: the macro
  programs kc takes are those its back ends take.
- kc's code in memory (the user, 2026-11-17, C449): "what if we drove
  ahead to get ourselves to building executable code in the GC'ed heap,
  the same way Fixpt does?" The plan, each step a C-entry and the full
  check: (1-2) x64.kfx's instructions as data, encoded to bytes by kc
  everywhere (the user: "surely it's better to just use the same byte
  emitter everywhere, as that is the thing that's most likely to expose
  bugs"), `kc build` writing ELF objects that `cc` links, the assembly
  text kept for comparison: each step program's objects as `as` makes
  them of it, byte for byte (done, C450: kc's assembler of the text,
  kc/asm.kfx, its objects `kc build`'s, scripts/kc-objects.py; the
  text made instructions as data, not yet); (3) kc's own convention (the user: "I'm
  okay with us inventing our own variant"; data stays Fixpt's): a stack
  limit, a failure that returns to the trampoline, a state register kept
  for fuel; linked into kayak-jit's code area, `kc run` in process, each
  step program as its executable answers (done, C451: FX-26's
  `seal-code` and `native-procedure`, the heap a shared flat array, the
  runtime kcrt's, its routines' addresses by a syscall, C452; left: `poll` gives fuel, no
  scheduler takes it; an absolute address in the image (none needed
  since C454, `kc_apply_address` relative to the code)); (4) a heap of Fixpt's layout
  that KFX owns, a region a transformer's call drops first, then the
  collector, code too (Fixpt's "A collected code area") (the collector
  done, C453: Cheney's, kcrt-core's, its roots every word of kc's stack,
  each frame zeroed, and the globals' cells; left: the
  region, done with step 5 (C454); kcrt's own objects, made in kc's heap
  and collected (C455: its strings and pairs; its symbols kept for the
  run, as Fixpt's table keeps them); kc's
  code freed (C456: the code area's room given back first fit, as
  Fixpt's; kc says what, as kayak-rt traces nothing, and a generation
  per block makes a call of freed code a failure; a transformer's uses'
  images, 16 most recently used kept; left: what the program forgets to
  free stays, where Fixpt's collector would find it); no
  generations; a large bloblet; tier 1's collector, KFX's, and this one,
  Rust's, two copies of Cheney's over Fixpt's layout, to-discuss 15);
  (5) `phase1`'s body a call of the compiled transformer, no process
  (done, C454: its image sealed once, each use a call in process, its
  heap one array used again from its start; left: a transformer's run
  no longer a process of its own, so a fault of its code (a bad store,
  a loop with no end) is kc's; kc's images freed, C456).
- kc's front end fails (`car or cdr of a non-pair`, in `kc-unlocated`)
  on a transformer's `(typecase (car (cdr form)) ...)`, a typecase of no
  local, where the engine refuses it ("a typecase of a local of a union",
  C62): a refusal, not a crash (found by C454's benchmark).
- Models (`docs/research/models.md`): exhaustive checking (Java 17+ for
  Apalache; the snap's OpenJDK 27 works with `QUINT_HOME` and a temporary
  directory it can see); the model of the asynchronous engine when it is;
  the model's traces replayed against the KFX engine; Kani, if wanted.
- The checker refuses `(null? '())` ("a type null? is called at that
  nothing here says"), which Fixpt takes (`#t`): stricter, a gap (found by
  kc's step 3, C402).
- The checker refuses `((proj new @h) 5)` ("a proj of new given the wrong
  number of arguments"), which Fixpt takes: 8 of Fixpt's programs (C426's
  sweep of kc over the corpus).
- The checker refuses `(cons (list 1) (list 2))` ("a int is expected
  here"), which Fixpt takes as a pair whose tail is a list of another
  type, `((1) 2)`: stricter, a gap (found by kc's step 7, C406).
- kc's edit loop (C414, C415, `scripts/kc-bench.py`): the check is
  memoized and a refusal a value, and `kc serve` keeps the database in
  memory; still of the whole program, the memoized check's walk over every
  form (0.6 s at 5000 procedures): a form's check a query of its own, its
  inputs its form and the bindings it reads. A `not yet` of kc's own still
  ends `kc serve` (an `attempt` around a request needs the request
  cancellable, which a database's writes are not). kc's modules (C425): a module as a value (`with` of a
  variable), makers, nested modules, `extend`, and modules in kc's Rust back
  end. kc's types (C423): a product's field by name
  (`extract p x`, `(product (x ..))`), its position the checker's (kfx's
  emitter reads an `extract-index` fact; kc's `checked` keeps no facts
  yet); sums, products and datatypes in kc's Rust back end. `kc build` (C419): objects no longer listed
  pruned from `EXE.o.d/` (done, C448); an edited procedure an object of its own (weak
  symbols in its run) should reassembling its run of 64 ever be seen.
- KFX's datatypes stay nominal (C410, the user's choice after the
  measurement): only `sumof` is structural. Were it revisited, the
  checker's types would first have to be shared (Fixpt's arena of
  `TyId`s) or a datatype kept by its name and unfolded where its shape is
  read; the expansion as Fixpt makes it is `git stash` "C410 WIP".
- `lang-items.py` lists the emitter's calls of kayak-rt by `NAME(`, so
  `sum_tagged::<P>(` (C499; `sum_new` before) and `sum_payload::<T>(`
  are not listed (C409), nor the macro `tag_lit!(`.


## 54. The expander's state in kc's collected heap (the user, 2026-11-17)

The user: "I believe I do want the expander's state in the collected
heap", and "work via those steps". As Fixpt has it: the expander runs in
the heap its transformers do, its bindings traced roots, a transformer's
code freed when its binding is, by the collector. kc's expander today is
Fixpt's (fx26/macros/expander-kc.fx), compiled by kfx into kc, on
kayak-rt, which traces nothing; so C456 frees a transformer's image when
kc's cache drops it (16 kept), not when nothing reaches it. Steps:
1. **Survey** (done, C457): kc made to compile the expander; what it
   has not yet, sized.
2. **The gaps closed** in kc's native compiler, each a step with its
   tests, until `kc build` makes the expander and it expands as kc's KFX
   one does (`kc-expanded` the yardstick).
3. **A heap that persists**: one for the expander, collected, never
   reset between forms and requests.
4. **Transformers as closures in that heap**: a `define-syntax`'s
   transformer compiled into a code bloblet of a collected code area,
   its binding the closure; a use a call of it.
5. **Code collected**: kcrt's collector marks code bloblets the heap
   reaches and sweeps the rest, as Fixpt's area (C456's room and
   generations under it); the cap of 16 gone.
6. **Incremental collection**: marking spread over the mutator's work,
   a write barrier; past Fixpt's, which stops the world. Its design to
   the user first; whether in this plan or after, to ask.
7. **The database at the boundary**: kc's front end asks the native
   expander a form at a time; memos hold data (expansions' text), never
   heap pointers.

The survey (C457; step 2's list): kc's checker takes the expander whole
(flattened: its types file inlined, its place one region, its items a
`module`'s); its code generator refuses, by kind, occurrences counted
over its 1664 definitions, 335 of them refused:
- ~~**Tuples** (`tuple`, `extract`): 1484.~~ Done (C461): Fixpt's
  products; KFX's checker expands a `tuple` an unquote holds. The survey
  then: 110 items refused, were 320; `reverse` 725, `append` 346,
  `int->string` 161, `i64->int` 43, `int->i64` 42, more than six 61, the
  type-level names as before.
- ~~**List procedures as forms**: `reverse` 519, `append` (not `,@`'s)
  325, `length` 3.~~ Done (C462), with `int->string` and the `i64`s
  below. The survey then: 13 items refused, were 110; left, more than
  six (39 calls, 26 procedures), the type-level names, the strings'
  and characters' rest, `f64`, a vector.
- ~~**Delimited control** (the parser's): `prompt` 112,
  `abort-current-continuation` 4, `make-continuation-prompt-tag` 1.~~
  Done (C458): prompts as frames, aborts that walk them; the survey's
  refused items 320, were 335. Capture too (C459): composable and
  one-shot continuations, the frames to the prompt copied into the heap.
  Whole continuations and marks too (C460). Left: a copy past a
  header's fields, `letregion`, the Rust back end's control.
- ~~**`i64`**: `int->i64` 39, `i64->int` 32, `i64+` 14, `i64>=` 10,
  `i64=` 9, `i64-quotient` 3, `i64<=` 2, `i64-` 2.~~ Done (C462): an
  `i64` an `int`, a fixnum (past one kc fails, Fixpt wraps).
- ~~**Strings and characters**: `int->string` 149 (C462), `char->string` 11,
  `char-whitespace?` 6, `char-numeric?` 7, `string-hash` 6,
  `symbol-name-hash` 4.~~ Done (C470), with `f64`, the vector and `error`
  below.
- ~~**More than six**: 26 procedures' parameters, 32 calls' arguments.~~
  Done (C464): to 64, the rest in `kc_args`; and `string-append` of any
  count. The survey then: `not yet` 264 times, was 343. Left: `vlambda`,
  `vsubr` and `apply`, which the expander has none of.
- ~~**Type-level names reached as values**: `expr` 26, `data` 25,
  `bvar` 24, `id` 22, `lambda` 18, ...~~ Not names at all (C469): the
  data of two `case`s (`read-sty-name`, `mx-parser-forms-kind-form?`),
  which kc compared with the variable of the datum's name, unquoted; a
  miscompile, as was a `case` on strings compared by `eq?`. Both fixed.
  The survey then: 11 items refused, were 13; `not yet` 46 times, was
  264, all of the strings' and characters', `f64`, a vector and `error`
  below.
- ~~**`f64`**: `f64?` 6, `f64->string` 4. **Vectors**:
  `datum-list->vector` 4. `error` 1.~~ Done (C470): Fixpt's layouts (a
  flonum kind 5, a vector kind 3) and Fixpt's hashes. kc's programs make
  no `f64` yet (no literal, no `int->f64`), so `f64->string` is tested in
  kcrt-core alone, against the pinned Fixpt's strings. The survey then:
  **no item refused, no `not yet`**: kc compiles the whole expander, its
  9.4 MB of assembly accepted by GNU `as` (1583 procedures). Step 2's
  code generator's gaps closed; what is left of step 2 is below.
- **kfx's reader has no `|…|` symbols** (found at C470): `kfx/read.kfx`,
  kc's and kfx's reader, reads `'||` as the symbol of two bars, Fixpt as
  the empty name (R7RS). The expander's source has none. To be fixed in
  the shared reader (a kfx change: its fixpoint), and its writer to bar
  a name that needs it.
- ~~**A file of a place**: kc's checker refuses `module-parameters` as a
  program (`a region KFX-0 does not have: mxp`); kc's code has one
  heap, so the place can be it.~~ Done (C471), by the route the user
  chose, loads in general: kc reads what a program loads, splices it
  in (kfx's `with-loads`), and lowers a file of a place's maker, each
  instance a tuple of its interface. A kc program loads the expander
  under a `letrena` as `kc-instance` does, and its native code gives
  Fixpt's answer (`tests/kc/loads/expander-facts.fx`), the executable in
  4 ms where Fixpt's interpreter takes 3.4 s. Its cost: §55 item 11.
  Left of loads: a loaded file's macros are not expanded (kfx's way:
  its text spliced as read; Fixpt expands it); a program's top-level
  `include` of a module, which kfx's checker refuses and Fixpt accepts;
  a `define-datatype` in a file of a place (`not yet`).
- **Module paths, `a::b`** (Fixpt's TODO §296, at the pin since C474):
  kc's expansion a form at a time knows not `m`'s shape where a path's
  `with` asks it (`#:relied #f`), so kc refuses `m::x`, which the engine's
  expansion of the whole program, and Fixpt, take
  (`modules/colons-macro.fx`, kc-expanded's KNOWN). And the engine's `with`
  takes a variable alone, not a path (`parser-exps.fx`'s `with-module`).
- **Gaps of the pin's (C474)**: Fixpt's interned symbol's name now a
  *text* (an immediate, its heap's name's number, `value.rs`'s
  `IMM_TEXT`), not a string; kayak's symbols, kc's and the engine's, hold
  a string still (Fixpt's formats everywhere: to follow with §55 item 2's
  heap design, a name table each heap). `gensym` (FX-26's, its own effect)
  neither kc nor the engine has.
- Not seen by a survey of the code generator: what runs (an `int` past
  a fixnum, which kc fails; ~~the expander's hashes~~, Fixpt's since
  C470), and the expander's host: its asker, the bridge's data, its files
  read.

- **kc atop kc: the survey** (C477; the user, after C476, asked for kc's
  code on a collected heap, then "Survey first, then decide"; recorded at
  their word, not chosen): kc's own program, its unit's 26 files made one
  (32,723 lines), its syscalls let declare. kc's checker takes it whole
  (48 s); kc's code generator refuses 355 items, `not yet` 4956 times, of
  107 kinds:
  - **`i64`** (3222): KFX's names (`i64->string` 1843,
    `string-length-i64` 481, `string-ref-i64` 301, `char->i64`,
    `i64-and`, `-shl`, `-shr`, `-xor`, `fnv-string`), and 17 literals
    past a fixnum (FNV's): kc's `i64` a fixnum (61 bits) where kc's
    hashes wrap at 64; a 64-bit integer in kc's native code, boxed or not,
    the new mechanism.
  - **Modules' `define-rec`s** (647, the largest names `place-of` 224,
    `poly-type?` 114, `expressions..nlist-size` 41): kc flattens a
    module's `define`s by renaming (C425), not its `define-rec`s, so the
    calls renamed find nothing.
  - **Products by field name** (543): `(extract d rev)`; the field's
    place the checker's type's, which kc's lowering has not.
  - **Strings and characters** (264): `list->string`, `string->list`,
    `char-downcase`, `has-char?`.
  - **The system** (170): files, processes, `args`, output, `exit`,
    `define-syscall`, the code area (`seal-code`, `kc-routine`).
  - **Bytes** (99): bytevectors, flat arrays, `make-bloblet`,
    `utf8->string`.
  - **`attempt`** (11): a failure caught as a value (the database's
    refusals); kc's native code's failures end the run.
  Not seen: whether the native kc answers as kfx's Rust does; its heap
  (kcrt's Cheney copies all that lives, kc's being large); the bootstrap
  to a fixed point. The other way weighed (C477): a collector in
  kayak-rt, conservative and not moving (every value a `&'static`, no
  stack maps), 4 to 6 steps, its errors use-after-free a stress mode
  finds; for every KFX program built to Rust, not toward K8's heap.
- **Handles for kayak-rt's heap, toward a collector** (C478; the user's
  intermediate step, after C477's weighing): every datatype's value, list
  pair and ref of KFX's Rust is a `kayak_rt::Handle<'r, T>` (7929 of kc's
  type mentions), made only by the heap's and regions' allocations. Left
  as references: closures (`&'static dyn Fn`, 345 in kc), bloblets, flat
  arrays (`Array` a newtype already, its field public), kayak-rt's own
  objects (`leak`: continuations, mailboxes, tables). Then ~~a `Trace`
  the emitter writes for each datatype~~ (done, C479: every datatype and
  shape, kayak-rt's own types; a reference opaque, a closure's captures
  too); ~~a census of what the stack reaches~~ (done, C480: at an edit's
  build, 2.5 to 8 MB of values and 2 to 4.4 MB of strings reachable
  while 155 MB is allocated; closures reach next to nothing); ~~closures
  as FX-26's heap grammar has them~~ (done, C481: a closure is its
  captures' traced fields, their plain bytes, its code; a procedure value
  a handle to `dyn CallN`, called by `.call`; `Trace` split from `Visit`).
  Left of closures: ~~kc's own Rust back end (`kc/rust.kfx`) still writes
  `lambdaN` and `&'static dyn Fn`~~ (done, C493: its closures kayak-rt's,
  `lambdaN` gone); ~~a continuation's closure (`Resume`) and a `letrec`'s
  knot reach their values through plain references~~ (C492: a knot
  visited; C493: a continuation conservative, as the stack is, its
  future's own block, `Box::pin`'s, to be made one a collector knows); ~~a closure of no captures is
  made at each evaluation (9000 of them at an edit's build), where one
  made once would do~~ (done, C494: a closure's code its own type, no
  pointer, so one of no captures no bytes, allocating nothing); and the corpus's programs of another machine
  (`made-*.rs` but x86_64's) are to be blessed again on it. Then the rest
  of the heap to the same grammar (the user: "everything conversion if
  you can swing it"): ~~a string~~ (done, C482: one word to an object of
  Fixpt's layout, header, trailer and text, `obj.rs`; a gap: its trailer
  keeps whether the text is ASCII, which Fixpt's runtime does not, and
  finds again when it needs it), then every bloblet-shaped object the
  same way (the user: "why not just handle all bloblets this way?"):
  ~~bytevectors~~ (done, C483), ~~bloblets~~ (done, C484: `BlobletAt`,
  its fields typed in its object; a sealed copy's code in a table of the
  thread's, its suffix frozen by its header's bit), ~~typed-suffix
  bloblets and arrays~~ (done, C485: `TBloblet` with Fixpt's header and
  trailer; `Array` one word to a vector's object, its elements the
  fields, a large one's count in Fixpt's second header word). Left of
  them: ~~an array in a typed suffix (`FlatArr`) boxed apart~~ (done,
  C487: the user allowed `unsafe` for it; `Flat` an `unsafe` marker,
  all zero bytes valid, the emitter's flat products implementing it; a
  typed-suffix bloblet one block made zeroed, one word, traced); ~~flat
  arrays (`FlatArray`, Fixpt's header already in its first cells, a
  plain reference to them)~~ (done, C489: one word, traced); ~~bloblets and arrays opaque to a trace~~
  (done, C486: each object reached once, its fields visited; a census
  traces them precisely); **Fixpt words in the heap** (the user's choice,
  2026-11-20, over types by page and headers but pairs: every value a heap
  object holds a Fixpt word, raw only off the heap, as Fixpt's native
  code; done in part, C495: `HeapWord`, `Trace::Stored`, a pair's, a
  cell's, a sum's scalars, integers and strings words; C496: a closure's
  captures and a product's object words too, `split` gone; C497: a list
  one word (Fixpt's empty list, or its pair's address tagged `001`), a
  pair Fixpt's headerless two-word cell; C498: a ref one word, tagged
  `100`, to Fixpt's box (kind 10, `RefObj`), and every kayak-rt object's
  header counting its trailer, a large array's extension word before its
  header, as Fixpt's; C499: a sum one word, tagged `100`, to Fixpt's
  sum (kind 36, frozen: its tag, a word to Fixpt's symbol, kind 2, made
  as rustc builds, `tag_lit!`; its payload, a word), its tag's
  discriminant its symbol's hash, Fixpt's `symbol_hash_of`, not SipHash.
  Left of a sum: its payload's type kept in one field Fixpt's has not,
  before its trailer (`SumDesc`, what traces it and checks a read of its
  payload), until its payload's word says its type itself; a tag
  symbol's name a string's word, where Fixpt's is its heap's text, and no
  global's place (−1); ~~kayak's `symbol` values not yet Fixpt's
  symbols~~ (done, C504: one word to Fixpt's symbol, a quoted one's made
  as rustc builds, `sym_lit!`; `string->symbol`'s in the heap), not yet
  one of each name (each `string->symbol` a new object; `eq?` compares
  objects, then hashes and names); C500: a datatype's value one word, to Fixpt's
  sum (36) whose payload is a word to Fixpt's product (37) of its
  variant's fields, as Fixpt's `define-datatype` expands (the user's
  choice over one sum of the fields in line, 2026-11-20): the two
  objects one allocation of kayak's; each variant's tag a symbol of its
  own, which a read of its fields checks; a `tagcase` an `if` per arm,
  only its variant's fields read. Its cost: the heap +33% (354.3 to
  470.4 MB at the 1000-procedure bench), an edit's build +20%, a cold
  one +11%. Left of a datatype: its fields' words those of the types
  not yet one word (a closure, an array, a union) as they are; a place's
  values boxed one by one (`DataBox`), not bumped; C501: a procedure
  value one word, to Fixpt's native closure (kind 41, frozen,
  `[free…][code][trailer]`, free value 0 nearest its code), its code a
  word to a frozen bloblet of its type's call and trace (where Fixpt's is
  a code bloblet of machine code); a closure of no free values an object
  too (24 bytes; 254,000 made in a cold build), where it was none. Left
  of a closure: kept in a box of its own (`Box::leak`), not the heap's
  arenas; a union holds it boxed. C502: an array held as Fixpt's word to
  its vector (kind 3), tagged `100`, one past its trailer, as a flat array
  (kind 42) its word to its first element, where each was held as its
  untagged pointer; a large array's trailer counting back to its main
  header, as Fixpt's, where it counted to its extension word. Left: an
  array in a union boxed; ~~kayak's datum vector (`Vector`) not Fixpt's~~
  (done, C505: an array of its elements, Fixpt's vector);
  C503: a bloblet's fields each a Fixpt word (`Slot`, a product's boxed),
  where each was its value in line (`Cell`), and a bloblet held as Fixpt's
  bloblet word, tagged `100`, one past its trailer, a typed suffix's too; peak memory up 3 to 6% at C495, not explained; a cold build's
  bignums, 82,546 (2.6 MB, of 36.0 million allocations) with kc's object
  cache empty, not yet traced to where its 64-bit integers are made);
  ~~a dynamic value (`Value`, an enum)~~ (done in
  part, C491: traced, its pair and its members' objects reached
  precisely, a member its own object, no box, but what has none yet; a
  sum precise, its head its cell's type (C499: its object, as its
  payload's type has it, found again by `Visit::sum_cell`). ~~Left: one word, not three,
  which needs every object it may point at to say its shape and type as
  Fixpt's headers do: a pair's, a datatype's value's, a sum's, a ref's,
  a closure's~~ (done, C506: one Fixpt word, its shape its tag's or its
  object's kind's; a typed list a list of values as it is; an array
  Fixpt's plain bloblet, kind 32, as Fixpt's `make-array`'s; a product
  held as its word. Left: a member narrowed by its kind, not its type,
  its element types the checker's to keep, as Fixpt's native code's,
  to-discuss 17; a member whose stored word is not Fixpt's own, as yet
  `&Slot`'s, copied where given, not narrowed again; a census reaches an
  array's, a box's, a product's, a bloblet's and a datatype's value's
  object from a value by its block, so a trace without one counts them
  opaque) ~~what is still traced as a plain reference, opaque~~
  (done in part, C492: a reference to what is `Visit` visited, a place,
  a module's state, an I-cell, a mailbox, a knot, a region's arena.
  Left opaque, each by an impl of its own: a continuation, scanned as a
  stack is (C493), and a blobref into a typed suffix, an interior
  pointer; `lambdaN` gone, C493), ~~a product held by
  value in its holder~~ (done, C490: boxed where a heap object holds it,
  `Prod`, one word to Fixpt's product object, kind 37, by its `split`: a
  pair's element, a sum's payload, a ref's and an array's cell, a
  closure's capture; on the stack its parts; the user: no unboxed
  products holding references. Left in line: ~~a bloblet's field~~ (C503), an
  `Option`, a datatype's field of a product type (none in kc), a
  message, an I-cell's value, a continuation's and a knot's), ~~an integer
  (`Int`, a bignum's limbs a `Vec`)~~ (done, C488: one word, Fixpt's
  61-bit fixnum or a bignum object, kind 6, its limbs in its suffix); then a
  collector, the Rust stack conservative and the heap
  precise (Bartlett's mostly-copying), its stress mode at every
  allocation (the user, after C506: "Go for it"). Its steps:
  1. ~~**One space**~~ (done, C507): every heap object (the arenas' types,
     strings, bignums, flonums, products, closures, arrays, bloblets,
     bytevectors, flat arrays but a big one of 0s) made by bumping in a
     page of the thread's (32 KB, from chunks of 2 MB the process shares),
     a bit per word where an object starts, so an object is found from
     any word inside it, tagged or interior; an object past 8 KB a block
     of its own. The arena per type gone (`heap.rs`). Its effect: an
     edit's build at 1000 procedures −15% instructions, a cold one −10%;
     the expander's (`procedural-many.fx`) −13% instructions and time,
     −7% page faults and peak memory. A census (`heap-stats`) finds each
     stack word's object by the bits, each a noted object's or a
     string's. Left: a big flat array of 0s (`zeroed_cells`, reserved,
     C139) not the space's; the space's chunks only appended to.
  2. ~~**What is held as itself, a word**~~ (done, C508): no heap object
     holds a value as itself, asserted as rustc builds (`Trace::WHOLE`,
     checked by each maker: a cell, a pair, a sum, a product's fields, a
     closure's captures); a bytevector its word (kind 4); a plain
     reference (an I-cell, a mailbox, a knot, a place, a module's state)
     its address, a word of tag `000`, which a collector neither traces
     nor moves, kayak-rt's own objects aligned to a word for it (one of
     no bytes, an empty region's, fixnum 0). An object read by its layout
     alone (`obj::laid`): a census traces the stack's roots both ways, and
     the layout reaches every allocation the types do, and the bignums
     and bytevectors the typed trace leaves uncounted (13 marks of a cold
     build at 1000 procedures). Left: `Option`, a handle, a continuation,
     a blobref and kayak-jit's `Code` still held as themselves, which the
     assertion keeps out of every heap object (none is, in the
     workspace's programs and the check's); an I-cell and a mailbox
     kayak-rt's, not Fixpt's objects (kinds 46, 47), what they hold the
     roots' (step 3), as the shared heap's to be (§55 item 2); a
     string's and a bytevector's header counting no field though a
     trailer follows it (C482, C483), where Fixpt counts the trailer in
     F, read so by `laid`. Larger than that (found after C508): kayak-rt's
     string is not Fixpt's at all. Fixpt's is UTF-32, no fields or a
     length field, code points two a word (`docs/object-model.md`'s
     table; kc's native runtime, `kcrt-core`, and kayak-jit's `string`
     lay it so); kayak-rt's (C482) UTF-8 after a trailer, so that
     `Str::as_str` lends a `&str` with no copy. The user's choice
     (2026-11-22): UTF-8 put to Fixpt, as table stakes for interchange,
     `docs/requests/fixpt-utf8-strings.md` (a length field, ASCII when
     it equals the bytes, breadcrumbs every 64 characters, mutable
     strings fixed-width); kayak-rt's UTF-8 a gap kept till Fixpt
     answers, then made Fixpt's layout as it says.
  3. ~~**Roots**~~ (done, C509): `obj::roots`, a registry for each
     thread: its stack from its base (the OS's), the callee-saved
     registers spilled in a small frame, a safepoint's; the constants'
     slots; blocks owned while they live (a region's arena's, a future's
     box, `pinned`, the emitter's async mode's too, a place's datatype
     value's, an async process's failure closure); what kayak-rt keeps of
     its own, scanned when reached (by its bytes, or a mailbox, an I-cell,
     a group by its type); thread locals (the marks, a group). A mark by
     them and the layout alone (`roots::reached`): on a cold build at
     1000 procedures, 13 marks, every allocation the typed trace reaches,
     and strings it cannot start from. Left: a thread's roots its own (a
     future or region moved to another thread under its maker's); a
     continuation never resumed keeps its future a root; not yet roots: a
     procedure given to native code (`foreign_procedure`), a capture on
     its way to its prompt (`CAPTURE`), a `spawn`'s closure before its
     thread runs; tables keyed by address (a sealed bloblet's copy, the
     string cursor) to update when an object moves; stale words in frames
     pin (kcrt-core zeroes its frames, C453). Its cost (C510): a pair
     and a cell of code with lifetimes now the space's, not a `Box` kept
     beside it (the synchronous compiler as fast as at C508); every
     boxed future still registered and given back, its thread's slab
     with no lock, which leaves the async compiler 11–14% slower than at
     C508. To come, if it matters: only a task's first future
     registered, the mark following words into kayak-sys's allocator's
     blocks (C370's, made for these boxes) instead.
  4. **Collect** (done in part, C511, C512): `obj::collect`, a thread's
     pages, those its ambiguous roots reach pinned whole, the rest copied
     (Bartlett's), each moved object's first word a forwarding word
     (Fixpt's tag 111), the precise fields found by the layout (`laid`);
     allocation the safepoint (`space::refill`, once the pages taken
     since the last pass reach twice those kept, 64 MB at least). Its
     stress mode (`KAYAK_GC_STRESS=n`, a pass every nth allocation) under
     the check (`scripts/gc-stress.sh`: kayak-rt's tests at 7, the corpus
     at 97, its programs of processes and `par` at every allocation, kfx
     making itself at 300007, about 75 s). A thread collects as the
     others run (C512): a live thread's objects read, never written, what
     they hold of its own pinned; an ended thread's written where they
     point to what moved, never moved; what another thread may reach of
     its own pinned whole, as a constant's: what each thread's list
     (`roots::Reach`) reaches (what its last pass found it reaching of
     others', what it took since, what its maker lent it) and what shared
     kept objects hold (a mailbox's messages, an I-cell's value, a group's
     failure closure); a `par` branch's pages, list and kept objects its
     maker's at the join. Measured: kfx making itself 5.45 s, 375 MB at
     its peak (`KAYAK_GC=0`: 3.85 s, 2.16 GB; C511 said 5.64 s, 3.17 GB,
     C510's build); kc building `expander-facts.fx` 136 s, 31.6 GB (none:
     129 s, 48.2 GB; what the rest is, step 5's). Left: `par2_shared`'s
     branches (`unsafe-par`) collect not at all while they run
     (`space::Concurrent`), as each writes the other's objects; an ended
     thread's pages never given back (a process's); what other threads
     reach followed whole at each pass (a large structure shared, read at
     every pass of every thread), and other threads' objects read as they
     run (a race a word wide, as a conservative collector's); a message
     shared, not copied (Fixpt copies it); large objects never moved or
     freed; kept objects all roots, never freed (a continuation, an eq
     table, a procedure given to native code, every mailbox and I-cell,
     scanned by every pass); kayak-rt code holding values in a `Vec`
     across an allocation marked by hand (`space::NoCollect`); a
     scheduler's boxed tasks not yet roots; aarch64's d8–d15 not spilled;
     a sealed bloblet's copy pinned through its table, not moved; the
     census's mark (`roots::reached`) heap-stats only.
  5. **Measured on kc**, then incremental collection's design to the user
     (§54's step 6 above).
- **What a fresh `kc` allocates** (C476; the user's "what the hell"): an
  edit's build at 1000 procedures 154 MB, kept for the run (kayak-rt has
  no collector), half its time the kernel's page faults. Left, largest
  first: the reader's lists of a text's characters (`read-all`'s
  `string-chars`, 36 MB: to read by index), texts read more than once
  (a form's text written then read back, `kc-load-forms!`; the memos
  replayed whole), the database's maps copied at each change. The whole
  answer: kc's own code on a collected heap (§54's direction).
- **`kc build` with no linker** (parked, the user, 2026-11-18): a
  prebuilt runner (kcrt and a loader) sealing the program's image as `kc
  run` does (C451, C454), the image beside it or appended; no `cc`,
  `collect2`, `ld`, `mkdir`, `rm`; one code path for a built program and
  one run in process. Measured first (after C475): an edit's build at
  1000 procedures 0.35 s, its processes 0.030 s (the link against the
  shared runtime 0.024 s): 9%, so parked for kc's own 0.32 s. To settle
  when taken: how the image travels, what becomes of objects by
  fingerprint (C419) and kc-objects, a built program's start.

- **A race in tier 1's `pselect` over processes** (found at C475's check,
  2026-11-18): `kayak/io/pselect-processes.fx`, under `--regs --compiled
  --gc-stress` on a loaded machine (7.7 s, not 0.7), gave `"0 one\nexited
  4"`, its second line lost: the process's end seen before its last
  output taken, which the program's `pselect` says cannot be (C296). Six
  runs alone, each right. To find: whether tier 1 marks a process ended
  while its output's pipe still holds text (kayak-rt's polling, C295).
  Again at C504's check (2026-11-21), the machine at a load average of
  ~450 from elsewhere: under cells, timed out at 600 s (a hang, not a
  lost line); 30 runs alone after, each right in under a second.

- **`cargo build --workspace --all-targets` fails to link kayak-kcrt's
  lib test** (found at C506, 2026-11-21; so at C505 too): its `#[no_mangle]
  main` and the test harness's are two `main`s. The check builds no such
  target (clippy checks it, unlinked). To mend: `#[cfg(not(test))]` on it,
  or `test = false` for the lib.

## 55. Debts against K8: what works against parallelism (the user, 2026-11-18)

The user, on kc's `kc_args` (C464) being the image's, so that two
threads running one image would clash: "I had hoped that Kayak and kc
were taking a more forward thinking approach to this. Can you make a
list of all the known places where we have opted for things that
actively work against a multi threaded program?"; then "record this
list", and the objective behind it, PLAN.md's K8. The yardstick is
FX-26's: a program means the same whether its processes take turns or
run on threads of their own, each with a heap of its own; they share
frozen data and the globals' bindings, read only.

kayak-rt (KFX's Rust) was built for threads (C101, C102: each process an
OS thread, its state thread-local, `SCHED` locked, `Send` checked by
rustc); nothing of it is on this list. kc's native side is, nearly all
of it chosen in C448–C464:

1. ~~**A run's state in cells of the image, not per thread**
   (`kc/x64.kfx`, `.bss`, reached `%rip`-relative): `kc_args` (C464),
   `kc_prompts` (C458), `kc_run_base` (C460), `kc_gc` (C453),
   `kc_heap_next` (C448). Their place is the state block `%r15` holds
   (Fixpt's convention, kc's since C451: the stack's limit, fuel, `poll`,
   the code's distance), one per run. Cheap; first.~~ Done (C466): the
   state block's words from 664, past `LEAVE`, 152 words in all; the
   entry's stack pointer too (it was the heap array's word 4). The image
   keeps the globals' cells, which processes share, and its heap (in
   process, the code bloblet's field), which is items 2 and 6.
2. **One heap per run, and a collector that sees one stack**
   (`crates/kayak-kcrt-core/src/gc.rs`, C453): Cheney's, the world
   stopped, its roots its own stack from its pointer to the base; its
   spaces (`SPACES`) thread-local, its block (`kc_gc`) the image's. No
   heap can be shared between threads this way. The design (the user,
   2026-11-18): **a heap per thread and a shared heap**, as OCaml 5's
   domains and Erlang's processes have them. An object two threads reach
   lives in the shared heap, so no shared object points into a thread's
   heap: a write barrier promotes what is stored into a shared object
   first. Each thread collects its own heap without stopping the others;
   the shared heap is where collection must be incremental (K8). **The
   shared heap is not assumed not to move** (the user: "we may still put
   a moving collector there at some point; we will just have to ensure
   it remains incremental"): so every pointer into it is one its
   collector can update (a thread heap's, by a remembered set or as
   roots; the symbol table's and its caches'; native code's registers at
   safepoints), and no address is an identity or a hash. That needs no
   read barrier: the user's thesis (Klock, *Scalable Garbage Collection
   via Remembered Set Summarization and Refinement*, Northeastern, 2011;
   Klock and Clinger, "Bounded-latency regional garbage collection", DLS
   2011) moves objects incrementally, one region at a time, with bounded
   pauses, utilization and space, by a write barrier, summaries of the
   locations that point into a region, and snapshot marking; the user:
   "We can accomplish this; the only question is whether we can bear the
   cost." Coopt read it (`../coopt/docs/research/heap-regions.md`, R5)
   and built it (M12, `coopt-heap`: its pauses bounded in work units and
   tested, no full collection under the generational adversary), for one
   mutator; its throughput, the cost, was planned to be measured
   (R5 §4.6) and is not yet. Coopt's crates are kayak's forks, so
   `coopt-heap` is the place to start. What threads add: each mutator's
   barrier and snapshot log, each thread's stack and heap among the
   shared regions' roots, and an evacuation's O(R) step with every
   thread stopped at a safepoint, or handed what moved. Code is apart:
   the code area (C456) does not move it, on purpose. To take with
   to-discuss 16 and §54's steps 3 to 6, which decide kc's heap.
3. ~~**Symbols interned per thread**~~ (`kcrt-core`, `SYMBOLS`): one name on
   two threads two symbols, where `symbol=?` is identity and FX-26's
   symbols are transmissible: wrong under threads, not only unready.
   Unifying symbols as a thread's are promoted into the shared heap (the
   user's question, 2026-11-18) fixes the one promoted, not another
   thread's of the same name: A's `foo` sent to B, promoted, and B's own
   `foo` never promoted, so B holds two, not `eq?`; mending that where
   they meet (B's references rewritten, a forwarding word followed at
   each comparison, or names compared) costs every comparison. So a
   symbol is interned in the shared heap, in one table the process
   shares, a concurrent map mostly read, with a cache per thread of name
   to symbol; a comparison stays one word's. Fixpt's symbol has a global
   binding's slot, process-wide by design; Java's `intern`, Erlang's
   atoms and Racket's symbols are so too. A gensym, uninterned, may be a
   thread's and promoted as any object. The symbols hash by name (FNV,
   Fixpt's), so the shared heap may move them (item 2); the table's
   policy for symbols nothing reaches, weak or kept, to choose. With
   item 2's shared heap, of which it is the first object.
   Done in part (C467): one table the process shares, locked (the user:
   "for right now I'm fine with a shared synchronized intern table"),
   tested from eight threads; the image's cells of quoted symbols, which
   every thread running it reads, now agree with what any thread's
   `string->symbol` gives. Left: the cache per thread, once interning
   shows in a profile; the symbols in the shared heap, with item 2.
   An idea of the user's, not dug into (2026-11-18): whether "A sends
   to B" has an encoding as a distributed-systems problem, N systems
   collectively and incrementally negotiating a shared vocabulary.
4. **What a run writes and its failure, per thread** (`OUTPUT`,
   `FAILURE`, `GIVEN`, kcrt-core's thread-locals): a second thread's
   never reaches the run that began it. Per run, with item 1.
5. **A built program: one state, one heap, one stack**
   (`crates/kayak-kcrt/src/lib.rs`: one state block leaked,
   `HEAP_WORDS`, `STACK_ROOM`). Per process, with kc's processes.
6. **In process, a heap per size, every run of the size given it again**
   (`kc/native.kfx`, `heaps`, C454): two runs of one size at once share
   it. A heap per run, or a pool.
7. ~~**`kayak:last-sealed/1`**, the code sealed last, the process's
   (kayak-jit's `LAST_SEALED`, kc's since C456): kc seals an image, then
   asks where the last sealed code is, to free it later; two threads
   sealing at once may be told each other's, and kc then frees the other
   thread's code. `seal-code` to give the address, or the record per
   thread. Cheap; first.~~ Done (C466): the thread's own, as each
   caller asks on the thread that sealed; tested from eight threads.
8. **Code freed while it may run** (C456): kc frees a transformer's image
   when its cache drops it; kayak-rt checks the code is live at entry
   only, so a thread inside it could have it freed under it. §54's step 5
   (code collected) to take every thread's stack as roots of code.
9. **kc's caches, one writer assumed** (`kc/main.kfx`'s `transformers`,
   C454, C456): harmless while `kc` and `kc serve` take one request at a
   time; a lock, if `kc serve` takes many at once.
10. **No concurrency in kc at all**: kc refuses `spawn`, `send`,
    `receive`, `par`, I-cells, `pselect` and `race`; nothing above has
    been run on threads. Each item, once done, gets a program run with
    processes on threads of their own (K8).
11. **A file of a place, one item to kc's database** (C471,
    incrementality): kc lowers a file of a place as its maker, one form,
    whose body holds every procedure of the file (each instance's items
    scoped as a program's globals, its values in an array of its own).
    An edit of one procedure in it lowers and assembles the whole file
    again: the expander's, its 1583 procedures, where a program's are an
    item each. To come: an instance's procedures items of their own,
    each taking its instance's environment (the array) as a closure's,
    the maker making the closures alone.
12. ~~**kc's database read and written whole at each run** (found at
    C471, scalability): a run of kc loads the database's payload and, if
    it changed, writes it again, whatever it reads. kc-steps' shared
    database grew to 43 MB once a program loaded the expander (its
    code's text), and each run of a small program after it took 0.94 s,
    not 0.012 s (0.66 s of it the system's: the file read): the full
    check went past 25 minutes and was stopped. kc-steps gives such a
    program a database of its own for now. To come: a value's payload
    read when a query reads it (C418 reads the graph so; the payload is
    the rest), and written by what changed, not whole.~~ Done (C472,
    `kayak-db 3`): a head; the graph in shards, a program's file's keys
    one, each read when asked for and written when changed; each value a
    file of its own. A run beside the expander: 0.012 s. Left: the head
    lists every shard (small: a line each), read and written whole.
    Then (C473) a value of 4096 characters or fewer kept in its shard's
    file, since FX-26's `write-file` flushes each file to the disk (a
    cold build of 1000 procedures, a file a value, 6.2 s, not 0.8 s).
    Left: each larger value written is flushed too (an edit of the
    1000-procedure bench writes five files, C471 two: about 25 ms more).
    Asked of Fixpt (the user, 2026-11-18): an atomic `write-file` that
    does not flush, for files whose loss is harmless (a value file is
    believed only if its fingerprint is the graph's); then kayak's value
    files and shards use it, the head alone flushed. Done (C475): Fixpt's
    `write-file-unflushed` (6f7bd73, the pin since C474); an edit's save
    one flush, the head's.

13. **What crosses between kayak-rt's threads, followed whole by every
    pass** (C512): a thread collects as the others run, pinning whole
    what they may reach of its own, found by following, at each pass,
    everything each other thread's list reaches and every mailbox's and
    I-cell's contents; and a message shared, not copied. A large
    structure the threads share is read again at every pass of every
    thread, its pages pinned for as long as one reaches it, and every
    mailbox and I-cell ever made is scanned by every pass. The scalable
    way is item 2's: a heap per thread and a shared heap, what crosses
    promoted into it (copied, as Fixpt copies a message), so that a
    thread's pass reads only its own heap and the shared heap's
    remembered set. Not taken now: kayak-rt's Rust closures (a process's
    body, a `par` branch) hold what crosses in words a collector cannot
    rewrite, so promotion needs the emitter to make them heap objects
    first.

Order: 1 and 7, each a step of its own size, before kc grows more; 3 a
table the process shares now (which is right under any design: kcrt's
symbols are outside kc's heap today), moved into the shared heap with
2; 2 with §54's heap steps and to-discuss 16; 5, 6, 8, 9 and 10 with
kc's processes.
