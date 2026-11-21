# Claude's choices

Decisions made without asking, each saying what was chosen, what else
was possible, and why, so any can be revisited. Newest last.

## C1. Reference clones in `~/Dev/Rust/kayak-refs` (2026-10-11)

The user asked for the analysis to be done on separate clones of Fixpt
and coopt.
- **Chosen:** plain clones at `~/Dev/Rust/kayak-refs/fixpt` and
  `~/Dev/Rust/kayak-refs/coopt`, outside the repository. Fixpt's pin is
  `FIXPT_REV` (`DONE.md` §4).
- **Not chosen:** coopt's own clone, `~/Dev/Rust/fixpt-ref`, which coopt
  moves on its own schedule; a submodule, which puts Fixpt's history in
  kayak's tree, as coopt also declined (its C1).

## C2. Fixpt as stage 0 (2026-10-11)

- **Chosen:** Fixpt runs `kfx.kfx` to make the first `boot/`. The
  transpiler is written once, in KFX, and the first run of the bootstrap
  is also K1's check.
- **Not chosen:** a stage 0 written by hand in Rust (draft 1's `kfx0`):
  the transpiler written twice.
- **Why:** the user's priority was closing the loop fast, and a probe
  showed Fixpt fast enough (a 60 KB source, 60K-deep recursion, instant).
  Once the loop closed, building no longer needs Fixpt.

## C3. Every value `Copy`; memory never freed (2026-10-11)

- **Chosen:** scalars are themselves; everything else is a `&'static`
  reference to memory leaked with `Box::leak`. Mutation, when it comes,
  is `Cell`.
- **Not chosen:** `Rc`, which frees memory but makes values not `Copy`,
  filling the generated code with `.clone()`, and leaks cycles anyway.
- **Why:** the mapping stays trivial, and a compiler run is short:
  compiling the compiler peaks at 10 MB. Places as lifetimes
  (`TODO.md` §14) generalise it.

## C4. Strings are ASCII (2026-10-11)

- **Chosen:** `string` is `&'static str`, and `string-ref` a byte index,
  refused past ASCII.
- **Not chosen:** a string of `char`s, which indexes characters but needs
  converting to print; a scan to the `i`th character, which is O(n).
- **Why:** O(1) indexing with no conversion; the compiler's source is
  ASCII. `TODO.md` §17.

## C5. The mangling (2026-10-11)

- **Chosen:** a primitive and the Rust function implementing it share a
  name: `-` is `_` (alone or last, `_minus`), `?` `_p`, `!` `_x`, `->`
  `_to_`, `+ * < > = /` `_plus _star _lt _gt _eq _slash`; a Rust keyword
  takes `r#`. Datatypes and type names are CamelCase.
- **Why:** reversible, so the Rust can be read back as KFX, and no table
  of primitives is needed in the compiler: the generated module imports
  `kayak_rt::*`, and the names meet.

## C6. Docstrings (2026-10-11)

The user approved strings as comments.
- **Chosen:** a string first in a procedure's body is its documentation,
  as in Lisp; at the top level a string is a plain comment.
- **Not chosen:** a top-level string before a definition as its doc
  comment, which took section headers for documentation (`DONE.md` §3).

## C7. The prelude's names (2026-10-11)

- **Chosen:** `string-length-i64`, `string-ref-i64`, `char->i64`,
  `i64->string`, after FX-26's own `int->i64`.
- **Not chosen:** redefining `string-length` and the rest at `i64`, which
  FX-26 would allow but which reads as the standard `int` versions.

## C8. Constants as functions (2026-10-11)

- **Chosen:** `(define x T e)`, `e` not a lambda, is `fn x() -> T { e }`,
  and each use `x()`.
- **Not chosen:** a `static`, which Rust allows only for constant
  initialisers; a lazily initialised cell, more runtime than the mapping
  wants.
- **Why:** trivial, and equal to FX-26's meaning while `e` is pure; the
  compiler's one constant is a string.

## C9. Mutable data is cells, not `&mut` (2026-10-11)

- **Chosen:** `(ref T R)` is `&'static Cell<T>`, and `(flatarrayof i64 R)`
  `&'static [Cell<i64>]`.
- **Not chosen:** `&mut`, which draft plans (TODO §18) and my own
  suggestion to the user had pointed at for flat data.
- **Why:** every KFX value is `Copy` (C3), so a reference can always be
  duplicated, and two copies alias, which FX-26 allows and `&mut` forbids.
  Fixpt's `distinct` says two *regions* do not overlap, not that a region
  holds one live reference; `&mut` needs an affine discipline that KFX
  does not have. A `Cell` means exactly what FX-26's mutable data means.

## C10. Regions erased to `'static`, for now (2026-10-11)

- **Chosen:** a region, constant or variable, is erased in Rust; the
  checker still checks effects on regions.
- **Not chosen:** a region variable as a lifetime parameter (TODO §14's
  rule), which a closure capturing such a reference would break, since
  `lambdaN` wants `'static`.
- **Why:** every allocation is the heap's, which is never freed, so every
  reference is `'static` already; lifetimes earn their place with places
  (`letrena`), and come with them.

## C11. A constant's value may only read globals (2026-10-11)

- **Chosen:** the checker refuses a constant whose value has an effect
  other than reading globals (or `spin`), e.g. one that makes a flat
  array.
- **Not chosen:** a constant evaluated once in Rust, with a `static`
  `OnceLock`, which a `Cell` (not `Sync`) cannot be in; or a
  `thread_local!`, more runtime than the mapping wants.
- **Why:** a constant is `fn x() -> T`, run at each use (C8); with
  mutable data, that would make a new array at each use where FX-26
  makes one. Stricter than FX-26 (`tests/kfx-stricter/constant-mutable`).

## C12. Flat arrays of i64 alone, through the prelude (2026-10-11)

- **Chosen:** `make-flatarray-i64`, `flatarray-ref-i64`,
  `flatarray-set-i64!`, `flatarray-length-i64`, defined in the prelude
  over FX-26's, with sizes and indices at `i64`.
- **Not chosen:** FX-26's own forms, which take `int` indices and a
  `(flatlayout T)`, for any of six scalar types.
- **Why:** KFX has `i64` alone among numbers so far; a layout argument
  would be a dictionary with no Rust counterpart yet (a trait would be).
  Other element types when KFX has the types.

## C13. A region the call does not say is refused (2026-10-11)

- **Chosen:** a call whose latent effect names a region binder that
  neither the arguments nor the expected type solve is refused: "write
  it, (the T e)". So are `distinct` binders, for now.
- **Not chosen:** FX-26's fresh region and its masking, so that a
  procedure making and using its own array is `pure`.
- **Why:** masking is a rule the checker would have to get exactly right
  to keep K1; refusing is safe. With places (TODO §14), a `letregion`
  says the region, and masking comes with it.

## C14. Strings of characters, which know if they are ASCII (2026-10-11)

Supersedes C4.
- **Chosen:** `Str` is a `&'static str` and a flag, whether it is ASCII,
  found when the string is made (`str_lit`, a `const fn`, for a literal).
  `string-length-i64` and `string-ref-i64` count characters, as FX-26
  does: at once for an ASCII string, by walking one that is not.
- **Not chosen:** ASCII only (C4), which crashed on 70 of Fixpt's
  programs, and gave a different answer from FX-26 for a string that is
  not ASCII; strings of `char`s (`&'static [char]`), O(1) always but a
  copy at every literal, and a conversion at every boundary with Rust.
- **Why:** FX-26's strings are of characters. The flag keeps the common
  case, ASCII, as fast as before; the reader takes its source with
  `string->list`, in one pass, so a source that is not ASCII is read in
  linear time too.

## C15. `int` as a bignum of kayak-rt's own (2026-10-11)

- **Chosen:** `int` is `Int`, a small-or-big integer in `kayak-rt`: an
  `i64` while it fits, a sign and base-2^32 limbs past that, every result
  that fits made small again. Its operations are FX-26's exactly
  (truncating `quotient`, `remainder` of the dividend's sign, `modulo` of
  the divisor's, `int->i64` keeping the low 64 bits). Division is a bit
  at a time for a divisor of more than a limb: slow, and simple to trust.
- **Not chosen:** a crate (`num-bigint`): the siblings write their own,
  and `kayak-rt` has no dependency; `int` as an `i64` that panics on
  overflow, which the user ruled out (it is not FX-26's meaning).
- **Why:** the user's ruling was against a construct whose meaning differs
  between FX-26 and Rust; a bignum's does not. 53 of Fixpt's programs
  first stopped at `int`.

**With it, facts.** Which literals are `int`s only the checker knows, and
the emitter knows no types. So the reader gives each list and number a
place (its file and offset), and the checker records facts about places,
in its inference state, then in `cv`; the emitter looks them up, as
Fixpt's compilers read its checker's facts. The same channel will carry
what escapes need (TODO §16).

## C16. A region no one names is `acyclic` (2026-10-11)

Revises C13.
- **Chosen:** a call's region binder that neither its arguments nor the
  type expected solve is `acyclic`, and effects on `acyclic` are pure.
- **Not chosen:** refusing it (C13), which would refuse `(cons 1 nil)`
  once lists may be at any region; or FX-26's fresh region and masking.
- **Why:** FX-26 makes such a region fresh, and masks its effects where it
  is not seen; a fresh region that nothing else names is, from outside,
  pure, as `acyclic` is. Mutable data is never at `acyclic` (its type is
  refused there), and KFX writes no list, so nothing written is made
  frozen by it. `tests/kfx-stricter/region-unsaid` is gone: KFX accepts it.

## C17. Type families: types and regions, no higher kinds (2026-10-11)

- **Chosen:** `define-datatype` and `define-type` with parameters of kind
  `type` and `region`: Rust generic enums and aliases, regions erased. A
  family is applied `(t arg ...)`; a type abbreviation is expanded by
  substitution, a datatype is kept by name with its arguments.
- **Not chosen:** higher kinds (`(f (=> (type) type))`), which Rust has
  no plain form for; most of Fixpt's family programs use them.
- **Why:** generic containers are what a compiler written in KFX needs;
  they map to Rust's generic types exactly.

## C18. Escapes: `Result` by effect (2026-10-11)

Answers `PLAN.md` Q3 (`docs/research/escapes.md` §5).
- **Chosen:** tags, prompts and aborts, as early returns. A procedure
  whose latent effect may `goto` returns `Result<T, Unwind>`, a call the
  checker found may abort takes a `?`, an abort is `return Err(..)`, and
  a prompt is a closure returning a `Result`, matched. Procedures that may
  abort are values too (`&dyn Fn(..) -> Result<..>`). `Unwind`'s payload
  is erased (`&'static dyn Any`), so every escape shares one `Result`;
  the tag's type parameters give it back its type.
- **Not chosen:** unwinding panics (quicker, but effects would decide
  nothing, and `catch_unwind` needs what closures over cells are not);
  kept continuations, `comefrom` (`call-with-composable-continuation`,
  `cwcc`), which Rust has no frames to resume; refused, with a message.
  Marks, for now.
- **Why:** effects deciding code is the plan's mapping, and this is what
  Rust programs write by hand: exact, nothing paid where nothing escapes.
  Whether a type's effect may abort is read from its text, through
  `define-effect`s; a call's or a lambda's, and whether a prompt contains
  every abort its body may make, are facts the checker leaves by place.

## C19. Modules: top-level ones, as Rust `mod`s (2026-10-11)

- **Chosen:** `(define m (module item ...))` at the top level is
  `pub mod m { use super::*; ... }`; `(with m body ...)` writes the names
  `m` gives as paths, `m::x`; `(select m t)` is `m::T`. Items are a
  program's definitions (`define`, typed or not, `define*`, `define-rec`,
  `define-type`, `define-generative`, `define-datatype`), each seeing
  those before it. A
  generative type in a module is abstract, as FX-26's is: a newtype with
  a private field, its conversions private to the `mod`, so that Rust's
  privacy holds the abstraction too. The checker names a module's types
  `m..t`, as FX-26 writes them, so a module's and the program's never meet.
- **Not chosen, for now:** modules as values (`let`-bound, passed, sealed
  with `the`), `load-module`, `extend`, functors (`include` and `hide`
  came later, the same day)
  (Sheldon's dependent procedures); effects in modules; a module's types
  with parameters; a module's datatype with a tag another datatype has
  (FX-26's sums are structural, its tags found by type; the emitter finds
  a datatype by its tag); items naming later items (FX-26's are a
  `letrec*`). All refused, `tests/kfx-stricter/module-*`.
- **Why paths, not `use m::*`:** inside `with`, FX-26 gives the module's
  names before any local, Rust gives a local before any import. Paths say
  which is meant, and are what a reader of the Rust wants to see.
- **Why a module's items may not reuse a global's name:** one name, one
  meaning in the emitter, which knows names, not scopes; KFX refuses more.

## C20. Termination: FX-26's size-change check, a subset (2026-10-11)

- **Chosen:** a recursive group (`define`, `define-rec`, `define*`,
  `letrec`, and so a module's) whose every run ends needs no `spin`, by
  FX-26's own check (Fixpt's `terminate.rs`): size-change graphs over
  parts (`car`/`cdr` at `acyclic`, `tagcase` fields) and over `int` and
  `nat` counting down to a fixed bound or up to one, closed under
  composition. KFX's is a subset of it, measure for measure, so that KFX
  accepts no group FX-26 does not: no `i64` measure (its subtraction
  wraps; FX-26 has none), facts only from what FX-26's parse of `cond`,
  `and`, `or` and `let*` gives (fewer where in doubt), and a smaller cap
  on the graphs (500, FX-26's 4000).
- **Not chosen:** KFX's own, broader rule (`i64`, say), which would make
  KFX accept what FX-26 refuses: K1.
- **Why:** pure recursion is what FX-26 programs write, and the check is
  the checker's alone: the Rust is the same, `spin` erased.
- **Left stricter:** a call of a procedure a local names is `spin` in
  KFX; FX-26 takes its latent effect, unless its type is recursive through
  a parameter (`kfx-stricter/local-call`).

## C21. A constant that allocates is made once, at its first use (2026-10-11)

- **Chosen:** a constant whose value allocates (a cell, a tag, a closure
  over state of its own) is a `fn` that makes it the first time and keeps
  it, in a thread-local `OnceCell` (`kayak_rt::once`); its value may read
  globals and allocate, nothing more. Other constants stay `fn`s run at
  each use.
- **Not chosen:** refusing them (as before: 14 of Fixpt's programs);
  making every constant at start-up, in order, which Rust's statics do not
  do without `unsafe` or a constructor the program calls.
- **Why:** FX-26 makes a constant once, where it is defined. Made at its
  first use instead, an allocation is not seen to be later: only reading
  mutable data, or writing it, would show when, and the checker refuses
  those (`kfx-stricter/constant-reads`).
- **With it,** FX-26's rule against knots through the store had to be
  KFX's too: a procedure kept at a region whose latent effect reads that
  region must say `spin` (`no_knot`), checked on written types and on
  each call's instance, a generative type's arguments taken as kept
  everywhere (more cautious than FX-26).

## C22. Products and sums: a generic Rust type per list of labels (2026-10-11)

- **Chosen:** FX-26's structural `(productof (l T) ...)` is a tuple struct
  `Product_l..<A0, ...>`, `(sumof (tag T) ...)` an enum `Sum_tag..<A0,
  ...>` whose values are `&'static` references, as a datatype's are; one
  of each for each list of labels the program writes, declared at the top
  of its file (in `mod.rs` for a program of several). Two products of the
  same labels and types are so one Rust type, as one FX-26 type.
  `(extract e l)` is `e.i`, the index the checker found; a `(sum tag e)`
  is checked against the sum expected, whose type, which Rust's enum
  wants as arguments, the checker gives the emitter.
- **Not chosen, for now:** a sum whose type nothing says (FX-26 finds a
  sum of its one tag), and width subtyping, a sum of fewer tags standing
  for one of more: each would be a conversion between two enums
  (`kfx-stricter/sum-*`); recursive types through an abbreviation (Rust's
  aliases cannot be: a generative type or a datatype can); numbered tags
  and flat sums.
- **Why:** labels in order are the type's identity in FX-26; a name made
  of them is that identity in Rust, with no declaration the programmer
  writes.

## C23. Places: not now (2026-10-11)

Answers `PLAN.md` Q4 (`docs/research/places.md` §4) for the time being.
- **Chosen:** no places yet: `letrena`, `letreap` and the `r`-allocators
  stay refused, every reference `'static`.
- **Not chosen:** one lifetime for the run (§3), or `letrena` alone, each
  touching every type the emitter writes.
- **Why:** nothing in kayak needs memory reclaimed (the compiler runs for
  milliseconds on itself), and the change reaches every type: worth doing
  when a long-running or large KFX program needs it, and then as §3 says,
  one lifetime first. The user may decide otherwise; the note stands.

## C24. Module parameters: traits and type parameters (2026-10-11)

- **Chosen:** a `define-type` of a `(moduleof (abs t type) (desc u T)
  (val x T) ...)` is a Rust trait: its abstract types associated types, its
  values functions of no receiver (`fn x(a0: P) -> R`, a value not a
  procedure `fn x() -> T`), a description replaced by what it describes. A
  top-level procedure's parameter of such a type is a type parameter
  (`fn three<M: Steps>()`), `(with m ...)` writing `M::inc(M::zero())`; a
  module defined at the top level given for it is a marker, `m::Module`,
  given as the type argument (`three::<counter::Module>()`), which
  implements the trait by calling the module's items. The checker checks
  the module against the type: each abstract type given, each description
  the same, each value of a type that fits, the abstract types the
  module's.
- **Not chosen, for now:** modules as values otherwise: made where they
  are given, made in procedures, `let`-bound, sealed with `the`, passed on
  from a parameter; dependent parameters (`(o ordered)`, `(select o t)`
  in the types after); a module type's value that may abort.
- **Why:** a module FX-26 passes is, where its giver is known, a static
  choice of code: Rust's monomorphised generics are exactly that, with no
  value at run time, and the trait says what the checker checked.

## C25. `spin` for a call of a procedure given, as FX-26 has it, cautiously (2026-10-11)

- **Chosen:** a call of a procedure a local names says `spin` only where
  its type's parameters or result may hold a procedure (through lists,
  cells, shapes, datatypes, a generative type's representation, or a type
  not known yet); naming a global procedure as a value says `spin` only
  within its own group. FX-26 says it for a type recursive through a
  parameter; KFX's rule is a superset of that, so KFX accepts no loop
  FX-26 would refuse.
- **Not chosen:** `spin` for every such call (KFX's rule before), which
  left no higher-order procedure `pure`.
- **Known:** FX-26, and so KFX, accepts self-application through a `poly`
  type variable (`(subr pure (a) b)`, `a` a datatype holding such a
  procedure), which loops with no `spin` (a question for Fixpt, `TODO.md`
  §17).

## C26. A list made where nothing says its region is at a fresh region (2026-10-11)

- **Chosen:** a `list` with no type to go by, or given for a parameter
  whose type is not known whole yet, is at `%fresh`, a region of its own,
  as FX-26's is at a fresh region: it fits no `acyclic` expected later.
  Effects on it are pure, KFX's lists never being written.
- **Not chosen:** `acyclic` for it (KFX's rule before: C16's), which made
  KFX accept programs FX-26 refuses; one fresh region for each list
  (FX-26's), which needs FX-26's masking of fresh regions where they are
  not seen, throughout.
- **Why:** the commonest K1 gap the probes found; this closes most of it
  with no change to the Rust.

## C27. KFX's names have no `_` (2026-10-11)

- **Chosen:** a name with a `_` is refused, wherever it is bound. KFX
  writes `-` (mangled `_`), and every Rust name kfx and kayak-rt write that
  a KFX name could meet has a `_`: kayak-rt's functions (`i64_plus`,
  `string_append`), the emitter's temporaries (`tagcase_`,
  `prompt_tag__`, `out_`, `a_0`), and the mangled forms of other KFX
  names (`ok?` is `ok_p`).
- **Not chosen:** a list of the names to refuse (KFX's had one, of a
  few), which a sixth probe showed incomplete: `(define i64_plus ...)`
  hid kayak-rt's `i64+`, and `(i64+ 1 2)` was not 3; a mangling one to one
  (it would rename kayak-rt's every function).
- **And:** two KFX names in scope that mangle alike (`ok?` and `ok-p`,
  both `ok_p`) are refused where the second is bound; the names in scope
  are searched only for a name that could clash (one with an operator's
  character, or whose mangling has one's). The compiler on itself takes
  0.4 s for it, not 0.08 s.
- **Then** (the seventh probe): a KFX value's name is a lower-case letter
  or an operator's character, then those, digits and `-`; no `--`; none
  whose mangling kayak-rt or the emitter writes. The emitter's own calls
  behind a program's back are qualified, `kayak_rt::cons`.
- **Then** (the user's, 2026-10-11; revised by C41): no KFX name is a Rust keyword, strict,
  reserved or weak (`type`, `match`, `fn`, `abstract`, `union`, `raw`,
  `safe`, `macro-rules`), or `self`, `super`, `crate`, or a type variable
  whose camel case is `Self`: everywhere a name is bound, values, types,
  type variables, abstract types and labels. FX-26 has them; Rust writes
  them only as `r#type` or not at all (`self`), and a reader of the Rust
  is better without either. `mangle` keeps writing `r#` for them, which
  the compiler no longer needs (`kfx-stricter/rust-keyword`, `self-name`).
- **Then:** a type variable, a `poly`'s or a family's, is held to the same
  rule (`kfx-name?`), and none two in one binder camel-case alike (`a`,
  `a-`), and none camel-cases as a type the program defines, anywhere in
  it: Rust's type parameter would hide that type where FX-26's hides it
  only in name. Stricter than scope by scope, and simpler. With the
  checks on types, tags and labels before it, every type-level name a
  probe found colliding in Rust is refused in the front end.

## C28. `void` is an empty enum, `kayak_rt::Void` (the user's, 2026-10-11)

- **Chosen:** FX-26's `void` is `pub enum Void {}`: Rust's stable empty
  type, usable where a type is (a parameter, a field, a list's element),
  `Copy`, and `Show` (an empty `match`). A call that gives one is written
  `match f(...) {}`, of any type, since FX-26's `void` is below every type
  and Rust's `Void` below none; `error` keeps returning `!`, which coerces
  to anything, and where it stands for an inference variable (a call's
  argument, a list's element) it is `the::<Void>(error(...))`, so that Rust
  infers `Void`, not `!`.
- **Not chosen:** Rust's `!`, unstable but as a function's result.

## C29. Types FX-26 leaves open, told to Rust (2026-10-11)

- **Chosen:** a poly call's type binders must be solved at the call, by
  its arguments and the type its place expects, as FX-26's are (`fixpt`'s
  `infer.rs`: "`a` cannot be inferred", "argument 1 ... not yet known
  here"); `(mk)`, `(h nil)`, `(cdr (error "x"))` and `(list)` with nothing
  to type it are refused, as FX-26 refuses them (K1: KFX had accepted
  them). `null?`'s binder is FX-26's argument's whole type, so it is let
  open for a name or an `error` (stricter than FX-26 otherwise).
  `car` and `cdr`, FX-26's over a pair `(pairof t1 t2 r)`, of an `error` or
  a name of type `void`, are refused: nothing says the pair's other half.
- **And:** what is left open is told to rustc, by the facts that every
  datum's place makes possible: each `nil` and empty `(list)` is written
  `nil::<T>()`, a program's poly procedure whose binder is its result's
  alone `mk::<T>(...)`, and a `null?` left open `null_p::<Void>(...)`. The
  checker notes inference variables, settled when the definition is; one
  still unsolved is `void`, the empty enum (C28), a type of no value that
  the program, which never meets one, cannot tell from any other.
- **Not chosen:** a turbofish on every poly call (the Rust's `map::<i64,
  Str>(...)` everywhere, for nothing rustc lacks); defaulting to `()`,
  which is a value a program could be thought to hold.
- **And** (found on the way): the closures that lift a procedure to return
  `Result` named their parameters `a_0`, which `a-0` mangles to, and so
  called their own argument: C27's temporaries with a `_` were not all
  apart from KFX's names. They are `a__0`, which no KFX name, having no
  `--`, mangles to.

## C30. `letrena` is a Rust arena, in safe code (the user's, 2026-10-11)

- **Chosen** (the user's idea, and their choice of our own arena): FX-26's
  `(letrena r body ...)` is a block that declares the region's type,
  `Region_R<'r>`, one `kayak_rt::Arena` for each type made in the region
  (`Cons<'r, T>` for an `rcons`, `Cell<T>` for an `rnew`), with `Holds<T>`
  for each; a `fn letrena__r<'r>(r: &'r Region_R<'r>, x: X, ...)`, the
  body, given the arena and the locals it names; and makes the arena and
  calls it. The arena is dropped when the block ends, by a return or by
  an abort (`?`), so the region is freed as FX-26's is. Rust's borrow
  checker then refuses what FX-26's refuses: nothing borrowed from the
  arena outlives the block.
- **The arena, in safe Rust**: chunks of `OnceCell<T>`, each twice the
  last, linked through `OnceCell`s; nothing is taken out of one, so
  `alloc(&self, v) -> &T` needs no `unsafe`. One type an arena, as
  rustc's `TypedArena`s are, and a region a struct of them, as rustc's
  `arena_types!` makes. KFX's values are `Copy`: no destructor runs.
- **Its chunks found, not walked** (2026-10-11): chunk `k` holds `16 << k`
  cells, so the `i`th value's chunk and place are computed from `i`; the
  chunks a fixed array of `OnceCell`s, made when first reached. A reference
  to the current chunk kept in the arena is what safe Rust will not have
  (it would point into the arena itself). Fixpt's `bench/lists-region.fx`:
  45 ms the walk's way, 7.8 the found way (Fixpt's best machine, 7.2).
- **A `fn`, not a block alone**: a block cannot name a lifetime, and a
  closure's or a `dyn Fn`'s parameter of an elided lifetime is a fresh
  one at each call; the `fn`'s `'r` is the region's, named in the types
  of the body's closures (`ListAt<'r, T>`, `RefAt<'r, T>`, `&'r dyn Fn`).
- **Lists and references have a lifetime**: `ListAt<'r, T>`, `RefAt<'r,
  T>`; the heap's are `List<T>` and `Ref<T>`, at `'static`, as before.
  `alloc`, the `lambdaN`s and `knot` leak to any lifetime.
- **Not yet** (refused): a place as a value but an allocator's (FX-26
  passes places to procedures); a region within a place; a `letrena` in a
  procedure poly over types; a body that names an outer `letrena`'s
  place; `rmake-array`, `rmake-icell`, `rmake-bloblet`, `rlambda`,
  `letreap`. Bloblets and blobrefs (`&'r` to a `#[repr(C)]` flat of
  `Cell`s) are the next step.
- **Not chosen:** `bumpalo` or `typed-arena` (kayak-rt has no dependency,
  and both are `unsafe` inside); an arena of our own with `unsafe` in one
  module.

## C31. Bloblets and blobrefs, faithful first (the user's, 2026-10-11)

- **Chosen** (the user approved the plan: faithful first, typed as a fast
  path after): a bloblet is `Bloblet<Fs>`, its fields a tuple of cells
  and its suffix words, `Box<[Cell<u64>]>`; `(bloblet (fields T ...) R)`
  is `&'r Bloblet<(Cell<T>, ...,)>`, `'r` its region's (a `letrena`'s, or
  `'static`). A blobref is `BlobRef<B>`, `B` its owner's Rust type (so a
  named owner type, `o`, is written `BlobRef<O>`), and a byte offset;
  `blobref-get` and `-set!` read the scalar the checker found
  (`get_f64`...), within its word when aligned to its size (C's layout
  makes every field of a flat product so), else byte by byte. `extract` of
  a blobref is `.field(n)`, `n` the offset the checker laid out;
  `blobref-index` is `.index(i, size, len)`. All of FX-26's model holds:
  one suffix read as several flats, any offset, `blobref+`; in safe Rust.
- **Measured** (`scratchpad/blob/bench.rs`, a million 24-byte products):
  the words 1.5 times the time of a typed `#[repr(C)]` struct of cells,
  both about a nanosecond an element.
- **Not yet** (refused): `f32` (KFX has none), frozen bloblets, a bloblet
  of more fields where one of fewer is expected (FX-26's lists at least
  its fields; KFX's exactly), flat sums and `blobref-variant!`, families
  over the kind `flat`, a field that may hold a procedure (a knot through
  the store no type here shows). Then the typed fast path, for an owner
  type the program reads as one flat at offset 0.
- **And** (found on the way): FX-26's effects and types are one
  namespace, an effect hiding a type of its name; KFX refuses a type
  named as an effect (K1). `run-fixpt-programs.py` counts a program that
  fails when run in both as agreeing.

## C32. A typed suffix, where the program allows one (2026-10-11)

- **Chosen** (C31's plan, the user's): a bloblet type whose suffix the
  program reads only as one flat, from offset 0, is typed: `TBloblet<Fs,
  S>`, `S` the flat as Rust holds it (`Cell<f64>`; a product struct of
  them, the emitter's own, given `Flat`; `FlatArr<T, n>`), and a blobref
  into it a reference, `&'r S` (the user's first idea, `&'place
  convert(T)`): `extract` is `&(r).1`, `blobref-index` `(r).at(i)`,
  `blobref-get` `.get()`. Typed when, in the whole program, every
  `blobref-at` into the type is at a literal 0 and of one flat, nothing
  reads its bytes otherwise (`blobref+`, `-owner`, `-offset`,
  `bloblet-byte`, `-set-byte!`, `-bytes`), and every one is made of a
  literal size the flat fits in; else C31's words. Types are compared by
  their fields written out (the emitter keeps type abbreviations' bodies),
  whatever the region: a procedure poly over the region writes one type.
- **Measured** (`scratchpad/perf`, a million 24-byte products, `x + y + n`
  read, `n` bumped): C31's words 31 ms and 10 ms; typed, 16.5 and 5.0;
  hand-written Rust 10.4 and 4.9. The rest was kayak-rt's small functions
  not inlined across the crate (LTO closed it): they are `#[inline]`, and
  KFX's code is hand-written Rust's, 10.5 and 5.0. A sum over a million
  `f64`s in a flat array was even with a `Vec<f64>` already.
- **And**: the emitter's fact lookups searched by making a list of every
  match (`concat` of `map`), at each name since C29; a search, now. The
  compiler compiles itself in 0.46 s (0.76 s this morning's way).
- **Not chosen:** typing by the flat of each blobref alone (a suffix read
  two ways would be two Rust types); deciding per bloblet value (a type is
  Rust's per type).

## C33. Keyed flat sums, faithful only (2026-10-11)

- **Chosen:** FX-26's keyed flat sums, `(sumof u8 (0 F) (7 G))` (`u16`,
  `u32`): a tag of the key's width holding the label as it is, then the
  variant at the widest variant's alignment, the whole padded to it.
  `blobref-variant!` writes the tag and gives a blobref to the variant;
  `tagcase` on a blobref reads the tag once, each arm (a literal label, one
  name) a blobref to its variant, `else` the blobref; a tag no arm has
  and no `else` is an error when run, as in Fixpt. In C31's words only:
  an owner type whose flat has a sum is never typed (C32), since a sum's
  variants share their bytes and a stale blobref to one sees what another
  wrote (FX-26: `2.5` read back as `2.5000000000000013` once an `i32` was
  written over its low bytes); a Rust enum or separate fields would not.
- **Not yet:** a sum with no key, whose tag indexes the owner's fields,
  which hold its labels as symbols: KFX has no symbols.

## C34. Places given to procedures, bounds that say what they hold (2026-10-11)

- **Chosen:** a procedure poly over a place, `(poly ((p place)) ...)`,
  takes it as a value of type `(place p)` and makes in it; in Rust a
  lifetime and a type, `fn build<'p, Place_P: Holds<Cons<'p, i64>>>(h:
  &'p Place_P, ...)`, `Place_P` bound to hold each type the procedure
  makes in the place, by itself or by the procedures it gives the place
  to. The checker notes each making with its kind and place
  (`rheld-cons@'p T`), sums a procedure's up when its definition is done
  (`place-needs build 'p cons T`), and at each call notes the callee's,
  at the types and place it is called at (`rneed-cons@'r T`). A
  `letrena`'s region type holds an arena for each type made or needed in
  its body. Places are values now: given to procedures, local lambdas,
  kept in data. Implicit instantiation suffices, as in Fixpt; `(proj ...)`
  KFX has not.
- **Why the bounds:** in safe Rust an arena holds one type, so a place is
  a struct of them (C30), and a procedure over any place can only ask,
  by its bounds, for those it needs; Rust checks each call's place has
  them, which the checker's needs make so.
- **Not yet:** regions within a place, `(r region p)`; families over a
  place; a nested `letrena` naming an outer one's place; `letreap`,
  `rlambda`, `rmake-array`, `rmake-icell`; places in mutually recursive
  procedures (a group's needs are not summed together).

## C35. A procedure lifted wherever it stands (2026-10-11)

- **Chosen:** where an expression found by itself has the type of a
  procedure that cannot abort and one that may is expected, the checker
  notes `lift N` at that expression (every datum has a place now), and the
  emitter writes it as a closure that returns its value `Ok`, wherever it
  stands: a list's element, a product's field, a branch, an argument, a
  `the`. It replaces `lift-arg`, which saw only a name given as an
  argument (the fifth and seventh probes' E0271s).
- **And** (K1): an `if`, a `let` or another control form given for a
  parameter whose type is not known whole yet, its effects' regions
  included, is found by itself, as FX-26 finds it (as a `list` was), so
  that its branches must be of one type; KFX had checked each against the
  parameter's type and accepted branches FX-26 refuses.
- **And:** a call whose value is a procedure, where one is expected, has
  its result fitted after its arguments, the expected type bounding it
  from above as FX-26's does: `(car fs)` of pure procedures where one that
  may abort is wanted is a pure one, lifted, not a list of them taken as
  one of procedures that may abort, which Rust cannot convert.
- **And** (names): a module parameter's Rust type parameter is `Mod_M`,
  apart from a type `mod-m`'s `ModM`; a module's types may not be named as
  kayak-rt's either (`str` in a module hid `Str`).

## C36. Arguments found by themselves, lists at fresh regions (2026-10-11)

- **Chosen** (K1, Fixpt's `infer.rs`): an argument for a parameter whose
  type is not known whole yet, a latent effect's region included, is found
  by itself, not checked against it (a lambda, told its parameters, is
  checked): a literal is an `int`, a control form's branches of one type.
  And a list a call makes, found by itself, at a region nothing said, is
  at a fresh region (`%fresh`), as FX-26's `default_regions` makes it,
  not one a later use may choose: `(h (cons "x" nil))` for a `(listof a
  acyclic)` is refused, as Fixpt refuses it. Seven of the fourteen K1
  differences the probes left agree now.
- **And** (`nat`): `+` keeps a `nat` only of literals, names of a `nat`
  and such sums, as FX-26's does (each literal its own `(nat v)`); a call's
  or a `the`'s `nat` makes it an `int`.
- **And:** a product found by itself with a bare `nil` field is refused
  (FX-26 keeps the field `nil`'s poly type; KFX's fields have one type);
  F17's self-application seen through products, lists, references and
  datatypes' arguments, as Fixpt sees it.
- **Not yet** (KFX accepts, Fixpt refuses): a pair whose tail is a pair,
  found by itself and given where a list is (FX-26's `cons` makes
  `(pairof t1 t2 r)`, its tail's type its own; KFX's makes a list's; a
  try at marking such pairs needed calls checked against a type to leave
  their regions open, which the compiler's own bootstrap did not take); a
  literal in such a pair's tail (`(rcons r k (rcons r 1 nil))`); a product of a poly `nil` and one of
  a list in branches; a procedure's self-application through a product.

## C37. Names Rust can write, all of them (2026-10-11)

- **Chosen:** `camel` mangles an operator's character as `mangle` does
  (`ok?` is `Ok_p`), `-` still beginning a word: a type, a tag, a sum's
  label named with one was no Rust name (`pub type Ok?`). And refused,
  where Rust's names would meet: two names bound together that mangle
  alike (`ok?` and `ok-p` in one `let`); two shapes whose labels mangle
  alike (`a?`, `a-p`); two module parameters that camel-case alike (`m`,
  `m-`); a module named as a crate the program sees (`kayak-rt`, `std`,
  `core`, `alloc`); a module whose own datatype an included one has.
- **And:** a procedure a top-level expression gives is shown as Fixpt
  shows one, whatever made it (a global's own `fn` has no `Show`); a flat
  array as Fixpt writes it, `#<flat-array 2 fields 16 bytes>`.
- **Not yet:** a list of a global's procedures shown at the top level (its
  `fn` items no `dyn Fn` Rust can infer there).

## C38. `rlambda`, and lists at the heap of a region's data (2026-10-11)

- **Chosen:** `(rlambda r params body ...)` is typed as FX-26 types it, its
  latent effect reading `r`, its making allocating there, and written as
  a `lambda`'s closure, which the heap keeps as long as anything wants it:
  where its memory is is all that differs. A `letrec` may bind one.
- **And:** a list or a reference at the heap (a `letregion`'s, erased)
  whose element's Rust type mentions a `letrena`'s lifetime is at that
  lifetime, `ListAt<'r, &'r dyn Fn ...>`: a `'static` pair may hold
  nothing that lives less.
- **Not yet:** a type abbreviation over regions, `(define-type (closures
  (r region) (l region)) ...)`, written once in Rust with its regions
  erased, `'static`; in a `letrena`, where a use of it means `'r`, a Rust
  error. Its regions want to be the alias's lifetime parameters.

## C39. Type abbreviations over regions, lifetimes (2026-10-11)

- **Chosen:** a `define-type` over regions is a Rust alias over lifetimes,
  `pub type Closures<'r, 'l> = ListAt<'l, &'r dyn Fn(i64) -> i64>`, its
  body written with its regions as a `letrena`'s are; a use gives each
  region's lifetime, a `letrena`'s its own and the heap's `'static` (in a
  `letrena`, its innermost one's: a heap list is one at any lifetime). A
  procedure type's lifetime is that of a region its latent effect names,
  `(subr (read r) ...)` `&'r dyn Fn`, else the innermost `letrena`'s.
- **Not yet:** datatypes over regions, whose regions stay erased.

## C40. Regions within places, families over places, `proj` (2026-10-11)

- **Chosen:** a region binder within a place, `(r region p)`, and a
  `letregion` inside a `letrena`, may hold data made in the place
  (`rcons`, `rnew` at the region the type expected says, if bound within
  the place, as FX-26 has it); in Rust such a region is at the place's
  lifetime, where its data is. A call or a `proj` giving such a binder a
  region bound outside the place is refused (`region-within` facts kept
  for each procedure). A family over a place, `(define-type (builder (p
  place) (r region)) ...)`, is an alias over lifetimes and, for a place
  its body names, its type (`Place_P`); an abbreviation's application is
  expanded where the emitter must see a procedure's type whole (a
  `letrec`'s, a definition's). `(proj f a ...)` instantiates `f` at the
  arguments in its binders' order, its needs and bounds noted as a call's,
  and is `f` in Rust; a call of a `proj` is no call of an unknown value
  (no `spin`). Fixpt's `caller-place.fx` (its `use` renamed: a Rust
  keyword), `two-regions-one-place.fx`, `place-is-a-region.fx` agree.
- **Not yet:** a family's region binder within a place, its bound kept
  (an alias's place parameter may be any region there, laxer than FX-26).


## C41. Rust's keywords as names, raw identifiers (2026-10-11)

- **Chosen** (the user's, revising C27's ruling): a KFX name may be a Rust
  keyword, strict, reserved or weak; the Rust is its raw identifier,
  `r#loop`, a lifetime too (`'r#fn`). A name the emitter makes from one
  (`loop__knot`, `letrena__box`) has it bare (`mangle-bare`). Refused
  still: `self`, `super`, `crate`, and a type variable camel-cased `Self`,
  which no raw identifier may be. A region or place named `static` is
  `'static_`, `'static` being Rust's own and no raw lifetime (KFX's names
  have no `_`, so none clashes). FX-26's own keywords (`type`, `let`) stay
  refused as FX-26 has them. Scheme's `|...|` is the reader's escape for a
  symbol's characters; `|if|` is `if`, Scheme reserving no word, so the
  nearer kin of `r#` is none: FX-26 needs no escape, Rust does.
- **And:** one list of the type names the emitter writes
  (`rust-type-names`), for types and type variables alike, with `Cell`,
  `Default` and `Procedure`, which neither list had; the refusal says
  where the type is defined.
- Fixpt's programs kfx accepts: 79 to 90, all agreeing.

## C42. Procedures poly over regions, lifetimes (2026-10-11)

- **Chosen:** a procedure's region binder not within a place, `(r
  region)`, is a lifetime of its Rust `fn`, `'r`, marked as a `letrena`'s
  is (`%letrena r`), so its lists, references and procedures are at `'r`
  (`ListAt<'r, T>`, `RefAt<'r, T>`, `&'r dyn Fn`) and a `letrena`'s data
  may be given it; rustc infers the lifetime at each call. Such a
  procedure makes nothing in `r` (KFX's allocators take a place, C34).
- **Refused:** a call of a program's procedure at a type over a region with
  a lifetime (a `letrena`'s, a place's, a region binder's) for one of its
  type variables, whose Rust parameter is `A: Copy + 'static`
  (`kfx-stricter/type-variable-at-letrena`): before C42 the same call
  from a `letrena`'s body made Rust rustc refused.
- **And:** a `define*`'s places are a `define`'s: what it makes in each
  kept (`place-needs`), and its regions within places, which `define*`
  had dropped, its Rust then lacking the `Holds` bound.
- The `lists` port's helpers are the program's own now, 5.4 ms as before.

## C43. The heap by bumping, in safe typed arenas (2026-10-11)

- **Chosen** (the user's ruling: safe typed arenas only; fragmentation is
  acceptable for the bootstrap): the heap's pairs, references, sums and
  datatypes are made in an `Arena<T>` of their type, one per type for the
  run, found in a thread's table by `TypeId` (the last found kept at hand),
  leaked so `&'static`. `kayak_rt::heap_alloc`, `heap_cons`, `heap_new`.
- **Where:** a `TypeId` is only a `'static` type's, and the heap may hold
  a region's data (C38), so only where every lifetime the emitter writes is
  `'static`: no `letrena`, place or region binder around. Type parameters
  are `'static` already. Elsewhere, and for closures, a `Box` each as
  before.
- **Measured:** Fixpt's `lists` (heap pairs) 21.9 ms to 15.6; the compiler
  on itself 4.19 s to 3.97, its peak memory 7.4 GB to 7.8 (each type's
  last chunk partly empty, chunks doubling).

## C44. What the compiler allocates, measured, and the waste removed (2026-10-11)

- **Measured:** kayak-rt's `heap-stats` feature (`cargo build -p kfx
  --features heap-stats`) counts every heap allocation by Rust type, and
  with `KAYAK_HEAP_SAMPLE=TYPE` the compiler's frames behind every
  millionth of that type. On itself, the compiler made 425 M allocations,
  6.9 GB, 82% of it `Cons<char>`: strings taken apart to be compared. Its
  datatypes (`Ty`, `St`, `Sexp`, ...) were about 1%.
- **Chosen:** `has-prefix?`, `string-has?`, `has-char-in?` compare in
  place (`string-ref-i64`), making nothing; facts searched with the prefix
  made once (`facts-after`); a call looks for `place-needs` only of a
  procedure given a place (`given-place?`); `mangled-clash` mangles a
  name in scope only if its mangled length (`mangled-length`, making
  nothing) is the other's. And a constant whose expression makes a list
  is made once and kept, as C21's allocating ones are, though an acyclic
  list's making has no effect: FX-26's lists are immutable, so once is
  each use (`rust-keywords`).
- **Then:** 13 M allocations, 228 MB; the compiler on itself 3.97 s to
  0.34 s, its peak memory 7.8 GB to 0.31.

## C45. Suspension: a capture and an abort, as a Rust future (2026-10-12)

`docs/research/async-rust.md`; `PLAN.md` §6.5, A1.
- **Chosen:** a task suspends as Fixpt's examples do, capturing up to its
  loop's prompt and aborting there at once; in Rust the continuation is
  the prompt's body, a future suspended at the capture
  (`kayak-rt/src/suspend.rs`). A procedure whose effect may `comefrom`
  is an `async fn`. The scheduler is the program's, KFX (K7): kayak-rt
  has no executor, only the capture, the prompt and the top level.
- **Resumed once.** KFX's form is `abort-with-continuation`, whose `k`
  fails at a second resume; `kfx/prelude.fx` defines it so in FX-26, for
  Fixpt (checked: one resume answers, two fail alike). Not chosen: taking
  `call-with-composable-continuation` and failing at a second resume in
  Rust alone (FX-26 runs the rest twice: K1 broken); refusing every
  capture (no tasks); continuations as data, an interpreter's (that is
  phase C's engine, for FX-26 at large).
- **Not chosen:** an executor in Rust whose policy is Rust (Coopt's
  scheduler): K7 asks for the policy in KFX, and Fixpt's examples show it
  needs none; Coopt's library as kayak's (Coopt's primitives, which Fixpt
  has not).
- **So Q6:** no async operations in the prelude beyond the one form; the
  library of tasks is KFX, and Fixpt runs it as FX-26.
- **Built** (the same day): the checker takes `(comefrom R)` in effects,
  masks it at a prompt as it masks `(goto R)`, and counts it as `goto`
  is, for a `Result` (a suspension is an abort to its prompt). It types
  `(abort-with-continuation tag (lambda ((k K)) body))` by a rule of its
  own: `K` written, the tag's bound with control, reading and writing at
  its region; the body may neither abort nor suspend. It notes
  `suspends` (lambdas, top-level forms), `suspending-call`,
  `suspending-body` (prompts), `lift-suspend N`; it refuses a suspension
  inside a `letrena`, as Fixpt does. The emitter writes `async fn` for a
  procedure whose written effect may suspend; a suspending procedure
  value is a `&dyn Fn(..) -> kayak_rt::suspend::Body<R>`; every call
  that may suspend is `kayak_rt::suspend::body(f(..)).await?`, boxed, so
  that an `async fn` may call itself (not chosen yet: boxing only calls
  within a recursive group, which the call graph would say); a prompt
  whose body may suspend polls it as a future, awaited where the code is
  `async` and else run to its end by `kayak_rt::suspend::block`, as a
  suspending top-level form is. `kayak_rt::suspend` is not in the glob
  import, so that its names (`body`, `block`) are free for programs.
  Fixpt's `ping-pong.fx` and `promise.fx` are corpus programs
  (`tests/kfx/tasks.kfx`, `promises.kfx`), their answers Fixpt's.

## C46. Polymorphism over effects (2026-10-12)

FX-26 has effect binders; KFX takes them (the user: KFX may take what
FX-26 has without asking). Wanted first for a library of tasks, generic
in what its tasks may do (`PLAN.md` §8).
- **Chosen:** `(poly ((e effect) ...) ...)`; in an effect, `e` is an atom
  of its own, `~e`. At a call each effect binder is an inference
  variable, `~?k`, solved as a row: fitting an argument's latent effect
  adds what it does to the row (`fit-effs`), which ends in another
  variable, open for more. The call's effect, the caller's, is the rows'
  atoms (`close-effs`); a variable nothing said is pure, as FX-26 has it.
- **Code:** what Rust does is decided by the *declared* type, which one
  `fn` serves for every instance: an effect binder may stand for an
  abort or a suspension, so a procedure whose effect mentions one is an
  `async fn` returning a `Result`, and its procedure parameters of that
  effect return boxed futures. A call that may suspend in code that is
  not `async` (a caller whose instance cannot suspend) is run to its end,
  `kayak_rt::suspend::block`; a lambda given for such a parameter is a
  boxed future. Exact, at a box per call and per closure.
- **Families over effects** (`(define-type (thunk (u effect) (l region))
  ...)`): an effect argument is a row, spliced where the parameter is.
  What Rust such an abbreviation stands for depends on the effect, so it
  has no Rust alias: each use is expanded. A datatype over an effect is
  not taken yet (one Rust type would have to serve every effect).
- **Not chosen:** a Rust function for each colour an instance needs
  (monomorphised by kfx), quicker and larger, later if it is measured to
  matter.
- **One Rust type for a family's every effect.** A procedure type whose
  effect is a binder's is, in Rust, a boxed future's, whatever effect it
  is later given: so an effect given to a family, or by `proj`, carries a
  mark, `~!` (`%effect-given` in the emitter), which keeps that
  representation and is within every effect. `(loop app @l)` in a program
  is then the Rust type `(loop u l)` is in the library.
- **A procedure that may suspend has no lifetimes** (C42's): its future
  is `'static`, as a capture's must be (`Any`), and it can touch no
  `letrena`'s data, none being suspended in (C45).
- **A global `async fn` named as a value** is a closure boxing its future
  (`async-global N`); a call through `proj` is boxed as a named one is.
- **A prompt's handler may abort or suspend**: its call is awaited and
  `?`ed.

## C47. A library of tasks, in KFX (2026-10-12)

`lib/tasks.kfx`, generic in the tasks' effect `u` and the loop's region
`l` (C46): a loop (its prompt tag, ready queue, sleepers, clock),
`spawn!`, `yield!`, `suspend!`, `sleep!` on a virtual clock that moves
only when no task is ready, `run!`, and ivars (`ivar-get!` waits,
`ivar-put!` wakes). After Fixpt's async examples and Coopt's library,
smaller: no cancel scopes, no preemption, no channels yet.
- **Calls written with `proj`.** Fixpt does not infer an effect binder
  through a loop's type (it solves the binder to the whole latent effect,
  control included, and the loop's type then fails), so the library and
  its users instantiate explicitly: `((proj spawn! app @l) lp thunk)`.
  KFX infers them, but a KFX program must be FX-26 too (K1).
- **Tested** by `tests/kfx/clock.unit` (the library and a program): two
  sleepers waking in time order, an ivar handing a value across, a
  yield; Fixpt's answer.

## C48. Runners, record and replay, in KFX (2026-10-12)

`lib/runner.kfx`; `PLAN.md` §6.1 (the user: "make sure to duplicate"
Coopt's record and replay). A program performs an operation (read a
line, write, read the clock) by suspending to its runner's prompt with
the operation and its rest (C45); the runner, a KFX procedure from step
and operation to answer, answers and resumes it. Three runners: `sim`
(scripted input, a clock that ticks an operation), `recording` (another
runner, each step, operation and answer kept), `replaying` (a record's
answers, each operation checked against the one recorded: parting from
the log, or running past it, fails saying at which step and how).
- **As Coopt's:** a run's inputs are the answers alone; outputs are
  operations, checked on replay, not stored apart.
- **Not as Coopt's, yet:** no events at scheduler steps (an operation is
  answered at once: no task waits on the world while others run), no
  live runner, no seeded delays or faults, no determinism check in
  `scripts/check.sh` (the sim is deterministic by construction; the check
  comes with seeds).
- **Live I/O is FX-26's gap**, raised (`PLAN.md` §8): FX-26 has no I/O,
  so a live runner's primitives have no FX-26 meaning, and K1 cannot hold
  for a program that uses them.
- **A procedure that takes or gives one that may suspend has no
  lifetimes** either (C46's rule, widened): its closures' futures are
  `'static`.
- **Tested** by `tests/kfx/replay.unit`: a program run under the sim,
  recorded, and replayed from the record; Fixpt's answer. A replay made
  to part from its log fails on Fixpt with the message the Rust's
  `error` gives.

## C49. FX-26's I/O and bytes, in KFX (2026-10-12)

Fixpt 17ac0c5 built what kayak asked for (`docs/requests/fixpt-world-io.md`,
`PLAN.md` §8): `u8` and `i8`; `(bytevectorof r)` and R7RS's operations on
it; the `(io k)` effects, a fixed family; `write-out`, `write-err`,
`exit`, `read-line`, `clock`, `sleep-until`, `args`, `read-file`,
`list-dir`, `write-file`; `io-error`; `main`'s result as the status. KFX
takes all of it as FX-26 has it, with the same types.
- **Confirmed to Fixpt, as built** (their `TODO.md` §84 asked): the
  allocating primitives taking a region; `io-error`'s kind a sum; bytes
  not UTF-8 a failure of its own kind; bytevectors written in place, no
  builder; R7RS's spans. Each suits KFX as it is.
- **`(bytevectorof r)` is kayak-rt's `Bytes`**, a row of cells, as a flat
  array is: aliased, written in place. A letrena's region is not yet
  distinguished (as for flat arrays): the bytes are the heap's.
- **The primitives' sums and products are kayak-rt's** (`world.rs`),
  declared as a program's own shapes would be, by the same names; the
  emitter leaves those shapes out of a program (`standard-shape?`), so the
  program's `(sumof (done unit) (failed io-error))` and the runtime's are
  one Rust type.
- **An `io` atom is `io:` and its kind**, with no region: never masked,
  apart from every other atom. A constant whose making does I/O is
  refused, as any effect but reading globals and allocating is (C21).
- **I/O primitives are plain Rust functions**, not `async`: a program
  blocks on them. Tasks that wait on the world while others run come with
  the live runner over `lib/tasks.kfx`.
- **Tested both ways**: `tests/kfx-io/`, programs whose `main` returns the
  status, run by `fixpt run` and as binaries of `crates/kfx-io`, compared
  on their standard output and error, status, and the files they leave
  (`scripts/io-programs.py`, in `scripts/check.sh`); `tests/kfx/bytes.kfx`
  in the corpus; three refused programs. Fixpt's bytevector programs are
  among those kfx now accepts (96, from 91).
- **The reader takes R7RS's hex escape** in strings (a backslash, `x41;`),
  which one of them used: kfx had read it as the letters. A control
  character in a string is written to Rust as a unicode escape.
  `i64->char`, in the prelude, over `integer->char`.

## C50. The compiler's driver in KFX, and stage 0 by `fixpt run` (2026-10-12)

`kfx/main.kfx`: reading the program (a file, or a manifest and its
files), compiling it, and writing the Rust (standard output, or a
directory's files), with FX-26's I/O (C49). `boot/src/main.rs` is now
`main` called on a big stack, and the status returned.
- **Stage 0 runs the same driver**: `scripts/stage0.py` is `fixpt run`
  of the prelude and the compiler, given the arguments kfx is given;
  Fixpt reads and writes the files itself. The source texts no longer go
  in as string literals, nor the output out as `bundle`'s one string. K1
  now covers the whole compiler, its I/O too: stage 0 makes
  `boot/src/kfx/` exactly, in 3.5 s.
- **The output directory must be there**: FX-26 makes no directories, so
  neither does kfx; the scripts make it first (`regen.sh`,
  `fixpoint.sh`, `corpus.py`, `bench.py`, `stage0.py`). A request for
  Fixpt if it is a nuisance: `make-directory`, `(io fs-write)`.
- **Messages and statuses are the binary's and Fixpt's alike**: a file
  that cannot be read ends the run with 1 ("kfx: cannot read PATH:
  DETAIL"), a wrong use with 2.
- **Fixpt 17ac0c5 writes notes on its standard output** under `run` ("; not
  in the native convention yet, ..."), among a program's own output;
  stage 0 leaves those lines out. Worth telling the Fixpt team.
- **The compiler on itself**: 0.61 s, as before the change (0.57 s, a
  little less input).

## C51. Tasks that wait on the world; a live runner; the log as text (2026-10-12)

`lib/world.kfx` joins `lib/tasks.kfx`'s loop and `lib/runner.kfx`'s
runners: a task performs an operation by filing it with an ivar for its
answer and waiting on the ivar, so other tasks run meanwhile. When no task
is ready, the operations filed are answered in the order they were filed,
each at the next step, by the runner's answerer; when none is filed, the
clock moves to the first sleeper. `lib/live.kfx`: the answerer that is the
world, by FX-26's I/O (C49).
- **Answered when the loop is idle**, not as they arrive: the live
  answerer blocks (FX-26's I/O does), and no task runs then anyway. A
  reactor that answers whichever is ready first would make the order of
  answers an input, which the log already records by step; it waits for
  I/O that does not block.
- **A run is a function of its answers**, however its tasks interleave:
  recorded live, it replays with no I/O. `tests/kfx-io/tasks-world.unit`
  runs two tasks live, writes the record to a file, reads it back, and
  replays: Fixpt and Rust alike, in output, status and the file.
- **The log as text** (`log-text`, `log-of-text`): a line an entry, its
  fields apart by tabs, escaped. The event stream, written by the program
  itself through the world; so the log's format is KFX's, not Rust's.
- **Operations on files** joined the runner's: `read-file` and
  `write-file`, of text; a failure's answer the system's words. The sim
  has no files.
- `run-ready!` and `sleeping?` in `lib/tasks.kfx`, for a loop driven from
  outside.

## C52. The compiler's I/O behind a runner: a compile recorded, replayed (2026-10-12)

`kfx/main.kfx`'s reads and writes are `lib/runner.kfx`'s operations, so
`kfx --record LOG ...` keeps each read's answer and each write, in order,
as text (C51's), and `kfx --replay LOG ...` answers from the log, failing
at the first step where the compile reads or writes otherwise; one that
ends before its log does fails too. A replay writes nothing. `runner.kfx`
and `live.kfx` are in `kfx/kfx.unit` before `main.kfx`.
- **A check of K1 and of determinism together** (`scripts/fixpoint.sh`):
  the compile of the compiler is recorded, then replayed as Rust and on
  Fixpt (`stage0.py --replay`, 4 s). The log holds every file the compile
  read and wrote (1.3 MB), so the replay on Fixpt checks every output
  byte against the Rust's, as `diff -r` does, from inside FX-26.
- **`main` is an `async fn`**: it suspends to the runner (C45), so
  `boot/src/main.rs` runs it by `block`.
- **Its errors and its exit stay direct I/O**, outside the runner: a
  failure to read is the runner's answer, recorded; what the compiler
  says about it is derived, and checked as any write is not. The `world`
  effect is every `io` kind, so the program and the live answerer have
  one effect.
- **Escaping the log is linear**: the first version appended a string a
  character, 7 s for the self-compile's log; now 0.77 s recorded, 0.69 s
  replayed, against 0.61 s plain.
- **Not recorded**: the arguments (a replay is given them again, and
  parts from the log if they differ in what they make it read or write).

## C53. Seeded simulation: delays as order, faults, determinism checked (2026-10-12)

`lib/sim.kfx`: one seed, a `u64` at a region (xorshift64*, whose `u64`
arithmetic wraps alike in FX-26 and Rust), that every random choice of a
run flows from, as in the practice Coopt's runners note surveys.
- **Delays are the order of answers**: `run-seeded!` answers one filed
  operation at a time, the one the seed picks, then runs what it made
  ready. In a world loop answered when idle (C51), when an answer
  arrives is only which comes before which; a seeded pick covers every
  order, where delays in ticks would cover the same ones less directly.
- **Faults**: `faulty` wraps an answerer and fails operations on files at
  a rate in a hundred, by the seed. Other kinds of fault (short reads,
  lost writes) when a program needs them.
- **The determinism check is a corpus program** (`tests/kfx/seeded.unit`):
  one seed run twice must give one trace, and its answer, with another
  seed's trace, is compared with Fixpt's, so the check runs on both
  implementations in `scripts/check.sh`. For the compiler, the record
  and replay of its self-compile (C52) is the check.

## C54. The first query: each form's Rust, kept between compiles (2026-10-12)

`kfx/incr.kfx`, `kfx --cache FILE PROGRAM.unit DIR`;
`docs/research/queries.md`.
- **Kept between runs, content-addressed**: kfx is a batch compiler, so a
  memo in memory alone (Coopt's, in its REPL) saves nothing. A memo's key
  is the fingerprint of all it was made from: the form's text, its facts
  at offsets from its start, and the context's fingerprint. No revisions
  or dependency lists: what a memo depends on is in its key.
- **FNV-1a in `i64`**, which wraps alike in FX-26 and Rust, rather than a
  hash in kayak-rt: K7, and Fixpt's stage 0 computes the same keys.
  Collisions (2^-64 a pair) are not guarded against.
- **The memo file is the next run's only memos**: those made or taken this
  run, so it never grows past the program. It is read and written as
  runner operations (C52), so recorded.
- **Checked by `scripts/incremental.py`**: a copy of the compiler's
  sources edited (a form moved, a body and its facts changed, a
  definition and a datatype added and taken away), compiled after each
  edit with memos and without, the outputs the same; in
  `scripts/check.sh` on three files, and with `--all` on every one (85
  compiles, all the same).
- **`scripts/regen.sh` uses memos** (`target/kfx-memos`);
  `scripts/fixpoint.sh` compiles without, so a wrong memo cannot hide.
- Measured on the compiler: 0.82 s cold, 0.32 s warm; checking is what
  is left, the next query (`Check(form)`).
- **Fixpt computes the same keys**: given the memo file the Rust wrote,
  stage 0 takes every memo, makes `boot/src/kfx/` and writes back the
  same file, byte for byte (`scripts/fixpoint.sh`).

## C55. The second query: each form's check, kept between compiles, chained (2026-10-12)

`kfx/incr.kfx`, `check-program-memo`. A form's check is a change to what
the forms after it see (the checker's `cv`): names bound, effects named,
facts noted, types found, and where the checker is (its definition, its
group, its type variables, which can carry to the next form). The memo
is that change, as text (types as an s-expression of their constructors,
read back by the reader), addressed by the form's text and a **chain**:
the fingerprint of the context the passes before the forms make, then of
each form's change in turn, its facts left out but those the checker
reads again (`place-needs`, `region-within`).
- **Early cutoff by content**: a body edited, its interface the same,
  leaves the chain, and every later form's key, the same.
- **Coarse, on purpose, first**: a form's key holds everything before
  it, not the names it reads. A new definition early re-checks every
  form after it (0.28 s on the compiler, the emitter's memos still
  taken); a body edited, only itself (0.12 s). Keys by the names read
  would narrow it: the checker would have to say what it looked up.
- **Modules are checked always**, and continue the chain by their text.
- **Checked**: `scripts/incremental.py` gained edits across files, a form
  whose type is found from a definition in another file that changes;
  a planted bug (the chain ignoring what forms bind) is caught by it.
  The compiler on itself: 0.73 s plain, 0.9 s cold with memos (writing
  them), 0.12 s warm.
- **A memo is one compiler's** (found 2026-10-12, after C55): a key says
  what a form's Rust was made from, not by which compiler, so a memo file
  kept across a change to the compiler gives the old emitter's Rust for
  every form that did not change. `scripts/regen.sh` names its memo file
  by a fingerprint of the generated compiler (`boot/src/kfx/*.rs`), one
  file per compiler; the other scripts make a fresh file each run. The
  driver itself cannot tell (FX-26 gives a program no way to know its own
  code): a memo file is the caller's to keep apart. In TODO.

## C56. The front end, stage 1: standard operations, symbols, arrays of values (2026-10-12)

`docs/research/front-end.md`, its plan's first stage. From Fixpt
17ac0c5's standard environment, at its types, each with kayak-rt's Rust
as Fixpt's runtime has it:
- integers: `bitwise-and`, `-ior`, `-xor`, `-not`, `arithmetic-shift`,
  bignums too (two's complement, a count past 2^24 too large);
- characters and strings: `bool=?`, `char<?` and kin, `char-downcase`,
  `string<?` and kin, `string-compare`, `string-search`, `string-ci=?`,
  `string-downcase`, `substring`;
- **symbols**: `symbol` a base type, kayak-rt's `Symbol` (its text; FX-26
  interns, so `eq?` of two is equality of their names); `'name` and
  `(quote name)` (a quoted datum of another kind waits for `datum`);
  `string->symbol`, `symbol->string`, `symbol=?`, `symbol-compare`;
- **arrays of values**, `(arrayof T R)` at a region (the flat array of a
  bloblet keeps its literal length): kayak-rt's `Array<T>`, a row of
  cells; `make-array`, `array-ref`, `array-set!`, `array-length`,
  `array->list`, `list->array`; printed as Fixpt prints them.
- **The reader reads `'d` as `(quote d)`** in a program's text only: the
  checker reads its own facts with the same reader, where `'r` names a
  region binder.
- `tests/kfx/standard.kfx`; kfx now accepts 105 of Fixpt's programs (96),
  all agreeing.

## C57. Unions stay out of KFX; FX-26's engine is their home (2026-10-12, the user's)

KFX does not take FX-26's unions of shapes. A union as a Rust enum is
sound only for subtyping at a value's outermost level (one injection,
which keeps a pair's identity); FX-26's covariant frozen lists and
procedures' contravariance would need a list copied or a procedure
wrapped, a cost and an identity FX-26 does not have: a meaning that
differs, which K1 forbids. A uniform representation in Rust would make
the transpiled code an engine. So unions, and `datum`, which is one, are
refused for Rust, and come with kayak's own FX-26 runtime (phase C),
whose values are uniform and whose subtyping is free.
- **The user (2026-10-12):** KFX need not become as advanced as FX-26,
  so long as the self-hosted compiler stays clean in KFX and its scope
  can grow to FX-26 cleanly. Neither needs unions in KFX.
- **The checker is not KFX the language**: as kayak's FX-26 compiler, it
  checks FX-26's unions when the FX-26 path needs them; KFX's
  restriction pass refuses them for Rust.

## C58. FX-26's engine, in KFX: an interpreter, a CEK machine (2026-10-12)

`fx26/`, `docs/research/engine.md`; phase C begun before B is done (the
user, 2026-10-12: features come into the checker and the engine
together, the restriction pass refusing for Rust what Rust cannot have).
- **A CEK machine**: a control (an IR node in an environment, a value, or
  an application), and a continuation that is a list of frames, data.
  Prompts are frames; a capture takes the frames above the nearest prompt
  of its tag; resuming puts them back, any number of times. One loop,
  one procedure calling itself in tail position: an application is a
  control too, since two procedures calling each other in tail position
  would grow Rust's stack.
- **Values are uniform** (one KFX datatype): integers of every width are
  `int`, wrapped by their operations; pairs' parts are cells. So unions
  (C57) will cost nothing here.
- **The IR resolves names** (frames' depth and slot, globals' index,
  primitives by name); FX-26's derived forms are lowered to the core;
  types are dropped.
- **The checker is the compiler's**: what kfx accepts, the engine runs,
  for now; FX-26 at large as the checker grows.
- **The heap is KFX's** (never freed); a collector of the engine's own is
  a later step.
- **Built as `kayak-fx26`** (`crates/kayak-fx26`, its Rust generated from
  `fx26/fx26.unit` and kept golden); `scripts/engine-programs.py` (in
  `scripts/check.sh`) runs each corpus program's `main` on it and
  compares with Fixpt's answer, K6. First: 21 of 53 agree, none differ;
  the rest stop at forms not lowered yet (regions, `case`, modules,
  bloblets, the fixed widths but `i64`). A program that agreed and no
  longer does fails the check (`tests/engine-passing`).

## C59. The engine grown: the corpus, and Fixpt's programs, compared (2026-10-12)

- **Modules** are products of the values they give: a module form is a
  `letrec` of its items, then that product; `(with m e)` puts the module
  in a frame of its own, through which `e`'s names are found, so that it
  shadows the locals outside it, as FX-26's does. What each top-level
  module gives is known when lowering; a module given as a parameter
  gives the names every `moduleof` type of the program names (a name
  not its own would be shadowed wrongly: none of the programs has one).
- **Regions run as the heap** (`letregion`, `letrena`, `letreap`, places
  given to `rcons` and `rnew` dropped); `case`; fixed widths, each
  operation on `int` and wrapped; `f64`; bytevectors; flat arrays of
  `i64`; I/O, its answers FX-26's sums.
- **Compared with Fixpt on its own programs too**:
  `scripts/run-fixpt-programs.py` runs the engine on each program kfx
  accepts; a wrong value fails the check, a program not run yet is
  counted (`tests/engine-fixpt-passing` keeps those that agreed).
- **Measured**: 49 of the corpus's 53, 97 of Fixpt's 105, agree; none
  differs. Left, in both: bloblets.

## C60. The restriction pass, begun: composable continuations (2026-10-12)

The user (2026-10-12): one checker, saying what FX-26 says, and KFX's
restrictions a pass after it, which kfx runs and the engine does not; a
program is KFX if it passes both. Less risky than a mode in the checker:
the checker's rules are FX-26's alone, and what Rust cannot have is
refused in one place. `kfx/restrict.kfx`.
- **First moved**: `call-with-composable-continuation`, now in the
  primitives' table at FX-26's type (its `(composable T A D R)` a type of
  its own, below the procedure it is, its `D` the same both ways as a
  tag's); the pass refuses it, its type, and a `proj` of a control
  primitive, with kfx's message of before.
- **What it asked of the checker**: a `proj` of a `proj` (FX-26's nested
  `poly`s, their binders in KFX's one order); the control primitives KFX
  types by rules of its own, at FX-26's types for a `proj`; effect bounds
  compared the same both ways with their variables solved (`same-effs`,
  the side that is open grown first, so that a variable is not taken for
  the least it may be).
- **Where the pass looks**: the program's text, for now (the primitive's
  name, the type written); facts from the checker when a restriction
  needs a type (unions).
- `tests/kfx-stricter/comefrom.kfx` is refused as before, by the pass now.

## C61. Names: KFX's rules to the restriction pass (2026-10-12)

- **Redefinition**: FX-26 may define a global again, and what used it sees
  the new one (Fixpt's `redefine/`). The checker now allows it at the
  same type; at another type it refuses at the redefinition, where FX-26
  refuses at the next use, so that the two refuse the one program. The
  engine's global is overwritten, FX-26's meaning. KFX defines each name
  once, none a primitive's or kayak-rt's: the pass's, with kfx's
  messages of before, at the definition's line.
- **Types' and modules' names as Rust has them** (none Rust's or
  kayak-rt's, `Cell`; KFX's names, C27; none two alike camel-cased): the
  pass's. A type named as an effect stays the checker's: FX-26's rule.
- **Left in the checker for now**: the rules on every binder's name (KFX's
  names, two that are one in Rust), which the pass would have to follow
  scopes to check; labels one in Rust.
- Beyond KFX, the engine agrees on 14 of Fixpt's programs (11).

## C62. Unions of shapes: the checker's and the engine's, refused for Rust (2026-10-12)

C57's decision, carried out. In the checker (FX-26's rules, `docs/fx26.md`
"Unions of shapes"):
- `(union T ...)`, flattened, each member once, of two members or more
  whose shapes at run time differ (a type variable or a type of no shape
  known is no member); `false`, the type of `#f` alone, below `bool`.
- **Subtyping**: a member below its union, by the member of its shape; a
  union below what each member is below. A literal where a union is
  expected is its member of the literal's shape; `#f` where a `false` is.
- **Narrowing**: a shape's test of a local of a union (`int?`, `null?`,
  and the rest, at FX-26's type) narrows it in its `if`'s first branch
  and its `cond`'s clause, to the member of that shape, and after it to
  the rest; `typecase`, each arm binding its member, what it leaves
  narrowing the arms after it.
- **Not yet**: `pairof` (a pair never `nil`) and the `nil` type, so
  narrowing of lists is coarse (a list stays a list); a datatype's shape;
  tests' propositions written in types (`(bool (then ...))`); `datum`.
- **The engine**: the shape tests, and `typecase` lowered to them.
- **The restriction pass** refuses `union`, `typecase`, `false` and the
  shape tests for Rust, with C57's reason.
- Of Fixpt's union programs it accepts, four now run on the engine with
  its answers (`false`, `shapes-narrow`, `typecase`, `union-subtyping`);
  beyond KFX, 18 agree, none differ. The compiler's own `procedure?` (of
  `sexp`s) is `lambda-form?` now: FX-26's is standard.

## C63. The compiler run on the engine: the second self-hosting, begun (2026-10-12)

`kayak-fx26 --run FILE ... -- ARG ...` runs a program as `fixpt run` does:
its arguments the ARGs, `main`'s result its status. Given the compiler's
own files (`kfx/kfx.unit`'s), the engine runs kfx itself, which checks,
lowers and runs some 9,000 lines of KFX and then compiles its argument.
- **The compiler on the engine compiles as the compiler does**: for
  `tests/kfx/arith.kfx`, `escapes.kfx`, `tasks.kfx`, its Rust is the
  corpus's, byte for byte, in 5 to 6 s (the compiler itself checked and
  lowered each time).
- **The self-compile does not finish**: killed after 196 s, of memory. The
  engine allocates about 4 GB a second (23 GB resident for the 5 s run),
  and nothing is freed: its values are kayak-rt's heap's, never freed
  (C43). The engine needs a heap of its own and a collector (engine.md,
  step 5), before the second self-hosting can close.

## C64. The engine's heap of its own, collected, in KFX (2026-10-12)

`docs/research/collector.md`; the user: the collector first, in KFX first
(a Rust one behind the same interface later, if measured to matter, or
when incremental collection does; a GC for kayak-rt itself, for every
KFX program, a separate question).
- **Words in two flat arrays of `i64`** (`fx26/heap.kfx`): fixnums,
  pointers, immediates and IR nodes by their low two bits; objects a
  header and a payload; Cheney's copying collector, written in KFX; the
  spaces grown, both, when what lives fills more than half.
- **The machine over words** (`fx26/vm.kfx`): the CEK machine's frames,
  environments and values all objects in that heap; frames never written
  once made, so a continuation is a count of frames from its top, copied
  at each resumption (any number of times). Collection only at the top of
  a step, its registers then the roots; a step that needs more room than
  the reserve (a primitive of its input's size, a continuation's frames)
  asks first, and the step is begun again after.
- **The IR as a table** (`fx26/table.kfx`), numbered from `lower.kfx`'s
  tree, unchanged; primitives by number (`fx26/vprims.kfx`, generated
  from the list), never by name.
- **Nothing of kayak-rt's allocated in a step**: no datatype's value, list,
  closure or string; KFX data only at loading (the IR, interned names)
  and for I/O, in proportion to what is read and written.
- **Measured**: the compiler run on the engine compiling `arith.kfx`,
  450 MB resident (23 GB before), 11 s (5.5 s before: the words cost);
  the same Rust. The corpus (49 of 53) and Fixpt's programs (97 of 105)
  as before; none differs.

## C65. The engine's REPL (2026-10-12, the user's request)

`kayak-fx26` alone, or `kayak-fx26 repl`: Fixpt's REPL's ways. A form is
read as it is typed (a continuation line's prompt `     | `, until the
text holds whole forms), checked against what was defined before it,
lowered and run on the machine over words, whose heap and globals it
keeps; written `value : type ! effect`, a definition as its name and
type. A refusal or a failure is written and the REPL goes on: the loop
is the hand-written Rust's, which catches it (the checker refuses by
failing, and KFX catches nothing). `fx26/repl.kfx`.
- **Effects written as FX-26 writes them** everywhere now (`show-effs`):
  `(maxeff (read (globals f)) spin)`, not kfx's atoms.
- Not yet: Fixpt's `,` commands; line editing; a datatype's definition
  writes nothing.

## C66. The second self-hosting, closed (2026-10-12)

The compiler, run on kayak's own FX-26 engine (`kayak-fx26 --run`, C63,
on its collected heap, C64), compiles itself and makes `boot/src/kfx/`
byte for byte: 278 s, 783 MB resident (the self-compile as Rust, 0.6 s).
`scripts/selfhost-engine.sh`; not in `scripts/check.sh` for its time.
- **The collector's share**, measured by sampling a compile on the engine:
  about 5% of the time the engine is busy; the rest the machine's loop,
  allocation and thread-local lookups of the heap's spaces. The KFX
  collector stands (C64's plan: Rust if its share mattered).
- **Speed, next**: the control primitives' numbers were found by name at
  each primitive's call (fixed: 5x); the thread-local lookups and the
  machine's dispatch are what is left to win.

## C67. The feedback loop: the check in 46 s, from about 4 minutes (2026-10-12)

The user's motive for speed: a faster cycle. Measured first, step by step
(warm): `run-fixpt-programs.py` 133 s, `incremental.py` 39 s,
`corpus.py` 36 s, `fixpoint.sh` 17 s, the rest a few seconds.
- **Fixpt's runs cached** (`scripts/fixptcache.py`): what Fixpt says of a
  file depends on the Fixpt pinned, the arguments and the file's text
  alone, so each run is kept under a hash of those (`target/fixpt-cache/`).
  Fixpt's values for each program computed once, not twice.
- **Run in parallel**, as many as there are processors: kfx's compiles,
  Fixpt's runs, the Rust programs', the engine's; an incremental check's
  two compiles of each edit; stage 0's three runs of the compiler.
- After: `run-fixpt-programs.py` 7 s, `corpus.py` under 1 s warm (4 s
  cold), `incremental.py` 23 s, `fixpoint.sh` 8 s; `scripts/check.sh`
  46 s, the same results. `scripts/regen.sh` after an edit to the
  compiler, 5 s (rustc 2 s of it).

## C68. Pairs and `nil`: FX-26's lists, in the checker and the engine (2026-10-12)

The user (2026-10-12): work through FX-26 beyond KFX, autonomously.
- **`(pairof A B R)`**, a pair never `nil`, its head and tail of any
  types; **`nil`**, the type of the empty list alone. A pair whose tail is
  the list, and `nil`, are below the list.
- **A list narrows**: a test of `nil` or of a pair takes a list as `nil` or
  its pair, so `(if (null? xs) a b)` has `xs` a pair in `b`; and the two
  again are the list. Checked not to change what kfx writes of KFX: the
  compiler regenerates itself unchanged but for the checker's own files,
  and the corpus's Rust is the same.
- **Narrowing through `not`, `and`, `or`**, as FX-26's size facts go:
  `(not t)` swaps `t`'s; `(and a b)` holds where `b` holds where `a` does;
  `(or a b)` fails where `b` fails where `a` does.
- **FX-26's rules where KFX's would refuse**: `car`, `cdr` of a local known
  to be a pair, of any types; `cons` where a pair is expected; `null?`,
  `pair?` of a local of a union, a pair or `nil`; `no-pair`.
- **For Rust**, the restriction pass refuses `pairof` and `no-pair` (C57's
  reason): KFX's lists stay `List<T>`.
- Not yet: narrowing a path (`(car x)`, `(extract p a)`), Fixpt's TODO §54.
- Beyond KFX, the engine agrees on 21 of Fixpt's programs (18).

## C69. `eq?`, `set-car!`, `set-cdr!`, `stay-cellular` (2026-10-12)

At FX-26's types, in the checker and the engine (`eq?` a word's identity:
a pointer's for an object, a value's for an immediate or a fixnum, as
FX-26's `eq`); a list where a union of `nil` and a pair is expected is
each of the two. Refused for Rust by the restriction pass: KFX's lists
are immutable in Rust, `eq?`'s identity has no Rust that means the same of
KFX's `Copy` values, and `stay-cellular` is a direction to Fixpt's
machines. Beyond KFX, 23 agree (21).

## C70. A `poly` directly in a `poly`, as one (2026-10-12)

FX-26 writes some types `(poly ((t type)) (poly ((e effect)) T))`, binders
by kind, as its standard environment does; that means one `poly` with the
binders in order (C60's `proj` of a `proj` already takes them so). The
checker reads a definition's type so; the restriction pass asks KFX's
programs to write one `poly`, as KFX's Rust has it. A `poly` as a type of
its own (a local's, a recursive type's), first-class polymorphism, is not
done: the checker's types have no such type, and adding one touches all
of them. Beyond KFX, 24 agree (23).

## C71. Variadic procedures; `define*`'s call of itself (2026-10-12)

- **`(vsubr E T R)`**, FX-87's, as FX-26 has it: a type of its own; a
  `vlambda` checked as the lambda of one list it is (its list a
  `(listof T acyclic)`), against the procedure of that list; a call with
  `n` arguments as the procedure of `n` `T`s; `(apply f xs)`. The engine:
  a procedure of arity -1 takes its arguments as its one slot's list;
  `apply` copies the list it is given, as FX-26's does (R7RS's fresh
  list). The restriction pass refuses them for Rust (a Rust `fn` has its
  arity), for now. `list` as a value, a `vsubr`, not yet.
- **`define*`'s call of itself reads itself**, `(read (globals name))`, as
  FX-26 finds it, not every global (`@globals`), which a body that called
  it then had to say. kfx's own Rust unchanged by it (regenerated, the
  corpus the same); kfx now accepts 106 of Fixpt's programs (105), all
  agreeing.
- Beyond KFX, the engine agrees on 28 (24).

## C72. Continuation marks: the checker's and the engine's (2026-10-12)

`(mark-key T R)` and FX-26's five operations at its types:
`make-continuation-mark-key`, `with-mark` (a thunk called with the mark on
the continuation: in place of one of the same key on the frame below, in
tail position, as Scheme's), `first-mark`, `current-marks`, `marks-of`
(a captured continuation's). In the engine a mark is a frame of its own
on the continuation, which a capture takes with the rest, so marks travel
with continuations as FX-26's do; a key an object compared by identity.
Refused for Rust by the restriction pass (C18's: a prompt in Rust would
save and restore the marks around each poll; for cancel scopes in KFX's
tasks, later). `marks-of.fx` waits on FX-26's inference of a lambda's
parameter from a later argument. Beyond KFX, 30 agree (28).

## C73. `case` on symbols, KFX's too (2026-10-12)

Symbols being KFX's (C56), a `case` on one is KFX's, not only the
engine's: its data compared by name in the `match` arm's guard
(`symbol_eq_p`), as a string's are; the engine's `%case=` compares them
as words. kfx now accepts 108 of Fixpt's programs (106), all agreeing; the
engine agrees on 100 of them.

## C74. Lambdas heard last, as FX-26 hears them; KFX's too (2026-10-12)

A lambda given with a parameter of no type written, where the
procedure's parameter, as instantiated, holds an inference variable, is
checked after the call's other arguments, as FX-26 checks lambdas, so a
later argument may say its parameters' types. KFX's own rule (rustc
checks a call in order, and finds a closure's parameters' types only from
what comes before it) is met another way: the emitter writes the closure's
parameters' types the checker found (`param-types`, already its fact for
a lambda whose types are all known). So KFX takes FX-26's rule; kfx's own
Rust is unchanged, and `tests/kfx-stricter/lambda-order.kfx` is a corpus
program now (`tests/kfx/lambda-order.kfx`), its answer Fixpt's.
`run/marks-of.fx` gets past it, to KFX's `spin` for a call of a
procedure it cannot see, a stricter rule of its own, left for now.

## C75. Unions of shapes in KFX, as a uniform `Value` (2026-10-12)

The user's proposal (`docs/research/uniform-values.md`), its first step;
C57 superseded for unions. A union's type is kayak-rt's `Value`
(`crates/kayak-rt/src/value.rs`): a scalar member unboxed in it, a list
as `nil` or its pair, any other member (a procedure, a reference, an
array, a sum, a product) behind `Any`, its shape kept beside it. Every
other type is Rust's own, as before, generic code instantiated at
`Value` where a type variable is a union.

- **The checker says where**, as facts, at the subsumption points it
  already finds: `check` and `check-by-synth`, where a non-union meets a
  union (`box scalar|list|other SHAPE`), and the reverse (`unbox`,
  `unbox-list`, `unbox-other`, with the member's type); a narrowed local
  named (a test's narrowing is now a bind of its own kind, `narrowed`,
  whose origin, the local it narrows, says what Rust it is); a
  `typecase` arm's binder; the call of a local narrowed to a procedure
  (`head-unbox`); `null?` of a union (`union-test`). A check that boxes
  has the union's type, not the member's.
- **The emitter writes them**: `kayak_rt::value::ToValue::to_value(x)`,
  `value::list(x)`, `value::other(x, Shape::S)`, `value::unbox::<T>(x)`
  and its kin; `typecase` as a chain of shape tests, each binding its name
  unboxed (no `let` when the arm does not name it); `int?` and the other
  shape tests generic over kayak-rt's `Shaped`.
- **`nil`, a list narrowed so, stays a list**, its element type Rust's to
  find, not a `Value`: the compiler narrows lists everywhere, and none of
  that may change.
- **A pair whose tail is a list is that list** (`(pairof int ints @r)`
  is `List<Int>`): `pairof` is written in KFX now. A pair of another tail,
  and a member of no shape kayak-rt boxes yet, the emitter refuses (C75),
  not the checker, which the engine shares.
- **Still refused, for Rust**: `eq?`, `set-car!`, `set-cdr!`, `no-pair`,
  `stay-cellular`: the argument that a copy or a box cannot be seen rests
  on their absence.
- **What a program sees**: nothing new. A box is a new object where FX-26
  has the old one, which only `eq?` could tell.

Measured: kfx accepts 114 of Fixpt's programs (108 before), all agreeing;
`tests/kfx/unions.kfx` in the corpus, Fixpt's answer, the engine's too.
Next (TODO §24): general pairs as `Value`s and `car`/`cdr` of a union,
then `datum` with `quote` of lists, a list given as a list of a union by a
view; first-class `poly` after.

## C76. Answering the user: why KFX says `spin` where FX-26 does not (2026-10-12)

The user asked whether `run/marks-of.fx`'s `spin` is KFX trusting declared
types less than FX-26. It is not: both trust a type without `spin`. The
`call-with-composable-continuation` call alone gets KFX's `spin`, from its
general rule for a polymorphic callee whose instance could be applied to
itself (`poly-self-application`), where FX-26 has a syntactic rule of its
own for the call: none when the receiver is a `lambda` that names its
continuation only as a call's operator, outside a `lambda` or a prompt.
Queued (TODO §25): FX-26's rule for that call, ahead of the general one.
It matters to the engine only, since kfx refuses the primitive for Rust.

## C77. `datum`, pairs of a union's, `quote` of any datum; `typecase` as FX-26 writes it (2026-10-12)

Step 2 of the uniform values (C75).
- **`datum`** is a type of the checker's, shared with the engine: a name
  whose members, `(union int f64 char bool string symbol nil (pairof datum
  datum acyclic) vector)`, are found when wanted (`members-of`), since it
  mentions itself. FX-26's has `bytevector` too; KFX's has not yet
  (stricter). A list fits `datum` when its elements do, at `acyclic`, and
  a list literal where a datum is expected is a list of datums.
- **A pair whose tail is no list is a `Value`** in Rust, as a union is: a
  pair of `Value`s, `cons`ed by kayak-rt's `value::cons` (`value-cons`).
  `car` and `cdr` of a union with a pair, or of such a pair, are kayak-rt's,
  checked (`value-call`), and unboxed where their part is no union. A pair
  may be at `acyclic` (FX-26's).
- **A typed list is given as a datum, or for a union, as it is**: its
  pair, seen through `PairLike`, boxes each element as it is read, a view,
  so no copy; narrowed back, it is found again by its type.
- **`quote` of any datum** KFX reads (no dotted list, no vector; `'()` is
  refused, write `nil`): an atom as itself, a list a `datum` built where it
  is written; in the engine, by the standard `cons` whatever the program
  binds the name to. `datum-list->vector` and `vector?`, in the engine too
  (an object kind of its own, `ob-vector`).
- **`typecase` as FX-26's parser has it**: `(shape name body)`, one body;
  an `else`, last, required, `(else body)` or `(else name body)`; the
  shapes FX-26's (`bloblet`, not `array`). KFX had accepted more (several
  bodies, no `else`), which K1 forbids.
- **Termination**: a `car` or `cdr` of a pair at `acyclic`, or of a
  union's pair member at `acyclic`, is a part, as FX-26 has it; a
  `typecase` arm's name is the value tested.
- The engine's primitives' generator is `scripts/gen-vprims.py` now, not a
  script kept in a commit message.

Measured: 117 of Fixpt's programs through kfx, all agreeing; the engine
109 of them. `tests/kfx/datum.kfx` in the corpus, the engine agreeing too.
Left (TODO §24): quasiquote (KFX's reader has no `` ` `` or `,`), a typed
list given as a list of a union (`(listof int)` as `(listof small)`),
`datum-int?`, `bytevector` in `datum`, first-class `poly`.

## C78. Lists invariant where they may be written; copied, boxed, at `acyclic` (2026-10-12)

The checker had taken every list as covariant in its elements, which
FX-26 does only at `acyclic` (frozen): at a region that may be written, a
list of ints given as a list of `(union int string)` would let a string be
written into the ints, and FX-26 refuses it. KFX accepted it (K1), harmless
until unions made subtyping of elements more than effects'. Now invariant
there, as FX-26 (`tests/kfx-refused/list-invariant.kfx`).

At `acyclic`, a list whose elements are no union's given as a list of a
union's is copied, each element boxed (`box elements`,
`value::boxed_list`): frozen, nothing can tell the copy from FX-26's one
list. One level only: a list of lists of ints given as a list of lists of
a union is still rustc's error, not kfx's refusal; and so is a procedure
given where one of wider parameters, or of a union result, is wanted (a
wrapper, TODO §24).

## C79. Quasiquote; quotes interned in the engine (2026-10-12)

KFX's reader reads `` ` ``, `,` and `,@` (in a program's text, as `'`,
C56). The checker takes FX-26's rule: `,e` at depth 1 is a datum, `,@e` a
list of datums (exactly: FX-26's `append` refuses a list of ints, so KFX
does too, K1); a nested `` ` `` deepens and `,` shallows, as R7RS counts.
kfx builds a quasiquote with `value::cons` and `value::spliced`; the
engine with the standard `cons` and `append`, whatever the program binds
the names to.

The engine interns quotes as FX-26's heap does: a quoted list is a
literal of its own (`v-datum`), made at loading as constants, each pair
and atom found again by its text, so that equal quotes, and a
quasiquote's constant parts, are one object (`eq?`,
`datum/quote-identity.fx`, which now agrees). kfx needs none of it: it
refuses `eq?`.

Still not read: vectors' `#(...)` and dotted lists, in a quote or out.

## C80. The reader's vectors and dotted lists, in quotes (2026-10-12)

KFX's reader reads `#(d ...)` as `(#vector d ...)`, a head no KFX name can
be (C27), which only a quote or quasiquote takes: a `datum`'s vector, by
`datum-list->vector` of the list. A `.` before a dotted list's last item
is taken by a quote, a quasiquote and the engine's loading; anywhere else
it is the name `.`, which the checker refuses as before. A `#(...)`
outside a quote is a call of `#vector`, refused: FX-26's is a vector, which
KFX's programs have had no use for.

Measured: 119 of Fixpt's programs through kfx, all agreeing (`datum/quote`
and `datum/quasiquote` now); the engine 111.

## C81. Self-application by FX-26's `cyclic`; Fixpt's refused programs checked (2026-10-12)

C76's question, answered in code. KFX said `spin` for a call of an unseen
procedure, and of a `poly` given such a procedure, whenever a type could
hold any procedure; FX-26 says it only when a type is *cyclic*: a cycle in
it runs through a procedure's parameter, so that a procedure of it could
be given itself (Fixpt's `cyclic`, infer.rs). KFX's `self-applicable?` is
that walk now: only a datatype or `datum` can be met again on the way
down, so the trail is their names, each with whether a parameter was
crossed on the way to it; a datatype's arguments are taken, cautiously,
as parameters, as FX-26 takes them. `run/marks-of.fx` gets past it (to
the engine's marks), and 28 of Fixpt's programs beyond KFX agree now (26).

A looser rule could accept what FX-26 refuses, so check.sh now runs
`scripts/refused-fixpt-programs.py`: every program Fixpt's checker
refuses (268), refused by kfx too. All are.

A gap found on the way: `(let ((ts2 (cons x ts))) ... (any? (lambda (...)
(f ... ts2)) ...))` in a recursive group, `ts` a list at `acyclic`, which
FX-26 types as a pair at a fresh region (so no `trails`) and KFX as the
list; the fixpoint check found it in the compiler's own text. Written
`(the trails ...)` there; the rule itself is TODO §25's.

## C82. A procedure given where one of a union's parameters or result is wanted: wrapped (2026-10-12)

`(subr (small) int)` where a `(subr (int) int)` is wanted, or `(subr ()
int)` where a `(subr () small)` is: the checker notes `wrap` with how each
argument the caller gives is boxed for the procedure, and how its result
is boxed for the caller; the emitter writes a closure that does so,
`lambdaN(move |a__0, ...| box((f)(box(a__0), ...)))`, as a lift (C45) is
written. A new closure where FX-26 has the old procedure: only `eq?`
could tell, which KFX refuses. One level, as C78's lists: a wrapper's
argument that is itself a procedure or a list to convert is not done, and
rustc, not kfx, refuses it.

## C83. Dogfood after C75–C82: what KFX's own code could use (2026-10-12)

The user's practice (2026-10-12): after a round, look for where the
compiler's own code could use what KFX gained. Found:
- **Quasiquote for syntax the compiler makes** (31 places build `(lst
  (list (sym "..." "" 0) ...) "" 0 0)`, settled types and rewrites): not
  taken. A quasiquote builds a `datum`, which has no source positions,
  and the compiler's `sexp` keeps a place on every node, which facts and
  messages are keyed by; converting to and from would cost more text than
  it saves.
- **Tables kept as text** (`primitives`, `prelude`, `control-schemes`,
  `io-error-text`): not taken. They are FX-26 text that Fixpt reads too
  (the prelude), and a fact's types are text by design (C54's memos).
- **"A list of one, or none"** (46 `(null? found)`, a lookup's answer):
  the one real use, `(union nil bind)`, says what it means and costs the
  same (a box where a `cons` was). Blocked: KFX's unions hold no datatype,
  where FX-26's do (shape `sum`, `unions/more-shapes.fx`). That FX-26 gap
  first (C84); then the lookups, one at a time, measured.

## C84. Datatypes in unions; `nil` for a union's `nil`; narrowing through `and`/`or`; the first union in the compiler; a slowdown found (2026-10-12)

- **A datatype is a union's member**, of shape `sum`, as FX-26 has it
  (`unions/more-shapes.fx` agrees now; with `f32?`). A generative type is
  named among the datatypes too, with no variants: still refused as a
  member, its shape unknown to FX-26.
- **`nil` where a union with a `nil` member and no list is wanted** is
  that member, as FX-26 takes it (KFX had refused it). Boxed, it is
  `Value::Nil`, written so, nothing computed.
- **Narrowing through `and` and `or`'s own operands**: each checked where
  those before it hold (`and`) or fail (`or`), as FX-26 reads `(if a b #f)`
  and `(if a #t b)`. Narrowing had reached only the branches of the `if`
  or `cond` they test.
- **The dogfood trial (C83)**: `find-family` returns `(union nil dt)`,
  where it returned a list of one or none; its ten callers take the `dt`
  as narrowed, no `car`. The compiler's speed is unchanged (0.77 s). The
  other lookups (`find-bind`'s 36 callers and more) may follow, one at a
  time, now that the pattern is shown and costs nothing; not a sweep. A
  bootstrap note: the new source may use what only the new compiler
  checks only after a regen has made it the seed (here, narrowing through
  `and` in `generative?`, written with `if` for now).
- **A slowdown, found by the trial's measuring**: kfx compiling itself had
  gone from 0.8 s to 2.5 s since C75. The emitter's new clauses (box,
  unbox, wrap) each searched every fact of the program for each expression
  written. `put-exp` now finds a place's facts once and asks of those.
  `scripts/fixpoint.sh` now says the self-compile's time, so that the next
  such slowdown shows.

C84's slowdown, measured after (the user asked it be understood first):
each commit's kfx built and timed compiling its own source, best of three
— 67b58e1 (before C75) 0.92 s; C75 1.86; C77–C80 2.1–2.2; C81 2.2; C82
2.4; C84 0.79. All of it was the emitter's clauses each searching every
fact for each expression: C75's box and unbox, C77's value calls, C82's
wrap, each one more. With a place's facts found once, the older clauses
(lifts, literals' facts) are cheaper too, hence below 0.92. `--record`
costs nothing (0.71 s either way); fixpoint.sh's first figure is one cold
run. The engine, which runs the same checker, is unchanged: both
engines, running the same compiler source on `arith`, 2.2 s and 450 MB.

## C85. A module's items as FX-26's `letrec*` (2026-10-12)

`docs/research/modules.md`, stage 1. A typed lambda may name any item of
its module, earlier or later; a value that is no procedure is made when
its item is, so everything it reaches through procedures must be made
before it, else refused as FX-26 refuses it. KFX checks the items in an
order their references allow (types as written, then each set of items
that reach one another after the sets it reaches), a set of procedures
that reach one another, or one reaching itself, as a `define-rec`: so
`spin` and termination are a group's, as FX-26's letrec gives them.
Items that reach one another not all typed lambdas are refused. A lone
`define*` stays as written. The emitter needs nothing: a Rust `mod`'s
items are in no order, and constants are made at first use (C21).
FX-26 no longer finds a value alone in a module that includes another,
so KFX's `found-alone` is gone (`kfx-stricter/include-literal` with it),
and `module-forward` is a corpus program now (`tests/kfx/module-letrec`).

## C86. A module's names: values and types apart; one type from two includes; hidden datatypes (2026-10-12)

Stage 2. A module's values and types are named apart, as at the top
level (a type and a value may share a name; hiding one does not hide the
other). A type two included modules both give is one name where it is the
same type both times (resolved through abbreviations, written alike, and
naming none of the modules' own types); the emitter imports it once. A
hidden datatype or generative type is allowed, its constructors hidden
with it, and refused, as FX-26 refuses it, where a type the module shows
mentions it. The modules a module includes are made before its items, in
the engine too (`modules/include.fx` had differed there).

Measured: 130 of Fixpt's programs through kfx (122 before stage 1, 120
before modules), all agreeing; the engine 121.

## C87. `extend`, effects through `include`, items shadowing primitives; `Cell` unreserved; a K1 gap closed (2026-10-12)

Stage 3 (`docs/research/modules.md`).
- **`(extend e0 e1)`** with `e0` a module defined at the top level: the
  module `(include e0 (except ...))` then `e1`'s items, or `(include e1)`
  when `e1` is a name, the names `e1` gives not taken from `e0`
  (`module-written`, read by the checker, the emitter, the restriction
  pass and the engine alike). FX-26's meaning comes out of Rust's own
  scoping: a `mod`'s items shadow what it imports, and `e0`'s procedures
  keep `e0`'s values. An `e0` made where it is extended is refused, for
  now (`modules/extend-abstract.fx`).
- **An included module's effects** are the module's, as its types are
  (`include-effects`, `extend-sees-types`).
- **A module's item may shadow a primitive or a global**, as FX-26's may
  and as a Rust `mod`'s item shadows a glob import; at the top level the
  rule is C61's, still.
- **`Cell` is no longer a reserved type name**: the emitter writes Rust's
  `Cell` as `std::cell::Cell` everywhere, and kayak-rt does not export it
  (`kfx-stricter/type-named-cell` gone).
- **A K1 gap the last showed**: `car`/`cdr` of a pair of a union's (C77)
  did not fit its part to the type expected, so a union's part went where
  an `int` was wanted (`unions/paths-untested.fx`, `paths-in-closure.fx`,
  `paths-region-variable.fx`, which FX-26 refuses, had been kept from kfx
  only by their type's name, `cell`). Fitted now; `refused-fixpt-programs`
  also reads Fixpt's `! ` lines.

Measured: 136 of Fixpt's programs through kfx, all agreeing; the engine
127.

## C88. `load-module` and modules in modules (2026-10-12)

Stage 4 (`docs/research/modules.md`).
- **A module in a module** is a nested Rust `mod`: checked as a module,
  its names the outer's items (`n x`, given on as `m n x`); a `(with n x)`
  among the outer's items names `n`'s `x`, not an item of the outer
  (`(define x (with m x))`, FX-26's re-export); a module item is made
  where it is written, so what uses it must come after it (FX-26's
  `reexport-too-soon`), and a `select` of its type may precede values; in
  the engine, a nested module's names are known to a `with` of it.
- **`(load-module "p")`**: the driver reads the file, and those it loads,
  each from the directory of the file that loads it, and each load is
  spliced in as the module `(module (%loaded "p") item ...)` before
  checking; kfx's driver and the engine's alike (`with-loads`, shared in
  check-defs.kfx). The checker reads a loaded module in the standard
  environment alone (no global, type or effect of the program), and
  refuses one whose making does more than read globals, with FX-26's
  message ("a loaded file is one value for all its loads"). A missing file
  and a load cycle are refused.
- **Each load is its own Rust `mod`**: FX-26 makes a path once and shares
  it, but its making is pure and its abstract types are named for the
  variable it is bound to, so two `mod`s of one file are what a program
  can see of FX-26's one value.
- **A Fixpt bug, worked around in the test only**: `fixpt eval` does not
  find a file that a loaded file loads from its own directory, though
  `fixpt check` does (`modules/load-reexport.fx`, "cannot read
  `bump.fx`"). `run-fixpt-programs.py` takes the program's stated value
  (`;; => 4`) for Fixpt's there. Worth reporting to the Fixpt team.
- **Not yet**: a file with `module-parameters`, `load-input`, a module
  made in a procedure (`linked`, `load-parameters`, `answer`).
- **Found on the way**: a K1 gap in KFX's typing of `cons` (its region
  tied to its tail's, where FX-26's is free), reduced to three lines and
  recorded in TODO §25; the compiler's one instance written around.

Measured: 139 of Fixpt's programs through kfx (136 before), all agreeing;
the engine 130; every program Fixpt's checker refuses, kfx refuses.

## C89. A module's types with parameters (2026-10-12)

Stage 5 (`docs/research/modules.md`).
- **The checker**: a module's `define-type` or `define-datatype` may have
  parameters, a family: `t` inside the module, `m t` outside, stand for
  `(m..t p ...)`, and `m..t` for its body (its own types qualified). A
  family is applied by `((select m t) arg ...)` too, and abbreviated by
  `(define-type b (select m t))`, which FX-26 reads as the family itself:
  `(b arg ...)` is it applied.
- **The emitter**: a module's type abbreviation with parameters is no Rust
  alias. Expanded where applied, by its name inside the module and by
  `m..t` outside, as an abbreviation over an effect is (C46): one whose
  body is a `Value` (a pair of a union's, C77) would leave a Rust alias's
  parameters unused, which rustc refuses. A family whose body applies a
  module's, and an abbreviation of one, are expanded likewise; an
  `include` imports no family. Datatype families stay Rust's generic
  enums, as at the top level.
- **A module's item may be named by a keyword**, as FX-26's local may: it
  is not called by that name (`include-family`'s `sum`).
- **Left**: a `proj` of an expression, `(proj (with m f) int)` (KFX's
  `proj` names a procedure, C42); the bare region `heap` (KFX reads
  `@heap`); `with` shadowing families applied (`with-shadows-applied`).

Measured: 146 of Fixpt's programs through kfx (139), all agreeing; the
engine 137.

## C90. Modules made in place: defined before the module that uses them (2026-10-12)

`(include (module ...))`, in a `hide` too, and `(extend (module ...) e1)`:
FX-26 makes such a module before the items of the one that uses it, and
it sees none of them, so KFX defines it at the top level just before,
named `made--P` (`P` where it is written; no KFX name has `--`, C27), and
uses it by that name. Done where a program's forms are read
(`with-loads`, `source-forms`), so the checker, the emitter, the
restriction pass and the engine see one program. Its abstract types are
the including module's by alias (C86), as FX-26's are the including
module's own; one hidden with its `include` and mentioned by a type the
module shows is refused, as FX-26 refuses it
(`include-abstract-hidden.fx`).

Measured: 149 of Fixpt's programs through kfx (146), all agreeing; the
engine 140.

## C91. The compiler's first module: `termination`; memos inside modules (2026-10-12)

The plan the user approved: modules in the compiler's own code, measured.
- **Memos inside a module** (incr.kfx): its items each with a memo, for
  checking and for emitting. The checker's are on a chain begun from the
  module's name and its items that are no values; the chain after the
  module goes on with its change, its interface, not its text. The
  emitter writes each item in the module's own context inside its `pub
  mod`. `check-module` is split for it (`module-prep`, `check-item`,
  `module-finish`) and the emitter's `put-module` likewise
  (`module-parts`, `put-module-around`); the engine, which shares the
  checker, runs the same code.
- **Two misses found by measuring**: the chain's start had hashed a `hide`
  whole, and the emitter's context fingerprint each module's whole text,
  so an edit inside a module re-made every item, then every form after
  it. Now the module's types alone, and its items' signatures (names,
  types, which are procedures), are hashed.
- **`terminate.kfx` is the module `termination`**: its helpers hidden, only
  `terminates?` given (`(with termination (terminates? ...))`); `i64s`
  and `index-of`, which the checker borrows, before it; `counting`, which
  the emitter borrowed, in base.kfx now. FX-26 takes no string as a
  module's item, so its comments between definitions are `;` comments
  (they no longer reach the Rust; a docstring inside a procedure still
  does).
- **Two emitter bugs the conversion found**: a datatype inside a `hide`
  was not noted as the module's, so its Rust from the checker's types
  lacked its `&'static`.
- **A bootstrap note**: a compiler fix that the new source needs must
  reach the seed first (a regen with the old source), then the new
  source.

Measured, the compiler on itself, best of three: cold 0.89 s (0.91 before
the module), warm with memos 0.26 (0.25), warm after an edit inside the
module 0.25 (0.26 for the same edit before). `incremental.py` edits inside
the module now too (27 compiles, each as one from scratch).

## C92. The compiler in modules: restrictions, incremental, expressions, emitter, writing (2026-10-12)

Step 3 of the plan: file by file, by what each would hide.
- **`restrictions`** (restrict.kfx: `restrict` given), **`incremental`**
  (incr.kfx: seven names given; its datatypes before it), **`expressions`**
  (check-exprs.kfx: 142 helpers hidden, 11 names given to check-defs.kfx
  and the engine's REPL), **`emitter`** (emit.kfx: 41 given) and
  **`writing`** (emit-tops.kfx: 9 given, `emitter` included and hidden in
  it; `incremental` includes both). A file's datatypes and type
  abbreviations stay before its module: types are what files share.
  Callers write `(with m (f ...))`, a Rust path, `m::f(...)`.
- **What the conversions cost, and what was done about it**, measured on
  the compiler compiling itself (cold the same, ±0.03 s of noise; warm,
  with memos, 0.29 s for the same source with no modules): warm 0.35 s
  at first. Made 0.31 s by
  - each module's included values found once (they had been found four
    times, each a walk of every binding);
  - an item's memo keyed by its text, not its printed form;
  - a module's emitting context fingerprinted from the program's, its
    name, and its items' signatures, as a list (a string joined with
    `words-of`, quadratic, had been the largest single cost);
  - the order a module's items are checked in kept as a memo by its text,
    its positions from the module's first item (else found again each
    compile, every body walked; absolute positions had broken when an
    edit moved the module, which `incremental.py` found);
  - only the interface of a module's change found for the chain (its
    whole change, every fact written out, kept by no memo);
  - the item graph's reachability by trees, found once per item.
  What is left (0.02 s) is each module's scope made each compile.
- **The conversion script** wraps a file's forms in `(define m (module
  (hide ...) given ...))`, its comment strings `;` comments (FX-26 takes
  no string as a module's item).

## C93. A string among a module's items is a comment; the pin moved to Fixpt 8da8ecf (2026-10-13)

The user asked Fixpt's maintainers for it (C92's trade-off: the converted
files' notes had become `;` comments), and Fixpt 8da8ecf has it: in a
`module`, a `hide`, and a `load-module` file's forms, a string binds
nothing and is not in the module's type.
- **KFX's**: `module-written`, what the checker and every walk of a
  module's items read, is `module-noted` without its strings
  (`uncommented`, a `hide`'s too); the emitter reads `module-noted`, so a
  string is a `//` comment in the module's Rust, as one between top-level
  forms is. `tests/kfx/module-strings.kfx`; Fixpt's `modules/strings.fx`
  through kfx and the engine. The converted files' strings restored
  (only the comments a conversion had made of strings, matched against
  the strings each conversion commit removed).
- **A memo bug found on the way** (`incremental.py` now has the case): a
  `define-rec` the module's order makes of items that reach one another
  (`rec-of`) was keyed by the text of its first item alone, every member
  given that item's position, so an edit to a later member kept the
  group's old facts (an emitted `match refuse(...) {}` lost). Now each
  member keeps its own position, and the group's key is the text of every
  item it has members in (`starts-of`).
- **What the pin's 28 other commits asked** (Fixpt's TODO §77): a new
  blobref's flat must lie whole in its suffix, checked where one is made:
  kayak-rt's `blobref_at` and `blobref_plus` take the flat's size (a
  `blob-size` fact, at `blobref+` now too) and fail with Fixpt's words;
  a flat array's literal length is 0 to 268435454, else refused (K1).
  And in `--toplevel` (Fixpt's programs run as Rust), each typed global
  value is made where it is defined, `let _ = g();`, as FX-26 makes it, so
  that one that fails fails there: a Rust global is made when first read.

## C94. The REPL's first stage: entries as tasks on the machine, a live transcript (2026-10-13)

The user (2026-10-13): improve the REPL, Fixpt's features on Coopt's
foundation, and an entry's line live while the next prompt is already up
(FrTime's spirit). Design: `docs/research/repl.md`.
- **The foundation is kayak's Coopt-like design, not Coopt's crates**
  (PLAN §6.1 had declined them: their executor's policy is Rust, against
  K7, and they bring `libc`): the REPL is a function of what arrives
  (`live-repl.kfx`, in KFX: keys, entries, which runs, the screen as
  bytes); the Rust only shuttles bytes and time (raw mode by `stty`, as
  Fixpt's; a reader thread; slices for 16 ms; failures caught, since
  KFX catches nothing). So a session is deterministic given its
  arrivals: `kayak-fx26 --keys FILE` runs one in virtual time, a slice a
  millisecond, and its bytes are the tests (Coopt's sim runner).
- **An entry is a task: the machine's registers, parked.** The engine is
  a CEK machine whose continuation is in its heap, so a run pauses when
  its fuel (steps) is spent, its seven registers kept in its slot of the
  collector's roots, and resumes from them; no Rust thread, no future.
  The fuel check cost nothing measurable (`(fib 27)` 0.14 s, 0.16 before).
- **Which run together**: an entry whose effect only reads globals,
  allocates and spins runs beside others; one that writes, defines or
  does I/O waits for all before it, and all after it wait for it; one
  that names `%k` waits for entry `k`. Plain mode (no terminal) runs them
  one after another and writes each line when it ends, as before.
- **Results are `%k`**, not `$k` (KFX's names, which the engine's checker
  keeps, have no `$`): an expression entry's value, the global bound at
  its type when it is checked, stored when it ends.
- **The screen**: a new entry's line is written below its draft; a line
  that changes is rewritten in place (ANSI up, clear, down) if on the
  screen, else written anew above the draft; output an entry writes is
  kept for it (`write-out` captured) and written as `[k]│ ` rows.
- **^C** drops the draft, or, with none, interrupts the newest entry not
  ended; ^D (or the input's end) leaves once none is unfinished.
- **`live-repl.kfx` is a module** (dogfood, C86–C93): its state and
  helpers hidden, the Rust given ten names. Its file is not named
  `session.kfx`: kfx's Rust for a file and a module of one name is
  ambiguous (TODO §27).

## C95. The REPL's editing and commands, as Fixpt's (2026-10-13)

Stages 2 and 3 of `docs/research/repl.md`, in `live-repl.kfx`:
- **Editing** as Fixpt's `lineedit.rs`: a cursor in the draft, which is
  one buffer however many lines; left and right, ^B ^F, Home ^A, End ^E,
  ^K (to the line's end, or its newline), ^U (to its start), ^W (a word
  back), Delete and ^D in a draft, Backspace; up and down move between
  the draft's lines, and at its first or last through the history (^P
  ^N too), the draft put aside meanwhile; ^L clears the screen; Enter
  gives the draft if it is whole, else is a newline where the cursor is.
- **History**: the entries given, in `~/.kayak_fx26_history` (a live
  session's only, as Fixpt's `~/.fixpt_fx26_history`; not a piped or
  scripted one), a line each, newlines escaped, the last 500.
- **Tab** completes the name before the cursor among the checker's as
  far as they agree, and says them if that adds nothing.
- **Commands**, each said as an entry's line (so that a refusal finds it
  as an entry's, and the screen has one way of writing): `,help [NAME]`,
  `,type E` (checked, not run), `,time E` (its steps and collections
  too), `,load FILE` (one entry), `,pending`, `,cancel K`, `,step-limit
  [N|none]` (an entry that takes more fails, its slot dropped),
  `,apropos TEXT`, `,quit` (now; ^D waits for the entries).
- **An entry of several forms** says its definitions' lines, then the
  last expression's value; `%k` is that value.


## C96. The draft checked as it is typed, and run early if it only reads (2026-10-13)

The user: "Implement speculative checking as you type." Fixpt's design
(`crates/fixpt-cli/src/speculate.rs`, and its FX-26 `Oracle`), in
`live-repl.kfx`:
- **Closing off.** A name, number, character or string still being typed
  at the draft's end is left out (and a quote before it); the lists still
  open are closed. What is checked is exactly what has been finished.
- **Checked, nothing kept.** The forms are checked in turn after what the
  entries before them defined (`checked-form`, threading the environment).
  kfx's checker writes nothing (its effect is `kfx`, reading globals), so
  the environment is dropped after; Fixpt had to truncate its interner.
- **Believing a refusal.** Only one about what the user has finished: the
  list it is about was closed by the user, or the draft is finished.
  That needs where the refusal is, which kfx's refusals did not say
  beyond a line: `refuse` now says an offset for a text named `<draft>`
  (`kfx: <draft>@17: …`; other files as before), and a name that is
  not bound is refused at the name, not its list (`at-offset`), so that
  `(+ 1 nosuch ` is said at once. The refusal's message is the only way
  out of a failure, so the offset is in it; the REPL reads it back.
- **The hint**, when nothing is refused that is believed: what the next
  argument of the innermost open list must be, if its operator names a
  subroutine the checker knows (`argument 2 of + wants int`, `f takes 1
  argument`). Fixpt also solves what the arguments so far determine, and
  says what the grammar wants (`next: an arm …`); not yet.
- **Run early** (Fixpt's licence): a finished expression whose effect
  only reads and allocates (and spins) — the effect an entry has that
  runs beside others (C94) — runs as the task in slot 0, by turns with
  the entries, at most 2,000,000 steps; its value is the note (`= 3 :
  nat ! pure`), before Enter. One that may do more says so and is not
  run; one naming an entry not ended waits, and is tried again when an
  entry ends. It does not run while an entry that writes is unfinished.
- **Its IR given back.** The run's nodes, children, arms and constants
  are numbered into the machine's tables like an entry's; the tables'
  lengths before are kept and put back when the draft changes, and an
  entry submitted drops the speculation first, so that the entry's
  nodes never follow a draft's. (Tags and labels interned stay; they
  are bounded by the names there are.)
- **Once a text.** Speculation runs once a round, in the Rust's loop
  before the slices (a refusal caught there, `speculation_refused`), for
  a draft whose closed-off text, trailing spaces aside, has changed: a
  burst of keys (a paste) is checked once, and a space after a finished
  `(fib 32)` does not run it again.
- **The note** is a row under the draft, dim (red for an error), its first
  line cut to the screen's width. Not in plain mode, nor for a command's
  line.

Found on the way, and fixed: entries given in the same keys (a paste of
several lines) wrote the second draft before the first entry's line;
the entries' new lines are now written before a draft is left on the
screen.

`tests/repl/speculation.keys`, `paste.keys`.

## C97. Hints from the checker at a hole, and the refused subform underlined (2026-10-13)

The user: "Work on implementing each of: Fixpt's smarter hints, which infer
types from the arguments already typed or say what the syntax expects
next; underlining the bad subform." As Fixpt's `describe_hole` and
`describe_argument` (`fixpt-fx26/src/top.rs`):
- **A hole.** The REPL puts `%hole` where the next item of the innermost
  open list goes, and checks that. The checker, reaching it, refuses
  saying what goes there (`the hole: an expression of type int`): kfx
  refuses by failing, so the description leaves the checker as Fixpt's
  error does, in the refusal. Where: `check` and `check-by-synth` (the
  type expected, solved as far as what came before it says, so `(cons 1 `
  wants `(listof int ?0)`), `synth` (`an expression`), a `tagcase`'s arms
  (the tags no arm has taken, each with its fields' types, or `nothing:
  every tag has an arm`), a `sum`'s tag (the tags and their values'
  types). A call whose last argument is the hole checks the arguments
  before it, against as many parameters, so that the types they fix show
  (`(f a b ` of a three-argument `f`); with every argument, it says so.
  The hole as an argument is said to be in `argument K` (the refusal's
  `in …`), so that the REPL says `argument 2 of + wants int`, and not of
  a `tagcase` arm's body.
- **Only in a draft.** Every one of these is for a datum read from a text
  named `<draft>`, which only the REPL's speculation reads (C96): a file's
  `%hole` means what it always did, and no fact or message of a compile
  changes (K1, and the fixed point).
- **Fillers.** Closing off can leave a form without its later parts, so
  that it is refused before the hole is reached: `(if c %hole)`, `(let ((a
  %hole)))`. As Fixpt, the hole form is tried again with `%fill` after
  the hole: in every list, and (kayak's) in each list around the
  innermost alone, which a `let` binding's value wants (its body filled,
  not its list of bindings). Fillers after the hole are no call's
  arguments. The first try that reaches the hole answers; else the
  narrowest refusal of a list around it says what that list must be.
- **Tries.** Each try is a check that may fail, so the speculation is now
  a sequence of them, the Rust calling `speculate` while it says there is
  another, each refusal caught (`KAYAK_SPEC_DEBUG` writes them).
- **An error the closing off hid.** `(set-car! (cons 1 #t) ` closed off is
  a call of too few arguments, refused before its argument; with the hole
  it is not, and the argument's error, in a finished subform, is said.
- **Underlined.** A refusal believed is about a datum at an offset: in a
  draft, an argument is refused at itself (`in-draft`), not its call, and
  a name not bound at itself (C96). The datum is underlined in the draft
  as drawn (its end found lexically: a list's close, a name's delimiter),
  across its lines if it has several.

## C98. Inference variables said by their binders' names, in the REPL (2026-10-13)

The user's: "let's print the type parameters name". A hint said
`argument 1 of car wants (listof ?1 ?0)`; Fixpt says the binders' names.
- **Noted at the instance.** Where a name's scheme is instantiated (a
  call's head, a name as a value), in a datum read from `<draft>` or a
  REPL entry (`<fx26:k>`), each inference variable made is noted as a
  fact of the file `%meta` at its number, `binder NAME`. No compile's
  file has such a name, so no file's facts change (K1, the fixed point);
  the file `%meta` and the prefix `binder ` are no other fact's.
- **Said in the message.** A refusal with types in it (`mismatch`, a
  literal's, a sum's, every hint at a hole) goes by `refuse-named`, which
  rewrites each `?k` with a noted binder as its name, the text as Fixpt's
  `show_apart`: the first of a name, in the order read, keeps it, the next
  is primed. So `(listof t 'r)`, and `(cons car ` wants `(listof (subr
  (read 'r) ((listof t 'r)) t) 'r')`, `'r'` being `cons`'s. A variable not
  noted (made by the checker for itself) is still `?k`.
- **On the text, not the type.** The rename is of the message, so that one
  priming covers every type in it (a `tagcase`'s arms each show one), and
  `show-ty` stays as it was for everything else.

## C99. Typing never waits on a computation: slices by the clock (2026-10-13)

The user's: "the speculation etc should not be blocking user interactions
… I should be able to keep typing and even hit enter … and the speculative
type-checking and evaluation should not be blocking me". With `(fact
1000)` typed, the REPL stopped answering keys for seconds.
- **Why it blocked.** The keys are read between slices, and a slice was
  20000 units of fuel. Fuel is spent on a node's evaluation alone: a
  return, and a primitive's application, spend none, so the unwinding of
  `(fact n)`, a multiply per return, was one slice however long. And each
  of those multiplies took its bignum apart and put it together limb by
  limb, quadratic, with a division by 2^32 done bit by bit: 355 s and 9.7
  GB for `(fact 3000)`.
- **A deadline.** The machine has one (`vm.kfx`, `deadline`, nanoseconds
  or -1): every 8 of its transitions, of any kind, it reads the clock and
  parks if past it. The live REPL sets it for each slice, 4 ms
  (`slices-timed!`, from the Rust); a scripted session sets none, so that
  its slices are steps alone and it replays the same.
- **The loop.** The round of slices ends as soon as a key has arrived (the
  reader thread counts what it sends), and what was typed is drawn before
  any slice runs. A key is drawn within a slice: 5 ms, measured, while
  `(fact 30000)` runs (`scripts/repl-latency.py`, by the clock, so not in
  `check.sh`); an entry made meanwhile runs beside it, `(+ 1 2)`'s value
  in 25 ms. The step-adaptive slice tried first (halving the fuel after a
  slow slice) is gone: it acted only after the slow slice.
- **Bignums by halves.** `words.kfx` takes an `int` to limbs, and back, by
  halves (`arithmetic-shift`, `bitwise-and`), n log n; `kayak-rt`'s
  division by a power of two is a shift and a mask, and its bit at a time
  division subtracts in place. `(fact 3000)`: 0.4 s, 500 MB.
- **What still waits.** One transition runs to its end: a primitive on a
  huge value (a product of two 100000-digit numbers), a collection of the
  heap, the check of a large draft (on each key, before the screen).
  Every one of the engine's globals is a thread's (`thread_local`), the
  session's and the machine's alike, so the answer to those is the
  machine on a thread of its own, the session sending it entries; in
  TODO. And KFX's `int`s are never freed, so arithmetic on bignums
  leaks what it makes (500 MB for `(fact 3000)`).

## C100. The pin moved to Fixpt fea5c66: processes, `par`, `pselect` (2026-10-13)

The user: concurrency in the language, asked of Fixpt (separate processes
sharing only immutable data; deterministic `par`); Fixpt built it
(`docs/research/processes.md` there: per-task place stacks; `spawn`,
`send`, `receive`, `(addr M R)`, `(inbox M r)`, `nondet`, transmissible
types, I-cells that suspend, `(init R)`; `par` checked for interference;
`pselect`). The pin's other commits: `append` and `string-append` as
variadic standard procedures, typed calls of native closures from
cellular code. Every check passes as it was: KFX has none of the new
forms, and refuses every program of Fixpt's that uses them (K1); none
of kayak's sources names `spawn`, `send`, `receive`, `par`, `pselect`,
`icell-get` or `icell-put!`, which are FX-26's keywords now. Next, in
order (the user's "go"): the forms in KFX's checker, as Fixpt's rules;
processes in the engine and runtime, each an OS thread with a heap of its
own (Fixpt's note: the transmissible rule makes per-process heaps an
implementation's free choice); the REPL's session and machine as two
processes.

## C101. FX-26's processes and `par`, in KFX's checker (2026-10-14)

The user's "go" after C100: Fixpt's forms in KFX, rule for rule
(Fixpt's `docs/fx26.md` "Processes and `par`", `processes.rs`).
- **Types and effects.** `(addr M R)`, `(inbox M r)`, `(icell T R)` are
  `ty-data` `%addr`, `%inbox`, `%icell`. The atoms on a region are now one
  list, `region-kinds`: FX-26's `read write alloc goto comefrom` and the
  processes' `send receive init await`; `nondet` has no region, as
  `spin`, and no masking removes it. `init` is within `write`
  (`effs-within?`). `make-icell`, `icell-get`, `icell-put!` are primitives.
- **Forms, as Fixpt parses them.** A call headed `send`, `receive`,
  `icell-get` or `icell-put!` is the form whatever binds the name (its
  type the binding `%form NAME`, which no program can name); `spawn`,
  `par`, `pselect` likewise. None is a keyword: Fixpt's `KEYWORDS` has
  none of them, and `(define send int 1)` is FX-26 (tried on Fixpt).
- **`spawn`** is checked as a `letreap` of the process's place
  (`letrena-form`, whose facts the emitter's `letreap` reads), the mailbox
  bound. What its body holds from outside is found as it is named: a
  marker `%process N` among the type variables and the bindings, and a
  name found older than the marker noted `held` (for every process it is
  outside of); once the body is checked, each held name's type must be
  transmissible (`untransmissible`, Fixpt's, in its words), as must the
  message type; the body's effect without its place must be what a
  process may do (`crossing-atom?`); `nondet` added if it does I/O.
- **`par`**: each branch's effect apart; none with `goto`/`comefrom`; no
  two interfering by FX's table extended to sends, receives and I/O
  (`interferes?`); two regions may be one unless both are constants of
  other names.
- **`pselect`**: arms `((icell-get c) x body ...)`, `((receive b) x body
  ...)`, typed as `if`'s arms are, and `nondet`.
- **Refused where Fixpt accepts** (K1 allows): a process defined as a
  global's value (KFX makes a constant at its first use, C21, so its
  value may not spawn); a `define-type` that mentions itself (Fixpt's
  `exchange.fx`, `deadlock.fx`); unions, non-list pairs, `vsubr`s as
  messages (C102). Found on the way, not processes': inside a global's
  value, Fixpt does not count a local lambda's reads of globals against
  its declared effect; KFX does (TODO).
- **A bug found on the way, older:** a `proj` of a global that may
  suspend, as a value, was `&f` where its future must be boxed; now noted
  `async-global` as a name is. Fixpt's `higher-kinds/proj-await.fx`,
  refused before (KFX had no `await`), is accepted and agrees.

## C102. Transmissible types as `Send + Sync`; `par` in two modes (2026-10-14)

The user: "emit transmissible types as Send/Sync, we should do that", and
for `par` "a mode switch … I am willing to have unsafe but I'll be
happier if we also have a non-unsafe variant to also experiment with".
So the checker's proof is carried into the Rust's types, and rustc checks
it again, rather than `unsafe` trusting it.
- **Procedures.** One whose latent effect a process may have (`crossing?`:
  sends, fills, awaits, I/O, `spin`, `nondet`, reading globals or frozen
  data, allocating at a constant) is `&'static (dyn Fn(..) -> R + Send +
  Sync)`, made by `lambdaNs`; the checker notes `sync` at such a lambda,
  and the emitter finds the same of a written type (`crossing-effect?`).
  The settled type the checker writes says `pure` for such an effect and
  `%local` for any other that neither aborts nor suspends.
- **What such a lambda holds** must then be `Sync` (`sync-type?`), or rustc
  refuses the Rust: KFX refuses first, in its words (`synced-lambda`). A
  local named in the body but bound in it too counts: KFX refuses more
  than it must, never less. Rust's `Send` is about what a closure holds,
  FX's rule about what it does; the extra rule makes them one.
- **A `letrec`'s knot** is a `OnceLock`, tied once, so that a closure
  holding it may cross.
- **Constants.** One whose type is `Sync` is kept for the whole program,
  `static ONCE: OnceLock<T>` (the checker notes `shared`), so a process
  sees the program's I-cell, address, value; any other is
  `kayak_rt::kept_constant!`: each thread's, which no process can tell,
  holding none.
- **Processes** (`kayak-rt/src/process.rs`): an OS thread each, `spawn`,
  `send` bounded by `Send`; mailboxes and I-cells a `Mutex` each; one
  lock and condition variable for every wait, counting the tasks that can
  run, so the program's wait with none left is Fixpt's deadlock, in
  Fixpt's words; a process's failure the program's, at its next meeting
  with the scheduler. Messages shared, not copied: never written.
- **`par`**: branches that hold only what may cross go to `par2`, threads,
  checked by rustc. Others to `par2_shared`: without the cargo feature
  `unsafe-par`, in turn (the same answer, by the interference check); with
  it, at once, by `kayak-sync`, the one crate with `unsafe` (two
  assertions, each said in a SAFETY comment), and every constant kept for
  the program (`kept_constant!` a `kayak_sync::Global`).
- **Emitted** (2026-10-14): `spawn` is `kayak_rt::process::spawn(move |box:
  Inbox<M>| { let _ = <the body, a letreap of the place>; })`, the body
  checked by `letrena-form` at the `spawn` so that the emitter's `letreap`
  finds its facts; a message's, an I-cell's, a mailbox's type is written
  with no `letrena` around (`in-heap`), so its procedures are `'static`.
  `send`, `receive`, `icell-get`, `icell-put!` are always
  `kayak_rt::process::…`, so that a program's own `send` cannot take the
  call. `pselect` binds each arm's cell or mailbox, `match`es on `sync`,
  and takes from the chosen. Between top-level forms of `--toplevel`, and
  after the last, `quiesce`.
- **`par` in turn needs an order.** Fixpt's `par.fx` has a branch await
  an I-cell another fills: in their written order, run in turn, that
  deadlocks. The checker notes `par-order`, each branch that fills a
  region before each that awaits one that may be it; the emitter nests
  them so. A cycle (each awaiting what the other fills) has no such order:
  run in turn it is a deadlock, said so; `unsafe-par` runs it.
- **Counting tasks.** A thread is counted from when it first meets the
  scheduler (a program's thread: `kfx-corpus` runs each program on a
  test's thread, not `main`'s), or from before it starts (a process, a
  branch, counted by its maker); uncounted when it ends, by a
  thread-local's drop. A woken waiter counts itself only when it gets the
  lock, so a send or a fill starts an epoch, every waiter stale until it
  has looked again: a deadlock is said only when no task runs and none is
  stale (a race found by running the corpus a hundred times).
- **Tests**: `tests/kfx/processes.kfx` (a server and its replies, a worker
  sent procedures, `pselect` with a stop cell, `par` on threads and `par`
  shared with dataflow; 100 runs each way, `unsafe-par` too);
  `tests/kfx-refused/process-*`, `par-interfere`, `pselect-arm`;
  `tests/kfx-stricter/process-constant`, `process-union-message`,
  `lambda-holds-ref`. Fixpt's `par.fx` and `pselect-deadlock.fx` agree.

## C103. The REPL's front and its machine on two threads (2026-10-14)

The user, asked whether the REPL's front end or its machine should become
a process: "Okay do option 1", the front apart from the machine, so that
no step of a run (a primitive on a huge bignum, a collection) holds up a
key, as a 4 ms slice (C99) still could.
- **Threads, not a KFX `spawn`.** The front calls the checker, which
  refuses by failing, and KFX catches nothing (FX-26's conditions, by
  escape tags, are not KFX's yet): only the Rust can catch a refusal, so
  the Rust makes the two threads, as it made the keys' thread, and
  catches on each.
- **Each side a copy of the whole session.** Every constant of no
  transmissible type, every ref among them, is kept for each thread
  (C102), so the front and the machine each have their own: the entries,
  the checker's environment, the machine's heap and tables. Nothing
  written is shared, which rustc checks: what crosses is a `request` or a
  `news` (live-repl.kfx), datatypes of numbers and strings, `Send`.
  kayak-fx26 refuses to build with `unsafe-par`, which would keep one
  copy for the program (`KEPT_FOR_PROGRAM`).
- **Who does what.** The front reads keys, edits, draws, and checks each
  entry (so that the draft is checked after it) but lowers nothing; it
  sends the entry, numbered, before checking it, so the machine numbers
  it the same, refused or not. The machine checks it again, lowers and
  runs it, and sends each entry as it is now (its state, line, output
  since, times, steps, collections) when it changed: `news-taken!`. The
  draft's early run: the front sends its closed-off text and type with a
  serial, the machine runs it as the task in slot 0, and the outcome comes
  back as a note, taken only if the serial is still the draft's. ^C,
  `,cancel` and `,step-limit` are requests; `,type`, `,help`, `,apropos`,
  `,pending` the front's alone (its copy of the entries' states).
- **Scripted sessions in step.** `--keys` runs the two sides by turns, a
  round of the front's requests then 16 slices on the virtual clock, so
  that it replays the same: the eight sessions are byte for byte as
  before.
- **Costs.** Each entry is checked twice, once on each side; the prelude
  loaded twice. The check of a large draft still runs on the front's
  thread, before the next key.

## C104. The pin moved to Fixpt e0418bd (2026-10-14)

The user: "you should update to latest Fixpt". One commit past fea5c66,
on native code's typed calls (TODO §93), nothing KFX has: check.sh
passes as before (156 of Fixpt's programs through kfx agree, 287 refused
by both).

## C105. Conditions, first: FX-26's escape tags, and `spin` for a call of a value (2026-10-14)

The user: "Let's add conditions to KFX then", so that the REPL's front
could become a KFX process (C103). FX-26's conditions are a library,
`fx26:conditions.fx` (Fixpt's `docs/research/conditions.md` §6b): `handle`
over an escape-only tag, `with-handler` and `raise` over continuation
marks, each procedure taking a region-polymorphic body (rank 2). KFX
lacked all four parts; this is the first two.
- **`spin` for a call of a computed procedure** only where FX-26 says it:
  the callee no `lambda` and its type cyclic (`self-applicable?`, Fixpt's
  `cyclic`, infer.rs), as for a call of a local (C81). KFX said it for
  every such call, which refused the library's `raise` (a call of the
  handler `first-mark` gives).
- **Escape tags**: `(escape-tag A H R)`, a type of its own (`%escape-tag`),
  never a `prompt-tag` and never the other way, as FX-26's is a generative
  type; `make-escape-tag` a primitive; `(escape t v)` typed as an abort is,
  a `goto` on `R`, of type `void`; a `prompt` of one bounds nothing its
  body does, and masks the `goto` as for a prompt tag. In Rust a prompt
  tag's `Tag<A, H>`, and an escape is an abort's early return. No capture
  takes one: KFX's only capture, `abort-with-continuation`, wants a
  `prompt-tag`. Not transmissible (a tag in `R`).
- **`proj` of a control primitive as a call's head**,
  `((proj (proj escape r) a h) t v)`, as FX-26's programs write it: the
  checker types it by the primitive's FX-26 scheme and notes `control` at
  the `proj` (so that a program's own `escape`, as Fixpt's
  `run/region-escapes.fx` defines, is its own), and the emitter writes the
  primitive's call; for `abort-current-continuation`, `escape` and
  `make-continuation-prompt-tag`. The restriction pass still refuses one
  anywhere else.
- Measured: kfx accepts 160 of Fixpt's programs (156), all agreeing, among
  them `control/escape-tag.fx` and the conditions examples
  `escape-polymorphic.fx`, `escape-through-generic.fx`; all 287 Fixpt
  refuses, kfx refuses. The engine agrees on 148 (145).

## C106. Continuation marks in KFX's Rust (2026-10-14)

The second part of conditions (C105): `make-continuation-mark-key`,
`with-mark`, `first-mark`, which the checker had (C72) and the
restriction pass refused for Rust.
- **The marks a stack, the thread's** (`kayak_rt::marks`): `with-mark`
  pushes its mark for its body's run and takes it off after, by a drop,
  however the body ends (a value, an abort's early return, a failure the
  REPL catches). A key an identity, its value's type erased in the stack
  and checked by a downcast.
- **A body that may suspend** (C45, C46) is a future: `with_mark_async`
  pushes the mark for each poll of it, on the poller's marks, so a
  continuation resumed elsewhere sees its own marks on the resumer's, as
  FX-26 composes them. The emitter chooses it by the call's
  `suspending-call` fact. This is what C72 left: KFX's effect-polymorphic
  code is async, and the library's `with-handler` is such code.
- **A procedure in a mark has one Rust form**, a future's, whatever its
  effect: the key's type is written as generic code writes `(subr e ...)`;
  where the key's procedure type cannot suspend, `with-mark`'s value and
  `first-mark`'s default are lifted to it (`lift-suspend N`), and
  `first-mark`'s value is run to its end where it is called (`mark-unlift
  N`). Else a handler made where its effect is known (`pure`, `(goto r)`)
  and called by `raise`, generic in it, would be two Rust types under one
  key. Only the key's procedure itself: a procedure deeper in a mark's
  value (in a list) is not lifted, and rustc refuses the Rust.
- **Processes and `par`**: a key's region is written, so reading one is
  not a crossing effect, and no branch that reads marks runs on a thread
  but with `unsafe-par`, whose branch is given its maker's marks.
- **Not yet**: `current-marks`, `marks-of`. FX-26's `with-mark` in tail
  position replaces its key's mark on the frame below; KFX's pushes. No
  `first-mark` can tell them apart; a list of the marks could
  (`tests/kfx-stricter/current-marks.kfx`). A loop that marks each time
  round grows the Rust stack, where FX-26's runs in constant space.

## C107. Rank 2 in regions, and Fixpt's library built in (2026-10-14)

The rest of conditions (C105): `fx26:conditions.fx` loads, checks and
runs, in kfx and in the engine, Fixpt's own text.
- **A type polymorphic in regions alone**, `(poly ((r region) ...) T)`,
  as a parameter's type (`%poly`, its binders named apart by where the
  type is written, `'r@P`); a `proj` of a local of one instantiates it;
  a `plambda` where one is expected is checked against its body, its
  regions rigid, in scope, none named as one in scope already, none
  escaping into a type found outside it (KFX binds a local at one type
  where FX-26 generalizes a `let`'s `nil`, so it refuses more:
  `tests/kfx-stricter/poly-type-binder-escapes.kfx`). Two such types fit
  when their bodies do, the one's binders the other's. In Rust the regions
  are erased: a poly type is its body's, a `plambda` its lambda, a `proj`
  of a local the local. A binder of a type or an effect is refused.
- **A `poly` in a `poly`, a `plambda` in a `plambda`, around a
  definition**, as the library writes `raise`: one of each, flattened as
  the forms are read (`flat-definition`, with `load-module`'s splicing),
  as C70 already took the type.
- **A prompt's handler any procedure**, as FX-26 has it, not only a
  `lambda`: its type's latent effect the prompt's, noted `fallible` or
  `suspends` for the emitter's call.
- **`(define x (with m y))`, `y` a module's poly procedure**: `x` at `y`'s
  scheme, and in Rust `pub use m::y as x`, not a constant.
- **Built in**: a `load-module` of `fx26:NAME` reads no file; its text is
  KFX's copy of Fixpt's, `kfx/built-in.kfx`, written by
  `scripts/built-in.py` from the clone at the pin and checked by
  `check.sh`. Only `fx26:conditions.fx` so far.
- **The engine**: an escape-only tag a prompt tag at run time, `escape`
  an abort (`prim-alias`), as Fixpt runs them; its marks and the rest were
  there (C72).
- Measured: Fixpt's `run/conditions.fx`, and its examples `handlers.fx`,
  `library.fx`, `unhandled.fx`, give Fixpt's answers in kfx's Rust; kfx
  accepts 163 of Fixpt's programs (156 before C105), all agreeing, and
  refuses all 287 Fixpt refuses; the engine agrees on 152 (145).
  `tests/kfx/conditions.kfx`: the library's raise, handle and restart, an
  escape tag and a mark by hand.

## C108. Monitors, `self` and deadlines; the pin at Fixpt 0a0e277 (2026-10-14)

Fixpt built process monitors at kayak's request (its `TODO.md` §94),
with `race`, `self`, the `sendable` kind, `pselect`'s deadline arm and
preemption. The pin moves there; KFX takes what the REPL needs:
- **`(spawn-monitored w on-failure (box M r) body ...)`**: `w` found, an
  `(addr W R)`; `on-failure` found, not checked, and fitted to `(subr
  pure (string) W)`, as Fixpt's `synth_spawn_monitored` does (a lambda's
  parameter must be written); the body `spawn`'s; the effect `spawn`'s,
  the two's, and `(send R)`. In Rust, `process::spawn_monitored(w, f,
  body)`.
- **Groups in kayak-rt**: a process and its `par` branches are one; the
  first failure in any is the group's, and cancels the rest at their next
  wait (a `Cancelled` unwinding, no failure of its own), so a branch left
  waiting does not hold the message back. An unmonitored group's failure
  is the program's at once (before, only when the process's thread
  ended); a monitored one's, `on-failure` of its message sent to the
  watcher, once. `error`'s string comes through as given. In a monitored
  process a failure of kayak-rt's (`error`, a division by zero) unwinds
  past the panic hook, which would print it: Fixpt prints nothing. A
  Rust panic of another kind (a stack overflow aborts) is not caught so.
- **`(self box)`**: the mailbox's address, `(addr M r)`, at the place;
  in Rust the mailbox itself (`own_address`).
- **`pselect`'s `(sleep-until t)` arm**, its value unit: a `Deadline`
  waitable; a waiter with one waits with a timeout, and while any does, no
  wait is a deadlock. The clock is now one for every thread (it was each
  thread's, so a deadline made by one process meant another time to
  the next).
- **`race`** is refused as a form wherever it heads a list, not taken for
  a call of what a program names `race`, which FX-26 parses as the form
  (K1: KFX accepted `(race 1 2)` of a procedure so named, which FX-26
  refuses as `nondet`). The same for `spawn-monitored` and `self`.
  `race` itself waits (`tests/kfx-stricter/race.kfx`), as does
  `sendable`, which KFX's `poly` refuses as a kind.
- Not taken: preemption, which kayak-rt's threads have of the OS.
- `tests/kfx/monitors.kfx` (Fixpt's `programs/monitors` in KFX: workers
  that fail by `error` and by a trap, a `par` failing once and twice, one
  message, a deadline, `self`); eight refused programs. kfx accepts 164 of
  Fixpt's programs, all agreeing, and refuses all 294 Fixpt refuses.

## C109. Bytes from the standard input, a key in `pselect`, terminal modes; the pin at Fixpt 09acf16 (2026-10-14)

Fixpt built these at kayak's request (its `TODO.md` §95), so that a
front end that edits a line as it is typed is an FX-26 program:
- **The primitives**, FX-26's types: `read-byte`, `read-bytes` (into a
  bytevector's span, `(write r)`), `stdin-ready?`, all `(io stdin)`;
  `terminal-raw`, `terminal-restore`, `terminal-size`, of a new kind,
  `(io terminal)`, each `not-a-terminal` when the input is none. Their
  sums and the size's product are kayak-rt's standard shapes.
- **`pselect`'s `((read-byte) b body ...)` arm**: ready once a read would
  not wait, its body given what `read-byte` gives. A waiter on it, like
  one on a deadline, keeps a wait of the program's from being a deadlock.
- **One stream in kayak-rt**: a thread of its own reads the standard
  input into a buffer, from which `read-line`, `read-byte` and
  `read-bytes` all take, and which tells the scheduler as bytes come.
  Fixpt polls the descriptor (`libc::poll`); safe Rust has no poll, nor a
  read that does not wait, and K4 keeps `unsafe` out of kayak-rt, so the
  thread is how a `pselect` knows a read would not wait.
- **Raw mode by `stty`**, on the standard input, as kayak-fx26's driver
  set it (C103), with Fixpt's flags; the modes before saved by `stty -g`
  and put back by `terminal-restore`, and by `kayak_rt::finish` at the
  run's end, which `kfx-io` calls, failed or not, as `fixpt` restores the
  terminal. `stty size` for the size. No `unsafe`, no new dependency.
- `tests/kfx-io/keys.kfx`: a line, bytes, a span, `stdin-ready?`, a key
  waited for by a process beside its mailbox, the terminal from a pipe;
  Fixpt's answers. On a pseudo-terminal (`script`), raw mode gives each
  key as typed, ^C as 3, and restores. kfx accepts 167 of Fixpt's
  programs, all agreeing, and refuses all 296 Fixpt refuses.

## C110. The engine's failures are values (2026-10-14)

For the REPL in KFX processes (C111): the machine's state is at `@h`,
which no process may write, so the machine is the program's own
computation, and FX-26 catches no failure there. So the engine says a
run's failure rather than failing, as a Rust program returns an `Err`:
- `engine-failed` and `engine-failure` (ir.kfx), set once by
  `failed-with!`, for every `error` the engine had: a program's `error`,
  `car` of nil, an index out of range, what the engine cannot do, its own
  invariants of a run. `run` stops at the top of its next step, giving
  `w-failed`; a primitive's loop that goes on meanwhile works on a dummy
  value, harmless since the run is over. The heap's own invariant
  (`alloc!` past the room it was given) and `load!`'s stay failures:
  going on after either could break the heap later runs use.
- **kayak-rt's failures beneath a primitive**, audited (its `fail` sites
  the engine reaches): division by zero (`i64`, `int`, the fixed widths),
  `arithmetic-shift` by a count past 2^32, `integer->char` of no
  character, `f64->int` of no integer. Each is checked first and said as
  the engine's, in kayak-rt's words. A debug build's backtraces show each
  goes by the engine's failure, none by kayak-rt's.
- **As before**: `kayak-fx26 FILE` fails with the message, raised again
  (`failure-raised!`), so the corpus's and Fixpt's programs run as they
  did; C103's REPL ends the entry (or the draft's run) failed, as the
  Rust's catch did, and numbering's failure is raised for its catch.
- Left failing: lowering's (`lower.kfx`, of effect `kfx`), which C111
  tries first in a worker; `exit`, which ends the REPL, as in Fixpt's;
  a stack overflow.

## C111. The REPL as FX-26 processes (2026-10-14)

The user's ask since C103: the REPL's front a KFX process, not Rust's
threads. With monitors (C108), keys and the terminal (C109) and the
engine's failures as values (C110), it is: `fx26/repl-tasks.kfx`, module
`tasks`, and `fx26/start.kfx`'s `main`.
- **Four kinds of task.** The machine is the program's own computation,
  the engine at `@h` (no process may write a region not its place, and
  the engine's tables, heap and globals are all `@h`'s: moving them into
  a place would rewrite the engine, option 2 of the user's choice). The
  front is a process, its state in its place, its procedures polymorphic
  in that region (`(fstate r)`, `(fentry r)`; refs made by `new` at `r`,
  since a process may hold no procedure taking a place, as Fixpt says
  too). Each check is a worker, `spawn-monitored`, its watcher a relay of
  one message into an I-cell, so a refusal comes back as `refused-done`.
  A relay process is the machine's mailbox: the program's own
  computation has none, so it asks the relay by an I-cell, waiting or not.
- **The front checks; the machine runs.** An entry's worker checks and
  lowers it (C103 checked it on both sides): the machine lowers it again,
  which cannot fail now, and numbers it, a failure of which (what the
  engine cannot number) refuses it there. An entry refused leaves nothing:
  C103 kept the definitions before the refused form in the checker's
  environment, never run.
- **The front never fails by the checker**: what reads or checks (the
  hole's list too, `hole-items`) is a worker's job. A failure of the
  front itself, a bug, is the relay's message (`front-failed`), which the
  machine says, restoring the terminal, and ends.
- **Scripted sessions** run front and machine in step, a round at a time,
  the round's end a request carrying where to put what it ran: the eight
  sessions give C103's bytes, the clock read at each look for a task, the
  last too, as C103's did.
- **The host**: kayak-fx26's Rust only gives the history file's path, as
  arguments (`kayak_rt::set_args`: FX-26 reads no environment), and puts
  the terminal back at the end (`kayak_rt::finish`). A new pty's size is
  0 by 0: 24 by 80 then, as C103 had it.
- **C103 kept**, behind `rust-repl` (`crates/kayak-fx26/src/rust_repl.rs`,
  `fx26/live-repl.kfx`): `repl-sessions.py --rust-repl` and
  `repl-latency.py --rust-repl` run it, built in `target/rust-repl/`;
  `check.sh` runs its sessions too.
- Measured, C111 against C103: the eight sessions the same bytes. Ten
  runs each, alternated (the single runs first reported, 0.76 s against
  0.90 s and 25 against 18 ms, were noise): the sessions in 679 ms
  against 605 ms (each run within 8 ms of its median), C111 some 7 ms
  slower in each; a plain `repl` with no input 17 ms against 10 ms
  (batch mode, the same engine, 9.7 against 9.0 ms), and some 55 µs more
  for each entry (200 entries: 34 against 16 ms), its two threads made
  and messages sent. The constant's cause not found: not the constants
  kept for each thread (built with `kayak-rt/unsafe-par`, the same). On a
  pty while `(fact 30000)` runs, the same within noise: each key drawn
  in 0 ms, `(+ 1 2)`'s value in 20 to 23 ms both, the draft `(fact
  1000)`'s early value in 73 to 76 ms (C103 twice 93 and 97). Failing entries
  (`car` of nil, `error`, a division by zero, a name not yet defined, a
  refusal) are said the same, and the REPL goes on.
- KFX gaps met: a procedure polymorphic in a place alone, whose place is
  only its mailbox's, leaves a Rust type parameter unconstrained (the
  relay's loop is in its `spawn` instead); constructor tags are global,
  so this file's are named apart from live-repl.kfx's.

## C112. A quick check (2026-10-14, TODO §28, the user's order)

`scripts/check.sh --quick`, 49 s warm against about 5 minutes: every
step but Fixpt's runs of the compiler (`scripts/fixpoint.sh --quick`:
K2 and the replay as Rust, no stage 0) and with fewer incremental edits
(`scripts/incremental.py --quick`: one file's first three edits and its
last, and the four across files; 9 compiles, not 31). Fixpt's programs
run both ways and the refused ones are kept: 13 s, and K1's widest net.
- Else possible: stage 0 in the background, its answer late; or no
  incremental run at all. The first leaves a check that has not ended
  when it says it has; the second, the memos unchecked for days.
- The full check stays the one before a commit that touches the compiler
  or the language: stage 0 is what shows Fixpt accepts the compiler's own
  source, and gives the same Rust.

## C113. A failure's backtrace on the engine (2026-10-14, TODO §29)

A run that fails on the engine says, after its message, the procedures
its continuation holds, innermost first: `in inner (bt.fx:2), 4 calls`,
then `in middle (bt.fx:4)`, and so on, at most 16 lines and a count of
the rest. In the REPL each line follows `! evaluation failed:` after
`!   `; `kayak-fx26 --run` writes `kayak-fx26: MESSAGE` and the lines
to stderr and ends with status 70, as Fixpt's run does (it ended with a
Rust panic, status 1, before). Fixpt has no backtrace on any machine: a
kayak addition.
- **What a line is.** A lambda in the IR keeps its name (`define`,
  `define-rec`, `let`, `letrec`, a module's item) or none, and where it is
  written (`lower.kfx`). The table (`table.kfx`) numbers a lambda's body
  just before it, so a lambda's nodes are a range ending at its own; each
  node's lambda is kept (`owners`), and each lambda's words (`wheres`),
  at loading.
- **Activations, from frames.** The machine's frames are an expression's
  rest (an `if`'s arms, a call's arguments), not calls. A frame's node
  says its lambda; its environment, which activation: one activation's
  frames have environments each an ancestor of the one before (a `let`
  inside adds one), where two calls of one procedure have siblings.
  Consecutive activations of one procedure are one line, counted.
- **Where it starts.** A primitive's failure is found after it returns,
  when the call's node is gone, so the call's node rides in the `node`
  register of an application (mode 2, where it was unused), and the
  primitive's caller checks for a failure before going on. The walk is a
  loop, keeping the first 17 lines, so a deep recursion's failure is
  said without a deep Rust stack. The machine's step is as fast as before
  (fib 1019 ms against 1039).
- **What it cannot say.** A proper tail call leaves no frame, so its
  caller is not there; Fixpt's machines would have the same gap. Frames
  outside every lambda (a top-level form's) say nothing.
- Else possible: a "procedure" frame pushed at each call, as a stack
  machine has: exact, but a frame a call, and tail calls no longer free;
  or a shadow stack only while a flag is on, which a failure on the first
  run would not have.

## C114. The checkers compared, form by form (2026-10-14, TODO §30)

`kayak-fx26 --check FILE` writes what `fixpt check` writes, and
`scripts/compare-checkers.py` compares the two over all 653 of Fixpt's
programs (`docs/research/fx26-checker.md`). In `scripts/check.sh`, it
fails only on a K1 breach (kayak accepting what Fixpt refuses); the rest
is §30's measure. That day: of 357 programs Fixpt accepts, 22 the same,
52 alike (effects sorted), 110 different, 173 refused by kayak; of 296
it refuses, kayak refuses all.
- **The checker for FX-26 is grown in place**, the question
  `front-end.md` left open: K2 and the incremental compile hold at every
  step, and the comparison measures each. Fixpt's checker in FX-26 is
  read for each feature's rules.
- `--check`'s lines are made after the checker, outside it: a
  definition's type as its scheme has it, inside the `poly` binders
  written (schemes keep binders' names only); its effect, its value
  checked against its type, as `check-top` checks a constant's.
- Else possible: compare only verdicts (accept or refuse). The types and
  effects are what a reference says, and where most of the differences
  are.

## C115. Effects written in Fixpt's order (2026-10-14, TODO §30 step 1)

`show-effs` sorts an effect's atoms, and the names in `(read (globals
...))`, as strings, as Fixpt writes them: `(maxeff (read (globals fib))
(read @globals) spin)`. Everywhere an effect is written: the REPL, kfx's
messages, `--check`. The comparison (C114): the same lines for 74 of the
357 programs Fixpt accepts, from 22; no program is now merely alike.

## C116. Code a program makes and runs, in KFX (2026-10-15, TODO §31; Fixpt's TODO §97)

FX-26 now places and runs machine code (Fixpt `bc3890e`, our request):
`seal-code`, `native-procedure`, `native-code?`, `(io native)`, and the
native convention declared (`define-convention`, Fixpt's
`native-arm64.conv`). KFX has the first three and the effect, and
`(subr (conv native) E (T ...) R)`; `crates/kayak-jit` places and runs the
code; `tests/kfx/made-code.kfx` (Fixpt's `leaves.fx`'s adder) answers
`native 42` as Rust and on Fixpt.
- **Where made code meets kayak's values.** On Fixpt the program and its
  code share one heap; kayak's values are Rust's. So what crosses is what
  both lay out alike: integers that are fixnums, booleans, unit,
  characters (kayak-rt's `Native`), and `seal-code` takes a bloblet of no
  fields. Stricter than FX-26, so K1 holds; widened as the layouts are
  made to agree (a bytevector with Fixpt's header before its bytes next,
  for a heap native code reaches through a field).
- **The direction this sets** for the native compiler's runtime: an FX-26
  program kayak compiles runs in a heap of Fixpt's layout that KFX code
  owns as data, collected by KFX, entered by native procedures, its
  routines (`foreign`, `cons`, `poll`, ...) KFX procedures reached through
  fields; the same KFX and the same machine code running on Fixpt and on
  kayak, compared as `made-code.kfx` is. Not kayak-rt standing in for KFX.
- **The convention**: kayak-jit's state block at the declared offsets;
  the pinned registers set by a trampoline that keeps every register Rust
  keeps. The routines are not kayak's yet: each entry is a stub back to
  the trampoline, so made code calling one fails, naming it, rather than
  jumping to nothing. `define-convention` not yet.
- **The code area**: as Fixpt's, mapped twice (`mach_vm_remap`), sealed
  copies laid out as Fixpt's (header, fields, trailer, code); never freed,
  as nothing of kayak's heap is. `unsafe` only in kayak-jit (Q7); kayak-rt
  stays safe.
- **The type**: `(subr (conv native) ...)` is a wrapper in the checker
  (`%native`, as `%vsubr` is), so that no lambda is one and none passes
  for a procedure of KFX's own; past the checker it is the procedure's
  Rust type (`without-conv`). `native-procedure` only under a `the` of
  such a type, of at most three parameters.
- Else possible: copy kayak's values into Fixpt's layout at each call
  (eq? would not hold, and the cost is per call); or let kayak-rt be the
  native runtime in Rust (the substitution the user ruled out, decision
  5 of `native.md`).
- **Then, the same day: code that is not a leaf** (Fixpt's `frames.fx`
  and `declared.fx`, as they are, in the corpus, `made-frames`,
  `made-declared`). kayak-jit's routines: `poll` gives the next slice;
  `overflow` and `trap` end the call in a failure (kayak's native stack is
  the thread's, so past its limit is an overflow, not a cache to flush);
  `cons` makes a pair of Fixpt's layout in memory of kayak-jit's, never
  freed; `foreign` calls a KFX procedure given to native code, a closure
  of Fixpt's layout over a Rust one, its panic carried past the native
  frames (the trampoline's way back) and resumed beyond; trampolines nest.
  `bloblet` and `closure` end the call, naming themselves.
- **What crosses, widened**: lists of what crosses, copied, which no KFX
  program can tell (KFX has no `eq?`, no `set-car!`); procedures of the
  fx convention given to native code. `(subr (conv fx) ...)` is a wrapper
  too (`%fx`): KFX's procedures and native ones fit it, it fits nothing
  but itself, as Fixpt has `fx` the top.
- **`define-convention`** accepts the native convention's clauses in any
  order, each printed as Fixpt prints it, and binds its name to `native`;
  `native-convention` gives them. The text is Fixpt's at the pin
  (`scripts/built-in.py` copies it into `kfx/built-in.kfx` and
  `crates/kayak-jit/src/native-arm64.conv`), and kayak-jit's constants are
  tested against it, so a change to Fixpt's convention shows when the pin
  moves.
- **Fixpt's encoder, as it is** (`tests/kfx/made-encoded.kfx`): `(define
  x (load-input "fx26:p"))` of a file built in, its one expression a
  module or a `let` of loaded modules around one. The module is a loaded
  one (the standard environment alone); the `let`'s modules are defined
  before it, `made--input--P--n`, and the `%loaded` mark names them, so
  that it sees them and nothing else of the program. A file of the
  program's, not yet. Besides: `#x` literals (to an i64); a label named as
  a function of kayak-rt's (`cons`), which a Rust field is apart from;
  a procedure's type written as an abbreviation of an included module's
  is written out by the checker for the emitter (`def-type`).
- **Fixpt's `generated.fx`, as it is** (`made-generated`): `frames.fx`'s
  code made from a convention read as data (`fx26:convention.fx`, reading
  kayak's own `native-convention`) by Fixpt's encoder. On the way: a
  hidden `include` of a named module leaves its types that module's, as
  Fixpt has it (only a module made in place gives the includer its
  abstract types, `include-abstract-hidden.fx`); `string-append` of more
  than two; and in the emitter, an included module's datatypes and their
  tags are a module's own, its tags first, since KFX's tags are found by
  name and another load of the same file (the program's `cvt`) has the
  same ones. Loads are not merged: Fixpt keeps each binding's abstract
  types apart, which one Rust `mod` would not.
- **Fixpt's `allocated.fx` and `linked.fx`** (`made-allocated`,
  `made-linked`). kayak-jit reserves its code area and its objects' memory
  as one space, the code area its first part, so that one card table
  covers both (`state.cards`, biased as the convention says); the
  `bloblet` routine makes a plain bloblet there, or refuses with Fixpt's
  message. Strings cross (copied: FX-26's are never written), and bloblets
  of what crosses, copied each way; a copy native code keeps does not see
  KFX's later writes, nor KFX native code's (kayak-rt's `Native` says so).
  A bloblet's field may hold a procedure whose latent effect reads no
  region, or says `spin` (no knot through the store can come of it); a
  sealed copy keeps its fields beside its code, a field of a procedure a
  closure native code calls by `foreign`, and `bloblet-set!` on a sealed
  copy writes the field there too (`bloblet_written`, a branch for every
  other bloblet). `append` of more than two, as `string-append`.
  `allocated.fx` is in the corpus with `(read @heap)` added to
  `pack-code`'s type: Fixpt masks reading lists a procedure made itself,
  KFX's checker not yet (TODO §30).

## C117. Loads shared, and files of `module-parameters` (2026-10-15, TODO §31 step 1)

FX-26 makes a path once, one value for all its loads, and its datatypes
are structural (`mu` types), so every load of a file gives the same type;
KFX's datatypes are Rust enums, one for each module. So a path, with the
regions it is given, is now one module, `made--load--K`, defined at the
top level before the form that first loads it, and each load is a module
that includes it (`(module (include made--load--K))`): one value, one
Rust enum for each datatype. A loaded module sees the standard
environment and these (their names no program writes).
- **`((proj (load-module "f") r ...))`** of a file whose first form is
  `(module-parameters ((name region) ...))`: the file's items, each
  parameter the region given, shared for the same regions. Regions alone.
- **Not quite Fixpt's**: a file of `module-parameters` is a `plambda`
  around a `lambda` in FX-26, so each call makes the module again, its
  state new; shared, two calls with the same regions share its state.
  And two bindings of one path keep their abstract types apart in FX-26,
  where here they are the shared module's, so KFX might accept a program
  mixing them that FX-26 refuses; none of Fixpt's programs does (the
  check runs all 298 it refuses). Both to be made exact when met.
- **Measured** (`scripts/survey-front-end.py`): the 41 files waiting on
  module parameters now pass their loads; 68 stop at the eager reader's
  state type, a `dletrec` of a composable continuation, which KFX refuses
  (C45: FX-26 may resume one again and again, a Rust future once).
- Else possible: loads kept apart and datatypes compared structurally,
  which Rust's nominal enums cannot.

## C118. Fixpt's reader types apart; no recursive abbreviations; a local load's types by `select` (2026-10-16, TODO §31 step 1)

Our requests to Fixpt of 2026-10-15/16 (its §98), done there, pinned at
`7641962`: A, the eager reader's continuation-holding states in a file
only the reader loads, and the parser loading no reader (`syn.fx`); B,
`call-with-oneshot-continuation` and `(oneshot T A D R)`; C, the reader's
working types (`cursor` and its kin) apart too.
- **No recursive type abbreviations** (talked over with the user): FX-26's
  are structural `mu` types; KFX keeps recursion to datatypes. Taking
  them would mean coinductive equality and subtyping in the checker, and
  in Rust a nominal wrapper folded and unfolded where the program writes
  nothing (K5). In Fixpt's front end only `cursor` (the reader's own) and
  `val` (the evaluator's, a union) were such; so request C, not KFX.
- **One-shot, not yet taken.** Fixpt's capture runs its procedure in
  place and counts only calls of `k`, so a normal return runs the rest
  again (`run/oneshot-once.fx` answers 43): two runs, which a Rust future
  cannot. KFX can take the shape Fixpt's own code writes (`processes.fx`):
  a procedure that, on every path, ends in an abort to the same tag,
  which is C45's `abort-with-continuation`. To do.
- **`(select m t)` of a load bound in a module** (`(hide (define m
  ((proj (load-module "f") r ...))))`, as `parser-types.fx` names the
  reader's `syn`): the load is a module including the shared
  `made--load--K` (C117), so in the items around it `(select m x)` is
  `(select made--load--K x)`, the same module, one defined at the top
  level, whose types a `select` names (`shared-names`,
  `selects-renamed`). Test: `tests/kfx/loaded-select.kfx`, the corpus's
  first program with files of its own (`tests/kfx/loads/`), which
  `scripts/corpus.py` now copies beside the text Fixpt runs.
- **Measured** (`scripts/survey-front-end.py`, 150 files): 11 accepted;
  59 stop at `check-types-types.fx`'s signature, a `moduleof` whose `val`
  is polymorphic in a type (C107 takes regions alone); 8 at `import`;
  about 60 at a `load-input` of a file that is one `lambda`; 4 at a
  bloblet field holding a procedure; 2 at a nested load's path joined to
  an absolute one (a kfx bug).

## C119. Module types' polymorphic and aborting values, knots in arrays and bloblets, `import` (2026-10-16, TODO §31 step 1)

Toward Fixpt's front end through KFX, its checker's and tables'
signatures (`check-types-sig`, `tables-sig`) first.
- **A `val` polymorphic** (`(val k-length (poly ((t type)) ...))`): its
  scheme, and in Rust a generic function of the trait, `fn
  first_or<T: Copy + 'static>(..)`, regions and effects erased as a
  top-level procedure's; each impl calls its module's. A module fits when
  its value is at least as general: the type's variables stay rigid, the
  module's instantiated. A `poly` in a `poly` there is one, as for a
  definition (C70). Tests: `module-poly-val`,
  `kfx-refused/module-poly-val-narrower`.
- **A `val` that may abort** returns a `Result`, as a top-level procedure
  does; one that may suspend is still refused (a trait's `async fn`). To
  fit a module's value whose effect's region only its parameters solve,
  a procedure's fit tries its parameters first when its effects alone
  fail (only then: what fitted before fits as before). Test:
  `module-aborting-val`.
- **Knots through the store, as Fixpt's `knot_in`**: arrays' elements,
  I-cells' values and bloblets' fields are kept at their region. Arrays
  went unchecked before: KFX accepted an array of procedures reading the
  array's region, which FX-26 refuses (a K1 breach, now
  `kfx-refused/knot-array`). A bloblet's field may now be a type
  variable, or any procedure that does not read the bloblet's region
  unsaid, as in FX-26: the knot found in the instance (C21's check of
  each call's), not refused at the type (`bloblet-generic`, Fixpt's
  tables' shape; `knot-bloblet`, `knot-bloblet-instance` refused).
- **`(import m x ... (type t ...) (effect e ...))`**, FX-26's (2026-10-15),
  expanded as it is read, as FX-26 does, at the top level and in every
  `module` and `hide`, of the program and of each file loaded; refused
  where FX-26 refuses it (its module not a variable, a name not a name).
  The module order check now knows a `(with m x)` of a module outside
  gives `x` from `m` (the expansion's `(define x (with m x))`). Fixpt's
  `modules/import.fx` agrees; `imports`, `kfx-refused/import-not-*`.
- **`symbol-name-hash`, `string-hash`**: Fixpt's FNV-1a, to the bit (a
  table's buckets' order). `hashes`.
- **A module's markers** (`m %toplambda`) are no names it gives: a module
  loaded twice in one had them "defined twice". The refusal now names
  the name.
- **Then, the same day:** an abbreviation with no parameters of a
  module's family, inside a module (`(define-type table (select m
  table))`, applied as `(table k v r)`), followed to the family through
  the module's own name for it (`select-behind`), and no Rust item
  re-exported for it (`loaded-family`); an effect a module selects from a
  shared load read with its own, so that those after it may name it
  (`kreads`, then `kmakes`).
- **A stack overflow in the emitter** (`expand-aliases`), on Fixpt's
  `check-holds-types.fx` and `check-modules-types.fx`: a type selected
  under its own name, `(define-type t (select m t))`, in a bloblet
  family's instance; writing `t` out met the select, whose `t` it wrote
  out again, without end. A `select`'s parts are names, not types, and
  are left as they are. Test: `loaded-bloblet-family` (overflows before).
- **`null?` of a call** whose declared result is a union, a pair or
  `nil` (Fixpt's `table-has?`), as of a local of one (`null-of-call`).
  And a top-level family whose body is a union is expanded where applied,
  with no Rust alias: `pub type Entry<K, V> = Value;` left its
  parameters unused, which rustc refuses (E0091).
- **The functor files looked at, not taken on** (the user: "don't go
  crazy"; TODO §31 step 1): 62 files, each applied once in
  `conductor.fx`; expandable statically, but their bodies meet every
  place KFX's checker is more cautious than Fixpt's.
- **Measured**: 54 of 150 files accepted (from 11); 56 after the fix. Next: `wcell`,
  `tword`, `wglobal`, the cellular machine's types that Fixpt's compilers
  in FX-26 use (14), a question for the tiers, not a quick one.

## C120. Fixpt's compilers in FX-26: strategy B (the user, 2026-10-16)

TODO §31 step 1 asked where stage A comes from: A, Fixpt's FX-26 code
through KFX as written; B, ported onto kayak's checker. For the files that
use `wcell`, `tword` and `wglobal` (Fixpt's compilers in FX-26, 16 at
`7641962`), **B** (the user: "Just switch to strategy B for them"). Those
types are Fixpt's cellular machine's, in FX-26's grammar and standard
environment only because Fixpt's compiler needed them; KFX does not take
them (TODO §32: they should leave the language; not raised with Fixpt
yet). The rest of the front end stays A as far as it goes.
`scripts/survey-front-end.py` counts those files apart.

## C121. Strategy B for stage A, and following Fixpt upstream (the user, 2026-10-16)

The user, after C120 and the functor files' look: "strategy B is
inevitable in some cases", then "So now we move ahead via strategy b,
right?" and, of the plan below, "Sounds great. Get started".
- **B for stage A**: kayak's front end (its reader and checker, grown in
  place, TODO §30 at its own pace), Fixpt's `compile*.fx` and
  `regcode*.fx` ported onto kayak's checked forms and facts. Fixpt's
  formats stay kayak's (native.md decision 1). What goes through KFX as
  written is still taken so (`arm64.fx`, `convention.fx`, the layout).
- **Taken whole, at the pin**: `scripts/built-in.py` now also builds in
  `fx26:layout.fx`, `fx26:layout-types.fx`, `fx26:native-layout.fx`
  (Fixpt generates `layout.fx` from its `layout.rs`), and copies
  `layout.fx` into `kayak-jit`, whose hand-written tags, kinds, closure
  field and header bits `tests/layout.rs` compares with it.
- **Ported files say from what**: a line `fixpt-source: PATH@REV` in the
  file's comments; `scripts/fixpt-drift.py` lists the upstream commits
  to PATH since REV, and with `--check` (in `check.sh`) fails while any
  is unhandled. Marked so far: `kayak-rt`'s symbol and string hashes
  (`fixpt-heap/src/heap.rs`), `kayak-jit` (`docs/object-model.md`).
  Handling a drift: carry the change over, or decide against it in a
  C-note, and move the mark to the pin. PLAN.md's pin move says so.
- **Words compared** with `fixpt compile`'s, cell for cell, once stage A
  makes them: a change upstream the marks missed still shows.
- **Not chosen**: a mirror of Fixpt's commits into issues, or a bot; the
  pin moves when chosen, as before.

## C122. Tier 1: Fixpt's cellular machine in KFX, its heap Coopt's collector's (2026-10-16, TODO §31 step 2)

`native.md`'s tier 1, "cells interpreted": the reference meaning of every
tier above. The user: keep going autonomously; and "you may want to also
look at Coopt's collector".
- **Where**: `cells/`, a KFX unit (`cells/cells.unit`), built as the crate
  `kayak-cells` from Rust kfx writes (kept, as the engine's is, and
  checked by `scripts/cells-programs.py`). KFX as Rust: no `unsafe`.
- **The machine is Fixpt's** (`fixpt-engine/src/cellular.rs`, marked
  `fixpt-source:`): a data stack and a return stack of 4-value entries
  (word, next field, frame, closure), registers `cur k fp clo`, the
  routines by Fixpt's numbers (from `fx26:layout.fx`), the operand counts,
  fuel by instruction cells, the stacks' limits, and the traps' messages
  word for word, so an answer or a failure is Fixpt's.
- **The heap is Fixpt's format** (`native.md` decision 1, 2): one flat
  array of `i64`, a value's address its index (`address << 3 | tag`, as
  Fixpt's images have it, relative to 0); pairs of two words; every other
  object a bloblet (header, fields at negative offsets, trailer, suffix);
  words, closures, global cells, strings and symbols laid out as Fixpt
  lays them. Constants from Fixpt's `layout.fx`, built in (C121).
- **Its collector, Coopt's** (`coopt-heap`, ported, `docs/research/
  coopt-heap.md`): Coopt's heap is Fixpt's object model already (its
  `layout.rs` "ported deliberately from Fixpt"), its collector safe Rust
  over words with a root table. First none (a heap that grows; Fixpt's
  answers never depend on collection: `eq` is bits, fields relative),
  then Coopt's Cheney baseline, then its steps (generational, regions,
  summaries, marking, pacing), each against the one before.
- **Words, first, Fixpt's own**: read from `fixpt compile`'s listing
  (`--fx26-machine fx-cells-rsinterp`), which is exact but where a literal
  is cut short (`…`) or two words share a name: those are refused, not
  guessed. So tier 1 is tested against Fixpt's own compiler's words and
  Fixpt's answers before stage A exists; stage A's words, when ported, are
  compared with the same listing, cell for cell. (Fixpt's `dump-heap` is
  Scheme-only, else its images would be the exact form: a request,
  later, if the listing falls short.)
- **Not yet**: bignums (an `int-add` past a fixnum traps, kayak's own
  message, until Fixpt's bignum is ported); prompts, marks and
  continuations (routines 30–38, 41, 51–55); native code; `prim`'s
  numbers are kayak's table's, by Fixpt's names (Fixpt's runtime
  primitives' numbers to be generated with the layout, later).
- **First target**: Fixpt's eight benchmark programs (`tests/programs/
  bench`), the ones its own `rs-cells-rsinterp` is timed on.
- **Built** (the same day): `cells/` (layout, heap, printer, primitives,
  the listing's reader, the machine, the driver, some 1,300 lines of
  KFX), `crates/kayak-cells`, `scripts/cells-programs.py` in `check.sh`.
  Of Fixpt's 674 FX-26 test programs, 175 agree on tier 1 (value or
  failure, word for word), 0 differ; 146 listings not read (primitives
  not yet, words listed twice that differ), 16 need what tier 1 has not
  yet (prompts, marks, bignums), 33 end with a definition, 304 Fixpt
  refuses. Fixpt's eight benchmarks: seven agree (`captures` needs
  prompts), at two to three times the time of Fixpt's own Rust
  interpreter.
- **Two readings of the listing, decided**: a word listed twice, cell for
  cell the same, is one (words are compared by no program); a closure the
  compiler made, a literal (`lit closure of W`), is a closure of no free
  values, one for each word, refused if its word reads one.
- **A primitive's failure** that Fixpt words through its runtime's error
  objects is marked `(C122)` and counted as not yet, until those words
  are ported.
- **Collected** (the same day): Coopt's baseline, its `forward`,
  `forward_pair`, `forward_bloblet` and `cheney` in KFX (`cells/heap.kfx`,
  at Coopt c366857), two spaces swapped (kayak-rt frees nothing, so none
  is made per collection), at the machine's safepoints only (`cons`,
  `closure`, `prim`), as Fixpt's: when three quarters full, or at every
  safepoint under `--gc-stress`, Coopt's stress policy. The roots: the
  stacks, `cur`, `clo`, the tables of what the listing made. Allocation
  now zeroes what it gives. `cells-programs.py` runs each agreeing
  program again under stress: all 175 agree so too. `lists` collects 2,999
  times, copying 3.2 million words, in about the time it took uncollected.
- **Prompts, marks and continuations** (the same day): Fixpt's routines
  `prompt`, `abort`, `callcomp`, `callcc`, `callone`, `resume`,
  `withmark`, `withmark-tail`, `firstmark`, `currentmarks`, `marksof`;
  a continuation a bloblet of Fixpt's fields (its stacks as two vectors),
  given as a closure over `slot 0; free 0; resume`; reinstating moves
  frames and prompts' heights by the depth's difference; one-shot ones
  fail a second resume with Fixpt's message. Regions, which tier 1 keeps
  in the one heap, are no part of a prompt's word yet. Not taken: the
  native-convention routines (51–55) and runs native code called
  (`crossing`). A stale tag held across `callcomp`'s safepoint showed
  only under `--gc-stress`, which is what it is for. Now 190 agree, 0
  differ, all eight benchmarks.

## C123. The pin at Fixpt 1925bca: full literals, and `->` and `->*` in KFX (2026-10-16)

The user asked Fixpt for listings whose literals can be read back
exactly (C122's 35 programs cut short, 2 more unwritable); Fixpt gave
`fixpt compile --full-literals` (1925bca), not `dump-heap` for FX-26.
Between the old pin and it, ten commits of FX-26's own: `(-> (T ...) R)`,
a pure procedure's type written short, and `(->* (T ...) R)`, a
definition's type whose effect its `lambda` says, found in one pass
(the user's, upstream). Fixpt's `arm64.fx` and `convention.fx`, which
kfx builds in (C121), now use `->*`, so the pin moves only with KFX
reading both.

- **Chosen:** both rewritten where `import` is expanded (`imports-in`),
  in no quoted datum: `(-> PS R)` as `(subr pure PS R)`, which it is;
  `(->* PS R)` as `(subr %found PS R)`, a marker only a definition's
  checking takes (`parse-effect` refuses it elsewhere, in Fixpt's words).
  A `define` of one has its effect found (`found-define`, by
  `lambda-effects`, as `define*` finds its globals: a call of itself
  reads itself, and what reads itself is `spin`, Fixpt's
  `arrow_star_spins`), written into its type, and the definition is then
  checked, and emitted, as an ordinary one. A `define-rec`'s members
  (`found-rec`): a first pass with each `->*` member standing for
  `pure`, then each given its own effect and every non-`poly` member's,
  `spin` if that reads a member's global.
- **In modules:** a module's `->*` items are checked as its other
  items are (`check-top`); a module value that names one is refused, as
  Fixpt refuses it (its effect is found only after the module's values
  are made: Fixpt's `arrows/star-module-value.fx`, which kfx first
  accepted). `load-input` files are rewritten too (`convention.fx`).
- **Not chosen:** Fixpt's components, callees first. KFX takes the whole
  group as one component, so a member may be given more effect than
  Fixpt finds it: KFX may refuse a group Fixpt accepts, never the other
  way (K1). Nor `->*` in modules' items, or `letrec`: refused, as the
  marker is anywhere but a top-level `define` or `define-rec`.
- **Why:** the rewrite leaves the checker's and emitter's every other
  path as it was: they see a `subr` whose effect is written, the effect
  the definition's lambda has.
- **Tests:** `tests/kfx/arrows.kfx` (both forms, alone, in a group,
  under `poly`; Fixpt gives the same answer); refused:
  `arrow-star-param` (a parameter's type), `arrow-pure-global` (`->`
  that writes). The built-in files check, and `made-*` run, at the pin.
- **Also with the pin:** `f64-exp`, `-log`, `-sin`, `-cos`, `-tan`,
  `-asin`, `-acos`, `-atan`, `-atan2`, `-expt` in KFX and the engine
  (FX-26's, which tier 1's primitives need; `tests/kfx/floats.kfx`).
  `scripts/gen-vprims.py` is stale (C110's refusals were written into
  `fx26/vprims.kfx` by hand since), so the engine's ten were added by
  hand; in TODO.
- **Drift at the pin:** `disasm.rs` (`cells/listing.kfx`'s source)
  changed only to thread a literal writer through; its default listing,
  the one tier 1 reads, is as it was (194 of Fixpt's programs agree at
  the pin, as before plus Fixpt's new ones). Reading `--full-literals`
  is the next step, in TODO §31.
- **A K1 breach found on the way:** KFX joined branches that mix a pair
  `cons` makes with a list, nothing expected; FX-26 types the pair a
  `(pairof ...)`, which joins no `(listof ...)`, and refuses (stage 0
  caught it in this very change). KFX now refuses such branches in an
  `if`, `cond`, `case`, `tagcase`, `typecase` (`pairs-mixed?`, by the
  branches' forms: `cons`, a `let`/`begin` ending in one, an `if` of
  two): `tests/kfx-refused/branches-cons-list.kfx`. Retyping KFX's
  `cons` as FX-26's `pairof` would be exact, and touch every use of it;
  not now. One place in `cells/` needed its type written.
- **Full literals read** (the same day): `cells/listing.kfx` reads
  `--full-literals` listings (`lit-datum`: Fixpt's `literal.rs`, every
  form; labels across the listing, register code's `const`s read for the
  labels they make; a labelled pair made before what is in it, a cycle
  through a vector or other bloblet refused, not yet). A word listed twice
  is compared with its labels taken out (`unlabelled`). Bignums and
  ratnums are made as Fixpt's (`cells/numbers.kfx`, through KFX's `int`);
  vectors and symbols are laid out in `alloc`'s order (`obj-ref`), as
  Fixpt's; `|#u|` is the symbol `#u`, which FX-26's cellular code uses
  for unit (`%fx26-unit-cell`), not the immediate tier 1 had read. Every
  program not run is now waiting on a primitive.
- **Three gaps in Fixpt's `--full-literals`**, worked around, to be
  raised: (1) where its two compilers make the same words but one shares
  a literal the other makes twice (flonums), its text comparison says
  they disagree and fails (63 programs); `cells-programs.py` then takes
  the FX-26 compiler's listing, if the plain listing says both made the
  same. (2) A cellular `lit` of a closure made at compile time is still
  `closure of W` (the register code's `const` writes it in full): tier 1
  reads it as before (a closure over nothing, refused if its word reads a
  free value). (3) `#bloblet(…)` says no frozen flags: tier 1 freezes sums
  and products, as Fixpt makes them, and nothing else.

## C124. Tier 1's primitives, Fixpt's runtime's (2026-10-16, TODO §31 step 2)

- **Chosen:** `cells/prims.kfx` anew, after Fixpt's `prim.rs` at the pin:
  each primitive's checks in its order and its failures in its words, as
  the cellular machine gives them (an error object's message: `expected
  …`, `division by zero`, `suffix ref: …`). About 300 primitives: lists,
  strings (code points, as Fixpt keeps them), characters, the fixed
  widths (`fixed_op`, through KFX's `int`, wrapped by `Width::wrap`),
  `f64` and `f32` (`f64_op`, `f32_op`), integers past a fixnum (bignums,
  `cells/numbers.kfx`), bitwise operations and shifts, bloblets, suffix
  refs and flat sums, flat arrays, bytevectors and UTF-8 (Rust's
  `from_utf8`'s valid prefix), i-cells' making and reading, `vlambda`,
  regions' makers (one heap, C122). The machine's `int-add`, `int-sub`,
  `int-less`, `int-eq` take bignums, as Fixpt's (`num::int_op`).
- **A primitive's cell** holds tier 1's own code, its operation above its
  arities, so that a call is integer comparisons, families by range;
  Fixpt's numbering waits for the layout's.
- **Not yet, marked `(C122)`:** a large header (fields past 2^18); a
  radix outside 2 to 36 (Fixpt's `to_str_radix` would panic); `modulo`
  of what is no exact integer; `%fx26-convert` to the other convention.
  Not there yet: the scheduler (`%fx26-sched`), the clock, eqtables,
  `%fx26-intern-datum`, I/O, native code (`seal-code` and the rest).
- **Result:** 292 of Fixpt's programs agree on tier 1 (was 194), 0
  differ, each again under `--gc-stress`. `%fx26-native-code?` is false,
  as Fixpt's Rust interpreter of cells says.

## C125. Tier 1's scheduler, Fixpt's runtime's (2026-10-16, TODO §31 step 2)

- **Chosen: Fixpt's, not Coopt's** (the user asked which). Tier 1's
  contract is Fixpt's answers on the words Fixpt's compiler makes, and
  the scheduler of processes, `par`, `race`, `pselect` and monitors is
  FX-26 code, `fx26:processes.fx`, already compiled into those words:
  what tier 1 provides is only what it calls, `%fx26-sched`'s 46
  operations (Fixpt's `sched.rs`), the machine's poll at a slice's end,
  and the machine's catching of a monitored task's failure. Those must be
  Fixpt's to the decision, or the order tasks run in differs. Coopt's
  `coopt-sched` is a host scheduler (Rust futures; priorities; effects
  deciding what may interleave; seeded; a virtual clock of steps) and
  its in-language tasks a different model (an async loop, nurseries),
  with no mailboxes or I-cells: it is the model for kayak's own runtime
  once kayak is the reference (PLAN §9), as Coopt's collector is the
  baseline under tier 1's Fixpt-placed safepoints (C122).
- **`cells/sched.kfx`:** the state a heap vector of Fixpt's 17 slots, the
  collector's root; tasks numbered lowest free first, each with Fixpt's
  quiet count and grace; the run queue, parking, mailboxes (messages
  copied as `copy_within` copies: pairs and the copied kinds, sharing and
  cycles kept, frozen flags kept), I-cells, timers, cancellation, reaps,
  monitored groups and `race` groups, failures kept for the handler. The
  dispatcher, `sched-op!`, in `prims.kfx`, with `%fx26-icell-put!`,
  `%fx26-preempt-done`, `%fx26-clock`, `%fx26-sleep-until`,
  `%fx26-address-identity` (0, as Fixpt's). The machine: Fixpt's fuel
  and reserve, a poll at each slice's end entering Fixpt's trampoline
  word (`call 0; drop; prim %fx26-preempt-done 0; drop; exit`) with
  `preempt`; a trap in a task of a monitored group an abort to the
  scheduler's prompt (`abort-to!`, the `abort` routine's body).
- **The clock is the wall clock**, KFX's `(clock)`, as Fixpt's
  `clock_ns`, so that deadlines come due as Fixpt's do; `cm` gains `(io
  clock)`. Coopt's virtual clock (steps are time) is where kayak's own
  runtime should go, which would make deadline programs deterministic.
- **Not yet, marked `(C122)`:** the standard input waited for
  (`STDIN_READY`, a timer wait with an input waiter); the seed (Fixpt's
  `FIXPT_SCHED_SEED`, which its tests do not set). Regions remain one
  heap (C122), so a task's places are only numbers.
- **`--slice N`** for kayak-cells, as Fixpt's `FIXPT_SLICE`, to preempt
  at nearly every poll in a test. `cells-programs.py` runs each program
  that runs tasks again at a slice of 2, with no step limit, its answer
  the same. Of the 34 such programs, run at slice 2 on Fixpt and on tier 1,
  all agree but two, neither tier 1's fault: `schedules/process-fails.fx`,
  whose task fails during the next form, which Fixpt then reports as an
  error of its run, not of a form; and `preempt/allocating-short.fx`,
  which needs more than 20,000,000 steps in all at that slice. Fixpt's
  limit is each top-level form's, and tier 1 runs the listing's one
  `program` word, so its limit is the whole program's: a program whose
  forms each fit but whose sum does not differs (none at Fixpt's slice).
  A slice of 1 is degenerate: after a preemption's grace no cell is a
  task's own, and Fixpt's run of `monitors/one-message.fx` does not end
  (tier 1's overflows its stack).

## C126. Tier 2, the hand machine: how it is built (2026-10-16, TODO §31 step 3)

The user: move to tier 2 now; tier 1's remaining gaps wait until a
discrepancy needs them.
- **What tier 2 is**: Fixpt's `fx-cells-asminterp`, the same cellular
  words tier 1 runs, interpreted by arm64 routines (Fixpt's
  `fixpt-native/src/cellular.rs`, Rust emitting arm64 with its encoder).
  Kayak's routines are emitted by KFX, with FX-26's own encoder
  (`fx26:arm64.fx`, built in), into a bloblet that `seal-code` places and
  `native-procedure` makes callable: made code, within FX-26 (C116).
- **On tier 1's memory, shared, not copied.** The hand machine runs on
  tier 1's heap, stacks and registers as they are: KFX flat arrays
  (`(flatarrayof i64 @c)`), which kayak-rt never moves or frees. Tier 1's
  values are already Fixpt's (an index times 8, tagged), which is what
  Fixpt's machine adds to its heap's base. So a flat array crosses into
  native code as Fixpt's flat array does, the word of its suffix, with a
  header of Fixpt's before its elements (kayak-rt), shared; FX-26 takes a
  flat array as a native procedure's argument (probed on Fixpt).
- **Grown a routine at a time.** The machine's code runs cells until one
  it has no routine for (a primitive, an allocation, a collection, a
  poll, a trap), and gives back to KFX, which runs that cell as tier 1
  does (`routine!`) and enters the machine again. So it agrees with tier 1
  from its first routine, and each routine moved into machine code is
  checked by the same programs (`cells-programs.py`, the machine named),
  as Fixpt checks its machines against each other.
- **Steps:** (1) flat arrays shared with native code; (2) the machine's
  entry, `NEXT` and exit, with the stack and control routines; (3) the
  rest of Fixpt's routines, by how often the programs give back.

- **First steps done (2026-10-16).** (1) Flat arrays shared:
  `tests/kfx/made-flat.kfx` (`native 42 42`, on kayak and on Fixpt); a
  native procedure of four parameters (`native_procedure4`). (2)
  `cells/hand.kfx`: the entry, `NEXT` (fuel counted per cell, as tier 1
  counts it, so step limits and preemption fall where tier 1's do),
  entering a word, the ways out, a return (`pop-return!`'s), and the
  routines `lit dup drop swap over eq + - < int-add int-sub int-less
  int-eq car cdr pair-car pair-cdr slot slot! global branch 0branch exit
  return`; every other cell given back. `kayak-cells --hand`;
  `cells-programs.py` runs each program so too: all 326 agree. Fixpt's
  `bench/fib.fx`: 0.50 s on tier 1, 0.18 s on tier 2, with 2.7 million
  cells still given back (calls).
- **KFX, on the way.** `load-input` and `load-module` in two files of one
  unit named their modules alike (`made--load--0`): now the file's stem
  is in the name. A datatype's arm naming its fields without a list
  crashed kfx ("a list is expected"): refused now, with a message
  (`tests/kfx-stricter/arm-unlisted.kfx`; FX-26 binds the name to the
  fields as a product). A native procedure's type in a `nil`'s turbofish
  (`(%native …)`) had no Rust: now a procedure's. kfx's "a type KFX-0
  does not have" names the type.
- **Calls (2026-10-16).** `call`, `tailcall`, `tcall`, `ttailcall` (a
  cellular closure checked by its trailer and header, Fixpt's
  `is_closure`), `free`, `field` (a field read as `field-ok?` allows,
  past a trailer or a header). All 326 agree on tier 2. Fixpt's
  benchmarks, user seconds, tier 1 then tier 2 (each from Fixpt's
  listing, nothing compiled): `captures` 0.10, 0.03; `closures` 1.57,
  0.20; `fib` 0.46, 0.03; `helpers` 2.45, 0.12; `lists-region` 1.31,
  0.22; `lists` 1.19, 0.17; `loop` 2.50, 0.09; `tak` 0.19, 0.01. (Fixpt's
  `fx-cells-asminterp`, which checks and compiles the program first, about
  0.2 s of each: 0.22 to 0.31.) What is given back most now: allocation
  (`cons`, `closure`) and `prim`: `closures`, `lists` and `lists-region`
  give back three million cells each.
- **Allocation (2026-10-16).** `cons` and `closure` bump tier 1's heap's
  `top` in machine code, laid out as tier 1's `cons!` and `make-bloblet!`
  lay them out; given back where tier 1's `safepoint!` would collect
  first (`top` past three quarters of the heap; always under
  `--gc-stress`) or `room!` grow the heap. `cells-programs.py` runs each
  program on tier 2 under `--gc-stress` too. Benchmarks, tier 1 then tier
  2, user seconds: `captures` 0.12, 0.03; `closures` 1.73, 0.09; `fib`
  0.48, 0.02; `helpers` 2.56, 0.12; `lists-region` 1.38, 0.25; `lists`
  1.22, 0.06; `loop` 2.15, 0.11; `tak` 0.21, 0.01. What is given back
  most now is `prim` (`lists-region`'s region primitives, three million).
- **`prim`, `global!`, `field@` (2026-10-16).** What is
  given back, counted (`--gc-stats` now lists it by routine, and a
  `prim` by its primitive), past the slices' ends: `lists-region`'s
  `%region-cons`, three million; `captures`' `prompt`, `abort`,
  `callcomp`, `resume`, twenty thousand each; collections. So `prim`
  has machine code for `%region-cons` (the rest given back), and
  `lists-region` takes 0.07 s on tier 2 (0.25 before). Fixpt's own hand
  machine calls out to Rust for `prim`, `cons`, `closure` and every
  control routine (`CONTROL_CALLOUTS`: prompts, marks, continuations),
  as tier 2 gives them back; it has in arm64 three routines tier 2 did
  not: `global!` (its count of writes kept) and `field@` (`field-read!`,
  as `field`), now added; and `execute`, which no listing of Fixpt's
  programs has, so it is left given back until one does (untested code
  in the machine would be code that agrees by luck).
  Continuations stay with tier 1 until a program needs them faster.

## C127. `->` and `->*` in kayak's own KFX (2026-10-16; the user asked)

- **`->`** (`(subr pure …)`) has almost nothing to replace in kayak's own
  code: KFX's procedures declare named effects, `kfx` (about 1,400) and
  `cm` (about 300), and `pure` 4 times. The 261 other `(subr pure …)` are
  FX-26's primitives' types, as text in `check-defs.kfx`, and Fixpt's
  built-in files.
- **`->*`** (the effect found from the definition's `lambda`, C123) is
  where the gain is, with a cost: a named effect is a bound the checker
  holds a procedure to, which says what it may do (the compiler's `kfx`,
  tier 1's `cm`); a found effect states nothing and grows with the body,
  and every caller's with it. So: `->*` for helpers whose effect is
  incidental, inside a module whose interface keeps named bounds; named
  effects where they are the contract (entry points, what other modules
  call, higher-order parameters, which `->*` cannot type anyway).
- **Begun with `cells/sched.kfx`**: its 63 top-level and `define-rec`
  procedures `->*`; kfx accepts it and makes the same Rust, byte for byte.
  The compiler's own files (`kfx/`) next, helper by helper, where a found
  effect reads as well as the bound, not as a sweep.

## C128. Tier 4: the listing's words compiled by the hand machine's routines (2026-10-16)

Fixpt's tier 4 (`fx-cells-fxjit-cellcc`) is `native.fx`, written in
FX-26, the twin of its Rust `assemble_word`; but it compiles words for
Fixpt's own hand machine (its registers, its state, its stacks growing
down, its routines jumped into), given as parameters
(`native-layout.fx`), and kayak's tier 2 runs on tier 1's stacks, growing
up, and gives cells back (C126). So kayak's tier 4 does what
`assemble_word` does, of kayak's machine: each cell of a word compiled to
its routine's code in `hand.kfx`, the same code the machine runs, the
dispatch between cells gone (`NEXT`'s step kept: the fuel counted per
cell, so step limits and preemption fall where tier 1's do), the ip kept
in step, so that compiled code and cells mix at any cell.
- **Entering compiled code.** A compiled word's entry is `r-count + s`
  for its number `s`, Fixpt's convention, which tier 1's `enter!` already
  takes as `docol`. A call, a return, the machine's entry and a word
  entered go on through `resumed!`: the word's table of where to resume
  (by cell, the offset of its code, or 0), as Fixpt's machine's resume
  tables; no code, the cell is interpreted.
- **The code**: a jump to the machine's entry, the tables (data, before
  the code, so that `adr` reaches them), the machine, the words. Each word
  gives back through its own stubs, near (a conditional branch reaches a
  megabyte), and words of more than 4,000 cells are not compiled. Labels
  are placed in a flat array, no longer looked up in a list.
- `kayak-cells --compiled`; `cells-programs.py` runs every program so,
  under `--gc-stress`, and for tasks with a slice of two.
- **Results.** All 326 agree compiled, plain, under `--gc-stress`, and
  for tasks with a slice of two. Fixpt's benchmarks, user seconds, tier 2
  then tier 4: `captures` 0.02, 0.02; `closures` 0.09, 0.07; `fib` 0.02,
  0.01; `helpers` 0.12, 0.09; `lists-region` 0.07, 0.05; `lists` 0.06,
  0.05; `loop` 0.09, 0.06; `tak` 0.01, 0.00. Fixpt measured its tier 4
  likewise a little faster than its tier 2 (fib 14 ms, 17): what is left
  is each cell's work, the operands still read from the word; register
  code (tier 5) is where the next factor is.

## C129. Tier 5 begun: register code read, and interpreted by tier 1 (2026-10-16, the user: "go ahead with Tier 5")

Fixpt's tier 5 (`fx-regs-rsjit-cellcc`, `fixpt-native/src/cellular/regcode.rs`)
compiles each lambda's register code (Larceny's MacScheme machine:
`RESULT`, `REG1`…`REG8`, frames, 35 operations, `layout.rs`'s
`regcode::OPS`) to arm64 inside its hand machine; nothing of Fixpt
interprets register code, so its meaning there is that code's. Kayak
takes the road it took from tier 1 to tier 2: register code interpreted
in KFX first, on tier 1's machine, the reference the compiled code is
checked against and gives back to; then compiled.
- **Read from the listing.** Fixpt's listings already carry each word's
  register code (`its register code (N cells, not compiled)`, written by
  `disasm.rs`'s `register_lines`): kayak's reader keeps it (it passed it
  over, but for its literals' labels) and makes a register word as
  Fixpt's `make_register_word` does, its twin the word. `lambda`'s third
  operand, the routine that makes the closure, is not listed: `closure`,
  the cellular convention's.
- **Fixpt's conventions, on tier 1's stacks** (growing up): `save M`
  pushes `M` slots of `#f` and the link (0 here: no `blr`), `fp` at the
  first; `pop M` drops them; a `return` pushes `RESULT` and returns as
  stack code does, so a caller of either kind finds the value on the data
  stack. A stack-code call of a word whose twin is register code goes
  through the adapter (its frame's values into the registers, the frame
  dropped); an `invoke` of one keeps the arguments in the registers.
  What may collect (`prim`, `lambda`, `cellular`, a call) has its
  operands pushed and its value come back on the data stack, as Fixpt's
  call-outs: the registers are dead there (Fixpt's compiler keeps what
  lives across in the frame), so the collector need not see them.
- **Points to resume at, in `kk`.** A register word's `kk` below zero
  marks where to resume once something else has run: after a call or a
  call-out, `RESULT` popped; after a poll that preempts, every register
  (spilled to the data stack first, as Fixpt's `poll_regs` keeps the live
  ones). So return entries, continuations, the collector and the
  scheduler work on register code unchanged (Fixpt marks the entries a
  `blr` made the same way, negating `8k`).
- **Steps**: one an instruction, as a cell (Fixpt's tier 5 polls at
  entries and loops); `--regs` links the twins (`kayak-cells --regs`).
  Tiers 2 and 4 give back a call of a word with register code (the
  adapter is tier 1's) and a return into register code (a register
  word's entry is -1, past every compiled word's), until tier 5 compiles
  it.
- **Dead registers cleared.** Fixpt's poll spills only the live registers
  (`liveness`); tier 1 spills all nine, so a dead one must not hold what
  a collection has since moved (forwarding it wrote into the heap at
  random: `schedules/arrival-order.fx`, a slice of two). Every register
  is dead where register code is entered from stack code past its
  arguments, and after a call or call-out: cleared to 0 there.
- **A slice of three, for register code.** At a slice of two a preempted
  task running register code never runs on: the trampoline's way back
  (`drop`, `%fx26-preempt-done`, `drop`, `exit`) spends the slice, and the
  next poll preempts it before its next instruction, as a slice of one
  does to cells (C125). `--regs` with tasks is checked at three.
  (At two, `schedules/process-fails.fx` answered 3, Fixpt's per-form
  answer: where preemption falls decides whether its failing process
  fails before the program's last form ends.)

## C130. Tier 5: register code compiled into the hand machine (2026-10-16)

As Fixpt's `assemble_register_word`, of this machine: `RESULT` in `x0`,
`REGk` in `xk` (as Fixpt's), `REG0` `clo`; the nine kept in the state
whenever the machine is entered or left (`st-regs`), so that it gives
back at any instruction, tier 1's interpreter (C129) going on from there.
- **Each instruction** counts its step and keeps its place in `x12`, as a
  cell does (Fixpt's polls only at entries and loops: kayak's tiers keep
  one count of steps). In machine code: the registers and frames
  (`save`, `pop`, `stack`, `setstk`, `load`, `store`), `const`,
  `global`, `setglbl`, `lexical`, `field`, `op1` of `car`/`cdr`, `op2` and
  `op2imm` of the fixnum routines, branches, `global-guard`, `cellular
  cons 2` and `prim %region-cons 3` (allocated in line), `invoke`,
  `tailinvoke` and `invokeself` of register code, `lambda` (a closure
  laid out as `closed!` lays one), `return`. Everything else is given
  back: other primitives, calls of stack code, `cellular` but `cons`,
  `setfield`. Every call-like instruction's point after has code in the
  table's second half, so that the machine goes on there after tier 1
  ran it.
- **Calls.** An `invoke` of a closure whose word's twin is register code
  of as many arguments keeps them in the registers and pushes a return
  entry to its point after the call (`-kk`, C129); a return of either
  kind pushes its value on the data stack, and the point after the call
  pops it (the table of where to resume has a second half for these).
  No `blr`: Fixpt returns to register code by its link; here every return
  goes through the return stack, as tier 1's does.
- **Stack code calling register code**: the adapter, in `called!`, as
  tier 1's `call!` (the frame's values into `x1`…, the others 0).
- `kayak-cells --regs --compiled`.
- **Results.** All 326 agree on tier 5, plain, under `--gc-stress`, and for
  tasks at a slice of three. Fixpt's benchmarks, best of ten runs, in ms,
  each from its listing (start-up included), tiers 2, 4 and 5: `fib`
  29.6, 20.9, 16.1; `tak` 13.3, 9.6, 7.5; `loop` 90.9, 60.5, 29.3;
  `helpers` 119.5, 99.2, 20.6; `closures` 89.5, 75.3, 52.6; `captures`
  27.5, 27.1, 28.9; `lists` 70.9, 54.5, 34.7; `lists-region` 69.9, 54.2,
  36.4. What register code gives back most now: `cellular` (`captures`'
  control routines, 60,000; `cons` when a collection is due).
  `--gc-stats` counts register instructions given back by name.

## C131. Tier 5's calls by link, and of stack code (2026-10-16)

As Fixpt's tier 5 calls register code: an `invoke` of a register word
whose code is compiled jumps to it by `blr` (in tail position `br`, the
link this procedure's), the arguments in the registers; `save` keeps the
link in the frame's link slot (0 before: tier 1's frames, and a link
that would not read as a fixnum); after a call the link is loaded again
from it. A `return` with a link goes straight back to the caller's code,
`RESULT` in `x0`, popping the entry the caller pushed (as tier 1 pops it,
and passing by none: a prompt's or a mark's on top sends it the stack's
way). With none (entered from tier 1, after giving back, from stack
code: `l-resume` clears `x30`), it returns the stack's way, as before.
Both stay right because the caller always pushes its marked entry and
has code to resume at: a link only skips the way round. The points a
`blr` returns to are 8-aligned within the code (`align-odd!`), and the
code itself 16-byte aligned (kayak-jit's `seal`, held to it by
`tests/code.rs`; the machine's entry checks it, and traps rather than
run otherwise), so that a link in a frame reads as a fixnum to the
collector, as Fixpt aligns them.
- **Calls of stack code** from register code in the machine too: the
  registers pushed as the frame, a return entry to the point after, the
  word entered (`resumed!`).
- **Results.** `fib` 16.1 ms to 12.9, `tak` 7.5 to 6.3 (the rest within a
  millisecond); `captures`' control routines are still given back.

## C132. Fixpt's programs and answers, kept in kayak (2026-10-17)

The user's (2026-10-17): store Fixpt's tests locally. Until now the scripts
that check kayak against Fixpt's FX-26 programs read them from the clone,
and asked the pinned Fixpt for every answer: no clone, no check. Kayak is
to be the reference (PLAN §9), and a retired Fixpt answers nothing.
- **The programs**: `tests/fx26/programs/`, Fixpt's
  `crates/fixpt-fx26/tests/programs/` at the pin, every file (710, 2.8
  MB, their relative loads intact), written by `scripts/fx26-tests.py`
  from the clone's index at `FIXPT_REV`; `--check` in `check.sh` holds it
  to the pin when there is a clone. Copied whole, not only what passes,
  so that what kayak does not do yet is in its own tree too.
- **The answers**: `tests/fixpt-answers/`, what Fixpt said of each run a
  script asked for (`eval`, `check`, `compile --full-literals`, with their
  options), kept by `scripts/fixptcache.py`, the one door every script
  already asked Fixpt through. Each is `NAME.HASH.json`, the hash of the
  arguments and the files' texts (not the pin), its output a line to an
  array element so that a pin's changes read as a diff; a run past its
  time limit is kept as one, and replays as it ended. `REV` says which
  pin they are. With no clone, the answers kept are used, and an answer
  not kept is an error that says so. With a clone and answers of another
  pin, each is asked again and rewritten where it changed;
  `target/fixpt-cache/` (every pin's runs) still sits under it, keyed as
  before, so that is quick. `check.sh` notes which answers it asked for
  and, with a clone, ends by dropping the rest and stamping `REV`: moving
  the pin is editing `FIXPT_REV` and running the check, and the commit
  shows how Fixpt's answers moved.
- **Not kept here**: the corpus's answers, which are kept already as
  `tests/kfx/NAME.out` (`corpus.py` asks with `keep=False`; Fixpt's
  refusals of `tests/kfx-refused` and `kfx-stricter` are kept);
  `io-programs.py`'s, whose programs read the world; and stage 0
  (`fixpoint.sh`), Fixpt running the compiler, which is a check of
  Fixpt as an implementation rather than an answer to keep, and which
  kayak's own engine is to take over (PLAN §6, the second self-hosting).
- **Not chosen**: the answers in one file (every pin's change one huge
  diff); hashes as names alone (a person could not find a program's);
  only the programs, still asking a live Fixpt (no check without one).

## C133. kayak's own code generator: FX-26 to cellular words (2026-10-17)

The user's (2026-10-17): "move ahead with our own FX-26 to cells
generator". Until now the tiers ran Fixpt's words, read from its listings;
kayak made none of its own.
- **Chosen: Fixpt's compiler written in FX-26, ported to KFX** (K7), in
  `fx26/cg.kfx`, decision for decision: `compile.fx`, `compile-exps.fx`,
  `compile-state.fx`, `compile-lift.fx`, `compile-programs.fx`, as far as
  they go so far. Fixpt keeps two compilers that make the same words, one
  in Rust and this one; following it means kayak's words can be compared
  with Fixpt's word for word, not only by the answers they give.
- **Its input**: our checker's trees, made Fixpt's parser's (`cx`, its
  `exp`), with its parser's expansions (`let*`, `cond`, `and`, `or`,
  quotes, integers past a fixnum as arithmetic, datatypes' constructors):
  each node keeps where it starts, which names a lambda's word
  (`outer/inner@N`). Not the engine's IR (C58), which drops positions and
  the names `let` binds, both of which word names show. The checker's
  facts give what types decide: an `extract`'s field (`extract-index`,
  noted already).
- **Its output**: the listing, as `fixpt compile --full-literals` writes
  one (`disasm.rs`, `literal.rs`), which kayak-cells reads: `kayak-fx26
  --cells FILE`. FX-26's unit is the interned symbol `#u` on the cellular
  machine, as Fixpt's compilers make it (`cellular.rs`'s `unit`), so
  `|#u|`.
- **Fixpt's `standard.fx`**, its table of each standard operation's
  runtime primitive, which Fixpt generates, is a built-in module
  (`scripts/built-in.py`), loaded as it is: KFX accepts it unchanged.
- **The check**: `cells-programs.py --ours`, Fixpt's programs (C132) with
  our words, run every way Fixpt's are (tiers 1, 2, 4, 5, the collector
  stressed, tasks sliced), their answers Fixpt's; those that agree kept
  in `tests/cellgen-passing`, and how many of them are Fixpt's words
  exactly counted. A word's register code (its twin) is left out of the
  comparison: not made yet.
- **First results**: 77 programs agree, none differ, 68 of them Fixpt's
  words exactly; `bench/fib.fx` runs on tier 4 from our own words. Most
  programs not compiled are our checker's refusals (TODO §30); the
  generator's own gaps, by count: modules, `letrena` and regions,
  prompts, bloblets, `typecase`, `define-generative`, floats, `vlambda`,
  `case`, `quasiquote`.
- **Not yet, and where the words differ**: Fixpt's middle phase
  (`compile-plan.fx`: inlining, specialization, `map-add@lambda@730`),
  lambda lifting (`compile-lift.fx`), register code (`regcode*.fx`), the
  redefinition of a global at a compatible type (Fixpt keeps the global),
  the checker's conversions (`%fx26-i64->int`, conventions), sharing in
  literals (`#n=`). A program defining `cons` or `nil` would shadow what a
  quote's rewrite means (Fixpt writes `(with #%fx cons)`).

## C134. The pin on Fixpt's x86_64 branch, and kayak's checks on Linux (2026-10-18)

The user asked for tiers 2 and up on x86_64, and chose, asked: the
x86_64 convention for §97 goes into Fixpt first, not drafted in kayak;
then, Fixpt's `main` building on macOS alone, "start your work now by
using the Fixpt branch at x86_64-fxjit-nativecc". The pin is that
branch's committed tip, `35c59a3` (the user's checkout there had changes
of its own, not taken); the clone is a fresh one at
`../kayak-refs/fixpt`.
- **The drift, decided:** of 24 ported sources, 6 behind, three of
  Fixpt's files. `fixpt-engine`'s `cellular.rs`: a prompt's tag kept
  across a collection, and a stack check, both in calls between
  machines, which tier 1 has none of: not carried over.
  `fixpt-runtime`'s `prim.rs`: `%fx26-remainder` never collects (kayak's
  tier 1 keeps no such list: Fixpt's safepoints come with the listing),
  `native-code?` false off arm64 (tier 1's is false), `%tword-*` of
  register words and `%tword-bits` (tier 1 has no `%tword-*`): nothing
  to carry. `fixpt-native`'s `cellular.rs`: code counted in bytes, the
  x86_64 machine chosen beside the arm64 one, whose routines did not
  change: nothing for `hand.kfx` now; its x86_64 twin will follow
  `cellular/x64.rs` (TODO §34).
- **Fixpt's answering machine named** (`fixptcache.ANSWERING`,
  `--fx26-machine fx-regs-rsjit-nativecc`, in `corpus.py`, `stage0.py`,
  `io-programs.py`, `run-fixpt-programs.py`): Fixpt's default on arm64,
  so nothing changes there; on x86_64 its default is register code on
  the cellular machine, whose fixed stacks overflow on `recursion` and
  `lists` where native frames do not. Not chosen: leaving each machine
  its default, which made the answers depend on which one ran.
- **Answers by machine:** `NAME.out` is Fixpt's answer on arm64 macOS,
  where the corpus was made; on another machine, where Fixpt's differs,
  `NAME.ARCH-OS.out` beside it (`fixptcache.answer_path`, and the Rust
  test by `std::env::consts`), which `--bless` writes there, and removes
  once `NAME.out` would do. Three today on `x86_64-linux`: `floats`
  (glibc's `tan`, `asin`, `acos` differ from macOS's in the last bit;
  Fixpt and the Rust agree on each machine), `made-code` and `made-flat`
  ("interpreted": no made code runs here yet). Not chosen: dropping the
  programs' machine-dependent parts, which would lose what they check.
  And so in `tests/fixpt-answers/` (C132's, met when this work was
  rebased onto it): its answers and their stamp `REV` are arm64
  macOS's; another machine's answer, where Fixpt's differs there, is
  `NAME.HASH.ARCH-OS.json` beside the shared one, written only where it
  differs, and that machine's stamp `REV.ARCH-OS`. A check elsewhere
  writes a shared answer only where there is none, prunes only its own,
  and stamps only its own, so that it never vouches for macOS's answers
  at a pin macOS has not run. Fixpt's messages name its files by path,
  so an answer is kept with the checkout's path written `<kayak>`
  (`fixptcache.HERE`) and read back with this checkout's: without it,
  691 answers differed from macOS's in the path alone; with it, 17 do,
  each the machine's (arm64's declared conventions, the default
  convention printed). So too a Fixpt program's path, the clone's copy
  or kayak's (the same files), as `<programs>`, read back as kayak's;
  and a script's temporary directory, macOS's or Linux's, as `<tmp>`,
  which differs at every run. Not chosen: one store for every machine,
  in which a Linux check rewrote macOS's answers and stamped them as the
  pin's.
- **`kayak-jit`'s routines** `foreign`, `bloblet_routine`,
  `cons_routine` are called only by arm64 macOS's trampoline, so clippy
  refused them elsewhere as dead: allowed so, until x86_64's trampoline
  calls them (TODO §34).
- **Results on Linux x86_64**, the first full check there: stage 0 runs;
  181 of Fixpt's programs kfx accepts agree; tier 1 agrees on 338 (new
  programs upstream); the engine differs on one beyond KFX
  (`native/crossing-overflow.fx`, TODO §34). Tiers 2, 4, 5 do not run
  here: `kayak-jit` makes no code area off arm64 macOS.

## C135. Made code on x86_64 Linux: the pin on Fixpt's §97 there (2026-10-19)

Fixpt did kayak's request (its "Code a program makes, on x86_64",
`5ef9fe8` on `x86_64-fxjit-nativecc`): `native-x86_64.conv`, which
`native-convention` gives on x86_64 and both its checkers compare a
`define-convention` with, `runs_made_code` true there, and its made-code
programs twinned in `programs/made-code-x86_64/`. The pin is there.
- **`kayak-jit` on Linux x86_64**, to that declaration: the code area's
  two views by `mremap` of a shared mapping (as Fixpt's
  `fixpt-memmgmt`), no cache to flush; the stack's limit by
  `pthread_getattr_np`; the trampoline with the state in `r15`, the
  arguments in `rdi`, `rsi`, `r8` and the state's `reg4`–`reg8`, the
  count in `r10`, the closure in `rbx`, fuel, limit and distance read by
  code from the state, so nothing to load or give back; Rust's
  callee-saved registers pushed, and the stack the way back from any
  depth (`rust-sp`), as on arm64. The routines, each entered by `call`,
  align the stack for Rust themselves (`and rsp, -16`), since nothing
  declares its alignment at the call. kayak's own `leave` field moved
  past `reg8` (656), out of either declaration. `DECLARATION` is the
  machine's, chosen as Fixpt's `NATIVE_CONVENTION` is. `tests/code.rs`
  is one set of tests over a `machine` module of each machine's
  instructions; `tests/declaration.rs` reads each machine's clauses.
- **kfx's checker compares a `define-convention` with the machine's it
  runs on**, `(native-convention)`, not a text built in: Fixpt's
  checkers do so, and stage 0 (Fixpt running the compiler) then agrees
  with stage 1 on every machine. `native-convention-text` and its
  `lines-of` went. Not chosen: both texts built in and the target named
  to kfx, a cross-compiler's question no one has asked.
- **Programs of one machine in the corpus**: a program that declares a
  convention is of the machine its `target` names
  (`fixptcache.machine_of`), since Fixpt refuses another's. Elsewhere its
  files are kept as they are, unchecked, and its module and entry in
  `kfx-corpus` are built only for its `target_arch`; `engine-programs`
  keeps its line in `engine-passing`. The five arm64 programs that
  declare (`made-allocated`, `-declared`, `-frames`, `-generated`,
  `-linked`) have twins, `made-NAME-x86_64.kfx`, from Fixpt's twins as
  the arm64 ones are from its originals (the last forms in `main`), with
  `(read @k) (alloc @k)` in `main`'s effect for the region Fixpt's code
  uses, and `spin` in `-declared`'s. Their answers are the arm64 ones'
  but `target`'s. Not chosen: one program per test choosing its encoder
  by `target`, which a declaration checked before the program runs
  forbids, as Fixpt found.
- **The programs that declare nothing** (`made-code`, `-encoded`,
  `-flat`, `-typed-calls`) run their arm64 bytes only where the
  convention's `target` is arm64 (`here?`, a procedure, since KFX makes
  no constant that reads), as Fixpt's `leaves.fx` and `typed-calls.fx`
  do now; else the answers they would give, so on x86_64
  `made-code`/`made-flat` keep their own answers ("interpreted").
- **Fixpt's x86_64 encoder built in** (`fx26:x86_64.fx`,
  `fx26:x86_64-types.fx`), which the twins load; KFX accepts it as it is.
- **The drift**: `prim.rs` changed only `runs_made_code`; tier 1's
  `native-code?` is false on every machine: nothing carried.

## C136. Tier 2 on x86_64: hand.kfx's machine, routine for routine (2026-10-22)

The user: "move the pin, then start tier 2" (TODO §34, step 3).
`cells/hand-x64.kfx` is tier 2's machine on x86_64: `hand.kfx`'s entry,
`NEXT`, ways out, return and every routine, in the same order, emitted
by Fixpt's FX-26 encoder (`fx26:x86_64.fx`) and run by `kayak-jit`, as
Fixpt keeps `cellular/x64.rs` beside its arm64 machine.
- **Registers, as Fixpt's table** (`r14` ip, `r13` `cur`, `r12` the data
  stack, `rbx` the return stack, `r15` the state, `rbp` the frame, `r9`
  `clo`, scratch `rax rdi rsi rcx rdx r8 r11`), and `r10` the heap's base,
  which kayak's machine needs (tier 1's values are offsets into its
  heap) and Fixpt's has not. What hand.kfx keeps in registers and x86_64
  has none left for is in the state: the fuel (as Fixpt's), the data
  stack's base and the cell being run (elements 30 and 31). The native
  convention's pinned `r15`, and every register the machine uses, kept on
  the machine stack and given back.
- **Code in bytes**, each jump to a label a 32-bit one (`jmp`, `jcc`, `lea`
  of `rip`), placed once every label is: none out of reach, so no long
  branches; the routines' table is a `jmp` of 5 bytes each.
- **What it gives back that hand.kfx does not**: a call of a word with a
  register twin (`--regs`), before changing anything, for tier 1 to
  enter as it enters it (hand.kfx adapts it, `h-adapted!`, register code
  being its tier 5); a word whose entry is not `docol` (no word is
  compiled here). Same answers, by tier 1.
- **Chosen by the machine**: `hand-code!` makes arm64's machine
  (`hand-code-arm64!`) or x86_64's by the running convention's `target`,
  and refuses elsewhere, so `--hand` never runs another machine's bytes.
  `--compiled` (tiers 4, 5) on x86_64 refuses, saying so, until §34's
  step 4. The machine's effect `cm` allows the encoder's region `@k`.
- **Checked**: `cells-programs.py` runs `--hand` and `--hand --gc-stress`
  on x86_64 Linux too: the 338 programs agree. `bench/fib.fx`: tier 1
  0.83 s, tier 2 0.09 s, the machine entered 452 times.

## C137. Tiers 4 and 5 on x86_64: hand.kfx's, at the same places (2026-10-22)

The user: "Go for it" (TODO §34, step 4). `cells/hand-x64.kfx` now
compiles the listing's words (tier 4) and register words (tier 5) under
`--compiled`, as hand.kfx's `word-compiled!` and `register-compiled!` do,
cell for cell and instruction for instruction, into tier 2's x86_64
machine (C136).
- **The same tables at the same places**: 8 bytes past the code's start
  the count of the words compiled and the table of their tables, each
  word's table of where to resume (and, for register code, its second
  half, the points after a call), offsets from the start. So `resumed!`,
  a return entry and `entry-address!` mean on x86_64 what they mean on
  arm64; only the bytes differ. The words' list walks are hand.kfx's,
  copied (`ywords-compiled!`, `yregisters-compiled!`): KFX takes no
  function as a parameter.
- **Register code's registers, not Fixpt's**: Fixpt's x86_64 register
  code keeps `REG1`…`REG3` in `rdi`, `rsi`, `r8`. This machine has fewer
  to spare: `r10` is the heap's base, values being offsets, and its
  routines (field reads, allocation, closure checks) need `rsi`, `rcx`,
  `rdx`, `r8`, `r11` as scratch. So `RESULT` is `rax`, `REG1` `rdi` (the
  few places register code used it as scratch use others), `REG2` `r14`
  (the ip's register: an operand is read from `cur` and the heap's base,
  and an instruction's place is made only when it gives back, by a stub
  of its own after the word's code), `REG0` `clo` as on arm64, and
  `REG3`…`REG8` stay in the state, where tier 1 keeps them. Leaving
  register code for the machine stores `rax`, `rdi` and `r14` there;
  resuming in it loads them, by a way in each register word's table has
  at field 0 (never a cell's), so that stack code's `r14`, its ip, is
  never touched. Under `--regs --compiled` on a Zen 2, against all in the
  state: `fib` −11% cycles, `loop` −14.5%, `closures` −9%, `lists`
  −9.5%, `tak` −6.5%, `helpers` −6.5%, `captures` −1% (`REG1` alone: −2%
  to −7%). A third would take one of those five scratch registers from
  every sequence that uses it: not yet.
- **No link register**: hand.kfx's `x30` is the state's element 32
  (`st-link`, the state one longer) while a word runs; `save` keeps it in
  the frame, a call's return reloads it, as on arm64. A call is a `call`;
  the callee's entry for a call (just before its first instruction past
  `args`, a fixed 7 bytes, so found from the tables) pops the link at
  once; a return pushes it back and `ret`s; a call in tail position
  pushes its own link and jumps to the callee's entry, which pops it. So
  the machine stack holds one link at most, and only between a `call` and
  its pop; each `ret` pairs with its `call`, which the processor predicts
  returns by. Points returned to are 8-aligned (`xa`, no-ops), so a link
  in a frame reads as a fixnum.
- **Why `call`/`ret`** (the user's question, measured on a Zen 2): at
  first a call stored its return point and jumped, and a return jumped
  through the link, an indirect jump. Under `--regs --compiled`,
  `bench/fib.fx`'s mispredicted branches went from 324k to 30k with
  `call`/`ret`, its cycles down 6%; `tak` 81k to 32k (−2%); `closures`,
  `helpers`, `loop`, `lists` the same within 0.5%. `captures` is worse,
  191k to 478k (+3% cycles): it aborts a continuation 20 calls deep, so
  those calls never `ret`, and the 20 frames' returns after it resumes
  match nothing the processor kept. A resumed activation could return by
  a jump instead, but it would have to know it was resumed after its own
  later calls, which the frame's link (a fixnum) cannot say: not worth it
  for one benchmark yet.
- **A call of a word with a register twin** is now adapted, as hand.kfx's
  `h-adapted!` (`yadapted!`), not given back (C136's difference gone).
- **Checked**: `cells-programs.py` runs every `--compiled` way on x86_64
  Linux too. `bench/fib.fx`: tier 1 0.85 s, tier 2 0.09 s, tier 4 0.05 s,
  tier 5 (`--regs --compiled`) 0.03 s.

## C138. Tier 6 on x86_64: register code as native procedures, left for tier 5 (2026-10-23)

The user: "Do tier 6" (TODO §34 step 5), "port away", on x86_64. Fixpt's
tier 6 (`direct.rs`, `direct/x64.rs`, `native-direct-x64*.fx`) cannot be
taken as it is: kfx reads none of its FX-26 files (they are functors over
signatures, with state in module-level refs), and they are Fixpt's heap
throughout (values are addresses, its `DState`'s offsets baked in), while
kayak's values are offsets from a pinned base and its tier 1 is a KFX
interpreter that calls the machine and must be returned to. Nor can tier
6 keep native frames while tier 1 runs, as Fixpt's keeps them while its
Rust runs. So this is kayak's own, in `cells/hand-x64.kfx`, on Fixpt's
idea: register code compiled to native procedures that call by `call`
and return by `ret`, with none of the cellular machine's bookkeeping.
- **Beside tier 5, the same state**: a register word all of whose
  instructions tier 6 has (`y-direct-able?`) is compiled twice, tier 5's
  code and tier 6's, which share each instruction's body (`yreg-body!`),
  its registers (`RESULT` `rax`, `REG1` `rdi`, `REG2` `r14`, the rest in
  the state, C137), and its frames, on the data stack as tier 1's.
- **What it drops**: the step counted at each instruction (fuel is spent
  a block at a time, at its first: exactly as many steps as tier 1's,
  since a block is run whole or not begun); the return entry each call
  pushes (a call pushes a record on the machine stack, `(ret nx fp clo
  cur)` from its top, what the entry would hold, and moves `rbx` past the
  entry's place without writing it); the tables a call of itself looks
  up; giving back.
- **Leaving instead of giving back**: what tier 5 would give back, tier
  6 leaves (`y6-exits!`, `l-materialize`): each record written in the
  place it kept on the return stack, as the entry it stands for, the
  records popped, the fuel of the block not spent given back, and on at
  tier 5's code of the same instruction, which then does as tier 1 would
  (gives back, or calls stack code, or runs on). Its frames' links are 0,
  so they return the stack's way once left. So tier 6 needs nothing of
  tier 1, and the collector never sees a record: the instructions tier 6
  has never allocate (no `prim`, `lambda`, `cellular`, `setfield`; `op2`
  only of the fixnum routines and `eq`); 61% of the corpus's register
  words, 2026-10-22, and `lexical`, which reads, brings most of the rest.
- **Ways in**: tier 5's entry for a call of such a word goes to its
  tier 6 code; so do the tables' entries for each block's first, and
  tier 5's branches to one, so that a word entered from stack code, or a
  loop that left tier 6 to poll, goes back in at its next block. Each
  pushes a bottom record, `(1 1)`, and keeps the link tier 5's code had
  (`st-t6link`, the state's element 33): a `return` over it leaves, for
  tier 5's `return` to return by that link.
- **Calls**: `invokeself` is a direct `call`; `invoke` finds the callee's
  tier 6 code by field 1 of its word's table (0 for none: then it
  leaves, for tier 5's call). A register word's entry for a call is now
  field 2 (no longer found 7 bytes before its code). Records are at most
  as many as the return stack's entries (65,536), 2.6 MB of the 1 GiB
  stack kayak runs on.
- **`--direct`**, implying `--compiled`: tier 6 where it can be, beside
  tiers 4 and 5; refused on arm64. `cells-programs.py` runs `--regs
  --direct`, under `--gc-stress`, and at a slice of 3, on x86_64 Linux.
- **Measured** on a Zen 2, cycles less the run's start (about 7M), tier 6
  against tier 5: `helpers` −54%, `loop` −46%, `fib` −32%, `tak` −22%,
  `lists` −13%, `closures` −10%, `captures` the same.
- **Not Fixpt's**: Fixpt's tier 6 frames hold their own slots on a
  native stack, with stack maps, flushed to heap frames for its
  collector; it allocates and calls out. Here the slots stay where tier 1
  keeps them, and leaving is the flushing. Fixpt is no oracle for these
  bytes.

## C139. Values are tagged machine addresses; indices a mode (2026-10-23)

The user, on hearing that kayak's values were indices into tier 1's heap
(`docs/research/native.md`'s decision 2, an earlier session's, never put
to the user): "I wanted it to carry tagged memory address", "the former"
(addresses everywhere, tier 1 too), "it will be useful for some of the
earliest tiers to have a mode where they use indexes instead of
addresses". So a value is `vbase + 8·index + tag`:
- **By default**, `vbase` is the heap array's machine address, so a
  pointer value is its word's address and its tag, as Fixpt's are. With
  `--indexes`, and where no native code runs (KFX cannot learn an
  address there), `vbase` is 0: the indices of before, Fixpt's images'.
- **Tier 1** converts at its few places: `address` subtracts `vbase`,
  `heap-ref` and the collector's `to-ref` add it (the other space's);
  forwarding words keep indices. Suffix refs keep their owner as an
  index relative to the space's base, as Fixpt's do (34 bits, relative to
  its heap's).
- **Nothing moves but by a collection**: with addresses, each space is
  reserved once at `heap-capacity` (2²⁸ words, 2 GiB of address space),
  committed only as it is touched (kayak-rt makes a flat array of 0s of
  2²⁰ elements or more by kayak-jit's `zeroed_cells`, `mmap`), and the
  heap grows by raising `limit` within it, where `--indexes` still copies
  to a bigger array. Both collect at the same points: the same
  collections and words copied, `bench/closures.fx` under stress.
- **Its address**: FX-26 has no operation that says where an array is
  (`array-identity` is an identity for eq-tables), so a native procedure
  of two instructions, made at the start, returns its argument, a flat
  array's suffix, less its tag (`address-code!`). Made code, §97: no
  extension.
- **The machines (tiers 2–6, both)**: the state's element 34 is
  `vbase`; the base register (`r10`, `x19`) is the heap's address less
  it, 0 with addresses, so `[base + value]` is a value's word either way;
  an allocation's address adds `vbase` to the top's, and its value is that
  address less the base register. Every tier runs in both modes.
  *Amended*: on x86_64, with addresses, there is no base register. The
  machine is made after the run knows its mode, so each access to the
  heap is emitted for it (`yld-v!`, `ylea-v!`, `yvalued!`): `[value +
  off]` with addresses, `[r10 + value + off]` with indices. Register code
  then keeps `REG3` in `r10`. Cycles fell by 1–3% at tier 5 and under 1%
  at tier 6 (fib, tak, lists, helpers, closures, loop). arm64 keeps its
  base register (`x19`, 0 with addresses) until macOS can try the change.
- **Checked**: `cells-programs.py` runs every way with addresses, and
  `--indexes` by tier 1 (with `--gc-stress`) and tier 2. The arm64 side
  (`hand.kfx`'s entry and allocations) is untried: macOS must run it.

## C140. Fixpt's calling convention for tier 5, as a toggle (2026-10-23)

The user, on hearing how kayak's tier 5 calls differ from Fixpt's register
code on x86_64: "add a toggle to opt into fixpt tier5 convention". They are
fine with the registers differing (C137's), so this is the convention
alone. `--fixpt-calls`, x86_64 only (refused on arm64, whose `x30` is
already where both keep the link), off by default:
- **The link where the `call` put it**: a register word's entry for a
  call pops nothing; until `save`, the point to return to stays on the
  machine stack, and a leaf returns by `ret` from there, as Fixpt's
  `regcode.rs` does. `save` pops it into the frame, and into `st-link`.
  From there on, everything is C137's: `st-link` a copy of the frame's,
  reloaded after each call. Fixpt reads it from the frame at `pop`;
  that is the same link, kept in two places here.
- **A link of none**: where it would be lost before `save` (a call there,
  or a way in from the machine), a 0 is pushed instead, and a return
  that finds 0 goes the stack's way. So the machine stack holds that one
  link at most, as in Fixpt. Every way to the machine drops it: each
  instruction's give stub, the stack-way paths of `invoke` and `return`.
- **Ways in**: the tables' ways in to an instruction before `save` go by
  a stub that pushes the 0 (`ylink-stubs!`); points after a call push
  it themselves; tier 6's ways in take the link from the machine stack
  into `st-t6link`; and tier 6, leaving for code before `save`, pushes
  it back (`l-materialize-pushed`). Which instructions are before `save`
  is known as each is compiled (`h-5-pushed`), the same order assumption
  as `h-link`'s.
- **Measured** (Zen 2, `--regs --compiled`): cycles within noise of C137's
  on fib, tak, lists, helpers, closures and captures; instructions −0.6%
  on fib, +0.2% on helpers and loop; return mispredicts the same.
- **Checked**: `cells-programs.py` runs it with tier 5 and with tier 6,
  under `--gc-stress`, and at a slice of 3.

## C141. The pin at Fixpt 8f76d65: stage 0 in 25 s (2026-10-24)

The user handed Fixpt a note on stage 0's workload (about 610 s a run,
the check's longest wait), then: "Try updating Fixpt to latest". Fixpt's
`main` in the user's checkout, 45 commits past `4caffc2`, nine of them
not yet pushed: the pin is its tip, `8f76d65`, fetched from that
checkout into `../kayak-refs/fixpt` (read only there). Until Fixpt
pushes it, a clone of GitHub's alone has no such commit.
- **Stage 0**: Fixpt's `4ab77e92` makes kayak's emitter and checker
  native (a record accessor with no registers left, a closure size
  limit copied from arm64), so no procedure of the compiler runs as
  cellular code: one run in 25 s, against about 610 s, its output
  byte for byte `boot/src/kfx/`. `fixpoint.sh` beside the rest of the
  check, 40 s (was 616): the check 566 s (was 642), the cells programs
  now its longest wait (465 s, 368 programs).
- **The drift, decided:** 13 marks behind, six of Fixpt's files, none
  carried over. `fixpt-heap`'s `heap.rs`: the live heaps' serials, and
  weak references into the code area: kayak's heap has neither (one
  heap, no weak references). `fixpt-native`'s `cellular.rs`: native
  slots reused across machines, 2^18 of them: tier 1 has no slot table.
  `fixpt-runtime`'s `prim.rs`: `%fx26-assert`, `%fx26-assume`, for
  FX-26's specifications, which kayak's checker does not take yet: tier 1
  without them until it does. `compile-exps.fx`, `compile-state.fx`,
  `compile-programs.fx`: the next words' names a list taken one at a
  time, for the Rust side's `native_define`; kayak's names one word at a
  time, for which the two are the same.
- **Taken whole**: `built-in.kfx` (`assert`, `assume` named as Fixpt's
  primitives), and the boot and engine Rust from it; Fixpt's programs,
  48 new in `specs/` (`requires`, `ensures`), each refused by kfx as by
  Fixpt or, where Fixpt accepts, by kayak's checker. The checkers'
  comparison (§30): the same on 84 of the programs Fixpt accepts, one
  fewer than at `4caffc2` (which one, not yet found: no program of the
  old set changed but a comment, so it is Fixpt's answer that did).

## C142. Stress spaced out where much lives; tier 1's leaks on its hot paths (2026-10-24)

With stage 0 at 25 s (C141), `cells-programs.py` was the check's wait,
and within it three of Fixpt's programs under `--gc-stress`: a
collection at every safepoint, each copying all that lives and
scanning both stacks, so a cost of their product.
`preempt/allocating.fx` (two tasks of 100,000 conses) took 389 s so,
0.17 s without; `native/deep-control.fx` 44 s and
`native/crossing-overflow.fx` 26 s, their stacks some 65,000 frames.
- **Chosen** (the user: "the periodicity k grows in proportion to the
  heap size, and ... a deterministic-but-randomized amount of jitter"):
  after a collection under stress, `stress-wait` safepoints pass before
  the next, one for every `stress-span` (4,096) words found live, heap
  and stacks both (the stacks being what made two of the three slow),
  jittered from half of that to half again by a linear congruential
  generator of a fixed seed: a run collects at the same safepoints each
  time. Below a span, every safepoint, as before; the collection when
  three quarters full is kept. The three: 7.5 s, 0.9 s, 0.6 s.
  Fixpt's own stress tests shrink their workloads instead ("the shapes
  ... are what matter, not the counts"); kayak takes Fixpt's programs as
  they are.
- **Leaks** (the user: "we should definitely fix that memory leak. I
  want us to aim to support long running programs/services"): the
  runtime's memory is never let go (C43), so whatever tier 1 makes of
  its own on a path a program takes again and again grows without end.
  Found by `kayak-rt`'s `heap-stats`, and taken out:
  - each collection's lists of the names (symbols, globals, words,
    literals): now one flat array of their values, forwarded in place
    (`named-values`); a run of 32,000 frames' stress, 17 MB to 5;
  - a primitive's arguments as a list: now copied into a flat array
    (`pargs`), read by `prim-apply!`; `list` and a procedure's rest
    arguments made on the heap straight from where they are;
  - the closure of a `letrec` within a procedure, made at each call:
    `closure-made!`'s, `vector-from!`'s (each continuation taken),
    `call!`'s tail call, register code's `save`, `make-string!`'s,
    `reverse`'s, `make-flatarray`'s, each a procedure of its own.
  At a slice of 2, `preempt/allocating-short.fx` made 345 MB of the
  runtime's, now 31 MB; what is left is in TODO §31.
- **Not chosen**: leaving the three out of `--gc-stress`, as Fixpt's
  benchmarks are (the user chose the spacing); starting each program's
  other ways as soon as its tier 1 answer is in (that phase takes 5 s).
- **Measured**: `cells-programs.py` 424 s to about 200, the full check
  566 s to 339; the stage's wait now the
  `--slice 2` ways of `preempt/allocating.fx` (126 s and more): some
  6,500 steps of the scheduler's at each preemption, against 20 a loop's
  turn without.

## C143. A slice's end asked lazily; the emitter's facts by position; effects expanded once a state (2026-10-24)

The user: "Keep looking for improvements". With C142 the check's
waits were the `--slice 2` ways of `preempt/allocating.fx` and, beside
them, `incremental.py`'s 62 compiles.
- **C142 corrected**: counted (a run's counters, taken out again), a
  preemption is some 160 steps, not 6,500: at a slice of 2, a task
  resumed has a step or two before the next, so a turn of the loop
  costs some 6,700 steps (40 preemptions), 300 times its 23. Fixpt's
  machine runs the same rule (a poll each step, a grace after
  `preempt-done`); what tier 1 paid more was its own work at each
  slice's end, every second step.
- **The slice's end** (`poll-due!`): whether the scheduler's prompt is
  on the return stack, a walk of it, was asked at every one; now only
  where the answer counts, a task's preemption otherwise wanted, as
  Fixpt's `wanted` calls `reachable` last (`wanted-how!`,
  `preempt-wanted!`). `timer-due?` reads the clock only with a timer set,
  and makes no closure (C43). Whether the scheduler has a state, its
  vector's header read at each `st`, is now whether `sched-state` is not
  `#f` (`sched?`): it is that or the vector, which the collector forwards.
  The short program at a slice of 2: 5.9 s to 2.2 s.
- **`kayak-rt`'s `once`**, under every global KFX reads: it made the
  maker an `Option` on the stack and called through `&mut dyn FnMut`
  even when the value was made, so small procedures (`slot-of`) stayed
  out of line. Now the value is asked first, inlined, and the making is
  a cold procedure of its own (`made_first`). Tier 1 some 12% faster;
  kfx no faster (its time is its own, below).
- **kfx's facts by position** (`fidx`, emit.kfx): every fact of the
  program in one list, and the emitter's questions at each name
  (`fact-with`, `facts-at`, `ctx-fact?`, `fact-at-place`,
  `all-facts-at`) a search of all of it: a cost of the program's size
  squared. kfx compiling itself took 2.0 s at `2a6b2af` (11,100 lines),
  4.3 s now (16,800); `fixpoint.sh`'s note said 0.8 s then, recording.
  Now a balanced tree of the facts by position, each position's in the
  list's order, so that the first found is the same; the list kept in
  `ctx` too, beside it (`fx`), for the questions of the whole program
  (`module-impls`, `all-facts-prefixed`, ...). A stable merge sort, its
  merge by an accumulator, as the list may hold some hundred thousand.
- **Effects fitted** (`fit-effs`, check.kfx): both effects expanded
  again at each atom of the first, though the state had not changed,
  and expanding an expanded effect gives it again (no effect variable
  solved in it, no atom twice, none frozen). Now kept while the state
  is (`fit-expanded`), expanded again only after a `solve`.
  `region-atom?` made nine strings at each call; `resolve-atom` the
  region of an atom that has none.
- **Measured**, kfx's output byte for byte the same as before on every
  unit kayak has: itself 4.3 s to 1.9 s, `cells/cells.unit` 13.4 s to
  1.9 s, `fx26/fx26.unit` 6.2 s to 2.8 s.
  The check 339 s to 209 s; `incremental.py` ends at 178 s, the
  cells programs at 193 s (their `--slice 2` ways still the longest).
- **Not taken**: fat LTO for the workspace's release builds, tier 1 18%
  faster, but every release build of the check (`cargo test`, clippy,
  each generated program's) under it; nor `codegen-units = 1` alone,
  3%. Nor fewer polls at `--compiled --slice 2`, which gives tier 1 back
  nearly every cell: that is what the way tests.

## C144. kayak's code generator: modules (2026-10-24)

The user: "do 1" (TODO §31 item 5, the generator's gaps by count): 59
programs the checker accepts stopped at `module`. Fixpt's compiler in
FX-26 (`compile-exps.fx`'s `c-module`, `c-with`; `compile-state.fx`'s
module values and waits; `parser.fx`'s `with-includes`) ported to
`fx26/cg.kfx`, decision for decision, as C133 began.
- **A module** is a frozen product (`%make-frozen`, tag 37) of its own
  members, made in slots in order as a `letrec*`'s: a lambda written with
  its type that names a member not made yet captures a placeholder, given
  the member once it is made (`c-give-waiting`), as a `letrec`'s
  siblings are. An abstract type's conversions are identity lambdas
  where its form is, in slots but not the product.
- **Its type**: where it is not the module's own members (a `hide`'s are
  members, not in its type; an `include`'s values are), the product is
  made again of the type's values, each a path of fields into it
  (Fixpt's checker's `k-reshapes`): the values of the modules it
  includes, not hidden, then its own that it gives.
- **`include`**: a first member `%include`, the module of the modules
  included, made again of their values; the names of theirs the members
  use, in the order they first use them, bound once in slots after it;
  each member's value where its item is (a lambda's body, so its word is
  named for the item's place, `lambda@P`). **`extend`**: a module of
  `e0`, then `e1` in a `with` of it, made again of `e0`'s values, `e1`'s
  in their place, then `e1`'s others. A module kfx's reader defines
  where it is included or extended (`made--P`, C90) is made in place
  again; a load (`made--load--`) is the global of the module its file
  is, as Fixpt's `%shared:`. `import` is `(define x (with m x))`; a
  top-level `define-generative` its two conversions, named for them.
- **`with`**: the module's values the body names, each loaded and its
  field taken, in slots. Where each is: a module made in the program, by
  its members, which the generator keeps as it reads them; a module
  parameter, by its type, which **the checker now notes** (`with-spec`,
  at the `with`), and a module given for one is made again of that
  type's values unless it is that already (`module-arg`, at the
  argument). Fixpt's checker notes both (its `with` facts and reshapes).
- **Words named alike**: Fixpt lists two words of one name, an abstract
  type's conversions say; the listing now does (cg-words were found by
  name), and kayak-cells takes a word listed twice cell for cell the same
  as one (C122). Two of one name that differ (a global defined again, a
  datatype's constructors in a module that includes one, named for the
  item's place) Fixpt lists alike too, and kayak-cells refuses that
  listing: ours names the second `name~N`, so that it runs.
  `redefine/compatible.fx` had agreed by chance, its words found by name
  (both closures the newer `f`, and the last value the same).
- **Fixpt's conservatism kept**: whether a procedure is only called
  (`c-calls-only`) is said of no module and no `with`, as Fixpt says.
- **Results**: 132 programs agree (77 before), 116 of them Fixpt's words
  exactly (68). Of the 60 that stopped at a module, 49 are Fixpt's words;
  the loads differ in names only (Fixpt's `lambda@1001:P`, a loaded
  file's places), `letrec-select.fx` by Fixpt's lambda lifting (not yet),
  the made-code programs and `run/conditions.fx` stop later (`case`,
  `letregion`), `convention-read.fx` needs `%fx26-native-convention` of
  tier 1.

## C145. kayak's code generator: regions (2026-10-24)

Next of TODO §31 item 5's gaps, 13 programs: Fixpt's `e-letregion` and
`e-rlambda` arms (`compile-exps.fx`). A `letregion` or `letfreeze` makes
nothing at run time; a `letrena` or `letreap` binds its region in a slot,
entered by `%region-enter` (`%reap-enter`) before the body and left by
`%region-exit` after it, the body so not in tail position. An `rlambda`
(`(rlambda r (param …) body …)`) makes its closure in its region:
the region, the values it captures, its word as a literal, then
`%region-closure`. Whether a procedure is only called is said through a
region's body as not in tail position, as Fixpt says it. 142 programs
agree (132), 120 of them Fixpt's words exactly (116); most of the others
differ by Fixpt's lambda lifting (`sum-to/build@382` called with its
closure as a literal), not ported yet.

## C146. kayak's code generator: prompts (2026-10-24)

Next of TODO §31 item 5's gaps, 8 programs: Fixpt's `e-prompt` arm.
`(prompt tag body handler)`: the tag, the handler, the body as a
procedure of no parameters, then the `prompt` routine, which runs it
under the tag. Whether a procedure is only called is said of the body as
of a lambda's (`c-calls-only-inside`), as Fixpt says it. 153 programs
agree (142), 124 of them Fixpt's words exactly (120); the prompt programs
that are not differ by Fixpt's specialization (`climb-n@lambda@1466`), its
middle phase, not ported yet.

## C147. Tier 6 allocates as tier 5 does (2026-10-24)

The user: "then 2" (TODO §34, tier 6's next: allocation). Tier 6 (C138)
took only what never allocates, so that the collector never saw one of
its records. Tier 5 already makes some objects at the heap's top, in
place (`yallocating!`): `cons`, `%region-cons`, a closure of at most
`REGS` values from the registers; where tier 1's `safepoint!` would
collect first or the heap would grow, it gives back. In tier 6 the same
code's way to give back is its way out (C138): the records written,
then on in tier 5 at the same instruction, which then gives back, and
the collector runs with no record live. So tier 6 now has those three
(`y-direct-op?`, its third operand now asked, a closure's kind), and
nothing else changed.
- **Measured** on a Zen 2, cycles, tier 6 before and after (`--regs
  --direct`, five runs each): `lists-region` 209M to 190M, `lists` 201M
  to 190M, `closures` 327M to 315M; `captures`, `fib`, `helpers`,
  `loop`, `tak` the same. The answers are the same.
- **What keeps the rest out**: a register word is tier 6's only if all
  its instructions are, and in the corpus's register code the most
  common that are not are runtime primitives (`%fx26-sched` alone 5,916
  times, then `prim1`, `prim2imm`, `prim2`, `%make-box`, `setfield`,
  `%make-frozen`). **Tried, not taken**: every word tier 6's, leaving at
  each instruction it does not have (its way out is there for each):
  `closures` 2% faster, `captures` 2% slower, the rest the same. What
  would pay is those primitives made in place, as tier 5 makes `cons`.

## C148. Tier 1's scheduler lets go of what it makes (2026-10-24)

The user: "then 3" (TODO §31, tier 1 on a long run, C142's "support long
running programs/services"). What tier 1 makes of the runtime's own is
never let go (C43); at each scheduling it made some:
- the scheduler's lists, which are the machine heap's, copied into KFX
  lists to be sifted (`heap-items`, `kept-of`, `list-of`), and the
  closures that said what to sift (`cancelled?`'s, `sched-op!`'s). Now
  sifted where they are, in the heap, which is collected: `without!`
  and `only!` take how an item is matched as a number, not a procedure
  (an item, its car, its cdr, either, its car among a list's, its car a
  time not after); `pop-any!`, `cell-put!` and the timers due reverse
  and sort in the heap too. `cons!` never collects (the heap grows, its
  values kept), so a list is walked while it is made.
- `%fx26-sched`'s operation 12, the scheduler's tag as a sum, of a KFX
  list of its two fields: now `frozen2!`, as `o-sum-cell` too.
- **Measured** (`kayak-rt`'s `heap-stats`), at a slice of 2:
  `preempt/allocating-short.fx` 31.0 MB to 9.0 MB, `preempt/allocating.fx`
  (ten times as long) 9.0 MB, the same as without a slice: what is left
  is the listing read, once (8.3 MB of characters).
- **Left as it was**: `groups-within` and `task-failed!`, which run when a
  monitored task fails, once for the group.

## C149. A message's copy, by a table, kept from copy to copy (2026-10-24)

Next of tier 1's long-run leaks (TODO §31): `copy-within!`, which copies
each message a process is sent (Fixpt's `copy_within`). It kept what it
had copied as a list of pairs, original and copy, and found each value
by walking it: quadratic in a message's size, and the lists, the
runtime's own, never let go (C43), a message's size each time; its
`letrec` of the fields to copy a closure for each bloblet copied, too.
- **Now**: a table, open addressing, keyed by the value (an address,
  which nothing moves while the copy is made: `cons!` and
  `make-bloblet!` never collect), its copy beside it; the slots used
  noted, to empty them after; what is still to be looked at, and what
  was copied in order, stacks of flat arrays. All are kept from copy to
  copy and grown twice as big when needed, so the runtime holds the most
  one message has needed, not the sum of all. The fields copied by
  procedures of their own.
- **Measured** (`heap-stats`): a program of 2,000 messages, each a list
  of 200 integers, answered by their sum: 53.3 MB of the runtime's own
  never let go, now 14.5 MB, of which the listing read is 8.4 MB and
  most of the rest the string primitives' code points (next).

## C150. Tier 1's primitives of any number of arguments take no list (2026-10-24)

Next of tier 1's long-run leaks (TODO §31): C142 copied a primitive's
arguments into a flat array (`pargs`), but those of any number of them
still made a list of the runtime's own of them (`pargs-list`), never let
go (C43): `%make-frozen`, which makes every product and sum, as well as
`%make-bloblet`, `%region-closure`, `append`, `string-append`,
`string=?`, `string`; and the string primitives went through lists of
code points (`string-append`, `substring`, the comparisons, `string-hash`),
as did those of a list the heap's (`list-copy`, `list->vector`,
`list->array`, `apply append`, `%product-cell`), and `vlambda`'s closure.
- **Now** each reads its arguments where they are (`fields-of-args!`,
  `appended-args!`, `args-joined!`, `string-chain` by index), or walks
  the heap's list (`fields-of-list!`, `copied-onto!`, `list-joined!`); a
  string is made of its length first (`string-made!`), then written,
  code point by code point (`point-set!`, `points-copied!`), and compared
  and hashed by index. Fifteen list helpers no longer used, gone.
- **Measured** (`heap-stats`), a loop making a product, a datatype's
  value and three strings each turn: 20,000 turns 8.9 MB of the
  runtime's own, 200,000 turns 80.9 MB; now 0.9 MB both, the listing
  read. The program of 2,000 messages (C149): 14.5 MB to 9.1 MB, its
  listing read.
- **Left**: `list->bytevector` and `string->utf8`, of lists still;
  `string-text`, a string of the heap's as KFX's, which only a failure's
  message and the value shown use.

## C151. UTF-8, `list->bytevector`, `string-text` by index (2026-10-24)

The last of C150's "left": `string->utf8` and `utf8->string` went
through lists of code points and of bytes, the runtime's own;
`list->bytevector` through a list of the runtime's of the heap's, and a
`letrec` closure made at each call; `string-text`, a string of the
heap's as KFX's (a failure's message, the value shown), made a KFX
string again for each character. Now each counts first (bytes, code
points), makes its result, and writes it by index; `list->bytevector`
walks the heap's list; `string-text` makes its string once, of a list of
its characters. Six list helpers no longer used, gone. A string of one,
two, three and four byte code points, to UTF-8 and back: 11051, as
Fixpt.

## C152. kayak's code generator: bloblets, blobrefs, floats (2026-10-24)

The user: "3 and 4" (the generator's small gaps, then tier 6's
primitives). Fixpt's `e-bloblet` arm (`compile-exps.fx`'s `c-bloblet`):
each bloblet and blobref operation its runtime primitive, with what the
checker found it takes, as Fixpt's `k-datum-operand` gives it, made of
the facts kayak's checker notes already: a field's index (written); a
new blobref's size (`blob-size`, then `%fx26-suffix-fits`); a scalar's
layout code (`blob-scalar`, `i32` 0 to `f64` 5); a flat array's length
and element size, `n << 32 | size` (`blob-element`); a keyed flat sum's
code, its key's width, 8, its variants' offset times 256
(`blob-variant`: kayak's checker names a variant by a number alone, so
the sum is keyed). An `extract` of a blobref's field, a blobref at its
offset (`blob-offset`). A float literal, `#f64x` and its bits, as
Fixpt's listing writes it. Not yet: a `tagcase` of a blobref to a flat
sum. 165 programs agree (153), 134 of them Fixpt's words exactly (124).

## C153. kayak's code generator: `case` and `typecase` (2026-10-24)

As Fixpt's parser expands them (`parser-exps.fx`'s `parse-case`,
`parse-typecase`). `case`: a variable key as it is, else bound to
`%case-key` by a `let` where the form is; then nested `if`s, a
clause's data each compared by its kind's equality (`=`, `char=?`,
`string=?`, `symbol=?`, `bool=?`), joined as `or` is. `typecase`: a
variable as it is, else bound to `%typecase-value`; each arm its shape's
standard predicate, then its name bound by a `let`; the `else`, its name
if it has one. Not yet: a `case` of many data (Fixpt's `case-tree-from`:
8 integers or more where each clause has one, ...), which Fixpt makes a
search tree of their sort values (`run/case-tree.fx`). The `case`
programs' words differ from Fixpt's only in literals shared (`#0="z"`),
not made yet (C133).
- **And `quote`**, which `typecase` let `datum/quote.fx` reach: a dotted
  list's tail is its last item, a vector `list->vector` of its items'
  list, and a datum not all literals is built, then interned
  (`%fx26-intern-datum`), as Fixpt's words do; before, it agreed only by
  stopping at its `typecase`. Its words are now Fixpt's; tier 1 has no
  `%fx26-intern-datum` yet (TODO §31), so it runs Fixpt's listing as
  little as ours. 170 programs agree (165), 138 Fixpt's words exactly.

## C154. KFX's restrictions noted by the checker, refused by kfx alone (2026-10-24)

The user: "go after the §30 checker work". Of the 428 programs Fixpt
accepts, kayak's checker refused 228. The plan's step 2
(`docs/research/fx26-checker.md`): KFX's own refusals, of what FX-26 has
and Rust cannot say, out of the checker, into the restriction pass that
kfx runs after it and the engine does not, so that the engine's checker
is FX-26's.
- **How**: where the checker would refuse for KFX alone it now calls
  `kfx-restricted`, which notes the message `refuse` would have said, a
  fact of the file `%kfx`, and checks on. kfx's `restrict` takes the
  facts too and refuses at the first noted, where and as before
  (`tests/kfx-stricter` still refused by kfx, each saying where).
- **First moved**: a constant whose value has an effect past reading
  globals and allocating (26 programs): KFX makes a constant at its first
  use, FX-26 where it is defined. Not moved, being FX-26's own: a loaded
  file's value made impure (`modules/load-stateful.fx`, which Fixpt
  refuses too, came out accepted when it was tried).
- **Measured** (`compare-checkers.py`): refused 228 to 209, the same 84
  to 92; of the 336 Fixpt refuses, kayak still refuses all.

## C155. Reals with their point first or an exponent, as FX-26 reads them (2026-10-24)

Fixpt's reader takes a base-10 token with a point or an exponent as an
`f64` by Rust's syntax (`fixpt-read`'s `parse_real`): `.5`, `-.25`,
`1e-3`, `2.5e3`, `1E6`. kayak's reader took neither, `.5` a name and
`1e-3` refused. Now both are reals (`point-first-real?`,
`exponent-real?`), and kfx's restriction pass refuses them (C154), KFX
writing a real into Rust as it is written. The engine's value of one
with an exponent: its digits times ten to its exponent, rounded once,
where both are exact in an `f64` (digits up to 2^53, ten up to the
22nd), which is every one Fixpt's programs write; past that, not read
yet, said so (`1e400`). Refused 209 to 205.

## C156. The checker: `list` as a value, `oneshot`, and other names FX-26 has (2026-10-24)

Of the names Fixpt's programs use that kayak's checker had not, the
small ones, each at Fixpt's type (`fixpt check` of the name alone):
- `list` as a value, `(poly ((t type) (r region)) (vsubr (alloc r) t
  (listof t r)))`; with it, a `vsubr` fitted its element and result
  first, then its effect, and `apply`'s result fitted to what is expected
  where it is, so that a region its procedure allocates at is solved
  before its effect is asked. Made at `acyclic` (an allocation there, of
  data frozen from its making, which nothing sees) is within any effect.
- `call-with-oneshot-continuation`, and its continuation's type,
  `(oneshot T A D R)`, as a composable one's (C60) in every rule.
- `datum-int?`; `rmake-array`, `(rmake-array p n x)`, as `rcons` and
  `rnew` are made, at a region within a place.
- The engine's `--check` writes nothing of a `define-convention`, as
  Fixpt writes nothing (it had checked it again as an expression).
kfx's restriction pass refuses each of these, KFX having none of them
yet (C154). Refused 205 to 192; the same 95 to 100.
- **Not yet, found on the way**: an untyped definition of a `poly`
  value, `(define f list)`, which Fixpt generalizes; `cwcc`, whose calls
  Fixpt checks where they are, with a rule for its termination; the
  `convention` form and the `cellular` convention; `letfreeze`, with
  `(const p)` regions.

## C157. Sizes, first: what a test shows of integers, and a `nat` difference (2026-10-25)

The first of FX-26's sizes (Fixpt's `docs/research/sizes.md`, N5b and
N5c; `sizes.rs`, `check-test-facts.fx`, `check-logic.fx`): the facts a
comparison shows in each branch, and a difference that is a `nat` where
they show it is not negative. Ten of Fixpt's programs were refused at
`(- n 1)` where a `nat` is expected.
- **Facts**: a linear expression of locals of type `int` or `nat`
  (literals, such locals, `+`, `-`, a literal times one), and that one
  is `= 0`, `>= 0` or `/= 0`. `(= a b)`, `(< a b)`, `(<= a b)`, `(> a b)`,
  `(>= a b)` and `(zero? a)` give one to each branch, through `not`,
  `and` and `or` as a union's narrowing does (C68), whose machinery
  carries them: a narrowing's environment, the fact a binding of no
  name a program writes. A fact holds while no name it mentions has
  been bound again.
- **Deciding** whether the facts in scope, and each `nat` local's
  `n >= 0`, show `e >= 0`: by Fourier-Motzkin elimination over the
  rationals, sound for the integers, not complete (that `e <= -1` has no
  solution); a `/= 0` fact split into `>= 1` and `<= -1`, both to show it.
- **`(- a b)`** where a `nat` is expected, the facts showing `a - b >= 0`,
  is a `nat`. Only where one is expected, so that what is found
  elsewhere is printed as Fixpt prints it.
- **Measured**: refused 192 to 187, the same 100 to 104; of the 336 Fixpt
  refuses, all still refused.
- **Noticed**: `regen.sh` failed once at its second generation, its
  first's memos (C54) written by the same compiler before a change to
  `check-exprs.kfx`'s module; with them deleted, the fixed point came at
  once. Not chased further here: `incremental.py` compares memos' compiles
  with fresh ones in the check.
- **And**: Fixpt running the compiler (stage 0) refused a `(list ...)` in
  a `map`'s lambda that kfx accepts, Fixpt finding it at a fresh region
  where an `acyclic` list was wanted: written `(the strings (list ...))`.
  kayak's own words, its checker refusing less: 187 programs agree (170),
  145 Fixpt's exactly.

## C158. Sizes: `(nat s)`, `(nlist T s)`, size binders (2026-10-25)

The user: "Go ahead with sizes". Fixpt's `docs/research/sizes.md`, as
kayak's checker can say it:
- **Types**: a size is `finite` (some size not known), a size variable
  (a `poly`'s binder of kind `size`, `#n` among a scheme's variables, or
  an inference variable), or linear in them (`%sz`: a constant, each
  variable's coefficient, as types so that instantiating and solving see
  the variables; one variable alone is itself). `(nat s)` is exactly the
  size `s`, `nat` itself `(nat finite)`; `(nlist T s)` a list of exactly
  `s` elements, frozen.
- **Fitting**: `(nat s)` below `nat` and `int`; sizes equal as the facts
  in scope show (C157), or the one wanted `finite`, an inference
  variable alone solved, to `finite` too, or one in a sum solved to make
  the two equal, its size shown not negative (`(nlist t (+ n 1))` where
  `m >= 1`: `n = m - 1`). A list of a size below `(listof T acyclic)`, a
  list at `acyclic` below one of size `finite`; `nil` below one of size
  0; a pair below one of size `s`, its tail of `s - 1`; a pair whose
  tail is a list of a size, at `acyclic`, as that list (C68).
- **What is learned and used**: `(null? xs)` of a list of size `n`, `n =
  0` where it holds and `n >= 1` where not; a `(nat s)`'s value is its
  size in the facts. A literal, a sum or a difference where a `(nat s)`
  is wanted, shown to be `s`; `cons` where a list of size `s` is wanted,
  its tail of `s - 1`; `cdr` of one of size `n`, of `n - 1` where `n >=
  1` is shown, else `finite`; `length`, a `(nat n)`. Termination: a list
  of a size's `car` and `cdr` are its parts, as a list at `acyclic`'s.
- **KFX**: its restriction pass refuses sizes (`tests/kfx-stricter/
  sizes.kfx`).
- **Measured**: refused 187 to 182, the same 104 to 109; `nat-basics`,
  `nlist-basics`, `nlist-listof`, `nlist-pair`, `solved-by-test` print
  as Fixpt prints them. Not yet: a size binder in a `plambda` or a
  `proj` (`finite-one-arg`), `poly` types written in a type (C107,
  `map`, `append`), `confirm-length`, `narrayof`, `const` regions.

## C159. Sizes' binders: poly types of types and sizes, rank 2 (2026-10-25)

Fixpt's `sizes/map.fx`, `append.fx`, `finite-one-arg.fx` write a size
binder where C107 took regions alone: a `define-type` of a `poly`, a
`letrec` binding of one, a `proj` of a `letrec`. C107 held to regions
because KFX's Rust erases them; the engine's checker need not.
- **A poly type written in a type** binds regions, types and sizes
  (effects not yet), each named apart by where it is written with its
  kind's mark (`'r@P`, `t@P`, `#n@P`), printed with its kind. A
  `plambda` where one is expected, a `lambda` where one is (as the
  `plambda` of the type's own binders), a `proj` of any expression of
  one (each argument read by its binder's kind), a call of a value of one
  (instantiated, each binder a new variable); a top-level `plambda` of a
  type an abbreviation names checked whole, one whose body is no lambda
  checked as that expression; a `letrec` binding of one, a `plambda`
  too, its termination asked of the poly's body.
- **A binder named again inside**, shadowing, taken where it is no
  region's (KFX's refusal kept for regions).
- **`finite` for a size binder** only where it is the size of one
  argument, itself (Fixpt's rule; `sizes/finite-inside.fx`, which Fixpt
  refuses, came out accepted until it was kept): noted, `not-finite`, as
  the scheme is instantiated.
- **A size solved for in a sum** of more than one variable: the first,
  whose not being negative is asked once the solution has no variables
  left (`append`'s `(+ n m)` fitted before its arguments).
- `(list e ...)` where a list of a size is wanted: each element, and
  the size shown to be how many.
- **KFX's**, each noted for kfx's restriction pass (C154), not refused
  here: a `plambda`, `lambda` or `proj` of a poly type of types or sizes;
  a procedure that a process may hold holding what Rust's `Sync` is not
  (C102, a rule of Rust's threads, not FX-26's).
- **Measured**: refused 182 to 177, the same 109 to 112; of the 336 Fixpt
  refuses, all refused. `map` and `append` now differ from Fixpt only in
  printing their abbreviations' names, which kayak's checker expands
  (§30's printing).

## C160. `confirm-nat`, `confirm-length`, `acyclic`; a list of `nat`s (2026-10-25)

Fixpt's forms that test at run time and say, in their body, what the
test showed (`sizes.md`'s `confirm`): `(confirm-nat e (x body) else)`,
`x` a `nat` there; `(acyclic e (x body) else)`, a list at `acyclic`;
`(confirm-length e n (x body) else)`, a list of size `n`, a literal or a
value whose size the facts know (a `length`, C158), else `finite`. Each
`e` data, a list no one may write still (`datum/acyclic-not-data.fx`,
which Fixpt refuses, came out accepted until it was asked); each
checked as an `if`'s branches, the body's `x` bound. kfx's restriction
pass refuses them, KFX having none.
- **A list of `nat`s**: C157's checker refused one, "FX-26 types each
  its own `(nat v)`", which is so of literals, an `int` each; a call's
  or a name's `nat` is a `nat`, and Fixpt finds `(listof nat r)`.
- Refused 177 to 174; the same 112 to 114.

## C161. `const` regions and `letfreeze` (2026-10-25)

Fixpt's `docs/research/acyclic-regions.md` and `places-and-regions.md`:
data frozen into a place, `(const p)`, which nothing may write any more,
and may be cyclic, having been written before; `(acyclic p)` the same
and never written; `acyclic` below `const`. kayak's checker had
`acyclic` alone.
- **`const`**, and `(const heap)`, `(acyclic heap)` (another place: not
  yet): a region constant. Reading it is pure; mutable data may not be
  at it; a list at it is covariant, as at `acyclic`; `acyclic`, and a
  fresh list's region, below it. `acyclic` and `confirm-length` (C160)
  take a list at `const` as data.
- **`(letfreeze r body ...)`**, `(letfreeze (r heap) body ...)`: a
  region for the body, masked; the value leaves with `r` frozen, at
  `acyclic` if the body never wrote or initialized `r`, else at `const`;
  refused if its type keeps a procedure that writes `r`
  (`regions/freeze-a-writer.fx`).
- **A call that writes frozen data** refused, its latent effect at
  `acyclic` or `const` (`regions/write-frozen.fx`): kayak's checker had
  taken writes at `acyclic` as nothing, sure that no data that may be
  written is there, which a poly writer's instance (`set-car!`) is not.
  Both programs Fixpt refuses came out accepted until they were.
- **KFX**: its restriction pass refuses `letfreeze` and a `const` region
  in a type or an effect (a name `const` it takes, as
  `tests/kfx/rust-keywords.kfx` has one).
- Refused 174 to 169, the same 114 to 119.
- **Noticed**: kfx crashes (`car: nil`, `item-value`) on a `define-rec`
  member that is no `(name type value)` (a `define-rec` written inside
  another), rather than refusing it.

## C162. Specifications in procedure types (2026-10-25)

Fixpt's `check-specs.fx`, `check-logic.fx` (and `specs.rs`, `logic.rs`):
`(subr E ((x T) ...) R (requires P) (ensures r Q))`, propositions of a
procedure's `int`s, `nat`s and `bool`s and of `pure` global procedures'
values. FX-26's checker reads them, prints them, and holds a procedure to
the specification its type expects; it proves no body (`fixpt verify`
does), so `specs/wrong-ensures.fx` is accepted.
- **A parameter named**, `(x T)`: a list of two whose first is no type
  form's, keyword's, type's or base type's name, as `k-param-name`; KFX's
  own type heads (`bytevectorof`, `addr`, ...) counted among the type
  forms, so that `(bytevectorof r)` stays a type.
- **A specified type** is its procedure's wrapped,
  `(ty-data "%spec TEXT" (P))`, `TEXT` the specification written plainly
  (parameters by number, the result `(res)`): the types' walkers see the
  procedure, and two specified types are alike where their texts are.
  The wrapper is unwrapped where a procedure is: a call, a lambda checked
  against it, `proc-type?`, the emitter's types.
- **Behavioural subtyping** where either is specified: the procedures
  fitted, then `a` within `b` if `b`'s `requires` entails `a`'s, and
  with `a`'s `ensures` entails `b`'s. **The prover**: the connectives
  split into cases (at most 10 deep), relations linear inequalities over
  the parameters, the result, and an unknown for each product of
  unknowns and each logic function's value; refuted by C157's
  Fourier–Motzkin. Sound, not complete, as Fixpt's.
- **A lambda** whose type's `ensures` says what a test proves
  (`(implies r P)`, `P` a relation of parameters and literals) refused,
  as Fixpt's `k-claimed`: a branch on its call would trust what its body
  is not checked to prove.
- **KFX**: its restriction pass refuses a `subr` type with clauses or a
  parameter named (`tests/kfx-stricter/specs.kfx`, `named-params.kfx`).
- Refused 169 to 143, the same 119 to 139; the specs directory 34 to 8
  refused (an operator's latent bool, `(bool (then ...))`, `->` types,
  `assume`), 6 more differing only in abbreviations' names (§30.1).

## C163. What a test proves, in a procedure's type (2026-10-25)

Fixpt's `check-proving.fx`, `check-test-facts.fx`: a `bool` result written
`(bool (then P ...) (else Q ...))`, propositions of the parameters by
number (`(shape i S)`, `(acyclic i)`, `(nat i)`, `(length i j)`,
`(< a b)`, `(<= a b)`, `(= a b)`, `(not ...)` of a shape or an `=`; terms
a parameter's number, `(length i)`, `(lit k)`), is C162's specification
of the result, `(and (implies r (and P ...)) (implies (not r) (and Q
...)))`, and is printed back so where an `ensures` has that form.
- **The primitives** take Fixpt's types: `=`, `<`, `>`, `<=`, `>=`,
  `zero?`, the shape tests (`int?`, `pair?`, ...), so that an alias at
  such a type is one (`specs/latent-within.fx`); `assume` and `assert`,
  for the harnesses. Not `nat?`: kfx's own source defines one, which a
  primitive's name would refuse.
- **A call in a test** of a procedure whose specification says what its
  result proves gives each branch its facts (C157), of the arguments'
  sizes, and narrows a local of a union by a shape (C62), as a
  primitive's test does by its name.
- **`(-> (T ...) R)`** with a specification or a `(bool ...)` result:
  `(subr pure ...)` as well; kfx's `imports-in` expands only the plain
  one.
- **KFX**: its restriction pass refuses a `(bool ...)` result (of
  `subr` or `->`), and `assume` and `assert`.
- Refused 143 to 133, the same 139 to 145. Of the 150 that differ, most
  differ by abbreviations' names (TODO §30.1), the largest group left.

## C164. Printing as Fixpt prints: abbreviations by name (2026-10-25)

TODO §30.1. Fixpt's types keep the abbreviations they were written with,
so `fixpt check` prints `(subr pure (ints) bool)` where kayak printed
`(subr pure ((listof int acyclic)) bool)`. kayak's checker expands an
abbreviation as it reads it, and keeps doing so for checking.
- **For printing** (`kayak-fx26 --check`, `fx26/start.kfx`) a
  definition's written type is read again, the marker `%keep-names`
  among the type variables, each abbreviation then kept by name,
  `(ty-data "%named N" (expansion arg ...))` (`named-as`); the checker's
  own type is printed, and each such abbreviation's expansion in it, a
  parenthesized type, replaced by its name, the longest first. The
  checker's type, not the written one, so that what it adds (a global's
  reads in a latent effect) is printed. An expression's line, a call,
  takes the names of its callee's written type.
- Where the reader inspects what it read (a blobref's owner and flat, a
  flat array's element, `flat?`), it looks through the name.
- **Also**: a region binder printed without its `'` (`(listof int r)`);
  `(fields)` for no fields; a flat array as `(flat-arrayof T n)`. C42's
  rule had looked for a `'` in a printed type; it tests the type now
  (`region-var-in?`).
- The same 145 to 201, differing 150 to 94; no program newly differs,
  none changes being accepted. Left of what differs: module types (61),
  a fresh region's name and allocation (`@r.1`), effects kayak's
  inference gets less precisely, constructors' lines of recursive
  datatypes.

## C165. Printing as Fixpt prints: modules, and base types named (2026-10-25)

TODO §30.1, after C164.
- **A module's line**: `kayak-fx26 --check` had printed nothing for
  `(define m (module ...))`. It prints Fixpt's `define m : (moduleof
  (desc t T) ... (abs t type) ... (val x T) ...) ! E`: the descriptions
  first (a type, a family as `(dlambda (params) T)`, an effect), then the
  values, each in the order written, an included module's in its place,
  a hidden one left out; `E` the included modules' reads. Each type is
  the item's written type, read again with names kept (C164), the
  module's own names resolved by the printer (a parameterless
  abbreviation an opaque type of its name, one of a family's application
  expanded, as Fixpt's is), its module prefix (`m..`) taken off. A
  module of what this does not print (a datatype, an `extend`, a type
  `->*` finds, a module type, a module from a loaded file) keeps no
  line, as before: nothing is printed rather than something wrong.
- **Base types named**: Fixpt's types are shared, and an abbreviation of
  a base type, `(define-type num int)`, names `int` wherever it is
  printed, the latest such, even before it (Fixpt prints after checking
  all); in a module, the module's own, visible, for its values, and
  those before a description for its body. The printer does likewise,
  token by token.
- The same 201 to 222, differing 94 to 73; none newly differs, none
  changes being accepted. Left among module lines: a module's own
  effect (values made with `alloc` or `spin`), Fixpt naming a structure
  by the latest abbreviation of it in a module (`(desc pair point)`
  names `point` `pair`), datatypes (`mu`), nested and loaded modules.

## C166. Printing as Fixpt prints: module types' names, fresh regions, settled effects (2026-10-25)

TODO §30.1, after C165.
- A module type, `(define-type pairs (moduleof ...))`, is printed by its
  name (it had been `%module pairs`).
- An expression's list at a fresh region (C36) is printed at `@r.1`, its
  effect allocating there, as Fixpt names and makes one.
- A definition's effect, and an expression's, printed with their regions
  as solved (`resolve-atom`), an allocation at `acyclic` or at a region
  never solved left out: Fixpt's `(define ys (listof int acyclic) ...)`
  makes its list purely.
- The same 222 to 239, differing 73 to 56; none newly differs.

## C167. A bloblet's width (2026-10-25)

Fixpt's `TODO.md` §77 (`bloblet/width*.fx`): a bloblet of more fields is
taken where one of fewer is wanted, its fields counting from the suffix
so that each keeps its offset; the region and the fields wanted the
same. What may be forgotten is what the wider type need not say: no
procedure (a knot through the store), no type variable (one could be a
procedure), and no region but the bloblet's own, `acyclic` and `const`
(storage the narrower type would not mention, `width-escape.fx`,
`width-exact-region.fx`). Without that rule the first try accepted four
programs Fixpt refuses (K1), each its own case of it.
- **KFX**: refused (`kfx-restricted`): a bloblet's Rust type is its
  fields' (`tests/kfx-stricter/bloblet-width.kfx`).
- Refused 133 to 131, the same 239 to 241.

## C168. `cwcc` (2026-10-25)

FX-26's `call-with-current-continuation`, `cwcc`, at Fixpt's type
`(poly ((r region)) (poly ((t type)) (poly ((e effect)) (subr (maxeff
(comefrom r) e) ((subr e ((subr (goto r) (t) void)) t)) t))))`, is a
primitive of kayak's checker (as one poly of three binders, which a
chain of `proj`s instantiates, C60), with Fixpt's rules for it
(`check-calls.fx`'s `k-may-spin?`, `soundness-findings.md` F3, F9):
- **`spin`**: a continuation called after `cwcc` returns comes back to it
  as often as it is called; only a receiver `(lambda (k) ...)` whose `k`
  is only called (never inside a procedure made there) and which itself
  calls no `cwcc` needs none.
- **Masked**: such a receiver's continuation is called only while `cwcc`
  runs, so its `comefrom R` and `goto R` stay within the call
  (`terminate/cwcc-escape.fx`); and a call of `cwcc` itself, its region
  one nothing names, fresh, masks its control there
  (`bidirectional/cwcc.fx`). The printer leaves out control at
  `acyclic`, where a region nothing names settles.
- **KFX**: refused, as `call/cc` is (C45;
  `tests/kfx-stricter/cwcc.kfx`).
- Refused 131 to 126, the same 241 to 246.

## C169. `race` (2026-10-25)

FX-26's `(race e ...)` (Fixpt's `check-processes.fx`, `k-race-te`), which
kayak's checker had refused as a form KFX has not (C108): its branches
run at once, the first to end gives the value, the rest are cancelled.
- Its type the first branch's not `void` (one that only fails loses,
  `monitors/race-lost.fx`), each branch fitted to it; its effect theirs
  and `nondet`.
- A branch must be cancellable, doing nothing another could see (no
  `write`, `init`, `send`, `io`, `goto`, `comefrom`, or effect variable);
  no two may interfere (`par`'s `clash-of`).
- **KFX**: still refused, now by `kfx-restricted`, with C108's message
  (`tests/kfx-stricter/race.kfx`).
- Refused 126 to 120, the same 246 to 252.

## C170. First-class modules, in the checker (2026-10-25)

Fixpt's `docs/research/first-class-modules.md` (M1, M4, M5): modules as
values, typed by module types, `with` and `select` on them, dependent
procedures. KFX's modules stay Rust's (C19, C24): every new form is
FX-26's, `kfx-restricted` or refused by the restriction pass.
- **A module type**, `(ty-data "%mtype" (item ...))`, structural: `%abs t`
  (its type, `ID..t`), `%desc t` (its type), `%edesc e` (an effect),
  `%val x` (its type). `(moduleof ...)` reads as one anywhere; a
  `define-type` of one KFX's traits cannot be (an effect described, or
  parameters) is one too (`trait-moduleof?`). Printed as Fixpt prints:
  types first, then values.
- **Named for its binding**: a module bound to `p` (a local, a global, a
  module's item) has its abstract types renamed `p..t`, equal to no other
  (Fixpt's `k-name-module`; a `let`-bound module opaque,
  `modules/let-opaque.fx`); none may leave the scope that binds `p`
  (`escape.fx`, `inner-escape.fx`).
- **`(module item ...)` as a value**: its types first, then its values,
  each typed one in scope for all and an untyped one not before it is made
  (`capture.fx`); a datatype a sum (a recursive one waits for `mu`); a
  name given twice refused; an `include` of a module value takes its
  items, its abstract types described (`include-abstract-escape.fx`).
- **`with`** of a module value opens its values, types and effects;
  **`(select m t)`** of one, and an effect's; a module defined at the top
  level is a value too, of the type its items give.
- **Fitting**: width, an abstract type given abstract or described, each
  description the same, each value fitting (M4).
- **Dependent procedures** (M5): a procedure type's parameter named, of a
  module type, its types `(select m t)` in the parameters after it and the
  result, `$i..t`, printed `(select $1 t)`; a lambda's own parameter names
  them inside; a call takes them from its arguments.
- C24's refusals of module parameters other than a top-level procedure's
  are `kfx-restricted` now; the termination check walks a module value's
  items' values.
- Refused 119 to 103, the same 252 to 260. Left: loaded files as values,
  `extend` as an expression, `include` of a value in a module defined at
  the top level, module types' families applied, printing.

## C171. Modules: loads, `extend`, and module values at the top level (2026-10-25)

After C170.
- **`load-input` of a file of one expression** (FX-26's M7, not only a
  module): its value a global defined before the form that first loads
  it, `%shared:input:PATH` as Fixpt names it, one for all its loads, not
  printed; KFX's restriction pass refuses it.
- **A file of module parameters given them but not called**, `(proj
  (load-module "f") r ...)`: the procedure that makes it, `(lambda ()
  ((proj ...)))`. A loaded file's making must be pure (kept), but a file
  of module parameters, made for each call in FX-26: there it is KFX's
  rule alone (`kfx-restricted`).
- **`(extend e0 e1)` as a value**: `e0`'s items `e1` does not name, then
  `e1`'s; one of `e0`'s kept whose type mentions an abstract type of
  `e0`'s that `e1` replaces refused (`extend-abstract-replaced.fx`).
- **A module defined at the top level that KFX's cannot be** (it includes
  a module made when the program runs, or has a module of its own that
  includes one not named): read as a module value, `(%module-value ...)`
  (C170).
- `(proj (with m x) a ...)` is `(with m (proj x a ...))`; `heap` names the
  heap's region; a global may shadow a primitive at another type
  (FX-26's; KFX names none; `heap` refused by KFX's restriction pass, its
  Rust from a `with` of shadowing families wrong); a `with` in a module's item of a module that
  includes a loaded one does not take the included names for the item's.
- **The generator** (`fx26/cg.kfx`, C133) declines a module made as a
  value and a `with` of one (facts `module-value`, `with-value`), which it
  has not yet, rather than make wrong words.
- Refused 103 to 93, the same 260 to 263. Left of modules: `load-layers.fx`
  (a module's effect read from an item made before it, which the module
  mechanism of KFX reads first), `types-mutual.fx` (`mu`); and printing.

## C172. Printing modules as Fixpt prints them (2026-10-25)

After C165, for C170's module values.
- A module value's line shows its own abstract types as its own, `t`, not
  `m..t` as the checker names them for the binding; a loaded file's, which
  only it names, the same.
- In a printed module type, a type one of its descriptions describes is
  printed by that description's name (Fixpt's types are shared, and named
  where described): `(val circle (subr pure (int) shape))`.
- A module defined at the top level whose items the written-type printer
  (C165) cannot print (a datatype, a load, an `extend`) is printed from the
  checker's type of it as a value, each datatype a description of its sum.
- A module in a module is a value of its items' type; hidden modules kfx's
  reader made (`made--`) are printed neither as lines nor in effects.
- Base types named (C165) by the abbreviations outside a module, in its
  line, but those its own types shadow; `(define-type t (select m t))`
  names `m..t` `t`.
- The same 263 to 270, differing 72 to 65.

## C173. Higher kinds (2026-10-25)

Fixpt's `docs/fx26.md`, "Higher kinds" (`docs/research/higher-kinds.md`,
Path 1 and Path 2): binders, families' parameters, generative types'
parameters and a module's abstract components of an arrow kind `(=> (k
...) k)`; FX-91's `dlambda`; effect functions. FX-26's alone: the
restriction pass refuses `=>` and `dlambda` (C24's traits are over
types).
- **Kinds**: `type`, `region`, `place`, `effect`, `size`, or an arrow, read
  canonically; a binder of an arrow kind is named `^K^f` (its kind in its
  name, `binder-kind` reads it back; printed `f`). A function to an
  effect takes regions, places, effects and sizes; a function given an
  effect, kayak's checker has not yet.
- **Functions**: a variable of arrow kind; `(ty-data "%dlambda" (binder ...
  body))`; a family, a generative type or `listof`/`pairof` named, not
  applied, `%fam NAME`; a `define-type` with parameters, or of a `dlambda`,
  as a `%dlambda`. **Applied** where known, beta (`fun-applied`, and
  `reduced` after every substitution of a poly's binders); a variable
  applied, `(ty-data "%app" (f arg ...))`, equal only to one of the same
  function to arguments the same. Arity checked against the kind; at a
  `proj`, the function's kind against the binder's (`kind-mismatch.fx`).
- **Inference first-order only**: an application against an application
  finds the function; against a family or generative type applied, that
  type (`infer-generative.fx`); anything else asks for `proj`
  (`cannot-infer.fx`). Two `dlambda`s are equal with their binders named
  alike.
- **Effect functions**: `(define-type rw (dlambda ((r region)) (maxeff
  ...)))` applied in an effect is its body at the regions given; a
  variable of kind `(=> (region) effect)` applied is an atom of its own,
  `~@E|r`, printed `(e r)`, which instantiating the variable with a
  function expands.
- **Modules**: `(abs f (=> (type) type))` and `(abs (f (a type)) type)`,
  an abstract constructor, `(ty-data "%fam ID..f" (KIND))`, applied
  `(ID..f a)`, named for the module's binding as an abstract type is
  (C170); a module value's `define-generative (stack (a type))`, one, its
  conversions polymorphic; a module type with one kind of constructor
  takes no module whose constructor is of another (`abs-kind-mismatch.fx`).
- Fixes found on the way: a `(f a)` among a procedure type's parameters,
  `f` a function, is no parameter named `f` (C162's rule); the plambda
  escape rule looks at the metas that could carry a binder out (the type
  expected, the locals), not every older one, which another plambda's
  binder of the same name had solved; the printer does not read again a
  written type with `select`, `moduleof` or `dlambda` (C164).
- `regen.sh`'s memos went stale three times this round (generation 2 not
  building): cleared, `rm -rf target/kfx-memos-*`, as C157 found; the
  cause is not found yet. And Fixpt running the compiler (stage 0) is
  stricter than kfx about a list's region: `(cons x acc)` where a list is
  wanted needs `(the strings ...)`, and `string->list` gives a fresh
  region where `string-chars` gives `acyclic`; kfx took both.
- Refused 93 to 83, the same 270 to 276; K1 still 0. Left of higher
  kinds: recursion through an application (`fix-through-pair.fx`), which
  waits for `mu` (task 3).

## C174. Recursive types: `mu` (2026-10-25)

Fixpt's `docs/research/recursive-subtyping.md`: a type abbreviation may
mention itself, and `(mu t T)` writes a recursive type with no name;
equi-recursive, a named type and its `mu` each a subtype of the other.
FX-26's alone: the restriction pass refuses `(mu t T)` where a type is
written (a definition's, a `the`'s, a binding's: `mu` may name a
procedure elsewhere, as `run/int-fast.fx`'s does, which the check
caught) and a `define-type` whose body reaches itself through others'
(a Rust type alias cannot; a datatype can), as the checker did before;
and a comparison that unfolds a `mu` notes `kfx-restricted`.
- **Representation**: `(ty-data "%mu" (X body))`, `X` a variable `%mu
  NAME`; an abbreviation's own mention in its expansion is `X`
  (`%expanding` as before), the expansion wrapped by `mu-wrapped` when it
  mentions it. A family may mention itself only with its own parameters,
  in order (`family-growing.fx` refused); after substitution, reduced
  (C173), so `fix-through-pair.fx`'s `(f (fix2 f))` at the identity is a
  list.
- **Grounded**: a recursion under no constructor is refused, "a recursive
  type must be built from a constructor": alone, in a `poly`'s body, or
  given to a type function not known, which may be the identity
  (`type-cycle-of-names.fx`, `polyonly.fx`, `fix-refused.fx`).
- **Compared** (`fits-shapes`): either side a `mu`, the pair assumed (a
  fact `%mu-assumed A <= E`), each unfolded once; a pair met again holds
  (Amadio and Cardelli).
- **Unfolded where a shape is asked** (`walk-mu`): `extract`, `tagcase`,
  `sum`, `product`, a `vlambda`'s expected, a bloblet made, read or
  written. Not in `walk`: recursive walks over types would not end.
- **Shapes**: a `mu`'s is its body's; a union's member that is the
  abbreviation's own mention, its body's read again (`mu-member-shape`);
  a bloblet's, FX-26's `bloblet`, only there (`member-shape`), since a
  union of a bloblet is no KFX union (`native/bloblet-tree.fx`).
- **Transmissible**: as its body is, its variable taken as itself
  (`processes/deadlock.fx`).
- **Printed** as Fixpt prints: `(mu %k ...)`, `k` the depth of the type it
  stands for, counted in the constructors around it (`unparse.rs`); the
  printer renumbers each line (`mu-numbered`, fx26/start.kfx). A `mu` of
  `(union nil (pairof E X R))` is `(listof E R)`.
- Refused 83 to 73, the same 276 to 283; K1 0 (one breach on the way,
  `(define-type a a)`, its `define-type` read without the wrap). Left:
  `run/well-founded.fx` (a `sum` with no type expected, not `mu`'s);
  printing where Fixpt names a type by an abbreviation of it
  (`parametric-rose.fx`, `types-mutual.fx`, `types-in-any-order.fx`); a
  fresh region's name in an expression's type (`family-knot.fx`).

## C175. General flat arrays (2026-10-25)

Fixpt's flats (Q6, its TODO §77) past KFX-0's: FX-26's alone, each
refused for KFX by the restriction pass or a `kfx-restricted` note.
- **`(flatarrayof T R)` of any `T`** (Fixpt's generative `flatarrayof`),
  `ty-flat` as before; `(flatlayout T)`, `%flatlayout`; the primitives
  `make-flatarray`, `flatarray-ref`, `flatarray-set!`, `flatarray-length`
  and the layouts `i32-flat` ... `f64-flat`, at Fixpt's types. KFX: its
  `-i64` prelude's alone (the names and a non-`i64` element refused).
  Found on the way: `(flatlayout t)` among a procedure type's parameters
  read as a parameter named `flatlayout` (C162's rule): a type's word
  now, as `flatarrayof` was.
- **Flat sums with no key**, `(sumof (a F) (b G))` where a flat is: its
  tag a byte indexing the owner's fields, which hold the labels
  (`%flat-sum` keyed `%fields`, `as-flat`); `blobref-variant!` of
  `'label`, `tagcase` arms by label; Fixpt's operand code `w + 8·numeric
  + 256·p` to the engine (`fields` in the fact: no 8). A variant may not
  be unsized.
- **Arrays to the end of the suffix**: `(arrayof F n)`, `n` `finite`, a
  size variable or a sum of sizes; `blobref-length`, a `(nat n)`
  (`nat` for `finite`); the compilers told `FLAT_TO_THE_END`. Only a
  product's last part may be unsized, no array's element; a new
  blobref's length is `finite`, not a variable's. Fixpt names a `finite`
  length where its blobref is bound: here, where the variable is used,
  `#%len NAME` (`lengths-named`), so that a procedure polymorphic in the
  length is given it, and two arrays' lengths are not one; a literal's
  length is no `finite`'s (`flat-literal-not-to-end.fx`). A `proj`'s
  size argument read as a size.
- **A K1 hole found by a test**: in checking mode the blobref and
  bloblet forms that read (`blobref-offset`, `-get`, `-index`, ...) gave
  their own type and were never compared with the one expected, so an
  `int` was taken for an `i64`, in KFX too. Now only the forms the
  expected type makes (`make-bloblet`, `rmake-bloblet`, `blobref-at`,
  `blobref+`) take it; the rest are synthesized and compared
  (`tests/kfx-refused/blobref-offset-int.kfx`).
- The cells generator (`--ours`) declines a `tagcase` of a flat sum
  (`blob-tagcase` noted), which it has not; it had met none before.

## C176. An index of the checker's bindings (2026-10-25)

The Fixpt team's proposal (TODO §35): `find-bind` walked all of
`cv-binds` from the innermost, a chain of dependent loads per element,
and a lookup that finds nothing walks it all; 12% of stage 0's cycles
and half its L2 misses on their measure. Both parts, at once:
- **An index**: `cv` gains `bix`, a persistent AVL tree keyed by name
  (`string<?`), each name's bindings innermost first. `cv` is passed
  functionally, so a scope's tree is its own and needs no undo. Where a
  `cv` is made with its list grown (`with-binds`, `with-type-binds`, a
  `with`'s opened bindings, the incremental compile's), the tree grows
  (`bix-adds`, in `bind-onto`'s order); where the list is rebuilt (a
  loaded file's bare scope, a module's start and end), the tree is
  rebuilt (`cv-reindexed`, rare).
- **The markers apart**: `%module`, `%loaded`, `%instance` and
  `%toplambda` also kept in a short list of their own (`marks-adds`),
  read for those names; they stay in the list as well, so that what walks
  it sees what it saw.
- **`scope-bind name v`** answers as `(find-bind name (cv-binds v))` did;
  64 of the callers moved (all that passed `cv-binds`). `find-bind` stays
  for those given another list (a module's opened bindings, the REPL's).
  Named so because `bound` and `in-scope` are locals' and terminate.kfx's
  names already.
- **Shown the same before it was trusted**: built first with every lookup
  answering from both, the index and the list, and stopping if they
  differed: the compiler's own fixed point, every program of Fixpt's
  corpus through the engine, and the full check, with no difference;
  then the list's walk taken out.
- **Measured** (this machine, one core, `perf stat`, two runs each, the
  pinned Fixpt 8f76d65, not the team's ae397dc8): stage 0
  (`scripts/stage0.py kfx/kfx.unit`) from 102G cycles to 65G (−36%),
  24.8 s to 15.7 s; L2 misses (`ic_dc_miss_in_l2`) 1.04G to 0.56G;
  instructions 264G to 160G. More than the team's 12% because this pin
  predates their self tail calls as loops, so each step of the walk was a
  call. The same output. The Rust kfx compiling itself (the check's
  fixpoint line) 3.88 s to 2.82 s, one run each.
- Not done: names interned to symbols (the team: not needed for the main
  gain); `mangled-clash`, which `with-binds` runs over the whole list for
  a name that might clash in Rust, still a walk (rare: `might-clash?`).

## C177. The pin at Fixpt ae397dc (2026-10-25)

The user: "upgrading your local Fixpt to the latest commit". Fixpt's
`main` has two heads today: the user's checkout, `ae397dc8` (11:09, 15
commits not pushed: x86_64's native convention reading strings and
characters in line, Rust allocating less, the code area's collection
policy, a definition compiled on its own calling itself in a loop), and
GitHub's, `e7cddc0e` (03:22, 8 commits of arm64 work from the Mac).
Neither holds the other. The pin is the latest, `ae397dc8`, fetched from
the user's checkout into `../kayak-refs/fixpt` as C141's was; until
Fixpt merges and pushes, a clone of GitHub's alone has no such commit
(macOS's check with it).
- **Stage 0**, here, one core, two runs: 65G cycles (C176, at 8f76d65)
  to 21.7G; 160G instructions to 46.4G; L2 misses 0.56G to 0.27G; 15.7 s
  to 5.3 s; the same output. The full check 4 min 34 s.
- **The drift, decided against**, each line moved to the pin: in
  `heap.rs` (`cells/heap.kfx`, `kayak-rt`'s `text.rs`), the code area's
  dirty cards and major collections' budget, strings made from code
  points in the heap, allocation sampling, native call-outs without a
  vector, the native front end kept: kayak's heap has no code area to
  collect and its strings are its own; in `prim.rs` (`cells/prims.kfx`),
  `reverse`, `string->list`, `list->string`, `substring`, `string-search`
  and the symbol conversions without Rust's vectors, and `run-word` of a
  native closure: the same answers, Rust's allocation not kayak's, and no
  native closures in tier 1; in `cellular.rs` (`cells/hand.kfx`), native
  closures and the collection budget, likewise; in `docs/object-model.md`
  (`kayak-jit`), when code is collected, which kayak-jit does not; in
  `compile-exps.fx`, `compile-state.fx`, `compile-programs.fx`
  (`fx26/cg.kfx`), a definition compiled on its own by `native_define`
  knowing its lambda, for register code: kayak's generator compiles a
  program whole and has no register code yet.
- **Taken**: `built-in.kfx` (its stamp only); Fixpt's programs, 5 new.
  Of them `native/char-reads.fx` used `char-in?`, a standard operation
  kayak's checker had not: now in its table (FX-26's; KFX refuses the
  name), in tier 1's primitives (`%fx26-char-in?`, `cells/prims.kfx`) and
  the engine's (`fx26/prims.kfx`, last in `prim-list`, whose numbers the
  control primitives' are checked against). Its `symbol-name-hash` the
  engine has not yet: not run there. The checkers' comparison: of 433
  programs Fixpt accepts, 298 the same, 74 differ, 61 refused; K1 0.

## C178. The pin at Fixpt b9dae98 (2026-10-25)

The user's checkout moved one commit past C177's pin, `b9dae986`, "The
heap's memory in huge pages" (Fixpt's heap mapping asks for transparent
huge pages); the user: "Move the pin now". Fetched from that checkout as
before; not on GitHub yet. Its one ported file, `heap.rs`
(`cells/heap.kfx`, `kayak-rt`'s `text.rs`), decided against: kayak's
heap is not mapped by Fixpt's `Words`. No program, built-in or answer
changed. Stage 0 here, one core, two runs: 21.7G cycles to 20.3G
(19.1G of them the user's), 5.3 s to 4.9 s, 68.6K page faults; the same
output. The checkers' comparison at this pin: of 433 programs Fixpt
accepts, 299 the same (C177's `char-in?` counted), 74 differ, 60
refused; K1 0.

## C179. Globals found in a local letrec; termination as Fixpt's (2026-10-25)

The largest cluster of programs kayak's checker refused that Fixpt
accepts (TODO §30, 11 of 60), "a lambda whose effect ... is not within",
was four rules:
- **A local `letrec`'s procedures need not name the globals they read**
  (Fixpt's fx26.md, 2026-09-30): each member checked as though its type
  read any global, the globals its body reads not within its type put
  in it, and again until the group's calls of each other add none
  (`letrec-found`). Where the written types suffice, they are kept, so a
  group that checked before checks the same. (`globals/letrec-reads-found`,
  `processes/self`, `exchange`, `pselect-stop`, `monitors/worker-fails`.)
- **A group whose members call none of each other ends**: the size-change
  criterion (`ends?`) asked for at least one path; with none there is no
  run to go on, as Fixpt has it. A non-recursive `letrec` had cost its
  caller a `spin` (`run/loops.fx`'s `hidden`).
- **An array's length is fixed**: an upper bound of `(array-length a)`,
  `a` bound outside the group, as `string-length`'s, as Fixpt's
  terminate.rs (`sizes/array-walk.fx`).
- **A generative type's conversions are the identity** to the
  size-change check: `(down-tree t)` tracks as `t`, its type not known, so
  that a `tagcase` still proves parts and a `car` not (Fixpt's
  `Tracked::Part` with `ty: None`; `generative/tree-walk`,
  `run/generative`). And a family's conversions print in a `poly` of its
  parameters, as Fixpt prints them (four `generative/` programs now the
  same).
- **Left, one rule of Fixpt's**: FX-87's masking at every subexpression,
  an effect on a region no free variable's type nor the result's names
  erased. `run/icells.fx` (a knot of I-cells at `@p` inside `parity`) and
  `native/raw-slots.fx` (`(car (the (listof int @heap) (cons b nil)))`
  pure in Fixpt: its `cons` at a fresh region, and that masked) need it;
  kayak masks a `letregion`'s, `letrena`'s and `letfreeze`'s region, and
  fresh regions where printed. A change to the effect system as a whole:
  TODO §30.
- Refused 60 to 51, the same 299 to 312; K1 0.

## C180. A placeholder binding is no redefinition (2026-10-25)

Of the four programs refused as "defined again, at another type": two
were not. Where an effect is found (`->*`'s `found-define`, a group's
`found-rec`, `check-define*`), the name is bound for one pass at a
placeholder type, reading itself or `pure`; that binding went through
`global`, whose redefinition rule compared the placeholder with the
earlier definition's type and refused. Now bound plainly for that pass;
the definition itself still meets the rule (`native/redefined.fx`,
`native/raw-args-across-forms.fx`).
- **Left**: Fixpt's redefinition at another type (fx26.md, 2026-09-28):
  every earlier definition using the name, and their users, checked
  again in order, each that checks defined again (and printed so), each
  that does not *broken*, an error only at a later use
  (`redefine/rerun-chain.fx`, `redefine/knot-spin.fx`). Kayak's checker
  refuses at the first fault, with no try, and its engine would have to
  run them again: a change of its own (TODO §30). Until then kayak
  refuses at the redefinition, as before.
- Refused 51 to 49; K1 0.

## C181. Calling conventions as Fixpt's checker has them (2026-10-25)

Fixpt's `docs/research/native-conventions.md`: a procedure type's
convention, `cellular`, `native` or `fx` (either), written
`(subr (conv C) ...)` or left out. Under `fixpt check` now, the
program's convention is `native`: a type prints its convention only
where it differs, so `(conv native)` prints as nothing and
`(conv cellular)` is written out. Kayak kept `native` apart (C116, KFX's
`native-procedure`, a Rust fn), and had no `cellular`.
- **The representation kept**: `%native` and `%fx` wrappers as before,
  and `%cellular` new (`conv-wrapper?`); `%native` printed as the
  program's, unwritten. A native procedure where the program's is, or
  the reverse, fits (the same convention to FX-26's checker), noted
  `kfx-restricted` (KFX's `native-procedure` stays apart).
- **Subsumption**: any convention into `fx`, free, as before.
- **Conversions**: a procedure of one convention given where one of
  another is expected (but `fx`) is converted, as Fixpt's checker
  inserts an adapter, or from `fx` a test of the value's kind: only
  where a value is given (an argument, a binding, a checked result:
  `converted`, before `check-by-synth`'s fit), never inside data
  (`fx-not-cellular.fx`, which a first try at doing it in `fits`
  accepted: a K1 breach, found by the comparison). KFX has none.
- **`(convention C e)`**: `e`'s procedure at `C`; the engine lowers it to
  `e` (its procedures are one convention); KFX refuses it. A `lambda`
  where a `cellular` or `native` procedure is expected is checked as the
  program's and given that convention (KFX's lambdas are its own).
- **Left**: `conv`-kind binders (`(poly ((c conv)) ...)`,
  `conventions/poly.fx`); `native/adapters.fx` needs `cons` as a value at
  Fixpt's type, a `pairof`.
- Refused 49 to 45, the same 314 to 319; K1 0. Tests:
  `tests/kfx-stricter/convention-form.kfx`, `convention-cellular.kfx`.
- The cells generator (`--ours`) declines `(convention native e)`, an
  adapter to native code, its closures being cellular (found by the
  check: `conventions/native-explicit.fx`'s value printed as a cellular
  closure where Fixpt's is native).

## C182. Masking as FX-87's, at Fixpt's forms (2026-10-25)

Fixpt masks an expression's effect at nearly every compound form (its
`mask`; fx26.md: "Masking applies at every subexpression that combines
effects, as in FX-87's reference"): an atom on a region that no free
variable's type names is dropped, unless it allocates or transfers
control at a region its result's type names. Kayak masked only a
`letregion`'s, `letrena`'s or `letfreeze`'s region and a few control
cases; FX-26's checker as Fixpt's needs it all (`run/icells.fx`, a knot
of I-cells private to a procedure, pure; `native/raw-slots.fx`).
- **`masked-res`**, on each compound expression synthesized or checked,
  at the forms Fixpt masks (`masking-form?`): an application, `proj`,
  `if` and `cond`/`and`/`or`, `let`, `let*`, `begin`, `product`,
  `extract`, `sum`. Not a `spawn` (its process's effects cross: a
  first try masked a reply cell's `init`, which Fixpt keeps), nor
  `letrec`, `tagcase` and the binding forms.
- **What is seen**: the regions in the types of the names in the
  expression (`regions-in`, Fixpt's `regions_in`): every name in it, bound
  inside or not, an over-approximation that only keeps more; a datatype's
  fields at its arguments; a procedure's latent effect's regions.
- **Never masked**, kayak's choices toward refusing where unsure: an atom
  on a frozen region (`acyclic`, `const`), on a region not solved yet,
  globals', `spin`, `nondet`, I/O, effect variables; and nothing at all
  where a name's or the value's type could hold storage it does not say:
  a type not solved yet, a generative or abstract type, a module type or
  a module's name (`%opaque`). The last found by the check: a module's
  name had carried no item types, so `prompt-module-goto.kfx`'s `goto`
  was dropped and KFX accepted it.
- **Fixpt's F9, found on the way**: with masking, `terminate/cwcc-captures.fx`
  (refused by Fixpt) was accepted: kayak's `cwcc` rule took a receiver
  that only calls its continuation as escaping only, where Fixpt also
  asks that the receiver capture no continuation (a `comefrom` in its
  effect). Kayak's rule is syntactic: a receiver that names
  `call-with-composable-continuation` or `call-with-oneshot-continuation`
  is no escape only, so the call says `spin`.
- **KFX**: masking is FX-26's and KFX's alike; the full check's corpus,
  its Rust compared and run, is the same, and kfx accepts 194 of Fixpt's
  programs, up from 185, each agreeing when run. Stage 0 20.3G cycles to
  20.6G.
- Refused 45 to 42, the same 319 to 330; K1 0.

## C183. Fixed widths below `int`; sums by width (2026-10-25)

Subtyping FX-26 has that kayak's checker had not, each refused for KFX
(a `kfx-restricted` note) where its Rust has no such thing:
- **A fixed-width integer is an `int`** (Fixpt's fx26.md, "Fixed-width
  integers", 2026-10-08, 2026-10-11): `i8`, `u8`, `i32`, `u32`, `i64`,
  `u64` wherever an `int` is wanted. (`sizes/fixed-width-literals`,
  `native/i64-as-int`, `bytevector-sum`, `run/bytevectors`.)
- **Sums by width**: a sum below one of more tags, each of its tags'
  types below the other's (`recursive/constcov`); and `(sum tag e)` with
  nothing expected is `(sumof (tag T))`, as Fixpt's table has it
  (`run/well-founded`), where kayak had asked for its type.
- **Variance as Fixpt's**: probing Fixpt, references, arrays, I-cells,
  lists, pairs, addresses and products are covariant, parameters
  contravariant, as kayak's `fits` has them; a generative type's
  parameters invariant (`generative/invariant.fx`,
  `lemmas/without.fx`: two K1 breaches width subtyping had opened, now
  each argument fitted both ways).
- **A sum's `else` sees the tags not named** (`bloblet/else-narrows`):
  its name bound to the sum of the variants left. KFX refuses it only
  where the name is used: its own `lib/runner.kfx` names one it does not
  use (the self-compile found it).
- Refused 42 to 35, the same 330 to 337; K1 0. Left among sums and
  pairs: `cons` of a tail that is no list, a general pair, as Fixpt's
  `cons` is (`width-infer`, `run/recursion`, `bidirectional/bounds`,
  `native/adapters`); paths narrowed by a test (`unions/paths-narrow`).

## C184. `cons` of a tail that is no list (2026-10-25)

Fixpt's `cons` is `(poly r (t1 t2) (subr (alloc r) (t1 t2) (pairof t1 t2
r)))`; a list's `cons` only where the tail is a list. Kayak's was the
list's alone, a pair of another tail only where one was expected (C68).
- **Where nothing is expected** (or what is, not solved yet) and the
  tail synthesized is no list (`list-like?`: a list, `nil`, a pair of a
  list, a list of a size, a union, or not known yet, are), `(cons a b)`
  is `(pairof A B r)` at a fresh region, printed `@r.1` with its
  allocation, as Fixpt's (`bloblet/width-infer`, `run/recursion`). The
  tail is synthesized first on a state then dropped, so a list's `cons`
  is checked as before; a hole in a draft is not looked at (the REPL's
  hints session found it: `(cons 1 ` had lost its hint). The same
  session's `(set-car! (cons 1 #t) ` now hints `set-car!`'s argument, as
  `(cons 1 #t)` is a pair (`tests/repl/hints.out` blessed).
- **`nil` where the type `nil` is expected** is it
  (`bidirectional/bounds`: `(pairof int nil @r)`); FX-26's: KFX's `nil`
  is a list's, and kfx had accepted the program and written Rust that
  did not build (the check's `run-fixpt-programs` found it).
- Left: `cons` as a value at Fixpt's type (`native/adapters`).
- `processes/pselect-deadline.fx` differed once under `cells-programs`'
  `--slice 2` (a deadlock reported, Fixpt's run none), then agreed on
  four runs: a race in tier 1's time slicing, noted in TODO §30.
- Refused 35 to 32, the same 337 to 340; K1 0.

## C185. Lemmas (2026-10-25)

Fixpt's `lemma.rs` and fx26.md, "Lemmas: proofs as erased identity
coercions": a definition whose type is `(proves (<= A B))`, or `(proves
(poly (binder ...) (<= A B) (<= X Y) ...))`, proves `A ≤ B` under its
hypotheses; subtyping then uses it, so an invariant generative type
widens, a non-regular one (`nest`) too, and one generative type stands
for another of its shape. FX-26's: KFX refuses `proves` (its Rust's
types have no subtyping to prove).
- **Its type** (`proves-type`): `(subr spin ((subr spin (X) Y) ... A) B)`
  under its `poly`, as Fixpt prints it; checked as any definition of that
  type (`check-top` wraps `check-top-plain`), then **its proof**, ported
  from Fixpt's `check_proof`, `rebuild` and `proof_coercion` over kayak's
  expressions: a guarded structural identity, taking apart only what it
  was given (a `tagcase` of the place given, each arm rebuilding its own
  tag, its fields in order), applying a hypothesis or a proof only to
  what was given at the same place, using itself only under a
  constructor rebuilt, and given only hypotheses, proofs, or coercions
  that rebuild. Each refusal says Fixpt's words, `this does not prove
  ...: why`.
- **Kept** as a binding `%lemma NAME`, its binders and `(A B X Y ...)`.
- **Used** (`lemma-for`, `lemma-match`, Fixpt's `lemma_instances` and
  `match_ty`) at the top of `fits-shapes`: where two generative types
  meet, not already the same, a lemma whose sides fit them, binders bound
  as they meet, its hypotheses instantiated and each fitted, the pair
  assumed meanwhile (a fact). Fixpt tries its own rules first and a lemma
  where they fail; kayak's `fits` has no try, so it takes the lemma first
  where one applies: where Fixpt's rules would hold but the lemma's
  hypotheses not, kayak refuses what Fixpt accepts, never the reverse.
- All 11 of Fixpt's `lemmas/` programs as Fixpt says: `tree-up`, `cross`,
  `nest` accepted, printed the same; the eight it refuses refused.
  Refused 32 to 29, the same 340 to 343; K1 0. Test:
  `tests/kfx-stricter/lemma.kfx`.

## C186. Binders of the kinds `data`, `sendable`, `flat` (2026-10-25)

Fixpt's kinds that say what a type binder takes (fx26.md, "The `data`
kind"; processes' `sendable`; Fixpt's TODO §77, `flat`). FX-26's: KFX
refuses each binder (the restriction pass, in a `poly`'s, `plambda`'s or
family's binders).
- **The binder** is a type binder marked as an arrow kind's is (C173),
  `^data^t`, so printed `(t data)`; named alone in a type, the variable.
- **Checked where instantiated** (`kind-holds`): at a `proj`, the type
  given; at a call, each binder's inference variable once the arguments
  have solved it (`kinds-held`, around `poly-settled`). `data` (`not-data`):
  base types, `datum`, products and sums, frozen pairs, lists and
  datatypes of data; no procedure, nothing writable, no generative type.
  `sendable`: transmissible (C101's `untransmissible`, a `sendable`
  binder itself so). `flat`: `flat?`. The refusals say Fixpt's words:
  "`m` is bound as sendable, and a (ref int @c) is not: a reference at
  @c, which can be written".
- **A `flat` binder is no type**: named only where a flat is (a blobref's
  flat, a flat array's element, a family's `flat` parameter, under the
  marker `%flat-ok`), else "`f` is a flat, not a type" (two K1 breaches
  on the way, `flat-not-type.fx`, `flat-family-not-type.fx`). A family's
  `flat` argument is checked a flat; a family's `data` or `sendable`
  parameter kayak still refuses, having no check for it there.
- Left: `conv` binders, `(t data p)` at a place. Refused 29 to 26, the
  same 343 to 346; K1 0. Test: `tests/kfx-stricter/data-binder.kfx`.

## C187. `nat?`, `certify-nat`, `rmake-icell`; names kayak-rt writes (2026-10-25)

Standard operations and names FX-26 has that kayak's checker had not:
- **`nat?` and `certify-nat`** at Fixpt's types, `nat?`'s result saying
  `(nat 0)` where true (C163's latent propositions, a size fact `x ≥ 0`).
  `certify-nat` only called by name, on what the facts in scope show no
  less than 0 (`size-shown?`), as Fixpt's "on the binding `nat?` has just
  confirmed"; named any other way, refused (Fixpt's F16:
  `certify-nat-alias.fx`, `certify-nat-outside.fx`, two K1 breaches on
  the way). The checker's own `nat?` (whether a type is `nat`) renamed
  `nat-typed?`, the standard one's name now, as KFX's restriction pass
  refuses it.
- **`rmake-icell`**: an I-cell in a region within a place, as `rnew` and
  `rmake-array` are (`run/region-allocators`).
- **Names kayak-rt or the emitter writes** (`nil`, `cons`, `once`, ...)
  are FX-26's to bind (`datum/quote-shadowed`: a `let` of `cons`, `append`
  and `nil`); the checker takes them, and the restriction pass refuses
  one bound for KFX (`reserved-bound`: a `let`'s, `let*`'s, `letrec`'s,
  a `lambda`'s, a definition's).
- `processes/pselect-deadline.fx` failed again under the full check's
  `--slice 2` (a deadlock reported); not reproduced in 200 runs alone nor
  100 with every core busy. TODO §30.
- Refused 26 to 23, the same 346 to 349; K1 0.

## C188. Variance as Fixpt's; variance marks; variadic `append` (2026-10-25)

- **A correction to C183**: its note that references, arrays, I-cells,
  addresses and pairs are covariant came from a probe that never ran
  (Fixpt stops at a file's first error, and each probe's first error was
  elsewhere). Probed again, one file to each, two definitions declaring
  what they read: Fixpt is invariant in what storage that can be written
  holds (a `ref`, an array, an I-cell, an address, a bloblet's fields, a
  mark key, a pair or a list at a region not frozen), covariant in frozen
  lists, products and a procedure's result, contravariant in its
  parameters. Kayak's `fits` had been covariant in the first, a K1 hole
  width subtyping (C183) and `nat ≤ int` opened: now each argument both
  ways (`frozen-region?` tells a pair's). A blobref's owner stays
  covariant, as Fixpt's (`bloblet/blobref-width.fx`).
- **Bounds from both sides, for a union**: a call's result, a type not
  known yet, where a union is expected, is fitted after the arguments
  (`bound-after?`), as a procedure's already was: solved first to the
  union, an invariant argument (an array of `int`) would not fit
  (`bidirectional/bounds.fx`, which the invariance above had refused).
- **Variance marks** on a generative type's parameters, `(t type +)`,
  `(t type -)` (Fixpt's fx26.md, "Generative types"): its
  representation must bear the mark out (`polarities`, each occurrence's,
  from `+` at its top: `bad-variance.fx`, a covariant one in a `ref`,
  refused); kept, `%variance NAME`, and the parameter so compared
  (`fits-variant`); printed unmarked, as Fixpt's. A mark on a
  `define-type` or `define-datatype` head refused, as Fixpt's ("a
  parameter is `(name kind)`"); KFX refuses marks.
- **A generative type is no constructor where its representation is its
  parameter**: a recursion through it is unguarded (`unguarded?`;
  `generative/unguarded-cycle.fx`, a K1 breach the marks had opened).
- **`append` and `string-append` of any count**, and as values, at
  Fixpt's `vsubr` types (`native/variadic-standard.fx`); KFX's are of two.
  An effect printed has each atom once, two resolving to one where a
  region was solved after (`atoms-once`).
- Refused 23 to 21, the same 349 to 351; K1 0.

## C189. F17 through `mu` and `proj`; KFX's names to the restriction pass (2026-10-25)

- **A lambda against a recursive type**: `proc-type?` and the `lambda`
  check see through a `mu` (C174), and a written convention (C181).
- **Fixpt's F17 through `mu`**: a type that holds itself through a
  parameter makes a call through it able to loop with no recursion;
  `cyclic-walk` met datatypes' names again, now a `mu`'s variable too.
- **And through `proj`**: a call of `(proj p T ...)` had never said `spin`;
  now it does where `p` is not a standard operation, takes a procedure
  over one of its type variables (`poly-takes-procedure?`, as the named
  call's `poly-self-application`), and its instance is cyclic. An identity
  at a cyclic type stays pure. (`terminate/poly-self-application.fx`, a
  K1 breach the `mu` lambda opened, refused again; its `-spin` sibling
  accepted.)
- **KFX's names (C27) are the restriction pass's**: the checker took
  every name it binds through KFX's rules (lower case, `-` ..., none that
  hides a type in Rust), refusing FX-26 programs (`app-s-at-T`, a type
  variable `t` beside a type `T`). Now FX-26's names are any; the
  restriction pass refuses for KFX a name bound (a definition's, a
  `define-rec`'s, a `let`'s, a `lambda`'s, a `tagcase` arm's) that is no
  KFX name or one kayak-rt writes, and a type variable named as no Rust
  type parameter can be (`binders-restricted`, its types those of its
  own file: a loaded module's binder `c` had met the loading program's
  type, which the full check found).
- Refused 21 to 20, the same 351 to 352; K1 0.

## C190. A lambda typed by the result expected; naming a `letrec` member (2026-10-25)

- **`(stay-cellular (lambda (x) ...))` where a procedure is expected**: a
  call fits a procedure expected after its arguments (C35, so that one
  that cannot abort is lifted); but where its result is a type not known
  yet that is also the type of a parameter given a `lambda` whose
  parameters' types are not written (`lambda-given-result?`), it fits
  first, so that the lambda has a type, as Fixpt's bounds give it
  (`native/made-native.fx`).
- **Naming a `letrec` member is no `spin`**: kayak said `spin` for a member
  that may not end named as a value; Fixpt does not (making a `letrec`'s
  value makes lambdas, pure), its type saying `spin` for whoever calls it.
- Refused 20 to 19, the same 352 to 353; K1 0.

## C191. `->*` groups by what each calls; `->*` in a `letrec` (2026-10-25)

Fixpt's fx26.md on `->*`: "in a group (`letrec`, `define-rec`, a module's
lambdas) components callees first, each member told those found, taking
its component's; `spin` if it reads a member's global".
- **`define-rec`** (`found-rec`): each member's effect found alone, then
  the effects of every member it reaches by calls (`called-of`,
  `reach-from`), itself among them. Kayak had given each the whole
  group's (C125's "no less than Fixpt's"), more than Fixpt's: a member
  that names another without calling it took its effect
  (`bidirectional/arrow-star-groups.fx`'s `noisy`).
- **A local `letrec`** may declare `->*` too, refused till now: its
  members found likewise (`letrec-stars`), `spin` where the group may not
  end and a member reaches one that calls itself round; then C179's
  globals found.
- **A lambda's body masked as a whole** (C182 masked its forms one by
  one, not a `letrec`'s storage no name outside sees), and the effect a
  `->*` or a `define*` finds the same.
- Refused 19 to 18, the same 353 to 354; K1 0.

## C192. Effects fitted both ways round a `cwcc` (2026-10-25)

`native/whole-captures.fx`: `(cwcc (lambda ((k k-int)) ...))`, its
receiver's parameter written at `(goto @k)` where `cwcc`'s is `(goto ?r)`.
- **A region of the given effect not solved** is solved to the expected
  effect's atom of the same kind (`fit-effs` solved only the expected
  side's).
- **An open row keeps a global's read literally**: an effect variable
  grown to hold a receiver's effect took `(read @globals)` first and then
  dropped `(read (globals keeper))` as within it; Fixpt keeps both, and a
  `define*` finds the globals it reads from them (expanded and resolved
  first, which `check-define*` had not).
- Left: the fresh region's number in an expression's type (Fixpt's
  `@r.2`, kayak's printer `@r.1` always).
- Refused 18 to 17; K1 0.

## C193. `letfreeze` into the heap in KFX (2026-10-25)

The Fixpt team's proposal (TODO §36), its step 1, at the user's word: the
compiler on kayak's runtime wants frozen data tasks share without copying.
- **The restriction pass** takes `letfreeze` (its plain form and `(r
  heap)`) and `const` in a type or an effect; data frozen into another
  place it still refuses, as the checker does.
- **The emitter** writes a `letfreeze` as a `letregion` (`frozen-region-name`
  reads `r` or `(r heap)`): what its body makes at `r` in the heap's
  arenas, the value of the type it would have at the heap (a type at
  `const` or `acyclic` written so). kayak-rt's `Cons` has no `Cell`, so a
  frozen list is `Send` and `Sync` where its elements are, rustc checking
  it, no `unsafe`.
- **Crossing threads**: reading `const` data was a crossing atom already
  (C102); a written effect reading `const` (or `(const heap)`) now counts
  as crossing for a procedure's `Send + Sync`, and a list at `const` is
  transmissible (`untransmissible`), as one at `acyclic`.
- **Frozen data holding what may be written** (step 2): the team's choice
  (a), which needs nothing: the checker refuses mutable data at `const`,
  and a frozen list of references at another region still holds
  references that can be written, so it crosses no thread
  (`tests/kfx-refused/frozen-ref-sent.kfx`, refused by Fixpt in the same
  words).
- `tests/kfx/frozen-shared.kfx`: a list built at the body's region while
  a reference counts its cells, frozen `const`, read by three `par`
  branches on threads (`par2`), and one never written, frozen `acyclic`;
  Fixpt's answer, by kayak's Rust, by default and with `unsafe-par`
  (`cargo test -p kfx-corpus --features kayak-rt/unsafe-par`, run once by
  hand: the check runs no corpus under that feature).

## C194. Tables keyed by identity, FX-26's and KFX's (2026-10-25)

TODO §36's eq-tables. Kayak's checker had none (`native/eqtables.fx`
refused), though the engine's tier 1 had the primitives (kind 43).
- **The checker**: `(identity k kr)` and `(eqtable k v kr r)` (Fixpt's
  generative types 3 and 4), invariant; `pair-identity`, `ref-identity`,
  `array-identity`, `icell-identity`, `make-eqtable`, `eqtable-ref`,
  `eqtable-has?`, `eqtable-set!`, `eqtable-delete!`, `eqtable-count` at
  Fixpt's types, each operation writing the keys' region, so that no
  table is keyed by frozen data. `native/eqtables.fx` the same as Fixpt's.
- **kayak-rt** (`eqtable.rs`): `Identity`, an address, for `&Cell<T>` (a
  reference), `Array<T>` and `&ICellBox<T>` (an I-cell), none copied
  where a `Value` boxes it; `IdentityOf<K>`, the dictionary;
  `EqTableAt<'t, K, V>`, a `RefCell<HashMap<usize, (K, V)>>` by a fixed
  hasher, made by `alloc` at the keys' lifetime, so rustc refuses a table
  outliving a `letrena` whose objects key it; `EqTable<K, V>` its heap
  form. No iteration: the order of addresses is never seen, answers
  Fixpt's on every run.
- **The emitter**: `(eqtable k v kr r)` as `EqTableAt<'kr, K, V>` where
  the keys' region (or what they hold) is a `letrena`'s, else
  `EqTable<K, V>`; `(identity k kr)` as `IdentityOf<K>`. A table is no
  `Sync` (`sync-type?`).
- **Pairs wait**: KFX refuses `pair-identity` (a list seen as a union's
  value might be rebuilt in Rust; `tests/kfx-stricter/pair-identity.kfx`).
  `eq?` stays refused.
- Tests: `tests/kfx/eqtables.kfx` (references, two alike being two keys;
  set, set again, delete, count; arrays; a table inside a `letrena` keyed
  by references in its arena), Fixpt's answer by kayak's Rust;
  `tests/kfx-refused/eqtable-outlives-keys.kfx`, a table given out of its
  keys' `letrena`, refused by both.
- The checkers' comparison: refused 17 to 16, the same 354 to 355; K1 0.
- Refused 73 to 60, the same 283 to 294; K1 0 (five breaches on the way,
  each a rule above). Left: the kind `flat` (`flat-kind.fx`), `narrayof`
  (`narrayof-lengths.fx`); printing a blobref's owner by its
  abbreviation's name in an expression's type.

## C195. The corpus under `unsafe-par` in the check (2026-10-25)

- `scripts/check.sh` runs the KFX corpus a second time built with
  `kayak-rt/unsafe-par` (C102), in `target/unsafe-par`: `par`'s branches
  sharing what may be written, in regions neither other touches, at once
  rather than in turn. Until now no check ran kayak-sync's `unsafe`, the
  one place kayak relies on the checker's interference table where rustc
  cannot see it; TODO §36 asked for `frozen-shared.kfx` both ways.
- The cost: a second release build of the corpus, beside the rest.
- Agreed with the user (a change to which checks run).

## C196. Paths narrowed by a test (2026-10-25; Fixpt's TODO §54)

Fixpt narrows a path from a variable, `(car x)`, `(cdr x)`, `(extract p
a)`, as it narrows a variable, while nothing may have changed what the
path reads (`unions/paths-narrow.fx`). Kayak's checker now does the same.
- **A fact** is a bind `%path ROOT STEP ...`, its type the narrowed one,
  its `where` `path K R ...`: how many writes the state had seen when the
  test made it, and the regions the path reads through (a pair's; a
  product's field none). A test of a standard predicate on a path pushes
  one into each branch, as `narrowed` does for a local; a path's type is
  found step by step, each prefix narrowed by its own fact.
- **What ends it** (Fixpt's `kill_paths`): the state (`st`) has a log of
  the writes seen, newest first, each expression's effect noted as it is
  checked (`path-settled`, in `synth` and `check`): a write's or a fill's
  region, or `*` for a transfer of control, a process's wait, or an
  effect not known. A fact is dead if a write since may be to one of its
  regions not frozen: two constant regions are two, anything else may be
  any. The log is threaded through the state, so a write in one branch of
  an `if` ends the fact for the other branch too, as Fixpt's dead flag,
  set as its checker goes, does (`path-written-other-branch.kfx`).
- **Closures** see no fact: each lambda's parameters come with `%closure`,
  where a fact's lookup stops, and `%paths` `closed`, so that writes in
  its body are not noted against the facts outside, which Fixpt keeps
  apart by closure depth.
- **Rust**: a path whose fact is in force is found at the fact's type,
  `coerced` from the path's own, so an unbox at it, as for a narrowed
  local.
- Tests: `tests/kfx/paths-narrow.kfx` (Fixpt's program, run: `61 s` by
  Fixpt and by kayak's Rust); refused by both, `path-written.kfx`,
  `path-in-closure.kfx`, `path-written-other-branch.kfx`. Probes of an
  effect variable and a region binder refused by both too.
- The checkers' comparison: refused 16 to 15, the same 355 to 356; K1 0.

## C197. `cons` as a value, at Fixpt's type; a definition again at a narrower effect (2026-10-25)

`native/adapters.fx`, accepted by Fixpt, refused by kayak's checker twice
over.
- **`cons` as a value** where a procedure making a pair is wanted, `(app2
  cons 7)`, its parameter `(subr (alloc @heap) (int ints) int-and-ints)`:
  FX-26's `cons` is `(poly (r t1 t2) (subr (alloc r) (t1 t2) (pairof t1
  t2 r)))`, kayak's the list's. Named where such a procedure is expected,
  `cons` is instantiated at the pair wanted, `(subr (alloc r) (a b)
  (pairof a b r))`, and fitted to it. KFX refuses it (`kfx-restricted`):
  its `cons` makes lists. Called, and as a value elsewhere, as before.
- **A definition again** whose type is the old one's but for an effect
  within it: `(define* down ...)` again, Fixpt's type for it with `(read
  (globals down))` beside `(read @globals)`. Fixpt assigns the global
  when the new type is a subtype of the old (`fits_old`); kayak compared
  the two types' texts. Now also: the same parameters and result, the
  effect within the old (`narrower-effect?`). Other subtypes still
  refused at the redefinition (C61), stricter than Fixpt, K1 kept.
- The checkers' comparison: refused 15 to 14, the same 356 to 357; K1 0.

## C198. A bloblet whose type nothing says (2026-10-25)

`native/many-values.fx`: `(let ((b (make-bloblet 0 x ...))) ...)`, no
`the` around it. Fixpt's `synth_bloblet` finds each field's type, and
gives the bloblet the place's region for `rmake-bloblet`, else a new
constant region, `bloblet.N` (`fresh_region_named`), which masking drops
where nothing names it. Kayak's checker refused it; now it does the
same, the region `@bloblet.N` (a constant, so that masking may drop it,
which it never does of an inference variable). KFX refuses it
(`kfx-restricted`): its bloblets are made under a `the`, their Rust type
said. Probes, each the same in both: such a bloblet given where one at
`@heap` is wanted, refused; read inside a `pure` procedure, accepted;
a field written at another type, refused.
- The checkers' comparison: refused 14 to 13, the same 357 to 358; K1 0.

## C199. `(with #%fx n)`, the standard name (2026-10-25; Fixpt's TODO §46)

`run/standard-refs.fx`: `(with #%fx n)` is the standard `n`, wherever it
is and whatever names it there (a local `+`, say), called or as a value.
- **The checker**: `#%fx` opened by `with` around one name, a standard
  binding: the body checked with that binding in front of the scope (the
  first bind of the name whose `where` is `prim`), so the name has its
  standard type, instantiated as where nothing shadows it. Anything else
  under `#%fx` refused, as Fixpt refuses it. KFX refuses the form
  (`kfx-restricted`): KFX shadows no standard name.
- **The engine**: lowered as the name is with no local and no global in
  scope (`lower-name n nil nil`): the primitive itself. `standard-refs.fx`
  answers `123456`, as on Fixpt.
- The checkers' comparison: refused 13 to 12, the same 358 to 359; K1 0.

## C200. `dletrec`; the standard types in the standard scope; pairs at two types (2026-10-25)

`pldi89/c7.fx`, accepted by Fixpt, took four changes; now the same as
Fixpt's, its types as printed too.
- **`(dletrec ((x T) ...) body)`**, types defined in terms of each other,
  read as `mu`s (C174): each name, where no enclosing one binds it, the
  `(mu %dl-x T)` of its definition, the names inside made so in turn
  (`dletrec-in`). Named apart (`%dl-`), so that no abbreviation of the
  program's of the same name is taken for one.
- **The standard bindings' types read in the standard scope**
  (`prim-binds`), as Fixpt's are: they were read with the program's
  types declared, so a program's `(define-type p ...)` made `assert`'s
  named parameter `(p bool)` an application of `p` and every program
  with such a type was refused ("a type given the wrong number of
  arguments").
- **`(proj (proj car @p) t1 t2)`**: FX-26's pair operations, `cons`,
  `car`, `cdr`, `set-car!`, `set-cdr!`, are poly in a region, then in two
  types, a pair's head and tail; kayak's take one, a list's element. A
  `proj` of one at three arguments is Fixpt's type at them
  (`pair-op-text`, from its standard.rs); KFX refuses it.
- **A call of a value of a `mu` type**: unfolded (`walk-mu`) to find its
  procedure.
- And a crash found probing: `(car x)`, `x` a local of a `mu` type of a
  pair, ended the checker on `car` of nothing (`local-pair` gave the `mu`
  unfolded nowhere); now unfolded. Unfolding `pair-expected` too changed
  `recursive/family-knot.fx`'s printing, so it stays as it was.
- `tests/kfx-stricter/`: `pair-proj`, `fx-module` (C199), `cons-value`
  (C197), `bloblet-untyped` (C198), each accepted by Fixpt and refused by
  KFX.
- The checkers' comparison: refused 12 to 11, the same 359 to 360; K1 0.

## C201. Frozen bloblets' type (2026-10-25)

`recursive/frozenblob.fx`: `(bloblet (frozen T ...) R)`, a bloblet whose
fields no one writes, covariant in them, as a frozen pair is. Kayak's
checker refused the type. Now `%fbloblet` (region, then fields), printed
as written, of no invariant kind, so fitted field by field one way; a
recursive one through `mu` as any type. Nothing yet reads one
(`bloblet-ref` asks a `%bloblet`): stricter than Fixpt, K1 kept. KFX's
restriction pass refuses the type (`tests/kfx-stricter/frozen-bloblet.kfx`).
Probes, refused by both: a frozen bloblet of a wider sum where a narrower
is wanted; a bloblet of fields written, either way.
- The checkers' comparison: refused 11 to 10, the same 360 to 361; K1 0.

## C202. Redefinition at another type, under `--check` (2026-10-25)

`redefine/rerun-chain.fx`, `redefine/knot-spin.fx`: Fixpt's
`top_defining` (fx26.md, "Redefinition"). A global defined again at a
type its uses may not take (not its own, nor its own but for a narrower
effect, C197) has each earlier definition that uses it, and each that
uses those, oldest first, checked again: each that checks is defined
again, and `fixpt check` writes its lines again; each that does not is
broken, and a use of it after is refused.
- **Where**: kayak's refusals end the process (`error`), and the checker
  is of effect `kfx`, which makes no process; so the checking again is
  the `--check` driver's (`fx26/start.kfx`, `lines-redefining`): each
  definition so far kept (its names and form, a later one of the same
  name replacing it), and each one checked again by a worker, as the
  REPL checks (`recheck-of`, exported from `tasks`; the job `recheck`),
  its refusal a value.
- **The checker**: `global` lets a name be defined again at another type
  where the driver marks the scope (`%reruns`), or where the old one is
  broken; a broken global (`where` `broken`) is refused where it is
  named, called, or projected.
- **Users**: the definitions whose forms name a name redefined (or one
  of a user's), by `names-in`: more than Fixpt's free names where a local
  is named alike, so a definition may be checked again that Fixpt's is
  not (its lines then written twice); none found so yet.
- **Not yet**: the REPL and the engine's run, which still refuse a
  definition again at another type (so the engine never runs a program
  whose definitions it would have to run again); KFX, whose restriction
  pass refuses any definition again.
- Probes, refused by both: a broken global given as a value, called
  through a user also broken, used by a later definition.
- The checkers' comparison: refused 10 to 8, the same 361 to 363; K1 0.

## C203. Types and effects a module selects from a value it makes (2026-10-25)

`modules/load-layers.fx`: three layers of module files, each loading the
one below as an instance, `(define rd ((proj (load-module ...) s)))`, and
selecting its type and effect, `(define-type cell (select rd cell))`,
`(define-effect own (select rd own))`. Now accepted; its printing differs
(the order of a module's items, `cell` by its name, the loads' effect,
kinds of difference §30 has already).
- **Order**: a module's items are checked types first, then values
  (`module-order`); an item selecting from a value the module makes
  (`selected-value`) is checked after that value's group instead
  (`selected-after`), in the order kept for memos too.
- **The module's prep** reads its effects for KFX's rule on aborting;
  one selected from a name not bound yet is read where it stands.
- **Outside**: such a type's abbreviation, `m..t`, is `(select r t m)`,
  read as `r` inside `m`, `m r` outside; the printer's module scope names
  the module's values so too (`%val x`).
- **Reach**: `(with r x)`, `r` a module value the module makes: any name
  inside may be `r`'s, so none counts as reaching an item (else `(define
  bump (with rd bump))` reached itself).
- **Nested modules' types and effects given on**: a module gives its
  modules' `n t`, `n e` on as `m n t`, `m n e` (`nested-aliases`,
  `nested-effdefs`), as their values already were; a module's own type
  (`module-type-of`) leaves them out, its modules' types saying them.
- Probes the same as Fixpt's: a module's type and effect selected from
  a module value it binds; its type printed.
- The checkers' comparison: refused 8 to 7, differs 62 to 63; K1 0.

## C204. `distinct` region binders (2026-10-25; Fixpt's region-anti-aliasing.md, (C))

`regions/distinct-swap.fx`: `(poly ((r1 region distinct) (r2 region))
...)`, Lucassen's rule (MIT/LCS/TR-408 §3.6.3): a `distinct` binder is
given a region apart from every other binder's and from every region the
type names besides its binders. Accepted now, and Fixpt's four refusals
(`distinct-mentioned`, `-not-loose`, `-one-region`, `-passes-on`) refused.
- **Marks**: in a scheme (a global's own `poly`), `%distinct 'r` after
  its binders, making no inference variable and counting for no `proj`;
  in a poly type (rank 2, as an abbreviation names one), the binder named
  `'r@P%d`, printed `(r region distinct)`; a `plambda`'s, a `letregion`'s
  region, `%distinct 'r` among the scope's type variables.
- **The check** (`distinct-held`): two regions are apart if they differ
  and one is distinct (a binder so marked in scope, a `letregion`'s), or
  both are constants; one not solved yet passed over, as Fixpt's is. Made
  after a call of a global whose scheme has the mark (in `kinds-held`,
  its variables known by position), after a call of a value of a poly
  type with one (`synth-call-of`, around `synth-call-of-plain`), and at a
  `proj` of either.
- **Subtyping**: a poly whose binder is `distinct` is no poly whose
  binder at that place is not (`fits`).
- **Stricter**: a global whose scheme has a `distinct` binder named but
  neither called nor projected is refused (its instance's regions would
  be given where no check sees them).
- **KFX**: its restriction pass refuses a `distinct` binder
  (`tests/kfx-stricter/distinct-region.kfx`), as it did by the checker's
  refusal before.
- Probes the same as Fixpt's: a proj at `@heap` the type names; a tight
  poly given where a loose one is wanted; called through a local, and
  through a parameter, given one region twice; given two letregions'.
- The checkers' comparison: refused 7 to 6, differs 63 to 64 (its
  effects, as without `distinct`, keep a letregion's region masking would
  drop); K1 0.

## C205. Convention binders (2026-10-25)

`conventions/poly.fx`: `(poly ((c conv) (e effect)) (subr e ((subr (conv
c) e (int) int) int) int))`, the binder solved by what is given, or
projected. Now the same as Fixpt's, printing too.
- **The binder** `(c conv)`, in a scheme, a poly type, a `plambda`:
  `^conv^c`, of kind `conv` (C173's marks), an inference variable where
  instantiated, a convention where projected (`native`, `cellular`,
  `fx`; anything else refused, `poly-wrong-kind.fx`).
- **`(subr (conv c) ...)`**, `c` a binder: `%convof` (the convention, the
  procedure); one whose convention is given reads as that convention's
  type (`conv-settled`: `native` the program's, `cellular` and `fx`
  their wrappers), printed so; called as its procedure.
- **Fitting** (`conv-fit`): two procedures of one convention, an inference
  variable solved to the other's (the program's procedure is `native`);
  any where `fx`'s is, free (C116); a binder only itself.
- **Stricter**: where Fixpt converts a procedure of one convention given
  for another of a binder's (an adapter), kayak refuses (probes: a
  binder's procedure where the program's is wanted; a binder given two
  conventions). K1 kept. KFX's restriction pass refuses the binder
  (`tests/kfx-stricter/conv-binder.kfx`).
- The checkers' comparison: refused 6 to 5, the same 363 to 364; K1 0.

## C206. Data frozen into a place (2026-10-25; Fixpt's places-and-regions.md)

`run/acyclic-in-a-place.fx`, `run/letfreeze-into-place.fx`: `(letfreeze
(r a) ...)` in an arena `a`, its value at `(acyclic a)` or `(const a)`,
walked by a procedure poly in the place, `(listof int (acyclic p))`, with
no `spin`. Both the same as Fixpt's now, printing too; Fixpt's refusals
(`freeze-into-shorter`, `acyclic-walk-escapes`, `frozen-outlives-place`,
`frozen-read-effect`) still refused.
- **The region** `(acyclic p)`, `(const p)` of a place `p`: `%frozen`,
  its kind and the place's region, so substitution reaches the place; in
  effects, `read:(acyclic 'p)`, renamed with the place (`rename-atoms`),
  written `(read (acyclic p))`. Frozen (`frozen-region?`), so covariant
  where a frozen pair is; never masked but by its place's end (`masked`
  of the place masks it; `maskable-atom?` not); a read of it an effect,
  as Fixpt's (F2).
- **`(letfreeze (r p) ...)`**: `r` a region within `p` (`%within`), so
  data `rcons` makes in another place is refused (`region-within`: a
  region a `letfreeze` put within one place is within no other); the
  value's `r` made `(acyclic p)` if the body never wrote it, else `(const
  p)`; KFX refuses it (`kfx-restricted`), and its restriction pass any
  type of a frozen place.
- **Termination**: a list at `(acyclic p)` ends, as one at `acyclic`
  (`acyclic-region?`, terminate.kfx).
- **Writes**: a call writing a frozen place's data refused, as one
  writing `const`'s (found probing: `set-car!` of it was accepted).
- Probes the same as Fixpt's: that write; a list of the place where one
  frozen in it is wanted; a walk of a `const` place's list, or of the same
  list, needing `spin`.
- The checkers' comparison: refused 5 to 3, the same 364 to 366; K1 0.

## C207. Arrays of a length, `narrayof` (2026-10-25; Fixpt's TODO §77, N5c)

`sizes/narrayof-lengths.fx`: `(narrayof T n R)`, an array whose length
its type says. The same as Fixpt's now, printing too.
- **The type**: `%narray` (element, size, region); below its array, and
  an array below one of a length `finite`; two of lengths, the lengths
  the same or the one expected `finite` (`size-fits`, as `nlist`'s).
- **Made**: `(make-array n x)`, `n` a literal, where one is wanted, of
  length `n`; `(the (arrayof T R) e)` where one is wanted keeps the
  length, as Fixpt's does. Else `make-array` is an array as before, so
  nothing of KFX's changes; KFX refuses the type (the restriction pass,
  `tests/kfx-stricter/narrayof.kfx`) and the making (`kfx-restricted`).
- **`array-length`** of one: `(nat n)`, fitted to what is wanted (a probe
  found it was not at first: `(define y (nat 4) (array-length five))`).
- Probes the same as Fixpt's: a length other than the one wanted, made,
  through `the`, asked of `array-length`; an array where one of a length is.
- The checkers' comparison: refused 3 to 2, the same 366 to 367; K1 0.

## C208. Made code's leaves; the last refusals (2026-10-25)

`made-code/leaves.fx`, `made-code-x86_64/leaves.fx`, the last two of the
programs Fixpt accepts that kayak's checker refused. Now none is.
- **What kayak-jit cannot run yet is KFX's to refuse** (C116): `seal-code`
  of a bloblet whose fields native code cannot read in kayak, a
  `native-procedure` of more than four parameters or of values that do not
  cross, were the checker's refusals; now `kfx-restricted`, KFX refusing
  them as before. The engine runs no native code (`native-code?` is no
  name of its), so it never runs these.
- **A local named by a word FX-26 has only in types and effects** (`read`,
  `ref`, `pairof`, ...), called: a call, as Fixpt's parser makes it, which
  treats as forms only its expression forms' heads; KFX's restriction pass
  refuses such names (C187).
- **`car`, `cdr` of a pair whose tail is no list, an expression's**, not a
  name's, `(car (read))`: its part (KFX refuses it; its pairs are taken
  apart where named, C68, C77).
- Probes the same as Fixpt's: such a part of the wrong type; its effect.
- The checkers' comparison: of the 433 programs Fixpt accepts, refused 2
  to 0, the same 367 to 369, 64 differing in printing; of the 336 it
  refuses, all refused; K1 0.

## C209. The generator's calls in the fx convention, and `proj` (2026-10-24)

The next round of C133's generator (its prompts are C146's).
- **Calls of a composable continuation are untyped** (`call`,
  `tailcall`), as Fixpt makes them: a continuation's type comes from the
  standard's declarations, in the `fx` convention, and Fixpt types a call
  only where the checker noted its callee `cellular` (`c-call-in`,
  `NodeFacts::call_conv`). Our checker keeps no conventions on procedure
  types, but does keep a composable continuation's (`%composable`, C60);
  so it notes **`call-fx`** at a call of one (`fx-noted`,
  `kfx/check-exprs.kfx`), and the generator reads that. Rejected: giving
  procedure types a convention field throughout the checker for this one
  case; no other standard operation gives a program a procedure.
- **`proj` is its own node** (`x-proj`), as Fixpt's `e-proj` is, not
  dropped by the parser: a standard name under a `proj` is not called as
  its operation, open-coded, but as its value, its own word, as Fixpt's
  `c-standard-name` decides (`bench/captures.fx`).
- **Results**, on arm64 macOS at its answers of 4caffc2: 284 programs
  agree, as before, 228 of them Fixpt's words exactly (from 218).

## C210. Tier 6 on arm64: C138's, by `bl` and `ret` (2026-10-24)

The user: "Take item 1" (TODO §34, step 5: tier 6, "the same on arm64,
measured on macOS"). hand.kfx now compiles register words a second time
as native procedures under `--direct`, as hand-x64.kfx does (C138): the
same words (those that never allocate), the same blocks, fuel spent a
block at a time, records of the calls on the machine stack, leaving for
tier 5 where tier 5 would give back (`materialize!`), entered by ways in
that push a bottom record. What arm64 changes:
- **Calls by `bl`/`blr`, returns by `ret`**: the callee's link is in
  `x30`, as everywhere in this machine; the caller keeps its own `x30` in
  its record and takes it back after, so a tier 6 `return` is a `ret` to
  the `bl` that called it, which the processor predicts (as C137 made
  x86_64's tier 5, and TODO's step on arm64's tier 5 would).
- **The record**: `(x30 nx fp clo cur -)` from the machine stack's top,
  48 bytes, for `sp` must stay 16-aligned where it is a base; the bottom
  record `(1 1)`, 16 bytes. Materializing reads `nx fp clo cur` as on
  x86_64, from the innermost; the code left gets the link the way in
  kept (`st-t6link`) if it is the outermost's, else none.
- **No registers moved**: register code's registers are `x0`…`x8` all
  along (C130), so a way in or out moves none, where x86_64's moves
  `RESULT`, `REG1`, `REG2` through the state.
- **An operand's place made only where read**: tier 5 makes each
  instruction's place in `x12` (it gives back by it); tier 6 never gives
  back, so only the instructions that read an operand from the word
  (`const`, `global`, `setglbl`, `op2imm`, `global-guard`) make it. Kept
  read from the word rather than made immediates: a collection moves the
  values cells hold.
- **Found by the tables**: arm64's tier 5 finds a callee's entry for a
  call by its table (instruction 2's entry, `entry-address!`), so the
  table's entries for each block's first are its ways in, and a call
  from tier 5, or a return or entry the stack's way, goes into tier 6.
  Field 1 of the table is the tier 6 code, or 0, which a tier 6 `invoke`
  reads, as x86_64's.
- **Shared**: which words (`direct-able?`), their blocks (`t6-leaders!`,
  `t6-blocks!`), their labels (`t6-prepared!`) and where a branch goes
  (`t6-to`) moved from hand-x64.kfx to hand.kfx, for both, renamed from
  `y-…`; hand-x64.kfx's code is otherwise as it was. `direct-able?`
  takes whether allocations are tier 6's, which x86_64 passes (C147's)
  and arm64 not yet. `--fixpt-calls` stays x86_64's (C140).
- **Measured** on an Apple M5 Max, wall time, the median of 25 runs
  less that of the same run stopped at its first step, tier 6 against
  tier 5: `helpers` −72%, `loop` −69%, `tak` −37%, `fib` −33%, `lists`
  −21%, `closures` −14%, `captures` the same (its continuation is
  aborted 20 calls deep, as in C137).
- **Checked**: `cells-programs.py` runs `--regs --direct`, under
  `--gc-stress`, and at a slice of 3, on arm64 macOS too: the 338
  programs agree, and the 85 of `--ours`.
- **Noticed**: x86_64's tables never send a block's first to tier 6,
  written after `h-6-on` is reset (TODO §34); not changed here, where
  x86_64 cannot run.

## C211. Tier 6 allocates, as tier 5 does; its frames keep tier 5's links (2026-10-24)

The user: "Do allocation" (TODO §34, step 5's next: "allocation (`cons`,
`lambda`, `prim`) by the heap's top as tier 5 allocates, which a
collection may not follow while records live").
- **Nothing new to allocate with**: tier 5's own allocations (`cellular
  cons 2`, `prim %region-cons 3`, `lambda` of a closure, C130) already
  bump the heap's top in the machine, and give back where tier 1 would
  collect first or grow the heap. In tier 6 that giving back is leaving
  (C138), which writes every record as its return entry before tier 5
  gives back; so a collection never sees a record, and these three are
  admitted (`direct-op?`'s `allocs`). The other forms of `prim`,
  `cellular` and `lambda` tier 5 gives back at once: a word with one
  would leave at it every time, so it stays out.
- **As x86_64's**: hand-x64.kfx's tier 6 has taken the same three since
  C147, by the shared `direct-able?`'s `allocs` (C210); arm64's now passes
  it too.
- **Tier 6's frames keep tier 5's links**: a frame saved in tier 6 kept
  0 for its link (C138, C210), so once left every return in it went the
  stack's way. With allocation, a word leaves when a collection is due,
  however deep it is (`closures`: 750 collections, 1000 calls deep). Now
  each call's record carries tier 5's point past the same call (labelled
  as tier 5 is compiled, `after-call!`), which the callee's `save` keeps
  in its frame, as tier 5's would; leaving gives the innermost that link,
  from the record on top. The bottom record carries the link tier 5's
  code had where it went in, so arm64 no longer needs `st-t6link`. It did
  not change the timings here; it makes leaving cheaper wherever it is.
- **Measured** as in C210 (an Apple M5 Max, the median of 25 runs less
  that of the run stopped at its first step), tier 6 against tier 5:
  `helpers` −71%, `loop` −69%, `lists` −34% (from −21%), `tak` −34%,
  `fib` −32%, `closures` −11% (from −14%), `captures` the same. Against
  C210's tier 6 directly: `lists` −16%, `closures` +3.5%, the rest the
  same. `closures`' 7 register words are all tier 6's now (from 2), so
  it does not leave for want of a callee's tier 6 code; what it loses is
  writing about 500 records as entries at each collection, which tier 5
  wrote as it went. Accepted: in exchange for the calls between.
- **Checked**: `cells-programs.py` (`--regs --direct`, under
  `--gc-stress`, where every allocation leaves, and at a slice of 3):
  the 338 programs agree, and the 85 of `--ours`.

## C212. Tier 5 does more of `prim` itself, and tier 6 takes it (2026-10-24)

The user: "Go" (TODO §34, step 5's next: "the rest of `prim` and
`cellular`, by how often they keep a word out").
- **Counted first**: over the 338 programs' listings (4591 register
  words, 42% tier 6's after C211), what keeps each word out, and which
  words one instruction alone keeps out. `%fx26-sched` keeps out 1496,
  but they are the scheduler's own words, in the 34 programs that run
  tasks, and a call to tier 1's scheduler is tier 1's by nature. Then
  `setfield` 130, `%make-box` 103, `%fx26-procedure?` 68, `%make-frozen`
  46, `%fx26-mul` 39 (`prim2` and `prim2imm`): these five, 386 words.
- **Tier 5 first**: tier 6 can take only what tier 5 does itself (else
  it leaves at it each time), and tier 5 gave all five back. Now, in the
  body tiers 5 and 6 share, each as tier 1 does it: `setfield` checks as
  a field is read (tier 1's `field-ok?`) and stores with no barrier, as
  tier 1's `slot-set!`; `%make-box` lays out `make-bloblet!`'s box;
  `%make-frozen` checks its kind is a sum's or a product's and lays out
  `frozen-of!`'s bloblet, frozen whole; `%fx26-procedure?` reads the
  header, past a trailer or not, for tier 1's six kinds; `%fx26-mul` of
  two fixnums is `mul`, given back where `smulh` says it overflows, for
  tier 1's bignum. `mul` and `smulh` are encoded here, as `adr` and
  `lsl` are: `arm64.fx` has neither.
- **Then tier 6**: `direct-op?` takes a second flag beside `allocs`
  (C147, C211), `fuller`, for what else arm64's tier 5 does itself since;
  x86_64 passes `allocs` but not `fuller`, its tier 5 lacking these.
- **Counted after**: 52% of register words are tier 6's (2392 of 4591),
  85% of those not the scheduler's. Left, by words one instruction keeps
  out: `%fx26-make-icell` 37, `prompt` 13, `%stay-cellular` 13, `modulo`
  13, `abort` 12, `%fx26-error` 11, `exact-integer?` 11 (TODO §34).
  Give-backs over the 338 programs fell by 3% only: most are the fuel
  out at a slice's end, and the scheduler's.
- **Checked**: `cells-programs.py`, every way (tier 5 does these in every
  `--compiled` way now, tier 6 under `--direct`): the 338 programs agree,
  and the 85 of `--ours`. A product past a fixnum is given back and agrees
  (`native/i64-as-int.fx`).

## C213. The engine: `with` of a module value; an instance per maker's call (2026-10-26)

The user (2026-10-26): finish the engine. Of the programs its checker
accepts, it ran 2 fewer than 106 short of all; the first kind: names a
`with` opens not found.
- **A `with` of a module value or parameter** (a local, an item, a
  module's module): the engine's lowering knew the names a top-level
  module gives (`%exports`), not a value's. The checker notes them at the
  `with` (`with-spec`, C170); the engine's main path now passes them to
  the lowering among the globals' names (`%with-at FILE:POS x ...`, a
  slot never used, as `%exports` is). The REPL's paths do not yet.
- **An instance per call of a file of parameters' maker**: a load with
  parameters is spliced once, its module shared by every load of the same
  arguments (fine for checking); at run time Fixpt makes a module each
  time the maker is called (`modules/load-parameters.fx`: two makes, two
  counts). The engine now defines such a module (`%instance` among its
  `%loaded`) as its maker, and each `include` of it calls the maker once,
  its values taken from that instance (`%inst m`, an item of its own).
- The engine on Fixpt's programs beyond KFX: agree 141 to 154, not run
  87 to 74; differ 1 (`native/crossing-overflow.fx`, as before).

## C214. The engine: what a parser makes of a form, a few standard names (2026-10-26)

Programs the engine could not run for a name or a form it lacked.
- **Forms lowered as Fixpt's parser makes them**: `(letfreeze r ...)` as a
  `letregion` (frozen data is data all the same); `(confirm-nat e (n
  body) else)`, `(confirm-length e k (x body) else)`, `(acyclic e (x
  body) else)`, each a `let` of the value and an `if` of its test
  (`guarded-bind`). Each expansion's variable, and `case`'s key, named for
  what its form does not mention, as Fixpt's `fresh_name` names it
  (`fresh-for`; `sizes/fresh-temporaries.fx`, whose program binds the
  first choices around the forms).
- **Standard names written in FX-26** (`standard-written`), lowered where
  no primitive has the name: `assert`, `assume` (failing as Fixpt's do),
  `nat?`, `datum-int?`, `certify-nat`, `certify-acyclic`, `acyclic?` (no
  pair met again on the way down to it), `list` as a value (a `vlambda`),
  `native-code?` (false: the engine runs no native code), and
  `symbol-name-hash` as Fixpt's heap computes it (FNV-1a of the name's
  UTF-8, wrapped to a fixnum of 61 bits).
- **`define-convention`** defines nothing the engine runs.
- **`char-in?`** had a number in `table.kfx` and no case in `vprims.kfx`
  (a failure, "a primitive of no number"): added. `vprims.kfx` has been
  edited by hand since `gen-vprims.py` made it (failures said beneath
  primitives, C110), so it is not made again; the script now knows
  `char-in?` too.
- With C213: the engine on Fixpt's programs beyond KFX agrees on 171
  (from 141), runs not yet 57 (from 87); differs on 1, as before.

## C215. The engine: bloblets and blobrefs (2026-10-26)

The engine had no bloblet (the checker and KFX have had them since C31;
the engine, begun for the core and the REPL, C58, never took them on).
- **A bloblet**: an object of its own kind (`ob-bloblet`, scanned), its
  bytes, an `ob-bytes` (a byte a word, as a bytevector's), then its
  fields. `make-bloblet`, `rmake-bloblet` (the place, the heap here) a
  form, its fields any number; `bloblet-ref`, `-set!`, `-byte`,
  `-set-byte!`, `-bytes`, `-freeze`; `array?` true of one, `typecase`'s
  `bloblet` shape.
- **A blobref**, as kayak-rt's `BlobRef`: an object (`ob-blobref`) of
  its owner and its offset; made (`blobref-at`, `blobref+`) only where
  its flat lies whole in the owner's bytes, as FX-26 checks it; each
  scalar read and written little-endian (`i64`, `u64`, `f64` by its bits,
  `i32`, `u32`, `u8`, `i8`); `blobref-index` range-checked; a flat
  array's length to the end of the suffix as many as are left; a flat
  product's field by `extract`; a keyed flat sum's tag read and written
  as its number (`blobref-variant!`, `tagcase`), an unkeyed one's the
  index of the owner's field holding the label. Each operation's layout
  is the checker's fact there (`blob-size`, `blob-scalar`,
  `blob-element`, `blob-variant`, `blob-offset`, `blob-tagcase`, as the
  KFX emitter reads them), given the lowering as `with-spec` is (C213,
  `%fact FILE:POS ...`), and lowered to primitives of literal layouts.
- **Not yet**: `f32`, which the engine has no value of (a blobref's
  `f32` fails, saying so).
- **A bug of KFX's found**: a local `lambda` capturing a list of the
  engine's IR (whose nodes hold `Cell`s) was emitted as a closure
  `Send + Sync`, which rustc refused; worked around (the arguments given
  at each use), and in TODO §31 to fix.
- The engine on Fixpt's programs kfx accepts: agree 177 to 184 (not run
  19 to 12); beyond KFX, 171 to 182 (not run 57 to 46).

## C216. Kayak's compiler to cells: module values (2026-10-26)

The user (2026-10-26): grow the CEK engine and the ladder together, feature
by feature, as Fixpt grew; the ladder's gap is kayak's own compiler to
cells (`fx26/cg.kfx`, C133), which declined what Fixpt's listings give.
Module values first (the engine has them, C213).
- **Declined no more**: a module made as a value, and a `with` of one
  (C170); the `with` by the checker's `with-spec`, each name at its place
  in the module's type.
- **So each module value is laid out in its type's order**, as Fixpt's
  are (`k-reshape-at`): a module made where a module's type is wanted is
  reshaped to it (`module-arg`, now noted wherever a module is checked
  against a module type, not only for a module parameter, C24); one made
  where none is, to its own type (`module-made`, noted by `module` and
  `extend`); an argument, by the layout the generator knows it has, else
  the order of the type the checker found for it (`module-had`, noted in
  `check-by-synth`, which knows it: `check` gives back the type wanted).
- **A load of a file of parameters**, a module made again where it is
  included (`instance-load?`, as the modules made in place are, C90): an
  instance each time its maker is called, as in the engine (C213).
- **The checker's order of an included module's values was reversed**
  (they came newest first, and `bind-onto` reversed them again): the
  type of a module that includes one listed its values backwards, which
  `with` of it at a place by the type then read wrong
  (`modules/load-layers.fx`). Now in order.
- Lesson: `eng.sh`'s output hidden, a failed build left the old binary
  to test; not hidden since.
- Kayak's own words on Fixpt's programs: agree 292 to 310, differ 0;
  not compiled 83 to 65.

## C217. Kayak's compiler to cells: the confirming forms, generative families, shapes' tests as values (2026-10-26)

As the engine has them (C214):
- `(acyclic x (n body) else)`, `(confirm-nat ...)`, `(confirm-length x k
  ...)` made as Fixpt's parser makes them (`confirmed`): the value bound
  once to a variable named for what the form does not mention, tested by
  the standard `acyclic?`, `nat?`, `length-is?`, the name bound to it
  certified (`certify-acyclic`, `-nat`, `-length`), as Fixpt's words are.
- A generative family, `(define-generative (box (t type +)) ...)`, its
  conversions named for its head (`generative-name`).
- The shapes' tests (`int?`, `string?`, ...) of one argument as values.
- Kayak's own words on Fixpt's programs: agree 310 to 326, differ 0;
  not compiled 65 to 49.

## C218. Fixpt's scheduler, `processes.fx`, checked by kayak (2026-10-26)

Processes, the next feature for both of kayak's paths: in Fixpt the forms
of processes are made by its parser into calls of `fx26:processes.fx`, a
module file built in, the scheduler written in FX-26 over its runtime's
`%sched` (tasks run under prompts, switched by their continuations).
Tier 1 has the runtime's part (C125); what kayak lacked is the file, and
its own compiler's and engine's making of the forms. First the file:
- **Built in** (`scripts/built-in.py`): `fx26:processes.fx`, as it is at
  the pin, beside `standard.fx`.
- **Checked by kayak's checker**, which needed: a poly over a `place` in a
  type and a `plambda` (a region binder, its place marked for `(place
  r)`); `(place heap)`; `%sched` at Fixpt's type, named only by a module
  file built in (`built_in_only`; by name and by `proj`); and, in a module
  file built in, `(define icell-get await)`, a poly global's name at its
  scheme (elsewhere refused yet: KFX would have to emit it).
- Not yet: the forms made into calls of it (the engine, the compiler to
  cells), and the engine's `%sched`.

## C219. Processes on both of kayak's paths: the engine's scheduler, the compiler's forms (2026-10-26)

The forms of processes (`spawn`, `spawn-monitored`, `par`, `race`,
`pselect`, and the rest of `process-heads`) made into calls of
`fx26:processes.fx` (C218) as Fixpt's parser makes them
(`processes.rs`), once (`process-expanded`, lower.kfx), for both paths:
- **The program given the module**: `with-processes` (main.kfx) puts
  `(define %processes (load-module "fx26:processes.fx"))` before a
  program that uses a form of them, for the engine and for `--cells`.
- **The compiler to cells**: `parse-x` expands the form. A `proj` is no
  longer a name: Fixpt's `e-proj` is not one, so a call of `(proj %sched
  ...)` is a call of a value, `%sched`'s standard word (its arity, 3, now
  known), not `%fx26-sched` inline. Inline, the scheduler's own code ran
  with no call between its operations, which preemption at a slice's end
  depends on: at `--slice 2`, `par` and the rest hung. Now the words are
  Fixpt's, and every program of processes, monitors, preemption and
  schedules that tier 1 can run agrees, plain and at `--slice 2`
  (`pselect-key` needs `read-byte`, not yet tier 1's). (Rebased onto the
  arm64 work, 2026-10-26: C209 had made `proj` its own node, `x-proj`, to
  the same end, from macOS; its node is kept, this one's `x-begin`
  wrapper dropped.)
- **The engine's scheduler** (`fx26/vsched.kfx`, new): `cells/sched.kfx`
  ported onto the engine's heap, its state among the engine's constants;
  I-cells (`ob-icell`) and mailboxes (`ob-mailbox`), the messages
  shared, not copied (the checker has made them transmissible); `%sched`
  (207), `make-icell`, `rmake-icell`.
- **`call-with-oneshot-continuation`** (210) in the engine, captured as a
  composable continuation marked one-shot: resumed again, it fails with
  Fixpt's message (`run/oneshot-twice.fx`).
- **Failures in a monitored task**: every failure of the engine's machine
  goes by `failed-at`, which, in a task of a monitored group with the
  scheduler's prompt on the stack, aborts to it, the failure kept for the
  scheduler (`task-failed!`, Fixpt's `task_failed`); else the machine
  stops, as before.
- **Preemption**: the engine counts steps to a slice's end as Fixpt's
  machine does (`SLICE`, 65536); there, if the scheduler would have
  another task run, its `preempt` is called, quiet until it returns, under
  a frame (`fk-preempted`, Fixpt's `VmFrame::Discard`) that then
  evaluates the node it stopped at.
- Not yet: `%sched`'s standard input (op 43) on either path, with
  `read-byte`.
- Measured: kayak's own words, 358 agree, 0 differ, 16 not yet compiled
  (were 326 and 49); beyond KFX, the engine 208 agree (were 182), 1
  differs (`native/crossing-overflow`, as before).

## C220. Whole continuations, `f32`, flat arrays, eqtables and the rest, on both paths (2026-10-26)

The user: "Keep going", feature by feature on the engine and the ladder.
- **`cwcc`** on the engine (211): the whole continuation, every frame
  (an `ob-cont` of -1 frames), resumed any number of times by going on
  at its frames, the current ones dropped; the engine's frames are never
  changed, so none is copied. Continuations print `#<continuation>`, as
  Fixpt's printer does. Our compiler had it already (`callcc`).
- **`f32`** on the engine: an immediate, as Fixpt's, a special one past
  the machine's own (its bits above 2^32), so `eq?` compares bits; its 29
  operations numbered past the fixed widths' (`f32-base`, 3000), done by
  KFX's own `f32`; a blobref's `f32` field read and written.
- **Flat arrays** (`make-flatarray` and the rest, the layouts) on the
  engine: an `ob-flatx`, the elements' width then the elements, words
  (the engine's values are its own); printed as Fixpt prints one, by its
  bytes. KFX's own `flatarray-i64` keeps `ob-flat`.
- **Eqtables**, both paths, as Fixpt's `eqtable.rs` (Larceny's design,
  one stamp): hashed by address, every entry relinked, nothing allocated,
  by an operation that finds a collection since. The engine's an
  `ob-eqtable`; tier 1's a bloblet of Fixpt's kind `eqtable`, its chains
  vectors. (A vector's header counts its trailer among its fields: the
  buckets are one fewer, `eq-size`.)
- **`native-convention`**, both paths: Fixpt's convention for this
  machine, as kayak-jit keeps it (KFX's `(native-convention)`), in place
  of the engine's `(target none)`.
- **Variadic `append` and `string-append`** on the engine: a call of
  other than two arguments took the first two (`(string-append "a" "b"
  "c")` was `"ab"`); now folded to the right, and, as values, `vsubr`s
  written in FX-26 (`standard-written`), as `list` is.
- **`apply` of a list made cyclic** on the engine: Fixpt's failure,
  "expected a list", where it counted forever (`proper-count`).
- **Tier 1's primitives**: `%fx26-assume`, `%fx26-assert`,
  `list->string`, `string->symbol`, as Fixpt's runtime has them.
- Not yet: `%fx26-intern-datum` on tier 1 (Fixpt's hash-consing in a
  weak table); `%fx26-seal-code`, `native-procedure` (made code); the
  standard input (`read-byte`).
- Fixpt, to raise: on its default machine, `native/mixed.fx`'s last form
  shows `#<continuation>` where `fx-cells-rsinterp` and the engine give
  42 (TODO §23).
- Measured: kayak's own words, 367 agree, 0 differ, 16 not yet compiled
  (were 358); beyond KFX, the engine 221 agree (were 208), 2 differ
  (`native/crossing-overflow`, as before; `native/mixed`, Fixpt's), 6 not
  run (`native-procedure`, made code).

## C221. Kayak's compiler to cells: flat sums, quasiquotes, `vlambda`, case trees, `#%fx`, includes of expressions (2026-10-26)

The rest of what kayak's own compiler to cells could not compile, but
native code, each as Fixpt's compiler or parser has it:
- **A `tagcase` of a blobref to a flat sum** (Fixpt's `c-tagcase`,
  `c-flat-arms`): `x-tagcase` keeps the flat sum's layout code (`w + 8 *
  numeric + 256 * p`, from the checker's `blob-tagcase` fact; -1 for a
  sum's), its tag read once by `%fx26-flat-tag`, each arm's name a
  blobref to the variant; arms' labels numbers too (`arm-label`).
- **Quasiquotes**: Fixpt's parser's `quoted` (parser.fx), made as this
  compiler's expressions: unquotes, splices (`append`), and what no
  unquote reaches one `%quote`. **`vlambda`**: `%vlambda` of a lambda of
  one, as Fixpt's parser makes it.
- **The standard names, whatever a program binds** (Fixpt's `(with #%fx
  n)`): a name marked `#%fx:` (`std-var`) the standard one wherever it
  is: what quotes build with, a `case`'s and `typecase`'s tests, and a
  program's own `(with #%fx n)`.
- **`case`s of many data** (Fixpt's `case-tree`, TODO §46): the data
  sorted by their values (the integer, or the runtime's hash of a string
  or symbol, KFX's own `string-hash` and `symbol-name-hash` the same),
  the clause found by halving, then its body by halving the indices; the
  bodies at the leaves where each clause has one datum and the `else` is
  one atom.
- **An `include` of a module an expression makes** (a module literal in a
  module's module, or a procedure's result), both paths: the checker
  notes what it gives at the expression (`with-spec`), and the engine
  binds it to a hidden name of its place, `%incl FILE:POS`, then its
  values; our compiler makes it where it is included, once, as a module
  made in place. Our compiler takes `%module-value` (C170's) as a
  `module`, as the engine does, and makes a module's includes before its
  items, as FX-26 does (no program's words changed by it).
- Measured: kayak's own words, 381 agree, 0 differ (272 Fixpt's words
  exactly), 1 not yet compiled (a conversion to native code); beyond KFX,
  the engine 223 agree (were 221), 2 differ (as C220), 2 its checker
  refuses (`redefine/`, the REPL's), 6 not run (made code).

## C222. Made code on the engine: none, as Fixpt's evaluator (2026-10-26)

The user: "Native code tiers". First the engine's side of the made-code
programs (`made-code*/`, Fixpt's §97): the engine, as Fixpt's own
evaluator (`fx-ast-eval`), runs no code a program makes. `native-code?`
is `#f` (as before), and `seal-code` and `native-procedure` are now named,
failing with Fixpt's messages if called (233, 234). Fixpt's programs ask
`native-code?` and answer alike either way but for that line, so:
- **`run-fixpt-programs.py` compares the engine with Fixpt's evaluator**
  (`--fx26-machine fx-ast-eval`) on a program that asks `(native-code?)`,
  else with Fixpt's default machine as before (`engine_expected`). The
  Rust of KFX programs is compared as before.
- `define-convention` prints no line (`head_of`): the values of
  `made-code-x86_64/declared.fx` were read one line off.
- Measured: beyond KFX, the engine 229 agree, 2 differ (as C220), 2 its
  checker refuses, 0 not run.

Tier 1's side, surveyed, not begun: it calls no native closure
(`call!`), its accessors read `mem` alone, and its collector knows no
object outside `mem`; `native.md`'s decision 3 puts code bloblets in a
non-moving code area of the heap, mapped twice, as Fixpt's
(`exec_view` of `mem`'s code range). KFX has no way to give part of a
flat array an executable view: an extension, Fixpt's first.

## C223. Fixpt's answers kept with the machines and pins that gave them (2026-10-26)

The rebase onto the arm64 work (C209-C212) showed the gap: the shared
answers in tests/fixpt-answers/ were the home machine's (arm64 macOS),
stamped once for all by `REV`; another machine kept its own only where
they differed, stamped by `REV.PLATFORM`, and trusted the shared ones
whenever its own stamp was the pin. When macOS rewrote shared answers
(the x86_64 made-code programs, which arm64's Fixpt refuses), Linux,
its stamp the pin's, read arm64's refusals. The user: key each answer by
the machine and pin, "a multi-key per entry format to easily allow for
sharing".
- **One file per question** (NAME.HASH.json, HASH of the arguments and
  texts as before), holding each distinct answer with the keys that gave
  it, `PLATFORM@PIN`, sorted, one to a line. An answer counts only for its
  keys; another key is asked of Fixpt (with a clone at the pin) and added
  to the answer it gave, or a new answer kept. No stamps, no machine's own
  files.
- **Pruning** (`fx26-tests.py --prune`): a machine drops its own keys
  that nothing asked for, and its keys at other pins; an answer no key
  holds goes, and a question with none. Another machine's keys are its own.
- **Without a clone**, what is kept stands in: this machine's at another
  pin, else one kept before keys were, else any.
- **Converted once** (a script, run here and removed): the shared answers
  `unknown@8f76d65` (their stamp; which machine gave each was not kept,
  some Linux's), Linux's own `x86_64-linux@b9dae98`; and a shared answer
  Linux's last full check used with none of its own beside it, Linux's
  too, that check having asked Fixpt every question again (C222's refresh).
  Of 3001 questions, 2978 are Linux's at the pin; 23 only `unknown`
  (arm64's corpus); 30 have answers that differ by machine. The home
  machine's prune drops the `unknown` keys, after its own full check has
  added its keys.

## C224. The pin at Fixpt f03cabe; syscalls, on every path of kayak's (2026-10-26)

The user: "update to latest fixpt and look into leveraging the new syscall
system". Fixpt implemented docs/research/syscalls.md (its `f03cabe2`,
its copy of the note recording what it decided), after fixing the bug we
reported (`19973abf`: `ncallcc` among the register machine's control
call-outs, so `native/mixed.fx` gives 42 on its default machine too).
- **The pin moved to `f03cabe`.** Fixpt's history had been rewritten
  since `b9dae98` (the same commits, new hashes: `b9dae98` is `1e4d4ed6`,
  `ae397dc` `d28cc6d3`), so the `fixpt-source:` markers at the old hashes
  were remapped where the file is the same at the new one, and where not
  (`heap.rs`: `copy_graph`, which kayak does not port), to the pin.
  `cellular.rs`'s change (Fixpt's `CONTROL_CALLOUTS`) kayak's hand machine
  has no counterpart of: its control routines are given back to tier 1.
- **The checker** (shared by KFX and the engine): `define-syscall` and
  `define-optional-syscall`, as Fixpt's `syscall.rs` checks them, its
  messages word for word: only in a file built in or a program its driver
  says is a runtime's (`check-program-as`, the alias `%syscalls`; the
  engine's `--syscalls`); the host name; a `subr` or a `poly` over
  regions only; the effect (`(io sys)`, a new `io` kind); the types that
  cross; at most six arguments. Its type printed in canonical form
  (`syscall-canonical`), the same text as Fixpt's (its `other-type.fx`'s
  message, alike), noted as the fact `syscall N TEXT`; KFX refuses one
  yet. `(syscall-available? NAME)` is read as `%syscall-available:NAME`
  where programs are read (`with-loads`).
- **The engine and the compiler to cells** expand a declaration into
  Fixpt's two definitions (`syscall-definitions`); the engine's runtime
  has its own three syscalls, as Fixpt's evaluator does (`exec-view` -1,
  `flush` nothing; `page-words` by the native convention's target, KFX
  having no way to ask yet), and tier 1 the same, as Fixpt's runtime's
  primitives (`%fx26-syscall-require`, `-available?`, `%fx26-syscall`).
  Fixpt's five programs (`syscalls/`), run with `--syscalls`, give
  Fixpt's answers on the engine and on kayak's own words.
- Next (C225): kayak-rt's syscalls for KFX, so that tier 1, a runtime of
  KFX, asks for an executable view of its heap's range by
  `code:exec-view/1`, and makes code as Fixpt's heap does.

## C225. Syscalls in KFX, by kayak-rt (2026-10-26)

The user: go ahead with tier 1's code area on syscalls. First the
syscalls themselves for a runtime written in KFX:
- **kayak-jit** (where `unsafe` lives, `native.md` decision 4):
  `exec_view` (Fixpt's `exec.rs`: a range of a reservation `zeroed_cells`
  made, whole pages, never viewed before, replaced by shared pages, zero,
  and mapped again read+execute; `map_space`, the code area's own, now
  made of the same `exec_alias`), `flush_written`, `page_bytes`, and
  `call_mailbox` (below); a test that the two views share their pages.
- **kayak-rt's table** (`syscall.rs`, as Fixpt's `fixpt_runtime::
  syscall`): `code:exec-view/1`, `code:page-words/1`, `code:flush/1`, at
  their canonical types; `syscall_require`, `syscall_available` and
  `syscall0` to `syscall6`, generic over `Native` (a `u64` now crosses
  too; a bytevector not yet).
- **KFX**: a manifest's line `--syscalls` says its program is a
  runtime's own (`runtime-unit?`, `compile-units-as`, the memo path too),
  which may declare syscalls; KFX's checker no longer refuses them. The
  emitter writes a declaration as two `fn`s of its declared type (which
  must be written out as a `subr`): `%syscall-available:NAME` (`:` now
  mangled), a constant, and `NAME`, calling `kayak_rt::syscallN`. KFX's
  constants run where used, so a required syscall's check runs at its
  first use, not when the program starts.
- **Used**: the engine and tier 1 ask kayak-rt for the page's words
  (`code:page-words/1`), in place of guessing it from the native
  convention's target (C224); both units say `--syscalls`, their effects
  hold `(io sys)`.

## C226. Tier 1's code area: code a program makes, run natively (2026-10-26)

Then what the syscalls were for (`native.md`, decision 3; Fixpt's
"collected code area"):
- **The code area**: a reservation of its own (`code`, 4M words,
  committed as touched), its whole pages given an execute view by
  `code:exec-view/1` when the heap is made addressed; else none, and
  `native-code?` is `#f` (with `--indexes`). Not one reservation with
  both semispaces, as Fixpt's `mem` is (C225's plan): that would have
  moved every space's indexing; a code bloblet's index is instead any
  value's, from `vbase`, and `word-at`/`word-set!` read an index outside
  the heap's reservation from the code area, one comparison. Native code
  reads values as addresses (C139), so the native tiers need nothing.
- **The collector** leaves a code bloblet where it is (`forward-object!`)
  and forwards every code bloblet's fields in place, as roots, at each
  collection (`code-forwarded!`); none is freed yet (Fixpt's marks and
  sweeps the area).
- **`seal-code`, `native-procedure`, `native-code?`** as Fixpt's
  runtime has them: the copy's header, fields, trailer and suffix, its
  suffix frozen and flushed (`code:flush/1`); a native closure over it;
  `#t` where the area was made. **`call!`** calls a native closure as
  Fixpt's machines do: its code entered at its read+execute address by
  kayak-rt's `kayak:call-native/1`, its arguments and answer words in a
  mailbox (KFX's `int` crossing would not carry any word), a trap of
  native code tier 1's own; given back to tier 1 from tiers 2 to 6.
- **The checker** notes a call of a procedure of the native convention,
  written, `call-fx` (as a composable continuation's, C209), so that
  kayak's compiler makes it a `call`, not a typed `tcall`, as Fixpt's
  `c-call-in` does.
- `made-code-x86_64/leaves.fx`, `declared.fx` and `typed-calls.fx` give
  Fixpt's answers on tier 1, every way, on Fixpt's words and kayak's.
  Not yet: the convention's routines on tier 1's heap (`foreign`, `cons`,
  `bloblet`, `poll`, `closure`): kayak-jit's allocate in its own space and
  call Rust procedures, not tier 1's; native code that calls one fails as
  a trap tier 1 has not yet (`frames`, `generated`, `linked`,
  `allocated`). And a conversion to native code (`convention`).
  An object native code gives back that is in neither tier 1's heap nor
  its code area (made by kayak-jit's `bloblet` routine, in kayak-jit's own
  space) is refused as such a trap too (`tier1-word?`), not read.
- Measured: kayak's own words, 386 agree, 0 differ (282 Fixpt's words
  exactly; were 381 and 277); Fixpt's, 386 agree; 4 need what tier 1 has
  not yet (the routines).

## C227. The convention's routines on tier 1's heap (2026-10-26)

The user: "keep going", to the routines native code calls. kayak-jit's
allocate in its own space and call Rust procedures; tier 1's heap is its
own, of Fixpt's layout too. So a host heap for a call:
- **Allocation** (`cons`, `bloblet`, and kayak-jit's strings): in
  `call_mailbox`'s host heap (`m[10]`, `m[11]`: tier 1's top to its
  reservation's end), a thread's for the call, saved and restored around
  nested calls; tier 1 takes its top back from where native code left it.
- **Cards**: a card table reserved for the host's range (its two spaces
  and its code area, `m[12]`, `m[13]`), the state's `cards` biased to it
  for the call, so a store's barrier marks a byte, not anywhere.
- **`foreign`**, given a closure whose code is no number of kayak-jit's:
  the host's procedure call, which `kayak-cells`' `main.rs` gives once
  (`set_host_foreign`, by kayak-rt), and which runs `cells/machine.kfx`'s
  `native-callback`: the closure entered with its arguments, run to its
  return at the return stack's height it began (`run!`, or `hand-run!`),
  the machine's registers put back after; its failure the call's trap.
- **No collection while native frames are live** (`native-depth`): what
  they hold stays where it is, and the heap grows instead; Fixpt's machine
  collects there, by the frames' stack maps (`frames.fx`'s point), which
  kayak's tier 1 does not read yet. The answers are the same. (C229: it
  collects there now, by the stack maps.)
- **A trap said otherwise** comes back with what it said, a string made
  in the host heap (`allocated.fx`'s refused size).
- **The callback runs by the interpreter**, the native tiers off while it
  does: run on their machine, under the made code's live frames, it
  failed (a flat array read out of range, then an abort, since a panic
  cannot unwind native frames). Thought, wrongly: kayak-jit's state
  entered again, not re-entrant. (C228: the run's end; now on every tier.)
- All seven of Fixpt's `made-code-x86_64/` programs give Fixpt's answers
  on tier 1, every way, on Fixpt's words and kayak's.

## C228. Made code's callbacks on every tier: the run's end, not re-entrancy (2026-10-26)

The user, of C227's callback by the interpreter: "Make call re-entrant.
I'm totally freaked out that the architecture you have evolved has such a
fundamental limitation… downgrading to an interpreter should try its best
to switch back to running the native code it has available". So the
cause, found first:
- **C227's diagnosis was wrong.** kayak-jit's `call` nests already: its
  trampoline keeps the outer one's Rust stack pointer (`push qword ptr
  [rdi]`, restored on the way out), the per-call words are read before
  anything else runs (`foreign`'s arguments) or are the caller's to keep
  (`reg4`-`reg8`, lost across calls by the convention), and C227's host
  heap and cards are saved and restored. A backtrace showed the failure in
  tier 1's `pop-return!`, under `hand-run!`: tier 1's interpreter ended
  the callback's run where the return stack came back to its height
  (`run! base`), but the native tiers' machine knows no such height, and
  went on returning past the called procedure, into the frames of the
  computation that had called the native code, popping below them.
- **The callback now ends its run as a call ends**: a return entry to a
  word of one cell, `halt`, pushed first, the procedure called as `call!`
  does; every tier returns there and gives `halt` to tier 1, which stops.
  So the callback runs on whatever tier runs, natively under `--hand`,
  `--compiled`, `--regs --compiled` and `--direct`; its registers and
  both stacks put back after.
- **Re-entrancy stated as a test** (kayak-jit's `calls_nest_through_
  foreign`): native code calls Rust by `foreign`, which calls native code,
  which calls Rust again, three deep, each returning to its caller.

## C229. Collections under native frames, and code freed (2026-10-26)

The user: "Now work on the GC issues you noted", the two C227 and C228
left: no collection while native frames are live, and code made never
freed. Both as Fixpt's machine does them.
- **The native frames' values, kept by tier 1 across a callback.** The
  `foreign` stub gives kayak-jit's `foreign` the caller's frame pointer;
  `traced_slots` walks the frames from there to the trampoline's
  (`rust_sp`, plus the trampoline's own 48 bytes on x86_64), by the
  convention's `frames`: each frame's map at fp+16, its live mask below
  bit 48 (negative: all live), its values from fp+24, a stub's frame
  (32 bytes) none: Fixpt's `frames_from` and `traced`. The values go to
  the host's callback (`HostForeign` now takes them, and gives them
  back), which `native-callback` keeps on tier 1's data stack, below the
  callback's own work, where a collection finds and moves them; written
  back into the frames before native code runs again. The outer machine's
  registers are kept on the return stack, as before. So the deferral of
  C227 is gone: `safepoint!` collects whenever it would.
- **The called closure kept while its code runs.** `native-called!`
  pushes it on the data stack for the call: its code, reached through it,
  is then marked by any collection meanwhile (a frame keeps its own code
  bloblet in a value slot, `keeps-its-code`, for the same reason).
- **The code area marked and swept**, Fixpt's `mark_code` within its
  copying collection: a reference into the code area, met by
  `forward-object!`, marks the bloblet (its header found through its
  trailer, when the reference is to its suffix) in `code-marks`, one word
  per word of the area, and puts it on a gray list; the collection's end
  scans the copies (Cheney), then traces the gray bloblets' fields in
  place (`gray-traced!`, which may mark more and copy more), and again,
  until neither leaves anything. C226's code roots, every code bloblet's
  fields forwarded at the start, are gone: code nothing reaches keeps
  nothing alive.
- **Then the sweep** (`code-swept!`): each bloblet not marked made free,
  adjacent free blocks one, a free run at the end the area's top again,
  each free block written as a raw bloblet all suffix (Fixpt's
  `put_free`), so the area stays a walkable run of objects; the marks
  cleared. `seal-code` takes the first free block that fits
  (`code-alloc!`), what it leaves written free again, else the top.
- **Tests, kayak's own** (`tests/fx26-kayak/`, answered by Fixpt as its
  corpus is, `fixptcache.kayak_programs`):
  `made-code-x86_64/kept-across-collections.fx` (a made procedure's frame
  holding a pair across a callback of a million pairs' churn, some sixty
  collections: the pair moved, its frame's slot put back) and
  `made-code-x86_64/code-freed.fx` (twenty thousand adders of 4096 bytes
  made, called and dropped, the area holding some eight thousand; one
  kept from the first and called last). Fixpt gives both the same.
  Seen, by a trap put in for the purpose and taken out: the first's
  callback collects (200 times at three million pairs), its one held
  value moved. The second cannot fit without the sweep by arithmetic
  (513 words each); it was not run without it. The first's churn is a
  million, not three: tier 1 counts its interpreted callback's
  instructions against the default limit (twenty million), and three
  million pairs pass it where Fixpt's machine does not.

## C230. Adapters between calling conventions (2026-10-26)

The user: "go ahead and do that work" (adapters; my "conversion to native
code" was a loose name for them, and is not used again). `%fx26-convert`
as Fixpt's (`prim.rs`, `fixpt-native`'s `adapt`): a procedure already of
the convention asked for itself; one of the other, an adapter; anything
else itself.
- **Tier 1's adapters** (`cells/prims.kfx`): to the native convention, a
  native closure of three fields with the trailer, its code the
  adapter's, its free value the procedure (Fixpt's `native_closure`);
  the adapter's code made once into the code area (`mov rbx, [rbx-28];
  jmp [r15+288]` on x86_64, `ldur x10, [x10, #-28]; ldr x16, [x24,
  #288]; br x16` on arm64, Fixpt's `native_adapter_code`), marked by
  every collection as a root. To the cellular convention, a closure over
  the procedure whose word, made for it, calls its free value 0 in a tail
  call (Fixpt's `cellular_adapter`). Tier 1's own call of an adapter
  calls its procedure (Fixpt's `Heap::adapted`): so where no native code
  runs (`--indexes`), an adapter is made all the same, its code `#f`.
- **The code area as native code sees it.** Made code calls what a
  closure holds as code natively if it is at least the state's
  `code-lo`, plus the state's distance; else by `foreign`. kayak-jit's
  state had its own code area's, not tier 1's: a native closure of tier
  1's code area went by `foreign`, the adapter itself then given to tier
  1. The mailbox now gives both for the call (`m[14]`, `m[15]`),
  kayak-jit putting its own back after. And tier 1's code area must be
  above its two spaces, as Fixpt's is its heap's top, or a cellular
  closure's word would be taken for native code: tier 1 reserves a code
  area first (top down, Linux) and uses it if it is above both, else one
  reserved after them (bottom up, macOS; not run), else none. Checked
  once, when the spaces are reserved; the call's own test is the
  convention's one compare.
- **Kayak's compiler to cells** (`fx26/cg.kfx`): the checker notes, where
  Fixpt's converts at run time (`k-convert-at`), `convert K` at a value
  given where a procedure of another convention is expected, and
  `convention K` at a `(convention C e)`; `K` the arity times 4, plus 1
  for `cellular`, 2 for `native`. The program's convention, a `subr` that
  names none, counts as `cellular` there, as Fixpt's checker has it for
  the cellular machine (`k-conv-default`); the checker's own types are as
  they were (C181). The compiler gives the expression to `%fx26-convert`
  with `K`, as Fixpt's `exp_unnoted`. A call of a procedure of `fx`, which
  may be of either convention, is noted `call-fx` too: a call that looks
  at its callee's kind, not the typed `tcall`.
- **Tests**: Fixpt's `conventions/native-explicit.fx` now runs on tier 1
  (Fixpt's words and kayak's); kayak's `conventions/both-ways.fx` (Fixpt's
  `native/adapters.fx` but for its procedure defined twice, which tier 1's
  reader of listings refuses, and Fixpt's two compilers list apart) and
  `made-code-x86_64/adapter-called.fx` (made code calling an adapter
  natively, across collections).
- **The user, on the check**: the bloblet layout is meant to let it go:
  every code bloblet, a cellular word too, carrying in its suffix a stub
  of machine code that dispatches, so that native code calls any
  closure's code with no test. The check is the convention's (Fixpt's
  `declared-conventions.md`, `closures`), compiled into code a program
  makes: a change for Fixpt first, to be written up for its team.

## C231. Weak pairs, and quoted data interned on tier 1 (2026-10-26)

The user: "add weak hashtable support (and I guess %fx26-intern-datum to
boot)". Fixpt has no weak table a program makes: its weak table is the
one behind `%fx26-intern-datum` (`heap/intern.rs`, `TODO.md` §51;
`docs/research/weak-references.md`: "interning needs only weak pairs"),
which every quote that is not all literals calls. So this is that.
- **Weak pairs in tier 1's collector** (`cells/heap.kfx`): a bloblet of
  kind `weak-pair`; the scan skips its car, the first field it meets, and
  notes the pair; when everything else is traced, each noted car is made
  its object's copy, or `WEAK_DEAD` (fixnum -1) if nothing else kept it
  (`weak-after`, Fixpt's `weak_moved`), before the code area's sweep.
  Tier 1 finds a bloblet's header through its trailer when it has fields,
  so its weak pair has one (car field 3, cdr 2; Fixpt's has none, car 2,
  cdr 1): without it, first written so, the pair was copied as garbage
  and never noted, found by a trap put in and taken out.
- **The table** (`cells/prims.kfx`, `intern-datum!`), Fixpt's port: a
  vector of the collections' count when last hashed, the count, and
  buckets of weak-pair chains, a root of the collector's (`interned`);
  hash-consing bottom up (a pair by its interned parts, a string, float
  or vector by its contents, a vector's elements interned); rehashed after
  a collection, the dead dropped; grown at twice the buckets.
- **Kayak's compiler to cells** made a quote of all literals at compile
  time, as Fixpt's does, but each anew: `(eq? '(1 2) '(1 2))` was `#f`.
  Fixpt's interns it as it makes it (`c-quote-now`, `wcell-interned`), so
  equal quotes, and equal tails, are one object, written labelled
  (`#0=` … `#0#`). Now the same: the pairs hash-consed into nodes
  (`quoted-interned`), written as marks in a word's lines, which the
  listing's writing replaces (`labelled-listing`): each node's reaches
  counted in the order written, its parts at its first only, then written,
  labelled where reached more than once, numbered as written (Fixpt's
  `literal.rs`).
- **Tests**: Fixpt's `datum/quote-identity.fx` (outside the corpus, its
  two compilers listing apart) gives 7 on tier 1 every way, on Fixpt's
  first listing and kayak's; kayak's words are Fixpt's but for Fixpt's
  labels on two strings each written once. Kayak's
  `datum/interned-across-collections.fx`: strings, vectors and all-literal
  quotes one object across many collections, 63 everywhere.
- **The departures note** (`docs/research/departures.md`), for the review
  the user asked for (C230): fifteen places kayak's runtime departs from
  Fixpt's, those a program can tell first; gathered by a research pass
  over every entry, its most surprising claim checked in the code (made
  code's `poll` never stops it).

## C232. FX-26's I/O on every machine; one syscall, to wait for input (2026-10-26)

The user: "Then after that, add standard input. (Is that the only IO
you're adding) For much of this, e.g. standard input, could you get it
shipped faster if you leveraged the new syscall subsystem, so that you
could just implement the core functionality in the runtime?"
- **What FX-26 has** (`fx26.md`, "I/O"): the standard streams
  (`write-out`, `write-err`, `read-line`, `read-byte`, `read-bytes`,
  `stdin-ready?`), `exit`, `clock`, `sleep-until`, `args`, the files
  (`read-file`, `list-dir`, `write-file`), the terminal (`terminal-raw`,
  `terminal-restore`, `terminal-size`), and a `pselect` arm that waits for
  input (`%fx26-sched` 43-45). The engine had all but `read-byte`,
  `read-bytes`, `stdin-ready?`, the terminal and the wait; tier 1 had
  only `clock` and `sleep-until`. Now every machine has all of it.
- **Syscalls were not the faster way, mostly**: KFX has every one of these
  as its own operation already, which kayak-rt gives, and tier 1 and the
  engine are KFX programs: each calls KFX's of the name, its result made
  the machine's (tier 1: Fixpt's sums and products, `fx26_sum`,
  `fx26_io_result`; the engine: its words, reading where its room is
  found, as `read-line` did). Syscalls are for what FX-26 does not have.
- **One syscall, `io:stdin-wait/1`** (`(subr (io sys) (i64) bool)`): when
  no task can run but some wait for input, the scheduler waits until the
  input is ready or the earliest timer is due, whichever is first
  (Fixpt's `timers_wait`, `wait_stdin`): KFX has `stdin-ready?` and
  `sleep-until`, not both at once. kayak-rt waits on its reader thread's
  condition, with a timeout; the engine and tier 1 declare it, their
  wakers of the input run when it is ready, oldest first.
- **`stdin-ready?` asks the descriptor too**, as Fixpt's does
  (`tty::ready(0)`): kayak-rt's reader thread, only just started, had not
  yet read input already there, so `pselect-key.fx`, input given, chose
  the deadline (-1, where Fixpt gives 65). The `poll` is kayak-jit's
  (`stdin_readable`), kayak-rt being built with unsafe code refused.
- **Tier 1's program arguments** are those after `--`, as `fixpt run`'s;
  its effect `cm` gains the I/O atoms, the engine's `(io terminal)`.
- **Tests**: kayak's `io/streams.fx` (each operation as a test's run finds
  the world: the input at its end, no arguments, paths not there): 1023
  on Fixpt's two machines, the engine, tier 1 every way, on Fixpt's
  words and kayak's. The input given late, with a deadline after it, and
  not before the deadline: the byte, and the deadline's arm, on Fixpt and
  the engine alike, at the same times. The engine's `read-bytes` took its
  third argument for a count, not the span's end, first: found by the
  test.

## C233. I/O in bounded space; the departures reviewed (2026-10-26)

**The review.** The user went through `docs/research/departures.md`, one
departure at a time; each decision is recorded there and in `TODO.md`
(the review's list). Among them: made code's `poll`, its frames in a
collection and its routines that may collect, one piece of work (Fixpt's
frames flushed into the heap, Larceny's way); the step limit a budget per
top-level form, kayak's compiler marking each (`%form`); `eq?` on
immutable data, and a space-efficiency class (Clinger's), to be written
down as kayak's definition of FX-26, FX-26's guarantees and not KFX's;
code mapped twice kept, `MAP_JIT` a macOS option, the code-stubs request
for Fixpt's team (`docs/research/code-stubs.md`); the collector, regions
and kayak's tiers left for the revision from Coopt's runtime and code
generation, but the measured overheads (1.2-5x behind Fixpt's machines)
to be addressed before it; `--indexes` to go; Fixpt's backward scan for a
header to be tier 1's. The user, of the overheads: "Frankly I was
shocked".

**The I/O leak** (departure 14; the user: "Yes definitely fix the IO
leak"). Measured: tier 1's `write-out` 2M calls 71 MB peak, 200k 13 MB.
Three causes, each fixed:
- **kayak-rt made every I/O result anew**, never freed (`heap_alloc`):
  now those that carry nothing new are one value each (`done`, `eof`,
  `not-a-terminal`, promoted statics), a byte's one of 256 made once, a
  count up to 4096 one of a table; a failure, or text, or data, still
  made.
- **Each machine made a KFX string of each string it wrote, and KFX made
  one of each line read**, never freed. Now text crosses as code points in
  the machine's own words, by five syscalls of kayak-rt's
  (`io:write-points/1` from a flat array's range, one or two code points
  to a word; `io:read-line-points/1` a line read and kept, its length;
  `io:take-points/1` put into a string the machine made;
  `io:failure-kind/1`, `io:failure-detail/1` on the failure's path
  alone). kayak-jit reads and writes the words (`words_at`, `words_put`),
  kayak-rt being built with unsafe code refused. Tier 1 packs two to a
  word, the engine one; tier 1 writes a string not in its space (made
  elsewhere) by KFX's still; the engine's REPL, capturing output, by its
  text still.
- **kayak-rt's reader read the whole input ahead.** Now it waits while
  64 KiB are unread; a read signals as it takes, inside a line's loop too
  (else a line longer than the buffer would wait on itself: tested, a
  200 KB line).
- Measured after: tier 1, `write-out` of a non-ASCII string 200k and 2M
  calls 5.2 MB both, `read-line` 100k and 1M lines 5.2 MB both; the
  engine, `write-out` 35 MB and 34 MB.
- **The harness given an empty input**: the first check of this timed out
  (600 s) in `cells-programs`, which ran tier 1 with the check's own
  standard input; `io/streams.fx` waited, rightly, on an input left open
  (the last check's had been at its end). Fixpt's runs are given
  `/dev/null` (`fixptcache`, `run-fixpt-programs`); now `cells-programs`'
  and `engine-programs`' are too (unless given an input), so that a
  program reading the input gets its end whatever the check was started
  with. No check changed, only made not to depend on that.
- **Not done**: `read-file`, `list-dir`, `args` and `write-file` still
  make KFX data per call (their data's size each); to be moved the same
  way.

**Found on the way, not fixed here**: the engine's `read-line` loop still
grew, and so does any loop whose call is in a `tagcase` arm, with no I/O
at all: 10M iterations 930 MB and 35 s, 40M an abort, the 1 GiB stack
spent. Fixpt and tier 1 run it flat. `vm.kfx`'s `run` and `tagcase-step`
call each other, and KFX makes a mutual tail call a Rust call: the engine
has no proper tail call through a `tagcase`, against FX-26's, and against
C64's "never recurses on the Rust stack" (departure 3). `TODO.md`.

## C234. The pin at Fixpt 8c7111d: every procedure's code runs (2026-10-27)

The user: "When you get a chance, you can pull in the latest Fixpt from
this machine". Two commits past `f03cabe`: `ae7d767` (a REPL's form
tried as you type leaves no file it loaded) and `8c7111d`, kayak's
code-stubs request (C230, `docs/research/code-stubs.md`) done the same
night. Fixpt's way, which kayak takes: the code area in two parts, below
the state's `code-lo` every cellular word with the stub that enters
`foreign` as its suffix, from it native code, so that code written to the
older clause tests as it did; the clause `(code-always-runs)`, the older
one still accepted; words only run left in the heap, words made again and
again alike made once.
- **Tier 1's code area in two parts** (`stub-lo`..`stub-hi`, then
  `code-lo`..`code-hi`, half the pages each), each with its top and free
  list, both swept; the mailbox's `code-lo` the native part's. The stub,
  one word: `jmp [r15 + 288]; int3` on x86_64, `ldr x16, [x24, #288];
  br x16` on arm64, Fixpt's `layout::stub` byte for byte.
- **Words in the stubbed part** (`make-code-word!`): the listing's, the
  undefined procedure's; made once and shared, marked by every collection
  (`shared-words`): the continuation's closure's word, `vlambda`'s, the
  adapters' by arity: roots each collection forwards (a code word so
  marked; a heap word, where no native code runs, moved: first only
  marked, which left `--indexes`' caches stale after a collection, 26
  programs differing there until found). A callback's return word and the preemption
  trampoline, only run, stay in the heap. Where no native code runs
  (`--indexes`), in the heap as before.
- **Escapes through native frames**, which `stubs.fx` makes (a
  continuation taken outside made code, resumed by it): a continuation
  records how many callbacks deep it was taken (`cont-depth`); resumed
  deeper, a whole one is kept (`pending-k`, `pending-v`, roots) and the
  callback's run stopped; its error unwinds the native frames by
  kayak-jit's trap; `native-called!` reinstates it if it was taken at its
  depth, else stops its own callback's run, so that an escape leaves any
  number of native runs. Fixpt's `NativeExit::Throw`. Not covered: an
  abort to a prompt outside the callback.
- **`foreign` taking a native closure** (Fixpt's, for code of the older
  clause): not needed on tier 1, whose native closures' code is always in
  the native part, at or past `code-lo`.
- **kayak's checker**: a native procedure that may abort or suspend was
  refused for FX-26 too; now KFX's alone (`kfx-restricted`). **kayak's
  compiler to cells**: a standard operation as a value of any arity, by
  its type in the checker's table (`primitives`), not only those of a
  list kept by hand (`bitwise-not`).
- kayak-jit's declaration (Fixpt's, copied) now `(code-always-runs)`; its
  test says so. Its layout test compared kayak's closure code field (2)
  with Fixpt's `closure-code`, which is the bytecode VM's closure's (3
  since `8c7111d`): now with `cellular-closure-word`, what it meant.
- **The older clause accepted** by kayak's checker as by Fixpt's
  (`NATIVE_CONVENTION_OLDER`): a `define-convention` of the older
  `closures` clause in its place is the native one (`older-clauses`,
  `newer-clause`); else KFX refused its own made-code programs. Their
  golden Rust blessed; the five of arm64 (`made-*.rs`) still hold the older
  `convention.fx` text, to be blessed on arm64.
- **Tests**: Fixpt's `made-code-x86_64/stubs.fx` on tier 1 every way;
  kayak's `made-code-x86_64/stubs-escapes.fx` (its three answers in one,
  and a continuation resumed two callbacks deep), `(-42 42 42 42)` on
  Fixpt's two machines and on tier 1 every way, Fixpt's words and
  kayak's. Every other made-code program unchanged. arm64 not run.

## C235. A bless that would drop a program refuses, unless told (2026-10-27)

The user, asking what blessing is, then: "make --bless refuse to remove a
program from a passing list unless given an explicit flag (say
--bless-drop), naming each one it would drop." Blessing accepts what runs
now as the record: the generated Rust, checked in, and the lists of the
programs that agree with Fixpt (`tests/cells-passing`, `cellgen-passing`,
`engine-passing`, `engine-fixpt-passing`), whose answers are always
Fixpt's. A bless rewrote a list whatever it lost, so a regression blessed
was a regression kept: C234's first bless dropped the 26 programs its own
fault made differ, and only reading its output caught it.
- `fixptcache.drop_refused`: a bless whose list would lose programs names
  each (`WOULD DROP`), writes nothing, and fails, unless given
  `--bless-drop` (which blesses, and names each `DROPPED`). Growing a list
  stays a plain `--bless`. In `cells-programs.py` (both lists),
  `engine-programs.py` and `run-fixpt-programs.py`.
- Tested: a list given a program that is not there, `--bless` refused it
  (status 1, the list unchanged), `--bless-drop` dropped it.

## C236. The engine's steps in one loop: proper tail calls through every helper (2026-10-27)

The user, of the performance plan's first items: "go for it". Item 0a
(found by C233): the engine's `run` (`fx26/vm.kfx`) and its helpers
(`list-step`, `tagcase-step`, `apply-step`, `abort-to`, `gc-run`,
`failed-at`) called each other in tail position, and KFX makes only a
procedure's call of itself a jump: each helper's call of `run` was a Rust
call, which LLVM made a jump or not as it chose. Through `apply-step` it
did (a loop through an `if` ran flat); through `tagcase-step` it did not:
10M turns of a loop through a `tagcase` arm 930 MB and 35 s, 40M an
abort, the 1 GiB stack spent. Against FX-26's proper tail calls, and
C64's "never recurses on the Rust stack".
- Now each helper, where it called `run`, keeps the registers
  (`next-step!`: `go-regs`, `go-on`) and returns; at each of `run`'s
  calls of a helper, `run` goes on with them, a call of itself. 26 calls
  so changed, 9 sites. No collection between the keeping and the taking.
- Measured: 1M, 10M and 40M turns 33-34 MB each, 10M in 10.3 s (was 35);
  the engine's `read-line` loop (C233's), 100k and 1M lines 34 MB each.
  A test that would fail without it needs some 12M turns: left to the
  space-efficiency tests (`TODO.md`), which measure memory.

## C237. Fixpt's backward scan for a header; weak pairs as Fixpt's (2026-10-27)

Item 0b of the performance plan (departure 15, the user: "Yeah"). Fixpt
finds a bloblet's main header from its suffix (`main_of`): a trailer just
below says how far; a header there is it (no fields); else that word is a
field, and the header is found by scanning back to the first header-tagged
word, which no field can be (no value's tag is a header's, a trailer's or
a forwarding word's). Tier 1 took the word below for the header whenever
there was no trailer: right only with no fields, so a trailer-less
bloblet with fields was copied as garbage (C231's first weak pair), and
such a literal can come in a listing (`listing.kfx` reads a trailer only
where one is written).
- One lookup, `header-below` (and `header-index`, `header-of` by it), for
  the accessors, `forward-object!` and `weak-after`; `code-marked!` the
  same in the code area (`code-header-scanned`).
- Weak pairs as Fixpt's: two fields, no trailer, the car field 2, the cdr
  field 1; the interning test (`datum/interned-across-collections.fx`,
  every entry such a bloblet) 63 every way, `--gc-stress` among them.

## C238. Tier 1's first space a nursery's size (2026-10-27)

Item 1 of the performance plan. Tier 1's space began at 4096 words,
collected at three quarters, and doubled only when more than half lived
after a collection: a program of little live data collected every 3072
words (`lists` 2999 collections, `closures` 750, Fixpt's 6 each). Now,
where values are addresses (the space reserved whole, committed as
touched), the first limit is 2^20 words, Fixpt's nursery's.
- Measured (best of 3, ms; tier 1, tier 2, tier 6): `lists` 1826 211 58
  before, 1832 196 43 after, 7 collections, 22 MB at most; `closures`
  2569 285 85 before, 2560 264 68 after, 7 collections, 25 MB. The rest
  unchanged (no collections), at most 10 MB.
- **A guess corrected**: the review's table (C233) put most of `lists`'
  and `closures`' gap at tier 6 (4-5x behind Fixpt's native machine) on
  collections; they were about a quarter of it. The rest is per
  allocation or per call: item 2, a profile, next.

## C239. Machine code named for profilers; where tier 6's time goes (2026-10-27)

The user, asked whether kayak supports `perf` and debug info for FX-26's
compiled code: "we should do both of those things; I'm especially
interested in what you can do for debuginfo support for FX26 itself". It
did not: the native tiers' code ran in anonymous mappings, `perf`'s
`[unknown]`.
- **A perf map** (Fixpt's `symbols.rs`, `FIXPT_PERF_MAP`): two syscalls of
  kayak-rt's, `kayak:note-code/1` (start, length, name: with
  `KAYAK_PERF_MAP` set a line of `/tmp/perf-PID.map`) and
  `kayak:last-sealed/1` (where the code KFX's `seal-code` sealed last
  runs, kayak-jit's `last_sealed`). Tier 2, once its machine is sealed,
  names it up to the first compiled word, and each compiled word from its
  first cell's label to its last's: tier 4's `fx26 NAME`, tier 5's
  `fx26 NAME (tier 5)`, tier 6's `fx26 NAME (tier 6)`, on x86_64 and
  arm64 (labels in bytes and in instructions).
- **A `profiling` Cargo profile**: the release build with line tables.
- **The levels of debug info for FX-26**, agreed with the user: names (this);
  jitdump with code bytes, then with source lines from line tables
  kayak's compiler writes into its listings; a profile of tier 1 by FX-26
  source line (`--profile`); the GDB JIT interface not now.
- **Measured, the plan's item 2**: run long enough (`tak 30 21 12`;
  `lists`' `rounds` 60000), tier 6 is 2.8x behind Fixpt's native machine
  on `tak` (103 ms, 36) and 1.7x on `lists` (409 ms, 241), not 5x and
  4.3x: the short benchmarks' times held kayak's start-up (the
  reservations, tier 2's machine encoded at each start), which Fixpt's
  "run alone" leaves out. And of those runs, 93% (`tak`) and 95% (`lists`:
  `iota` 73%, `add-up` 22%) is in tier 6's own code: hand-backs to tier 1,
  the collector and the runtime's Rust under 2% each. The gap is tier 6's
  code's quality; which of its instructions, a jitdump (code bytes for
  `perf annotate`) will say.

## C240. A jitdump: the native tiers' code annotated (2026-10-27)

The second level of FX-26's debug info (C239): Linux `perf`'s jitdump.
With `KAYAK_JITDUMP` set, kayak-jit writes `/tmp/jit-PID.dump` (its
`jitdump` module; unsafe code, which kayak-rt refuses): the header, then,
for each piece of code named (C239's `kayak:note-code/1`), a
`JIT_CODE_LOAD` record of its address, size, name and bytes; the file
mapped executable once, which is how `perf record` notes it; times by the
monotonic clock (`perf record -k 1`). Linux only; elsewhere nothing.
Then `perf inject --jit` and `perf annotate` show tier 6's instructions.
- **What it shows of `tak` on tier 6** (C239's 2.8x): no one hot place,
  the time spread over code of a costly shape: a bounds check before every
  data-stack slot's load or store (tier 1's, carried into the code); the
  arguments written into the frame's slots on tier 1's data stack and read
  back, not kept in registers; a frame's slots made one store and one
  `add` each; a comparison made a boolean (`cmov` of `#t` and `#f`), then
  compared with `#f`, not a compare and branch; fuel counted at several
  blocks of each call. Each a choice of tier 6's code generation, to be
  improved before the Coopt revision (`TODO.md`).
- Source lines (`JIT_CODE_DEBUG_INFO`) next: kayak's compiler's listings
  to carry them.

## C241. Tier 6's code: three of C240's costs gone (2026-10-27)

The user, of C240's findings: "lets do those four clear changes". Three
here, each measured alone on C239's runs (`tak 30 21 12`; `lists`, 60000
rounds; tier 6, best of 3); the fourth, arguments in registers across
calls, is the call protocol's, next.
- **No bounds check on a slot `save`'s frame holds.** `stack`, `setstk`,
  `load` and `store` checked each slot against the data stack's top; where
  the word's `save m` made the frame (`h-link`, known at code generation)
  and the slot is below `m`, no longer (tiers 5 and 6, both machines). The
  register code is a compiler's, kayak's or Fixpt's, not a program's.
  `tak` 103 ms to 89.
- **A frame's slots made with one move of the stack** (x86_64; arm64's
  were one store each already, post-indexed). 89 to 85.
- **A comparison and the branch on it made one** (Fixpt's `branch_on`):
  in tier 6's code, where the branch is no block's first and both ways
  on begin with an instruction that sets `RESULT` before reading it
  (Fixpt's `sets_first`), a `cmp` and a conditional jump, no boolean made
  (`fused-branch`, `yfused!`; x86_64 only for now). `tak`'s code has no
  `cmov` left; `lists` 404 to 381, `tak` unchanged (its calls dominate).
- In all: `tak` 103 to 87 ms (Fixpt's native machine 36), `lists` 409 to
  381 (241). Fuel is still counted at each block: counted once a call, it
  would trip where tier 1's does not (departure 2's rule).

## C242. Tier 6 keeps what registers hold of the frame (2026-10-27)

The fourth of C241's changes, in its contained part. `tak`'s register code
stores its arguments into the frame and reads them straight back
(`store 1 0` … `load 1 0`, `stack 1`): the compiler's code, not tier 6's
protocol. Fixpt's native compiler keeps, in straight-line code, which
slot each register has (`held`, `direct/x64.rs`): "a read of the slot is
then a move". Now tier 6's too (x86_64): `h-held`, for `REG1` to `REG3`
where they are machine registers; a `store` notes it (the frame still
written: the collector, callbacks and tier 5's code read it), a `load`
of what the register holds makes nothing, of what another holds a move,
a `stack` of it a move; all forgotten at a block's first (where tier 5's
code comes back in), after anything that may change a register, and at
each word's start (else a word's last knowledge would reach the next's
code, which begins at instruction 2, not always a block's first: caught
before it was run).
- Measured: `tak` 87 ms to 84, `lists` 381 to 377. Since C239: `tak` 103 to
  84 (Fixpt's native machine 36), `lists` 409 to 377 (241).
- **What is left of the gap is the call itself**: each call's frame on
  tier 1's data stack, a record on the machine stack, the link and its
  reloading, fuel at each block. Frames on the machine stack, arguments
  kept in registers across the call, are the call protocol's change, with
  the frame-flushing work (departures 1, 5, 6; `TODO.md`).

## C243. Start-up: the stacks reserved, not written (2026-10-27)

The user: "lets tackle 'tier 2 encoding its whole machine on every start,
which dominates short runs' next". Measured first (a one-form program,
best of 40, against `/bin/true`'s 2.27 ms): tier 1 1.6 ms above it, tier 2
0.3 more, tier 6 0.6 more. So tier 2's encoding never dominated: a guess
of C239's, corrected.
- **What did**: kayak-rt wrote every flat array of 0s under 2^20 elements
  element by element, and tier 1's data stack (132096 words, 1 MB) and
  return stack (262208, 2 MB) are under it: 3 MB written, some 800 pages
  faulted, at every start, of which a run uses a little. Now from 2^16
  elements such an array is reserved (`zeroed_cells`), committed as
  touched; not lower, each reservation a mapping of its own, of which a
  process may have some 65000. Tier 1's start 1.3 ms less (5.56 to 4.25,
  best of 10); the short benchmarks on tier 6 3-7 ms less each (`fib` 25
  to 19 ms, `tak` 17 to 13, `loop` 28 to 23, `lists` 43 to 40, `closures`
  68 to 61, `helpers` 29 to 25).
- **What is left**: 1.6 ms over an empty process, of which kayak's own
  code is about 1% (`perf`), the dynamic loader and libc 5%, the kernel the
  rest: loading and relocating a large binary, the big stack's thread
  (148 page faults to `/bin/true`'s 65; some 100 system calls). Static
  linking or a binary not position-independent might take some of it:
  changes to the whole build, for little; not done.

## C244. `--indexes` gone: values are addresses (2026-10-27)

The user, in the review of the departures (departure 13): "Let's kill
`--indexes`"; then "lets remove `--indexes`". C139 kept tier 1's values
as indices into its heap as a mode beside addresses, run by
`cells-programs` three ways (`--indexes`, `--hand --indexes`, `--indexes
--gc-stress`): coverage the address mode's own runs give, and a mode in
which no code a program makes could run.
- **Tier 1**: values are addresses, always; `heap-addressed!` at every start,
  and where it cannot be (no native code, so no array's address to be
  had: neither of kayak's machines) tier 1 refuses to run, saying so,
  rather than fall back. The `addressed` flag and its branches gone: the
  heap's growth by copying to a bigger array (`room!`), the other space's
  resizing, the limit reset after a collection; on x86_64 the machine's
  loads, `lea`s and values by the base register (`yld-v!`, `ylea-v!`,
  `yvalued!`, the allocation's index, the base's set-up), and `REG3` in
  `r10` always. arm64's machine keeps its base register (`x19`, 0 in
  address mode): its code unchanged, as arm64 runs nothing here; to go
  when it is run.
- **The check**: the three ways gone from `cells-programs` (the user's
  decision; C244 is it). Fixpt's heap images were indices, but a dump of
  addresses normalizes them as it writes (the user, departure 13).


## C245. Source lines, and tier 1's `--profile` (2026-10-27)

The user: "sure, okay, lets add source lines and a tier 1 --profile for
debug info", after C240's jitdump showed tier 6's code with no FX-26
line to tie it to.
- **Line tables**: kayak's compiler to cells notes, as it compiles each
  expression, where the expression's code begins (an item of no cells,
  `it-pos`), and, after it, its parent's position again, so that what
  the parent's own code does (a call's cell after its arguments) is the
  parent's. After each word's lines, the listing has
  `; at FILE CELL:POS …`: an entry where the position changes, the last
  of several at one cell. A comment at column 0: what compares listings
  with Fixpt's passes it over, and Fixpt's listings have none. `FILE` is
  the program's (`cg-source`); a word from a module file of its own is
  not told apart yet. Positions are characters, as the reader counts.
- **`kayak-cells --profile`**: tier 1 only (refused with `--hand`,
  `--compiled`, `--direct` or `--regs`, whose code tier 1 does not step).
  `cell-step!` counts each cell by its word and its place there, in an
  open-addressed tally (words never move, in the code area, so an address
  is a key); one test of a flag per cell when not profiling. At the end,
  on the standard error, even when the program fails: the cells run; the
  source lines and the expressions (LINE:COLUMN and the expression's
  text) that ran the most, each cell given its word's last table entry at
  or before it; and the words. `tak 22 16 8`: 12,000,329 cells, 0.41 s to
  0.36 s without; `(not (< y x))` 15%.
- **Next**: the jitdump's debug-info records for tiers 4-6 (TODO).

## C246. Tier 4's code given its FX-26 lines, for `perf` (2026-10-27)

The user: "Go for it", C245's next step: the jitdump's debug-info
records, so that `perf annotate` shows FX-26's lines beside the native
code (C240 gave it the code, with names only).
- **What is noted**: when tier 2 seals its code, each compiled word's span
  (`span`, now with the word) is looked up in the listing's line tables
  (`table-named`; the first listed of a name listed again, as its word is
  made). Each table entry at or before the word's last cell gives a pair,
  the address of its cell's code (the cell's label, `ls`) and its source
  position, and the span is noted by a new syscall,
  `kayak:note-code-lines/1` (start, length, name, file, pairs, count),
  in place of `note-code`.
- **kayak-rt**: with `KAYAK_JITDUMP` set, the positions are made lines of
  the file (its line starts counted once per file, in characters, as the
  reader counts) and a `JIT_CODE_DEBUG_INFO` record, the file's path made
  absolute, written before the code's `JIT_CODE_LOAD`, as the format asks.
  If the file cannot be read (a listing's relative path, from another
  directory), no record: no lines rather than wrong ones. Without the
  variable, as `note-code`; the perf map unchanged.
- **Measured**: `tak 24 16 8` on tier 4, `perf inject --jit` then
  `perf annotate -l -s "fx26 tak"`: the code under its source line,
  `tak.fx:3`. `lists` (3000 rounds): `perf report --sort srcline`, 46%
  `lists.fx:3` (`iota`), 38% `lists.fx:5` (`add-up`).
- **Not given lines**: tiers 5 and 6 run register code, which only
  Fixpt's listings have, and they have no line tables; kayak's compiler
  makes no register code yet. Columns: perf's injected ELF keeps lines
  only, so an expression's place within a line is `--profile`'s (C245),
  on tier 1.

## C247. `KAYAK_JITDUMP=expr`: perf by expression (2026-10-27)

The user, of C246's lines only: "I wonder if there's some insane hack
like encoding the source L:C as a virtual-line … and then map the
results back ourselves"; then that splitting by expression is often
what is wanted, an opt-in, and that a value out of the file's lines
might say it is encoded; and "surely KAYAK_JITDUMP=expr is a better
opt-in signal?".
- **The encoding**: a jitdump's line is 32 bits (perf's handling of it
  signed, so under 2^31). With `KAYAK_JITDUMP=expr`, a record's line is
  N + 2 + P, P the character position where the expression's code
  begins and N the source's lines: past N + 1, so it says itself it is a
  position, whatever reads it.
- **The side file** the records name, `/tmp/jit-PID.NAME.expr` beside the
  jitdump: the source's N lines, verbatim; a line saying so (`;;; kayak
  expr (C247): the N lines above are PATH's; …`); then line N + 2 + P,
  `LINE:COLUMN  TEXT` where a token may start at P, else empty. So
  `perf annotate -l` shows each expression's text over its code with no
  decoding, and a plain line (1..N) still reads as the source's.
- **`scripts/perf-expr.py PERF_DATA [--lines] [-n COUNT]`**: runs `perf
  report --sort srcline --full-source-path` and reads each line back as
  `PATH:LINE:COLUMN` and the expression; `--lines` sums the expressions
  of each source line, as plain `KAYAK_JITDUMP=1` would report. `lists`
  on tier 4: `(add-up (cdr xs) …)` 11.6%, `(iota (- n 1) …)` 9.3%; by
  line 41% and 39%.
- **`docs/profiling.md`**: the four ways together (`--profile`, the perf
  map, the jitdump, `expr`).
- Not run by `check.sh`: it needs `perf` and its permissions.

## C248. Register code from kayak's own compiler, stage 1 (2026-10-27)

The user: "Yes I've been waiting days (weeks?) for register code in
Kayak"; and, while it was planned, "If you could try to make your own
architecture more modular compared to Fixpt, it would probably yield big
dividends". The plan and its stages: `docs/research/register-code.md`.
- **Why**: tiers 5 and 6 run register code, which only Fixpt's listings
  had, so a program kayak compiled stopped at tier 4.
- **Fixpt's output the oracle, not its structure.** Fixpt's generator
  (`regcode*.fx`, 5,100 lines) is one recursive group whose state is
  globals set and restored around calls. Kayak's is six layers, each on
  those before it: the instruction set (`kfx/regs-isa.kfx`, shared with
  tier 1, whose own copy it replaces), an assembler that knows nothing of
  FX-26, places and parallel moves, the analyses (standard operations,
  constants, tests, whether an expression may collect), the expressions'
  compiler, and a lambda's entry (leaf or framed, fast and plain
  versions). The state is a record passed along (`rgen`, its `rctx`):
  what is made, what the body assumes. Each feature not ported is one
  `r-decline`.
- **The stack compiler's side**: each lambda's word noted as a twin
  (`cg-twins`: its parameters, body and scope, what it knows of itself,
  the definition it is the value of, where the lambda is); the form loop
  and the listing split into `fx26/cg-top.kfx`, after the layers, which
  makes each form's twins after its words, as Fixpt's form loop does, and
  counts each global's writes, which a guard expects (by name, as tier 1
  keeps its globals).
- **What the checker gives it**: a call of a native procedure noted
  `call-native` beside `call-fx` (its `invoke`'s convention, 2); and a
  `define*`'s body that may suspend noted `suspends` at the definition, as
  a lambda's is at the lambda (where the fast version is unsound, Fixpt's
  summary 3). A written `(with #%fx n)` marked apart from the parser's own
  standard names (`#%fx!:`), as Fixpt keeps its `with`, which its
  register code takes for no operation in line.
- **Measured**: every program agrees on every `--regs` way
  (`cells-programs --ours`, which ran them before on Fixpt's register code
  only). Of the 292 programs whose stack words are Fixpt's, 523 register
  words are Fixpt's exactly, 72 differ, 241 declined, none kayak's alone
  (`cells-programs --ours` now says so; `--regs-differ` names them).
  `tak`'s is Fixpt's, all 241 cells; tier 6 on kayak's `tak 30 21 12`,
  0.07 s, as on Fixpt's. The differences: 68 inlining and constant globals
  (not yet), 2 Fixpt's `the` kept as a node, 2 its specialization's
  registry; each in the plan.
- **One mistake, undone**: an exploratory command ran `git stash` in a
  loop; the stash it made was popped at once, the work whole.

## C249. Register code's line tables: tiers 5 and 6 under `perf` (2026-10-27)

The user, offered inlining or line tables next: "Do line tables first. I
want us to be on top of performance from the outset now".
- **The generator** notes, as the stack compiler does (C245), where each
  expression's code begins and where its parent's resumes (`ri-pos`, an
  item of no cells; the position kept in the context, `rctx`); a body's
  entry, after `args` (its frame made, its parameters stored, a fast
  version's guards), is the lambda's own position. The listing writes, after
  a word's register code, `; regs at FILE INSTRUCTION:POS …`.
- **Tier 1's reader** keeps those tables apart (`reg-line-tables`), and
  tier 2 notes each register word's spans with them, as each cellular
  word's (C246): a span now says whose code it is, a cellular word's or a
  register word's, and where its cells' or instructions' labels start
  (tier 6's span begins past `args`); an entry whose instruction's code is
  outside the span (fused into the one before it, or before tier 6's
  start) is left out.
- **Measured**: `tak 30 21 12` on tier 6, `KAYAK_JITDUMP=expr` and
  `scripts/perf-expr.py`: 35% in the lambda's own line, its entry; in
  `perf annotate`, a fuel count, the guard, a second fuel count, three
  stack bounds, six slots and a header written, three parameters stored.
  The call protocol's cost, which the frame work is to take away
  (TODO, performance item 3); `(< y x)` 11%, the calls 6-9% each.

## C250. A register word's entry, part by part, at the letters of `(lambda` (2026-10-27)

The user: "Can you get finer grain data on [the entry's 35%] … maybe by
attributing each part to distinct letters in l a m b d a ?"
- **The positions**: a body's entry parts each at a character of its
  lambda's `(lambda`, P its position: the entry itself at P (`(`), a fast
  version's guards at P+1 (`l`), the frame made at P+2 (`a`), the first
  three parameters stored at P+3 to P+5 (`m`, `b`, `d`), any more at P+6.
  Lines are as before (all the lambda's line); `KAYAK_JITDUMP=expr` tells
  them apart. Its side file now gives every position but a blank's its
  text, not only where a token may start, so that each letter has one.
- **Measured** (`tak 30 21 12`, tier 6): the entry 33% of the samples:
  the frame made 18.6% (a fuel count of the block, three stack bounds,
  six slots written `#f` and a header word: these writes about 14%),
  the guard 7.8% (a fuel count, the global loaded, its writes loaded, the
  count expected loaded, not an immediate, and compared), the stores
  1.3%, 2.6%, 2.9%. The frame made is `tak`'s one dearest part, above any
  expression of its body: what the call protocol's work starts from.

## C251. Tier 6's entry made cheaper: fills, the guard, fuel (2026-10-27)

The user, of C250's next steps: "Yeah do 1, 2, then 3"; the first, the
entry's quick costs C250 measured.
- **Fewer fills**: tier 6's `save` fills with `#f` only the slots the
  run of `store`s after it does not write (`stored-after-save?`): a run
  in the same block, so that nothing between may give back or collect;
  tier 5, which counts each instruction's fuel and may give back between
  any two, fills them all. `tak`: three slots filled, not six.
- **The guard's count an immediate**: what a `global-guard` expects is
  fixed when its code is made: compared as an immediate (arm64's where it
  fits 12 bits), not loaded from the word.
- **The entry's fuel counted once**: a `global-guard`'s fall-through no
  longer begins a block (`t6-leaders!`); where it holds, on in its block,
  one count; where it fails (its global written again), the rest of its
  block counted too, a few steps more, never less.
- **Measured** (`tak 30 21 12`, tier 6, `perf stat -r 10`): 335 to 286
  million cycles (-14.6%), 1,367 to 1,296 million instructions; the entry
  33.2% of the samples to 26.9%, the frame made 18.6% to 14.5% (its
  three stack bounds the most of it now), the guard 7.8% to 6.2%.
- **arm64**: the same, made but not run here (TODO's arm64 items).

## C252. Register code inlines small procedures, folds constant globals, unrolls (2026-10-27)

The user: "Yeah do 1, 2, then 3"; the second, inlining and constant
globals in register code (68 of C248's 72 differences from Fixpt's).
- **Inlining** (Fixpt's `c-record-inline`, `r-inline`): a typed
  definition of a lambda whose body is small (at most 20 nodes,
  `r-room`), names neither itself nor `stay-cellular`, noted
  (`cg-inlines`) as it sees the globals defined so far. A call of it with
  its arity: the arguments made and kept (one a slot, a free value or a
  constant used where it is); a guard on its global, unless a fast version
  assumes it; the body in a scope of its own, its parameters the
  arguments; where the guard fails, the call.
- **Unrolling** (Fixpt's `c-note-unroll!`, `r-unrolled`): one small but
  calling itself, whose definition neither suspends nor runs an effect
  not known, inlined where called with a constant list, at most 16 deep,
  no guard within its own. Kayak's constant lists are only `nil` yet.
- **Constant globals** (Fixpt's `c-note-constant!`, `r-consts-named`): a
  definition's value a literal, or a constant global's, noted
  (`cg-consts`); a fast version folds those its body names, each assumed
  (a guard at its start).
- **Modular, as C248**: what Fixpt keeps in globals set and restored
  around an inlined body (`c-genv`, `c-inlining`, `r-consts-now`) is said
  in the register environment, by names no program has: `%genv` (the
  globals a body sees), `%inlining:G`, `%const:G`; the analyses read it
  there, as any binding.
- **Measured**: of the 292 programs whose stack words are Fixpt's, 573
  register words Fixpt's exactly (from 523), 11 differ (from 72), 252
  declined (an inlined body declined declines its caller). Tier 6 on
  kayak's listings as on Fixpt's: `fib` 59.8 to 59.8 million cycles,
  `tak` 38.6 to 37.8, `lists` 136 to 133, `helpers` 86.3 to 85.7; `loop`
  124 to 85 and `closures` 251 to 218 behind, for what stage 2 does
  (closures, `letrec`).
- **Not yet**: globals holding constant lists (`unrolled-lists.fx`: the
  checker's note of a list nothing writes, Fixpt's `c-const-list`, pairs
  and symbols as constants), a module's member re-exported and inlined
  (`reexport-inlined.fx`).

## C253. Register code, stage 2: closures, `letrec`, `tagcase` and more (2026-10-27)

The user: "Yeah do 1, 2, then 3"; the third, stage 2 of
`docs/research/register-code.md`.
- **Closures**: the stack compiler notes, per form, each lambda's word
  with its parameters, own name and captures (`cg-form-lambdas`); register
  code closes over the oldest made for the same body (Fixpt's
  `c-made-word`): its free values moved into REG1…REGn (in parallel, those
  in registers), or past `register-regs` as a call-out's operands; a
  sibling not made yet given as `#f`, to be patched.
- **`letrec`**: each closure made into its slot, then each placeholder
  patched (`setfield 3+j`); join points (Fixpt's `c-join-ok?`: a lambda
  only called in tail position, there and in the `letrec`'s body, no
  sibling mentioning it) no closure: their parameters in registers or
  slots, their calls jumps, their bodies after the `letrec`'s.
- **And**: `tagcase` (not of a flat sum), `apply` (as the stack code, no
  copy), `list`, sums and products (`%make-frozen`), and those of
  constants made when compiled, `#bloblet(…)` literals as Fixpt's
  (`l-frozen`); prompts (the body a thunk); a lambda applied at once a
  `let`; a mark in tail position; more parameters than registers; the
  standard operations' words as values their own register code (Fixpt's
  `r-standard-word`).
- **`(the T e)` kept as a node** (an `x-proj`, no code): Fixpt's register
  code sees its `e-the`, an expression not simple and a node counted; with
  it kayak inlines what Fixpt inlines, and makes arguments in its order.
  The stack code unchanged (292 programs' Fixpt's exactly, as before).
- **Measured**: every program agrees on every `--regs` way; of the 292
  programs whose stack words are Fixpt's, 733 register words Fixpt's
  exactly (from 573), 9 differ, 94 declined (from 252). The 9: globals
  holding constant lists (5), a module's member re-exported (2), Fixpt's
  specialization's registry (2). Declined: bloblets, regions, modules,
  `with` (stage 3). Tier 6, kayak's listings as Fixpt's: `loop` 85.6 to
  85.2 million cycles (from 124); `closures` 253 to 216, Fixpt's
  specializing `map-add` at the lambda it is given (stage 4).

## C254. KFX a sublanguage of FX-26: Fixpt accepts each of kayak's KFX units (2026-10-27)

The user, on Fixpt refusing kayak's engine's own source: "KFX is meant to
be a sub language of FX26. I think when i said divergence was allowed, I
meant that the generated code might differ. But surely any KFX program
accepted by Kayak should likewise be accepted by Fixpt?"; then "Yes go
ahead" to: a check of it, the sources fixed, kayak's checker aligned, and
`car` and its kin given FX-26's types (`peval`).
- **The check** (`scripts/fixpt-accepts.py`, in `check.sh` with a Fixpt
  clone): for each KFX unit (`kfx/`, the compiler; `fx26/`, the engine;
  `cells/`, tier 1), the prelude Fixpt runs KFX with and the unit's files
  as one program, given to `fixpt check` (`--syscalls` for a runtime's
  own; `fixpt run` checks and runs file by file, so a syscall Fixpt has
  not stops it before the later files are checked); a refusal said at its
  file and line. Under 4 s for the three.
- **What Fixpt refused, now written as it wants** (35 lines, 8 files): a
  `cons` or `list` whose type nothing around it says (a `let`'s binding, a
  `map`'s lambda's value, an `if`'s arms), which FX-26 gives a region of its
  own (`@r.N`), where an `acyclic` list or one at `@c` is wanted: its type
  written (`(the scope …)`, `(the (listof i64 acyclic) …)`); and tier 2's
  two kinds of machine code kept in `@c`, which they read (`hand-code`,
  `address-code`), said to `spin`, as FX-26 has a procedure that could
  reach itself say. The compiler, `kfx/`, Fixpt accepted already (the
  fixpoint check runs it on Fixpt); `fx26/` and `cells/` it had never
  checked.
- **Not yet** (next): kayak's checker accepts what Fixpt refuses, in
  FX-26 too (a `let`-bound `cons` given where an `acyclic` list is wanted:
  FX-26's `cons` makes a `pairof` at a region of its own unless what is
  expected says which; KFX's makes a list at its tail's); and its `car`,
  `cdr`, `set-car!`, `set-cdr!` take only lists, where FX-26's take any
  pair that may be `nil` (`scheme-bench/peval.fx`, refused).

## C255. The knot rule sees through a convention (2026-10-27)

Of the cases C254 found, the one unsound one (the user asked which were
inexpressive, weak or unsound): tier 2's machine code, `(subr (conv
native) (maxeff (read @c) (write @c)) …)`, kept in `@c`, which it reads,
said no `spin`; FX-26's rule (Fixpt's `k-knot-in`) has a procedure that
could reach itself so say. Kayak's checker has the rule (`knot-in`), but
did not look inside a procedure of a convention written (`%native`,
`%fx`, `%cellular`, `%convof`): now it does, so it refuses what Fixpt
refuses (a `(subr (conv native) (read @c) () int)` kept in a list at `@c`).
The region cases are a typing rule, not unsoundness: kayak solves an
allocation's region from later uses where FX-26 makes one its arguments
and what is expected leave unsaid fresh at the call (`docs/fx26.md`,
"implicit projection"); and `peval`, kayak's `car` and kin of lists only.
Both next.

## C256. `car` and `cdr` of a union's pair; `scheme-bench/` compared (2026-10-27)

The user, on kayak refusing `scheme-bench/peval.fx`: "Why is Kayak
rejecting peval?"; then "Yes go ahead". FX-26's `car` and `cdr` take apart
a union with a pair member, checked at run time (`docs/fx26.md`); kayak's
did so of a named path (C77) and of a pair expression (C208), not of a
union expression, `(car (the (union nil sxpair) no-pair))`. Now they do,
FX-26's only for KFX (`kfx-restricted`). `peval` is accepted, and its
answer, at 2 iterations, is Fixpt's. The checkers' comparison now takes
Fixpt's ports of the R7RS benchmarks (`scheme-bench/`, from the clone):
of its 53, kayak refused 13 (12 after C257); its tests had had no `car` of
a union expression.

## C257. `cons` as FX-26 types it: a pair at a region of its own (2026-10-27)

The user, on the region cases of C254: kayak should be at FX-26's point
("I'm pretty sure I want to keep on knowing the region at allocation
time"). FX-26's `cons` is `(t1 t2) -> (pairof t1 t2 r)`, `r` from what is
expected, else fresh at the call (implicit projection, `docs/fx26.md`);
kayak's had been KFX's, a list at its tail's region, which a later use
could not then choose apart: so kayak accepted programs FX-26 refuses
(`kayak/checker/let-cons-region.fx`). A `cons` where nothing is expected
is now a pair at a region of its own (`%fresh`), its tail a list or not
(C184's case, which had been a tail that is no list's alone); and `car` and
`cdr` take apart any pair an expression gives, its tail a list or not
(C208's, likewise), so `(car (cons x tail))` keeps checking
(`native/wide-frame.fx`). A tail that is a list is still of the head's
type (KFX's lists are typed: the corpus's `(cons nil nil)` made Rust that
did not build otherwise), only the pair's region of its own. Kayak's own KFX, which Fixpt accepts (C254),
checks under it unchanged; the compiler makes itself again. In theory
(the user asked whether kayak's typing had been sound but not principal):
kayak's was the more general on regions and the less general on pairs,
neither point containing the other; FX-26's is not principal by design
(an unsaid region fresh at the call), and a reference is at FX-26's point
on both.
- **The comparison** takes kayak's own programs too
  (`tests/fx26-kayak/`, new `checker/`: the region, `car` of the pair, the
  knot of C255, `car` of a union's pair). Of 840 programs, Fixpt refuses
  344, kayak each; of the 496 Fixpt accepts, kayak refuses 12 (all
  `scheme-bench/`, for other gaps: a `typecase` of a union's local, `#\nul`,
  `f64`'s arithmetic, recursive bloblet types and others).

## C258. Tier 1's `string->list` and `symbol->string` (2026-10-27)

The user: "File all of these as todo's. Then do tier 1's missing
primitives." (`TODO.md`, "Next, in order".) Of the primitives the KFX
compiler compiled to cells and `scheme-bench/peval.fx` call, tier 1 lacked
two: the 64-bit integers' operations it has already, made by family
(`fixed-families`), which a search for their names had missed. Each a
`prim` of tier 1's (`o-string-list`, `o-symbol-string`), the tiers after it
calling out to it. A test, `kayak/datum/string-list.fx`, on every tier.
- **`peval` on kayak's tiers**: its answer at 2 iterations the engine's
  (Fixpt's) on tiers 1, 4 and 6; its 2000, 281 s on tier 6: a benchmark of
  size for the native work, to measure against Fixpt's.
- **The KFX compiler on kayak's tiers** (with a `(main)` after it, as
  `fixpt run` calls `main`): compiled to cells by kayak's FX-26 compiler
  (45 s, 55 GB at peak), then run on tier 6 to compile itself, it was
  killed at 556 s and 248 GB, where Fixpt runs it in 7.7 s and 0.6 GB:
  tier 1's heap grows without bound on it (a leak, or what is kept
  reachable). Filed (`TODO.md`).

## C259. Register code stage 3: regions, bloblets, flat sums, modules (2026-10-27)

The user's order (`TODO.md`, "Next, in order"), its second: register code
stage 3. The 94 register words kayak declined, where its stack words are
Fixpt's, listed by a new `cells-programs.py --ours --regs-declined`: modules
the most, then bloblets and blobrefs, regions, flat sums. Each as Fixpt's
register code makes it (`regcode-core.fx`, `regcode-helpers.fx`,
`regcode-modules.fx`), in `fx26/regs-gen.kfx`:
- **Regions**: `letregion` (`%region-enter` or `%reap-enter`, the region in
  a slot, the body not in tail position, `%region-exit` of both); a closure
  made in a region, an `rlambda`'s and a `letrec`'s, by `%region-closure`
  of the region, the free values and the word. A leaf declines both.
- **Bloblets and blobrefs**: each operation as Fixpt's `r-bloblet-op`,
  mirroring the stack code's `c-bloblet` (`%blobref-field` is Fixpt's
  `extract` through a blobref): `prim2imm` at a literal offset, the
  suffix's room checked in line, `prim3imm` for a blobref's element, write
  and variant (`%fx26-blobref-index`, as Fixpt's register code calls it).
  `%blobref-field` never calls out, now said so to the analyses.
- **A flat sum's `tagcase`**: the blobref and its tag, a label, kept
  (where they are already if a variable), each arm compared and its name a
  blobref to its variant; no arm a failure.
- **Modules, `with`, reshaping**: a module's items in slots, a typed lambda
  naming a later item capturing it once made (`r-give-waiting`); `with`
  loading each value its body names; a reshape's values into slots, then a
  product. A `with` calls only as its body does (`r-collects`).
- **Listing**: a word constant shown as its cell is, `#word "NAME"`.

Where kayak's stack words are Fixpt's, 830 register words are Fixpt's
exactly (from 736), 9 differ (specialization, and C253's), none declined.
- **One of Fixpt's choices matched, a question for its team**: a `case`'s
  comparison (`symbol=?`), where the program binds a local `symbol=?`,
  Fixpt's checker leaves as `(with #%fx symbol=?)`, which its `r-collects`
  takes for a call (`run/standard-refs.fx`): a value kept across it goes in
  the frame. Kayak's analysis now does the same (`r-inline-op?`), so the
  words agree; filed in `TODO.md` to ask.

## C260. Specialization: a procedure copied for the lambda it is given (2026-10-27)

The user's order's third: specialization (Fixpt's middle phase). Of the
programs whose stack words were not Fixpt's, 4 were so for it alone:
Fixpt's listing has a *copy* (`map1@lambda@705`), named by the register
code of a call that gives `map1` a lambda. As Fixpt's
`c-record-inline`, `c-call-only`, `c-make-copy`, `r-specialize` and
`r-spec-lambda`:
- **Noted**: a typed definition not small enough to inline (or calling
  itself) but under 60 nodes, with a parameter it only calls (every call of
  one arity) or passes on to itself as itself, nothing binding either name
  again (`r-call-only`, `fx26/regs-know.kfx`): a *special*. Only its newest
  definition's counts, as Fixpt drops one when its global is defined
  again.
- **Copies**: before a form's register code, its twins' bodies walked for
  calls of a special given a lambda small enough to inline, of the arity
  it is called with (`r-twins-copies!`, `regs-entry.kfx`); for each, once,
  the procedure's body compiled again to a word named for it and the
  lambda, and that word's register code made in the copy's context: where
  the parameter is called, the lambda's body inlined, its parameters the
  arguments and its captured values read from the parameter's closure;
  where the procedure calls itself passing the parameter on, its own entry
  (a label at the copy's start), behind a guard. The walk stands in for
  Fixpt's plan (`compile-plan.fx`), which kayak otherwise does without: it
  makes the copies the plan would in a form's own context, and register
  code finds them (`cg-copies`), declining where there is none.
- **The call**: the copy's closure made into a slot; the arguments; then
  the copy called if the global has not been written since, else the
  global (`r-specialized-call`). A leaf with one declines, and a special is
  no plain callee to the analyses.

Measured: the 4 programs' stack words Fixpt's (299 in all, from 295);
register words Fixpt's exactly 910 (from 830), 7 differ (a module's member
re-exported and inlined, 2; a global holding a constant list unrolled, 5:
C252's remainder), none declined. `bench/closures` on tier 6: 220 million
cycles, Fixpt's listing 218 (from 253); `native/captures` 312 to 311.
Found: kayak keeps a global's inlining after it is defined again, which
Fixpt drops (`c-push-global`); filed.

## C261. Compile memory: the listing joined at once (2026-10-27)

The user: "Start work on compile time and memory, starting with fresh
measurements". Compiling `native/fixed-width-ops.fx` (11 KB) to cells:
310 ms, 256 MB at peak, 64K page faults (Fixpt: 10 ms in Rust, 31 ms in
FX-26), as before. By type and by who allocated (the engine built with
`heap-stats`, a feature now of `kayak-fx26` as of `boot/`; the sampler
weighing by bytes, of any type with `KAYAK_HEAP_SAMPLE='*'`, every
`KAYAK_HEAP_EVERY`th, `KAYAK_HEAP_DEPTH` frames): of the heap's 187 MB, 86
MB strings, 83 of them the listing's, each piece appended onto the rest
(`put-sep-lines`, `spaced`, `listing-from`), quadratic; then the reader
(about 40 MB), `c-arity`'s characters of a name at each call, the
checker's mangling. Not regions kept, at this size: work done.
- **The user's way**, over a mutable string buffer, or KFX's `apply` of
  FX-26's variadic `string-append` (KFX has no `vsubr`, a Rust closure
  having its arity, C71; to be had by representing one as the procedure of
  its list, as the engine does, should KFX want them): a function written
  in KFX, `string-list-append` (`kfx/base.kfx`), the same on every engine.
  It appends adjacent strings pairwise, each round halving the list: each
  character copied some log2(n) times, where appending onto the rest
  copied it n times; no list of characters (16 bytes each, on a heap never
  freed).
- **Used** by the listing: `put-sep-lines`, `put-sep-words` (and `spaced`)
  join their pieces with their separators at once (`strings-after`,
  `strings-between`), and `listing-from` keeps its words' text as pieces,
  joined at the end. The listings byte for byte as before.
- **Measured**: strings in the heap 86 MB to 5.4; `fixed-width-ops` 240 ms
  and 179 MB at peak, `case-tree` 210 ms and 178 MB to 130 ms and 96 MB,
  `captures` (a smaller listing) as before. What is left is filed
  (`TODO.md`, the reader first).

## C262. Compile memory: the primitives' table read once; syscalls of no required effect, asked of Fixpt (2026-10-27)

- **Where the reader's work went.** Of an 11 KB program's 211K
  s-expressions, nearly all were kayak-rt's primitives' table (198 KB of
  text, `kfx/check-defs.kfx`), read whole again by `c-arity` for each name
  it was asked of that is no standard operation of its short lists: the
  analyses ask of every variable (`r-standard-value-name?`, which keeps its
  answers, but per name), so a program read the table once a distinct
  name. Found with the sampler's new `KAYAK_HEAP_SKIP` (frames left out,
  a recursion's, to see its caller). Now kept once read
  (`cg-primitive-forms`, `c-arity` of the effect `gen`, as its callers
  are). A top-level value would not have done: KFX computes one once only
  where the checker found it so (`once`), else at each use. Listings byte
  for byte as before; `fixed-width-ops` 240 ms and 179 MB at peak to 80 ms
  and 55 MB (from 310 ms and 256 MB before C261), `case-tree` to 70 ms and
  45 MB, `captures` 150 ms and 111 MB to 50 ms and 36 MB; the heap 108 MB to
  32.
- **A presized join, and what it wants.** The user asked whether KFX
  wants a string buffer: it has one, FX-26's bytevectors (`make-bytevector`,
  `bytevector-copy!`, `utf8->string`), but sizing it wants a string's
  length in bytes, which FX-26 does not give. A syscall could
  (`kayak:string-utf8-length/1`), but FX-26's syscalls must hold `(io sys)`
  in their effect, a rule of kayak's own design, which would make the join
  and every procedure joining strings say it. The user: the trust any
  syscall asks for is unavoidable, the runtime and the program coordinating
  on its type; so the rule buys nothing. Proposed to Fixpt (the amendment in
  `docs/research/syscalls.md`): a syscall's effect what it declares, `pure`
  among them. The presized join waits on it (the user: "We won't bother
  with 2 until fixpt makes the requested change"; `TODO.md`).

## C263. The pin at Fixpt 5731f52: named parameters `(x : T)`, syscalls of any effect (2026-10-27)

The user: "Pull the updated Fixpt. It's made a lot of changes, so while
you will get the syscall change you requested, you will also end up with
a bunch of other changes". 26 commits, a fast-forward from `8c7111d`.
- **Taken**:
  - *A procedure type's parameter named `(x : T)`* (a8e6471): `(h T)` is
    always `h` applied. Kayak's checker reads and prints them so
    (`param-name-written`, `show-named`), its restriction pass too; its
    primitives' table's `assert` and `assume` written so; the two
    `kfx-stricter` programs migrated.
  - *A syscall's effect what it says* (5731f52, kayak's request, C262):
    kayak's checker no longer requires `(io sys)` (an effect of no atoms
    printed `pure`, the message Fixpt's). `kayak:string-utf8-length/1`,
    `(subr pure (string) int)`, as Fixpt's runtime and evaluator have it:
    in kayak-rt's table, and the engine's and tier 1's, each asking
    kayak-rt by a syscall of its own. Fixpt's `syscalls/pure.fx` and
    `pure-declared-io.fx` give Fixpt's answers on the engine.
  - *`attempt` a form wherever it heads a list* (b41a7fb): kayak's checker
    refuses it, not having it yet, as Fixpt now reads
    `scheme-bench/triangl.fx`'s own `attempt` (which kayak had accepted
    and Fixpt now refuses: K1).
- **Decided against, for now** (each mark moved to the pin, `TODO.md`
  4b): the features Fixpt added for Heist (`attempt` itself, `(io
  process)` and `process-*`, `type-at`, `load-moduleof` and its kin,
  `restrict`, a top-level `include`, one abstract type for every load of a
  file), whose programs kayak's checker refuses; Fixpt's `gdb`/`lldb`
  support (`jitdebug::flush` before native code runs, `Heap::plausible`,
  `%set-register-twin`'s places, which tier 1 does not have); the
  scheduler's failed task kept quiet until its failure is taken, which
  comes with `attempt`.
- **Measured at the pin**: Fixpt accepts kayak's three units; 202 of
  Fixpt's programs kfx accepts, all agreeing, and 232 beyond on the engine
  (`crossing-overflow` as before); tier 1 425 agree (from 405), kayak's own
  words 407 (300 Fixpt's exactly), register words 911 Fixpt's exactly, 7
  differ; of 373 programs Fixpt refuses, kayak refuses all.

## C264. Compile memory: names tested against made once (2026-10-27)

The next of the heap (C261, C262): of 32 MB compiling `fixed-width-ops`,
10.8 MB of lists of strings, most of them lists written in place in the
predicates the analyses ask of every name (`r-never-collects?`,
`r-prim-name?`, `r-test-name?`, `c-arity`, …), made again at each call,
where a top-level list is made once (KFX keeps a top-level value that
allocates, `once`). Thirteen hoisted to top-level definitions in
`fx26/cg.kfx`, `regs-know.kfx` and `regs-gen.kfx`. Listings byte for
byte as before; `fixed-width-ops` 80 ms and 55 MB at peak to 70 ms and
47 MB, `case-tree` 70 ms and 45 MB to 60 ms and 41 MB. Not done in the
compiler for every list written of literals: a list made once is shared,
which a program writing it, or comparing by identity, would see.
- **A correction**: the syscall amendment (C262, `docs/research/
  syscalls.md`) had Fixpt's strings as UTF-8; they are characters (the
  user). Fixpt's `kayak:string-utf8-length/1` counts each character's
  width; kayak-rt's has the length at hand.

## C265. Compile memory: words named apart without their names' characters (2026-10-27)

Measured again at the size that had been 55 GB: kayak's FX-26 compiler
compiling the KFX compiler to cells (3,465 words, a 17 MB listing),
18.3 s and 10.9 GB at peak after C261-C264 (Fixpt: 4.0 s, 0.49 GB). Of
its heap's 6.6 GB, 4.9 GB were characters' lists, made by `c-named`,
which, for each word assembled, walked every word made before it and
made each one's name's characters (`shown-name`) to compare it: quadratic
in the words. Now compared in place (`listed-as?`: the name, or the name
then a tab); and the new word's text, its lines, made only where a word of
its name is found, not for every word. Its listing byte for byte as
before, as the corpus's: 6.6 s and 2.4 GB at peak. Next: the listing
itself, 17 MB joined pairwise (some 20 copies of each byte at this size),
better written out as made.

## C266. Compile time: globals by name, the listing word by word (2026-10-27)

The same measure (kayak's FX-26 compiler compiling the KFX compiler to
cells), at 6.6 s and 2.4 GB after C265; Fixpt 4.0 s and 0.49 GB.
- **The listing word by word**: `listing-from` gives each word's text, in
  order, and `main` writes each in turn (`say-each`); quoted nodes'
  labels (`labelled-listing`) counted across the words, then each word's
  marks replaced, so no text of the whole listing is made (it had been,
  and then its characters, and then joined again by appending each piece
  to all before it, `strings-joined`, quadratic, now gone). 2.4 to 2.2 GB.
- **Where the time was**: a profile (perf) had 16% in `fact-noted?`,
  searching all the checker's facts at each application parsed for its
  call's convention, and 12% in `r-where`, whether a name is one of the
  first `k` globals by walking the list of them. The call facts are now
  picked out once (`cg-call-facts`); the globals kept by name too, in a
  table by the name's hash (`string-hash`, an array of buckets), each with
  the count of globals at its first definition, so whether a name is a
  global, or one of the first `k`, is one look (`global-first`,
  `global?`, `globals-count`). The standard names' answers
  (`r-standard-value-name?`) by hash likewise. kayak-rt's
  `string-append` sized once (`format!` grew it as it wrote): every KFX
  program's.
- **Measured**: 3.65 s (2.1 user, 1.5 system) and 2.2 GB at peak, against
  Fixpt's 4.0 s and 0.49 GB; the listing byte for byte as before, as the
  corpus's. What is left is memory: 1.5 GB of heap kept, most of it the
  listing's pieces and the checker's lists; the system time its pages.

## C267. Bytevectors at a place's region, freed with it (2026-10-27)

The user, on compile memory: regions that end and arenas reused (1 and
2), then: "we can just do all this with bytevectors today then".
FX-26's text in a region is a `(bytevectorof r)`; kayak-rt made every
bytevector in the heap, kept for the run, even at a `letrena`'s region.
- **kayak-rt**: `BytesAt<'r>`, a bytevector's cells for the region's
  lifetime, `Bytes` the heap's (`BytesAt<'static>`); `make_bytevector_in`,
  `string_to_utf8_in` and `bytevector_copy_in` make in a place's arena
  (`Holds<BytesCells>`), freed when the place's `letrena` ends, as its
  pairs and references are (C30, C34). The rest read and write either.
- **The checker**: a call of the standard `make-bytevector`,
  `string->utf8` or `bytevector-copy` whose result is at a place's region
  (a `letrena`'s, or one a procedure is given, `(poly ((p place)) …)`,
  the place named as its region) notes `rheld-bytes@'p`, so that the
  place holds bytes (and a procedure given it says so, C34), and
  `rbytes-in p`; else nothing, the bytes the heap's, as before.
- **The emitter**: such a call is `NAME_in(p, …)`; `(bytevectorof r)` at a
  region of a lifetime `BytesAt<'r>`; a place's arena of bytes holds
  `BytesCells`.
- **Test**: `kfx/bytes-region.kfx`, a text built in a buffer that doubles
  in its place, a thousand times; Fixpt's answer. No golden file else
  changed.
- **A sum's parts the heap's**: a sum is `&'static` in Rust, so its parts
  are typed with no place's lifetime (the emitter's `in-heap`, as a
  message's are): the REPL's `read-file` at a place's region, `(sumof
  (data (bytevectorof r)) …)`, had become `BytesAt<'r>` inside one, which
  rustc refuses. No program could have had a place's list there.

## C268. The listing written in places that end (2026-10-27)

The user's 1 and 2 for compile memory, by bytevectors (C267): the
listing, 622 MB of the 1.49 GB heap compiling the KFX compiler to cells
(its strings kept for the run, some 37 times the 17 MB it writes), is now
written word by word into a buffer of bytes in a `letrena`'s place, which
doubles in the place as it fills, made one string per word to be written
out, and freed with the place. `fx26/listing.kfx`, after `cg-top.kfx`:
- **Writers** (`w-str!`, `w-byte!`, `w-i64!`, `w-digits!`, `w-padded!`,
  `w-name!`, `w-escaped!`, `w-lit!`, `w-opnd!`, `w-ins!`, `w-lines!`,
  `w-rlines!`, `w-table!`, `w-word!`), each given the place, the buffer
  and its length, as `lit-text`, `lines-of-items`, `rcode-said` and the
  line tables write their text, byte for byte; literals and numbers
  written as bytes, no string made of them (the string versions stay, for
  the compiler's other uses: `word-text`, a word's name told apart).
- **The walk** in two steps: the words listed, in order, from the names
  their instructions give (`words-listed`, no text made); then, where
  there are quoted nodes, their reaches counted across all of them in the
  order the text writes them (`words-reached!`, a word's cells' literals,
  then its register code's, each literal's parts), and each word written,
  its quoted nodes' labels written as they come (`q-written!`), where
  marks had been written and replaced in the text after.
- **Measured**: the listing byte for byte as before (the corpus's, and the
  KFX compiler's against the engine before, on the same sources); 3.65 s
  and 2.2 GB at peak to 2.72 s and 1.31 GB (Fixpt: 4.0 s, 0.49 GB); the
  heap 1.49 GB to 914 MB, its strings 335 MB to 29. What is left is the
  checker's (lists of strings, 254 MB; characters' lists, 134 MB).

## C269. Compile memory: the checker's cheap fixes (2026-10-28)

The user: go for the larger refactor (the checker's data in regions that
end), "But you can do cheap fixes first if you like". First, then, what
the samples showed of the checker (kayak's FX-26 compiler compiling the
KFX compiler to cells, 2.72 s and 1.31 GB after C268):
- **Lists of names written in place made once**, as C264's: 31 in
  `kfx/check.kfx`, `check-exprs.kfx` and `check-defs.kfx` (the shapes'
  names, a fact's kinds, …), each a top-level definition.
- **`after-prefix` a substring**, where it made the string's characters,
  then dropped some (none if the string is no longer than the prefix,
  as it was).
- **`letrec-items`**: the set of a group's names made once, for every item
  but a module (which leaves out its own); made for each item, it was
  quadratic in the items, some 3,000 in the KFX compiler.
- **`names-in` onto an accumulator**, where each list's names were copied
  again at each list around it; **`distinct` by a set of those seen**,
  from the last, where each name was looked for in all after it
  (`nset-added` now in `base.kfx`, beside the set).
- **Measured**: the listing byte for byte as before; 2.72 s and 1.31 GB to
  2.57 s and 1.14 GB (Fixpt: 4.0 s, 0.49 GB).

## C270. Datatypes in places (2026-10-28)

The user: "go for the larger refactor. The reason to *have* arenas is to
enable architectural choices like this" — the checker's work kept in
regions that end. Its plan, `docs/research/checker-regions.md`: A,
datatypes in places, in KFX; B, the checker's structures over a region,
each form's work in a place of its own. This is A. FX-26 has datatypes
with region parameters, and a value whose type mentions a region cannot
leave it; its sums and products are in no region, so where kayak makes
one is kayak's to choose. KFX erased a datatype's regions (C17).
- **The emitter**: a top-level datatype's region parameters its variants
  name (`lived-regions`; Rust has no unused lifetime, so the others are
  erased as before, as are a module's datatypes') are its enum's
  lifetimes, `Rose<'r, T>`, its fields at them; an application is `&'r
  Rose<'r, T>` at a region of a lifetime, else `'static`
  (`lived-arguments`; kept as `%lived NAME`). Its constructors are at the
  lifetimes, made in the heap by kayak-rt's `alloc` (the heap's arenas hold
  only `'static` types), and, with one lived region, have `_in` twins made
  in a place's arena. Its `Show` likewise.
- **The checker**: a constructor's call whose datatype is at a place's
  region in scope (`place-named`, as C267's bytevectors) notes
  `rheld-data@'p T` (the place, and a procedure given it, holds the enum,
  C34) and `rdata-in p D`; the emitter calls the `_in` twin where `D` has
  one lived region, and declares an arena only then.
- **Test**: `kfx/data-region.kfx`, a rose tree of a hundred leaves built in
  a place and summed, a thousand times, and one in the heap; Fixpt's
  answer. `families.rs` regenerated (its `rose` now at a lifetime).

## C271. Datatypes' heap twins; `cons` and `list` in places (2026-10-28)

The user, of the checker's work in places (`docs/research/checker-regions.md`,
B): "Go with 1. I want to see the result." Two things B wants first:
- **Heap twins**: a datatype with lived regions (C270) is made in the heap
  by kayak-rt's `alloc`, a `Box` each, the heap's arenas holding only
  `'static` types; the checker, whose types will be such datatypes, makes
  millions. Each constructor has a twin, `TAG_heap`, at `'static`, in the
  heap's arena; the checker notes a constructor's call's type
  (`rdata-type`) where it is at no place in scope, and the emitter calls
  the twin where that type's lived regions are all at `'static`.
- **`cons` and `list` in places**: as C267's bytevectors and C270's
  datatypes, the standard `cons` and `list` whose list is at a place's
  region in scope, the place named as its region, note `rheld-cons` and
  `rlist-in`, and are made in the place (`rcons`), so that code over a
  place need not write `rcons` for each pair.
- **Test**: `kfx/data-region.kfx`'s `build` now writes `cons` and `list`;
  its tree, and the heap's (`kept`, by the twins), as before. No golden
  file else changed but `families.rs`, its twins.
- **Next, the user's question**: module parameters over the place, so that
  a group of items shares it without each taking it: a module made in a
  procedure, closing over its parameters, FX-26's and not yet KFX's (its
  modules are Rust's `mod`s, C19, C24). Then the checker a module made per
  form, of the form's place.

## C272. A module value's procedures' effects at the place given (2026-10-28)

The user: build modules made in procedures in KFX first (the checker
then one, made per form of its place; `docs/research/checker-regions.md`).
Its first test, a procedure over a place whose body is a module, opened by
a `with` in a `letrena`, Fixpt accepts; kayak's checker refused it, the
module's procedures' effects at an inference variable, `alloc:?0`, where
their types said the place given. Two causes, both fixed: a `with` of a
module value opened its type walked only at its top (`mtype-opened` of
`walk`), its members' types not; and `zonk` resolved a procedure type's
parameters and result but not its effect's region atoms (now each by
`resolve-atom`). Test: `kayak/checker/module-place-effects.fx`, Fixpt's
answer on the engine. The modules themselves, KFX's, next.

## C273. A `plambda` where nothing says its type; the engine's failures its own (2026-10-28)

The user, at the engine's REPL, `(plambda ((t type)) (lambda ((x t)) x))`
refused "(plambda ...) is FX-26's form, which KFX does not have": "so
explain this". Three things, all fixed ("yeah fix it"):
- **A gap**: kayak's checker took a `plambda` only where a poly type is
  expected (`plambda-form`, C107); elsewhere it fell to the call of a
  name, which refuses a keyword. Now its body is found with its binders,
  rigid, in scope, and its type is the poly of them (`plambda-synth`; its
  binders renamed as a written poly type's are, `fold-renames`; none may
  escape into a type found outside). Fixpt gives it `(poly ((t type))
  (subr pure (t) t))`. Test: `kayak/checker/plambda-synth.fx`.
- **The message** was KFX's, from when the checker was KFX's alone: a
  refusal, not a restriction noted for kfx's pass, so the engine said it
  too. Now "FX-26's form, which kayak's checker does not take here".
- **The prefix**: the checker and the reader, shared with kfx, say `kfx: `
  first. The engine's REPL now says a refusal without it
  (`engine-said`), and its batch runs say a failure `kayak-fx26: …`
  alone, where Rust's own hook had said the thread and where it panicked
  first (a hook of `main.rs`, for batch runs only: the REPL catches and
  says its own).

## C274. Modules made in procedures of places (2026-10-28)

The user: "Can we use module parameters to abstract over the place for a
group of items?", then "lets try building the module feature first". KFX
now has FX-26's module made in a procedure, where the procedure's
parameters are places alone: a top-level `(define m (poly ((p place))
(subr E ((place p)) (i p))) (lambda (p) (module item ...)))`. Its
interface is `(define-type (i (p place)) (moduleof (val …) …))`. Its items
are procedures, typed, and close over the place: a `cons`, `list` or
constructor in one finds the place by name (C270, C271). Nothing else
changes; module values otherwise stay FX-26's alone (C170).
- **The checker** notes `module-maker` at such a definition, and lifts
  C24's "no procedure gives a module back" for it. It keeps `%maker m` in
  scope after it. A `let` binding of a maker's call keeps `%maker-value x
  m`, and a `with` of that variable notes `with-maker x m`. A `with` of
  any other module value is refused as before (C170). `restrict.kfx` lets
  through a `moduleof` define-type whose head's parameters are all places
  (`place-family?`).
- **The Rust**: the interface is no trait but the struct `I<'p,
  Place_P>(&'p Place_P)`, `Copy`. Its methods are the interface's values,
  each delegating to `mod m`, where every item is a `fn` given `p: &'p
  Place_P` first. The items' calls of each other are given `p` too, at
  the source's positions, so the checker's facts still apply. The maker
  `fn m(p) -> I<'p, Place_P>` makes one. Every one of these is bound to
  hold what any item makes in the place: the maker's own `place-needs`,
  summed over its whole body (C34). `(i r)` as a type is `I<'r, …>`, and
  `(with x (f a))` is `x.f(a)`.
- **Not yet**: a maker of more than one place, or of parameters other
  than places; an item that is poly, fails or suspends; an item named
  other than called, inside the module. Each gives kfx's error.

Test: `tests/kfx/module-place.kfx`, with Fixpt's answer, 5050000. Next,
B by this (`docs/research/checker-regions.md`): the checker one module
made per form.

## C275. The pin moved to 6447f9b: strings at a region (2026-10-28)

The user: "FX26 can place strings now". I had said a place could hold
no string. That was the old pin's FX-26, 5731f52; Fixpt's 41c0823f has
`(stringof r)`, and `string` is `(stringof const)`. The pin moves to
6447f9b, which also has a module's expressions as its items, and the
`module-parameters` of modules. `built-in.py` takes the new primitives'
names, and `fx26-tests.py` takes nine new programs. The drift: the
runtime's new string primitives (`cells/prims.kfx`) and the parser's
module items (`fx26/cg.kfx`). Neither is carried over yet, and their
marks move. KFX's `stringof` comes next, for plan B.

One thing I tested on both pins stands: a place's region takes no heap
data by subsumption. `(rcons r 5 heap-list)` is refused where a
`(listof int r)` is wanted, and so is a heap tree in a node at `r`. So
plan B's per-form data meets the heap's scope only through copies.

## C276. The checks under cargo nextest; the incremental compiles first (2026-10-28)

The user, after the check's steps were timed: "I want us to go straight to
nextest." The Python scripts each ran their jobs as wide as the machine.
But each step waited for its own slowest job, and the steps ran one after
another, so there was no queue for the whole check. Now the check's tests
are `cargo nextest` tests, one queue. They start with the incremental
compiles, 427 s of the check's 460 s on its own track.
- **The harness**: `crates/kayak-checks`, a test crate whose binaries are
  `harness = false`, each `kayak_checks::run(tests)`. It speaks the part
  of libtest's command line that nextest uses: `--list --format terse`,
  and `NAME --exact`. Plain `cargo test` runs every test at once. It has
  no dependency, since the workspace has none (no `libtest-mimic`).
- **`tests/incremental.rs`** (was `scripts/incremental.py`): the same 31
  edits, now as six chains, each its own test from its own copy and its
  own memos. Those are each chosen file's edits, the module's items, a
  `define-rec` item, and the edits across files. A chain's compiles stay
  in order; the chains run at once. `KAYAK_CHECK_QUICK` gives
  `--quick`'s fewer edits, and `KAYAK_CHECK_ALL` every file.
- **`check.sh`**: `cargo nextest run --release --workspace` takes the
  place of `cargo test` and of `incremental.py`, beside the fixed point.
  The `unsafe-par` corpus run is nextest's too. `.config/nextest.toml`
  calls a test slow at 2 minutes and stops it at 10.

Measured: that track went from 427 s to 99 s, every test kept. The whole
check stays at 460 s for now: the Python steps ahead of it run in turn,
the two `cells-programs` runs 317 s of that. They come next, a test for
each program, with the Fixpt answer cache (`fixptcache.py`) ported to
Rust. Its JSON is to be written byte for byte as Python writes it, so that
`tests/fixpt-answers/` stays the same while some checks are still Python.

## C277. The check on a snapshot of the working tree (2026-10-28)

The user, of my threads waiting for each check to finish: "i just don't
know about these". `check.sh` reads the sources as it goes, so an edit
while it ran could pass a tree that was never whole; I had stopped
editing while it ran, idle. `scripts/check-snapshot.sh` takes a snapshot
of the working tree instead: every file `git add -A` would add, made a
commit through an index of its own, the tree and its index untouched. It
checks the snapshot out in ../kayak-snapshot, a worktree kept from run to
run so that its `target/` builds incrementally, its Fixpt cache this
tree's. It prints the directory (the user's: "make sure your script
prints out the directory"), the snapshot and the log. When the check
passes it brings back the Fixpt answers the check kept, and records the
tree. `--same` says whether the working tree is that tree.
`--commit < MSG` commits that tree on HEAD, the working tree left as it
is. The script runs from a copy of itself, since `sh` reads a script as it
goes; I learned that by editing it while its first run went on. That
first run took 492 s, building its `target/` from nothing.

## C278. Tier 1's programs under nextest (2026-10-28)

C276's next step. `scripts/cells-programs.py`, both of its runs, is now
`crates/kayak-checks/tests/cells.rs`: a test for each program and each
words' source, `fixpt/NAME` and `ours/NAME`, its ways run at once within
it, and `generated`, that kfx's Rust of cells/ is the Rust kept.
`fixptcache.py` is ported to Rust beside it (`src/fixpt.rs`): the same
keys, by a SHA-256 of its own (`src/sha256.rs`), and the same files, by a
JSON of its own (`src/json.rs`) that writes as Python's `json.dumps(…,
indent=0, ensure_ascii=False)` writes. A test requires every answer kept
to read and write back byte for byte. Each test keeps what it found in
target/kayak-checks/cells/. `checks-summary cells` counts that, says it
as the script did, and fails on a program not run, one that differs, or
one that no longer agrees. With `--bless`/`--bless-drop` it rewrites the
passing lists (C235). The generated Rust is blessed by `regen.sh --all`.

Measured against the Python, on the same tree: the same counts, Fixpt's
words (432 agree; 5, 40, 391 not run) and kayak's own (410 agree, 303 of
them Fixpt's exactly, 27 not generated yet; register words 918 and 8).
Both runs together took 174 s, on a machine busy with a snapshot check;
the Python's took 139 s and 178 s, one after the other. The longest test
is `preempt/allocating.fx`, its slice ways.

## C279. The checkers compared under nextest (2026-10-28)

`scripts/compare-checkers.py` is now `crates/kayak-checks/tests/checkers.rs`:
a test for each program, `checkers/NAME`, Fixpt's, its `scheme-bench/`
and kayak's own. Each fails only on a K1 breach, kayak accepting what Fixpt
refuses. `checks-summary checkers` takes the script's flags (`--list
KIND`, `--why`, `--dir`, `--summary`). On the same tree, the same counts
as the Python: of 535 programs Fixpt accepts, 405 the same, 90 differ, 40
refused; of 386 it refuses, kayak accepts none. `--why`'s messages and
counts are the same, but ties are in a different order. The first run's
counts were two off: `root()` was `crates/kayak-checks/../..`, so the
engine printed paths unlike those in Fixpt's answers kept. It is
canonical now, as the Python's `abspath` was.

## C280. A slice of four or five, jittered, for the preempted ways (2026-10-28)

The check's longest tests were `preempt/allocating.fx`'s ways at a slice of
two polls (three for register code): `--compiled --slice 2` took 116 s
alone. Measured (TODO 6b), it was almost all the scheduler's own work, the
same in Fixpt's interpreter, and the native tiers entered at every cell
or two. The user: "what does 2 buy us, just getting extreme/pathological
cases to arise". A tiny slice exists to preempt at every poll where a
value could be lost across a preemption. Catching such a loss needs a
preemption at that poll at least once, not at every poll, and a fixed
slice may fall into step with a loop. So, the user's: "jitter between 4
and 5". kayak-cells takes `--slice-jitter J`: each slice drawn from N to
N+J polls by a fixed-seed linear congruential generator, as
`--gc-stress`'s waits are (C142), each run the same. The ways that run
tasks take `--slice 4 --slice-jitter 1`, stack code and register code
alike: three for register code (C129) was only to escape the
trampoline's way back, which four does too. At the full program,
every such way now takes 16-24 s, its answer the same; a nextest JUnit
file (target/nextest/default/junit.xml) gives each test's time.

## C281. Fixpt run from a copy, built under a lock (2026-10-28)

The first check on the Mac since 4413bbe6 failed in `corpus.py`: Fixpt's
binary was not there, and was a moment later. Cargo's build of nothing
still removes `target/release/fixpt` and links it again (its inode
changes), and since 4413bbe6 the fixed point and the tests run beside the
corpus, each calling `scripts/fixpt.sh`, each building: one's build could
remove the binary as another ran it. `fixpt.sh` now builds under a lock
(Perl's `flock`, which macOS and Linux both have, released if the holder
dies) and prints a copy of the binary in kayak's own
`target/fixpt/fixpt-PIN`, made with `cp -p` and put in place by a rename,
and only when the build made other bytes: a run never finds it missing,
and Fixpt's front-end cache, keyed by its executable's path and time,
stays warm. Forty `fixpt.sh` at once while the binary ran 200 times: no
failure. `fx26-tests.py` and `bench.py` found the clone three directories
above the binary; they find it as `built-in.py` and `fixpt-drift.py` do,
`KAYAK_FIXPT_CLONE` or `../kayak-refs/fixpt`.

## C282. The engine's corpus programs that ask `native-code?`, against Fixpt's evaluator (2026-10-28)

On the Mac, `engine-programs.py` found `made-code` and `made-flat`
differing: the engine said `interpreted 42`, NAME.out `native 42`. C222
made the engine as Fixpt's evaluator, running no code a program makes
(`native-code?` `#f`), and named `seal-code` and `native-procedure`, so
these programs, refused before (not run yet), now run. NAME.out is
Fixpt's default machine's answer, and on arm64 that machine runs made
code; on x86_64 Linux, these arm64 programs say `interpreted` either
way, so the two agreed there. As C222 did for `run-fixpt-programs.py`, a
corpus program that asks `(native-code?)` is compared with Fixpt's
evaluator (`--fx26-machine fx-ast-eval`): the prelude, its files, then
`(main)`, as `corpus.py` asks, kept in tests/fixpt-answers
(`tests--kfx--NAME--evaluator`). On the Mac: 87 of 93 agree, 0 differ.

## C283. Another machine's programs in tier 1's tests, kept as they were (2026-10-28)

On the Mac, `kayak-checks::cells` failed on the nine programs that
declare `define-convention x86_64-native` (`made-code-x86_64/`, Fixpt's
and kayak's): Fixpt refuses another machine's convention (C135), so
"fixpt-refuses", where tests/cells-passing, written on x86_64 Linux, has
them agreeing. The Python checks skip such a program off its machine
(`fixptcache.machine_of`); the Rust did not. `kayak_checks::fixpt` has
`machine_of` and `elsewhere`, the same rule; such a test passes,
writing the kind `elsewhere`, and `checks-summary` keeps a passing-list
name of that kind as agreeing for the list (not dropped by `--bless`, not
"no longer agrees"), counted apart ("N another machine's"), as
`engine-programs.py` keeps `was & elsewhere`. On the Mac: 2719 tests
passed; tier 1, 430 agree, 0 differ, 9 another machine's. Whether this
is the right rule is TODO §17's, to look at with the user.

## C284. FX-26's three new typing rules, in kayak's checker (2026-10-30)

The user: kayak should follow FX-26's changes in its own FX-26 compiler,
not only keep its ports from drifting; "Go for it", the typing rules
first. All three are in the checker that kfx and the engine share.
- **Masking keeps reads and writes on the result's regions** (Fixpt's
  61a2e792, Heist's report). `masked-res` kept only `alloc`, `goto` and
  `comefrom` on a region the value's type mentions. It now keeps `read`
  and `write` there too (Lucassen and Gifford's rule); only `alloc` on a
  place's frozen data. A procedure that ties what it makes into a cycle
  must say `(write r)`, and is refused where it is used at `acyclic`
  (`regions/result-write-is-kept.fx`, `result-write-at-acyclic.fx`,
  Fixpt's verdicts).
- **Frozen data covariant in the order of lifetimes** (55d130bd, kayak's
  request). `fits` takes a frozen region where one is expected whose kind
  is no stricter (`acyclic` fits `const`) and whose place it outlives. The
  heap outlives every place, and a place outlives those bound inside it
  (`frozen-fits?`, by the order of `%place` marks in scope). Writable
  regions stay as they were. `rcons` makes its pair at a frozen region of
  its own place when one is expected, the allocation the place's
  (`frozen-in-place`). Fixpt's seven programs (`regions/frozen-covariant*`,
  `terminate/frozen-place-walk`) get its verdicts; the two accepted give
  its types, but for how a datatype is printed.
- **A list nothing places is `acyclic`** (e4af8997, Heist's request). A
  `poly`'s region binder a call leaves unsolved is `acyclic` where the
  `poly` uses it only as a pair's region, allocated and read
  (`not-only-data`, as Fixpt's `not_only_data_regions`); else, as before,
  a region of its own (`%fresh`). A place binder nothing settles is the
  heap, its frozen data `acyclic` (`name-regions`, `heap-frozen`). A
  bare `cons`'s and `list`'s region, and a list's found by itself
  (`freshened`), are `acyclic` too. C123's refusal of branches mixing a
  `cons` pair with a list is gone: the pair, `acyclic` too, joins the
  list (`fits`'s pair-below-list), and a first branch that is such a pair
  is taken as that list (`pair-as-list`, as Fixpt's 5b0d032a shows one).
  Still refused, where Fixpt accepts: a first branch whose tail is itself
  a pair, `(if b (cons 1 (cons 2 nil)) xs)`. `modules/unplaced-lists-are-
  acyclic.fx` and `list-joined-with-nil.fx` give Fixpt's lines, and
  `kfx-stricter/branches-cons-list` and `fresh-list-region` are no longer
  stricter.

## C285. Strings at a region, `(stringof r)`, in kayak's FX-26 (2026-10-30)

Fixpt's 41c0823f, the user's: a string made, read and written at region
`r`, as a bytevector is. `string` is `(stringof const)`. The user's "go
for it", after C284.
- **The checker**: `(stringof r)` is `%stringof` of its region, and
  `(stringof const)` (or `acyclic`) is `string` itself. A `string` fits
  where a `(stringof r)` is expected, `r` then `const`. A `(stringof r)`
  fits where a `string` is only if `r` is frozen. `%stringof`'s region is
  invariant. The readers (`string-length`, `string-ref`, `substring`,
  `string=?`, `string->list`, `string->utf8`) take a string at any region,
  pure at `const`, as Fixpt types them. `make-string`, `string-set!`,
  `string-fill!`, `string-copy` and `string-copy!` are new primitives.
  `rmake-string` is made as `rcons` is (`region-alloc-call`), at the place's
  region if nothing says which.
- **C284 mended**: a call's unsolved region binder was settled to
  `acyclic` by `name-regions` before C284's rule saw it, so a storage
  binder (a fresh string's, a bytevector's) could become frozen.
  `name-regions` now keeps a region of its own for one the instance's type
  uses as storage or writes (`not-only-data`, `%stringof` and `%bytes`
  among its storage kinds). A cell nothing places is so kept fresh, where
  it was `acyclic`; `unplaced?` refuses it at either, as before
  (`kfx-stricter/unplaced-ref`, its effect otherwise lost).
- **The engine's word VM**: six primitives at the end of `prim-list` (no
  number moved), written by hand in `vprims.kfx` (`gen-vprims.py` is not
  run again), a character's code a word, a span checked as a
  bytevector's is.
- **Tier 1**: Fixpt's runtime names (`%fx26-make-string`, …,
  `%region-make-string`), in place on its strings, two code points a word;
  the copy reads all of its span before it writes.
- **KFX**: none of it (`restrict.kfx`). kayak-rt's `Str` cannot be
  written, so KFX's `stringof` waits for plan B, which needs it.
- `run/stringof.fx` gives Fixpt's values on the engine and on tier 1, of
  Fixpt's words and of kayak's own. Its checks give Fixpt's lines, and
  `regions/stringof-escapes.fx` and `write-a-string.fx` are refused where
  Fixpt refuses them.

## C286. Module parameters of every kind; expressions as module items (2026-10-30)

FX-26's module features since 6447f9b (Heist's, 2026-10-28), in kayak's
FX-26, the user's "go for it". kayak loads a file statically: each load,
with what it is given, is one module spliced in, `made--load--K` (C116,
C171). That stays, and now takes what FX-26's files take:
- **Parameters of every kind**: `(module-parameters ((name kind) …))` and
  modules, `(m T)`. A kind's binder is replaced by the argument given,
  which may be a type written out, not only a name (`substituted`). A
  module's binder is renamed to the module given, and that module's names
  are included, hidden, unless it is `qualified`, as Fixpt's `(plambda
  (kinds) (lambda (modules) (module (hide (include m) …) item …)))`. The
  loaded file sees the modules it is given, as well as the standard
  environment. The loads are `((proj (load-module "p") k …) m …)`,
  `((load-module "p") m …)` and `((load-module "p"))`; the arguments are
  kinds' then modules', which the file's own parameters tell apart.
  `(module-parameters ())` is a maker of none, its state its own: any file
  of `module-parameters` is an instance (`%instance`), whose making may
  allocate.
- **`load-moduleof`** as a type: the load it names, made as any load is,
  and its module's type as a value (`(%module-made made--load--K)`, read
  as `%module made--load--K`, opened by `module-type-of`, now in
  check.kfx before `fits`). A procedure's parameter of that type is opened
  by `with`, as a module value is (C170).
- **An expression among a module's items** (`run-items`): run as the
  module is made, `(hide (define %run-N form))`, numbered from 0 in each
  module. One that only reads does nothing, and is refused as Fixpt
  refuses it (`modules/expression-item-pure.fx`).
- **Not the same as FX-26**: a load is made once where it is spliced, at
  the top level, where FX-26 makes a maker's module at each call, its
  state new. They differ only where one load is evaluated more than
  once. A module parameter's type, `T`, is not checked against the
  module given. kayak prints a load's module by its name, where Fixpt
  prints its `moduleof`.
- `modules/module-parameters-none.fx`, `-modules.fx`, `-qualified.fx` and
  `load-moduleof-define-type.fx` are accepted, and run to Fixpt's values,
  4, 57, 10 and 1. `restrict` and `load-input` makers (a `plambda` of an
  effect) come next.

## C287. `restrict`, and `the` sealing a module (2026-10-30)

C286's next. `(restrict T name …)` (Fixpt's, 2026-10-27) is module type
`T` with only the names given. kayak reads it where it is written
(`restricted`, in `parse-type`'s group, as `module-type-of` and
`as-mtype` now are, so that a load's module type, `made--load--K`, is a
module type wherever one is read). It keeps the values, effects and types
named. An abstract type not named stays as `%hidden:t`, so a value of it
may be passed but the type not named. A description not named goes,
since kayak's types spell theirs out. It refuses a name the module does
not give, and a `restrict` of what is no module's type. `the` now seals
a module to the type written, as FX-26's M4 has it: `(the T m)` is of
`T`, its other names gone. Before, it was of `m`'s own type, so a
restricted abstract type could still be named through it, a program
Fixpt refuses (`restrict-unnamed.fx`). A form whose only load is a
`load-moduleof` is spliced too, its file read. Fixpt's five `restrict`
programs get its verdicts, and `restrict.fx` runs to its value. And
`run-fixpt-programs.py` skips a line of Fixpt's `NAME = TYPE`, a module
type a `define-type` names, which it had taken for a value.

## C288. Modules: local modules, shared loads, includes of loads, labels (2026-10-30)

The rest of Fixpt's module programs kayak refused, one by one:
- **A local module's names** (`with-local-module*.fx`): letrec*'s "made
  before it" walk (`refs-in`) took a local module's `f`, opened by `with`,
  for the enclosing module's `f`. A `let`'s module bindings are now known
  to it as an item's module is. A `module` form's own items' names are its
  own, not references to the outer items.
- **A shared load read through `with`**: reading a shared load's module,
  `made--load--K`, is no effect, since it is made once and never written,
  as Fixpt's `%shared:` globals are. Renaming `(with c x)` to that module,
  as `(select c t)` is (C118), was tried and dropped: where `c` is a load
  given regions, its module is made per call, not shared, and the engine
  failed on `loaded-select`. So `load-shared.fx`, which reads a load
  through a module the file names, is still refused.
- **`(include LOAD)`** (`include-in-*file.fx`): the load's module is
  included by its name, which KFX's `include` takes.
- **A label spelt as a type**: a module's type naming one of its own types
  as a field's label, `(productof (cmds cmds))`, had its label qualified
  with the type, `m..cmds`, and was refused as a label written twice.
  `label-of` reads a label so qualified as its name, and the label check
  goes through it. I did not find which path qualifies it.
- **Effect binders** in a `plambda` nothing types (`plambda-synth`, C273):
  `~e`, as a poly type's are.
- **Not done**: a `load-input` maker whose loads are given its own
  binders, `(plambda ((ed region) (world effect)) (lambda ((core
  (load-moduleof "made-core.fx" ed world))) …))`. kayak's static splice
  makes a load once at the top level, so it cannot be made at a region a
  procedure is still abstract over; that needs parameterised module types,
  not a splice. Nor a family another module gives, imported into a module
  (`import-list-and-family.fx`). Both are in TODO 4b.

## C289. Types printed by the abbreviations in scope (2026-10-30)

Fixpt's d0b239fa: "a type is shown by the innermost name in scope whose
type it is, by structure, wherever it came from, if it has parts to
abbreviate"; so `(define-type pos int)` names no `int`. kayak printed a
name only where the source wrote one (C164), and named base types by their
descriptions in module types (C172). Now:
- Each parameterless `define-type` keeps the type the checker read for it,
  in its own scope, as `%shows NAME` (`m %shows t` in a module), and the
  engine's printer (`scope-abbrevs`) puts each name back for its
  expansion, newest first, as C164's own were. A module's is printed
  `(select m t)`, and `io-error` is a name in scope too. Reading each
  abbreviation's body again when printing was tried first. That refused
  12 programs, since a body read out of its scope may name an effect or
  region not there, and KFX cannot catch a refusal.
- A base type is printed as itself, never by a name: not by a
  description in a module type (`desc-named`), nor by a `%named` the
  source wrote.
- A dependent parameter's family applied is `((select $1 f) T)`.
Measured: of 541 programs Fixpt accepts, kayak's checker prints the same
types for 423, where it did for 414; 88 still differ, mostly datatypes,
which Fixpt prints as their `mu` structure (kayak by name), a module type
named as a whole, the effects of shared loads (C286's splice), and nested
`poly`s Fixpt keeps as written.

## C290. The pin moved to 63b95ef; reading `@globals` says `spin` (2026-11-01)

The user: "Update Fixpt from GitHub, then start work on adapting Kayak to
match the new functionality in its FX26 language and library." 67
commits, among them two that make Fixpt refuse what it took. So Fixpt
refused all three of kayak's KFX units, which KFX ⊂ FX-26 forbids:
- **Reading `@globals` says `spin`** (b9ed7daf). The exemption of a
  first definition is dropped: a procedure that may read any global may
  read itself. kayak's `no-reaching-itself` now requires it, and the
  units' effects are mended by Fixpt's own `scripts/add-spin.py`, 27
  files, the corpus's among them.
- **A procedure a global holds without being one** (7906bc63): a
  product's field, a list's element, a reference's contents, a module's
  value, whose latent effect may read that global or `@globals`, says
  `spin`. Else the global, defined again, ties Landin's knot. kayak
  refuses such a global's first definition, its type read through its
  module type (`held-knot?`).
- **`@globals` absorbs single globals' reads** (34f7fe7e): `define*`
  drops them where it finds `@globals`, and an effect is printed without
  them (`absorbed`).
Fixpt's seven programs on these (`redefine/knot-held*`,
`knot-effect-var*`, `globals/absorbed.fx`) get its verdicts, and Fixpt
accepts kayak's three units again. The other changes since 7038269 are
TODO 4c's, the ported sources' drift marks moved (their changes are the
features 4c lists).

Fixpt now refuses three of its own programs, unchanged since it last ran
them: `generative/nested.fx`, `recursive/parametric-rose.fx` and
`terminate/invariant-bound.fx`. Each reads `@globals` without saying
`spin`. kayak refuses them too, so they leave
`tests/engine-fixpt-passing`; this is the decision `--bless-drop` asks
for (C235), not a lost program. That list had not been blessed since
C235 and now holds the 207 it agrees on. The REPL's sessions print
effects absorbed, `(maxeff (read @globals) spin)`.

## C291. Keywords, in the checker and the engine (2026-11-01)

TODO 4c's first item, Fixpt's 42c1b40d, 7885d18c and b526aba2
(`docs/fx26.md`, "Keywords"):
- **Read as they are.** kayak's reader already reads `#:name` as a
  symbol of that name; `keyword-name?` tells one (`#:` and a name that is
  no number).
- **Labels and tags written as keywords are their names.** One pass,
  `labels-unkeyworded`, in `with-loads` and in the REPL's `program-read`,
  writes the label or tag of a product, `productof`, `extract`, sum,
  `sumof`, `tagcase` arm and datatype variant as its name, everywhere but
  in a quote. So the checker, the engine and KFX's emitter after it see
  one spelling, and only a keyword that is a value stays one. The
  checker's `label-of` and `tag-of` strip `#:` too, for what is read back
  from facts.
- **The checker**: a keyword's type is `keyword`, a `datum`'s shape of its
  own (`keyword?`, a `typecase` arm), quoted as itself; `symbol->keyword`
  and `keyword->symbol`. A bare `'()` is `nil`, as Fixpt's `quoted`
  takes it, where kayak had refused it. KFX's own `keyword?` (FX-26's
  form names) is now `form-keyword?`, as the primitive took its name.
- **Types print their labels as keywords**, `(productof (#:n int))`, a
  number bare (`show-label`, Fixpt's `show_label`), flat sums' too.
- **The engine**: a keyword is a symbol's immediate, its name's number
  past `keyword-bias` (2^40). That is the f32 trick (C220), since all
  eight immediate kinds are taken. So one word for each name, and `eq?`
  is word equality. `symbol?` excludes keywords; ops 256-258 are
  `keyword?`, `symbol->keyword` and `keyword->symbol`; a `v-symbol`
  literal named `#:n` is the keyword, which is how `#:n` is lowered; it
  prints as `#:n`.
- **KFX refuses them** (restrict.kfx): a keyword value and the three
  procedures, until kayak-rt has a keyword. Labels written as keywords
  KFX takes, since the pass leaves none for its emitter.
- **Not yet**: tier 1 (cg.kfx and the cellular machine's keyword object,
  kind 48), so the cells tests of Fixpt's four keyword programs are
  "cannot" yet; `#:|a b|`.
Fixpt's `run/keywords*.fx`, `keyword-tags.fx` and `keyword-datatype.fx`
get its types and its values on the engine.
`run/keywords-printed.fx` leaves `tests/engine-fixpt-passing`, the list
of what kfx accepts: kfx took it only because it read a quoted `#:a` as a
symbol of that name, which printed alike. It now refuses the keyword,
and the engine runs the program beyond KFX, with Fixpt's value.

## C292. Branches of different sums join (2026-11-01)

TODO 4c's second item, Fixpt's fdb622df and 39318012 (`docs/fx26.md`,
"Products and sums"). kayak's branches, with no type expected, had the
first's type, and each other branch had to fit it (`join-state`). Now,
where both are sums and the later has a tag the first lacks, or a tag
both have whose payloads so differ, they join (`sum-joins?`,
`sum-joined`, `sum-join-state`, check.kfx). The join is the later, if
the first is within it; else the first's variants in order, then the
later's new ones; a shared tag carries its payloads' join, or the
narrower payload fitted to the wider. That is Fixpt's `branch_join`
order. kayak fits structurally, by tags, where Fixpt asks `subtype`,
since kayak's `fits` refuses rather than answers. `joined` and
`join-state`, always called together, decide alike, so an `if`'s
branches and a `tagcase`'s arms join; folded from the last arm, the
variants come out in the same order.

Arguments: a `list` whose first element is a sum has each other element
synthesized and joined into its element type (`elements-joined`).
Fixpt's rule is wider: any type binder bounded only from below by a sum
(`sum_lower_only`). kayak's calls solve such a binder at the first
argument, so the rest of the rule is TODO 4c's. KFX refuses both
programs already, as a sum made with no type known (C183).
`run/sum-join.fx` and `run/sum-join-args.fx` get Fixpt's types, and
their values on the engine.

## C293. What a path is: `file-info` and `read-symlink` (2026-11-01)

Half of TODO 4c's I/O item, Fixpt's 09dbb15f (Heist's F5), in every part
of kayak:
- **Types**, the checker's primitives: Fixpt's own.
- **KFX**: kayak-rt's `file_info` (`lstat`, a link not followed: kind,
  size, modification time in seconds, the mode's permission bits) and
  `read_symlink`, after Fixpt's `world.rs`. Their four shapes are
  declared there, and are the emitter's standard shapes.
- **The engine**: ops 259 and 260, as `terminal-size`'s are: the answer
  asked for when its room is reckoned, made after (`info-room!`,
  `file-info-word!`, `link-room!`, `read-symlink-word!`); the product's
  labels made at loading.
`tests/kfx-io/file-info.kfx` asks of a file it wrote, the directory, a
path not there and a path that is no link, run as Rust and by Fixpt.

The other half, `process-exit` and `process-news` and their `pselect`
arms, waits on what kayak has none of yet: the OS-process primitives,
`process-start`, `process-output` and `process-status`.

## C294. Keywords on tier 1 (2026-11-01)

C291 left out tier 1. Now:
- **The cellular machine** (cells/, Fixpt's in KFX) has Fixpt's keyword
  object: a frozen bloblet of kind 48 (`k-keyword`), its one field, slot
  2, its name's symbol, made by `keyword!` as Fixpt's `make_keyword`
  makes one, and interned. `datum-hash` mixes 5 with the name's word, and
  `datum-same?` compares names, as Fixpt's `intern.rs` does. Its
  primitives are `%fx26-keyword?`, `%fx26-symbol->keyword` and
  `%fx26-keyword->symbol` (ops 210-212). It prints as `#:name`.
- **kayak's code generator** (cg.kfx) writes `#:n` as Fixpt's parsers
  do, `(symbol->keyword 'n)`, the standard one, in quoted data and
  quasiquotes too. A `typecase` arm `keyword` tests `keyword?`; any shape
  the predicate table lacked had fallen through to `bytevector?`.
Fixpt's four keyword programs agree on tier 1, Fixpt's words and
kayak's alike.

## C295. Programs run beside this one (2026-11-01)

Heist's F4 and F6, in Fixpt since before the pin: `process-start`,
`-output`, `-status`, `-write`, `-close-input` and `-signal`, and F6's
`process-exit` and `process-news`. They are typed as Fixpt's standard
environment types them, with `(io process)` an I/O atom.
- **kayak-rt** (`world.rs`): a table of children, each one's standard
  output and error a single pipe that a thread of its own reads, so that
  asking never waits. This is Fixpt's `World`, less what needs `unsafe`,
  which kayak-rt denies: a process group of its own where Fixpt makes a
  session (the child keeps the terminal); a signal sent by `kill`; an end
  found by polling `try_wait`, where Fixpt's thread waits with
  `WNOWAIT`. A process has ended once it has exited and its output has
  all been read; `process-exit` waits for that.
- **The engine**: ops 261-268. `process-start` and `process-output`
  answer when their room is reckoned, the rest at once; `vm` and `eng`
  have `(io process)`.
Fixpt's `process/basic.fx` gets Fixpt's types and its value on the
engine; kfx refuses it, a constant with I/O. So
`tests/kfx-io/processes.kfx` runs the same as Rust and by Fixpt, with
`process-exit` and `process-news` too.

Not yet: `pselect`'s arms `((process-output p) text …)`, `((process-exit
p) how …)` and `((process-news) p …)`, which wait in the scheduler (Fixpt's
events 46-54 and its wake pipe).

## C296. `pselect` on programs run beside this one (2026-11-01)

The rest of Heist's F6: the arms `((process-output p) text …)`, ready once
`p` has output to take; `((process-exit p) how …)`, once it has ended,
its output all read, so that an arm for its output sees all of it first;
`((process-news) p …)`, once any process has news.
- **The checker** takes the three arms.
- **The engine** lowers them to the library's `process-output-event`,
  `process-exit-event` and `process-news-event` (Fixpt's `processes.fx`
  at the pin). The scheduler answers their `%sched` ops 46-54 as Fixpt's
  `sched.rs` does, its state three slots longer (the output, end and news
  waiters). When no task can run, `timers-wait!` waits for a child as
  well as for the input and the timers. It does so through three
  syscalls of kayak-rt's: `io:process-output-ready/1`,
  `io:process-ended/1` and `io:process-wait/1`.
- **KFX**: kayak-rt's `ProcessOutput`, `ProcessExit` and `ProcessNews`
  are `Waitable`, each from outside the program, as the standard input
  is, and the arms' values are taken by functions of the same names.
- **Wakeups.** A child's news wakes `sync`'s waiters, and is counted for
  the engine's wait, in place of Fixpt's wake pipe. The reader threads
  report output and end-of-output. A thread for each child polls for its
  exit, every 2 ms, where Fixpt's waits with `WNOWAIT`; both of those need
  `unsafe`. The news is never reported with the children's table locked:
  the scheduler takes its own lock before that one, when asking whether
  a child is ready.
`tests/kfx-io/pselect-processes.kfx` runs the same as Rust and by Fixpt;
`tests/fx26-kayak/io/pselect-processes.fx` gets Fixpt's types, and on the
engine its value, by hand: no check runs kayak's own FX-26 programs on
the engine.

## C297. `attempt` (2026-11-01)

Heist's F1, in Fixpt since b41a7fb, which kayak's checker had refused
wherever it heads a list (C263). `(attempt e on-failure)` is `e`'s
value, or, should `e` fail, `(on-failure message)`.
- **The checker** (`attempt-form`, after Fixpt's `synth_attempt`): `e`
  is cancellable, as a `race` branch is, so its effect has no write,
  fill, send, I/O or control out of it; `on-failure` is a procedure of
  one string; its result fits `e`'s type; the effect is theirs and
  `nondet`. Checked by synthesis where a type is expected, as `race`.
- **The engine** lowers it as Fixpt's parser does, `((with lib attempt1)
  (lambda () e) on-failure)`. The library's `attempt1` runs `e` as a
  race branch and offers its outcome, which the engine's scheduler had
  for `race` already.
- **KFX** refuses it, as it does `race`; tier 1's code generator has it
  not yet.
Fixpt's `attempt/` programs: the eight that run give its values on the
engine, and seven of them its types too. `evaluator-after-failure.fx`
differs only by printing `nat` for Fixpt's `(nat 3)`, which kayak does
throughout. The three refusals say Fixpt's words.

Also, kayak's own FX-26 programs (`tests/fx26-kayak`), which no check
ran on the engine, run beside Fixpt's beyond KFX
(`run-fixpt-programs.py`): all 14 agree.

## C298. Programs run beside this one, on tier 1 (2026-11-01)

The cellular machine has what C293, C295 and C296 gave the engine:
- **Primitives** (ops 213-222): `file-info`, `read-symlink` and the eight
  `process-*` primitives, by kayak-rt's own (cells/ is KFX). `file-info`'s
  product is made by `frozen4!`, beside `frozen2!`.
- **The scheduler** has the same three waiter slots (its state 20 long)
  and `%sched` ops 46-54. Its `timers-wait!` waits on children through
  the same syscalls, as the engine's does.
- `(io process)` is in `cm`.
`process/basic.fx` and `kayak/io/pselect-processes.fx` agree on tier 1,
both with Fixpt's words and with kayak's.

`attempt` agrees on tier 1 since C297: kayak's code generator expands it
as the engine's lowering does (`process-expanded`). All eight of Fixpt's
running `attempt/` programs agree there, its words and kayak's.

## C299. Types as reps: the design (2026-11-01)

The user chose Fixpt's "heap types" (its `heap-types.md`) before `stage`
and sessions, so that those are built on reps from the start, not on
text and then moved, as Fixpt's were. `docs/research/type-reps.md` maps
it onto kayak:
- Fixpt's `fixpt-reps` is taken whole, being of no type language (all
  but its `heap.rs`, Fixpt's heap's), copied and checked at the pin, so
  that kayak's reps are Fixpt's byte for byte.
- `rep-of-nodes` and `rep-nodes`, standard FX-26 procedures, go into
  kayak-rt, the engine and tier 1. kayak's checker, in KFX, calls them,
  as Fixpt's FX-26 checker calls its runtime's.
- kayak's `ty` maps to Fixpt's heads (its `reps.rs` table): a list is a
  pair node whose tail is itself, `%mu` is a cycle, `%named` is seen
  through, and an unsolved inference variable is declined.
- The facts that carry a type carry its rep instead, for the emitter,
  the lowering, tier 1's generator and the memos; then `stage` on reps.

Two questions are left for the user: whether Fixpt should print reps
(`fixpt check --reps`, Fixpt's change to make), so that the checkers'
agreement can be by reps; and a copy of the crate against a path
dependency on the clone.

## C300. Types as reps, step 1: the crate and the procedures (2026-11-01)

- **`crates/kayak-reps`** is Fixpt's `fixpt-reps`: its `lib.rs` (as
  `fixpt/mod.rs`), `binary.rs`, `fill.rs` and `text.rs`, copied as they
  are by `built-in.py` and checked at the pin, loaded from a root of
  kayak's under `#[rustfmt::skip]`, since Fixpt's text is not rustfmt's.
  Its `heap.rs` is kayak's own: Fixpt's tag and its
  `FIXPT_VALIDATE_REPS` toggle, as a rep crosses kayak's runtimes as a
  bytevector. Fixpt's 16 unit tests of the crate pass on the copy.
- **`rep-of-nodes` and `rep-nodes`** are in kayak-rt (`reps.rs`), after
  Fixpt's runtime's, typed as Fixpt's `standard.rs` types them. Their
  shapes, `P 1 2`, `P 1 2 3`, `P 1 2 3 4 5` and `S node bound hole const
  sym int flag`, are kayak-rt's, and so among the emitter's standard
  shapes; `shapes.kfx`'s Rust no longer declares `P 1 2`.
- **`tests/kfx/reps.kfx`** makes the reps of `int`, of `(listof int)` (a
  pair whose tail is itself, a cycle) and of a `poly` with a bound
  variable, and reads each back. The corpus compares its answer with
  Fixpt's, which is made by Fixpt's own `fixpt-reps`: the binary forms
  agree byte for byte.
Not yet: the two procedures on the engine and on tier 1, which only an
FX-26 program calling them needs (kayak's checker runs as Rust).

## C301. Types as reps, step 2: kayak's export and import (2026-11-01)

`kfx/reps.kfx`, in the engine's unit (after the checker), after Fixpt's
`check-reps.fx` and its `reps.rs` table of heads:
- **The export** threads its state as a value (`rx`: the nodes made, the
  next slot, the holes, whether a part was declined), not in global
  references as Fixpt's does, since kayak's checker keeps no global state.
  It covers base types, `void`, `nil`, `false`, unions, variables (bound
  by their binder and place, else holes), `subr` (its convention, its
  effect as a set of atoms, its parameters, its result), `poly` and its
  binders, `ref`, lists (a pair, maybe `nil`, whose tail is itself),
  `%pairof`, products and sums, `%mu` (its node a cycle), `icell`,
  `mark-key`, `place`, arrays, `%named` (seen through), and a datatype or
  generative type as a constant by its name. Regions and effect atoms
  are as Fixpt writes them. Anything else is declined.
- **The import** (`rep-ty`) is the reverse. A node met again while it is
  being made becomes a recursive type's variable, its node then a `%mu`.
- **`kayak-fx26 --reps FILE`** writes each definition's rep in hex, and
  whether, imported and exported again, it is the same rep.
- **The test** (`crates/kayak-checks/tests/reps.rs`, one per program,
  Fixpt's and kayak's own) fails on any `differs`. Of the 1365
  definitions, 1035 come back as themselves and 330 are declined; none
  differ.
Where kayak's reps may not be Fixpt's (which cannot be compared until
Fixpt prints reps):
- a datatype's constant is keyed by its name and 0, and has no `gendef`
  node;
- a plain `subr` says `cellular`, Fixpt's checker's default;
- a region constant is its name as kayak writes it (`@a`).
Declined so far: `addr`, module types, specifications, blobrefs,
bloblets, `nlist`, prompt tags, and an effect variable applied.

## C302. Types as reps: more of kayak's types (2026-11-01)

The export takes more heads, the import each back:
- **Fixpt's generative types**: kayak's `%addr`, `%inbox`, `%blobref`,
  `%stringof`, `%bytes`, `%eqtable` and `%identity`. In Fixpt's type
  language each is a named generative type, not a head of its own, so
  each is a `named` constant applied to descriptions (`d-type`,
  `d-region`, `d-size`). A datatype's arguments are descriptions too.
- **Prompt tags**, `(prompt-tag A H E R)`.
- **Module types**: `(moduleof {t …} (binder type) … (descs d T …) (vals
  x T …))`, the abstract types bound by the node, as Fixpt binds them.
  Imported, an abstract type is a constant `rp..t` again. Effect
  descriptions and type constructors are declined for now.
- **Bloblets**, `(bloblet frozen? R T …)`.
- **Sizes**: `(finite)`, or `(lin k (&terms (term v c) …))`, the terms a
  set; with them `(nat N)`, `nlist`, `narrayof`, and flat arrays as
  Fixpt's generative `flat-arrayof`.
`--reps` says why a type is declined: the first part found. Of the 1365
definitions, 1249 come back as themselves; 116 are declined, none differ.
Most of the rest are specifications (about 40), flat sums (20), `vsubr`,
module values and effect descriptions. Two choices are kayak's until
they can be compared with Fixpt's: an `nlist`'s region (kayak's has
none, so `(heap)`), and a flat array's literal length, which comes back
as a size where the emitter takes a literal (`flat-length`).

## C303. Types as reps: Fixpt's numbering, flat sums, specifications (2026-11-01)

- **Constants keyed as Fixpt keys them.** A standard generative type's
  key is made of its name and its number in Fixpt's `check.rs`, where
  C301 and C302 used 0: `vsubr` 0, `flatlayout` 1, `flatarrayof` 2,
  `identity` 3, `eqtable` 4, `blobref` 5, `flat-arrayof` 6,
  `flat-keyed` 7, `bytevectorof` 8, `escape-tag` 9, `addr` 10, `inbox`
  11, `process` 12, `event` 13, `stringof` 14, `sessionof` 15, `pending`
  16, the evidence types 17-21. A program's own datatypes keep 0 until
  kayak numbers them as Fixpt does, after the standard ones, in
  definition order.
- **Flat types**: `flatarrayof`; a keyed flat sum as `flat-keyed` of
  its key's width (a size) and the sum, as Fixpt's `k-parse-flat-sum`
  makes one; an unkeyed one a plain sum.
- **`vsubr`**, of an effect and two types, and **`escape-tag`**.
- **Specifications**: a `subr`'s fifth operand, `(spec {p … r} requires
  ensures)`, after Fixpt's `rp-spec`, its formulas `true`, `false`,
  `rel` (by name: `lt`, `le`, `eq`, `ne`), `holds`, `not`, `and`, `or`,
  `implies`, over `lit`, `add`, `sub`, `mul` and `(atom NAME)`. A name is
  `param`, `result`, `length`, `acyclic`, `nat`, `length-is`, `shape` (by
  Fixpt's shape numbers) or `call`. kayak's own text is read
  (`spec-parts`) and written back from the nodes on import.
Of the 1365 definitions, 1334 come back as themselves; 31 are declined,
none differ. Left: a module type by its abbreviation's name (`%module
NAME`, whose expansion is the checker's environment's, which the export
has not got), module types with effect descriptions, higher kinds
(`dlambda`, `app`), composable continuations, and an effect variable
applied.

## C304. `stage`, in the checker (2026-11-01)

Part one of `stage` (Fixpt's `docs/fx26.md`, "Staging code"), now that
types have reps:
- **Fixpt's stage programs** (`fixpt-cli/tests/programs/stage`, whose
  answers its tests write inline) are kept in `tests/fx26/stage`, by
  `fx26-tests.py`, which keeps a list of trees now. The checkers'
  comparison takes them, as `cli-stage/NAME`.
- **The form**: `(stage m E R code (p body …) (why else …) [(given g
  …)])` is written by the same pass as the keyword labels as Fixpt's
  parsers write it, a `tagcase` over a call. But the call is of a form of
  kayak's own, `(%stage E R m code given)`, not of Fixpt's poly procedure
  `%stage`: kayak runs a `poly` in a `poly` together (C107), so its
  `proj` could not instantiate the outer one alone.
- **Its type** (`stage-form`): `m` a module, by its name, as Fixpt's
  rule has it; `code` a `datum`; a `(sumof (ok (subr E () R)) (failed
  stage-failure))`, of the effects of its parts and `stage` and `spin`.
  `stage` is an effect atom, and `stage-failure` a standard type,
  printed by its name, as `io-error` is.
- **Evidence** is not taken yet. A stage whose `E` or `R` names a
  variable is refused: stricter than Fixpt, which takes it given, never
  looser.
- KFX refuses `stage`.
Of the 15 stage programs Fixpt accepts, kayak's checker says the same of
4 (`stage.fx`, `kept.fx`, `sealed.fx`, `tail-across.fx`); the rest need
evidence, sessions, or a module made where it is staged. Over 561
programs Fixpt accepts, kayak's checker now says the same of 440, and of
406 Fixpt refuses, it accepts none. The engine runs no stage yet: part
two, with its check in a worker, as the REPL's (C111).

## C305. `stage` on the engine, by rep (2026-11-01)

The user chose "rep from start": the stage's type crosses to run time as
a rep, never as text. kayak's checker and lowering are `kfx`, which may
not allocate, and a rep is a bytevector `rep-of-nodes` allocates. So the
work is split as Fixpt splits it, the checker exporting and the runtime
making the rep canonical:
- **The checker exports.** `stage-form` exports what the code is checked
  as, `(subr pure (M) (subr E () R))`, as nodes (`ty-export`, which is
  pure), and notes them as a fact, `stage-rep`, in a text of the nodes
  (`rp-nodes-text`: every name a string, so that none breaks the
  reading). A type a rep does not take is refused, as Fixpt refuses one
  (`NO_REP`). `reps.kfx` now comes right after `check.kfx` in both units.
- **Loading makes the rep.** The lowering writes `stage` as a call of
  the primitive `%stage-run` (op 269) with a literal `v-rep` of that
  text. Numbering, where the engine allocates, reads the nodes back
  (`rp-nodes-of-text`) and makes the rep: a bytevector constant.
- **The run** (`fx26/vstage.kfx`):
  - It imports the rep (`rep-ty`, region-polymorphic now) and turns the
    code's datum back into what the reader would have read, each list at
    a position of its own.
  - It checks `(lambda (stage-module) (lambda () (with stage-module
    CODE)))` against the type, first in a monitored worker, as the REPL
    checks (C111). The checker refuses by failing, which fails the
    worker alone, and the refusal comes back as `(sum failed …)` of a
    `stage-failure` (`refused`, the checker's words, places 0 0).
  - If it passes, it is checked again where the program runs, for the
    facts its lowering reads (a `with`'s, C213), and the thunk is lowered
    in a frame binding `stage-module`, numbered, and made a closure over
    the module, which its frame holds: `(sum ok THUNK)`.
  The engine's `vm` effect has the worker's atoms.
Of Fixpt's stage programs, on the engine: `sealed.fx`, `state.fx` and
`tail-across.fx` print what Fixpt's run prints. `stage.fx` and `kept.fx`
do too, but for their refusals' words and places, which are kayak's
checker's (`a string is here, where a int is expected` for Fixpt's
`argument 1 is a string, where a int is expected`) and 0 0. Matching
those is a decision of its own (kayak's checker's messages throughout).
The rest need evidence, sessions, or rules kayak's checker has not got.

A slip found on the way: `%stage-run` first went into the primitives'
list before the processes' (C295), renumbering them; it is last now.

## C306. A test of `stage` on the engine (2026-11-01)

`crates/kayak-checks/tests/stage.rs`: each of Fixpt's CLI's stage
programs, run by Fixpt (`fixpt run`, which calls `main`) and by the
engine (the program, then `(main)`, whose value the engine writes last
and the test leaves off). The ones listed in `tests/stage-passing`
(`sealed.fx`, `state.fx`, `tail-across.fx`) must print what Fixpt
prints; the rest are kept as found, `same` or `differ` and both outputs,
in `target/kayak-checks/stage`. Like the other passing lists, the list
only grows.

## C307. Evidence (2026-11-01)

Fixpt's `docs/fx26.md`, "Evidence": `stage` within a `plambda`, checked
against its variables as they were instantiated.
- **The types** `(effectrep e)`, `(typerep t)` and `(regionrep r)`,
  Fixpt's generative types 17-19, are invariant. An `effectrep` holds its
  effect as a procedure's latent one, as `vsubr`'s does, and two of them
  fit only if their effects are the same (`same-effs`).
- **`(reify-effect E (given g …))`**, `reify-type` and `reify-region`
  (`reify-form`) make evidence of what they are given. The checker
  exports `(evidence X)` as nodes and notes them, with the names of the
  variables the `given` is evidence of, in order.
- **The rule, Fixpt's, in its words**: a variable free in what a `stage`
  is checked as (its `M`, `E` or `R`), or in what a `reify-…` is of, must
  be given evidence, or "`stage`'s code is checked, as the program runs,
  against `e`, a variable: give it its evidence, a `(effectrep e)`
  (`reify-effect`), in `(given …)`". The variables are the export's
  holes. A region needs none: one a variable names, given none, is a
  region of its own, `%region-N`, named afresh where the program runs.
- **Where the program runs** (`vstage.kfx`), a rep's holes are filled
  from the evidence given: each hole becomes what its evidence is of, a
  hole among an effect's atoms that effect's atoms, spliced into the set,
  as Fixpt's `fixpt_reps::fill` does. The nodes are joined by renumbering
  each piece of evidence's past the type's own. Then the rep is made
  canonical, and a set's members are kept each once (Fixpt's
  `fill::dedup`). A `reify-…` (`%reify`, op 270) gives that rep;
  `%stage-run` imports it.
`evidence.fx` gets Fixpt's lines from the checker, and on the engine
prints what Fixpt prints but for its two refusals' words (kayak's
checker's). That includes evidence made of evidence: `(maxeff e (io
stdout))` given `pure`'s. `evidence-region.fx`'s evidence checks, but
kayak refuses a module that `main` makes at `@a`, where Fixpt does not
count that allocation against `main`: a checker difference of its own.
Bounds (`effectbound`, `typebound`) are not taken yet.

## C308. Evidence bounds (2026-11-01)

Fixpt's 265a166b: evidence of at most an effect or a type.
- **The types** `(effectbound e)` and `(typebound t)`, Fixpt's
  generative 20 and 21, are covariant: evidence of less stands for more.
- **`effect-bound` and `type-bound`** make a bound of exact evidence. In
  kayak's checker, a bound of an effect not solved yet, where one of a
  known effect is wanted, is solved to that one, the most it may be, so
  that the exact evidence it is made of may be of it. Without that, the
  call's result, checked first, solved the variable to the least, `pure`.
  On the engine (ops 271, 272) a bound is its evidence's rep.
- **The rule, Fixpt's, in its words** (`stands-safe?`): a variable given
  a bound may stand only where checking against less is safe: not in `M`;
  in `R` only where `R` gives what it is, a frozen list's elements, a
  procedure's result, a product's or sum's parts, never a parameter's
  type or a reference's. "`stage` is given a bound of `t`, which may be of
  less than `t` is: …". A `reify-…` takes exact evidence only.
- **Found on the way**: kayak's checker synthesized a `with` of a module
  value's body with nothing expected, then fitted it, so a lambda there
  without its parameters' types was refused. Fixpt gives the body what is
  expected, and now kayak does too (one more of Fixpt's programs the same,
  442 of 561).
`evidence-bound.fx` runs as Fixpt's but for one refusal's words;
`evidence-bound-taken.fx`, `-reify.fx` and `-region.fx` are refused as
Fixpt refuses them.

## C309. Sessions, in the checker (2026-11-01)

Part one of Fixpt's sessions (`docs/fx26.md`, "Sessions"), the checker's,
as C304 was `stage`'s:
- **The types** `(sessionof M r)`, invariant, and `(pending E r)`,
  covariant in `E`, Fixpt's generative 15 and 16; and the standard types
  `session-decision` and `session-redefinition`, printed by their names.
- **The procedures** `make-session`, `session-redefines`,
  `%session-ready` and `%session-committed!`, typed as Fixpt's
  `standard.rs` types them.
- **The forms**, written as Fixpt's parsers write them, by the pass that
  writes `stage`: `session-check` as a `tagcase` over kayak's own form
  `%session-check`, typed as `stage` is, but with its `s` a session, and
  what it makes an expression's procedure or a definition pending,
  `(sumof (expression (subr E () R)) (definition (pending E r)))`, with
  effect `(read r)` too. `(session-commit! s d decision)` becomes
  `%session-ready`, which gives the initializer for the program to run,
  then `%session-committed!`, as Fixpt's parser writes it.
- A module type named by an abbreviation (`%module NAME`) is expanded
  before its export, so a stage's or session's type takes it.
- KFX refuses sessions.
Fixpt's session programs now check as Fixpt checks them, but for how a
module's type is printed: by its abbreviation in Fixpt (`api : api-type`),
kayak writing it out. Over 561 programs Fixpt accepts, 443 the same,
none Fixpt refuses accepted. The engine runs no session yet.

## C310. Sessions, on the engine (2026-11-01)

Part two of Fixpt's sessions, after its `sessions.rs` lowered (its
`rs-scheme-vm` path), in `fx26/vstage.kfx`:
- **A session** is `#(m index)`; its state, made at its first use, when
  the module's type is known from the step's rep: the checker's
  environment (the standard one, `stage-module`, and each of `m`'s values
  defined as `(define x (with stage-module x))`), its globals, and its
  definitions' forms. Its globals are the machine's, past the program's
  (`session-global!` grows the arrays), each lowered by its name where it
  is its name's newest, the program's unnamed, so that staged code sees
  only the session's. A pending definition is `#(session index)`, its
  name, form, type and generation kept here.
- **The checker in a session** (its `%session` mark): reading a global is
  no effect of the code's, as Fixpt's `globals_effects` off; a definition
  may be defined again at another type (`%reruns`, C202's). A definition
  written with no type is checked as the REPL checks one
  (`check-untyped`).
- **`session-check`**: an expression checked at `(subr E () R)` and made
  the procedure that evaluates it; a definition checked as it is, its
  initializer, unless a `lambda`, within `E`, then pending. Each check by
  a worker first, as `stage`'s (C305).
- **`%session-ready`**: stale (another session's, committed already, or
  the session changed since), refused (`refuse` of a name defined), or
  committed: at a type every use can take, the global assigned; else a new
  one, `shadow` leaving the users with the old, `rerun` checking each
  again (C202's `users-of`), each that checks run again at its global, the
  rest broken, a use refused as Fixpt says it ("`g` is broken, since `a`
  was redefined (WHY): define it again to use it", the checker's
  `%broken-why`). Then `(sum go p)`, `p` the procedure that runs them.
  `session-redefines` gives the name, both types and the direct users.
- C202's helpers moved from `start.kfx` to `vstage.kfx`, and `globals`,
  `global-name-table` and `label-position` from `vm.kfx`, as KFX has no
  forward references.
- `fx26:session-conditions.fx` built in (`scripts/built-in.py`).
`commit.fx` and `session-conditions.fx` run as Fixpt runs them;
`session.fx`, `session-knot.fx` and `evidence-session.fx` but for the
checker's words in refusals (kayak's, not Fixpt's). Not yet: sessions in
the REPL (its globals' list does not know a session's), tier 1, KFX.

## C311. A session's globals, named as Fixpt names them; sessions in the REPL (2026-11-03)

The user's model, checked against Fixpt's `sessions.rs` and
`docs/research/eval.md` ("Sessions"): each session a namespace of its
own, its globals named `fx26-session-K:NAME`; `m`'s values its first
definitions, bindings copied and so shadowed in it alone, never changing
`m`, the values themselves shared (a reference in `m` is the host's and
every session's); the program's globals invisible to it; the REPL a thing
apart, whose globals a session's live beside. kayak's (C310) were unnamed
slots past the program's, which the REPL's next definitions could take.
Now:
- **One list of the machine's globals** (`machine-globals`, vstage.kfx,
  was the REPL's `repl-globals`): a program's, made at loading; the
  REPL's; a session's, each named as Fixpt's lowering names it. What is
  defined after a session's globals has an index of its own.
- **The REPL lowers with the checker's facts** (`entry-facts`): the
  front's check of an entry gives the machine what it noted of the entry's
  forms, as a program's lowering has them (`with-entries`). `stage`,
  `session-check` and evidence lowered in the REPL had no type and failed
  the machine (`car: nil`) since C305. A draft's run early has them too;
  in C103's REPL, whose machine has no checker's state, a draft that
  stages is not run early.
- Test: `tests/repl/session.in`, both REPLs: a session's definitions
  among the REPL's, a module's value shadowed in the session alone, a
  reference in the module written by the session and read by the host,
  the REPL's globals not the session's.
Found, not done: `(with m x)` of a module value at the REPL fails in the
engine (`nothing is named x`), its exports not among the REPL's globals;
so before this too.

## C312. Refusals in Fixpt's words (2026-11-03)

The user's item 2: refusals said as Fixpt says them, where Fixpt's words
are the better or the same; where kayak's are better, kayak keeps them
and Fixpt is asked (the user's, the same day: "each play off each other,
with a usual assumption being that if there is ambiguity, then Fixpt is
probably right").
- **Measured**: the checkers' comparison (`tests/checkers.rs`) compares
  both refusals' words, places aside, `said` where they agree; the stage
  test compares a program both refuse as it is checked by its words.
  Over the 406 programs Fixpt refuses: 46 in Fixpt's words before, 122
  now. Of the 21 stage programs, 16 print what Fixpt prints (5 before).
- **A mismatch** (Fixpt's `check-expect.fx`): "a W is expected here, and
  this is a G", the whole types at the expression (kayak's `%fitting`
  mark, which `check-by-synth` gives `fits`), and for procedures a second
  line, "  beyond what is expected, it has E". As an argument of a call by
  the general rule (`%arguing`, `%saying`): "argument N is a G, where a W
  is expected"; a definition's initializer: "`x` is declared a D: " first;
  a literal's, its type (`literal-type`); a lambda whose effect exceeds
  the one expected, said so too.
- **Others**: "unbound variable `x`" (a keyword where no form is, too);
  "expected N argument(s), got M"; the knots' (`no-reaching-itself`,
  `no-knot`) as Fixpt's `check-program.fx` words them; `stage` and
  `session-check` taking a module or session by its name, as `infer.rs`'s
  `note_staged`.
- **A staged refusal's place** (`start`, `end`): the code's parts placed by
  their offsets in the code as written (`code-sexp!`, each code past a
  base of its own so that a session's kept facts stay apart); the checker
  says `<stage>@N`, as a REPL draft's `<draft>@N`; a refusal of what kayak
  wraps around the code is the whole code's, as Fixpt's.
- **Kept kayak's, a note for Fixpt** (TODO.md): `(+ 1 "x")` and `-`, which
  Fixpt's rule of sizes says as "a int is expected here, and this is a
  string" where its general rule says "argument 2 is a string, where a int
  is expected". kayak says the latter for every call.
- **Not yet**: Fixpt's "; it may not end: WHY" after a `spin` mismatch
  (its termination checker's reasons); the storage knot's "and it is a
  T"; types printed alike (`car`'s `(union nil (pairof ? ? r))`, kayak's
  `(listof ?1 ?0)`); the long tail, each a message of its own.

## C313. Small things, found along the way (2026-11-03)

The user's item 3:
- **A session's global defined again as a new one** is named as Fixpt's
  `Globals::define` names it: `fx26-session-K:a`, then `…:a:2`, `…:a:3`.
- **`with` of a module value at the REPL** (`(with api base)`): the
  machine lowers with the modules' `%exports` the entries defined
  (`machine-modules`, `entry-modules!`), as a program's lowering has them
  (`module-entries`); the front's trial lowering keeps them among its own
  names. Both REPLs (`tests/repl/modules.in`). It failed in the engine
  before ("nothing is named base").
- **A module's type printed by its abbreviation** (`api : api-type`): a
  `moduleof` abbreviation is kept for printing (`%shows`, C289), and a
  module's line names its type by an abbreviation of it whole, not one of
  its parts, and not one with abstract types (the module's own, so not
  the abbreviation's). `kayak-fx26 --check` reads plain abbreviations
  ahead (`shows-ahead`, each by a worker, one that cannot be read ahead
  left), as Fixpt declares types ahead, so one written after a definition
  names its type too.
- **Masking past a module type** (`evidence-region.fx`): a module's type
  holds the regions of its values and descriptions, as Fixpt's
  `regions_walk` has it, no longer `%opaque`; so a `let` of a module at
  `@a` given to a procedure over a module type masks `@a` as Fixpt does.
The checkers: the same of 450 (443), refuses 23 (24), none Fixpt refuses
accepted. Stage programs as Fixpt prints them: 17 of 21. Left:
`generative.fx`, `in-a-process.fx` (the checker's), and `kept.fx`,
`session.fx` (`+`'s wording, C312, and `car`'s type as printed).

## C314. The pin moved to Fixpt 19b31f1 (2026-11-03)

29 commits, both of `docs/requests/fixpt-parity.md`'s requests among
them (82fedd92: `+` and `-` name the argument that is wrong; 19b31f10:
`process/basic.fx` takes the output once more). Each drifted source:
- **`sched.rs`** (3e026d63): op 55, `attempt-within`'s deadline, the clock
  `a` milliseconds on, carried over to both schedulers (`cells/prims.kfx`,
  `fx26/vsched.kfx`); `fx26:processes.fx`, copied whole, defines
  `attempt-within1` with it.
- **`parser-exps.fx`** (d144eeb5): `stage`'s and `session-check`'s `E` read
  as `(maxeff E)`, so that an effect a `with` gives, or a `select`, is one;
  carried over to kayak's `stage-expanded` (`stage/named-effect.fx` as
  Fixpt's). Its `attempt-within` form (3e026d63): not yet, TODO.
- **516abc8e** (a compiled word's code in a code bloblet of its own, the
  machine keeping none: `cellular.rs`, `cellular/x64.rs`, `layout.rs`,
  `disasm.rs`, `fixpt-engine/src/cellular.rs`) and **6c46d762** (arm64's
  register code): decided against for now. They are Fixpt's machine's
  code management, made so that sessions' code no longer fills it; kayak's
  compiled tiers run no session yet. Before they do (TODO), this is to
  port.
New programs kayak's checker does not take yet (refused, Fixpt accepting):
`attempt/within-*`, `stage/session-types.fx` (a module's type describing
a type of parameters, `dlambda`, which the module defines, Heist's §211,
3), and others among the 11 new. Checkers: 454 the same of 572; none
Fixpt refuses accepted; 126 of 409 refusals in Fixpt's words. Stage
programs as Fixpt prints them: 19 of 23. The corpus's `made-encoded` and
`made-flat`, which load `fx26:arm64.fx` (grown upstream, §81), blessed:
their Rust has the encoder's new operations, their answers the same.

## C315. The checker's gaps the stage programs showed (2026-11-03)

The user's, before the compiled tiers: the stage programs kayak did not
run as Fixpt runs them, for its checker.
- **A name a `lambda` or a `letrec` binds is its own** where it is bound,
  not a module's item of that name (`refs-in`): `generative.fx`'s
  `(define total (letrec ((total …)) total))` was refused as made from
  itself.
- **`stage` may cross into a process** (`crossing-atom?`), as Fixpt's
  `crossing_atom` has it: `in-a-process.fx`.
- **A module type may describe a type function** (`(desc pair-of
  (dlambda …))`): such a `moduleof` is FX-26's, not a KFX trait, and its
  description is read as a function. Reps (`kfx/reps.kfx`) carry a
  `dlambda`, its parameters bound as a `poly`'s, and a module's effect
  description as Fixpt's `effect_desc` has it, `(dlambda {} (d-effect
  E))`, among the descriptions in the order written. A session's checker
  has its module's types and effects in scope (`module-types-opened`),
  its abstract types named `stage-module..t`, as Fixpt's: so
  `session-types.fx`. (Exporting `mtype-opened` from the `expressions`
  module broke the bootstrap's second generation, its Rust not building;
  not looked into, the opening written in vstage.kfx instead.)
- **`attempt-within`** (Fixpt's 3e026d63): lowered to
  `fx26:processes.fx`'s `attempt-within1` as Fixpt's parser writes it
  (op 55, C314); checked as `attempt` is, `ms` an `int`, `on-timeout`
  taking nothing, its value fitted to the body's, the effects all theirs
  (Fixpt takes the wider of the two types; kayak fits the timeout's to
  the body's, the same where they are one type). `attempt/within-*` check
  and run as Fixpt's, on the engine and tier 1.
- **`kayak-fx26 --run` of a program with no `main`** runs it and exits 0,
  as Fixpt's `run`; the stage test runs each program so, comparing its
  output and status, rather than appending `(main)`.
Checkers: 461 the same of 572 (454), refuses 22 (29); stage programs:
22 of 23 as Fixpt prints them, `session.fx` but for `car`'s type printed.

## C316. `stage` and sessions on the compiled tiers: a design (2026-11-03)

`docs/research/compiled-stage.md`. Three ways: (a) kayak's front end (KFX
compiled to Rust) linked into the cells program, staged code compiled to
cells and read into the running heap by tier 1's own listing reader, as
Fixpt's compiled machines compile staged code with a front end of their
own; (b) staged code given to the engine, impossible for procedures while
the heaps are two; (c) one heap, the engine moved onto tier 1's (Fixpt's
layout). The user asked whether the heaps must be two: they need not,
being so by history (the engine, C58 and C64, before C121 took Fixpt's
formats; tier 1, C122, on them); one heap would still leave two kinds of
procedure, needing adapters both ways. Recommended (a), (c) apart. Four
questions for the user: which binary, staged code at tier 1's speed for
now, the engine-independent half of `vstage.kfx` shared, and one heap.

## C317. One heap first: a plan (2026-11-03)

The user's, answering C316's question 4: unify the heaps before `stage`
comes to the compiled tiers. `docs/research/one-heap.md`: the engine's
heap API (some 700 uses, nearly all through `alloc!`, `obj-ref` and the
rest; the encoding's arithmetic confined to `heap.kfx` and four places)
written over tier 1's heap, Fixpt's layout, each engine object given a
kind of Fixpt's (an interpreter's own among them: `closure`, `env-frame`,
`continuation`), the collector tier 1's given the engine's roots; then
printing and numbers shared, one scheduler, one program; then the
compiled-stage design on the one heap.

## C318. The engine's values in Fixpt's encoding (2026-11-03; one heap, step 1a)

`docs/research/one-heap.md`, step 1, its first part: the engine's words
are Fixpt's values (`value.rs`), in `fx26/heap.kfx` behind the API every
caller uses (`fixnum`, `immediate`, `immediate-kind`, the `w-` constants).
- **Tags**: three bits, Fixpt's: a fixnum `000`, shifted by 3 (61 bits,
  Fixpt's range, was 62); an immediate `011`, its subtag in bits 3 to 8,
  its payload above; an object `100` (Fixpt's bloblet's), at its index
  shifted by 3, `vbase` 0 while the heap is the engine's alone.
- **Fixpt's immediates** where Fixpt has one: `#f` 3, `#t` 11, `nil` 19,
  `#u` 27, the unbound marker 59, characters (subtag 8), `f32`s (subtag
  9, their bits the payload, no longer a bias past the engine's specials).
- **Not yet Fixpt's**, marked so: symbols, primitives, prompt tags and the
  machine's own specials (`w-paused`, `w-failed`) are immediates of
  subtags Fixpt has none of, 24 to 27, until each is an object as Fixpt
  has it (1c); objects keep the engine's header and payload, pairs among
  them (1b).
- **An IR node** in a frame or a procedure is a fixnum, its number in the
  IR's table: the engine's fourth tag, `11`, gone (Fixpt has no such tag).
All 3788 tests as before. `scripts/engine-bench.py` (new): Fixpt's
benchmarks on the engine, best of 3, some 2% slower (407 to 417 ms for
`captures.fx`, 8425 to 8641 for `loop.fx`), the subtag's mapping to a
kind. Found on the way (TODO): the compiler no longer runs on the engine,
which has no `rep-nodes`.

## C319. The engine's objects in Fixpt's layout (2026-11-03; one heap, step 1b)

The engine's objects are Fixpt's (`heap.rs`, `layout.fx`), in
`fx26/heap.kfx`, its collector Cheney's over them.
- **A pair** is Fixpt's: two words, tag `001`, no header (`pair!`,
  `pair-car`, `pair-cdr` direct).
- **Any other object** is a bloblet, its value at its suffix, tag `100`:
  Fixpt's header (the kind in 8 bits, F fields in 18, B suffix bytes in
  32; past 2^18 fields an extension word before it, kind 255), its
  fields, the last a trailer (tag `101`, F) where it has any. An object of
  raw words (strings, bytevectors, reals, bignums) has them as its suffix,
  never traced, as Fixpt's; one of no fields no trailer.
- **Kinds** are Fixpt's where the engine's object is Fixpt's: box, sum,
  product, an FX-26 array (Fixpt's bloblet of fields: the engine printed
  it so already), datum's vector, closure, continuation, environment
  frame, flat array, eqtable, flonum. The others are kinds past Fixpt's,
  200 on, until each is Fixpt's (1c): datatype values, the engine's flat,
  its frames, constructors, FX-26 bloblets and blobrefs, strings,
  bytevectors, bignums.
- **Two orders of fields.** Fixpt's own kinds keep `alloc`'s order,
  payload word `i` the word `i` after the header, as tier 1 reads them
  (`obj-ref`, its trailer read for F). The engine's private objects
  (frames, environments, closures, continuations), which only it reads,
  are in Fixpt's slot order, word `i` field `i + 2` at `p - 2 - i`, no
  trailer read (`slot-ref`): the machine's hot path.
- **The room** a step may use is two thirds of what is free: an object
  is at most half again its old size (a trailer), a pair smaller.
- **The collector**: Cheney's, its forwarding a tag-`111` word in a
  pair's first word and below a bloblet's suffix; the copies scanned
  header by header (a word that is no header begins a pair).
All 3788 tests as before, both REPLs' sessions. `engine-bench.py`, best
of 3, from before C318: 11 to 17% slower (`loop.fx` 8425 to 9713 ms,
`tak.fx` 757 to 843): a kind is only in the header, two loads to reach,
and Fixpt's kinds' fields need their count read. Measured on the way:
the first layout, `alloc`'s order everywhere and half the room, was 60%
slower; the slot order for the engine's own objects took it to 24%,
direct pairs to 15%.

## C320. The engine's strings and bytevectors, Fixpt's (2026-11-04; one heap, 1c)

Fixpt's layouts, as tier 1 has them: a bloblet of no fields, kind
`string` (1) or `bytevector` (4), its suffix a word of the length, then
the code points two to a word, or the bytes eight to a word, the first in
the low bits. In `fx26/words.kfx`: `string-alloc!`, `bytes-alloc!`,
`raw-length`, `char-at`, `char-set!`, `byte-at`, `byte-set!`, and
`raw-ref`/`raw-set!` by kind for what the two share (copies, equality);
every string and byte operation moved to them from `obj-ref`. Text
crosses to I/O two code points to a word, as the points syscalls already
took them for tier 1 (`per` 2). Three the tests had not reached, found
reading what was left on `obj-size`: `bytevector-length` and
`bloblet-bytes`' length counted words, and `utf8->string`'s room was too
small for a string now half a word a character; a program of kayak's
now reaches them (`tests/fx26-kayak/heap/strings-and-bytes.fx`, every
tier, as Fixpt prints it). Two more the check found (`engine-programs`,
the corpus on the engine: `bloblets`, `f32`, `flat-sums`): a bloblet's
byte read and write (`bloblet-byte`, `bloblet-set-byte!`) were still a
word each. All tests as before; speed as C319's.

## C321. The engine's symbols and keywords, Fixpt's (2026-11-04; one heap, 1c)

A symbol is Fixpt's object, kind `symbol` (2): its name (a string), its
hash (`symbol-name-hash`, Fixpt's) and its global slot; a keyword,
kind `keyword` (48), its symbol. Interned, one object a name, found by the
name's number in a table of each (`symbol-objects`, `keyword-objects`),
made the first time it is asked for and kept by the collector, which
forwards the tables as roots. Not yet as Fixpt's: the global slot holds
the engine's number for the name (tier 1's holds 0), so that a name's
number is one read away, as a sum's tag and a product's labels are still
numbers (next); settled once the heap is one. Printing, staged code's
reading back, `symbol?`, `symbol->string`, keywords' conversions through
the objects; `string->symbol` and `symbol->keyword` given room for what
they may make. The engine's immediates of subtag 24 are gone. All tests
as before; `engine-bench.py` some 6% faster than C320's.

## C322. The engine's sums, Fixpt's (2026-11-04; one heap, 1c)

A sum is Fixpt's (`make_frozen`): kind `sum`, frozen (its header's bits
12 and 13, `frozen!`), its tag, a symbol (C321), slot 0, its value slot 1
(`sum-made!`, `sum-tag-number`, `sum-value`); was a fixnum, the tag
name's number, then the value, in `alloc`'s order. Made by the machine's
`sum` (its frame holds the tag's number), by `sum-word!`, and by I/O's
results; read by `tagcase` (the tag's number from its symbol's third
field, C321) and printing. Datatype values (`ob-data`) keep a number tag
until they are settled (a kind past Fixpt's). All tests as before; the
corpus on the engine as before.

## C323. The engine's products, Fixpt's (2026-11-04; one heap, 1c)

A product is Fixpt's (`make_frozen`): kind `product`, frozen, its fields
in slots 0 on in its type's order, no label field (was a fixnum, the
labels' number, then the fields). So a field is found by its index, not
its label: the lowering takes the checker's `extract-index` fact (now
among the facts it is given, `with-entries`) and makes `i-field`, a slot
read; a numbered field the lowering wrote itself (`par`'s `(extract %par
1)`) is field `k - 1`, as the cell generator has it. What the runtime
makes (I/O's errors, sizes and file information, a stage's failure, a
session's redefinition, evidence given) is made so (`product-of!`).
- **Module values stay labelled**, a kind past Fixpt's (`ob-module`,
  209), their own IR node (`i-module`), each value found by its name
  (`i-extract`), as a `with` of one needs: making them Fixpt's products
  needs their reshaping where a type orders them otherwise (C216), as
  the cell generator does: a step of its own.
- `tagcase` takes a product's fields from its slots, a datatype value's
  (still the engine's, a number tag) as before.
The corpus on the engine (`engine-programs`, which the test suite does
not run) found the last two: datatype values destructured as products,
and `par`'s fields, which no fact names. All tests as before; the corpus
on the engine as before. `engine-bench.py`, best of 3, against before
C318: 0 to 4% faster (`loop.fx` 8425 to 8125 ms, `tak.fx` 757 to 728),
a field by its index rather than its label searched.

## C324. The engine's datatype values, Fixpt's (2026-11-04; one heap, 1c)

A datatype value is what Fixpt makes it: a sum of a product, the sum's tag
the constructor's symbol, the product its arguments in order (was a kind
of the engine's own, `ob-data`, a number tag and the fields). So
`tagcase` sees only sums: a tag is a sum's tag, the value's fields a
product's slots (`product-to-slots!`), and the engine's `ob-data`,
`args-fields!`, `fields-to-slots!` and `vshow`'s case for it go. `sum?`
(op 122) is a sum's kind alone. All tests as before; the corpus on the
engine (`engine-programs`) as before, 89 of 96.

## C325. The engine's bignums, Fixpt's (2026-11-04; one heap, 1c)

A bignum is Fixpt's (`write_bignum`, and tier 1's `int-value!`): kind
`bignum` (6, was the engine's 208), a raw suffix of its sign (a fixnum, 1
if negative), its count of limbs (a fixnum), then the limbs, 64 bits each,
the low first (were 32 bits, as object words, the sign a raw word). A limb
above 2^63 is kept as the word of its 64 bits (`limb-word`, `word-limb`).
Still taken apart and put together by halves (C99). The room asked for
(`int-room`, `text-room`, `shift-room`) is in suffix words, now fewer for
the same integer, so it stays enough. Fixpt's bignum programs
(`bignums`, `int-fast`, `shift-counts`, `big-literals`) print alike.

## C326. The engine's arrays, bloblets and blobrefs, Fixpt's (2026-11-04; one heap, 1c)

An FX-26 array and a bloblet are one thing, as in Fixpt: a plain bloblet
(kind `bloblet`, 32; `%make-bloblet`), its values in slots from 0
(`bloblet_value`, below the trailer, the last first), a trailer always,
its bytes its own suffix (`bloblet-alloc!`). Before, an array's elements
were in `alloc`'s order, and a bloblet was an object of its own (204), its
bytes a bytevector in its field 0 and its fields after.
- **A blobref is Fixpt's suffix ref** (tag 010, the owner's suffix's
  index in bits 3 to 37, the byte above), not an object (205) of an owner
  and an offset: it allocates nothing, and the collector moves it with its
  owner (`copy-word!`). Printed as Fixpt prints one, `#<suffix-ref
  bloblet +8>`.
- A flat sum's tag with no key is the index of the owner's value holding
  the label (`%fx26-flat-tag`), now a slot.
- The engine's own tables (`eqtable`'s buckets and entries) stay arrays of
  the engine's making, in `alloc`'s order: an eqtable is not yet Fixpt's.
Fixpt's bloblet programs (`tests/fx26/programs/bloblet`, the runs among
them) print alike.

## C327. The engine's flat arrays, Fixpt's (2026-11-04; one heap, 1c)

A flat array is Fixpt's (`make_flat_array`): kind `flat-array` (42), its
layout's code in its one slot (Fixpt's `FLAT_*`: 0 `i32`, 1 `u32`, 2
`i64`, 3 `u64`, 4 `f32`, 5 `f64`; `i32-flat` and the rest now give these,
not a width), its elements' bits its suffix, 4 or 8 bytes each, two to a
word when 4; read as `flat_value` makes a value of them, written as
`flat_bits_of` (`flat-value`, `flat-bits-of` in vprims.kfx). Before, its
elements were the engine's words, whatever their layout, after a width.
KFX's `make-flatarray-i64`, which the engine ran as a kind of its own
(201), is a flat array of `i64`s.

## C328. The engine's module values, Fixpt's (2026-11-04; one heap, 1c)

A module value is a Fixpt product, its values in its type's order and no
labels (was a kind of the engine's own, 209, its labels' number then its
values, each found by name). As the cell generator does (C216), the
lowering knows each module's layout and finds a value at its place:
- **Made**: a `module` or `extend` in the order of the type wanted where
  it is (`module-arg`), else its own type's (`module-made`); a module
  defined, `(define m (module ...))`, as written, which a named module's
  `%exports` says (`lower-own-module`). A generative type's conversions
  are the module's own, not values it gives (they were in `%exports`).
- **Read**: a `with`'s names at their places in what opens it (a named
  module's `%exports`, else the checker's `with-spec`), a slot read
  (`i-field`); an `include` likewise. A `with` whose layout nothing says
  opens nothing (was every name any module gives, by name).
- **Given where a type orders its values otherwise**: reshaped, a product
  of the type's values picked from it by place (`lower`, Fixpt's
  `k-reshape-at`), from a named module's `%exports` or the checker's
  `module-had`. The checker's module facts reach the lowering as
  `%mfact` entries, kept in a top scope's last frame (`top-scope`), so
  that each expression looks among a few.
- A session seeded from a module (C310) takes its values by their places
  in its type.
- **Lowered with no facts** (the REPL front's draft, tried before it is
  run, C311), an extract or a `with`'s value whose place nothing says
  lowers to a failure if run, as the fallback by name did, so that the
  try is not refused.
- `i-module`, `i-extract`, their nodes and frame, and `ob-module` go.
**The checker noted no order for a type written `(define-type ab
(moduleof ...))`** (a `%module ab`, a module parameter's type), only for a
`%mtype`: a module given for one was not reshaped, so the cell generator
read its values at the wrong places too (`(first-b k)` gave 1, not 6, on
tier 1). Now both (`module-value-order`). A new test,
`tests/fx26-kayak/modules/values-in-order.fx`; Fixpt's checker panics on a
reshape inside `list` (docs/requests/fixpt-reshape-in-list.md, for the
user to take to the Fixpt team), so the test keeps its reshapes out of
one. All 890 of Fixpt's programs print on the engine as before.

## C329. Eqtables, Fixpt's, on the engine and on tier 1 (2026-11-04; one heap, 1c)

An eqtable is Fixpt's (`eqtable.rs`) on both: a bloblet of kind `eqtable`
with three fields and no trailer (the count of collections when its keys
were last hashed, its count, its buckets), the buckets a `vector`, bucket
`i` its element `i`, an entry a vector of key, value and next; a key's
bucket SplitMix64's finalizer over its word, modulo the buckets
(`bucket_of`). Before, each was kayak's own (C220): the engine's an
object with a trailer, its buckets and entries FX-26 arrays; tier 1's a
bloblet with a trailer, its entries' elements in slot order (the key
where Fixpt's `next` is), and both hashed `key >> 3 ^ key >> 12`. Fixpt's
eqtable has not changed since (0fb1d40f, 2026-09-30): these were kayak's
departures, not drift.
- **A bloblet with fields and no trailer**, its header found below its
  fields (Fixpt's `find_main`): the engine's `fields-alloc!`, `field-at`,
  `main-in` (which `header-at` and the collector use). Tier 1's heap had
  it (`header-scanned`).
- **The hash in machine words**: KFX's `i64*` wraps, so SplitMix64 is
  `i64` arithmetic, its right shifts made logical by a mask and its
  remainder a `u64`'s by halving (checked against `bucket_of` on 200000
  random keys). Through KFX's `int` and `u64`, tier 1's `native/eqtables.fx`
  under `--gc-stress`, which rehashes at every operation, ran past the
  test's time; now 15 s.
Fixpt's eqtable programs (`run/eqtable.fx`, `native/eqtables.fx`, 10000
entries across collections) print alike on both.

## C330. The engine's prompt tags and mark keys, Fixpt's (2026-11-04; one heap, 1c)

A prompt tag and a continuation mark key are what Fixpt makes them
(`cellular.rs`: `make-continuation-prompt-tag`, `make-escape-tag`,
`make-continuation-mark-key`): a new box of unit, compared by identity.
The engine's tags were an immediate of its own (subtag 26), numbered as
made; its mark keys a box of 0. Its last immediates past Fixpt's but one
go (a symbol's, 24, unused since C321; the "special", 27): a primitive
stays one (25), a procedure, which reaches compiled code through an
adapter as the engine's closures will (one-heap step 4).


## C331. Tier 1's heap, ready to share (2026-11-04; one heap, step 1's last part)

The engine is to load tier 1's heap (`cells/layout.kfx`, `cells/heap.kfx`)
and be written over it (docs/research/one-heap.md, step 1). First, tier 1's
side, which changes nothing it does:
- **Its names that the engine's program defines too, renamed** (27; by a
  rename that leaves strings, comments and locals alone): `alloc!` is
  `heap-alloc!`, `top` `heap-top`, `obj-ref` and `obj-set!` `elt-ref` and
  `elt-set!` (an element in `alloc`'s order), `tag-of` `word-tag`,
  `bytes-of` `suffix-bytes`, `words-of` `header-words`, `code` (the code
  area's array) `code-space`, `code-free` `code-frees`, `globals`
  `named-globals`, `list-of` `heap-list-of`, the `named` datatype
  `named-slot`, `index-in` `position-in`, `v-unit` `v-unit-word`, and the
  routines' numbers `r-` `rt-` (all 53, three clashing). Those that mean
  what the engine's of the same name mean (`pair-car`, `pair-cdr`,
  `is-pair?`, `slot-set!`'s neighbours, `collections`, `zeroed!`,
  `symbol-text`, `copied-into!`, `sub-char`, `sub-f32`) keep their names:
  the engine is to use them, or rename its own.
- **Its heap's effects narrowed** from `cm`, which has I/O, to what a heap
  does: `chw` (reading and writing `@c`) and `cha` (and allocating there,
  for its names, its code blocks' and weak pairs' lists). `cm` has both.
  Only `make-code-word!` keeps `cm` (`code:flush/1`).

## C332. The engine on tier 1's heap (2026-11-04; one heap, step 1's last part)

The engine's program loads tier 1's heap (`cells/layout.kfx`,
`cells/heap.kfx`, in `fx26/fx26.unit`), and its own heap file, now
`fx26/vheap.kfx` (a unit's files are Rust modules by their names, and
two `heap`s clash), is written over it:
- **One region**: the engine's `@h` is `@c`, tier 1's, everywhere in the
  engine (a rename by tokens, not strings). Tier 1's heap effects
  (C331) are three, `cread` (only reading, which the engine's reading
  procedures need), `cwrite` and `calloc` (C331's `chw`, `cha`: `chr`
  would be the reader's character datum). The engine's `hp` is `cwrite`.
- **One heap**: tier 1's spaces (`mem`, `other`), each reserved at its
  capacity and committed as touched, its `heap-top` and `limit`, its count
  of `collections` (which the eqtables stamp with, C329). The engine
  allocates from them its own way (`bump!`, not zeroing, as it writes each
  word); it grows by the limit (`room!`), never by copying.
- **One collector**: tier 1's, Coopt's baseline, now with large headers
  (an extension word before the main one, Fixpt's `put_header`, which the
  engine makes for more than 262142 fields and tier 1 never did). The
  engine's `collect!` gives it its roots between `collect-begin!` and
  `collect-end!`; the engine's own Cheney copy is gone.
- **One symbol table**: a name's symbol is tier 1's (`symbol!`); the
  engine keeps its name's number in its global slot, which tier 1 leaves
  0, and its cache by number.
- **Shared accessors**: `address`, `heap-ref`, `is-pair?`, `pair-car`,
  `pair-cdr`, `is-sref?`, `make-sref`, `sref-owner`, `sref-offset`,
  `header-below`, `slot-of`, `slot-set!`, `frozen-whole!`, `suffix-bytes`,
  `symbol-text`, `word-tag`, `sub-char`, `sub-f32`, and a new pair,
  `mem-at` and `mem-set!`, a word of the heap's own space with nothing
  tested (pairs and the engine's values are never in the code area, which
  tier 1's `word-at` tests for first). The engine's of the same meaning go;
  its value-order slots are `bloblet-value` (Fixpt's `bloblet_value`), as
  tier 1's `slot-set!` is a field's by number.
- **Inlining**: so written, the engine ran 55% slower (`loop.fx`, 8.6 to
  13.4 s): rustc inlines no `fn` into another codegen unit unless it is
  `#[inline]`, and the engine's hot reads were now in tier 1's module.
  One codegen unit gave 8.1 s but built in 40 s, not 12; ThinLTO did
  nothing. So kfx now marks a small definition `#[inline]`
  (`emit-tops.kfx`'s `weight-under`: no loop, no `async`, at most 24 atoms
  and lists), every program's generated Rust with it, boot/ at its fixed
  point again; builds take as long as before.
All 907 of Fixpt's and kayak's FX-26 programs print on the engine as
before. `engine-bench.py` against C330's: 0 to 6% slower (`loop.fx` 8395
to 8827 ms, `helpers.fx` 8312 to 8793), `lists.fx` 6% faster (5225 to
4929): the values' base (`vbase`), a thread's constant read at every
`address`, is most of it (2.2 s of `loop.fx`'s 5 before the inlining).

## C333. The engine's numbers and printing, tier 1's (2026-11-04; one heap, step 2)

The engine's program loads tier 1's `numbers.kfx` and `show.kfx` too, so
that a number is made, read and written one way, and a value printed one
way, everywhere (docs/research/one-heap.md, step 2):
- **Numbers**: the engine's bignums are tier 1's `int-value!` and
  `int-of`, its reals `make-flonum!` and `flonum-of`, its `f32`s
  `f32-value`, `f32-of`, `is-f32?`. Tier 1's bignums now take integers
  apart and put them together by halves, the engine's way (C99: n log n
  for n limbs, not n squared, which tier 1's limb by limb was); the
  engine's code moved there.
- **An `f32` was not Fixpt's on the engine**: its bits from bit 8 (C318),
  where Fixpt's `Value::f32` puts them from bit 32 (`bits << 24`, then the
  payload's shift of 8), as tier 1 had. Now Fixpt's.
- **Printing**: the engine's `show-word` is tier 1's `written` (Fixpt's
  `print.rs`), which writes characters and strings as Fixpt does (more
  characters named, `\t`, `\r`, `\x…;`) where the engine's named and
  escaped fewer. Tier 1's `written` gains Fixpt's eqtable (`#<eqtable of
  N>`), closure (`#<procedure>`) and continuation (kind 14,
  `#<continuation>`). A procedure the engine writes is `#<procedure>`, not
  `#<closure>`, which was the engine's own (Fixpt writes one by its
  machine, `#<native-closure 2 fields 0 bytes>` on x86_64, its FX-26
  printer `#<procedure>`): three of Fixpt's programs print so now, which
  no answer compared.
- **Clashes** renamed: tier 1's `byte-at` is `raw-byte-at`, its `hex`
  `hex-text`; the engine's `written` (an I/O outcome, prims.kfx)
  `write-outcome`; the engine's `two-64` goes, tier 1's the same. Tier 1's
  numbers and printing take the heap's effects (`cread`, `cwrite`), not `cm`.
All 907 programs print as before but those three.

## C334. One scheduler's state for the engine and tier 1 (2026-11-04; one heap, step 3)

The engine's program loads tier 1's `cells/sched.kfx` (C125, a port of
Fixpt's `sched.rs`): the scheduler's state vector (`sched-state`, which
the engine's `collect!` now forwards; it was among the engine's
constants), its tasks, run queue, I-cells, mailboxes, timers, groups,
monitors and cancellations, the waits on the input and on processes.
The engine's `vsched.kfx`, which was a second port of the same, is now
its dispatcher alone (`sched-op!`, `%sched`'s operations by number), as
tier 1's is in its prims.kfx; the two dispatchers stay apart, each giving
failures as its machine does: the engine checks an I-cell written twice
and an empty mailbox before tier 1's procedures would `trap!`.
- **The engine's I-cells and mailboxes were kinds 16 and 17**, Fixpt's
  `promise` and `hash-table`; now tier 1's, Fixpt's `icell` (46) and
  `mailbox` (47), their state in slots 2 to 4.
- **Messages are copied**, as Fixpt's and tier 1's are (`copy-within!`:
  sharing and cycles kept, what is shared not copied); the engine's
  were shared, which its checker's rule that what crosses is
  transmissible made the same to a program but for identity.
- Renamed, as the engine's program has the names: tier 1's `st`,
  `st-set!`, `st-cons!` (the checker's state is `st`) are `sched-st`,
  `sched-st-set!`, `sched-st-cons!`; its `filled!` `items-filled!`. Its
  `kept-of` takes a predicate of `cread`, not `cm`, which gave every
  caller tier 1's I/O.
All 907 programs print as before, and the REPL's sessions.

## C335. The front end in tier 1's program (2026-11-04; compiled stage (a), step 1)

The user (2026-11-04), of `docs/research/compiled-stage.md`'s (a), the
front end linked into the compiled program's runtime as Fixpt's
conductor is: "Start on (a)". Its open questions taken as the note
leaned: one binary (`kayak-cells` grows by the front end, as Fixpt's
runner is one), staged code at tier 1's speed first, and the engine's
staging half that is no engine's shared. Step 1, which changes nothing
tier 1 does:
- **`cells/cells.unit` loads the front end**: kfx's checker
  (`kfx/names.kfx` to `check-defs.kfx`) and kayak's compiler to cells
  (`fx26/ir.kfx` to `fx26/listing.kfx`, as the engine's unit has them),
  before tier 1's own files. The one heap (C318 to C334) made most of the
  note's 25 clashes go already.
- **Two files renamed**, a unit's files being Rust modules by their
  names: tier 1's `prims.kfx` is `cprims.kfx` (after the engine's
  `vprims.kfx`), its `listing.kfx`, which reads listings, is
  `listing-reader.kfx` (`fx26/listing.kfx` writes them).
- **Nine names renamed on tier 1's side** (the rename that leaves strings,
  comments and locals): `program-args` `run-args`, `wrapped`
  `int-wrapped`, `before-char` `chars-before-char`, `drop-n`
  `chars-dropped`, `word-named` `listing-word`, the datatype `item`
  `hand-item`, `name-of` `word-name-of`, `padded` `padded-to`, and
  `keyword!` `keyword-made!` (one in Rust with the front end's
  `keyword-x`, which no scan of KFX's names finds).
- **Its cost**: kfx makes tier 1's Rust in 16.9 s, not 4.3; the crate
  rebuilds in 5.3 s, not 2.8; the binary is as it was (2,450,848 bytes,
  not 2,450,792), nothing calling the front end yet.

## C336. Staging's machine-free half shared (2026-11-04; compiled stage (a))

`docs/research/compiled-stage.md`'s question 3, taken as leaned: what of
`stage` is no machine's moves from the engine's `vstage.kfx` to
`fx26/staging.kfx`, which both units load (after tier 1's scheduler):
the code's datum read back as the reader would have read it, each part
placed (C312); what it is checked as and the check, in a worker first
(C111); a refusal's words and place, made a `stage-failure`; evidence
filling a rep (C307). Written over tier 1's heap, which is the engine's
(C332): its few makers its own (`staged-sum!`, `staged-product!`,
`heap-kfx-bytes`, `kfx-bytes-heap!`). The engine keeps `stage-run!`, which
lowers what passed and makes it an engine closure, and sessions (to move
when tier 1 has them, step 5). The datatype `span` is `code-place`
(tier 1's hand machine has a `span`). Nothing either machine does
changes, but for one found on the way:
- **A symbol tier 1 made had no number for the engine.** The engine kept
  a symbol's name's number in its global slot (C321, C332), which tier 1's
  `symbol!` leaves 0: so a failure's tag `failed` that staging made first
  read as name 0, and a session's `tagcase` of it missed (the REPL's
  `session.in`). The slot now holds the number plus 1; 0, not set, the
  number found by the symbol's name (`name-at`), as a tier 1 symbol's in
  the one heap must be.

## C337. `stage` on tier 1 (2026-11-04; compiled stage (a), step 2)

Staged code runs on tier 1 as on Fixpt's compiled machines (`stage.rs`):
compiled to cells where the program runs, by kayak's own compiler, which
tier 1's program holds (C335).
- **Compiled**: the cell generator writes `(%stage E R m code given)` as a
  call of tier 1's new primitive `%kayak-stage-run` with the module, the
  code, the evidence given, and the stage's facts as literals, its rep's
  text and the evidence's names (`stage-rep`, `stage-given`), as the
  engine's lowering writes `%stage-run` (C305).
- **Run** (`cells/stage.kfx`, called by the machine's `prim-run!`, as it
  reads a listing, which `cprims.kfx`, before the reader, cannot): the rep
  made from its text (`rep-of-text!`) and filled from the evidence; the code
  read back and checked in a worker (staging.kfx, C336); refused, a
  `stage-failure` with its places, as the engine's; else checked again for
  the facts, compiled by `compile-cells` as `(define %stage-K (lambda
  (stage-module) (lambda () (with stage-module CODE))))`, its listing read
  into the running heap (`listing-added!`, new in the reader: a word the
  heap has by its name, its own `program` and the standard words both
  listings hold, left as it is), and the thunk's word, found by its body's
  place (as the compiler names a lambda's word; each stage's places are
  its own), made a closure over the module: `(sum ok THUNK)`.
- **`cm`, tier 1's effect, grows** by the compiler's region (`@g`) and the
  check's worker (`@q`, `@w`, `nondet`).
- **Programs with a `main`, run as `fixpt run` runs them**: `kayak-fx26
  --cells-run` lists the words with `(main)` last; `kayak-cells --main`
  writes no value and exits with an integer one.
- **The stage test runs tier 1 too** (`crates/kayak-checks/tests/stage.rs`):
  of Fixpt's 23 stage programs, 13 as Fixpt runs them
  (`tests/stage-cells-passing`): `stage.fx` (refusals' words and places,
  a procedure made, thirty stages while collections come), `generative`,
  `in-a-process` (a stage in a process beside another, tiny slices),
  `kept`, `sealed`, `state`, `tail-across`; and six refused as they are
  checked. The other ten need evidence (`%reify`, bounds; step 4) or
  sessions (step 5). Staged words run as cells under any tier; the other
  tiers are step 3.

## C338. Staged words under every tier (2026-11-04; compiled stage (a), step 3)

What a stage compiles runs as cells whatever tier runs the program: tier
2's hand machine hands back what it lacks, and tiers 4 to 6 compile only
what they loaded, so a stage's words, read in later, are run by tier 1's
cells. The stage test now runs each staging program seven ways
(`tiers`): tier 1, `--regs`, `--gc-stress`, and, where the hand machine
runs (x86_64 Linux, arm64 macOS, as the cells test has it), `--hand`,
`--compiled`, `--regs --compiled`, `--regs --direct`; each must print
what Fixpt prints. The 13 of C337 do, every way. Staged words compiled
to machine code stay later (the note's step 6, after Fixpt's 516abc8e).

## C339. Evidence and bounds on tier 1 (2026-11-04; compiled stage (a), step 4)

`reify-effect`, `reify-type` and `reify-region` compile to a call of tier
1's new `%kayak-reify` with the evidence given, a product of them in
order, and what the checker noted (`reify-rep`, `reify-given`), as the
engine's lowering writes `%reify` (C307); tier 1 fills the rep
(staging.kfx's `reify-run!`, and `rep-of-text!`, moved there from
cells/stage.kfx). A bound, `effect-bound` or `type-bound`, compiles as
Fixpt's does, to `%fx26-evidence-bound`, which tier 1 lacked: the
evidence as it is. Of Fixpt's stage programs, 16 now run on tier 1 as
Fixpt runs them, every way (`evidence`, `evidence-bound`,
`evidence-region` more); the seven left need sessions.

## C340. Sessions on tier 1 (2026-11-04; compiled stage (a), step 5)

Sessions (C310, C311, C313) run on tier 1 as on the engine, compiled.
- **Their machine-free half to staging.kfx** (as C336 did for `stage`): a
  session's state (its checker's environment, globals, definitions,
  generation), the pending definitions, the checks in a worker, the
  redefinition rules and the reruns, `make-session`, `session-redefines`,
  `%session-committed!`, over tier 1's heap (a session and a pending
  definition each a vector), and an effect, `staging`, which both
  machines' have. The engine keeps its globals and its lowering.
- **Tier 1's half** (`cells/stage.kfx`): a session's globals are tier 1's
  global cells, named as Fixpt names them, `fx26-session-K:NAME`, numbered
  when defined again as a new one, a global's index its place in
  `session-names`; its code compiled with the session's names as their
  globals' (the cell generator's new `cg-aliases`, consulted after the
  locals); a commit compiled to a procedure of `(%kayak-global-set FULL
  init)` forms, kayak's own, which the generator writes as `global!`
  (an `x-bloblet` of its own, which register code declines, so the word
  runs as cells). `session-check` compiles to `%kayak-session-run`, as
  `%stage` does; it and `%session-ready` are the machine's, as `stage` is.
- **Staged code compiled as `(define %stage-K unit X)`**, `stage`'s too:
  written with a type (whatever it is: the generator notes only that it
  has one), its words are named for `%stage-K`, not by their places, which
  a staged code's, counted from its own base, may share with the
  program's (`lambda@P`).
Of Fixpt's 23 stage programs, 22 run on tier 1 as Fixpt runs them, every
way: the engine's 22 (`tests/stage-cells-passing` is `tests/stage-passing`);
`session.fx` differs on both only in a refusal's type printed (TODO).

## C341. The pin at Fixpt 8616f2f (2026-11-04)

Before porting 516abc8e (a compiled word's code in its own bloblet), the
user's "go ahead": the pin from 19b31f1 to `origin/main`, 8616f2f, three
commits.
- **bb635fe8, ec10e701**: 516abc8e on arm64 too, and its notes. The one
  ported source behind the pin, `cells/hand.kfx` (`fixpt-native`'s
  `cellular.rs`), is carried over with 516abc8e itself, the next step: its
  mark moved to the pin, as its 19b31f1 mark stood with 516abc8e's x86_64
  half not carried over (TODO).
- **8616f2fe**: a module reshaped twice over is reshaped once, both of
  Fixpt's checkers: kayak's request (docs/requests/fixpt-reshape-in-list.md,
  C328), landed, crediting it. Its test, `modules/reshape-in-list.fx`,
  found a silent case as well (the same values in the other order, the
  wrong one taken); kayak gives `(7 7 9)` on the engine and on tier 1, as
  Fixpt now does, neither case kayak's.

## C342. A compiled word's code reaches the machine by the state (2026-11-05; word code, step 1)

`docs/research/word-code.md`'s step 1, the user's "go for it" (arm64 on
Fixpt's interim, compiling nothing until it can be run; the port in steps,
the block's words kept meanwhile). On x86_64 a compiled word's code now
reaches the machine's own code only through the state, as Fixpt's does
since 516abc8e, so that it may be anywhere, a code bloblet of its own:
- **The machine's exported places** (`y-exports`): `NEXT`, the ways to
  give back, to resume, to return, to run out, the misaligned exit, the
  code's start, tier 6's ways back to tier 5 (`l-materialize`, `-pushed`),
  and, until step 3, the block's table of words. Their addresses are in the
  state from `st-exports` (the state 52 words now, `st-code-delta` among
  them for step 2), written by the machine at each entry, from a routine
  after the words (`l-exports`), once every place is.
- **While words are compiled** (`y-far-from`, a label's number, the first
  a word's), a jump, a call or an address of one of those places is
  `jmp [r15 + slot]`, `call [r15 + slot]`, `mov r, [r15 + slot]`: no
  register, as `r11` carries tier 5's code to `l-materialize`; a
  conditional jump goes to a stub of its own that jumps so
  (`y-stubs`). A place not exported is refused, loudly (tier 6's reach of
  the table of words was found so).
All tests as before, every tier.

## C343. A compiled word's code in its own bloblet, entered by its tag (2026-11-05; word code, steps 2 and 3)

`docs/research/word-code.md`'s steps 2 and 3 on x86_64, as one (step 2's
bloblets cannot run until step 3's dispatch reads them), Fixpt's 516abc8e:
- **A word's code** (`yword-made!`, `word-code!`): each word compiled after
  the machine is made and sealed, alone, its labels from the machine's last
  (`y-machine-labels`), so that every label of the machine is far
  (`y-far?`) and goes by the state (C342). Placed from an 8-byte field,
  then its code, then its data (its table), and put in a code bloblet of
  the code area's native part (`code-alloc!`, `code-set!`, `code:flush/1`),
  kind `bloblet`, frozen, field 0 the table's place in bytes past the
  suffix. Its entry is its reference: the collector marks it from there
  and frees it with the word. Its spans are noted for the perf map at the
  place it runs.
- **Where it runs**: the state's `st-code-delta`, set by tier 1's `hand!`,
  is the code area's distance less 12 (the field's word and the tag), so
  that the reference plus it is where the table's offsets are from.
- **The dispatch by tag**: `NEXT`'s word and `execute` (`yword-entered!`),
  a resume (`yresumed!`: a number below the routines' count the cells,
  else, a register word not compiled, given back, as before), register
  code's call of a twin (`yentry-address!`) and tier 6's invoke
  (`y6-invoked!`) test the entry's tag: a pointer is code, its table at
  the field's bytes past its suffix. Tier 1's `enter!` runs a word whose
  entry is code as cells, said, not by its number's size.
- **The block** holds the machine alone: its table of words, the export of
  it and the slot numbers past `rt-count` are gone.
- Where tier 1 has no code area (no `code:exec-view/1`), nothing is
  compiled: words run as cells, as Fixpt's interim on arm64.
fib, tak, loop, closures, lists and captures on `--compiled`, `--regs
--compiled`, `--regs --direct` time as before, but whole-process runs of
10 to 160 ms say little: step 6 measures. The cells tests (1817) pass.

## C344. Words read as the program runs compiled too; arm64 on Fixpt's interim (2026-11-05; word code, steps 4 to 6)

The rest of `docs/research/word-code.md`:
- **Compiled as read** (step 4): `compiled-read!`, through which a stage's
  and a session's listing reach the running heap (C337, C340), compiles
  the words `listing-added!` made (`words-read-compiled!`: the first of
  `words` and `register-words`, past the counts they had before), with
  `--compiled`, where x86_64's machine is made. So staged code runs as
  machine code on tiers 4 to 6, as on Fixpt's sessions; stage.kfx's
  "staged words run as cells" is gone. `cli-stage/stage.fx` compiles 77
  words on `--compiled`, 150 on `--regs --direct` (69 tier 6); the 23 stage
  programs pass on all seven tiers.
- **arm64** (step 5): the user's choice, Fixpt's interim at 516abc8e. Its
  machine compiles no word, so that no entry of it is a slot number, which
  tier 1's `enter!` no longer reads as compiled (C343); its empty table of
  words stays, which its routines read. bb635fe8's port is TODO, for an
  arm64 to run it on: code nobody has run is a guess.
- **Measured** (step 6): within 5% either way on every program and tier,
  compiled cells some 2% dearer; the table in word-code.md.

## C345. What staged code and sessions keep: kayak's, not Fixpt's (2026-11-05; docs)

The user asked whether Fixpt's API makes sessions grow without bound. It
does not: on Fixpt's compiled machines a session is all in the heap
(`sessions26.rs`, 6ddd924d: its front end, its globals that front end's
cells, its code its words'), collected when unreachable; only its Rust
checker's path (`sessions.rs`'s `SESSIONS`) grows. TODO.md's entry, which
said "Fixpt keeps them too", corrected; kayak's own causes named (every
word read named, so a root; a session's globals the machine's table; its
checker state and pendings kept); a plan, `docs/research/session-memory.md`,
not started (type reps first, the user's). `compiled-stage.md`'s status and
TODO.md's `stage` entry brought up to C344.

## C346. Reps compared with Fixpt's, byte for byte: the first part (2026-11-05)

The user's: parity with Fixpt, the pin's (8616f2f, not its newer logical
types, which wait for the user's review). `docs/research/type-reps.md`'s
"Parity", item 1 (the declined), led to the first comparison of kayak's
reps with Fixpt's: until now each side's reps were only checked against
themselves (kayak's `--reps` round trip; C300's byte-for-byte test was of
hand-written nodes, the crate's, not of kayak's export). A scratch tool
(not kept: it links the pinned clone's `fixpt-fx26`, `Checker::export_ty`)
printed each definition's rep as Fixpt's Rust checker makes it, for every
program of `tests/fx26/programs`: 800 of 1280 were Fixpt's, 480 differed.
Now 1091 are, by:
- **A type variable a `var` node** over it (both of Fixpt's checkers,
  `reps.rs`, `check-reps.fx:308`), a recursive type's variable its node
  still; and so in staging's fill (`rp-filled`, `rp-var-aliases`), a
  `var` node over a filled hole its filler, as Fixpt's `fill`.
- **`@heap` a region constant**, `(rconst @heap)`, as Fixpt reads a
  written `@heap`; Fixpt's `(heap)` is a written `heap`, another region
  (below).
- **Binders named as written** (`a`, not `a@148`; `p`, not `'p`; a spec's
  unnamed one empty, not `%none`), the kind in the binder node; the import
  spells each as kayak's checker does again, `@r` and its node, `%d` if
  distinct.
- **`datum`** the union Fixpt's standard abbreviation is, read back as
  `datum`; **`nil` or a pair** the pair that may be `nil`; an **`nlist`'s
  region** `(frozen #t)`.
- **Declined no more** (30 to 6): a module type by its abbreviation's name
  expanded by the checker's environment (`mtypes-expanded`, at `--reps`,
  a stage's and evidence's export); composable continuations,
  `(composable T A D R once?)`.
Left (189): a generative type's `gendef` node and Fixpt's numbering of
generative types (144), a `define-datatype` as the sum it is (17), a place
binder's kind (10), `param-sel`, a `poly` of several kinds nested, a
module's abstract type outside it a hole, descriptions' order, inferred
effects (`alloc` of `(frozen #t)` missing in one), and 6 still declined
(`app`, `convof`, two effect atoms).

Found on the way, a checker difference: at the pin Fixpt refuses
`(subr pure ((listof int heap)) (listof int @heap))` of the identity, `heap`
and `@heap` two regions; kayak takes them as one (C171) and runs it. TODO.

## C347. Reps compared with Fixpt's: datatypes, generative types, type functions (2026-11-05)

C346 continued: 1252 of the 1286 definitions of `tests/fx26/programs`
now have Fixpt's rep byte for byte (31 differ, 3 declined), by:
- **The checker's environment in the export** (`ty-export-in`,
  `ty-rep-in`; the state's `rxenv`, with a memo of the nodes made by key),
  where `--reps`, a stage and the round trip have it.
- **A `define-datatype` the sum it is**, as FX-26's is a `define-type` of
  one (each variant a product labelled from 1), a node made once for each
  application so that a recursive one is a cycle, its type functions'
  applications reduced (`reduced`).
- **A generative type's definition**, Fixpt's `(named <g KIND key> (gendef
  {params} binder… (variances v…) rep) d…)`, made once: Fixpt's 22
  standard ones from their declarations (`check.rs`, copied in
  `rp-generative-decls` with its `fixpt-source` mark, read as kayak reads a
  type); a program's from its checker's own (its parameters, its
  `%variance` marks, the representation `up-NAME` takes). Its key is
  Fixpt's number of it: a program's after the 22, in the order defined.
  The import skips a `gendef`.
- **Type functions applied** (`app F d…`), descriptions that are type
  functions `d-fun`, and **arrow kinds** `(=> K … R)` (kayak spells one
  `(=> (K …) R)`, read back so).
Left, each in kayak's checker's own types rather than the export: a place
binder kept as a region binder (11; the kind `place` is lost once read);
nested `poly`s merged into one (2); a module's abstract type seen outside
it (Fixpt a hole, `?0:counter..t`) and in a functor (`param-sel`), and a
module's abstract type of arrow kind (16); a module's descriptions' order;
a loaded module's global's name; a written `heap` (C346); one inferred
effect (`alloc` of `(frozen #t)`); and declined, `convof` and two effect
atoms (an effect family applied, `~!`).

## C348. `heap` and `@heap`, two regions (2026-11-05)

The user's: "Why are you equating heap and @heap? … Yeah fix that." C171
read a written `heap` as `@heap`, kayak's own choice, never Fixpt's: at the
pin Fixpt's parser reads every `@NAME` as a region constant and `heap` as
`Region::Heap`, the collected heap as a place (`docs/fx26.md`), and
refuses a `(listof int heap)` where a `(listof int @heap)` is expected;
kayak ran it. Now the heap's place is a region of its own, `heap` (no
mark: no constant or binder is spelt so): a written `heap`, `(place
heap)`, and the place nothing settles (C284, as Fixpt's `infer.rs` takes
`Region::Heap` for one), so too frozen data of the heap's place
(`frozen-region-string`, `heap-frozen`). An unsolved region printed as
written stays `@heap`, a default, not the heap's place. Reps: `(heap)`,
read back as `heap` (`with-shadows-applied.fx` now Fixpt's rep). Test:
`tests/fx26-kayak/checker/heap-is-not-at-heap.fx`, refused by both. No
program's answer changed.

## C349. A place binder's kind kept (2026-11-05)

The user's next, after C348. A binder declared `(p place)` was kept as any
region binder, `'p`, its kind gone once read (the parse's `%place 'p`
was the body's alone), so its rep said `(binder (region) #f)` where
Fixpt's says `(place)`, and it printed `(p region)`. Now:
- **A definition's scheme** keeps `%place 'p` among its binders' marks,
  after them, as `%distinct 'r` is (C204); `projected`, which a `proj`
  walks, stops at a mark as `instantiate-vars` does (a mark there crashed
  it, `cdr: nil`). Renaming the binder itself (`'p%p`) was tried and
  undone: a body's written names are found among the scheme's binders.
- **A `poly` type's binder** is named `'p@P%p`, as a distinct one `'r@P%d`.
- **Printed** `(p place)` (`binder-kind-shown`), and its **rep's** binder
  `(binder (place) #f)` (`scheme-poly`, marks to marked binders); the
  import spells one back `'p@rK%p`.
`binder-kind` still says `region` for a place, as the checker's other
uses of it want (a place is a region). 1262 of the 1286 definitions'
reps are Fixpt's; left, C347's list but the places, and a region bound
within a place (`(binder (region) #f #0.0)`, `caller-place.fx`).

## C350. A request: `fixpt check --reps` (2026-11-05; docs)

The user asked for a request document and a short prompt for the Fixpt
team: `docs/requests/fixpt-check-reps.md`, a flag printing each
definition's rep (`NAME HEX` or `NAME declined`), so that kayak's checks
compare reps through the `fixpt` binary instead of a program linking
Fixpt's crates (C346's scratch tool, its source in the request).

## C351. The pin at Fixpt 1311497 (2026-11-05)

The user's: "Pull Fixpt from GitHub. Adapt your workflow to fit its
addition of `check --reps`. … then start work on … allof/anyof/oneof." The
pinned clone fetched from its GitHub `origin` and moved from 8616f2f to
13114974 (20 commits): Fixpt's logical types (U0 `oneof`, U1 `allof`,
U2/U2b `anyof`, U3/U3b rule 5) and `fixpt check --reps` (kayak's request,
C350). Carried over:
- **`oneof`** read as `union` was (`union` still read, as Fixpt's), printed
  `oneof`, its rep's head `oneof` (`union` still imported), the emitter
  and the REPL's lists of type heads too: Fixpt's prelude
  (`kfx/built-in.kfx`, copied) and its `unions/` tests are written with it
  now. The rest of U0 (two sums of no tag in common one sum, two sharing
  one refused) and U1 to U3 are the next work.
- **`check.rs`'s generative declarations**, copied in `kfx/reps.kfx`
  (C347): unchanged since 8616f2f (only `DATUM`'s `union` became `oneof`,
  which kayak's export writes itself); the mark moved.
The new Fixpt programs (`allof/`, `anyof/`, `unions/oneof-sums*.fx`) are
not yet kayak's; no passing list dropped.

## C352. Reps against Fixpt's by `fixpt check --reps` (2026-11-05)

The user's "Adapt your workflow to fit its addition of `check --reps`"
(Fixpt 13114974, kayak's request C350). C346's scratch tool, which
linked Fixpt's crates, is gone; instead:
- **`tests/reps.rs`** asks Fixpt `--machine fx-cells-rsinterp check --reps`
  (through `fixpt::ask`, its answers kept as every other's): a cellular
  machine, since a plain `subr` takes the machine's convention, the
  default's `native`, kayak's checker's `cellular`. Each program is
  `same` (every name's rep the same bytes, or both declined), `differ`,
  `refused` (kayak refuses what Fixpt takes), `both` or `fixpt-refuses`,
  kept in target/kayak-checks/reps/NAME.fixpt; it fails if a program on
  **`tests/reps-passing`** (grows only) is no longer `same`.
- **`checks-summary reps`** counts them (a step of the check), `--list
  KIND` names them, `--bless` grows the list (`--bless-drop` to drop).
- **`kayak-fx26 --reps`** lists names as Fixpt's does (`rep-names-of`):
  a module's (its type as a value, `module-type-of`), a datatype's
  constructors, a generative type's `up-` and `down-`; not kayak's own
  globals of a load (`made--`).
Now: 431 programs same (on the list), 63 differ, 28 refused (the logical
types' 15, and four whose module's type as a value kayak cannot read,
`include-types.fx`, `include-types-inside.fx`, `include-family.fx`,
`import-list-and-family.fx`), 413 refused by both. Most of the 63 are
modules' reps, compared now that a module's name is listed: abstract
types, descriptions, their order.

## C353. U0: a `oneof`'s sums (2026-11-05; logical types, step 1)

The user's "start work on implementing its allof/anyof/oneof type
constructors", after the pin at 1311497 (C351, which read `oneof`).
Fixpt's U0 (ba49c7bf, `check-unions.fx`'s `k-union-sums-merged`): a
`oneof`'s members are told apart at run time, by shape, or, for two sums,
by their tags, none in common. So two sums with no tag in common may be
members, made one sum where the `oneof` is written (`sums-merged`: the
same values, which `tagcase` takes apart), and two sharing a tag are
refused in Fixpt's words, each named as written (`a a and a b both have
`x``). A `sum` given where a `oneof` is expected is checked against its
sum (`sum-form`). Fixpt's three tests (`unions/oneof-sums.fx`,
`oneof-sums-share.fx`, `union-still-read.fx`) now give Fixpt's answers.
Not ported: Fixpt's refusal of a `oneof` of several sums one of which is a
recursive type's own name not yet defined (kayak finds such a member's
shape otherwise, `mu-member-shape`).

## C354. U1: `allof` (2026-11-05; logical types, step 2)

Fixpt's U1 (c3dc2406; `docs/research/logical-types.md` §8.5, rules 1 to
4), in kayak's checker:
- **The type**: `(allof (subr …) …)`, `%allof` of monomorphic procedure
  types of one convention (an `allof` in it its members), printed and
  exported (`allof`) as Fixpt's; KFX's restriction pass refuses it and
  `anyof` (KFX's Rust has no intersections).
- **Subtyping**: below an `allof`, below each member; an `allof` below a
  type, through the first member that is.
- **Rule 1**: a `lambda` where an `allof` is expected is checked once per
  member (`check-allof`), in a scope marked `%allof`.
- **Rule 2**: there, a sum's `tagcase` skips an arm whose tag the
  member's subject lacks (`%arm-skipped`, `%arm-taken`); one skipped under
  every member is refused (`a T has no tag `c``, the last member's).
- **Rule 3**: what each member's check leaves, at each place both left
  facts, the same, else refused in Fixpt's words ("one body cannot be each
  member …"). Compared: every fact but the checker's own (`%`) and those
  only KFX's emitter reads (`allof-emitter-only`: `param-types`,
  `tagcase-sum`), which no `allof` reaches. Each member's facts kept.
- **Rule 4**: a call of an `allof` (`allof-chosen`): its arguments
  synthesized once; every member they fit (`quick-fits?`, a subtype test
  that answers without refusing, as Fixpt's `k-tes-below?`: where it cannot
  tell, no, so a call fits fewer members, never more); the call as the
  first's parameters, its result the meet (`meet`: a type with itself,
  `nat`/`int`, sums by their common tags, products field by field,
  immutable lists, procedures their `allof`), its latent effect the least
  if one is. Refused where none fits, or two results have no meet.
Fixpt's tests: `compiled-apart`, `dead-in-all`, `fits-none`, `no-meet`
refused as Fixpt's (`dead-in-all` names the sum written out, not `ab`:
kayak's types keep no abbreviation's name), `meet` (1 2 3) on the engine
and every tier. Not yet: `swap.fx`, refused for a gap of kayak's own, not
`allof`'s: kayak's termination check does not see recursion through a
recursive sum's payloads (`tagcase`, then `extract`) end, which Fixpt's
does; the same `swap` at one plain type is refused alike (TODO).

## C355. U2: `anyof` (2026-11-05; logical types, step 3)

The user's "Do anyof first" (before the termination gap of C354). Fixpt's
U2 (1d3b33a4) with U3b's sums kept as written (bcc6ecea), in kayak's
checker:
- **One node with `oneof`'s**, `%union`, printed `oneof` where no two
  members share a shape (`members-apart?`), else `anyof`. Its normal form
  (`anyof-trimmed`): flattened, `void` and a member the same as one before
  it (`ty-same?`, structural, as Fixpt's `k-same-type?`: the parse has no
  subtyping) dropped; sums kept as written. Each member's shape known; a
  member that may be written (`may-be-written?`: a cell, an array, a
  bloblet, a pair or list not frozen) told apart from every other by its
  shape, else refused in Fixpt's words, members named as written.
- **A member of a shape** (`member-of-shape`) only where one alone has it,
  as Fixpt's inference; a test keeps every member of the shape it proves
  (`narrowed-to`, `members-of-shape`): their union.
- **Below a union**: of several members of a value's shape (a list's for
  `nil` or a pair), the first it is below (`quick-fits?`, which now sees
  through a recursive type, and takes `nil`, a pair and `false`); a list
  written where several lists are, found then fitted.
- **A union of sums as one sum** where one is needed (`union-as-sum`, U3b):
  `tagcase` takes it apart, and a `sum` given for it is made of it, each tag
  once, a shared tag's payload the union of its payloads.
Fixpt's `anyof/` tests: `oneof-same`, `overlap`, `sub-sum`, `sums-share`
give Fixpt's answers on the engine and tiers 1 and 6; `cells`,
`written-lists`, `shown` refused in Fixpt's words, `mixed-list` refused
(naming `ib` written out). Left: U2b's splits (`distribute*`, `budget*`)
and greatest member (`greatest-pair`), U3's rule 5 (`car-split`,
`allof-split`, `swap-anyof`), and `recursive-sums`, `swap-anyof` on C354's
termination gap.

## C356. U2b: a union's greatest member, and splits (2026-11-05; logical types, step 4)

Fixpt's U2b (96dd3d1f, 7f459806; `logical-types.md` §8.8):
- **The greatest member of a shape** (`greatest-of-shape`): where
  inference takes a union's member of a shape (a pair it is taken apart as,
  `pair-member`; one `cons` builds, `pair-expected`, now also an `anyof`
  of several lists' greatest's pair, `union-pair-expected`; a list written
  where one is), of several the one every other is below (`quick-fits?`,
  now immutable pairs part by part too); none if none is.
- **Splits** (`split-fits`): a value below a union none of whose members
  of its shape takes it whole, if a product, a sum or a frozen pair, is
  split at its first field whose type is a union and on which the
  members of its kind differ (`kin-fields`), each piece then below the
  union; 64 splits a question, as Fixpt's (question 17). A list is not
  split along its tail. Shown so, the value is taken without its
  inference variables solved by it (it has none where a split is tried:
  its fields' unions are written).
Fixpt's tests: `distribute`, `distribute-product`, `budget-within`,
`greatest-pair` give Fixpt's answers; `distribute-missing`, `budget-past`,
`mixed-list` refused (kayak names `ib` written out).

Found on the way, not looked into: `pair-expected` written with a `cond`
clause `((and (string=? (shape-of x) "union") (not (null? (F …)))) (F …))`
made the bootstrap's second generation not build (403 errors in the Rust
it emitted: `refuse` found `Void` where a `Res` is expected, `filter`'s
lambdas mistyped), where the same written as its own `define`
(`union-pair-expected`) builds. TODO.

## C357. U3: rule 5, a call's union argument taken apart (2026-11-05; logical types, step 5)

Fixpt's U3 (a647790c; `logical-types.md` §8.5, rule 5): where a call
cannot take a union argument whole, an argument that is a variable of a
union is narrowed to each member in turn and the call checked again; the
call's type the union of theirs, its effect theirs together, what the
compilers read of it alike under each (as an `allof`'s body, C354, and
refused in Fixpt's words otherwise). kayak cannot check a call, see it
fail and try again (a refusal ends the run), so it decides first
(`split-argument`): an argument that is a local of a union the callee's
parameter (an `allof`'s, each member's) takes no whole but takes each
member of. The test (`qf`) gives each inference variable one type and
keeps to it, unlike `quick-fits?` (C354), so that `car`'s `?t1` cannot be
both an `int` and a `bool`. Else the call goes as before, and its own
error stands (`car-split-misfit.fx`).

With it, `narrowed-from` (a test's other branch) now drops every member of
the shape tested, not only one alone: an `anyof` of lists, split as `nil`
and pairs, had kept its `nil`s past a `null?`.

Fixpt's tests: `car-split` (7 1 0), `allof-split` (5 #t),
`car-split-misfit` refused as Fixpt's. `swap-anyof` and `recursive-sums`
wait on C354's termination gap.

A correction to C356: the `cond` clause I found breaking the bootstrap
did not; nor did this entry's change, which seemed to. Twice
`scripts/regen.sh` failed at generation 2 where the generations built by
hand without `--cache` were at the fixed point; with the memos
(`target/kfx-memos-*`) deleted, it passed. A failed attempt's memos, kept
by the same compiler, were taken for other source (TODO).

## C358. Recursion through a product's fields ends; an `allof` judged at each member (2026-11-05)

C354's gap, after `anyof` (the user's order), and what it held up:
- **A product's field of a part is a part** (`terminate.kfx`'s
  `tracked-of`, `(extract e l)`): strictly smaller, a product being
  immutable and in no region, as Fixpt's termination check has it. With
  the sum arm's binding already a part, `swap` at one plain type, and
  Fixpt's `anyof/recursive-sums.fx`, are seen to end.
- **A group with an `allof`'s type ends if it does at each member's**
  (`terminates?`), as Fixpt's U1 has it.
- **`sum-type`** joins the facts only KFX's emitter reads, which rule 3
  (C354) passes over: an `allof`'s members note a sum built at their own
  types.
- **Rule 5 (C357) counts a result below no expected type as failing
  whole**: Fixpt checks a call against what is expected, so a union
  argument some member takes whole is still taken apart where that
  member's result is not what is expected (`swap` of an `(anyof expr op)`,
  whose `sexp` member takes it whole).
- **A union's sums seen through a recursive member** (`sum-of-member`
  unfolds a `%mu` once), so that `tagcase` takes apart an `(anyof expr
  op)` whose `expr` is recursive.
Fixpt's `allof/swap`, `anyof/swap-anyof`, `anyof/recursive-sums` now give
its answers, on the engine and tiers 1 and 6: of Fixpt's 28 logical-types
programs, 25 as Fixpt's, and three refused as Fixpt refuses them but in
other words (`dead-in-all`, `distribute-missing`, `mixed-list`: kayak's
types keep no abbreviation's name, so it writes `ab`, `ib` out).
Found by the check (`recursive/mu-and-name.fx` hung): rule 5's decision
had been made to try `qf` on a call's argument before knowing it was a
union, and `qf`, with no trail of what it assumes, compared `ilist` with
its `mu` written out unfolding each forever. Now `qf` is tried only of a
union, and both quiet tests are bounded: `qf` past 200 steps, and
`quick-fits?` past 64 unfoldings deep, answer no (so a call fits fewer
members, or splits less, never more).

## C359. A check memo of a group whose items are apart (2026-11-05)

The user's "get to the bottom of that caching issue" (C357's TODO). The
cause: a module's items are checked in the order the checker makes
(`module-order-memo`), mutually recursive ones as one `define-rec` group
(`rec-of`), whose items may be apart in the text, other items between
them (check-exprs.kfx's great group of `synth`, `check` and the rest, with
`narrowed-from` among them). The group's check memo (`check-items-memo`)
was keyed by its items' texts joined, gaps left out, and kept its facts at
offsets from the group's start. An edit to an item between two of the
group's, not of it, left the key the same but moved the group's later
items: the memo was taken, and their facts given where they had been, so
the emitter found none (`nil()` for `nil::<T>()`, `refuse(…)` for `match
refuse(…) {}`), and the Rust did not build. It showed in
`scripts/regen.sh`, whose memo file is kept from run to run for the same
compiler: generation 1 compiled the edited source with the memos the
earlier source made. Not "a failed attempt's memos" (C357's guess), nor a
`cond` clause (C356's).

Reproduced: the same compiler, the earlier source then the new on one memo
file, against no memos, the outputs apart. Fixed: the key holds where each
of the group's items is from its start (`layout`), so such an edit misses
it. Now the same: the memo compile and the plain one. The incremental test
(`tests/incremental.rs`, `define-rec-item`) gained the edit that shows it,
a string put between two of the group's items, and taken away; with the
compiler before the fix, it differs. No fact in a memo file lies before
its form (no negative offset).

## C360. A group's facts kept at each item's offsets (2026-11-05)

The user's "Why aren't you using a hash of the content as the cache key
instead of just spans?" The key was a hash of the content; what C359
mended was a value that depended on where the group's items were (facts
at offsets from its start), with the key made to hold that layout too. So
an edit between two of a group's items made the whole group be checked
again, though nothing in it changed. Now the value does not depend on it:
each fact of a group's memo is kept at its offset from the start of the
item it is in (`fact-code-at`: a 2, the item's place among the group's
`starts-of`, the offset; `code-fact-at` back), a fact in none of them where
it is; the key is the items' texts and the chain again, no layout. As top
forms' and the emitter's memos already were, each item's own.

Checked: the same compiler on the source, then on it with a string put
between two of check-exprs.kfx's great group's items, on one memo file:
the output the same as without memos, and of 3456 memos 2 new (the group's
taken). The incremental test (`define-rec-item`) as C359 left it.

Found on the way, not this change's: a warm memo compile of the compiler
is slower than a plain one (15.4 s against 8.4 s; so too at df07641e,
14.8 s against 8.2 s). TODO.

## C361. Where the compile's time went (2026-11-06)

The user's "investigate where the cost is coming from and how to make it
more efficient", after C360 found a warm memo compile of the compiler
slower than a plain one (15.4 s against 8.4 s). Measured with `perf`, then
phase by phase (a scratch build of the generated Rust with timers around
`compile-units-memo`'s parts and the memo file's reading and writing, not
kept). Two causes, both fixed:
- **`put` indexed its string** (`base.kfx`): it added a string to the
  output a character at a time by `string-ref-i64`, which for a string not
  ASCII counts from the start at each (C14), so a long one cost its length
  squared. The emitter's memos are whole forms' Rust, some long and with a
  comment's `…`: `string_ref_i64` was half of a warm compile. Now one pass
  (`string->list`, then onto the output). Warm 15.4 s to 8.1 s.
- **The restriction pass, 4.3 s of every compile** (`restrict.kfx`, no
  memo): `binders-restricted`, at every list of every form, filtered the
  whole program's types to its file's and camel-cased them, though nearly
  no list binds a type variable; and its caller made `file-types` of all
  the forms once for each form. Now the file's types only where there are
  binders, and `file-types` once. 4.3 s to 0.3 s.
The compiler on itself: plain 8.4 s to 4.1 s, warm 15.4 s to 3.3 s, its
peak memory 6.4 GB to 2.9 GB; the output the same as before, with memos and
without. What is left, phase by phase, in TODO.md; a form's facts found
from the emitter's tree rather than its list was tried and was slower.

## C362. The memo file read and written in runs (2026-11-06)

The user's "Yes please" to C361's next step: the memo file (3.3 MB) was
written a list cell a character (`escape-onto`, then `rev->string`) and
read the same way (`string-chars` of the whole file), 0.41 s and 0.25 s
of a 3.3 s warm compile of the compiler.
- **Chosen:** each memo's text written and read in runs, a `substring`
  from one escape (a newline, a backslash) to the next, the runs joined by
  `string-list-append` (`incr.kfx`: `escaped-runs`, `memos-runs`,
  `memo-lines`, `memo-text`, `char-from`). The file's format is the same.
- **And in kayak's runtime** (`kayak-rt`): `string-ref`, `substring` and
  `string-search` at a character index of a string not ASCII counted from
  its start each time (C14), so scanning such a string by index (as the
  reader now does; the file holds comments' `…` and `§`) cost its length
  squared. Now the last index counted to is kept (`char_byte`'s cursor:
  the string's text's address and length, the character, its byte), and
  the next is counted from it, or from the start if nearer. A string is
  never changed or freed (`leak_string`), so its address names it.
  `substring` no longer counts the whole string for its bounds. FX-26's
  meaning is the same; a test, `counted_from_the_cursor`, compares every
  index, part and search of a mixed string with counting from the start.
- **Not chosen:** `string-search` for the line ends: Fixpt's copies its
  whole string at each call (its performance.md, 2026-10-06), and stage 0
  runs this code too. A different file format (no escapes): the file is
  kayak's own cache, but the escapes cost little once taken in runs.
Memos read 0.25 s to 0.08 s, written 0.41 s to 0.07 s; the compiler on
itself warm 3.3 s to 2.95 s; the output the same with memos and without.

## C363. The compile's allocation cut (2026-11-06)

The user's "start on step 1" (the allocation, before any Salsa-like query
layer): a warm compile of the compiler was 1.2 s of its own work and 1.7 s
of the kernel's, faulting in 2.5 GB the heap never frees (75 M
allocations, 1.6 GB, half of it a list cell a character). Each change
keeps the output the same, with memos and without; measured with the
`heap-stats` build's counts and samples.
- **The reader** (`read.kfx`): a token's and a string's text taken as a
  `substring` of the source (`token-at`; `string-body` in runs between
  escapes) rather than a list of its characters; whitespace skipped
  without a `here` made at each character (`skip-at`). `read-all` and
  `program-read` take the text as a string, every caller changed.
- **A form's text** (`form-texts`): a `substring` from its start to the
  next's, not the source's characters dropped and taken.
- **Fingerprints** (`fnv-from`): a string hashed a character at a time by
  index, not by its list.
- **The restriction pass**: its lists of names, written in place, were
  made again at every symbol of the program (240 MB); now constants.
- **`nset-without`** (`refs-in`, which walks every item's body): it
  flattened, sorted and rebuilt the module's set of names at every
  `lambda`; now it gives the set back unchanged when none of the names
  removed is in it, as a `lambda`'s parameters mostly are not.
- **`with-binds`' clash check**: `mangled-clash` made a list over every
  name in scope for each name bound, and `might-clash?` and `kfx-name?` a
  character list and a literal list each call; now the first clash found
  by a walk, the names indexed, the lists constants.
- **Not chosen**: `mmap`ing the sources (the user's question): they are
  1.7 MB, read in milliseconds; the cost was what the reader made of
  them. The emitter's output a list cell a character (`put`, and a
  module's body copied at each enclosing module, `put-module-around`),
  `mangle`'s character lists, and the heap's arenas initialising each
  chunk whole (a chunk twice the last, so up to half of the last unused
  but faulted in) are left, in TODO.
The compiler on itself: warm 2.96 s to 1.99 s (its memory 2.57 GB to
1.52 GB), plain 4.1 s to 3.3 s (3.2 GB to 2.1 GB).

## C364. Names, arenas, strings, and the sources read once (2026-11-06)

The user's "Go for it", after C363, to the emitter's output as the next
cost. Measured first: the emitter's own `put` was no longer what made
most; `mangle` was (80 MB of character lists), the arenas' unused chunk
space (a third of the memory faulted in), a `malloc` for every string,
and the sources read two or three times a compile.
- **`mangle`** (`names.kfx`): a name's Rust in runs, `substring`s between
  the characters that mangle as others (`mangle-runs`), the name itself
  if it has none; `mangle-chars` gone. `substring-of` a `substring`.
- **The arenas** (`kayak-rt/src/arena.rs`): a chunk past `BLOCK` (4096)
  cells made in blocks, each when first reached; a chunk was made whole,
  twice the last, so up to half of each type's last chunk written and
  never used (570 MB of 1.7 GB on a plain compile of the compiler). One
  more `OnceCell` reached per allocation; 1024 and 65536 cells were
  measured too, and came out the same.
- **Strings** (`str_of_parts`): a string's text copied into chunks of
  1 MB by bumping, as the heap's other data are (a long one, 64 KB or
  more, its own), not a `malloc` each and a `realloc` to shrink it;
  `leak_string`, `string-append` and `substring` make theirs so.
- **The sources read once** (`compile.kfx`): `compile-units-as` read each
  file again for its module's Rust, and `compile-units-memo` read each a
  second and third time, for its forms' texts and its Rust; now each
  file's forms are read once and passed on (`modules-put`,
  `sources-texts`, `memo-modules`).
- **Not done**: a module's body copied for each module around it
  (`put-module-around`, 40 MB): indenting it without the copy wants a
  walk as deep as the body, which Fixpt's stage 0 runs too. The output
  turned to a string at the end (`rev->string`, 40 MB) would want a
  primitive Fixpt has not.
The compiler on itself: warm 1.99 s to 1.34 s (1.52 GB to 0.91 GB),
plain 3.3 s to 2.55 s (2.1 GB to 1.27 GB); the output the same with
memos and without.

## C365. Clashes looked up, numbers and lists written in place (2026-11-06)

The user's "keep going", after C364. A frame-pointer build's profile:
checking is half of a plain compile of the compiler, emitting a seventh;
in the check, `with-binds` a seventh, half of it `mangled-clash`.
- **`mangled-clash` by the index** (`scope-mangled-clash`): for each name
  bound, every name in scope of the same mangled length was mangled and
  compared. Now the names that may mangle as it are made (`unmangled`: at
  each `_`, each mangling that begins there, a `-` and a `_`; a Rust
  keyword's `r#`, `self_`, `-`'s `_minus`), each looked up in the scope,
  and mangled to be sure; if one is bound the scope is walked as before,
  so that the refusal names the same two. A name of more than 6 `_`s
  (64 names or more to look up) is walked as before. Tried on each kind
  of mangling (`->`, a last `-`, `=`, `?`, past 6 `_`s).
- **The runtime's string makers**: `list->string` measured its list, then
  wrote it in place, where a `String` grew a character at a time;
  `i64->string` and `char->string` written on the stack, then kept, where
  `to_string` was a `malloc`; a test, `made_in_place`.
- **Tried and backed out**: the scope's index (`bix`, 180 MB, the most of
  any one type) leaving a scope's newest bindings unindexed and walked,
  put in the tree together past 16 (and 64, 256); and a module's check
  growing the outer scope's index rather than making it again
  (`cv-reindexed`, twice a module). Neither measured above the noise
  (2.36-2.43 s plain): the batches still copy a path of the tree for each
  binding, and the modules' rebuilds were not most of the tree.
The compiler on itself: plain 2.55 s to 2.38 s; warm 1.34 s to 1.36 s
(noise); the output the same with memos and without.

## C366. The second self-hosting restored, on tier 1 (2026-11-06)

The user's direction (docs/direction.md, written this day after the user
said kayak had "forgotten that element of the plot"): kayak exists so that
one KFX source replaces Fixpt's two FX-26 implementations, eventually; its
runtime to be rewritten in Coopt's async style; self-hosting driven, and
first. The second self-hosting (C66, the compiler on kayak's own runtime)
had stopped working, unnoticed, as it was in no check.
- **The IR engine's path** (`kayak-fx26 --run`, C66's): "nothing is named
  rep-nodes", a primitive the compiler has used since its types became
  reps (C301). Left: direction.md §7 retires that engine.
- **The cellular path**, chosen: the compiler, its prelude first as
  Fixpt's stage 0 reads it, compiled to cellular words by `kayak-fx26
  --cells-run` (a 19.7 MB listing, 3 s) and run by `kayak-cells --main
  --compiled`. What stood in its way, each fixed:
  - **Two files' lambdas, one name**: a lambda's word was named for its
    procedure and its offset, which two files can share
    (`map@lambda@24085`), and a copy keyed so (`r-copy-of`) could have been
    shared by them, wrong code. Now each form is compiled knowing its file
    (`parse-tops-filed`, `c-filed`), and outside the program's own file a
    name's place is `FILE:OFFSET` (`filed-place-written`), as Fixpt's
    `cellular.rs` writes it, and a copy's key carries the file
    (`filed-place`). The program's own file is number 0, its names as
    before, so single-file listings are Fixpt's as they were. Differs from
    Fixpt: kayak's file number is by order met and its offset in
    characters, Fixpt's in bytes. A lambda inlined from another file's
    procedure is keyed by the form's file, not its own.
  - **Fixpt's `%fx26-rep-of-nodes` and `%fx26-rep-nodes`** were not tier
    1's: now its heap's nodes made KFX's for kayak-rt's `rep-of-nodes` and
    back (`cprims.kfx`, `rep-of-nodes!`, `rep-nodes!`).
  - **Memory**: the run filled a 32 GB cap (and, uncapped, the machine).
    Tier 1 is KFX, so whatever its primitives make in KFX is kayak-rt's,
    never freed: every `i64` operation made two bignums (`int-wrapped`,
    `(arithmetic-shift 1 64)`; 31 M on one small file), and `list->string`
    appended a character at a time (`chars-text`), a module's output its
    length squared. Now a width's bounds are constants and an in-range
    value is not wrapped, a fixnum's width checked by `i64`s
    (`fixnum-in?`), and `list->string` one pass.
- **Measured**: the compiler on tier 1 makes `boot/src/kfx/` byte for byte,
  97 s, 7.5 GB (the Rust build 2.4 s). Of the 7.5 GB, some 4 GB is
  start-up's, never freed: the listing read (2 GB of characters) and the
  words compiled to x64 (2.2 GB); direction.md §7's runtime would collect
  them.
- **Kept checked**: `scripts/selfhost-cells.sh`, in the full check beside
  the fixpoint (not `--quick`), its memory capped by systemd where it can
  be, so that a run that grows stops rather than starving the machine.
  `kayak-cells` now reports kayak-rt's heap with `heap-stats`, and the
  sampler sees tier 1's frames (`kayak_cells::cells::`).

## C367. The lang items, over the vocabularies kayak has (2026-11-06)

Step 1 of docs/research/coopt-adoption.md §7, the user's approval of the
note ("Everything here sounds right") with its defaults: one table,
`lang/items.sexp`, an s-expression file the build reads (no syntax added
to KFX, which stays a subset of FX-26), of what the compiler, the runtimes
and the standard library name across their boundaries, after rustc's
lang items (`rustc_attr_ir/src/lang_items.rs`: a closed list the compiler
knows, the definitions the libraries', each once).
- **Its rows**, each from where it is defined today
  (`scripts/lang-items.py --update`): Fixpt's runtime primitives and their
  arities (its `PRIMITIVES`, the first of a name, as Fixpt finds them) and
  tier 1's own; Fixpt's immediates (`value.rs`); kayak-rt's syscalls with
  their canonical types; each KFX name `check-defs.kfx` types, as a
  kayak-rt function of its mangled name or the emitter's own; the
  kayak-rt functions the emitter's Rust calls. 1,003 rows.
- **Checked** (`--check`, in the full check where there is a clone): the
  table is what its sources make; tier 1's primitives are rows, of their
  arity; the immediates' three hand-kept copies (`cells/layout.kfx`,
  `fx26/vheap.kfx`, kayak-jit) are the table's; each declared syscall is
  the registry's, declared alike in each of the six files that declare
  one; each typed name's function is kayak-rt's; no kayak-rt function is
  called by the emitter that the table does not list. Two planted drifts
  (an immediate, an arity) were each caught.
- **Found**: Fixpt's `PRIMITIVES` names `%fx26-string-copy` twice
  (`prim.rs:1431`, 3 arguments, and `:2358`, 1 argument); Fixpt finds the
  first, so the second is unreachable. Said by the check as a note, not
  a failure: Fixpt's to fix (to be passed to the Fixpt team). Tier 1
  agrees with the first.
- **A mirror for now**: the copies are checked against the table, not yet
  made from it; the runtime written in step 2 on reads its vocabulary
  from it. The pin's move runs `--update` (PLAN.md).

## C368. Coopt pinned; its base, trace and scheduler taken (2026-11-06)

Step 2 of docs/research/coopt-adoption.md §7, begun, with the note's
answer 2: Coopt's code copied into kayak, marked, from a pinned clone.
- **The pin**: `../kayak-refs/coopt`, cloned from Coopt's GitHub origin
  (as Fixpt's is), at 9283bba, the commit the note was read at;
  `COOPT_REV`. `scripts/fixpt-drift.py` now knows two upstreams (its
  `UPSTREAMS`), and `scripts/coopt-drift.py` runs it for Coopt's
  `coopt-source: PATH@REV` marks; in the full check where the clone is.
- **Taken whole**: `coopt-base` (fixed hashers, the seeded generator,
  fingerprints, stable encoding), `coopt-trace` (the event stream) and
  `coopt-sched` (futures as tasks, fuel, priorities, effects, the virtual
  clock, the inbox, stress), as `kayak-base`, `kayak-trace`,
  `kayak-sched`: renamed, each file marked, nothing else changed. Their
  46 tests pass, as in Coopt; kayak's rustfmt and clippy take them as they
  are. The fingerprint key stays Coopt's ("coopt fp…"), so that the two
  fingerprint alike.
- **Not yet**: the runner (it needs `coopt-sys`, with `libc`, which would be
  kayak's first dependency from outside the workspace: kayak-jit makes its
  system calls without one), and KFX's tasks and
  the compiler's I/O moved onto the scheduler (step 2's rest).

## C369. Coopt's crates are kayak's forks (2026-11-06)

The user, asked whether kayak's syscalls make `libc` unneeded for Coopt's
runner: "I think you should opt to fork coopts crates at will, rather than
treat them as 3rd deps to preserve and maintain."
- **Chosen**: the crates taken from Coopt are kayak's own, changed as
  kayak needs; their `coopt-source:` marks stay, as provenance, and
  `scripts/coopt-drift.py` runs in the full check without `--check`:
  what changed upstream is said, not required (Fixpt's ports keep their
  duty, as FX-26 is Fixpt's to define).
- **`libc`**, with `coopt-sys`: kayak's syscalls are the typed bridge from
  FX-26 code to the host's operations, which says what a program may ask;
  the Rust behind them still reaches the OS, and epoll, kqueue, signalfd,
  termios and the window size take structs whose layout differs by
  platform, which `libc` holds. Kept in the one `unsafe` crate, as Coopt
  keeps it; replaceable, the fork being kayak's.
- **Forked too**: `coopt-sys` and `coopt-runner`, as `kayak-sys` (with
  `#![allow(unsafe_code)]`, as in Coopt, and `libc`) and `kayak-runner`
  (sim, replay and live runners, the log, the worker pool), marked as
  C368's. The five forks' 86 tests pass under nextest (one skipped, as in
  Coopt). Not yet used by kayak-rt: moving FX-26's processes onto the
  scheduler and its I/O onto the runners is next.

## C370. kayak's own allocator, for the binaries (2026-11-07)

From the async-everywhere experiment (branch `experiment/async-everywhere`,
`docs/research/async-everywhere.md` there), where `malloc` and `free` were
38% of the async compiler's run: `kayak-sys`'s `Alloc`, a `GlobalAlloc` of
per-thread free lists by size class (16 to 1024 bytes, by 16), refilled
by bumping through 1 MB chunks from the system, which are never given back;
larger or more aligned blocks the system's. In the one crate where `unsafe`
is allowed. The experiment found it worth having without async too, and so:
- **Chosen**: the `#[global_allocator]` of kfx, kayak-fx26 and kayak-cells.
  The compiler on itself: plain 2.38 s to 1.97 s (1.26 GB to 1.11 GB), warm
  1.36 s to 1.04 s; the output the same. A test of its own
  (`crates/kayak-sys/tests/alloc.rs`: each size and alignment, blocks used
  again, freed on another thread).
- **Not yet**: the accounting per task the experiment proposed (bytes
  charged to the task polling), for when kayak-rt's tasks are the
  scheduler's.

## C371. The emitter's async mode, on main (2026-11-07)

The user, after the async-everywhere experiment (branch
`experiment/async-everywhere`, its `docs/research/async-everywhere.md`):
"this async codegen is promising", "keep going". The experiment's switch,
a constant, made a mode of the one emitter: `kfx --async PROGRAM.unit DIR`
writes every procedure as an `async fn` that returns a `Result`; without it,
the Rust is as before, byte for byte.
- **The mode** is a fact the driver adds (`moded-facts`: `async-mode` at no
  place, -1), read through the emitter's fact index (`async-mode?`), so
  that no structure changed. Not with `--cache` yet: the memos' keys would
  not tell the modes apart.
- **The checker, the same in both modes**: what a program means is not the
  mode's. It now notes, whatever the mode, a primitive's call (`prim-call`,
  where a call's head is found a primitive) and a named procedure that is
  no local used as a value (`proc-value N`, and `prim-ref` for a
  primitive's), which only the async mode reads.
- **The emitter reads facts through `mfact?`**: in the async mode every
  procedure, lambda, handler and prompt body suspends and may abort; a call
  is awaited and `?`'d if its callee is a KFX procedure (`kfx-callee?`: a
  local, a procedure value, or a name no form's keyword, no primitive's,
  no prelude's, no constructor's), and its future boxed only if the callee
  is of the group being written (`boxed-call?`, `%rec`: a definition sees
  only those before it, so only a group's calls close a cycle); a named
  procedure as a value is a closure of its future, a primitive's lifted;
  nothing is `Send + Sync` (the scheduler is one thread's). The written
  types say so too (`suspending-effect?`, `fallible-effect?`,
  `crossing-effect?`).
- **A latent bug fixed**: a procedure type both suspending (or fallible) and
  of an effect a process may have lost its closing ` + Send + Sync)`.
- **Checked** (`scripts/async-check.sh`, in the full check beside the
  self-hosting run, 105 s): the compiler compiled `--async`, built as a
  crate of its own, makes `boot/src/kfx/` in plain mode, as the
  synchronous one does, and its own async Rust again with `--async`.
- **Measured**: the async compiler on itself 7.3-8.5 s (the experiment's
  6.7 s had none of the mode's tests); the synchronous one 2.03 s to
  2.11 s for the facts now noted in both modes and the mode's test at each
  fact read, 4%.
- **Next**: fallibility apart from asynchrony (a preemption is no abort);
  procedures that cannot reach a suspension (leaves, bounded work) kept
  synchronous; fuel charged and the tasks the scheduler's; the corpus in
  both modes.

## C372. The async mode apart from aborting; `and`/`or` tails loop (2026-11-07)

The user's "Go ahead" to C371's next step: in C45's design, to suspend was
to capture to a prompt, which aborts, so the async mode made every
procedure return a `Result` and every call `?`'d. A preemption is no
abort.
- **Fallible only where the effect says**: in the async mode, procedures,
  lambdas and calls are fallible as the checker's facts say (`fallible`,
  `fallible-call`), as in the synchronous mode; written types too
  (`fallible-effect?`). A future of a plain value has a type and a box of
  its own (kayak-rt's `suspend::Fut<R>` and `suspend::boxed`, beside
  `Body<R>` and `body`): a lambda that cannot abort is `boxed(async move
  ..)` of its value; a call's box is `body` or `boxed` as the call may
  abort or not (`call-box`); a named procedure as a value is wrapped as
  its effect says (the checker's new `proc-value-fallible`). Prompt
  bodies stay fallible: an abort to a prompt is what `Result` is for.
  The async compiler's Rust has 461 `?`s, from 18,604; it builds in 68 s
  from 87 s; it runs in 4.76 s from 7.3-8.5 s.
- **`and` and `or` in a looping tail** (both modes): the emitter followed
  a self tail call through `if`, `cond`, `tagcase`, `let`, `begin`, not
  through `and` and `or`, which it wrote as `&&` and `||`, so `any?`,
  `all?` and what is made of them recursed. KFX's `and` and `or` are of
  `bool`s, so in a looping procedure's tail one whose last part makes a
  self tail call is written as the `if`s it is (`as-ifs`): 46 more
  procedures loop (460 to 506). In the async mode each of their steps had
  been a boxed future.
- **Measured** (the compiler on itself): synchronous 2.11 s to 2.03 s (the
  4% C371 cost, made back); async 4.76 s to 3.69 s, 1.8x the synchronous.
  The async compiler's output in plain mode is the synchronous one's, and
  its own async Rust again (`scripts/async-check.sh`).
- **A lang item more**: `boxed`, an `emitter-call` (lang/items.sexp, C367):
  the table's check found the emitter calling a kayak-rt function it did
  not list, as it is meant to.

## C373. The corpus in the async mode, begun (2026-11-07)

The user's "Yes go ahead" to checking the corpus in both modes: the
compiler is the only program the async mode was checked on (C371), and
the corpus exercises much it does not (regions, processes, modules, native
code, generative types).
- **`scripts/async-corpus.py`**: every program of tests/kfx compiled by
  `kfx --async` (now for a single file too: `kfx --async FILE.kfx`) into a
  crate of its own under target/async-corpus, nothing kept; each `main`
  run to its end must give the program's NAME.out, Fixpt's answer. A
  program of another machine's native convention is skipped, as
  scripts/corpus.py skips it. Not yet in the check: it does not pass.
- **Fixed on the way**, each found by it:
  - **What is a KFX procedure's call** is now said by the checker
    (`kfx-call`, where a call's head is a local, a global or an item the
    program binds), not guessed by the emitter by what a name is not: a
    `par` was taken for one, and wrapped in a box nothing closed.
  - **Boxes**: an awaited call that may close a cycle is `Box::pin`'d,
    which needs no `'static` future, so a region's data may be in it; a
    call run to its end (`block`) is boxed not at all.
  - **`par`'s branches** are closures that do not suspend, in the async
    mode too.
  - **`Send + Sync`** where an effect may cross threads, in the async mode
    as in the other (processes are still threads); a named procedure
    wrapped as a value captures nothing, so `lambdaNs`.
  - **A module item as a value** named by its path, as `put-var` names it.
  - **Generative conversions, syscalls' wrappers and module makers'
    functions**, written as plain `fn`s by paths of their own, are `async
    fn`s in the async mode, as every procedure is.
  rustc's errors on the corpus's async Rust: 192, then 63, over some 30
  programs: module makers' parameters (C274: their items are a trait's
  methods, written `fn`, where the async mode wants `async fn`), regions'
  lifetimes in futures, lifted procedures, unions, a few native-code ones.
- **Found by the self-hosting check**: the facts noted at every call
  (`kfx-call`, `prim-call`) overflowed tier 1's stack (its return stack
  65,536 entries; the Rust build's stack is 1 GB): some walk of the facts
  recurses as deep as the list. Now the async mode's own facts are dropped
  before the synchronous mode's emitter (`sync-facts`, a loop), which also
  makes back their cost; the walk that recursed is not yet found, and
  more facts would find it again (TODO).
- The synchronous mode's Rust is unchanged, byte for byte.

## C374. `map` no deeper than 4,096 (2026-11-07)

C373's open question, the walk of the facts that overflowed tier 1's stack,
and the user's "can you rewrite that function to be tail recursive with an
accumulator?"
- **Found**, by a report tier 1 now gives on a stack overflow (cells/
  main.kfx's `overflow-shown`: the words most often on the return stack):
  `map`, 43,626 of its frames, 99.9% of the stack. `kfx/base.kfx`'s `map`
  was `(cons (f x) (map f xs))`, as deep as its list, and the facts are
  mapped over in a dozen places (the `(concat (map ...))` filters).
- **Tried, as asked**: `map` as a loop with an accumulator (`map-onto`) and
  one `reverse`. No overflow; but every map then made its list twice, and
  kayak-rt's heap never frees the first: the compiler on itself 2.07 s to
  2.6 s, 120 MB more.
- **Chosen**: `map` recursive for its first `map-deepest` (4,096) elements,
  which is all of nearly every list it is given, made once; past that, the
  rest by `map-onto`'s loop and one `reverse` (`map-at`). As deep as 4,096
  whatever the list. The compiler on itself 2.14 s (2.07 s before; the
  depth counted at each step), the same memory; on tier 1, with every
  fact the checker notes kept, no overflow, its output `boot/src/kfx`.
- C373's dropping of the async mode's facts before the synchronous
  emitter stays: they are its work, not the other's.

## C375. Module types' procedures async (2026-11-07)

The largest of C373's buckets: a module type is a Rust trait (C24) whose
procedure items were written `fn`, so the async mode's awaited calls of a
module parameter's items (`Mod_M::inc(..).await`) had no future. Rust has
`async fn` in traits (1.75) for static dispatch, which is all KFX's module
parameters are (a type parameter `Mod_M: Steps`, a top-level module given
as its type argument; C170 keeps first-class module values out of the
Rust build, so none is ever `dyn`).
- **Chosen**: in the async mode a module type's procedure item is an
  `async fn` in its trait (`put-val-sig`) and in each module's impl, whose
  body awaits the module's own procedure (`put-impl`); a value item stays
  a plain `fn`. The user asked whether first-class modules (Sheldon and
  Gifford's, FX's) would want `dyn` traits instead: in the Rust build they
  are refused; should they come, a module value as a record of procedure
  values, which the async mode makes futures already.
- The async corpus's rustc errors 64 to 40: module parameters' programs
  build. Left, each an interaction: regions (a boxed future is `'static`),
  C45's lifts of a value already async, native procedures' values,
  constructors reached by `with`, procedure values given to the runtime's
  own functions (`spawn-monitored`), a module's `define-rec`.

## C376. The corpus in the async mode, whole (2026-11-07)

C375 left 40 rustc errors in the async corpus, each an interaction of the
async mode with something older. Each is settled, and the corpus is in the
full check (at the end of `scripts/async-check.sh`): every program of this
machine's, 91 of 96, every procedure async, gives Fixpt's answer.
- **Regions**: a boxed future was `'static` (`Fut`, `Body`); kayak-rt's
  are now generic in a lifetime (`FutAt<'a, R>`, `BodyAt`, `boxed` and
  `body` of any `'a`). A procedure's future type is at the `letrena`
  region its latent effect names; at the innermost region only where the
  checker wrote the effect without its regions (`%local`), and otherwise
  `'static`, since `Fn`'s result is fixed and a `'static` procedure must
  still coerce to one at a region. C46's erasure of a suspending
  procedure's region lifetimes now asks whether it may really suspend
  (`comefrom-effect?`, a `comefrom` in either mode), not whether the mode
  makes it `async`, so region-poly procedures keep their lifetimes. A
  `letrena`'s parameters, which come from outside it, are written at
  their types outside it.
- **C45's lifts**: in the async mode each procedure value is a future's
  already, so a lift supplies only fallibility (`Fut` to `Body`, the
  answer `Ok`), and a lift from one that may abort does nothing.
- **Native procedures** (C116) are synchronous: `readyN`/`readyN_body`
  give their answers as futures ready at once, and a procedure of the async
  mode given to native code is `Native` too, its future run to its end
  (`block`) when native code calls it. `lambda5`/`lambda5s` for a primitive
  of five lifted.
- **Constructors reached by `with`**: a module's constructor is no KFX
  callee (`item-constructor?`): not awaited, and boxed as a ready future
  when it is a value.
- **The runtime's own functions**: `spawn-monitored`'s `on-failure` is
  wanted synchronous, so it is given through `blocking1`.
- **Module items calling each other** without `define-rec`, and a module
  maker's items, are one `%rec` group in the async mode, so their calls
  are boxed; a maker's methods are `async fn` (C274's refusal now of a
  maker's item that may really suspend).
- **Unions' wraps** (C82): the result boxed from the procedure's
  future.
Synchronous output is unchanged: every change asks the mode, and boot's
Rust changes only for the new procedures.

## C377. Leaves synchronous in the async mode (2026-11-07)

The user's go-ahead for the next of C376's steps: procedures that need no
`async` kept plain `fn`s, so that their calls box and poll nothing.
- **Chosen**: the emitter decides, as it decides each call. Each top-level
  procedure (a `define`'s, or a `define-rec`'s, its group `%rec`) is
  written once as a probe, every call of the program's procedures taken as
  synchronous; one whose Rust has no `.await` is a candidate, and the
  leaves are the least set of candidates whose callees, but themselves,
  are leaves. So no recursion but a procedure's own tail calls, which are
  loops. Facts say it to the emitter proper (`leaf`, `leaf-call`,
  `leaf-ref`), added in the async mode only (`leafed`, compile.kfx); in a
  module, the leaves its own definitions hide are left out. `main` is
  never a leaf, as its caller awaits it.
- **Loops in leaves**: with them left out ("leaf and bounded", as C376's
  next step put it), the compiler has almost no leaves, since its loops are
  `define-rec`s. With them, 494 of its procedures are leaves (1,549 async
  left), and the compiler built async compiles itself in 3.43 s, not
  3.65 s (the synchronous build, 2.1 s). The cost: a leaf's loop is no
  point to be preempted at. When fuel is charged, a leaf will charge it
  and yield to none, its caller yielding after it.
- What is left of the overhead is, by a profile, futures' boxes,
  polls and moves (21% of samples) in recursive walks and in the
  higher-order procedures (`any?`, `map`) given procedure values, which no
  leaf rule reaches.
- The probe costs the synchronous compiler a little: `kfx --async` on the
  compiler 4.1 s before, 4.2 s after. It looks for `.await` in the probe's
  characters itself, newest first: `string-search` would be cheaper (4.0
  s), but tier 1, where the compiler runs as FX-26 in the second
  self-hosting, has no such primitive yet.

## C378. Fuel in the async mode (2026-11-07)

The user's go-ahead ("Yeah fuel sounds good"): the async mode's programs
pay fuel, and run as tasks of kayak-sched (Coopt's scheduler, C368), which
the fuel preempts.
- **The charge** is kayak-rt's, not kayak-sched's `Cx::charge`, which
  borrows a `RefCell` and is reached through a `Cx` the generated code does
  not have. `suspend::spent()` decrements a thread-local count; once in
  `QUANTUM` (1,024) units it is spent, and `settle().await` pays the
  quantum, and what was owed, to the scheduler of the task the thread runs
  (`suspend::run`), whose own `Charge` may end the slice: `Pending`, and
  the task polled again in a later slice. In `block`, which cannot yield,
  a spent quantum is only owed, paid at the next charge that may.
- **Where it is charged**: where a closure begins (`boxed`/`body(async
  move { ... })`), at each turn of a loop whose body awaits already, and
  before each boxed call (`{ if spent() { settle().await; } Box::pin(..)
  }.await`). Async Rust boxes every cycle of calls (E0733) and every call
  of a procedure value is a closure's, so a program that runs on passes
  a charge at each round. Not at each `async fn`'s beginning: a call that
  cannot recur need not pay. Not in a loop that awaits nothing: as a
  leaf's (C377), no point to be preempted at, its state kept in registers,
  where a charge's `.await` would keep it in the future. Not in the probe
  for leaves (`charging?`).
- **`suspend::run`** runs `main` as a task of a scheduler of its own
  (quiet: kayak-sched's report of a failing test is not a program's to
  say, a kayak change to the fork), and fails as `block` did if the task
  is still pending when nothing can run. boot's `main.rs` and the async
  corpus use it; in the synchronous build nothing charges, and it polls
  once. A test (`fuel_preempts`) shows a task that charges for long
  ended at a quantum and another run before it ends.
- **The cost**, by instructions (the machine was loaded, so not by
  time): the compiler built async, compiling itself, 21.1 G without fuel,
  23.2 G with (10%): about 1.1 G at boxed calls, 1.0 G at closures and
  loops; 99 million charges, 97,000 quanta. The synchronous build is
  12.1 G. Tried and no better: an awaited `Charge` future at every
  `async fn`'s beginning (23.9 G), the same split into `spent()` and
  `settle()` (23.7 G), a wrapper future that pays before it polls (23.4
  G), loops charged whatever their bodies (23.4 G).
- Next, which would cut both boxes and charges: C376 made a module's
  procedures one `%rec` group, so every call within a module is boxed
  (and now charged); the real groups, the call graph's strongly
  connected components, would box only calls that can recur.

## C379. Recursion groups, minimal (2026-11-07)

C378's cost was half at boxed calls, and those were too many: C376 made a
module's procedures one `%rec` group, since in a module they may call
each other with no `define-rec`, so every call within a module was boxed,
and now charged. The user: the groups should be the written `define-rec`s
where there are some, and otherwise "surely you can compute the minimal
cliques".
- **Chosen**: in the async mode, the groups are the strongly connected
  components of each scope's graph of direct calls (the program's top
  level; each `module` form's body, a maker's included, its `hide`s
  flattened), by Kosaraju's two depth-first passes, as facts (`recs NAME
  ...` at each procedure's name in a group of more than one, or that
  calls itself; `rec-facts`, before the leaves', `leafed`). `put-define`
  makes its own group `%rec`, and nothing else does: not a module's
  whole body (C376), a maker's items (C376), nor a `define-rec`'s written
  group (C371), which may hold members that do not recur. The leaves'
  probe takes the same groups.
- The compiler's boxed calls in the async mode, 3,881 to 1,878. Built
  async, with fuel, it compiles itself in 21.9 G instructions, 14.2 G
  cycles, 3.58 s (C378: 23.2 G and 15.3 G; without fuel then, 21.1 G):
  fuel now costs 4% over C377's build. The synchronous build: 12.1 G,
  about 2.1 s. And `kfx --async` itself, 4.2 s to 3.2 s: the emitter no
  longer looks through a module's every name in the locals of each of
  its procedures.

## C380. Processes as tasks in the async mode (2026-11-07)

The user's go-ahead for the next step after C379: the program's
processes on the scheduler its fuel preempts (C378), as
coopt-adoption.md's step 2 has it, rather than OS threads.
- **Chosen**: in the async mode, a process is a task of the scheduler
  `suspend::run` runs `main` on (kayak-rt's `async_process`; not `tasks`,
  a name the FX-26 front end's own module has, which kayak-rt's glob
  import would make ambiguous). `receive`, `icell-get` and `pselect` are
  futures: not ready, each keeps its task's waker and is pending; `send`
  and `icell-put!` wake every waiter, which looks again, as the threads'
  condition variable does. `par`'s branches are futures of the same task,
  joined (`par2`), not threads. The synchronous mode keeps its threads
  (`process`), unchanged.
- **Where a process's body is async**: its `spawn` closure gives an
  `async move` block, and a `letrena` (a process's body is one, of its
  place) is an `async fn`, awaited, wherever it is written in what is
  `async` (`tasks-here?`); in what is not (a `block`), as before.
- **Processes are spawned pure**: a task preempted by fuel keeps its
  section open, and kayak-sched runs no task whose effect interferes
  meanwhile; with an unknown effect, processes would interleave only at
  their waits. A process shares with the rest only what is transmissible,
  which none writes, and mailboxes and I-cells, which it reaches only at
  its waits.
- **The run**: `suspend::run` steps the scheduler until `main` ends (a
  process that runs on ends with the program, as threads do). A failure
  is caught at each poll of a process: an unmonitored one's fails the
  program, a monitored one's is sent to its watcher. When no task can
  run, a wait for a deadline or for the standard input sleeps until it
  may be ready; with none, `main`'s wait is a deadlock, said as the
  threads' runtime says it.
- Tried by hand beside the corpus (whose `processes`, `monitors` and
  `frozen-shared` now run as tasks): a deadlock on an I-cell is said as
  before; a failing process fails the program, its message said twice as
  in the synchronous mode; a process that loops for ever does not keep
  `main` from ending, fuel preempting it. The compiler, which spawns no
  process, is unchanged (22.0 G instructions).

## C381. I/O through kayak-runner in the async mode (2026-11-07)

The user's go-ahead ("Yeah let's do I/O next"): the rest of
coopt-adoption.md's step 2, the run's I/O a runner's, so that a task
waiting for input or for its output to be taken waits as a task.
- **The runner** is kayak-runner's `Live` (Coopt's): a reactor on the
  standard streams, a worker for an input it cannot wait on, and writes
  queued in order. Two changes to the fork: `Live::cooked()`, for a
  program of its own, leaves a terminal as it is (the REPL's runner puts
  it in raw mode for keys; a KFX program asks for that itself,
  `terminal-raw`) and watches no signal, so that a ^C ends the program
  at once, as before; and `now()`, `drain_writes()`. `suspend::run`
  installs one for the run, polls it before each step, and, when no task
  can run, waits on it until something comes or a deadline (a
  `sleep-until`, a `pselect`'s), or soon if a wait is for what it does
  not see (a program run beside this one, still polled); with nothing to
  come, `main`'s wait is a deadlock. Not kayak-runner's `drive`, whose
  loop has no place for the program's own end, failures and deadlocks.
- **Writes** go to the runner at once, in order, the reactor turned so
  that what can be written is; a task awaits the answer, so a pipe no
  one reads holds up only its writer. One where nothing may wait (in a
  `block`) is not awaited, its answer abandoned (kayak-sched's
  `abandon`). Each is written before the run ends, however it ends, and
  before an `exit`. Writing at once keeps them in order with a failure's
  message, which the panic hook says when it happens.
- **The standard input** is read by a task of its own, through the
  runner, into the very buffer the synchronous mode's reader thread
  fills (`world`), when a read wants more and less than its read-ahead is
  there; so `read-line`, `read-byte`, `read-bytes`, `stdin-ready?` and
  `pselect`'s key arm take from it as they do there, and answer alike.
- **`sleep-until`** waits as a task, for its deadline.
- **The emitter**: in the async mode, where it writes what is `async`, a
  call of `write-out`, `write-err`, `read-line`, `read-byte`,
  `read-bytes` or `sleep-until` (the checker's `prim-call`, its name
  not a local's) is `kayak_rt::async_io::NAME(...).await`.
- **Checked**: `scripts/async-io-programs.py`, now at the end of
  `scripts/async-check.sh`, builds each of `tests/kfx-io` with `kfx
  --async` and compares its output, status and files with the
  synchronous build's (which `io-programs.py` keeps Fixpt's): all 8 alike.
  By hand: a process printing ticks while `main` waits for a line that
  comes late prints them first, as the synchronous build does. The
  compiler built async: unchanged (22.0 G instructions), its compile
  errors said as before.
- **Left as the synchronous mode's**: files, programs run beside this
  one, the terminal's modes; a read of the standard input where nothing
  may wait (a `block`) fails, as a capture there does.

## C382. The rest of I/O through the runner (2026-11-07)

The user: "Yeah do rest of I/O": after C381's standard streams, files,
the terminal's modes and programs run beside this one, in the async mode.
- **Files and the terminal's modes**: what the system is asked (a read,
  a write made atomic, a listing, `lstat`, `readlink`, `stty`) runs as a
  job of the runner's (kayak-runner's `Op::Job`, on its worker pool), the
  task waiting for its completion; the job fills a slot of kayak-rt's
  own and gives `()`, so no stable encoding is needed of what it read.
  The answer, a KFX value, is made back on the program's thread: the
  heap's arenas are a thread's. So each of world.rs's operations is now
  its system part (plain Rust data) and its answer, which the synchronous
  mode composes as before.
- **Programs run beside this one**: each one's reader threads say their
  news (output read, the end found) to the runner's reactor too (a waker
  of `Live`'s, a fork change), so a task waiting for one (`pselect`'s
  arms, `process-exit`, now a task's wait, not a sleeping loop) looks
  again when there is news; the run's wait for them is a second at the
  most, not C381's millisecond polling. A write to one's input, which
  waits for it to read, is a worker's job, the input under a lock of its
  own rather than the table of programs', so that the job holds nothing
  the program's thread needs. Starting one, its output, status, news and
  signals do not wait, and are as before.
- **The emitter**: `read-file`, `write-file`, `list-dir`, `file-info`,
  `read-symlink`, `terminal-raw`, `terminal-restore`, `terminal-size`,
  `process-write` and `process-exit` join C381's list.
- **Checked**: the 8 I/O programs, as the synchronous build makes them
  (`async-io-programs.py`), the processes ones taking the same time (0.20
  s), so news wakes the run; the async corpus; the compiler built async,
  which reads and writes its files so, unchanged (22.0 G instructions).

## C383. KFX's runner library kept (2026-11-07)

coopt-adoption.md's step 2 said `lib/tasks.kfx` and `lib/runner.kfx`
would be retired. Looked at before doing it: `lib/runner.kfx` and
`lib/live.kfx` are the compiler's I/O layer (`kfx/kfx.unit`), which
records a compile and replays it with no I/O, on Fixpt as in Rust (the
fixpoint check's "the compile recorded replays, as Rust and on Fixpt");
kayak-runner is kayak-rt's Rust alone, so the driver on it would run on
Fixpt no more, against the single source (KFX a subset of FX-26). And in
the async mode the two stack already: `lib/live.kfx`'s answers are
`write-out`, `read-file` and the rest, which go through kayak-runner
(C381, C382). Chosen, the user agreeing ("Fine"): both kept, retired only
if FX-26 comes to have runners of its own (Fixpt first); `lib/tasks.kfx`
kept while the FX-26 REPL's loop is built on it. The note's step 2 says
so.

## C384. Queries keyed by what they read; `lib/query.kfx` (2026-11-07)

coopt-adoption.md's step 3, the compiler as queries, its first stage (the
user chose C83's way C, "Go ahead"): the queries in KFX, which run on Fixpt
too, with a Rust implementation of the same names to come under them.
- **What was coarse** (C55): a form's check was keyed by its text and a
  chain of every earlier form's change to the checker, so a change to an
  early interface re-checked every form after it; a form's Rust was keyed
  by a fingerprint of the emitter's whole context, so a definition added
  re-emitted every form.
- **Chosen: keys by name.** A form's check is keyed by its text, the
  bindings of each name it mentions as the checker finds them (its index,
  `bix-at`), the abbreviations of those names, the innermost marker of
  each kind (all the checker reads of them, `scope-bind`), where the
  checker is, and a chain of only what no name reaches: effects named and
  the facts the checker reads again (`place-needs`, `region-within`); and
  the context made before every form but its bindings and abbreviations
  (the datatypes, effects). A form's Rust is keyed by its text, its facts,
  each name's entries in the emitter's context (built once per scope as a
  name index, `emit-reads`), and the scope's locals. A binding's type
  names only types, so names one level deep suffice. Early cutoff is what
  it was: a body edited, its interface the same, changes no other key.
- **Made cheap**: a name's fingerprint is made when a form first names it
  and kept with how many bindings it was of (between modules the checker
  only adds bindings; at a module's start and end the cache starts over);
  the chain moves only when something ambient is added, so that it counts
  no forms. Measured, the compiler on itself, warm with memos: a
  definition added first in `base.kfx`, 4.01 s before, 1.54 s now; its
  type changed, 3.97 s to 1.55 s; a warm compile 1.34 s to 1.53 s, the
  keys' cost.
- **`lib/query.kfx`**, the module `queries` (not `query`, the name of its
  file's Rust module): the fingerprints, a table of names and
  fingerprints, the memo table and its text, out of `incr.kfx`, which
  includes it. The names a Rust implementation can give later, as
  kayak-rt gives the prelude's.
- **Checked**: the incremental verifier (`crates/kayak-checks/tests/
  incremental.rs`), every file, 21 chains, with a new step: an
  abbreviation a form names in its body alone, changed; with the
  abbreviations left out of the keys, it fails, so it is a test of them.

## C385. Keys made walking the form (2026-11-07)

After C384, the user's go-ahead for the cheap speedups first. Measured
first: the memo lookup by list (`memo-find`) is not in a warm compile's
profile; the keys are, a set of each form's names built as a tree
(`form-names`, 7%) and each fingerprint written as decimal text to be
hashed. Neither is needed: the form's text, in the key already, says
which names it has and in which order. So a key walks the form and folds
in each name's fingerprint as a number (`fnv-i64`, in `lib/query.kfx`),
no set made, no text. A warm compile of the compiler with memos, 9.16 G
instructions to 8.77 G; what is left is reading the sources and the
memos' texts. The incremental verifier, every file, as before.

## C386. The memo format, made for reading (2026-11-07)

The user: profile what is left, and if nothing else, the memo format.
Profiled, a warm compile of the compiler with memos (8.77 G instructions
after C385): the memo machinery a quarter of it, by three parts.
- **Item texts by `substring`**: a module's items' texts, for their keys,
  were cut from a list of the module's characters, a character at a time
  (`drop-to`, `take-to`); now `text-between`, a `substring`. 8.77 G to
  8.12 G.
- **The memo file, length-prefixed**: a first line naming the format
  (`kayak memos 3`), then each memo's fingerprint, its text's length in
  characters, and the text as it is: written by joining the parts, read by
  a `substring` each, nothing escaped. Each memo was an escaped line,
  looked at a character at a time, writing and reading. A file of another
  format gives no memos, never a misread one. 8.12 G to 6.85 G.
- **A change's facts in a format of their own**: a memo of a form's check
  was one s-expression, its facts most of it, read by the reader. Now the
  rest is the s-expression, a line, and the facts follow, each a line of
  numbers (its kind, two numbers, its strings' lengths) and its strings as
  they are (`fact-record`, `fact-records`). 6.85 G to 5.90 G.
- So a warm compile of the compiler, 9.16 G instructions before C385 to
  5.90 G; 1.53 s to 1.17 s, below what it was before C384 (1.34 s) with
  C384's finer keys. A definition added first in `base.kfx`, its type
  changed, a body edited: each 1.15 s. The output is boot/'s; the
  incremental verifier, every file, as before.
- What is left, spread out: emitting from memos (the emitter's index,
  each module's text indented again) and checking from memos (22% and 21%
  of samples), the reader on the sources and on the changes' small
  s-expressions (13%), the keys (10%), and lists of characters some
  string helpers make (`words-of`, indenting).

## C387. The compiler's identity in its memo file (2026-11-07)

The user's choice of the next step. C55's note: a memo says what a form's
Rust was made from, not by which compiler, so a memo file kept across a
change to the compiler gives the old compiler's Rust; the driver could not
tell (FX-26 gives a program no way to know its own code), so callers kept
memo files apart (`regen.sh` naming its file by the generated compiler).
- **Chosen: the identity as data.** `kfx/identity.kfx`, generated by
  `scripts/compiler-identity.py`, defines `compiler-identity`: a
  fingerprint (SHA-256, 16 hex digits) of the compiler's sources, the
  files of `kfx/kfx.unit` (identity.kfx itself left out) and
  `kfx/prelude.fx`. The memo file's first line is the format and the
  identity (`kayak memos 3 c8314d8cf8d59fc1`); `memos-of-text` takes the
  memos of a file only if its first line is its own compiler's, and
  `memos-text` writes it. A file of another compiler, or another format,
  gives none, and is written again as this compiler's.
- **Not kayak-rt's Rust**: it runs the compiler and writes nothing of its
  own into what the compiler writes; and the identity is KFX the Fixpt
  run reads too, so Fixpt running the same sources has the same identity
  and takes the Rust compiler's memos (the fixpoint check).
- **Kept current**: `scripts/regen.sh` writes it before its generations;
  the full check says whether it is the sources' (`--check`). Its memo
  files are still named by the generated compiler, so that generations
  in turn keep each its own warm, rather than each taking none.
- Checked: a memo file's first line marked another's gives a cold compile
  (3.66 s, as with none) and is written again as this compiler's; the next,
  warm (1.04 s). The output is boot/'s.

## C388. Code that must not wait, by its effect (2026-11-07)

The user asked whether the type and effect system can ensure that what
waits runs only on another thread or task, so that a routine trying to
stay responsive never calls it directly; and, where it cannot yet, to
reflect the effect into a runtime object that can be inspected ("reps").
- **Already so in FX-26**: waiting is an effect (`(receive r)`, `(await
  r)`, `(io stdin)`, a `comefrom`), a procedure's declared effect bounds
  what it calls, and `spawn` keeps a process's waits out of its spawner's
  effect. A handler of an effect with none of them can only hand work to
  a process and send; the checker refuses it a wait.
  `tests/kfx/responsive.kfx` (Fixpt's answer, 42; in the async corpus
  too) and `tests/kfx-refused/responsive-handler-waits.kfx` show each.
- **kayak's own places that run code to its end** (C376's `block`): a
  constant's value, already only reading globals and allocating in KFX;
  a monitor's `on-failure`, already `pure` (its note said "not checked",
  meaning found: corrected); a native procedure's callback, unrestricted.
  Now KFX refuses a native procedure type whose procedure parameters may
  wait (`waits?`: `receive`, `await`, the standard input, a `comefrom`, an
  effect variable); a sleep, a file, a program run beside this one hold
  the thread a time, as the synchronous mode's do, and are allowed.
  `tests/kfx-stricter/native-callback-waits.kfx`, which Fixpt accepts.
- **`block`** says "a wait where nothing may wait" when what it runs was
  left pending by a wait (`wait_pending`), not "no prompt for the tag",
  which is a capture's. A kayak-rt test makes one.
- **Not done, FX-26's to have first** (the user's rule; and a KFX program
  using a kayak-only primitive would be one Fixpt refuses):
  `docs/requests/fixpt-responsive-effects.md` asks for sleeping as an
  effect apart from reading the clock; evidence that can be inspected,
  the reps the user means, as `effect-narrow`, which tests a procedure's
  evidence against an effect and gives it at that effect if within; and
  bounded effect variables, the same at check time. Evidence exists
  (C307, C308) but is abstract by design, made by `reify-effect` and used
  by `stage` alone.

## C389. A supertype of evidence for syscalls, asked of Fixpt (2026-11-07)

The user asked whether kayak could add a syscall to inspect evidence: it
controls its runtime and front end. Not under the rules, which the user
chose to keep: a syscall's type is polymorphic over regions alone and its
arguments plain data, so none can take `(effectrep e)`, and KFX accepts no
declaration Fixpt refuses. The user's question: whether the reified reps
need a supertype a syscall declaration can name. Yes: one standard type,
`rep`, without a parameter, of which all evidence is a subtype, crossing
as the canonical text evidence already is at run time; inspected, never
made evidence again. `docs/requests/fixpt-responsive-effects.md`'s second
ask is now that, in place of a new `effect-narrow` form; narrowing a
procedure's type stays `stage`'s with evidence, or bounded effect
variables'.

## C390. Asked of Fixpt: an effect `wait` (2026-11-07)

The user, on the request (C388, C389): why not call the effect "wait",
the natural word, as in wait-free algorithms? Better than the `(io
sleep)` the request asked for, which only took sleeping apart from the
clock: one bare atom, `wait`, beside `spin` and `nondet`, which every
operation that may block until something else happens has besides its
own (`receive`, `icell-get`, `pselect`, the standard input's reads,
`sleep-until`, `process-exit`; a blocking write we would count, files
not, Fixpt's call). A responsive procedure is then one of an effect
without `wait`; `clock` no longer goes with `sleep-until`; and an
inspection of evidence asks one thing, whether `wait` is there. The
request's first ask is now that. kayak's own `waits?` (C388) keeps its
list of kinds until FX-26 has the atom.

## C391. `wait` and wait-freedom, in the request (2026-11-07)

The user asked whether the effect system can check wait-freedom, or
whether that conflates notions. Answered, and added to the request at the
user's word: no `wait` is non-blocking; with no `spin` too (FX-26's
termination check, C20) and no mutable memory processes share, a call
ends in a finite number of its own steps whatever others do, which is
wait-freedom, though not a bound on its steps. And `write-out` and
`write-err` should have `wait` as `process-write` would: a write to a pipe
no one reads waits, a task waiting on its runner in the async mode.

## C392. Fixpt's answer: effect markers (2026-11-09)

Fixpt answered the request (C388-C391) with a design note,
`docs/research/effect-markers.md` (Fixpt a95025ad), nothing built yet:
markers derived from an effect's atoms by a fixed table, as Rust's
`Send` from a type's fields (`nonblocking`, `terminating`,
`deterministic`, `wait-free`, the last kayak's sense), in place of an
atom `wait`, which would unlock nothing and have to be kept in step by
hand; the three `io` capabilities that both wait and do not split; masking
a wait leaves `spin` (their finding from our write-up: a private I-cell
awaited after spawning its filler masked to an effect that seemed not to
wait); bounded effect variables with markers as bounds, `(e effect
wait-free)`; and `effect-case`, a test of evidence narrowing `e` in the
branch that passes, in place of `rep`. kayak's answers to their
questions, given to the user to pass on:
- **The standard streams' writes**: honest, waiting. A write that buffers
  without bound trades the wait for unbounded memory; if a `wait-free`
  handler must log, a separate bounded write answering `full`, with a
  flush that waits. kayak needs none now; its async mode awaits each write.
- **`rep`**: not needed beyond this use, which `effect-case` covers.
- **`comefrom`**: no wait, agreed; kayak's restriction on native callbacks
  keeps refusing captures, for its own reason (no capture crosses native
  code's frame).
- **Their open question, `nonblocking` alone**: wanted. kayak's code run
  to its end (native callbacks) loops (`made-frames-x86_64`'s callback has
  `spin`), so `wait-free` would refuse it; `nonblocking` is the marker it
  wants. A private wait there passes it (masked to `spin`) and fails as it
  runs, saying so (C388's `block`): rare, and said plainly, so no residue
  of its own is asked for.
When it lands: the pin moved, kayak's checker and built-in types split as
Fixpt's are, `waits?` (C388) replaced by the table, and kayak-rt's file
operations refusing what is not a regular file, as Fixpt recommends.

## C393. The pin moved to Fixpt 0d933ff (2026-11-11)

The user: "Pull from origin main" (kayak's own origin had nothing new; so
Fixpt's, the pin moving when the user asks). 13114974 to 0d933ff, 118
commits: Fixpt's first answers to the responsive-effects request (masking a
wait leaves `spin`, the `io` kinds split into parts that never wait,
bounded effect variables), procedural macros and hygiene, binding shapes,
`tupleof`, and more. What it took:
- **New refusals ported to kayak's checker**, so that KFX still accepts
  nothing FX-26 refuses:
  - a wait masked away leaves `spin` (`waits-dropped`, `masked-waiting`:
    `letregion`, `letrena`, `letfreeze` and a compound expression's
    masking; not `spawn`'s, whose process's waits take not the spawner's
    time): `processes/wait-masked.fx`;
  - a `plambda` body must be pure but for reading globals, or an
    `rlambda` that may allocate (`plambda-pure`, `plambda-body-pure`: the
    value restriction): `allof/poly-cell.fx`;
  - no variant named for a type form (`type-form-names`): kayak's own
    `fx26/cg.kfx` had `(define-datatype place (place i64 i64))`, now
    `label-at`, which Fixpt had refused, so `fx26.unit` and `cells.unit`
    with it;
  - a `tagcase` arm takes apart by position only a product labelled 1,
    2, …, a `tupleof` (`positional-labels?`): the corpus's `shapes.kfx`
    binds its named product whole and `extract`s its fields, and takes a
    new `tupleof` member apart (its answer, 785224, made the shared one:
    no machine's own).
- **New syntax**: `(tupleof T …)` and `(tuple e …)`, rewritten early
  (`imports-in`) to `(productof (1 T) …)` and `(product (1 e) …)`; Fixpt's
  programs that took named products apart now use them, and the engine
  agrees with Fixpt on them again (`generative/tree-walk.fx`,
  `run/generative.fx`).
- **kayak-rt**: `read-file` refuses a directory, or anything not a regular
  file, which could wait on another program, as Fixpt's
  `read_regular_file` does; asked of the file's metadata before opening,
  as kayak-rt has no `libc` for `O_NONBLOCK`. `tests/kfx-io/read-directory.kfx`,
  Fixpt's words alike.
- **Drift**, each source read: `check.rs`'s generative declarations, the
  only part ported, unchanged; `prim.rs`, `read-file`'s rule carried, the
  dead one-argument `%fx26-string-copy` dropped (our report; tier 1 never
  had it), `%fx26-expand-procedural` (procedural macros' transformers) not
  taken, as kayak's FX-26 front end has no procedural macros; the two
  `cellular.rs`, profiling's bookkeeping alone; the FX-26 compiler's and
  parser's sources (`fx26/cg.kfx`'s), Fixpt's own reorganizing (path
  keys, binding shapes' records, hygiene) and a quoted list made one
  constant, an optimization not taken yet. Every mark at 0d933ff.
- **An `allof` call's argument converted is one value made once**
  (Fixpt d667f281): where a member the call fits takes an argument only
  converted to another convention, every member it fits must take the
  same type there (`allof-made-once`), in Fixpt's words;
  `allof/converted-twice.fx`, which kayak had taken (a K1 breach the full
  check found). The rest of that commit (a member fitting by conversion
  or a reshape, a `lambda` argument of no types) not taken: kayak refuses
  those, a gap, not a breach.
- **A Fixpt bug, worked around in the test only**: at 0d933ff (and its
  head, 9e75efab) Fixpt's evaluator's checker, the one written in FX-26,
  refuses a module's binding shape that `fixpt check` takes, both checkers
  agreeing: "`core` gives other names than were read from the program
  before it was checked" and "the names `core` gives are not known before
  it is checked", or, where a program loads such a file, "the checkers
  disagree" (`module-files/made-buffer.fx`, `made-client.fx`,
  `made-tickets.fx`; `modules/load-moduleof.fx`, `restrict-parameter.fx`,
  `module-parameters-modules.fx`, `load-moduleof-define-type.fx`, which
  state `;; => 15` and so on). The cells test (`shapes_bug`) takes the
  stated value for Fixpt's, as `run-fixpt-programs` does for C223's bug,
  and kayak's engine agrees with it on all four; one that states none is
  Fixpt's refusal. Worth reporting to the Fixpt team.
- Measured: 211 of Fixpt's programs kfx accepts, all agreeing (209), the
  engine on all 211; beyond KFX 299 agree (292); of Fixpt's 528 refused,
  kfx refuses every one; Fixpt accepts kfx.unit, fx26.unit, cells.unit.

## C394. The pin moved to Fixpt b41109e: its fix of the bug C393 found (2026-11-11)

The user: "Pull Fixpt from github; they say theyve addressed the bugs you
found". 0d933ff to b41109e, four commits. Fixpt's b41109e1 is the fix
(bisected there to 97400f83, binding shapes' step 2 in FX-26): its
expander took a module-parameters file's function's shape for the module
it makes. All seven programs C393 named now evaluate in Fixpt as they
state (`modules/load-moduleof.fx` 15, `restrict-parameter.fx` 3,
`module-parameters-modules.fx` 57, `load-moduleof-define-type.fx` 1; the
three `module-files/made-*.fx` a procedure), so the cells test's
`shapes_bug` workaround is gone, and those programs are compared with
Fixpt's own answers again.
- **Drift**: `prim.rs`'s `%provenance-id` and `heap.rs`'s provenances
  (9e75efab, macro provenance on the native-convention machines), not
  taken: kayak's FX-26 front end has no procedural macros, so nothing it
  makes asks for a provenance. Both marks at b41109e (`heap.rs`'s were at
  19b31f1, unchanged until now). The other commits (`hide` at the top
  level, templates' `bloblet-*` forms) touch no source kayak ports; their
  programs come with the sync.

## C395. The pin moved to Fixpt eed406e (2026-11-11)

The user: "Pull Fixpt from github main again". b41109e to eed406e, two
commits, Heist's macro items: a file of module parameters that takes
modules uses its own macros (the FX-26 parser leaves its `define-syntax`es
out of the items it reads), `import-with-syntax` in a module file refused
rather than a panic, a use that binds a keyword, a file of macros alone
imported. Drift: `parser-exps.fx` alone, its `without-syntax` on an input
file's forms, not carried: kayak's port of the FX-26 parser
(`fx26/cg.kfx`) reads no macros, so it has no such path. Its mark at
eed406e; the new programs come with the sync.

## C396. The pin moved to Fixpt 4e13016 (2026-11-12)

The user: "Pull the Fixpt from GitHub main and adapt to it". eed406e to
4e13016, 23 commits, nearly all of them Fixpt's own machinery: a session
on a host with no Scheme engine unless lowered code runs (TODO §252), the
front end native by default and entered from Rust, procedural macros'
transformers compiled, phase times (`FIXPT_PHASE_TIMES`), tests sharded and
faster, and the heap's areas (a static area remembered, code a generation
of its own, globals and symbols in tables of chunks). Two primitives
added, `%gc-major-every!` and `%gc-code-every!`. Drift, each source read,
none carried:
- `check.rs`: a hash set's type only (`IdSet`); its generative
  declarations, what `kfx/reps.kfx` ports, unchanged.
- `prim.rs`: primitives that run code a kind of their own (`Reenters`, in
  place of a list of names), leaves marked as they run, entries called
  from Rust; the two `%gc-…-every!`, tier 1 having none of the
  `%gc-every!` family and no program asking for them.
- The two `cellular.rs` and `x64.rs`: those kinds followed, phases
  entered, assertions that a leaf runs no code, a code collection apart
  from a major one; no change to the words or what a cell does.
- `heap.rs`: its areas and generations (tables, code); the format kayak
  ports (values' tags, bloblets' layout) and the symbols' hash
  (`kayak-rt/src/text.rs`) unchanged.
Every mark at 4e13016.

## C397. FX-26 as Fixpt has it, stage 0: `#%toplevel` (2026-11-12)

The user: "start implementing FX26 to match Fixpt. It's had a lot of new
features. And its runtime got a lot better". Measured at 4e13016 and
planned in `docs/research/fx26-macros.md`: features first (macros, most
of the 151 programs kayak's checker refuses that Fixpt accepts), then
speed through the planned Rust runtime (`coopt-adoption.md` steps 4-6),
the user choosing both; and, the user adding, Twobit-style optimizations
in the one KFX source, which Fixpt's two compilers made hard. Things that
do not seem right are kept for the user in `docs/to-discuss.md`.

Stage 0 is `#%toplevel`, which every expansion uses (Fixpt's `docs/fx26.md`,
"`#%toplevel`, the top level's bindings"):
- **Where the top level begins**: each top-level form is checked with a
  marker, `%toplevel` (`toplevel-marked`, in `check-tops` and the
  incremental `check-tops-memo` alike), saying how long its scope's lists
  of bindings, abbreviations and effects were; what follows from there is
  the top level's (`toplevel-binds`, `toplevel-view`: those lists, no type
  variables). Module members come in as `item`s, and a `with`'s under
  their bare names, so no tag tells them apart: the lengths do. The
  incremental keys leave the marker out and read `#%toplevel:x` as naming
  `x`, so an earlier definition added re-checks only what names it.
- **Values**: `(with #%toplevel n)` and `#%toplevel:n`, in an expression
  and at a call's head (`toplevel-value`, `toplevel-call`): the top
  level's binding put innermost, so every rule a call of `n` has holds;
  where nothing shadows `n`, `toplevel-plain` noted, as Fixpt's checkers
  note it plain. KFX refuses every spelling, values and descriptions
  alike, in the restriction pass (`restrict.kfx`), as kfx's Rust has no
  way to say it (the first full check found a type through it reaching
  Rust).
- **Descriptions**: `#%toplevel:t` as a type, an effect, a description to
  `proj`, a function's head (`(#%toplevel:box int)`, or the type form
  where the top level binds no function `listof`), `(select #%toplevel T)`
  and `(select #%toplevel:m t)`, each read in the top-level view. A
  family's arguments are read there too: right where they name nothing
  local, as in Fixpt's tests; to read them in place when a test needs it.
- **`cg.kfx`**: plain, the plain name; else `#%top:n`, the program's
  global whatever binds `n` (`c-where`, `r-where`), or the standard `n`
  where the program defines none (Fixpt's `c-fx-name`), so that a
  shadowed `(#%toplevel:car l)` is still the in-line `pair-car`. The
  globals' table moved ahead of the parser.
- **Found on the way**: `(listof T)`, its region left out, crashed the
  checker (`car: nil`); it is `(listof T acyclic)`, as Fixpt has had it
  since 2026-11-05 (`listof/acyclic-by-default.fx` now the same). And
  `listof/nested.fx`'s list of lists against a recursive union is refused,
  a gap of its own (TODO).
- Tests: Fixpt's `modules/toplevel-qualified-*.fx`, eight agreeing in
  value and lines (but for kayak naming an abbreviation where Fixpt
  writes what it stands for, which kayak does everywhere), two refused by
  both; kayak's `toplevel/shadowed.fx`, a shadowed global and a shadowed
  standard name, its words Fixpt's exactly.

## C398. kc, a compiler from scratch; its query database (K0) (2026-11-12)

The user, in turn: "think harder about making this as incremental as
possible. I want the system to eg store the dynamic dependence graph,
salsa style"; "think of Kayak as being a compilation service rather than
a batch compiler"; "part of me thinks you should plan to start from
scratch, maybe in the style of Abdulaziz Ghuloum's micro pass compiler
tutorial"; and "your new compiler should start off treating both FX26
and KFX as its source languages. And it should handle transpiling KFX to
async Rust as one backend, and then also compile both FX26 and KFX to
whatever other backends you choose to make (obviously one native one with
our own async and parallel runtime and incremental GC ... maybe wasm)".
Answering `compilation-service.md`'s questions: the engine in KFX, both
builds; the graph persisted.

The plan of record is `docs/research/kc.md`: kc, one compiler in KFX, of
FX-26 and KFX (KFX a mode of one checker), its back ends async Rust (of
KFX), native code on kayak's own runtime (of both), perhaps wasm; many
small passes, each a query per definition; grown in Ghuloum's steps, each
a working compiler checked against Fixpt's answers; the service its long
form. What it keeps (the reader, the checker, the corpus, the lang items,
Coopt's runtime crates) and retires once it covers them (kfx's emitter,
`fx26/cg*`, `cells/`, the IR engine) is there; parity with Fixpt becomes
answers, not listings. `fx26-macros.md`'s order is superseded; its stage
0 stands. Choices kc.md makes, to revisit: native code first as x86-64
assembly through the system's `cc` (this machine has `cc`, `as`, `ld`,
`wasm-ld`), kc in `kc/`.

K0, the engine (`lib/db.kfx`), salsa's design after Coopt's `coopt-query`:
inputs set at a durability (0 a draft to 3 fixed), a new revision when a
value is new; each query run in a frame that records what it reads, in
order, and the least durability of that; asking a node: current if
verified this revision, or if no input of its least durability changed
since (the per-durability revisions, `last`), or if every read, made
current in order, changed no later than it was verified; else run again,
keeping its revision of change if its value's fingerprint is the same
(early cutoff). Cycles refused. Values and keys are text at the engine's
level, which makes the whole database text to keep (`db-text`,
`db-of-text`, a first line saying the format and whose it is), and its
type names no procedure: the bodies are one dispatching procedure the
client gives each ask. Synchronous, one region `@q`, one effect `qe`: the
compiler's passes are pure; generic in both when a client needs it.

Tested by `tests/kfx/db.unit`, in the corpus, so on Fixpt as FX-26 and as
kfx's Rust alike: `first=8/3 again=8/0 same-length=8/1 longer=9/2
read-back=9/0 edited=10/2 another's-keys=0` (a value, then how many
bodies ran: an edit to the same length runs only the file's `len`, early
cutoff sparing `total`; read back from its text, none).

## C399. kc, step 1: integers, end to end, on both back ends (K1) (2026-11-12)

kc's first step, Ghuloum's: a program that is one integer, compiled and
run, by every back end that takes it, its value Fixpt's.
- **kc** is `kc/kc.unit`: the checker it keeps (kfx's files, the
  restriction pass among them), the engine (`lib/query.kfx`,
  `lib/db.kfx`), then its own files: `front.kfx` (the inputs, `text
  PATH`; `checked PATH`, as FX-26 or, for a `.kfx` file, as KFX; `kfx
  PATH`, as KFX whatever the name, which the Rust back end asks), `l0.kfx`
  (step 1's language, `(int N)`, a fixnum), `rust.kfx` (`rust PATH`, a
  module whose `main` gives the value as Fixpt writes it, on kayak-rt),
  `x64.kfx` (`asm PATH`, `kc_entry` giving the value in `%rax` in Fixpt's
  format), `main.kfx` (the driver: `kc [--db DB] [--stats] (--check FILE
  | --rust FILE OUT | --asm FILE OUT)`). Every key a query's name and its
  argument; every body given the database and the one procedure that runs
  any query, so that kc's files need not name each other in a cycle.
- **Built** by kfx into `crates/kayak-kc` (the binary `kc`; `regen.sh
  --all` makes it, as it makes the engine's), and its native code's
  runtime, `crates/kayak-kcrt`, a static library: at step 1 a `main` that
  runs `kc_entry` and writes its value. Linked by the system's `cc`.
- **Its database kept** with `--db`: a second run with nothing changed
  runs no query body; an edit runs only what read it. The database's
  first line says whose it is, `kc` and kc's identity, a fingerprint of
  kc's sources (`kc/identity.kfx`, made by `compiler-identity.py` as
  kfx's is, C387), so that one kc's database is never another's.
- **The harness**, `scripts/kc-steps.py`, in the full check: each program
  of `tests/kc/stepN/` states its value, which must be the pinned Fixpt's
  `eval`; compiled natively, assembled, linked and run; the KFX ones by
  the Rust back end too, all as modules of one crate (`target/kc-rust`).
  Step 1: six programs (0, 42, -7, the fixnums' ends, one `.kfx`), all
  agreeing; a second sweep runs no query body.
- **Reuse checked, not argued** (the user: "How are you checking that
  different inputs that end up yielding equivalent intermediate results
  allow for the remaining work to be punted"). Early cutoff is sound when
  a body reads only through the database, its value's text carries all
  its dependents use, and it is deterministic; a fingerprint's collision
  (2^-64 a pair) is accepted, not guarded. Checked two ways: the database
  says which keys ran (`db-ran`, `kc --stats`), and `kc-steps.py` edits a
  program to text that means the same (`  42   ; ...`) and requires that
  only `checked` and `l0` ran again, `asm` spared, and to another value
  that `asm` ran; and `tests/kfx/db-verify.unit`, 300 random edits (at
  random durabilities) of a graph whose reads depend on what was read and
  whose values often stay the same, each followed by a random query asked
  of the database and of a fresh one given only the inputs: they must
  agree. **It found a bug at once**: 59 of 300 disagreed. An input whose
  durability was lowered (3 to 0, say) was recorded as changed at the new
  durability alone, so what had read it at the old one took the
  durability shortcut and kept a stale value. Now the change counts at the
  higher of the two (salsa records it so). 0 disagree; the database ran
  318 bodies where fresh ones ran 717.
- **Found on the way**: a unit's datatypes are declared ahead across all
  its files, so a library's names meet the compiler's (`lib/db.kfx`'s
  `got` was the reader's; `nth-i64` the terminator's): renamed. KFX's
  `pure` is no effect at all, so a procedure calling another global is
  `kfx` (reading globals): the engine's helpers say so.

## C400. A model of the query database, in Quint, and the bug it found (2026-11-12)

The user: "We may need to make some kind of model of the incremental
computation mechanism"; "I think you should look at Quint. And maybe Kani,
though I'm less convinced of that latter bit for this purpose."
`docs/research/models.md` has it: `models/db.qnt`, the engine's
algorithm abstractly (three levels of queries, reads depending on what is
read, verification in read order, early cutoff, the durability shortcut),
its property that every value is a run from scratch's and every node the
shortcut takes as current is; a fixed instance and an unfixed one, the
engine before the fixes; `scripts/models.py` in the full check, Quint's
simulator on both (the fixed must hold, the unfixed must fail); Quint
0.33.1 pinned in `tools/quint`.

**It found a bug at once** that `db-verify`'s 300 random edits had not:
a node verified by its reads, each made current, kept the least
durability it had when it last ran, though one read's had been lowered
since (an input lowered from 2 to 0, its value the same, `m 1` run again
to the same value, `s` then verified by its reads); a later edit at the
low durability did not touch the shortcut at `s`'s stale one, and `s` was
taken as current, 0 where a run from scratch gives 1. Now a node verified
by its reads takes their least durability as it is (`db-reads-durability`,
`node-reverified`). The counterexample is a fixed case of `db-verify`
(`regression s=1`; the engine before gives `s=0`).

Exhaustive checking set aside, the user agreeing ("this may have been a
distraction"): Apalache, and Quint's TLC backend through it, need Java 17;
a snap's OpenJDK 27 runs them with workarounds, but a first bounded run
took more than ten minutes. Kept for when the engine is asynchronous.

## C401. kc, step 2: the immediates (2026-11-12)

Ghuloum's second step: booleans, characters, unit and the empty list.
- **The core language** (`kc/l0.kfx`, now kc's core, grown by step):
  `(bool 1)`, `(bool 0)`, `(char CODE)`, `(unit)`, `(nil)` (of `nil` and
  `'()`) beside `(int N)`.
- **Native**: each as a word of Fixpt's format (`value.rs`): tag 3, the
  kind in the five bits above it (false 0, true 1, the empty list 2, unit
  3, a character 8), a character's code from bit 8; kayak-kcrt writes them
  as Fixpt's `print.rs` does (`#\space`, `#\x1`, `#\λ`).
- **Rust**: kayak-rt's `Show`, which already writes values as `fixpt
  eval` does, of the value built (`true`, `'\u{3bb}'`, `()`, `nil()`).
- **The reader**, kfx's and so every front end's: Fixpt's character names
  (`fixpt-read`'s `read_char`): case aside; `linefeed`, `nl`, `return`,
  `null`/`nul`, `alarm`, `backspace`, `delete`/`rubout`,
  `escape`/`altmode`/`esc`; and `#\xHEX`, a character by its code. kfx's
  read only `space`, `newline` and `tab`, refusing `#\x3bb`, which Fixpt
  reads.
- Tests: `tests/kc/step2/`, thirteen programs (three KFX), each Fixpt's
  answer on every back end that takes it.

## C402. kc, step 3: unary primitives (2026-11-12)

Ghuloum's third step, as FX-26 has it: `not`, `null?`, `zero?`,
`char->integer`, `integer->char` (Ghuloum's `add1`, `fixnum?` and the
rest are not FX-26's; typed, it has no `integer?`).
- **The core language**: `(prim NAME ARG)`; no binding form yet, so a
  primitive's name is the standard one.
- **Native**: the first code that computes. Each term is compiled to code
  that leaves its value in `%rax` (Ghuloum's `emit-expr`): a constant
  moved in; `not`, `null?`, `zero?` a comparison and `cmov` of `#t`'s and
  `#f`'s words; `char->integer` two instructions (`(c << 8 | 67) >> 5`, its
  tag bits cleared, is the fixnum `c << 3`); `integer->char` a range check
  (a Unicode scalar value: 0 to 0x10ffff, but no surrogate) and the
  immediate built. `kc_entry` keeps the stack aligned for a call, so that a
  failure calls the runtime's `kc_fail`, which says Fixpt's words
  (`evaluation failed: not a Unicode scalar value`) and ends the run.
- **Rust**: kayak-rt's own (`not`, `null_p`, `zero_p`, `char_to_integer`,
  `integer_to_char`). `integer_to_char` now fails in Fixpt's words, and
  for a big integer too, whose low 64 bits it had taken for the code (so
  that `(integer->char (+ 97 (expt 2 64)))` was `#\a`).
- **Failures are answers**: the harness writes Fixpt's `! evaluation
  failed: M` (on its standard error), the native run's, and a Rust
  module's panic (each run under `catch_unwind`) alike as `! M`.
- Tests: `tests/kc/step3/`, sixteen programs (three KFX), three of them
  failures; 35 in all, every one Fixpt's answer on each back end.
- **Found on the way**: the checker refuses `(null? '())`, which Fixpt
  takes (TODO); and kc's names meet the checker's in their one namespace
  (`labelled`): kc's files to be modules (TODO).

## C403. kc, step 4: binary primitives (2026-11-12)

Ghuloum's fourth step, as FX-26 has it: `+`, `-`, `*`, `quotient`,
`remainder`, `modulo`, `max`, `min`, `=`, `<`, `<=`, `>`, `>=`, `char=?`,
`char<?`, `char<=?`, `char>?`, `char>=?`, `eq?` (FX-26's `+` takes two,
not any number).
- **The core language**: `(prim NAME ARG ...)`, of any arity.
- **Native**: a primitive's first argument kept in a slot of the frame,
  `8 * i(%rsp)`, while the second is made, then the two in `%rax` and
  `%rcx`. `kc_entry`'s frame is as many slots as the code uses, rounded so
  that the stack stays aligned for a call. A fixnum is its value times 8,
  so a sum or difference of two is their words' and its overflow the
  64-bit one, and a product needs one shifted back first. Comparisons a
  `cmp` and `cmov`, characters comparing as their words do (the code above
  the same low bits). `quotient` and the rest by `idiv`, a zero divisor
  failing in Fixpt's words (`division by zero`); `modulo` the remainder
  moved to the divisor's sign. A failure's call is now out of the line,
  after the `ret`, a check a conditional jump to it.
- **Integers are fixnums alone, as yet**: a result past one fails in kc's
  words (`kc: not yet: an integer past a fixnum`), until the heap, and big
  integers on it (step 7's).
- **Rust**: a primitive's kayak-rt function is its name as kfx spells it
  (`names.kfx`'s `mangle`: `+` `_plus`, `char<?` `char_lt_p`), as kayak-rt
  names FX-26's standard operations; `eq?`, of immediates, `==`. kayak-rt's
  division by zero now fails in Fixpt's words (it said `quotient: division
  by zero`). The IR engine still says it so (`fx26/vprims.kfx`, kept as
  it is; `tests/repl/plain.out`), as it is to be retired.
- Tests: `tests/kc/step4/`, 37 programs (four KFX), three of them
  failures; 72 in all, every one Fixpt's answer on each back end.

## C404. kc, step 5: `let` (2026-11-12)

Ghuloum's fifth step: local variables.
- **The core language**: `(let ((K NAME E) ...) BODY)` and `(var K NAME)`,
  `l0` resolving each name as it makes the term and numbering every binder
  apart from every other in the program, so that the passes after it
  never meet two binders of one name (the nanopass convention). A name a
  binding form binds is a variable, shadowing a primitive's (`(let ((not
  5)) (+ not 1))`) or `nil`'s. `let` is parallel: each value made where the
  `let` is. `let*` is `let`s nested; a body of more than one expression,
  and `begin`, `(seq E ...)`, each made (a discarded one may still fail).
  A call of a local is not yet (procedures are step 8's).
- **Native**: each variable in a slot of its own, its value moved there
  once made; a use, a load.
- **Rust**: Rust's own `let`s in turn, each named by its number and its
  name as kfx spells it (`v1_y`), which keeps a parallel `let`'s binders
  from seeing each other; `seq` a block of `let _ =`s. kayak-rt's `Int` is
  `Copy`, so a variable used twice is no move; values that are not will
  need it said (when strings and the heap come).
- Tests: `tests/kc/step5/`, 19 programs (three KFX); 91 in all, every one
  Fixpt's answer on each back end.

## C405. kc, step 6: `if` and the derived conditionals (2026-11-12)

Ghuloum's sixth step, as FX-26 has it: a test is a `bool` (no Scheme
truthiness; `(if 1 2 3)` is refused), `if` has its three parts, `and` and
`or` take any number of bools, `cond` must end with `else`.
- **The core language**: `(if T C E)` alone; `l0` makes `and`, `or`,
  `cond` and `case` `if`s: `(and a b ...)` `(if a (and b ...) #f)`, `(or a
  b ...)` `(if a #t (or b ...))`, a `cond` clause an `if` of its body; a
  `case` its key bound once (a binder of its own) and each clause's data
  compared with it, each by its kind's equality (`=` of an integer,
  `char=?` of a character, `eq?` else: kayak-rt's `Int` has no `==`, as
  `eq?` of big integers would be identity).
- **Native**: the test's word compared with `#f`'s, a jump to the else.
- **Rust**: `if T { C } else { E }`.
- Tests: `tests/kc/step6/`, 26 programs (four KFX), among them `and` and
  `or` not making what they need not, a `case`'s key made once, and an
  untaken branch's failure not happening; 117 in all, every one Fixpt's
  answer on each back end.

## C406. kc, step 7: pairs, and the heap (2026-11-12)

Ghuloum's seventh step: the heap, and pairs on it; no collector yet, as
his has none there.
- **The core language**: `cons`, `car`, `cdr`, `pair?` as primitives;
  `(list a b ...)` made `cons`es, its last onto `nil`, its elements made
  in order. `set-car!` and the rest wait for regions (FX-26's frozen data
  refuses them otherwise), `length` and its kin for procedures.
- **Native**, in Fixpt's format: a pair is its cell's address, tag 1, a
  cell of two words, car then cdr. `kc_entry` is given the heap's first
  free word and its end, kept in `%r15` and `%r14` (callee-saved: pushed
  and put back, the frame still keeping the stack aligned); `cons` takes
  two words, a heap used up failing in kc's words (`kc: not yet: the heap
  used up`); `car` and `cdr` check the tag (`car or cdr of a non-pair`,
  Fixpt's words) and load at -1 and 7 from the value.
- **The runtime**: a heap of 64 MiB, given to `kc_entry`; a pair written
  as Fixpt's `print.rs` writes one, its elements, and a tail that is not a
  list after ` . ` (no labels: nothing can make a cycle yet).
- **Rust**: kayak-rt's lists, `heap_cons`, `car`, `cdr` (KFX's `cons`
  makes lists, C197); `pair?` that a list is not empty; `nil` of no type a
  use says only when it is the whole program. kayak-rt's `car` and `cdr` of
  `nil` now fail in Fixpt's words (they said `car: nil`).
- Tests: `tests/kc/step7/`, 27 programs (six KFX); 144 in all, every one
  Fixpt's answer on each back end. **Found on the way**: the checker
  refuses `(cons (list 1) (list 2))`, which Fixpt takes (TODO).

## C407. kc, step 8: procedures, and a query for each form (2026-11-12)

Ghuloum's eighth step: procedures the program defines, and calls of them;
here also a program of many forms, each compiled by queries of its own,
so that kc is incremental below the whole program.
- **Each form a key of its own**: a definition its name, any other form
  `#expr` and how many came before. The front end's queries: `forms PATH`
  (each form's key and datum, written by kc's own writer, `kc-datum`,
  whose characters are by code and strings escaped, so that reading it
  back gives the same datum: the checker's `show-sexp` does not), `keys`,
  `form PATH KEY`, `interface PATH KEY` (`(proc NAME ARITY)`, `(value
  NAME)`, `(expr)`), `scope PATH KEY` (the interfaces before it), `type
  PATH NAME` (as written, or as the checker found it: `checked` now gives
  the types of the definitions written with none). Then `l0-item`,
  `asm-item`, `rust-item` of each form, and `asm` and `rust` of the whole.
  So an edit of one procedure's body runs again its form's, item's and
  code's queries and the whole's, the other forms' spared, the interfaces
  the same so no `scope` again (`kc-steps.py`'s cutoff case); an edit of
  spacing or a comment, `checked` and `forms` alone.
- **The core language**: items `(proc NAME ((K NAME) ...) BODY)`,
  `(global NAME TERM)`, `(expr TERM)`; terms `(gvar NAME)`, a global's
  value, and `(call NAME ARG ...)`, of a procedure the program defines
  (its own name in its scope: it may call itself). A procedure as a value,
  and a call of a value, are step 9's (closures).
- **Native**, kc's own convention: arguments in `%rdi`, `%rsi`, `%rdx`,
  `%rcx`, `%r8`, `%r9` (more, not yet), made into the caller's slots first
  and moved there at the call; the result in `%rax`; each procedure a
  function `kcp_NAME`, its parameters stored in its frame's first slots,
  the frame rounded so that it calls aligned; `%r15` and `%r14` the heap's,
  every procedure sharing them. A global value a cell, `kcg_NAME`, in
  `.bss`; `kc_entry` makes the value definitions and expressions in order,
  each expression's value given to the runtime's `kc_show`, which writes
  it on a line, as `fixpt eval` writes each. Assembly names are a name's
  letters and digits, any other character `_`, its code in hexadecimal,
  `_`, so that every name is one the assembler takes and no two are one.
- **Rust**: a procedure a function, `kcp_NAME`, its types its declared
  `subr`'s (`int` `Int`, `bool`, `char`, `unit` `()`, `(listof T r)`
  `List<T>`); a value definition a thread's cell, set as `main` comes to it
  (made, and failing, where Fixpt makes it), read through `kcg_NAME()`.
  Still synchronous Rust: the async back end (kfx's async mode, C376-C382)
  is to come.
- Tests: `tests/kc/step8/`, 28 programs (six KFX), among them recursion
  (`fact`, `fib`, a sum to 100), six arguments, a global read by a
  procedure, failures in a procedure and in a global's value; 172 in all,
  every one Fixpt's answer on each back end.
- **Found on the way**: KFX's `map` takes no procedure that reads the
  database, so kc asks a query of each key by a procedure of its own
  (`kc-get-all`).

## C408. KFX: two datatypes may share a tag (2026-11-13)

The user, of kc's names meeting the checker's: "Given that FX sums and
unions are structural ... can't you map keywords to unique u64, eg by
SipHashing them, and then manually assign those uid's as discriminants?";
then "Why not retrofit the emitter? You might benefit from this extension
to the KFX language in your own C immediate work."

KFX had refused a module's datatype whose tag another datatype has (C19,
C86: `tests/kfx-stricter/module-tag.kfx`), FX-26 taking it (its tags are
the sum's, found by its type), because kfx's emitter found the enum of a
`tagcase`'s arms by the tag alone (`tag-datatype`). Two things were
tangled there: which enum an arm is of, and what a tag is at run time.
The first is what the restriction needed: the checker now notes, at each
`tagcase` of a datatype, which one (`tagcase-datatype D`), and the
emitter writes the arms' enum from it (a module's own datatype by its bare
name inside the module, `datatype-path-here`), the tag's lookup only
where no fact says (code the checker did not see). The refusal is gone;
`tests/kfx/module-shared-tag.kfx` has the program's `point` and a
module's `place` both tagged `pt`, taken apart inside the module and out
(`7 9 0`, Fixpt's), and the stricter test is no more. Elsewhere the Rust
is as it was, but for three of this machine's programs, whose loaded
module's datatype is now named by the module that defines it.

The second, tags as `u64`s hashed from their names and given to Rust as
explicit discriminants (`#[repr(u64)]` with fields, stable since Rust
1.66; two tags hashing alike in one enum a duplicate discriminant, which
rustc refuses), buys structural sums: one tag one discriminant in every
sum that has it, so widening a sum to a larger is no re-tagging, and a
keyword a plain value. KFX's datatypes do not need it, and it would cost
them: `#[repr(u64)]` gives up Rust's niche layout, so every enum the
compiler uses would grow. So it is kept for where structural sums are,
kc's Rust back end of FX-26's `sumof`s (`docs/research/kc.md`).

## C409. KFX's sums structural: one `Sum`, its tags hashed (2026-11-13)

The user, of C408's keeping hashed discriminants out of KFX: "I think this
choice is going to make it harder for KFX to reflect FX26's structural
subtyping rules"; then "Yep" to: KFX's structural sums (`sumof`) first,
its datatypes after a measurement. And: "we shouldn't ever encounter
SipHash colliding, right?"

**The representation** (`crates/kayak-rt/src/sum.rs`). A sum, of whatever
tags, is kayak-rt's one `Sum`, `&'static SumHead`: a cell of its tag's
discriminant, a fixed hash of the tag's name (`tag_hash`, SipHash-2-4, a
fixed key of kayak's, a `const fn`, checked against the reference vector),
its tag's name (for writing it, `#<sum TAG>`, as Fixpt does), and its
payload's type, then its payload. Not reinterpreted enums (a `&Big` to a
smaller `Small`'s cell would be undefined in Rust whatever the prefix):
the cell is erased, so a sum of fewer tags is a sum of more as it is, a
list of them a list of the more as it is, and a union's member is the sum
as it is. The one unsafe operation, `sum_payload::<T>`, reads the payload
at the type the reader says; what makes it sound is that a checked
program gives each tag of a sum one payload type, and that every cell
carries its payload's type (`fn() -> TypeId`, so a cell may be a `const`
table's), which every read checks: kfx's idea of a primitive's sum and
kayak-rt's are two, which rustc no longer compares, so a disagreement
fails loudly, never reads at the wrong type.

**Collisions** (the user's question): for n tags, about n²/2^65, some
10^-12 for ten thousand; but not left to the odds, since an erased sum
has no enum for rustc to refuse a duplicate discriminant in. kfx writes,
in each program, `const _: () = kayak_rt::tags_apart_from_runtime(&[...])`
of the program's tags, which rustc evaluates as it builds: a collision,
among them or with kayak-rt's own (`RUNTIME_TAGS`, themselves asserted
apart), fails the build.

**The emitter**: a `sumof` type is `kayak_rt::Sum`; `(sum tag e)` is
`sum_new::<P>(const { tag_hash("tag") }, "tag", e)`, `P` the tag's payload
type in the sum the checker found expected (`sum-type`), or Rust's
inference where it found none; a `tagcase` of a sum reads the tag once and
is an `if` on each arm's tag's discriminant, the payload read at the type
the scrutinee's sum gives that tag (a new fact, `tagcase-sum-type`), or
taken apart as a product; `else` binds the sum as it is; no arm, no
`else`, `unreachable!()`. No sum is declared any more, only products.

**What it lifts**: KFX now takes FX-26's width subtyping of sums, a sum
whose type nothing says, and a `tagcase`'s `else` seen at the tags no arm
names (the three C183 refusals): `tests/kfx-stricter/sum-width.kfx` and
`sum-unknown.kfx` gone, `tests/kfx/structural-sums.kfx` instead (a narrower
sum, and a list of them, where the wider is wanted; `3 39 20 -1 0`,
Fixpt's). Of Fixpt's programs, kfx now takes 219 (212 before), every one
agreeing.

**What the erasure cost, and the fixes**:
- kayak-rt's own sums (the I/O primitives' answers, `rep-nodes`'
  operands) are `Sum`s, made by `sum_of` and its static cells (the bytes'
  and counts' tables `SumOf::new` in a `const`), taken apart by name.
- rustc had inferred a poly procedure's type arguments from a sum's enum's;
  a `Sum` says none. The checker now notes a `proj`'s type binders'
  arguments (`proj-type-args`; not its regions, effects, functions, sizes,
  conventions) at the procedure's name, and the emitter writes them as a
  turbofish after it, wherever it writes the name (plainly, or in the async
  mode's adapter).
- A generative type whose parameter only its sum's payloads used would use
  none in Rust: each now holds its parameters in a `PhantomData`.
- **The bootstrap**: `boot/`'s compiler was written with the enums, which
  the new kayak-rt has not; built three steps: the seed and its first
  generation on the committed kayak-rt (the new emitter, written the old
  way), then the second generation, written the new way, on the new
  kayak-rt, the fixed point after it.

**Measured**, the compiler compiling itself, best of three: 2.31 s and
1.215 GB at most, from 2.27 s and 1.206 GB (2% and 0.7%, the new source
larger too). The compiler's own data are datatypes, not structural sums,
so this says little of step 2 (KFX's datatypes as structural sums too, as
FX-26's `define-data` makes them), which waits on a measurement of its own.


## C410. KFX's datatypes stay nominal, after a measurement (2026-11-13)

C409's step 2: KFX's `define-datatype` expanded as FX-26's is (Fixpt's
`parser.fx`, `expand-datatype`), a `sumof` of products and a procedure for
each constructor, so that a datatype's subtyping would be a sum's. Built
to the compiler's fixed point, then measured, the compiler compiling
itself: 10.6 s and 7.5 GB at most, from 2.31 s and 1.2 GB; and kfx
compiling `cells/cells.unit` was killed at 15 GB.

**Why.** The checker's types are trees, copied by every walk (`zonk`,
`replace-var`, `show-ty`, `parse-type`): a datatype named is its whole
body, every datatype in it written out too, so that a type's size grows
with how deep its datatypes nest. Fixpt's checker keeps each type once,
in an arena (`TyId`, `ast.rs`), and so is not bothered. Two narrower
fixes (a recursive type compared with itself not unfolded; the noted
pair of a recursive comparison keyed by the abbreviation's name) did not
help: the cost is in every walk.

**The choice**, of three put to the user (a datatype kept by its name and
unfolded where its shape is read; the checker's types shared, as Fixpt's;
or KFX's datatypes nominal, as they were): the user, "3; it will probably
be a better match for Rust code gen anyway." KFX stays within FX-26: a
nominal datatype is refused wherever a structural one would be, and more.
`sumof` is structural (C409) for a program that wants a sum's subtyping.

The work is kept, uncommitted, as `git stash` "C410 WIP" (the expansion;
the checker writing a recursive type by its abbreviation's name rather
than as `%mu NAME`, a symbol the reader cannot read back; the emitter
finding a named sum's tags; the tags' `const` assertion hashing each tag
once, which rustc otherwise gave up on at some 170 tags).

## C411. kc's files as modules (2026-11-13)

The user: "Do you think we should move kc's files into modules first?",
then "Go for it". kc's files shared one namespace with the checker's
(`kc.unit` lists both), so that a name of kc's met one of the checker's
(`got`, `labelled`, `but-last`: renamed one by one, C401 to C407).

Each of kc's files is now a module, as `restrict.kfx` is (C92): `kc-front`,
`kc-l0`, `kc-rust`, `kc-x64`, `kc-driver`. What no other file uses is in a
`hide`; what one does, the file gives, and its user imports it by name in
a `hide` of its own (`(import kc-front arg-first ... (type qbody))`,
FX-26's `import`, C119), so that each file says what it takes of which.
A module may hide several runs of definitions between those it gives, so
each file keeps its order. `main` stays at the top level, Rust's entry,
calling the driver's `kc-main`; `kc-identity`, which
`compiler-identity.py` writes, too.

The names kc prefixed to keep apart from the checker's (`x64-`, `kc-`)
are left as they are: a rename would be churn, and a new name of kc's may
now be any name. 172 programs as before (`kc-steps.py`), every one by
native code, 30 by Rust.

## C412. kc, step 9: closures (2026-11-13)

The user: "Go for it" (kc's files as modules, C411, then the steps).

**L0** (`kc/l0.kfx`): `(lambda ID (PARAM ...) (FREE ...) BODY)`, its
parameters numbered after its own number, each `(K NAME)` or, where the
type is written, `(K NAME TYPE)`; its free variables found as the term is
made: the variables its body uses, a lambda inside it contributing its own
free variables, less the lambda's parameters and the `let`s inside it.
Binders are numbered apart in a form (C404), so a number names one binder
and no renaming is needed. `(app F ARG ...)`: a call of a procedure as a
value, a local's, a global value's, any expression's (`((car fs) x)`).
`(gproc NAME ARITY)`: a procedure the program defines, as a value (its
arity from the scope's interfaces, for the Rust back end).

**Native** (`kc/x64.kfx`): a closure is Fixpt's bloblet (`fixpt-heap`'s
`layout.rs`, `closure`; `heap.rs`'s `make_closure`), laid out from its
header up: the free values, the last first; the code; the heap's stub
(none yet, 0); a trailer, the words back to the header; the value its
suffix's address, tag 4. So field `k` is at `-4 - 8k` from the value,
whatever the closure's size: the code at -28, free value `i` at
`-36 - 8i`, one load each, as Fixpt meant. The header is Fixpt's (kind 8,
`F` the fields with the trailer, `B` 0). Two departures, each for a later
step: the code word is the function's address (aligned to 16, so that it
reads as a fixnum to a collector), not a code bloblet; and the stub is 0.
A lambda is a function of its own (`kcl_FORM_ID`), placed out of the line
with the failures' calls; entered with its arguments in the six registers
and its closure in `%r10`, it stores both its parameters and its free
values in its frame's slots, so that the body sees variables only in
slots, as before. A call of a value: the procedure into a slot, the
arguments after it, moved to their registers, then `call *-28(%r10)`. A
procedure the program defines, as a value, is a closure over nothing of
its `kcp_` function, made where it is used. The checker's types make
every called value a closure of the right arity, so no check is made.

**Rust** (`kc/rust.kfx`): as kfx's emitter does (`put-lambda`), kayak-rt's
`lambdaN(move |x: T| BODY)`, a `&'static dyn Fn` on its heap, what it
captures `Copy`; its parameters typed where written, else rustc infers
them; a procedure's type `&'static dyn Fn(A, ...) -> R`; a call
`(F)(ARG, ...)`; a defined procedure as a value `lambdaN(kcp_NAME)`.

**The runtime** writes a closure `#<procedure>`: Fixpt writes one by its
machine's name for it (`#<cellular-closure 2 fields 0 bytes>` on
fx-cells-rsinterp), so no step's program states one as its value.

`tests/kc/step9/`: 26 programs (17 FX-26, 9 KFX), each of Fixpt's answer:
a closure called at once, captured values, one returned from a procedure,
curried three deep, a procedure given one and one given as one, closures
in a list, a closure's failure, six parameters. 198 programs in all, each
by native code, 39 by Rust; none wrong.

## C413. kc, step 10: tail calls (2026-11-13)

The user: "Go for it" (step 10).

**Native** (`kc/x64.kfx`, `x64-emit-tail`): a procedure's and a lambda's
body is made in tail position: an `if`'s branches, a `let`'s body, a
`seq`'s last term, each in turn. A call there, of a procedure the program
defines or of a value, makes its arguments as any call does, moves them to
their registers (and a value's closure to `%r10`), gives the frame back
and jumps to the callee (`jmp kcp_NAME`, `jmp *-28(%r10)`), which returns
to this function's caller. Since kc's calls take their arguments in the
six registers alone (more is `not yet`), nothing of the caller's frame is
read after it is given back, and no argument is written over the
caller's: Ghuloum's shuffle is not needed yet. Any other term in tail
position returns its value (`addq`, `ret`), so an `if` in tail position
needs no join. A function's frame size is known only once its code is
made, and each return and tail call gives it back before then: the frame
is a local symbol (`.L…_frame`), `.set` after the code, which the
assembler resolves (checked: `subq $sym, %rsp` with `sym` set later is
assembled with the size).

**Rust**: nothing yet. Rust guarantees no tail calls (`become` is not
stable), so the KFX programs of this step loop a thousand times, not a
million; a trampoline, or the async back end's own loop, is for later.

`tests/kc/step10/`: 12 programs of Fixpt's answers: loops of a million in
an `if`, a `let`, a `begin`, a `cond`; a tail call to another procedure;
through a closure (a lambda applied in tail position that tail-calls the
procedure); with a closure as an argument; with six arguments; one
recursion not in tail position, of ten thousand. Without tail calls each
loop of a million overflows the stack. 210 programs in all, none wrong.

## C414. kc against Fixpt, edit by edit; the forms as inputs (2026-11-13)

The user: "once kc can handle something approaching a real program, I will
want us to start benchmarking it relative to Fixpt, especially when it
comes to responsiveness to incremental changes to code. Actually maybe we
should make such evaluations part of our workflow now?"; then "make sure
that we cover edits that introduce errors and then undo them, as well as
introducing an error followed by fixing forward (in a way where the final
code is still different from the original) ... I want to check that we
are leveraging incremental compilation cache in smart way, where we don't
throw things away too eagerly".

**The bench** (`scripts/kc-bench.py`): a program of N procedures put
through a session of edits: cold; the same again; a comment added; one
body edited; that body made a type error; undone; the error again; fixed
forward to a third body; a procedure added and called. Each step timed
end to end, as one at the keyboard meets it: kc's compile (`--db`), then
`cc` and the run; Fixpt's `fixpt eval` on its fastest machine. And for
kc, the query bodies that ran, against what the edit needs at least: a
body beyond that is work thrown away. With `--record`, the rows go to
`bench/kc.tsv` with the commit; a step's numbers are for discussion, not
a gate (timings are noisy). To be run at each of kc's steps.

**What it found, at once**: one body edited of a thousand ran `form` a
thousand times and `keys` again, 15.8 s against Fixpt's 0.67 s; cold,
21 s. Every `form PATH KEY` read `forms PATH`, the whole program as text,
and so was re-run at every edit, each re-reading all of it (a value is
text): quadratic. `scope PATH KEY`, each form's interfaces of every form
before it, was quadratic too when cold. The database's early cutoff
stopped each re-run, so nothing beyond them ran: the waste was in the
re-runs themselves.

**The fix** (`kc/front.kfx`): what is syntax alone is inputs, set by the
driver from one reading of the text (`kc-load!`): each form's `form PATH
KEY` and `position PATH KEY`, and `keys PATH`; a form no longer there is
set `()`, its position -1. An input set to what it was is no change
(`db-set!`), so an edit of one form changes one input. A form's scope is
no query: `l0-item` asks, of each name its form mentions, `interface PATH
NAME` (a name the program does not define is `(expr)`: `form` of a key no
form has is `()`), and `position` of those defined, so that what an item
depends on is the names it mentions. The salsa way, a tracked struct for
each item, in an engine whose values are text. `forms`, `scope` and the
`keys` query are gone. kc-steps' cutoff checks now ask less: an edit of
one body of two runs `asm`, `asm-item`, `checked`, `interface`,
`l0-item`, each once.

**Measured** (best of 3, kc's compile alone, then Fixpt's whole `eval`):

| N | step | kc | Fixpt | bodies | beyond the need |
|---|---|---|---|---|---|
| 1000 | cold | 0.66 s | 0.67 s | 3026 | |
| 1000 | same | 0.17 s | 0.67 s | 0 | none |
| 1000 | body | 0.36 s | 0.67 s | 5 | none |
| 1000 | error | 0.14 s | 0.34 s | | |
| 1000 | undo | 0.17 s | 0.67 s | 0 | none |
| 1000 | fix-forward | 0.37 s | 0.67 s | 5 | none |
| 5000 | body | 2.54 s | 4.85 s | 5 | none |
| 5000 | same | 0.92 s | 4.81 s | 0 | none |

(`cc` and the run add 0.15-0.25 s to kc's.) An error ends kc's run before
the database is written back, so an undo finds the database as it was
before the error, and runs nothing; a fix forward runs the edited form's
queries alone. Nothing is thrown away eagerly, as the session measures it.

**What is left, both whole-program**: `checked`, the checker over the
whole text at every edit (at 5000 procedures, 1.3 s of the 2.5 s; the
comment's edit runs it alone), the next to make incremental; and the
database read and written whole at every run (0.9 s of `same` at 5000),
which `kayak serve`, keeping it in memory, would not do. And an error is
not yet a value in the database: a run that fails keeps none of its
work, so the error itself is found again in full at each run.

## C415. kc's two whole-program costs: the check memoized, refusals as values, `kc serve` (2026-11-13)

The user: "Okay. Address those two costs next" (C414's: `checked`, the
whole program checked at every edit; and the database read and written
whole at every run).

**The check, memoized** (`kc/front.kfx`'s `checked`). kfx has checked
with memos since C55/C384 (`--cache`): each top-level form's check a memo,
its change to the checker's state, addressed by a fingerprint of the
form's text, of the bindings of the names it reads, and of the chain of
forms before it, so that a form whose memo is found is not checked again,
its change replayed. kc now checks so too, the memos kept in a side store
of the database (`lib/db.kfx`'s `db-side`, `db-side-put!`: text under a
name, no query's input or value, no revision, written and read with the
nodes). Content-addressed, the memos may spare work and change no value,
so no query depends on them. Checked under gdb at 5000 procedures, one
body edited: one form checked, 5001 replayed.

So that kc may have them without kfx's emitter, the checker's memos are a
file of their own, `kfx/incr-check.kfx` (module `incremental-check`), the
emitter's memos left in `incr.kfx`, which includes it (so `with
incremental` finds both, as before). Its one use of the emitter's,
`concat-strings`, is `string-list-append`, the same.

**Refusals as values.** A refusal is the checker's `error`, a panic: the
run ended there, its work lost, and a server would end with it. FX-26 has
`attempt` (Fixpt b41a7fb, Heist's F1): `(attempt e on-failure)`, `e`'s
value, or `on-failure` of a failure's message; `e` must be cancellable.
kayak's checker took it (C297) but KFX did not: now it does (C415), kfx's
emitter writing `kayak_rt::attempt(|| { E }, |m: Str| { BODY })`, a
`catch_unwind`, its handler, a `lambda` of the message, a closure in place;
in an `async` function (the async mode, C371),
`kayak_rt::attempt_async(async move { E }, |m: Str| async move { BODY }).await`,
the body polled inside `catch_unwind`, each poll; a handler that is no
`lambda` is called as a procedure of the message (not yet in the async
mode); the panic hook keeps quiet about a failure an `attempt` catches
(`tests/kfx/attempt.kfx`: `42 7 failed: a negative number`, Fixpt's). `attempt-within`, whose refusal had been `attempt`'s, is
refused on its own (KFX's `attempt` has no timeout yet). Of Fixpt's
programs, kfx takes 226 now (219 at C409), every one agreeing.
KFX ⊂ FX-26 is kept: the form is FX-26's, the checker's rules for it
Fixpt's. `checked` attempts the check (the `kfx` effect writes nothing)
and is `(ok TYPES)` or `(refused MESSAGE)`; the driver asks it first and,
refused, says the message and ends with status 1, its database written as
any run's is. `qe`, the queries' effect, has `nondet` now, `attempt`'s:
what a failure says is the machine's, and a database is one compiler's.

**`kc serve`** (`kc/main.kfx`): requests a line each on the standard
input (`check FILE`, `asm FILE OUT`, `rust FILE OUT`, `save`, `quit`),
answered a line each (`ok N`, `refused N MESSAGE`, `error MESSAGE`; with
--stats the bodies' names after a tab). The database stays in memory, so
a request pays for its edit, not for 23 MB read and written whole (the
memos made it larger) at 5000 procedures. The first form of the
compilation service of `compilation-service.md`. A `not yet` of kc's own
is still an `error` that ends it (kc's limits, not the program's).

**What the bench found on the way**: with a refusal a value, a check's
value went `ok`, `refused`, `ok` across an error and its undoing, and
every `l0-item`, which read `checked` (to be of a checked program), ran
again: 1000 of them. Nothing of `checked`'s value is used there (the types
are `type`'s, by name), and the driver asks `checked` first, so `l0-item`
reads it no more. The bench's needs for the error steps: `checked`; for
the undo, `checked`, and the edited form's `interface` and `l0-item`
(made again, as the engine keeps a query's last value, not each before it,
and found as they were, so nothing after them runs). Every step's bodies
are now exactly its need.

**Measured** (`bench/kc.tsv`, best of 3; kc's compile, `serve`'s, Fixpt's
whole `eval`; the link and run 0.15-0.25 s more for kc):

| N | step | kc | serve | Fixpt |
|---|---|---|---|---|
| 1000 | same | 0.16 s | 0.04 s | 0.66 s |
| 1000 | body | 0.31 s | 0.17 s | 0.66 s |
| 1000 | error | 0.20 s | 0.08 s | 0.33 s |
| 1000 | undo | 0.25 s | 0.12 s | 0.66 s |
| 5000 | same | 0.90 s | 0.25 s | 4.77 s |
| 5000 | body | 1.79 s | 1.09 s | 4.77 s |
| 5000 | error | 1.12 s | 0.45 s | 1.91 s |
| 5000 | undo | 1.50 s | 0.83 s | 4.78 s |

(At 5000 before: body 2.54 s, same 0.92 s.) What is still of the whole
program, in serve: the memoized check walks every form, a memo replayed
each (about 0.12 ms a form: 0.6 s at 5000); and the driver reads the file
whole, each form's input set (most to what it was). A form's check as a
query of its own, keyed by the interfaces it reads, would make an edit's
check the edited form's alone.

## C416. kc's items depend on the names they look up, and no others (2026-11-13)

The user, of the database's file: "Yeah let's talk about the database file
format"; then, of the plan (kc's dependencies first, then keys as data, datum
labels, a graph file apart from its payloads): "Making keys structured data is
totally up my alley".

Measured first (5000 procedures, the database 22.9 MB): keys and dependencies
were 10 MB of it, and an `l0-item` read the interfaces of every symbol its
form has (C414's `kc-scope`): its parameters, `lambda`, `define`, the type
words `int`, `subr`, `pure`, and some twice: twelve dependencies of p2500's
item, which needs four.

Now `l0-of` asks for what it looks up as it comes to it, and for nothing
else (`l0-global-kind`): a name its scope does not say yet is `kc-need NAME`,
an `error`, which `l0-item` catches (FX-26's `attempt`, C415: the making of
an item writes nothing) and runs it again with that name's entry from the
database (`kc-scope-entry`: its interface, and its position if it is
defined; `(none NAME)` if it is none before the form). A form of k global
names is made k+1 times, each small; its dependencies are the same code
path's as its making, so no second walker of the core language can drift
from the first. A procedure's own interface is in its scope from the first.
p2500's item now depends on its form, its position, its interface and `*`
(a program may define `*`). The kc-steps sweep runs 2241 bodies, from 3036.
The database at 5000 procedures: 19.6 MB; the bench's steps a little faster
(`same` 0.84 s, from 0.90; in serve, as they were).

## C417. The database's keys as data, its files a graph and a payload (2026-11-13)

The user, of the database's file: "surely this could be datum formatted,
using the #n= and #n# notation to handle sharing? ... Separating the graph
file from the payload file sounds like a good idea", "It's fine for *this*
to be specific to Kayak", "Making keys structured data is totally up my
alley"; of non-ASCII text, "What kc / KFX source is non ascii?" (none of
kc's; the payload holds the user's program's text, and some of the
checker's messages have `≤` or `…`); "Should we design the format to allow
for faster lookups? Eg include before each sexp the number of bytes one
should skip to jump to next sexp?", then "Go ahead"; and, of what is made:
"It would be nice if you detected that the transpiled rust ends up
unchanged and thus rustc invocations can be avoided".

**Keys as data** (`lib/db.kfx`): a key is a datum, a query's name and its
arguments, `(l0-item "PATH" "p2500")` (`key1`, `key2`; `key-query`,
`key-arg`), interned the first time it is met by its text (`key-text`) as
a number: the nodes by number (an AVL tree, `imap`), what a node read a
list of numbers, a node its key. kc's queries take their key's arguments,
`(d run PATH KEY)`; the runner dispatches on `key-query`.

**The files**: the graph, `DB`, three parts: a header `(kayak-db 2 WHOSE
REVISION (LAST ...) PAYLOAD-LENGTH)`; `(keys (LEN #N=(QUERY ARG ...)) ...)`,
each key labelled by its number, each text in the keys written more than
once (the path) labelled where first written, `#M="..."`, `#M#` after;
`(nodes (LEN #N# INPUT VERIFIED CHANGED FINGERPRINT DURABILITY (#M# ...)
VALUE) ...)`, a node's key and reads by their labels, its value `(at OFFSET
LENGTH)` in the payload, `(atx ...)` if escaped there, or `none`. Each
record's first element is its length (the user's suggestion), so that a
reader may step over it; the file is a datum with labels all the same,
which `read-data` (below) reads. The payload, `DB.payload`, is the values
end to end, cut out when a query reads one.

**ASCII alone**, both files, so that an offset is a character's (kayak-rt's
`substring` and `string-ref` are constant time on ASCII text, a walk from
the start on any other): a key's text past ASCII written `\u{HEX}`; a value
past ASCII written with `\` and what is past ASCII escaped, and marked
`atx`. kc's programs are ASCII, but a program's text, its memos and some of
the checker's messages need not be.

**Read by offset** (`db-of-files`): a scanner of the graph, by index, that
makes no list of its characters and no datum of it; a key's text, by which
it is interned, assembled from its parts as written (the writer wrote them
as `key-text` does), not printed again. **Verified without values**: a
node's reads made current by revisions and fingerprints alone
(`db-made-current`), so that a run reads from the payload only the values
its queries read.

**Outputs as files**: the whole program's output (`asm`, `rust`) is not
kept in the files; the file it was written to is. `db-fingerprint` makes a
node current and gives its fingerprint, its value neither read nor made;
the driver writes the output only if the file's text has another
fingerprint, and leaves it, and its time, as it was otherwise, so that
cargo's rustc, or the assembler, need not run again (the user's ask).

**Datum labels in data** (`kfx/read.kfx`'s `read-data`): `#N=` and `#N#`
resolved after reading, in the order written, for data only, as Fixpt's
reader has them (`fixpt-read`'s `datum_labels`) and its listings write
them; FX-26's program text has none (`fx26-grammar.md`, a statement the
user doubts, `docs/to-discuss.md` 8), so `read-all` reads `#0=` as a name.

**Measured** (5000 procedures, best of 3): the files 3.1 MB and 7.8 MB, from
23 MB of text; an unchanged run 0.79 s on the command line (0.90 s at C415)
and 0.26 s in serve (0.25); a body edited 1.82 s and 1.16 s (1.79 s, 1.09
s). The bench's rows (`bench/kc.tsv`) are labelled with the commit the work
was measured on, C416's. On the way: the general reader on the graph took
three times as long as C415's format (2.5 s); what a node read printed
again to be interned; the payload's values unescaped at every read. The
scanner, the parts' texts as written, and escaping only past ASCII, each
measured, made it so.

Not yet: a record read only when its node is asked for (the lengths make
it possible, as the user suggested; a whole build verifies every node, so
it helps a partial one); `kc build`, each form's code an object file of its
own, reused while its fingerprint is (the user: "detect that the generated
object code is actually unchanged in order to avoid downstream work").

## C418. The graph read as it is needed; a database written only when changed (2026-11-13)

The user: "Go for it" (C417's next: a graph record read only when its node
is asked for; `kc build`).

Measured first, an unchanged run at 5000 procedures (0.79 s): over half of
it the kernel's (pages for strings made and dropped), the rest strings
built, text checked to be UTF-8, the program's text read again. What an
unchanged run made was the files as they were, written whole again, and the
source's forms read again to set inputs to what they were.

- **A database written only when it changed** (`db-dirty?`, set by any
  write of a node since it was made or read): a run that changed no node
  leaves its files as they were.
- **The source read again only when its text changed** (`kc-load!`): the
  text compared with the `text` input first; as it was, its forms'
  inputs are too.
- **A record read as it is needed** (`db-node-at`): at a load, each node's
  record is noted, its offset kept in its key's node (verified -1, its
  offset as its revision of change), and stepped over by its length (C417's
  length-first records, the user's suggestion); read the first time the
  node is asked for (`node-record`). An unchanged run asks for the output's
  node alone, current at once, its revision the database's.

**Measured** (5000 procedures): an unchanged run 0.17 s on the command line
(0.79 s at C417), 15 ms in serve (0.26 s); edits as they were (1.77 s a body
on the command line, 1.14 s in serve): what is left of theirs is the
memoized check's walk over every form, and the files written.

## C419. `kc build`: objects by fingerprint, assembled and linked only when new (2026-11-13)

The user: "are you architecting kc to compile each Rust crate to its own .o
file? I'm wondering if we can go so far as to detect that the generated
object code is actually unchanged in order to avoid downstream work"
(C417 made the KFX side's: Rust left as it was is not written, so rustc
does not run), then "Go for it"; and, of the chunks below: "I assume the
chunking is for cold builds? Ie warm builds work at finer grain?"

**`kc [--rt RUNTIME] build FILE EXE`** (and `build FILE EXE` in serve):
the program checked, then `objects PATH`, a query of what the program is
assembled of, each object named by the fingerprint of its code: runs of up
to 64 procedures, in order (their `asm-item`s' fingerprints folded), and
`asm-entry`, what is no procedure's (the globals' cells, `.globl` now, and
`kc_entry` with the value definitions and expressions). `objects` reads
fingerprints, not values (`db-get-fingerprint`, a read recorded as any is:
a node's revision of change moves when its fingerprint does, so the
reader runs again exactly when a reader of the value would). If the list,
and the runtime, are those `EXE.objects` says EXE was last linked of, and
EXE is there, nothing more is done; else each object not yet in
`EXE.o.d/` is written as assembly there, all of them assembled by one `cc
-c`, and the program linked (FX-26's `process-start` and `process-exit`).
A comment, an undo, an edit whose code is as it was: nothing assembled,
nothing linked; a body edited: its run of procedures assembled, and the
link.

**Why runs of 64, cold and warm alike**: an object each started the
assembler once for each procedure and gave the linker as many: 25 s cold
at 5000 procedures, and each edit's link of 5001 objects 0.6 s. A run of 64
is 80 objects at 5000; reassembling one, the edit's, takes under 10 ms
(measured: 58 KB of assembly), so a finer grain would spare nothing seen.
If it ever would: a run's procedures as weak symbols, an edited one an
object of its own, strong, linked beside its run, until the runs are made
again.

**Measured** (`bench/kc.tsv`, best of 3; the bench's kc and serve columns
are whole builds now, the third the run's alone): at 5000 procedures, cold
6.05 s (Fixpt's eval 4.82 s); unchanged 0.16 s, 6 ms in serve; a body edited
1.68 s, 1.00 s in serve (C418's 1.77 s compile and 0.25 s link before);
a comment 1.43 s with nothing assembled or linked. kc-steps builds every
program by `kc build`.

Not yet: objects no longer listed are left in `EXE.o.d/` (no pruning).

## C420. The checker's walk over a program's forms, no longer quadratic (2026-11-13)

The user, of kc's next steps (each form's check a query of its own, to
spare the memoized check's walk over every form; then types in kc's core):
"Yes".

Profiled first (a comment added to 5000 procedures, `--check`): 17% of the
run was one closure of `incr-check.kfx`'s `reads-key`, nearly all its own
time, and 5.5% `toplevel-marked`: quadratic, both. Before each top-level
form the checker binds a `%toplevel` marker, and kept every marker in its
list of markers (`cv`'s `mk`, `marks-adds`); `reads-key` looked each other
kind of marker up in it (`find-bind`), and a kind a program has none of
(`%module`, `%loaded`, `%instance`, `%toplambda`) walked it whole, at
every form. And `toplevel-marked` counted the bindings, all of them, at
every form.

- **One marker of each kind** (`marks-adds`): only the innermost of a
  kind is read (`scope-bind`, `reads-key`), so a new one takes an older's
  place; the binding list keeps them all, as before.
- **Bindings counted from the last marker** (`binds-counted`): those since
  the newest `%toplevel`, and its own count, and itself.

The compiler's fixed point as it was, every answer as it was. At 5000
procedures, a comment's check in serve 0.52 s (0.82 s); a body edited 0.89
s (1.00 s). The walk is now some 2.5% of a check: what is left of one is the
source read twice (`kc-load!`'s inputs, `checked`'s check), the memos read
from text and written again, and on the command line the database's files.
So each form's check as a query of its own, which would spare the walk, no
longer pays for itself: not done.

## C421. kc, step 11 (first part): strings and symbols (2026-11-13)

The user: "Go ahead" (step 11, then types in kc's core, which need symbols:
a sum's tag is one in Fixpt's layout). And: "I'm sorry: did you say fx26
has no vectors?" It has: arrays, `(arrayof T r)` (`make-array`,
`array-ref`, `array-set!`, `array-length`), FX-26's mutable vectors; a
reader's `#(...)`, an opaque `vector` in a `datum`; bytevectors and flat
arrays. What it has not is Scheme's `vector` and `vector-ref` by those
names, which was all I had found. Arrays are step 11's second part (C422).

**L0**: `(string "TEXT")`, `(symbol "NAME")` (a quoted symbol); the
primitives `string-length`, `string-ref`, `string=?`, `string<?`,
`string-append`, `substring` (L0's first of three arguments),
`symbol->string`, `string->symbol`, `string?`, `symbol?`.

**Native, in Fixpt's layout** (`fixpt-heap`'s `make_string`, `intern`):
a string a bloblet of kind 1, all suffix: its length, then its code
points, two a word, UTF-32 as Fixpt's are, so that `string-ref` is a load;
a literal in `.rodata`, its value its suffix's address, tag 4.
`string-length` and `string-ref` inline (the index compared unsigned, so
that one below 0 is past the end; past it, the runtime's
`kc_string_index_fail`, Fixpt's words, `string index out of range (length
N)`). A symbol a bloblet of kind 2, its fields its name, its hash (FNV-1a,
a fixnum) and its global slot, and a trailer; interned by the runtime
(`kc_intern`), so that `eq?` is a comparison of words; a literal's symbol
kept, once interned, in a cell of its name's, `.comm`, which every object
that names it shares (so `kc build`'s objects agree on one symbol each).
What is made past a literal (`string-append`, `substring`, a symbol
interned) the runtime makes in its own memory (no collector yet), as are
`string=?` and the rest. The runtime writes strings as Fixpt's `put_string`
does, symbols by their names.

**Rust**: `str_lit`, `symbol_lit`; the primitives kayak-rt's, by `mangle`;
`string` `Str`, `symbol` `Symbol`. kayak-rt's `string-ref` past the end now
says Fixpt's words, `string index out of range (length N)` (it said
`string-ref: out of range`). KFX refuses `eq?` (C75), so its symbols are
compared not, as yet.

`tests/kc/step11/`: 26 FX-26 programs and 19 KFX, each Fixpt's answer:
literals, escapes, the empty string, code points past ASCII, `string-ref`
at even and odd positions and past either end, comparisons, appends,
substrings and their failure, symbols interned through `string->symbol`
and compared, strings in procedures, closures and lists.

## C422. kc, step 11 (second part): arrays (2026-11-13)

The user: "I'm sorry: did you say fx26 has no vectors?" (C421: it has,
arrays). FX-26's arrays, `(arrayof T r)`: `make-array`, `array-ref`,
`array-set!`, `array-length`; and `(the T e)`, which L0 makes `e`.

**Native, in Fixpt's layout** (its `make-array`, `%make-bloblet-filled`;
`array-ref`, `field@` of the index and 2): a bloblet of kind 32, as many
fields as elements and a trailer, from the heap (`%r15`), each field the
value; element `i` the field `i + 2`, counted back from the value (`-4 -
8 (i + 2)`, a load); the length the trailer's distance less one, so
`array-length` is a load and a subtraction. The index compared unsigned
against it, so that one below 0 is past the end. All inline; no runtime
call but a failure's.

**What a failure says**: Fixpt's machines each word an array index out of
range their own way (`docs/to-discuss.md` 9), so kc says its own, in a
string's way, "array index out of range (length N)", and no step's
program states an answer for one; a negative length, as most of Fixpt's
do, "a bloblet's field count must be non-negative". The runtime writes an
array as Fixpt does a bloblet, `#<bloblet N fields 0 bytes>`.

**Rust**: kayak-rt's `Array<T>`, its `make_array` and the rest by `mangle`.

`tests/kc/step11/`: 8 FX-26 programs and 7 KFX more: reads, writes, the
empty array, one written out, one under `the`, of strings, filled and summed
in loops, made in a procedure. 270 programs in all, 68 by Rust, none wrong.

## C423. kc: types in its core: abbreviations, products, sums, datatypes (2026-11-13)

The user: "Yes" (per-form checks, then types in kc's core: C420 found the
first was a quadratic bug, and step 11 the second's need, symbols); "Go
ahead".

**A type's form is no code**: `define-type`, `define-datatype`,
`define-effect`, `define-generative` (`type-form?`) are keyed
`%type:NAME`, so that one added leaves the keys of the expressions after it
as they were (`#exprN` counts the others), and are items of kind `none`,
of no code in either back end. An item depends on the interfaces of the
names it looks up (C416), and a type's name is none it looks up, so an edit
of a type reaches the checker alone, as C423's question asked of it.

**Datatypes as Fixpt's parser makes them** (`parser.fx`'s
`expand-datatype`): a datatype's form is followed, in the front end's
forms, by each variant's constructor, a form of its own keyed by its name,
`(define C (lambda (%x1 ...) (sum C (product (1 %x1) ...))))`, so that an
item that calls `circle` reads `circle`'s interface as it would a
procedure's. The checker checks the text as written.

**L0**: `(product E ...)` of fields by position (`(1 E) (2 E) ...`; a field
by name needs its type's order, which kc has not yet from the checker:
`not yet`); `(extract E K)`; `(sum "TAG" E)`; `(tagcase E ARM ...)`, an arm
`(parts "TAG" ((K NAME) ...) BODY)` taking its product apart, `(whole "TAG"
(K NAME) BODY)` binding it, or `(else (K NAME) BODY)` binding the sum; its
binders numbered as any (C404), so a lambda's free variables leave them out.

**Native, in Fixpt's layout**: a product a bloblet of kind 37, its fields
in order, a trailer; a sum one of kind 36, its tag (a symbol, interned:
C421) and its payload. A value `i` is the field `i + 2` back from the
value, as an array's element (C422): `extract` a load. `tagcase`: the sum
into a slot; each arm's tag's symbol compared with the sum's, the first
the same binding its payload, whole or its fields, or the sum; its body in
tail position where the `tagcase` is (C413). The runtime writes them as
Fixpt does, `#<sum TAG>`, `#<product of N>`.

**Rust**: a type's form no item; sums and products `not yet` (KFX's
datatypes are nominal in kfx's Rust, C410, a mapping of their own to make).

`tests/kc/types/`: 16 programs of Fixpt's answers: abbreviations (one
between expressions), products and `extract`, the empty product, sums and
`tagcase` whole and `else`, datatypes shown, taken apart, of an empty
variant, in lists, recursive (a tree summed), returning closures, of
strings, and a `tagcase` in tail position looping 100000 times.

## C424. kc, step 12: refs (2026-11-13)

The user: "Yep keep going" (step 12, assignment). FX-26 assigns through refs
alone (`set!` is unbound): `new`, `get`, `set`, `(ref T r)`, constants of
the initial environment as FX-87's are.

**Native, in Fixpt's layout** (`make_box`): a ref a box, a bloblet of kind
10, one field and a trailer, from the heap; its value the field 2 back, as
an array's first element is (C422): `get` a load, `set` a store and unit.
The runtime writes one as Fixpt does, `#<box V>`. **Rust**: kayak-rt's
`Ref<T>`, `new`, `get`, `set`.

`tests/kc/step12/`: 10 FX-26 programs and 9 KFX, Fixpt's answers: a ref
set and read, written out, `set`'s unit, a global counter, a local ref, two
closures each over a ref of its own, refs of strings and of lists, a ref of
a ref, and a sum kept in a ref by a loop of 100000 tail calls (1000 in
KFX, which has no tail calls in Rust: C413).

## C425. kc, step 13: modules (2026-11-13)

The user: "Yep keep going". A module's value, as Fixpt makes it, is the
product of the values it gives, in order (`#<product of N>`); `with` takes
its names by their places (the checker's).

**In the front end, as datatypes are** (C423): a top-level module's items
are made top-level forms of their own, each named `m..x` (the checker's
spelling of a module's name, C89), its hidden items too, the module's
names renamed so inside them, a binder of the same name hiding one (a
lambda's parameters, a `let`'s, a `let*`'s in turn, a `tagcase` arm's;
nothing inside a `quote`); the module itself `(define m (product (1 m..x)
...))` of what it gives. `(with m E)` of such a module is `E` its names
renamed so; `(import m x ...)` is each `(define x m..x)`. So each item is
an item as any, keyed by its name, and an edit inside one reaches what
looks it up, as a procedure's does (C416). The checker is given the text as
written, `import`s expanded as kfx's `with-loads` expands them
(`import-items`; it had none, so an `import` was an unbound name to it).

Not yet: a module given as a value (`with` of a variable: its names'
places are its type's), a maker, a module in a module, `extend`; and a
module's value in Rust (a product, C423), so step 13 has no KFX programs.

`tests/kc/step13/`: 10 programs, Fixpt's answers: `with` of values and
procedures, a module written out, `hide`, `import`, a parameter hiding a
module's name, two modules one using the other, a recursive procedure, a
string, a ref hidden in one (a counter), a datatype in one.

## C426. kc: `define*`, `define-rec`, `letrec`, `plambda`, `proj` (2026-11-13)

The user: "Yep keep going". Before step 14, a sweep of kc over Fixpt's
corpus (`tests/fx26/programs`, 1169 programs) for what stops it: 77
compiled; the checker stopped 688 (most meant to be refused; but 154 for
`define-syntax`, FX-26's macros, which kfx's checker has not, 49 for
`load-module`, 19 for `module-parameters`); `not yet` of a form 384, most
`define*` (138), `plambda` (37), `letrec` (36), `make-icell` (28), `up-t`
(14), `make-bloblet` (13), prompts (11), `letrena` (10), `define-rec` (7),
`proj` (7). The cheapest first, here:

- **`define*`**: a definition whose effect is found from its lambda; its
  code a `define`'s (`form-name`).
- **`plambda`, `proj`**: a poly's value and one instantiated, their types
  the checker's, their code the value's; `((proj F T ...) ARG ...)` is `(F
  ARG ...)`.
- **`define-rec`**: each member a definition of its own, all at the
  group's position (`(KEY DATUM same)`), so that each sees the others (a
  name is in scope at a position not after its own, `<=`).
- **`letrec`**: `(letrec ((K NAME) ...) (VALUE ...) BODY)`, its binders in
  scope in each value (each a lambda, FX-26's rule) and the body. Natively
  Ghuloum's way: the binders' slots 0; each closure made, its free values
  as the slots then are; each closure's free values that are the group's
  binders patched from their slots, now made; the body, in tail position
  where the `letrec` is. Rust: `not yet` (closures that name each other).

`tests/kc/forms/`: 7 programs, Fixpt's answers: `define*`, recursive too;
`define-rec` of `even?` and `odd?` (100001 deep, tail calls); `letrec`,
recursive, mutual (a million deep), capturing a procedure's parameter;
`plambda` and `proj`. Found on the way: kayak's checker refuses `((proj new
@h) 5)`, which Fixpt takes (TODO).

## C427. The pin moved to Fixpt 196dd2a (2026-11-13/14)

The user: "Pull Fixpt from GitHub main, and adapt to it". 4e13016 to
fcb33f0, 21 commits, most of them Fixpt's own machinery: heap syntax (its
Rust front end reads, expands and parses FX-26's `syn` in the heap, no
`Syntax` left), no hash map seeded (one process's caches the same bytes as
another's: `fixpt-hash`), dumps that outlive their sources, `,disassemble`
naming a module's members, `cons` and primitives in line on x86_64, a
scheduler that fires what is due each time round (TODO §270, in the
`processes.fx` kayak copies), and §69 decided (a hidden abstract type in
an exported one stays refused, its message now saying the ways out). Two
things kayak carried over:
- **Masking through `#%toplevel`** (f0eb630d, TODO §267): an expression's
  effect keeps what its free names' types can see, and kayak's checker,
  like Fixpt's Rust one before, looked `n` up where `(with #%toplevel n)`
  is, so a local `n` hid the top level's: `(let ((g 5)) ((with #%toplevel
  g) g))` lost `g`'s latent `(alloc @heap)`. Now `named-refs-in` finds
  each `(with #%toplevel n)`, `#%toplevel:n` and `(with #%fx n)`, and
  `masked-res` sees the regions of the top level's or the standard binding
  named. KFX refuses `#%toplevel` (C397), so only FX-26 programs are
  affected; `tests/repl/toplevel-mask.in` asks `,type` of Fixpt's three
  shapes, the engine running no `#%toplevel` yet. (Fixpt's third case, a
  functor's dependent type, kayak's checker refuses, as it did.)
- **`fixpt-hash`**: the `fixpt-reps` kayak copies whole (C300) names its
  maps `fixpt_hash::HashMap`, so its file is copied whole too
  (scripts/built-in.py), as a crate `kayak-hash` that `kayak-reps` names
  `fixpt-hash`.
- And a module member's word named for it, as a `let`'s binding is
  (a43473c7, in `fx26/cg.kfx`'s `c-module-make`).

Drift otherwise read and not carried:
- `heap.rs`, `value.rs`, `layout.rs`, `print.rs`, `literal.rs`, the
  engine's `cellular.rs`, `prim.rs`: unseeded hashers, `#[inline(always)]`,
  a heap never at the fixed base; no format or behaviour kayak ports.
- `disasm.rs`: where a procedure was written, in `native_source`'s header,
  which kayak's listing reader does not read.
- The native `cellular.rs` and `x64.rs` (fa620fad): `cons` made in line,
  and a word's `prim P N` called directly with `P` and `N` known. Kayak's
  x86_64 machine (`cells/hand-x64.kfx`) already makes `cons` in line and
  does its primitives, giving the rest back to tier 1: nothing to carry.
- `check.rs` otherwise: heap syntax and hashers; its generative
  declarations (`kfx/reps.kfx`) unchanged.
Every mark at the pin.

What the check found:
- **Reals of 1e21 or more** (6864363b): Fixpt's checkers both read the
  Rust reader's syntax now, so `1000000000000000000000.0` is an `f64` in
  each, where FX-26's had read an int and the two disagreed. KFX refused
  such a literal for that (`kfx/read.kfx`); it reads it as Fixpt does now,
  and `tests/kfx-refused/real-past-1e21.kfx` is
  `tests/kfx/reals-past-1e21.kfx`, Fixpt's answer kept.
- **Fixpt's `cost/diamond-*`**: their last form is a macro's use that
  makes definitions, which the cells test, reading the text, took for a
  value (`#u` against Fixpt's `m40`). Where tier 1 leaves `#u` and Fixpt
  wrote something else, it asks `fixpt check`, which says `define NAME`.
- **A regression of Fixpt's** (6864363b too; `docs/to-discuss.md` item
  10): a `load-module` of a built-in module after about 50 definitions is
  refused, the FX-26 checker finding no loaded file ("it was not read").
  Four of kayak's corpus programs meet it, KFX's prelude before them.
  Reported; Fixpt fixed it in 95f92e6f (spans in characters, the table
  of texts by base gone, whose base 0 a built-in module took), and the
  user, 2026-11-14: "Update Fixpt from GitHub and try again". The pin
  moved on to 196dd2a (that fix, and 196dd2a8's reader finding ASCII
  once); neither touches a source kayak ports, the marks moved with it,
  and the corpus agrees again.

## C428. The pin moved to Fixpt 120b70e (2026-11-16)

The user: "Pull Fixpt from github main. There's a lot of changes to
adapt to". 196dd2a to 120b70e, 50 commits, most of them Fixpt's macros:
signatures written `(macro σ made)`, `desc` for a description's part, one
signature a macro, trailmarks, `begin-for-syntax`, a macro that defines a
macro, procedural macros' transformers in a phase-1 namespace of the
session's runtime, expansion limits; and under them a heap that tenures
syntax and budgets its cards, x86_64's write barrier marking and counting
as `Heap::set_slot` does. Kayak has no expander yet (§51's stage 1 is
next; `docs/research/fx26-macros.md` now names the pin's notation as the
port's), so those programs it refuses, as it did.

Carried:
- **A generative type's conversions named where it is defined**
  (cb172689): `(define-generative t T #:seal mk #:unseal get)`, both
  required, no `up-t`/`down-t` made from the type's name. Fixpt refuses
  the old form, so KFX does too (KFX ⊂ FX-26), in Fixpt's words
  (`generative-names`, from `parser-forms.fx`, now marked). The checker
  binds the two names, and, for what it asks of the type itself, `%up t`,
  `%down t` (a representation's lookups, `rp-program-gdecl`'s) and
  `%conversion NAME` (which type a name converts, for `names-conversion?`
  and the termination pass): no name read for its prefix. The emitter's
  newtype functions, the engine's lowering, its REPL lines and `cg.kfx`'s
  module members take the form's names; the restriction pass reads the
  keywords there as names, not values. Every test's form was given the
  names it had (`#:seal up-t #:unseal down-t`), so its Rust is the same;
  new: `tests/kfx/generative-named.kfx` (Fixpt: `42 2 1`), and
  `kfx-refused/generative-{unnamed,one-named,not-derived}.kfx`.
- Naming a conversion reads its global, `(read (globals mk))`, as Fixpt
  has it: `generative/named.fx` and `module-files/counter.fx` are refused
  by both (a first reading of Fixpt's `push_global` had it otherwise; the
  check's comparison with Fixpt said so).

Not carried (TODO §53):
- A `moduleof`'s label and binder written apart (687accf1): refused.
- `get-environment-variable` and `(io env-read)` (be3b4291): unbound.

Drift read and not carried: `heap.rs` (tenured syntax, the old space
spilled into, the card budget, a card's byte saying who dirtied it) and
the native `cellular.rs`, `x64.rs` (the barrier's mark and count): tier
1's heap is a semispace with no generations, cards or barrier, and its
x86_64 words store with none; `prim.rs`'s three primitives (the macro
limit and procedural call are the expander's; the environment's, §53);
`check.rs`'s checkers lent a runtime. `bc1f53a6`'s `vsubr` arguments
looked at once: kayak's checker was never exponential there
(`listof/append-nested-deep.fx`, 36 ms). Every mark at the pin.

## C429. Macros, stage 1 begun: Fixpt's expander made KFX by a script (2026-11-16)

The user: "Go ahead with the macro expander. Do your best to keep it
cacheable and incremental". The plan was a port by hand, decision for
decision (`docs/research/fx26-macros.md`), and it began so: the syntax
types (`macros-types.fx`) written again in KFX. Then two findings
changed it. Fixpt's expander is 25 files, 13,056 lines, each loading the
next, so a first program needs most of them; and kfx already takes the
FX-26 they are written in (anonymous tuples, `extract`, `->*`, symbols,
`case` of symbols, `datum`). So `scripts/macros-port.py` takes Fixpt's
text and makes it one KFX module, `fx26/macros/expander.kfx`:

- the files from the two roots `parser-top.fx` loads (`macros-imports.fx`,
  `macros-shapes.fx`), in load order; the expander's whole, of Fixpt's
  parser's files (`parser.fx`, `parser-types.fx`, `eager-reader-types.fx`,
  `syn.fx`, `parser-loads.fx`, `parser-forms.fx`) only what is reached,
  name by name;
- one module, so that its items are no globals, as Fixpt's module files'
  are not: at the top level, naming each would read a global (`(read
  (globals atom))`), which their types, Fixpt's, do not say;
- loads, module aliases and imports gone; each name the definition it
  resolves to, by namespace (a value's or a description's, apart, as
  FX-26 has them: `def-envs` is both in `macros-procedural.fx`), written
  as its global name, its own unless kayak's unit or another file has
  it, or kfx refuses it (`region`, `marks-of`), then `mx-FILE-NAME`; a
  `tagcase`'s tag found through its type; quoted data and `case` data as
  written;
- the module's region parameters, by position, `@ms @me @mm @mc @mp`,
  where a type or effect is written (a value may be named `rs`);
- forms ordered so that each follows what it names, each group that
  names each other a `define-rec` (Tarjan), types that name each other
  written out into their datatype;
- `(op a b c)` of `string-append`, `append`, `+`, `*` nested; a local
  `fail` (a name kayak-rt writes) and a local `list` renamed; two
  procedures a module value calls as the module is made given their
  effect (`mk-symbol`, `def-env-top`, `pure`); two identity tests of
  syntax, `eq?`, a shortcut, made `#f` (the same syntax made again);
- what KFX lacks, a shim written by hand (`fx26/macros-shim.kfx`):
  keywords as names `#:x` (C291), `eq?` of booleans, `datum-int?`,
  `datum-dotted`, `parse-nat`, `parse-number` of integers; the
  procedural primitives as not yet (stage 5), the limits Fixpt's
  defaults.

kfx found the expander's text, as made, by refusals that were kayak's
own, each now as Fixpt has it, tested by `tests/kfx/quoted-names.kfx`:
the restriction pass read a quoted datum's names as uses (`oneshot`);
the module checker's rule that a module value names no `->*` procedure
of the module, and its rule of what is made before what, read them so
too; and an `if`'s branches, `(cons a (cons b out))` and `out`, were not
joined as a list (C284 had one pair, Fixpt joins any), nor, so joined,
written a list's cells in Rust (`general-pair?`).

The user, 2026-11-16: "If you encounter deltas that seem incidental (eg
naming choice for freshly constructed names) but for some reason
interfere with incrementality or cacheabilty or parallelizability, make
a note of it in a file, call it INCR_IMPED.md". It has three so far:
fresh names numbered program-wide, the expander's state in globals
across its files (the `mu` table by number), and positions as offsets
into one text of every file loaded.

Where it stands: kfx checks the whole unit with the expander in it; the
Rust it writes does not build (its emitter writes a `->*` procedure's
signature from the type written, so `pfail`, which may abort, returns no
`Result` while its callers unwrap one), so the unit does not list it
yet. `check.sh` holds it to the pin (`macros-port.py --check`), and the
pin's moving remakes it. Incremental by design, its state a value per
form (`docs/research/fx26-macros.md`, "Cacheable and incremental"), when
the driver is written.

## C430. Fixpt's expander, made KFX, builds (2026-11-16)

C429's `fx26/macros/expander.kfx` checked, but kfx's Rust of it did not
build (1,866 errors). Three causes, each kfx's:

- **A `->*` procedure's signature.** The checker reads `(->* (T …) R)`
  as `(subr %found …)` and finds the effect (`found-define`,
  `found-rec`), but in its own copy of the form; the emitter wrote the
  signature from the form as written, so `%found`, no effect: `pfail`,
  which may abort, returned no `Result`, while its callers, which saw
  the effect found, unwrapped one. Now the checker notes each `->*`
  definition's type as found at its lambda (`def-type`,
  `found-types`), as it notes a type written as a name, and the emitter
  writes the signature from that (1,866 errors to 8).
- **`'()` given for a datum** was boxed as a list,
  `value::list(Value::Nil)`; a quoted datum is a `Value` already.
- **Shared procedures that hold syntax.** A procedure of an effect a
  process may have is `Send + Sync` in Rust (C102), and the checker
  refuses one that holds what is not. It took a datatype of no
  parameters as shared without reading its variants, so Fixpt's `syn`,
  which holds `datum`s (`Value`s, not `Sync`), passed, and rustc refused.
  Now such a datatype is read by its variants, as one of some is: so the
  checker refuses three of the expander's procedures. Making `Value`
  `Sync` would bar a union's procedures; kayak-rt has no unsafe code. So
  `macros-port.py` patches their types, `pure` to `(read rs)`, an effect
  no process has (TODO §51: a `Datum` of its own, `Sync`, ends this).

The expander is in the engine's unit, built, run by nothing yet: the
driver is next.

## C431. The engine expands macros: Fixpt's expander, driven (2026-11-16)

The user, asked whether a "driver" meant someone to run the script: the
code in the engine that calls the expander between the reader and the
checker, as Fixpt's `parser-top.fx` does. `fx26/macros-driver.kfx`,
written by hand, read into the expander's module last (so it names its
items, no globals), as `parse-tops`, `top-made`, `expanded-tops` and
`written-top` drive it: each top-level form made Fixpt's syntax; a
`define-syntax` defined, a name a global already refused, its typed
templates' checks given the checker as hidden definitions
(`%template:NAME:K`); any other form expanded (`expand-top`), each form
it makes shaped, sealed, its aliases resolved (`top-shaped`,
`sealed-written`, `aliases-resolved`), a definition of a macro's name
refused; then the reader's datums again. Places: a file's positions as
Fixpt's are (its index times `load-base`), a template's part's key its
place in the template (`source-of`); a list's line kept by its position,
an atom's the last list's before it. A failure, Fixpt's `p-err`, is
caught at each form and said as kayak's refusals are. `#t` and `#f` are
booleans to it, as Fixpt's reader reads them. The engine's `main` calls
it for a program that names `define-syntax` (`expanded`); a program
that does not is read as before. Not yet, each refused saying so: a
file's macros, `hide`, `begin-for-syntax`, a shape's definition.

What it took besides:
- The engine compiles `(with #%toplevel n)` and `#%toplevel:n`, the top
  level's `n`, as the expander writes a template's free names (TODO §51
  had it as a gap, C397's checker taking it).
- The expander's failure is a `goto` at a region of the top level, which
  a prompt there does not mask (Fixpt's rule too): `engine-main` and
  `main` say it (`expands`), not `driving`, which the REPL's procedures
  share (in it, their Rust changed); `main.rs` unwraps the `Result`.
- `kayak-fx26 --expanded FILE`: the program as expanded, a form a line.
  `run-fixpt-programs.py` counts Fixpt's lines by it for a program that
  defines a macro (a use that makes definitions writes one for each; a
  template's check none).
- `refused-fixpt-programs.py` holds the engine to Fixpt's refusals too,
  for each program that defines a macro (`--cells`, checked, not run):
  it found one, `macros/typed-letrec-pure.fx`. A `letrec` member of a
  group that may not end, named other than as a call's operator, now
  says `spin`, as Fixpt's does ("whoever is given it may call it again,
  with anything"); kayak had said nothing (C190), refusing only at a
  type written around it, which a template's check has not.

Fixpt's programs beyond KFX the engine agrees on: 296 before, 379 now;
it differs on none but `native/crossing-overflow.fx`, as before.

## C432. Expanded as written: before kayak's own rewrites (2026-11-16)

The user asked how bootstrap and test times look and whether
incrementality holds: kfx on the engine's unit 4.3 s to 11.0 s with the
expander (2.2 to 5.2 GB); kfx's own bootstrap, kc and its incremental
steps untouched (neither's unit has it); a run of a macro program 0.02 s;
a program with no macros not expanded. Macros' expansion itself is not
yet incremental (TODO §51); `kfx --cache` on the engine's unit fails, as
it did before the expander (TODO).

Then the order of TODO §51, the driver's not-yets first. Nine programs
failed, "car or cdr of a non-pair": the engine's reading had rewritten
the program before the expander saw it (`with-loads`: imports, `(tupleof
T …)` as `(productof (1 T) …)`, keywords' labels, staging), so a
template's `(tupleof f g ...)` reached it as a `productof` of fields. Now
each file is expanded as read, then rewritten (`read-files-expanded`), as
Fixpt's expander sees a program before its parser does; `--expanded`
writes the expansion before the rewrites. A top-level `hide` with macros
is its forms, as if written there (`tops-made`). `KAYAK_FX26_BACKTRACE`
set, the engine's failure says its backtrace (by Rust's names; a build
with line tables, `CARGO_PROFILE_RELEASE_DEBUG=line-tables-only`, says
where), which found both.

`refused-fixpt-programs.py` found what the move cost: a macro's name an
`import` defines again (`macros/import-macro-name.fx`), which the guard
had met as the rewrite's `define`s. The guard now reads what each form
defines as Fixpt's parser does, written: an `import`'s values, a
datatype's constructors, a generative type's conversions, a syscall.

Fixpt's programs beyond KFX the engine agrees on: 379 to 386. What the
rest found is the checker's (TODO §51's list).

## C433. A module maker's state: its values and refs, its instance's (2026-11-16)

The REPL's front could not expand: the expander's state is in global
regions (`@ms`…, where C429's translator put Fixpt's module's region
parameters), and a process may share nothing that can be written. The
user, asked what of this is the translator's: "Could you revise your
translator to map region parameters in modules to some other kind of
dynamic state that you transmit?", then, of extending C274's module
makers so that each holds its state ("route 1"): "Go with route 1; this
seems like a worthwhile KFX extension". Its first step.

A module maker's module (C274) may now have values that are no
procedures, `(define x T e)`, refs `(define n (ref T p) (new e))` among
them: its instance's state, made when the maker is called. In Rust:
- `IState<'p, Place_P>`, a field each, `Cell<Option<T>>`; the instance
  `I(place, &state)`, still `Copy`;
- each item given the place and then the state (`maker-st`), its calls of
  the items given both (`place-given`); a value's name in an item, its
  field (`maker_st.x.get().unwrap()`, KFX's values being `Copy`);
- `m::maker_state(p)`, in the module, where the items are named: the
  state allocated, then each value made in order, its expression able to
  call the items and name the values before it (FX-26's `letrec*`, the
  checker's rule of what is made before what); the maker `I(p,
  m::maker_state(p))`.
A ref is made by `new`, in the heap, not in the place: its type at the
place takes it. The async mode found one more (`async-corpus`): its
search for leaves (C377) tried each top-level procedure by writing it,
a maker too, whose values a procedure's writing takes not; a maker is
written as one (`put-maker`), so is no leaf. Test: `tests/kfx/module-place-state.kfx` (two counters in
two places, apart; a value of a procedure; Fixpt's answer, `212 111`).
Next: an item that fails, or is `poly`, or is named other than called;
a datatype in the module; then the translator makes the expander such a
maker.

## C434. A module maker's items: failing, `poly`, given as values (2026-11-16)

Route 1's second step (C433): a module maker's items as Fixpt's
expander's are.
- **One that fails** (an abort to a prompt tag that is a value of the
  module): its method returns a `Result`, as its function does.
- **One that is `poly`** of types: the checker took a `lambda` where a
  poly type of types is expected only of regions in KFX (C107), a module
  value's item being no Rust function of its own; a maker's is, so in a
  maker (`%module-maker` in scope) it is taken. Its method is generic.
- **Only those that need it take the instance** (`maker-needy`, to a
  fixed point): an item whose type names the place, or whose lambda
  names it, a value of the module, or an item that takes it. The others
  are plain functions, called and given as values as at the top level.
- **One that takes the instance, given as a value**: a closure over the
  place and the state, of the item's arity (`%maker-item NAME K`). It is
  not `Send + Sync`: the place and the state are not. A procedure of an
  effect a process may have (`pure`) that needs the instance cannot be so
  given yet; Fixpt's expander gives such ones where `(write rs)` is in
  the type (its `fails`), which no process has.

Test: `tests/kfx/module-place-items.kfx` (Fixpt's answer, `18 after 11;
refused: negative`), in the async mode too, which needed: a closure of
an item given as a value boxing its future (`boxed`, or `body` if it
fails), a plain item's capturing nothing, `Send + Sync`; and a `prompt`
in a maker's item, whose body holds the place, no `'static` future:
kayak-rt's `prompt_at`, a prompt of a body with a lifetime, an abort to
its tag ending it as `prompt`'s does, a capture under it refused (one
would keep the body past its place; Fixpt's expander captures none;
`lang/items.sexp` lists it among the emitter's calls). Found by it: `--bless` on
this machine keeps a new program's first answer as its `.out` (the
shared one, arm64 macOS's home, C134) and a later one beside it as this
platform's; a new program's `.out` written by hand once its text settles.
Not taken: a datatype, a type, an effect in a maker's module (the
emitter learns a program's datatypes and effects in a pass a maker's
module is no part of). The expander's types are no state: its
translator puts them outside the maker, or writes them out where they
name the place. Next: the translator.

## C435. A module maker's `->*` items, their effects found (2026-11-16)

Route 1's third step: nearly every procedure of Fixpt's expander is
`->*`, its effect found from its lambda (C191). A top-level module's
items are found so (`check-top-plain`'s `found-define`, `found-rec`); a
maker's module is checked as a module value, which reads every item's
type first, and refused one. Now `check-top-plain` finds a maker's
module's `->*` items first (`maker-found`, `maker-items-found`): in
order, each seeing the items before it, as the module's `letrec*` has
them (a group of procedures that call each other being a `define-rec`,
as the translator writes them), in a scope as the maker's lambda's
(each place a region variable, `'p` and `%place p`); the maker checked
with the types found written in, and each noted for the emitter
(`def-type`, C430), whose maker items' signatures come from them. Test:
`tests/kfx/module-place-star.kfx` (Fixpt's answer, 2), in both modes.
A maker for this is a procedure of places alone whose module has a
`->*` item (`maker-form?`): the cells tests found a functor taken for
one, its module's types unbound in the finding pass.

## C436. A module maker's types and effects, those naming its place (2026-11-16)

Route 1's fourth step. Of the expander's 23 datatypes and 119 type
abbreviations none but one abbreviation (`namer`) names its regions, nor
any datatype; of its 2 effects both do. The translator will define the
rest outside the maker; what names the place stays in it. So a maker's
module may now define type abbreviations and effects (`maker-type?`):
written in its Rust module before its state's struct, which moved into
the module, `m::IState`, where the types its fields name are; its items
written in the module's context as a top-level module's are
(`module-parts`), so that its effects and abbreviations are known where
they are named. A datatype in a maker's module is refused: the checker
reads a module value's as a structural sum, recursive ones not at all.
Test: `tests/kfx/module-place-types.kfx` (Fixpt's answer, 19), in both
modes.

## C437. The expander, a module maker: each program its own instance (2026-11-16)

Route 1's last step (C433-C436 its parts): `scripts/macros-port.py`
writes Fixpt's expander as a maker, `mx-in`, over one place, `mxp`
(no local of the expander's is so named; Rust's parameters are one
namespace), Fixpt's five regions (`@ms` … `@mp`) all that place; the
engine makes an instance per program (`expanded`, fx26/main.kfx, in a
`letrena`), so that nothing writable is shared, and a REPL, or a
process, may hold one of its own. Its interface, `mx-face`, gives
`drive-program`.

- Its types: those that name no region of its, nor a type or effect that
  does, in a top-level module, `mx-types`, which the maker's module
  includes, so that its constructors are no globals, as in Fixpt, where
  each is another file's item (as globals, a constructor's call read
  one, which Fixpt's `pure` types do not say). What names the place
  stays in the maker: one abbreviation, `namer`, and those through it,
  `fails`; two effects. `WRITTEN` stays: kfx finds a maker's `->*`
  items before its values are made, but Fixpt refuses a module value's
  value that names one (`fixpt-accepts.py` found it), and kayak may
  accept nothing Fixpt refuses.
- The checker: a maker's `->*` item found as a group of one
  (`found-rec`), so that a recursion that ends reads nothing, an item
  being no global (`found-define`'s reading of its global, and so
  `spin`, is a top-level definition's: Fixpt's `arrow_star_spins`); the
  finding sees the module's `%module`, its includes (`include-opened`),
  effects and abbreviations, each item at its place; `parse-scheme` sees
  the type variables in scope; a maker's `include` of a top-level module
  names it as no value (`%maker-include`); a maker's item's own lambda is
  a Rust function, no closure held on another thread (`maker-fn`); a
  `plambda` of types in a maker, as C434's lambda; the termination walk
  skips a `quote` (the expander's `reserved-words`).
- The emitter: a maker's `include`, a `use`; each item at its type as
  found; a `plambda` item its lambda; an abbreviation written out where
  named (`%maker-abbrev`), so that a procedure's type in it is at the
  place's lifetime; an item whose type names an abbreviation or effect
  that names the place takes the place (`maker-placed`); calls given the
  place by scope (`place-given`: no binding, no shadowed name, no
  `cond` test is a call).
- The driver's forms ordered callees first (`drive-form`, `drive-forms`
  a `define-rec`), as a module value's `letrec*` needs.

Test: `tests/kfx/module-place-include.kfx` (Fixpt's answer, 23), in both
modes. kfx on fx26.unit: 15.6 s, 7.5 GB (11.0 s, 5.2 GB at C431).

## C438. A module maker's value may not name its `->*` items (2026-11-16)

C437's check found it: kfx found a maker's `->*` items before its
values were made, and so accepted a value that names one, which Fixpt
refuses (its `->*` effects are found only after a module's values are
made; the expander's `WRITTEN` exists for it). KFX ⊂ FX-26: kfx refuses
it too (`maker-found`), in Fixpt's words, as `check-module` does a
top-level module's. Test: `tests/kfx-refused/maker-value-star.kfx`.

## C439. The expander's session written; a process may make no instance (2026-11-16)

The REPL's step of TODO §51, begun and parked. The driver gives an
instance's session what Fixpt's `parse-program` gives its REPL: each
entry read from the reading state the one before left (`macros-at!`,
`shapes-at!`), its state after kept by a number of its own
(`macros-after!`, `shapes-after!`) for the entry after it.
- **The driver** (`macros-driver.kfx`):
  - `drive-entry`: an entry's forms, from state `s`, give the forms and
    the state they leave, or, refused, why, said as the checker says its
    own, and `s` again. `drive-draft` does the same for a draft's,
    `,type`'s or a hint's, and leaves no state. `drive-macros?` says
    whether a state has a macro, so an entry read in it is expanded though
    it names none.
  - The places met are the session's, so a macro is placed where it was
    written when a later entry uses it. A draft's are let go of after it.
  - `mx-face` gives all three beside `drive-program`.
- **Tests:**
  - `tests/kfx/module-place-held.kfx`: an instance's item held as a
    closure in a product of refs, as the front's state was to hold its
    expander's. Fixpt answers 7, and so does kfx's Rust.
  - `tests/kfx-refused/process-maker.kfx`: a process making an instance
    in its own place, refused by both.
- **Why parked:** the front, a process, was to own an instance in its
  place (C437's plan). Fixpt refuses that: a process's body may hold only
  what is transmissible, globals too, and Fixpt's `untransmissible_in`
  takes no place, though the maker's own `poly` binds it ("its body holds
  `counter-in`: a (place p)"). The machine's thread could hold one, but
  the user chose to ask Fixpt first, for a place at a region the
  procedure's own `poly` binds to be transmissible, as a mailbox at one is
  (`INCR_IMPED.md` §4, `docs/to-discuss.md` 11).
- **The front's side**, written and set aside: an `f-expanded` that
  submit, `,type`, drafts and holes read through, refusals said as the
  checker's; the instance's items as closures in the front's state; the
  session's state made the front's only when the worker accepts the
  entry; `(goto r)` on the front's procedures that may expand.
- **Records:** `INCR_IMPED.md` §2 also gets what C437 meant to add, which
  was never written: parallelism, and kayak's way round.

## C440. A shape's definition; `with`s held to what the expander read (2026-11-16)

The first of the driver's not-yets (TODO §51). Of Fixpt's programs, two
met one: `modules/modnames-sealed.fx`, a shape's definition, and
`procedural/for-syntax-helper.fx`, `begin-for-syntax`, which is stage 5's.
`import-with-syntax` is met by none, since the engine has no top-level
`include` for it to come through.
- **The driver**: a shape's definition, `(define-syntax s (modnames …))`,
  is held to the globals and defined (`shape-defined!`), as Fixpt's
  `top-made` has it. It makes nothing.
- **The checker**, what the expansion then asks of it:
  - `(the (modnames member …) e)` (Fixpt's `the_sealed`): `e`'s module
    type sealed to the names (`sealed-to`, over C286's `restricted`).
    Each member's kind is checked as Fixpt's `sealed` checks it, in its
    words. The values kept are noted (`module-arg`) so the engine
    reshapes the module, as `check` notes any module value where a module
    type is wanted. A `modnames` inside a `modnames` is refused, as kayak
    doesn't take it yet.
  - `(with m #:relied ((value …) (type …)) body …)` (Fixpt's
    `shape_held`): what the shape names, `m` gives; a name in the body
    that `m` gives, the shape names. The body's names are all the symbols
    it holds, so this is stricter than Fixpt's, which takes the values the
    body uses of the module.
  - Fixpt's `with_as` reads the expander's `with-uses` table by each
    `with`'s place. kayak's checker sees datums, so the driver writes a
    `with`'s uses into it: `(with m #:uses ((name projection?) …) …)`,
    after the module. That keyword is kayak's, written by the driver and
    read by the checker. The checker refuses, in Fixpt's words, a macro
    the body uses by a name `m` gives a value of, and a template's `(with
    m name)` where `m` gives no `name`.
  - The engine's lowerings, `lower.kfx` and `cg.kfx`, drop both keywords.
    The cells' `cg.kfx` reshapes a `the`'s module as it reshapes an
    argument's (`parse-arg`), which the check's cells tier found.
- **What the `with` rules found**: before this, any `with` carrying
  `#:relied` failed the checker for its arity, so three of Fixpt's refused
  programs (`macros/with-shadows-macro*.fx`, `with-value-missing.fx`)
  were refused by accident. Taking `#:relied` alone made the engine
  accept them. `refused-fixpt-programs.py` found that, and `#:uses` puts
  the refusals back.

Fixpt's programs beyond KFX the engine agrees on: 386 to 390
(`modnames-sealed`, `modnames-the`, `macros/with-value`,
`with-macro-not-given`). Fixpt's 594 refused programs are all refused.

## C441. The pin moved to 0585488: Fixpt's answers to INCR_IMPED.md (2026-11-16)

The user: "Pull fixpt main from GitHub. They claim they've addressed some
of your issues". Three commits since 120b70e, each of `INCR_IMPED.md`:
fresh names per form, `%x:K.N` (§1); the `mu` table gone, recursive
syntax types as values (§2); a procedural macro's aliases per form,
`%~K.N:x`. Not §4: `processes.rs` is as it was, so the REPL stays parked.
- **The expander**, regenerated (`macros-port.py`), takes in two new files
  of Fixpt's, `macros-syntax.fx` and `macros-fresh.fx`. One patch is new:
  `one-var`'s procedure holds syntax, so it gets `(read rs)`, as C429's
  three do (C102: a `datum` is no thread's to share).
- **The driver** follows `parser-top.fx`: each top-level form begun
  (`drive-begun!`, Fixpt's `form-begun!`: its key, and what one form
  keeps reset), and `aliases-resolved` given the expander's `captor-tag`.
- **The shim**: `mx-call-procedural`'s expansion is now its tag, a string.
- **Drift**: the expander's 13 sources and `parser.fx`, whose change
  (an alias's digits `K.N`) no kayak code reads; `cg.kfx`'s mark moved.
- `INCR_IMPED.md`: §1 answered, §2 in part, with what Fixpt's TODO §290
  leaves for kayak's cache to say (expansion numbers, source keys).

Fixpt's programs beyond KFX the engine agrees on: 390 to 391. Fixpt's
596 refused programs are all refused.

## C442. A file of a place, loaded: an instance, made at each load (2026-11-16)

Fixpt's answer to `INCR_IMPED.md` §4 (`docs/research/closed-code.md`, in
their working copy): a process may not hold a module maker defined at the
top level, but it may load a file of `(module-parameters ((p place)))` in
its own place, since a load sees the standard environment alone and the
spawn check exempts a load's globals. The user chose this route ("Load
route"). Its first step, kfx's side.
- **The load**: kayak makes a load static, one module per path and
  arguments (C116, C117). A file whose parameters are one place is now
  instead a module maker (C274, C433), `made--load--K`, defined at the top
  level as a load's module is, and each load a call of it with the place
  given. That is FX-26's meaning, a file of parameters made again at each
  call, where C117's sharing was not quite it. A second load of the path
  calls the same maker.
  - **Its interface**, `made--load--K--face`: the file's procedures whose
    types are written (a `subr`, not a `->*`'s), as `(val name T)`. The
    file's own effects and type abbreviations are spelt out there, since
    the interface is outside the file. A value of no written type is not
    in it, so kfx refuses a `with` that names one: stricter.
  - **What it sees**: its module begins `(%loaded path %instance)`, as a
    load's does. The module-value path (`module-expr`) and `maker-found`
    read it as `check-module` does a top-level one (`loaded-scope`): the
    standard environment, the loads' modules, and the maker's own place
    and marks. The emitter skips the mark.
- **A process may hold a load's globals** (`holds-checked`), as Fixpt's
  `shared_globals` exempts them.
- **The corpus**: a refusal may say where in a file the program loads.
- **Tests**:
  - `tests/kfx/process-load-place.kfx` (with `loads/counter-place.fx`):
    Fixpt's example from closed-code.md. Fixpt answers 7, and so does
    kfx's Rust.
  - `tests/kfx-refused/load-place-global.kfx`: such a file naming its
    loader's global, refused by both.

Next (C443): the expander written as such a file, so that the REPL's
front and the engine each load an instance in a place of their own.

## C443. The expander a file of a place, loaded where it is used (2026-11-16)

The load route's second step: the expander is no longer `mx-in`, a maker
in the unit, but a file of `(module-parameters ((mxp place)))`, which the
engine loads in a place of its own (C442), as the REPL's front, a process,
will.
- **The translator** (`macros-port.py`) writes two files:
  - `fx26/macros/expander.fx`: the shim, Fixpt's files, and the driver,
    the items of one file of a place. Fixpt's five region parameters
    become its one place, as in C437.
  - `fx26/macros/expander-types.fx`: the types that name no region of its
    (C437's `mx-types`), a file of no parameters. `expander.fx` includes
    it, `(include (load-module "expander-types.fx"))`. Being a load, it is
    made once for every instance, so they all share its Rust enums.
- **The boundary**: a loaded file sees the standard environment alone,
  not kayak's reader's `sexp`, nor its prelude (the `i64` helpers). So
  the expander is given and gives standard data, located datums: a node
  is an atom, `(0 file line pos d)`, or a list, `(1 file line pos node
  ...)`. `fx26/macros-bridge.kfx`, in the unit where the expander was,
  converts the engine's forms to them and back, and keeps no state. The
  driver (`macros-driver.kfx`, read into the file last) reads and writes
  them in place of `sexp`s. Its line table uses the standard arrays in
  place of the prelude's flat arrays.
- **The interface**: the driver's `drive-program`, `drive-entry`,
  `drive-draft` and `drive-macros?`, each of a written type over
  `datum`. Of a file's procedures, those whose types name a type of a
  file it loads (`syn`, …) are left out of a load's interface (C442's,
  `type-names-in`), since the top level can't name those types.
- **kfx's units read their files' loads** (`sources-of`, `unit-loads-read`):
  each file's loads, from the manifest's directory, are given to its
  reading, as a single file's are. A unit's file had none before. A
  load's text is in no memo's key yet (`kfx --cache`; TODO).
- **`fixpt-accepts.py`** gives Fixpt the unit's loads beside it. Fixpt
  accepts the engine's unit with the expander loaded so.

The engine: 391 of Fixpt's programs beyond KFX agree, as before; Fixpt's
596 refused programs are all refused.

## C444. The REPL expands: its front loads an instance of its own (2026-11-16)

The load route's last step, and TODO §51's REPL, parked since C439. The
front, a process, loads the expander (C443) in its own place, `((proj
(load-module "macros/expander.fx") p))`, in its spawn's body, which Fixpt
allows (C442). It gives the instance's procedures to `f-new` as closures,
so `f-new` itself takes no place.
- **The front's side** is C439's, set aside then and redone now over the
  instance's located datums (`macros-bridge.kfx`). `f-expanded` reads an
  entry's, a draft's, `,type`'s or a hole's text and expands it from the
  session's reading state, if the text defines a macro or that state has
  one. It writes the expansion out for the worker that checks it and the
  machine that runs it. An entry's state becomes the session's once the
  worker accepts the entry. A refusal is said as the checker's are. A
  hole the expander refuses is not tried. The front's procedures that may
  expand have `(goto r)`, the instance's prompt tags.
- **kfx: a unit's loads shared across its files** (`with-loads-known`,
  `units-forms`). Two of the engine's files load the expander now, so its
  types' file, which each loads in turn, must be one module, as in one
  program.
- **Test**: `tests/repl/macros.in`. A macro an entry defines, used by a
  later one; and two refusals, a template placing an expression twice and
  a use that fits no pattern. The values and refusals are Fixpt's REPL's,
  in its words. The effects differ: kayak's checker shows a `(with
  #%toplevel get)`'s region unsolved (`(read ?0)`), as it does for one
  written by hand (TODO).
- **Not yet**: the legacy `rust-repl` (live-repl.kfx, C103), which
  `repl-sessions.py --rust-repl` runs on the sessions without macros; a live session's drafts and hints with macros,
  untested.

## C445. kc expands: each form a query, by the names it reads (2026-11-16)

TODO §51's per-form caching, folded into kc (the user's choice, asked
first: nothing kayak runs today expands a form twice but a REPL's draft;
kc already re-runs only what an edit reaches). kc's step 15 (`kc.md`):
a file that defines a macro has each written form expanded as a query of
its own, `expansion PATH KEY`, by an instance of Fixpt's expander made for
it (`kc/expand.kfx`; making one costs nothing measurable).
- **What a form's instance is given** (`drive-given`, `macros-driver.kfx`),
  and nothing else, so its dependencies are what its query reads:
  - its text, cut from the file where it begins, read alone at line 1, so
    that a form moved is as it was (the driver moves the places back);
  - the definitions of the macros it names, followed through what each
    names, at the form and at the definition (`macro-in`, a query of the
    scope), in the order written (`macros-for`, whose value, not the
    expansion, changes when a form is put before another);
  - of each name met, what the last form before written to define it says
    of it (`global-in`, `entry`): its shapes (the top level's `top-vals`,
    `top-descs`: a `with`'s `#:relied` reads its module's type's), and
    that it is a global (so that a macro may not take its name, as
    Fixpt's check says);
  - how many forms of its hash were begun before it (`wseen`, an input),
    the key of its fresh names (`form-begin!`, Fixpt's `%x:K.N`).
- **First tried in the engine**: each form of a program expanded alone,
  each in an instance of its own, against the program's, positions too.
  With macros alone, 241 of Fixpt's 269 macro programs agreed. The rest
  showed what else a form reads: the shapes of the globals it names
  (`cost/diamond-*`, `with` of a module), then the globals for the
  "defined already" refusals. That mode is gone; kc is compared with the
  engine directly.
- **What is no written definition's**, the driver finds after, in order,
  and refuses: a macro a use makes, or a `hide` holds, named by a later
  form, not yet (6 programs); a global a use defines, its shape known,
  named later, not yet (2); a macro taking the name of a global a use
  defined, as Fixpt does.
- **Check**: `scripts/kc-expanded.py`, in the full check: kc's
  `--expanded-located` against the engine's (now the reader's data, so an
  atom keeps no line in either) for each of Fixpt's 269 programs that
  defines a macro. 261 agree, node for node and place for place; the 8
  are KNOWN, each with why.
- **Measured** (`kc-steps.py`'s `expansions`, five forms, two macros):
  cold, 5 expansions; the same text, none; a procedure's body edited, 1
  (its own; the last form names it, but its entry is the same); a macro
  edited, 3 (it, the macro that uses it, the procedure that uses that); a
  form put first, 1 (it), and 3 `macros-for` again, their values the same.
- **kc runs them**: `tests/kc/step15/` (four programs: a use, hygiene, a
  macro using one defined before, macros that make definitions), their
  values Fixpt's. kc's l0 lowers `(with #%toplevel NAME)` as the global
  where no local hides it, which every expansion writes.
- **The program as expanded**: kc's forms' inputs are made of the
  expansion (`kc-load-forms!`), and `checked` checks it, from an input
  `expanded PATH` (each form's located datum) that `kc-load-file!` sets.
  A refused expansion is `checked`'s refusal.
- **Found**: the expander's type `part` and kc's database's datatype
  `part` were one type in kc's Rust (the emitter's names are a unit's),
  as `atom` would be. The port now gives the expander's types kc's types'
  names apart too (`unit_names`, kc's types alone). kfx should refuse it,
  or keep a loaded file's types apart (TODO).
- **Fixpt's TODO §290** asks what kayak's cache needs of expansion numbers
  and source keys: neither. Each form is expanded in an instance of its
  own, whose numbers start over, and the driver writes places, not keys
  (`drive-placed`). What a form needs of the forms before it is by name.

## C446. Procedural macros: a transformer's request, answered by the engine, kc's input (2026-11-17)

TODO §51's next, stage 5 (`fx26-macros.md`). The user, on asking for it:
"I suspect this is the case where we will maybe regret the implementation
strategy you have chosen (of translating Fixpt's expander directly), since
I imagine we could hypothetically wire the compilation of procedural
macros into the demand driven model quite directly".
- **The seam is Fixpt's own**: its expander asks its runtime by text
  (`%forms` for `begin-for-syntax`, `%define` where a macro is defined,
  `%apply` at a use: `procedural::asked`), and reads the answer back as
  text, `=` and what the transformer gave, or `!` and why. That is a
  function of the request's text alone, the transformer deterministic (its
  type allows reading phase 1's globals and nothing else). So the port
  does not stand in the way of the demand-driven model: what it asks is a
  query's key, and its answer the query's value. kc keeps each as an
  input, `phase1 REQUEST`, set by its driver, kept in the database, so a
  request runs once per database, and a form's expansion depends on the
  answers it read. Where the translation costs something is below.
- **The shim** (`macros-shim.kfx`) answers from what the instance was given
  (`drive-answers!`, in the order asked, so each is found at once) and
  keeps the first request it was not given (`drive-needed`), saying not
  yet. The host answers it and expands again: the expander, deterministic,
  asks what it asked before, then more. A host that does not answer (the
  REPL, whose heap is live) says not yet, as before.
- **The answer is the engine's** (`kayak-fx26 --phase1 REQUEST`): Fixpt's
  prelude (`procedural-prelude.fx`, made a KFX string by
  `macros-port.py`, so drift-checked), the phase-1 forms, the transformer
  at its style's type with `procedural::helpers` around it, and of
  `%apply` the call (`call_transformer`'s arguments), checked and run as a
  program; its value written. In a process of its own: the engine's heap,
  machine and node tables are one, the program's (the engine runs itself,
  `/proc/self/exe`; kc runs the `kayak-fx26` beside it). Fixpt's
  checker's and runtime's messages are kayak's (`car of nil` where Fixpt
  says `car or cdr of a non-pair`); refusals are counted, not compared.
- **kc's own code for transformers, later**: kc compiles neither `datum`
  nor `typecase` yet, nor the prelude's `letrec`s. Once it does, a
  transformer is one more definition kc checks and compiles as a query, and
  a request's answer kc's run of it: the engine's process is the one part
  to replace, behind the same input.
- **What the restart costs**: an expansion runs again for each request
  not answered yet (a form of k procedural uses, k+1 runs, each from the
  form's start), where Fixpt's expander calls its runtime and goes on.
  Measured on `returns-its-input.fx` (a transformer that returns its use,
  refused at the depth limit, a thousand uses): 35 s, all restarts and
  processes. A syntactic closure's transformer is never given the
  expansion's tag (`call_transformer`), so the shim keys its `%apply` with
  tag `0`: one answer for all thousand, 65 ms (INCR_IMPED.md §6). An
  explicit renaming one's chain would still be quadratic.
- **`begin-for-syntax`** in the driver (`for-syntax!`, as Fixpt's
  `parse-top` has it), and among the heads that make a program expanded.
  In kc a procedural macro's definition needs the `begin-for-syntax` forms
  before it (`wforsyntax`, an input), replayed before it.
- **A label is never an alias** (`label-of`, kfx/check.kfx): `%~K.N:x` is
  `x`, as Fixpt's parser's `label-name` has it
  (`procedural/renamed-type-label-quote.fx`).
- **Found**: kc's identity did not cover the expander it loads (a file of
  a place, in no unit), so a database kept memos an older expander made;
  `compiler-identity.py` now hashes `fx26/macros/expander.fx` and its
  types too (C445's gap).
- **Checks**: the engine on Fixpt's 41 procedural programs (40 in
  `procedural/`, one in `macros/`): each gives Fixpt's value, or is
  refused where Fixpt refuses, but `split-er.fx`, whose `f` (an `allof`
  with a `typecase`) kayak's checker refuses before any macro.
  `kc-expanded.py`: 259 of 269 as the engine; 2 more KNOWN, a procedural
  macro that makes a macro (MADE, as C445's 6). `tests/kc/step15/`: three
  more (explicit renaming, syntactic closures, `begin-for-syntax`), run by
  kc's native code. `kc-steps.py` measures a procedural file's edits:
  cold, 6 expansions (two of them asking again); the same, none; a use's
  operand edited, 2.
- **The full check**: beyond KFX, the engine agrees with Fixpt on 411 of
  its programs (391 before), its checker refusing 62 (82).

## C447. A procedural macro's transformer compiled by kc, its phase-1 program queries (2026-11-17)

The user, on C446's report ("kc can't run them with its own native code
yet: it doesn't compile datum or typecase"): "you aren't done. This sound
important." So kc answers each transformer's request itself, behind the
same input (`phase1 REQUEST`, C446); the engine's process is gone from kc.
- **A request's phase-1 program is a file of kc's database.** The program
  the engine built (Fixpt's prelude, the phase-1 forms, the transformer at
  its style's type, the use's call) is now `phase1-parts`, in a file both
  units load (`fx26/macros-phase1.kfx`), in two parts: what it defines, and
  the call. kc's driver (`phase1-asked`) loads it as a file named by the
  fingerprint of the definitions (`%phase1-FP.fx`), so each transformer is
  a file of its own whose forms are queries as any program's (`checked`,
  `l0-item`, `asm-item`, `objects`), builds it with `kc build`, runs it, and
  takes its last line: `=` and the value, `=#u` of none, or `!` and why
  (Fixpt's words, `evaluation failed: ...` taken off; a refusal's place in
  the program no one wrote taken off, `phase1-unplaced`). A use's call is
  the one new form at each use: its code assembled and linked, the
  transformer's kept. That is the wiring the user had in mind (C446): the
  compilation of a transformer is demand-driven, in the same graph as the
  program that uses it.
- **What kc compiles now**: a quoted datum of any kind (its pairs made as
  `cons`es when met: `l0-quoted`); `quasiquote`, as the checker reads it
  (C79): `,e`, `,@e` (the runtime's `kc_append`, a copy), nesting kept
  (`l0-quasi`); `typecase` (int, char, bool, nil, pair, string, symbol,
  procedure), `if`s on the shape's predicate, each arm's name a `let`
  (`l0-typecase-as-if`); `int?`, `char?`, `bool?`, `procedure?`,
  `symbol=?` (symbols interned, so `eq?`); `(list)`. No change to the
  checker: kc's front end was already FX-26's. Not yet: a typecase of a
  keyword, a vector and the others, a dotted datum.
- **The runtime shared, for transformers**: `kc build` links each program
  against `libkayak_kcrt.a`, 23 MB, 121 ms a link; a transformer's
  program links against the same crate built as a `cdylib` too
  (`libkayak_kcrt.so` beside kc, its directory the run path), 19 ms. A
  user's program is still linked static. Where it is built: `DB.phase1/`
  beside a database (kept, so a transformer built once is built no
  more), else `/tmp/kc-phase1-PID`, taken away at the run's end. Without
  `--rt`, the runtime is the one beside kc, not one under the working
  directory.
- **What it costs**: a cold run, kc against the engine's in-process
  `--expanded-located`: `aif-er.fx` 0.18 s (0.08), `returns-its-input.fx`
  0.12 s (0.06), `twice-sc.fx` 0.25 s (0.10), `contract-defn.fx` 0.33 s
  (0.13); before the shared runtime, 0.53, 0.35, 0.71, 0.88. With a
  database: the same file again, 0.00 s (no request runs); a use's operand
  edited, 0.09 s, 11 query bodies (the new call's `l0-item` and `asm-item`
  one each), one assembler, one link, one run. Each use still links (one
  call a form of the program); a transformer given its use at run time
  would make each use a run alone, but kc's code has no reader yet.
- **Found**: kc's message where a transformer fails is Fixpt's (`car or
  cdr of a non-pair`), the engine's its own (`car of nil`):
  `procedural/transformer-fails.fx` now KNOWN in `kc-expanded.py` (WORDS),
  as kc's being the right one. The engine's words are kept: two REPL
  transcripts state them.
- **Checks**: `kc-expanded.py`, 258 of 269 as the engine, 11 known (one
  more, WORDS). `tests/kc/step15/`: 4 more (a quoted datum, a quasiquote,
  a typecase, a transformer that takes its use apart by `typecase`), each
  Fixpt's value. `kc-steps.py`'s procedural edits now count `asm-item`s
  too: cold 8 (the prelude's 6, the transformer, the call), the same 0, a
  use's operand edited 1.
- **The full check**: all passed; the engine's comparisons as before
  (beyond KFX, 411 of Fixpt's programs agree, 62 its checker refuses), the
  REPL sessions as before.

## C448. C447's gaps: a use a run alone, the REPL's procedural macros, the rest of `typecase` (2026-11-17)

The user, after C447: "Make a todo to revisit this question later" (an
expansion run again, or suspended: TODO §51, with C447's reasoning and
what to measure), "Focus now on remaining gaps listed in TODO §51".
- **A use is a run alone.** A transformer's program (of `%define` and
  `%apply`) now holds no call: kc adds its own `%phase1-read`, a reader of
  the data Fixpt's expander writes a use as, and `%phase1-apply`, the
  transformer called on the use read and the tag, as `phase1-called` calls
  it (`kc/phase1-read.kfx`, FX-26 that kc compiles with the rest; Fixpt
  takes it too). The program is built once, where the macro is defined;
  each use runs it with `--apply`, the tag and the use's text on its
  standard input (`process-write`: no command line's limit). The runtime
  makes the definitions (`kc_entry`, which now keeps the heap's next word,
  `kc_heap_next`), then calls `%phase1-apply` through `kc_call2` (kc's
  entry object's: the heap in `%r15` and `%r14`, as kc's code has it) at
  `kc_apply_entry`. A use the reader does not read (a vector, a keyword,
  `#u`, a number past a fixnum: `%phase1-unread`) is called as before, the
  call a form of the program, built and run. Measured, a use's operand
  edited with a database: 0.09 s, 11 query bodies, an assembler, a link
  and a run (C447), now 0.04 s, 5, one run. Cold: `contract-defn.fx` 0.33
  s, now 0.16 s; `aif-er.fx` 0.18, now 0.15.
- **The REPL's procedural macros**: its front answers each request its
  entry makes as the engine's driver does (`phase1-asked`, the engine run
  again, `--phase1`: a process of its own, the REPL's heap untouched), keeps
  the answers for the session, and expands again. A draft is expanded with
  the answers known alone, so that no process runs as one types (a live
  session's draft with a new use is untested: its speculation should say
  not yet until it is entered). A process holds nothing
  that reads `@io`, so `phase1-asked` is now of any region
  (`phase1-asked-in`), the front's its own. `tests/repl/procedural.in`.
- **`typecase`'s other shapes** in kc: `bloblet` (`array?`), `box`
  (`ref?`), `sum`, `product`, `vector`, `bytevector`, `keyword`, each a
  bloblet's kind (Fixpt's `layout.rs`; the runtime's `kc_is_kind`), with
  Fixpt's predicate names (`TYPECASE_SHAPES`). `f64` and `f32` not yet: kc
  has no floats. A dotted datum, quoted or quasiquoted (`(a . b)`, the
  reader's `.` before the last item); a quoted vector still not yet.
- **Kept tidy**: `kc build` takes away the objects in `EXE.o.d/` its
  program no longer lists (C419's gap); a run that made
  `/tmp/kc-phase1-PID` takes away, with its own, any whose process is no
  more (one a `not yet` ended). kc's identity covers its runtime
  (`crates/kayak-kcrt/src/lib.rs`): a transformer's answer kept in a
  database is the runtime's run of it, its words the runtime's.
- **Checks**: `tests/kc/step15/`: a dotted datum, a typecase of kinds, a
  use the reader leaves to the compiled call, each Fixpt's value.
  `kc-steps.py`'s procedural edits: cold 9 `asm-item`s (the prelude's 6,
  the transformer, the reader, `%phase1-apply`), a use's operand edited
  none. `kc-expanded.py` as C447's.
- **The full check**: all passed; kc-steps 336 programs, 0 wrong; the REPL
  sessions 14 (one more, `procedural.in`); the engine's comparisons as
  before (beyond KFX, 411 agree).

## C449. A transformer's request asked as the expansion goes: no expansion run again (2026-11-17)

The user, on my answer to "isn't that middle path in fact giving us the
best of both worlds?": "remove the quadratic case before steps 1–5, yes"
(steps 1-5: kc's code in memory, TODO §52). An expansion that met a
request not answered yet ran again from its form's start once it was
(C446): k+1 runs of a form of k new requests. Not async, nor a
continuation: Fixpt's expander calls its runtime and goes on, and so now
does kayak's.
- **The shim asks.** `mx-expand-procedural` calls the asker its host gives
  (`drive-asker!`) and keeps each answer by the request's key (C446's tag
  `0` for a syntactic closure's: `returns-its-input.fx`'s thousand uses
  one request still). An asker that answers `` is said not yet. The
  restart's machinery is gone: `drive-answers!`, `drive-needed`, kc's
  `(needs ...)`, the drivers' loops (`entry-answered`, `load-answered!`,
  `given-answered`, the REPL front's).
- **kc's asker is a query.** `phase1 REQUEST` is no input now but a query
  whose body is C448's (the request's phase-1 program a file of the
  database, built and run), asked by the expansion query as it goes, so
  recorded as one of its reads and memoized as any query's value. That
  needs two changes to the database (lib/db.kfx; kc's, and two corpus
  programs', `db` and `db-verify`, whose Rust is re-blessed, their answers
  as they were):
  - `qe` may build and run a program (process, files, `@io`): the
    transformer's run is as deterministic as any body, of its key, the
    request's text, its files named by what they hold;
  - `db-set!` of a key no node is of makes no new revision: no one has
    read it, so nothing made is stale, and the `phase1` body may make its
    file's inputs inside a query without sending every node to be verified
    again. `models/db.qnt` has no such input yet (TODO §51).
  Where it builds and on what runtime are the run's side values
  (`db-side`, `phase1-dir`, `phase1-rt`), which no query records as read.
  The build's helpers (`build`, `phase1-built`, `phase1-ran`) moved before
  `kc-run`, each given the runner.
- **The engine's and the REPL's asker** is the engine run again
  (`phase1-asked`, `--phase1`), its command line now a constant list, so
  that the REPL's front, a process, holds nothing of `@io`; a draft's asker
  answers ``, so typing runs nothing.
- **Two expander files.** The asker's effect is the host's: kc's is `qe`,
  the engine's `(io process)`. One file of a place and an effect, `(mxe
  effect)`, is FX-26 (Fixpt's `made-core.fx` is of a region and an
  effect), and kfx took the file once taught (its maker polymorphic in the
  effect, the effect's binder in scope for the maker's items), but KFX's
  Rust writes no module type with an effect parameter (C24, C170), and an
  instance in a `letrena` saw the effect variable as a suspension. So
  `macros-port.py` writes the expander twice from one text, the asker's
  effect written in each (`ASKS`): `expander.fx` (the engine's, the
  REPL's) and `expander-kc.fx` (kc's), both drift-checked; kfx unchanged.
  Fixpt's `parse-tag` is patched to allow its delimited computations the
  asker's effect (where Fixpt's is its runtime primitive's own).
- **Measured**, an explicit renaming chain (`(chain n)` making
  `(+ 1 (chain n-1))`), cold, `kc --check`: n=25 0.35 s, now 0.21; 50,
  0.87, now 0.25; 100, 3.47, now 0.34; 200, 18.61, now 0.57 (about 2 ms a
  use, a run of the transformer built once). The engine: 200, 5.61 s, now
  5.11, its time the engine's process for each request (in-memory code is
  the step that makes each a call). `kc-steps.py`'s procedural edits:
  cold 4 expansions (6), a use's operand edited 1 (2).
- **Checks**: `kc-expanded.py` as C448's; REPL sessions 14; kc-steps 336
  programs, 0 wrong.
- **The full check**: all passed (the first stopped at the corpus: the
  two programs of lib/db.kfx, re-blessed); the engine's comparisons as
  before (beyond KFX, 411 agree); `macros-port --check` of 4 files.

## C450. kc's own assembler: its objects `kc build`'s, each as `as` makes it (2026-11-17)

The user: "Go ahead" (TODO §52's steps 1-2, kc's code in memory: "surely
it's better to just use the same byte emitter everywhere, as that is the
thing that's most likely to expose bugs").
- **An assembler of kc's text, not instructions as data, first.** The plan
  was x64.kfx's instructions as data, rendered as text and encoded as
  bytes. Taken instead: `kc/asm.kfx`, an assembler of the text x64.kfx
  makes (the `asm-item`s the database keeps), a subset of GNU as's x86-64
  syntax (about 35 mnemonics, `.quad`, `.comm`, `.set`, `.p2align`, five
  sections), made an ELF relocatable object. One byte emitter
  everywhere still, and kc's and `as`'s given the same input, so a
  difference of their objects is the encoder's alone. The text made
  instructions as data is left for later (TODO §52), and changes no byte.
- **As `as` makes it**, settled by trying GNU as 2.38, each rule at the
  file's head: a jump of its own section, local or global, resolved and
  relaxed (short, grown until none changes, every branch measured on one
  layout a pass); a call of a local of its own section resolved, else
  `R_X86_64_PLT32`; a `%rip` operand of a global, common or undefined
  symbol `R_X86_64_PC32` against it, of a local of another section against
  that section, of its own resolved; `.quad` of a symbol `R_X86_64_64`; an
  immediate of 8 bits where it fits, but of a symbol not `.set` yet 32;
  `%rax`'s short forms; `.p2align` in code padded with `as`'s no-ops of
  each length (1 to 11; more, 11 and the rest).
- **`kc build`** writes each new object's text (`NAME.s`, for reading and
  comparing) and kc's object (`NAME.o`); no `cc -c`. `cc` still links.
  `kc --object FILE OUT` assembles a file alone. The bytes go out through
  a bytevector at `@io`, so `qe` (lib/db.kfx) has `(write @io)` too, and
  kc's asker's effect with it (C449's `ASKS`).
- **`scripts/kc-objects.py`**, in the check after kc-steps: each object
  kc-steps' builds made (the step programs', the transformers'), by kc
  and by `as`, compared section by section (`.text`, `.data`,
  `.rodata`'s bytes, `.bss`'s size), relocation by relocation, symbol by
  symbol (but the sections'). Found on its first run: 7 of 459 objects,
  the transformers' (17 KB of code), longer by a few hundred bytes: a
  branch measured against the layout being made, not the one its target's
  offset was of, grew where `as`'s stays short. Now 441 of 441.
- **Measured**: kc-steps cold 78 s (the mean of its warm runs 65); a
  transformer's cold run as before (`contract-defn.fx` 0.18 s, the chain
  of 200 0.54 s): kc's assembler costs what `cc -c` did.
- **Checks**: kc-steps 336 programs, 0 wrong, each native one linked of
  kc's objects; kc-objects 441 objects, 0 differ; kc-expanded as before.
- **The full check**: all passed; kc-objects 459 objects (the check's
  kc-steps builds), 0 differ, in 1.5 s; the engine's comparisons as before.

## C451. kc's code run in process, under kc's own convention (2026-11-17)

The user: "Go ahead" (TODO §52's step 3, kc's own convention: "a stack
limit, a failure that returns to the trampoline, a state register kept
for fuel; linked into kayak-jit's code area, `kc run` in process, each
step program as its executable answers"). Asked before starting, as the
plan's "linked into kayak-jit's code area" assumed a way in that KFX has
not: only FX-26's `seal-code` and `native-procedure` place and call code,
and C116 lets only some values cross. The user chose "Widen the boundary"
over a runtime of scalars alone or a kayak-only primitive. What crosses
already sufficed (C116's note was older than what kayak-rt's `Native`
does): an `i64` flat array, shared (C126), and a procedure of KFX's in a
code bloblet's field, called by `foreign`. So nothing of FX-26 or kayak-rt
changed.
- **kc's convention**: Fixpt's data and Fixpt's state, its calls its own.
  The state in `%r15`, as Fixpt's native convention keeps it; the heap
  moved to `%r12` and `%r13` (was `%r15`, `%r14`). Each function's entry,
  after its frame is made, compares the stack pointer with the state's
  limit (failure 8, `stack overflow`) and takes a unit of fuel, calling
  the state's `poll` when none is left (kayak-jit's gives the next slice).
  A cell (a global, a symbol's cache, `kc_heap_next`) is written through
  the state's code distance (Fixpt's `delta`), as code in the code area
  runs from its executable view. So **the same bytes** run as a built
  program, whose state (kayak-kcrt's, new) says a distance of no pages,
  a limit 1 MB above the end of the program's stack (now a thread of 64
  MB), and fuel with a `poll` of kcrt's; and in process.
- **`kc run FILE`**: the `asm` query's text after `x64-run-support`,
  assembled into one image by kc's assembler (`asm-image`: `.text`, then
  `.rodata`, `.data`, `.bss` and its commons, every relocation resolved),
  sealed (`seal-code`) with two fields, the heap (a flat array of 2^23
  words) and the service, and called (`native-procedure`, the native
  convention): `kc_native_entry` takes the array, writes its own and the
  image's addresses to its first words, and calls `kc_entry`.
- **The runtime in KFX** (`kc/native.kfx`): each of kcrt's procedures that
  kc's code calls is a stub of its number, which calls the service by
  `foreign`, the heap's next word and the arguments in the array's
  words, the answer back from one. The service is kcrt's runtime again,
  in KFX on Fixpt's layout: the printer, `intern`, the string
  operations, `append`, the predicates, the failures. It reads a word
  from the heap or from kc's own copy of the image (constants; it reads
  no cell), and makes objects on the heap from the array's next word.
- **A failure returns to the trampoline**, by kc's convention, not by a
  trap: the service keeps its words and answers 1; the stub goes back to
  `kc_native_entry`'s frame (its stack pointer kept in the array) and
  returns 1 through the trampoline. A KFX `error` would have reached the
  trampoline too (`foreign` carries a panic past the native frames), but
  then nothing catches it: an `attempt`'s body may not write output
  (KFX's), and the program writes as it goes. kayak-jit's own traps end in
  a KFX error, which is why kc does not use them.
- **Found on the way**: the first kc-steps sweep had 160 programs
  failing in process, all of them deterministic per run and none
  reproducible by hand. The step database held `asm-item`s made by the
  kc before the registers moved, as the kc's identity (kc/identity.kfx)
  had not been made again after the edit. This is the standing rule:
  `scripts/compiler-identity.py` after each kfx edit. Once it was run,
  0 programs failed.
- **Checks**: kc-steps 336 programs, every one run in process too, its
  output, its failure and its status the executable's (336 of 336); and a
  convention check, a recursion past the stack's limit and a loop of
  3,000,000 calls (`poll` called twice), built and in process. A warm
  `kc run` of a step program: 0.01 s. `kc --image FILE OUT` writes the image
  of an assembly file, for looking at it.
- **The full check**: all passed in 373 s (the first try stopped at its
  start: `cargo fmt` changed kcrt's `lib.rs`, one of kc's identity's
  sources, after the identity was made; identity, regen, fmt, in that
  order, and the identity checked after). kc-steps 336 of 336 in process;
  kc-objects 463 objects, 0 differ.
- **Measured** (best of three, after the commit, for to-discuss item
  14): startup 3 ms built, 24 ms `kc run`; kc's code alike in both (an
  arithmetic loop of 20M calls, 77 and 65 ms); a near-empty runtime call
  (`string?`) about 4 ns built and 32 ns through `foreign`; `string=?` of
  8 characters 65 ns in kcrt's Rust, 630 ns in the KFX service (its
  strings made lists of characters), and 49 ns written in FX and compiled
  by kc.
- **Left** (TODO §52): a transformer's program in process (step 5) needs
  `kc_apply_entry`'s absolute address, which `asm-image` refuses; the heap
  is a flat array never collected (step 4); kcrt's runtime is written
  twice, in Rust and in KFX (docs/to-discuss.md item 14); `poll` only
  gives fuel, no scheduler takes the slice.

## C452. kc's runtime once, kcrt's, reached in process by a syscall (2026-11-17)

The user, on C451's second copy of kcrt in KFX: "didn't the old Kayak
code base already use syscalls? Did you not review what came before when
you started this?" It did (C224, C225): a runtime's program
(`--syscalls`) declares `define-syscall`s, which kayak-rt answers in Rust,
each a lang item (C367). C451's premise, that only `seal-code` and
`native-procedure` reach native code from KFX, was wrong, and so was its
question to the user. Then: "When starting a new step and before claiming
something cannot be done ... review the Fixpt code base, review the Coopt
code base, review the Accomplice code base, and review the previous Kayak
work." Done for this step:
- **Fixpt**: its native code calls its Rust `extern "C"` call-outs
  indirectly, through addresses the runtime puts in the state
  (`fixpt-native`'s `direct.rs` `callout`, `cellular.rs`'s `pure_call`,
  `prim_call`), never an address baked into the code as an immediate; a
  call-out that fails keeps its message (`LAST_MESSAGE`), sets the trap,
  and the code goes to its trap stub: a failure of the form, not of the
  process. Syscalls name a runtime's own operations in its namespace
  (`kayak:`, docs/research/syscalls.md).
- **Accomplice** (Larceny): millicode called through a table of function
  pointers in the globals array, held in a callee-saved register
  (`call [GLOBALS+offset]`); an error a callout to a Scheme handler by
  longjmp to the entry's setjmp. Its measured cost is the whole context
  saved at each call (its fast paths moved inline).
- **Coopt**: no native code yet; a failure a `Result` returned, never an
  exit nor an unwind across machine code.
- **kayak**: kayak-rt's syscall table (`kayak:call-native/1` and others),
  the units that say `--syscalls` (fx26, cells), lang/items.sexp.

What changed:
- **kayak-kcrt-core**: kcrt's routines, apart from a built program's
  `main`, its state and `poll` (kayak-kcrt, which re-exports them, its
  static and shared libraries' symbols as before). A crate, not a cargo
  feature: features are unified across a build, and kc must not get
  `main`. Each routine that may fail has a variant that keeps its words in
  a thread-local and answers 0 (`here_…`), as Fixpt's call-outs do.
- **Two syscalls** of kayak-rt's, lang items: `kayak:kc-routine/1`, a
  routine's address by the name kc's code calls it (`kc_show`, ...), the
  variant of one that may fail; `kayak:kc-failure/1`, the words of the
  last failure, taken.
- **kc a runtime's program** (kc.unit's `--syscalls`). `kc run` asks each
  routine's address and writes them as a table in the image
  (`kc_routines`); each stub jumps through its cell, as Larceny's code
  through its globals and Fixpt's through its state; one that may fail
  calls through it and, on 0, goes to `kc_failed` (C451's return to the
  trampoline).
- **kc/native.kfx's service is gone**: the printer, `intern`, the string
  operations in KFX. What is left: the syscalls' declarations, the heap
  array, sealing and calling. to-discuss item 14 answered.
- **The full check**: all passed in 404 s; kc-steps 336 of 336 in
  process; kc-objects 463, 0 differ; lang-items 19 syscalls, every copy
  agrees; nextest 5500 passed.
- **Measured** (best of three, `kc run`; built for comparison): `string?`
  1M calls 32 ms (C451's KFX service 56, built 7; kc's startup about 24
  of it); `string=?` 1M 100 ms (651, built 68); `string-append` 300k 77
  ms (324, built 99).

## C453. kc's collector: Cheney's, kcrt's, its roots kc's stack read whole (2026-11-17)

The user: "Go for it" (TODO §52, step 4: a heap of Fixpt's layout, then
the collector). Reviewed first, as the user asked (C452):
- **Fixpt** (the pin): its native code bumps inline against the state's
  `alloc_limit`, and past it calls out; a collection runs only inside a
  call-out, never in the line. Roots by precise stack maps (a map word at
  fp+16, values from fp+24, written before each call that may collect);
  before a collection every frame made a `native-frame` bloblet in place
  (the stack lies in the nursery), moved, then the innermost copied back,
  the rest on return (Larceny's flush and underflow). Cards and a
  generation table for its barrier. Its regions: arenas (never moved,
  reused when they end) and reaps (copied, an ended one's chunks
  quarantined until a collection finds nothing in them). The collected
  code area: code bloblets marked, never moved, swept to a free list.
- **Accomplice** (Larceny): no stack maps and no conservatism: every
  word of the Scheme stack a tagged value, return points padded so that
  each return address reads as a fixnum; registers saved to the globals
  at a millicode call; the stack flushed to heap frames before a
  collection. Two barriers (SATB, and generational to an SSB).
- **Coopt**: no native code; `Root` handles, the interpreter's whole
  state in them; arenas (`letrena`) dropped whole and quarantined.
- **kayak**: tier 1's collector in KFX (`cells/heap.kfx`, Coopt's
  baseline, Cheney's, of Fixpt's layout) with the code area marked and
  swept (C226, C229); kayak-jit's `traced_slots` walks Fixpt-shaped
  frames by their maps (C229).

What was done, kc's code needing neither maps nor a flush:
- **The collector** (`crates/kayak-kcrt-core/src/gc.rs`): Cheney's, two
  spaces, of Fixpt's layout, as tier 1's and Coopt's baseline (a
  forwarding word, tag 7, in a pair's first word, a bloblet's header, and
  the word below its suffix when that is no trailer). In Rust, in
  kcrt-core: one copy, a built program's and `kc run`'s (C452's reason;
  the user: "Why are we writing the runtime itself in KFX instead of
  Rust?"). A space grows (doubled, collected again) when what lives and
  what is wanted fill more than half; past 2^31 words, failure 5, now
  "the heap past its most".
- **The roots, Larceny's way, by Fixpt's tags**: every word of kc's stack
  from the caller's pointer to the base its entry kept, each a value of
  Fixpt's format, or something no pointer into the space (a return
  address, in the code): only a word that points into the space being
  collected is followed. So each frame is zeroed when made (after the
  entry's checks), and a value a register holds where an allocation does
  not fit is pushed for the collector (`cons`'s car and cdr;
  `make-array`'s value, its length made a fixnum while it waits). The
  globals' cells, one run in the entry object (`kc_cells_lo` to
  `kc_cells_hi`), forwarded in place; the pairs kcrt makes in its own
  memory (`append`), never moved, their words roots.
- **kc's code**: each allocation's check, past the end, calls
  `kc_gc_call` out of the line and checks again (in place of failure 5);
  `kc_gc_call` gives kcrt's `kc_collect` the bytes, the caller's stack
  pointer and the block (`kc_gc`: the space, the stack's base, the cells'
  run, as writable addresses, so that the same bytes run in process),
  and makes `%r12`/`%r13` the heap it gives back. In process
  `kc_collect` is a routine that may fail (C452's table). `kc_call2`
  takes the heap's end from the block.
- **`kc run --heap WORDS FILE`**: the first space that size.
- **Tests** (scripts/kc-steps.py): every program in process again with a
  heap of 64 words, so that nearly every allocation collects, its output
  and status the executable's: 336 of 336. And `collected`, built and in
  process: 20000 rounds of a list of 1000, an array of 50 and a closure
  (some 330 MB made), a global's list and a global's closure kept across;
  built, 0.32 s and 132 MB at most (two spaces of 64 MB); in process with
  1024 words, 21 MB.
- **The full check**: all passed in 463 s; kc-steps 336 of 336 in
  process, and 336 of 336 with a heap of 64 words; kc-objects 465, 0
  differ; lang-items 19 syscalls, every copy agrees; nextest 5500 passed.
- **Measured**: zeroing frames, on a loop of calls and nothing else (20M
  tail calls and 20M non-tail), 0.11-0.13 s against 0.11 s with the zeroing
  lines taken out of the same assembly (best of five).
- **Left** (TODO §52): the region a transformer's call drops (with step
  5, where there are calls to drop it after); kcrt's own objects (strings,
  symbols, `append`'s pairs) never freed; kc's code never freed (the code
  area's sweep, with step 5's images); no generations, so no barrier; a
  large bloblet (more than 2^18 fields) kc's `make-array` cannot make
  yet, and the collector refuses one; tier 1's collector and this one two
  copies of Cheney's over Fixpt's layout, KFX's and Rust's.

## C454. A transformer called in process: its image sealed once, no process (2026-11-17)

The user: "Step 5, go for it" (TODO §52: `phase1`'s body a call of the
compiled transformer, no process). Reviewed first:
- **Fixpt**: its expander calls a transformer in its own process, a
  procedure of the running machine (`phase1.rs`'s `call_transformer`),
  its use given as data and its answer read back; kayak's engine runs
  `--phase1`, a process of its own (C448).
- **Accomplice** (Larceny): macros expand by `eval` in the running
  heap, no process.
- **Coopt**: no native code, no transformers of its own.
- **kayak**: C451's `kc run` (an image sealed and called), C452's
  syscalls to kcrt-core, C453's collector; C448's `--apply`, a built
  transformer given the tag and the use on its standard input, its answer
  its last line.

What changed:
- **A transformer's image sealed once** (`phase1-sealed`, kc/main.kfx):
  its program loaded as a file of the database, checked, its `asm` made
  an image after `x64-run-support` and sealed in the code area, kept by
  its name (the fingerprint of its text) for every use after; each use a
  call (`native-call`). No executable, no `DB.phase1/`, no process, no
  link: `phase1-dir`, its orphans and the database's side values are
  gone; `kc --rt` is `kc build`'s alone.
- **The call, as `--apply`**: KFX gives the tag and the use
  (`kayak:kc-give/1`) and puts 1 in the heap's word 5; `kc_native_entry`,
  after `kc_entry`, calls `%phase1-apply` (`kc_call2`) on the use and the
  tag (`kc_given`, strings kcrt makes) and writes its answer (`kc_show`).
  In process `kc_show` keeps each value's line (`kayak:kc-output/1` takes
  them): `kc run` writes them, a transformer's answer is the last.
- **No absolute address**: `kc_apply_entry`, a word holding
  `%phase1-apply`'s address (abs64, which `asm-image` refuses), is now
  `kc_apply_address`, a function giving it by `leaq` relative to the
  code; kcrt's `--apply` calls it. Failure 9, "this program has no
  transformer to apply", kcrt's words.
- **The region a call drops**: one heap array of each size, every run
  given it again from its start (kc/native.kfx's `heap-of`); at each
  run's start the collector's own spaces and kept pairs dropped
  (`kc_gc_reset`), so nothing of the run before is reached or kept.
- **The effects**: `qe` (lib/db.kfx) and kc's expander's asker
  (`macros-port.py`'s `ASKS`, fx26/macros/expander-kc.fx) allow `(io
  native)` and `(io sys)`.
- **The corpus**: lib/db.kfx's two programs (`db`, `db-verify`) re-blessed,
  their Rust's doc comment alone new (the first check stopped there).
- **The full check**: all passed in 398 s; kc-steps 336 of 336 (in
  process, and with a heap of 64 words); kc-objects 431, 0 differ;
  lang-items 21 syscalls, every copy agrees; REPL sessions 12 as before;
  nextest 5500 passed.
- **kc-objects** compares 431 objects, was 465: the 34 were transformers'
  programs' objects, which are no longer made.
- **Measured**: C449's explicit renaming chain, cold, `kc --check`, best
  of three: n=25 0.16 s (C449: 0.21), 50 0.17 (0.25), 100 0.17 (0.34),
  200 0.20 (0.57), 400 0.29: about 0.35 ms a use, was about 2 ms (a
  process each). kc-steps 144 s, was 198.
- **Left**: a transformer's run is no longer a process of its own, so a
  fault in its code is kc's (fuel is counted, but no scheduler takes a
  slice); kc's images never freed (code that nothing reaches, Fixpt's
  collected code area); kcrt's own objects never freed.

## C455. What kcrt makes for kc's code, made in kc's heap and collected (2026-11-17)

The user: "Address these", of two gaps C454 left: kc's sealed images
never freed (C456), and kcrt's own objects never freed (this entry).
Reviewed first:
- **Fixpt**: its runtime's routines make what they make in the heap the
  code runs on (`fixpt-heap`'s `make_string`, a pair the `cons` routine
  makes), collected as the code's own are; a routine that may collect is
  a call-out, at which the frames are flushed into the heap. Its symbols
  are its heap's, held by its symbol table for the heap's life
  (`heap.rs`'s `intern`, `Which::Symbols`): never freed while it runs.
- **Accomplice** (Larceny): the runtime's allocations are the mutator's,
  in the same heap; a C routine that allocates is given the heap's
  pointers and hands them back.
- **Coopt**: its runtime objects are `Root` handles into its heap, which
  its collector moves.
- **kayak**: kayak-jit's `HOST` heap (tier 1's, C229): a routine making
  objects in a heap its caller gives, its next free address handed back;
  C453's `kc_gc_call`, the stack the roots from a stack pointer the call
  gives.

What changed:
- **kcrt's strings and pairs in kc's heap**: `string-append`,
  `substring`, a `,@`'s pairs (`kc_append`) and a transformer's given use
  and tag (`kc_given`) are made in kc's heap, collected with kc's own
  objects; the pairs kcrt kept as roots (C453's `kept_pair`) are gone.
- **How a routine allocates**: kc's code calls `kc_string_append` as
  any routine; that is now a wrapper of the entry object's
  (`x64-heap-routine`, x64.kfx) which puts the arguments in three stack
  words of its own (a 0 for each not given), the heap's next free word
  and that stack pointer in the collector's block (two new words, 5 and
  6), and calls kcrt's `kcrt_…` on the words and the block. The routine
  takes its room by `gc::words`, which collects if it does not fit, the
  roots from that stack pointer up, the arguments among them; it reads
  its arguments after it has its room. An append takes the room for all
  its pairs at once. The wrapper makes the block's next word and end
  `%r12` and `%r13`. kc's code keeps no value in a register across a
  routine's call (its frames' slots hold them), so the stack is the whole
  of what lives.
- **One transformer's run, built and in process**: `kc_apply_run` (in
  both entry objects) does what `kc_native_entry` did after `kc_entry`
  (`%phase1-apply` on the use and the tag, its answer written), the first
  string kept in a stack word above a base of its own while the second is
  made. A built transformer's `--apply` calls it after `give`, so it no
  longer makes its strings in kcrt's memory; tested by hand (an apply,
  and failure 9 of a program with none), as no check runs a built
  transformer since C454.
- **Symbols stay kcrt's**: interned once a run, each with its name, as
  Fixpt's symbol table keeps its own; kc's code keeps each symbol it
  interns in a cell of its name's (`kcy_…`, `.comm`, outside the
  globals' run), which a moving symbol would leave behind.
- **A test**: tests/kc/step15/made-collected.fx, 300000 rounds of a
  substring, a string appended and two `,@`s, past a heap of 64 words
  (kc-steps' every way: built, in process, `--heap 64`, and Fixpt's
  answer). In process with a heap of 64 words, 18.6 MB at most, 0.19 s;
  before, kcrt's memory would have kept every round's, about 50 words a
  round by their layout, some 120 MB.
- **The full check**: all passed in 453 s; kc-steps 337 of 337 (in
  process, and with a heap of 64 words); kc-objects 433, 0 differ;
  nextest 5500 passed.

## C456. kc's sealed code freed: the code area's room given back, first fit (2026-11-17)

The user: "Address these" (with C455): kc's sealed images never freed.
Reviewed first:
- **Fixpt**: "A collected code area" (`docs/object-model.md`; `PLAN.md`
  steps 1-4): code bloblets in a fourth part of the heap, taken first fit
  from a free list, never moving, marked within the copying collection
  and swept, each unmarked block made free and joined to the free ones
  beside it; tables that name code cleared when it is freed. Its test
  (`fixpt-native/tests/code_gc.rs`) generates, runs and drops code past
  one space's room.
- **Accomplice** (Larceny): code is a bytevector in the heap, collected
  as any object.
- **Coopt**: no native code.
- **kayak**: kayak-jit's code area (C116), a bump allocator, never freed,
  as kayak-rt collects nothing (`alloc` is `Box::leak`); FX-26's regions
  that end (`letrena`, `letreap`, a process's reap) end with a body or a
  process, not when a cache drops an entry; a syscall takes only
  integers, strings and the like (KFX refuses a bloblet), and
  `kayak:last-sealed/1` (C239) gives where code sealed last runs.

So the area is Fixpt's but for what decides what is freed: kayak-rt
traces nothing, so the program says it, and a check makes saying it
wrongly a failure rather than a fault.

What changed:
- **The code area's room** (kayak-jit): taken first fit from the blocks
  given back, else past the top; `free(entry)` gives a block back,
  joined to the free blocks beside it, the top lowered if it ends there.
- **A generation per block**: a table beside the area, a word for each
  16-byte granule (reserved whole, nothing committed until touched),
  holds the generation of the block starting there while it lives;
  `live(code)` is one load. kayak-rt checks it at each call of a native
  procedure, at `native-procedure`, where a sealed bloblet crosses, and
  before a field written is written beside its code: freed code fails,
  "native code: its code was freed", and is never run, nor a reused
  block written through an old bloblet.
- **`kayak:unseal/1`** (`(subr (io sys) (int) unit)`, lang/items.sexp's
  22nd syscall): code freed by where it runs; an address of no live code,
  nothing done.
- **kc's images**: each keeps where it runs (`last-sealed` after its
  `seal-code`), and `native-freed` frees it. `kc run`'s image is freed
  after its run. A transformer's uses' image is kept, the most recently
  used first, at most 16 (`transformers-most`); the least recently used
  freed past them, sealed again if wanted again. A program run once (a
  macro's definition's, a use its call is a form of) is sealed, run and
  freed (`phase1-once`), so the 16 are all of uses.
- **Tests**: kayak-jit's `tests/code_free.rs`, 600 rounds of 1 MiB of
  code sealed, run and freed (past twice the area's 256 MiB), the top as
  it was after, each freed `live` no more and not freed twice; blocks
  freed joined and used again, first fit. tests/kc/step15/
  procedural-many.fx, 18 procedural macros, the first used again last:
  in process 38 images sealed and 22 freed (counted once by hand, with a
  log since removed: 18 definitions, 19 uses' images, `kc run`'s own;
  3 evicted).
- **The full check**: all passed in 522 s; kc-steps 338 of 338, 238 s
  (was 169: procedural-many's 18 transformers compiled cold in each of
  its ways, 9457 query bodies, was 6786); kc-objects 434, 0 differ;
  lang-items 22 syscalls; nextest 5501 passed; selfhost-cells 145 s, as
  before (the check at each native call no cost seen).

## C457. The expander's state in kc's heap: the survey of what kc lacks (2026-11-17)

The user, after C456: "I believe I do want the expander's state in the
collected heap", and "work via those steps" (TODO §54). Step 1, the
survey. Reviewed first:
- **Fixpt**: its expander runs on the machine its transformers do
  (`phase1.rs`'s `call_transformer`, a procedure of the running heap),
  its environments heap objects, a transformer a closure; its code in the
  collected code area, freed with its binding.
- **Accomplice** (Larceny): macros expanded by `eval` in the running
  heap; the expander is heap code like any.
- **Coopt**: no expander of its own.
- **kayak**: kc's expander is Fixpt's, ported (`macros-port.py`, C429),
  a file of a place (C442), loaded by kc's `kc-instance` under a
  `letrena` and compiled by kfx into kc; kc has never compiled it itself.

How: the expander made one file (its types file inlined, `mxp` made
`@io`, the items a `module`'s, so that naming one reads no global, as in
the place's file), then `kc --check` and `kc --asm` of it. The checker
takes it whole. The code generator stops at its first gap, so for the
survey alone a build of kc whose `not-yet` said its words and went on,
and whose `asm-item` query caught its item's failure (the generated Rust
edited by hand, then restored from git, kc rebuilt): every item's gaps
in one run, 335 of the 1664 definitions refused. Each failure that was
not a `not-yet` (126 `car or cdr of a non-pair` in kc's x64, and "no
primitive" for 189 more) came after a `not-yet` in the same item: what
the survey's going on made, not gaps of their own.

The gaps, sized, are TODO §54's list: tuples (1484), `reverse` and
`append` as procedures (844), delimited control (117, the parser's
backtracking), `i64` (111), strings and characters (183), more than six
parameters or arguments (58), type-level names reached as values (228,
the form to find first), `f64`, a vector, a file of a place. What runs
is not surveyed: an `int` past a fixnum, the hashes, the expander's
host (its asker, the bridge, files).

What the plan's order makes of it: step 2 is most of the work; tuples,
list procedures, `i64` and strings are kc's steps of the kind done
before, delimited control the one new mechanism (Fixpt's native
convention has it, prompts and marks frames of known kinds a walk of the
stack finds, `docs/research/native-conventions.md` step 5; kc's own
convention has none yet). Incremental collection,
whether in the plan or after: to-discuss 16.

## C458. Delimited control in kc: prompts as frames, aborts that walk them (2026-11-17)

The user: "Start with delimited control in `kc`" (TODO §54, step 2; the
expander's parser backtracks by it: one prompt tag, `parse-tag`, 14
prompts, 4 aborts, nothing captured). Reviewed first:
- **Fixpt**: FX-26's typed control (`docs/fx26.md`, "Control, typed"):
  `(prompt tag body handler)`, syntax; `abort-current-continuation` of
  `(goto R)`; an escape-only tag a prompt tag at run time. A tag is a box
  of unit (`cellular.rs`: `(%make-box #u)`), compared by `eq?`; a prompt
  is `(%prompt tag handler thunk)`, so the tag is made first, then the
  handler, then the body runs. Its native convention keeps prompts and
  marks as frames of known kinds a walk of the stack finds
  (`docs/research/native-conventions.md`, step 5); an abort cuts to the
  innermost for its tag and calls the handler there; none, "abort: no
  prompt for this tag" (`fixpt-native`'s `control.rs`), the cellular
  machine's words too.
- **Accomplice** (Larceny): continuations by its stack cache's segments;
  an escape is a jump to a saved frame.
- **Coopt**: nothing of delimited control in its runtime.
- **kayak**: kayak-rt's escapes are `Result`s each call checks (C18,
  `escape.rs`): a cost at every call, which kc's convention need not pay.

What changed:
- **l0** (kc/l0.kfx): `(prompt TAG HANDLER BODY)`, in the order made;
  `make-continuation-prompt-tag` and `make-escape-tag` a fresh ref of
  unit (Fixpt's box, kind 10); `abort-current-continuation` and `escape`
  binary primitives.
- **x64**: a prompt's record, four of its frame's slots: the prompt
  before it, the tag, the stack pointer, and its handler's call's
  address; the innermost prompt's record in `kc_prompts` (a cell of the
  entry object, `.globl`, none at each `kc_entry`). Its body runs not in
  tail position, and the record is taken off after. An abort calls
  `kc_abort`, which walks the records for the tag by `eq?`, takes that
  one and those inside it off, makes the stack pointer the record's and
  jumps to the handler's call with the value; the handler is called in
  the prompt's continuation. No record of the tag: failure 10, Fixpt's
  words. Nothing at a call: no check after a return, no frame of a call
  that does not prompt.
- **The collector**: each word of a record is a stack address (a
  fixnum's tag), a value (the tag, a box the collector moves, the
  handler), or a code address outside the heap, so the stack's scan
  (C453) wants nothing new.
- **Tests** (tests/kc/control/, kc-steps' every way and Fixpt's answer):
  `prompts.fx` (a body's value, an abort from 1000 calls deep, the
  innermost prompt of a tag, an abort past another tag's prompt, one
  from a closure, 2000 rounds each aborted); `prompts-heap.fx` (the
  expander's way, a datatype through a global tag; a body that collects
  under its prompt, with a heap of 64 words, its abort's payload a list;
  an escape tag); `abort-unprompted.fx` (no prompt of the tag, past
  another's). Built too, and, by hand, 1000000 rounds each aborted from
  50 deep in 0.19 s.
- **The survey again** (as C457, a build of kc that goes on past
  `not-yet`): the expander's prompts and aborts all lowered; 320 items
  refused, were 335. That build's generated Rust was put back from git,
  which, after this step's regeneration, took the step's code with it:
  regenerated again (a memory now says to copy them aside).
- **Left**: `call-with-composable-continuation`,
  `call-with-oneshot-continuation` and marks (`with-mark`, ...), which
  copy a stack's segment as Fixpt's native convention does (the
  expander uses none); `letregion` (kc has none yet, so a tag of a
  region a `letregion` makes is not yet kc's); kc's Rust back end, which
  has no prompt.
- **The full check**: all passed in 589 s; kc-steps 341 of 341 (in
  process, and with a heap of 64 words); kc-objects 439, 0 differ;
  nextest 5501 passed.

## C459. Capturing continuations in kc: the frames to the prompt copied into the heap (2026-11-17)

The user: "No let's go ahead and get capturing continuations in `kc`
too" (after C458; before tuples). Reviewed first:
- **Fixpt**: its native convention takes a continuation by copying the
  frames up to the prompt (`call-with-composable-continuation`) or the
  stack's top (`cwcc`) into a vector on the heap, links as offsets,
  return addresses as fixnums; called, the frames go back, a delimited
  one on top of its caller (`docs/research/native-conventions.md`,
  "Continuations" and step 5). A prompt's body is compiled as a closure of
  no arguments, so that what is inside the prompt is above its marker
  (`compile.fx`'s header). A one-shot continuation's second resume fails,
  "a one-shot continuation resumed twice" (`docs/fx26.md`); a capture of
  no prompt, "call-with-composable-continuation: no prompt for this tag"
  (`fixpt-native`'s `control.rs`), the cellular machine's words too.
- **Accomplice** (Larceny): its stack cache, frames copied to the heap
  when taken, frames live on the stack until then; Fixpt's design is
  that.
- **Coopt**: none.
- **kayak**: C458's prompt records; kc's closures (C412) and the
  collector's scan (C453), which follows only words into the space it
  collects, so a return address in a copy is no harm.

What changed:
- **A prompt's body a procedure** (l0): `(lambda () BODY)`, called from
  the prompt's frame after its record is made, so that everything inside
  the prompt is in frames above it, the return to the prompt just below
  the record's stack pointer. Aborts are as they were.
- **Capture** (`call-with-composable-continuation`,
  `call-with-oneshot-continuation`, binary primitives): `kc_capture`
  (x64.kfx's entry support) puts the procedure, the tag, the innermost
  prompt, `kc_resume`'s address and the once-only flag in stack words of
  its own and calls kcrt's `kcrt_capture`, which finds the tag's prompt,
  takes room for the copy and its closure at once (collecting if need
  be; the stack's words moved, so read after) and copies the segment,
  from the capture's return to the prompt's, into a bloblet of kind 32
  with a trailer (its words: the segment's bytes, the innermost record's
  offset, the flag, the words), each prompt record's stack addresses
  inside made offsets; the closure (of one free value, the copy) is of
  `kc_resume`. The procedure is then called on it in tail position.
- **Resume** (`kc_resume`, the closure's code, aligned so its address
  reads as a fixnum): the copy's place is just below its caller's
  return; past the stack's limit, failure 8; the stack pointer moved
  below that place, kcrt's `kcrt_resume_copy` puts the words there, its
  records' offsets made addresses there, the outermost's link the
  current innermost prompt, and gives the innermost record (plus 1, as 0
  says a failure in process); then the capture's return taken with the
  value. The copy's last return is the resume's caller's: composition.
  Once-only resumed again: failure 12.
- **Tests** (tests/kc/control/, kc-steps' every way and Fixpt's answer):
  `captures.fx` (resumed twice; once only; from 100 frames deep, twice; a
  prompt inside the copy, relinked at each resume, one resume aborting to
  it; frames holding lists, collected with copies live under a heap of
  64 words); `oneshot-twice.fx`, `capture-unprompted.fx`, the
  failures, in Fixpt's words. By hand, 100000 captures 50 deep each
  resumed twice: 0.21 s (0.10 s with a heap of 64 words).
- **Left**: `cwcc`, a whole continuation (the stack to `kc_entry`'s
  base, which resuming replaces); marks (`with-mark`, `first-mark`,
  `current-marks`, `marks-of`), as frames of their own; a copy past
  262140 words (a large bloblet, which kc's collector has not yet);
  kc's Rust back end.
- **The full check**: all passed in 619 s; kc-steps 344 of 344 (in
  process, and with a heap of 64 words); kc-objects 443, 0 differ;
  nextest 5501 passed.

## C460. Whole continuations and marks in kc (2026-11-17)

The user: "cwcc and marks" (after C459). Reviewed first:
- **Fixpt**: `cwcc` takes the frames to the run's base; called, the copy
  replaces the stack and returns where the current run does
  (`docs/research/native-conventions.md`, step 5); its tests keep one and
  call it after `cwcc` has returned (`native/whole-captures.fx`). Marks
  (`docs/fx26.md`, "Control, typed"; KFX's `check-defs.kfx` types):
  `make-continuation-mark-key`, `with-mark KEY VALUE THUNK`, `first-mark
  KEY DEFAULT`, `current-marks KEY` (innermost first), `marks-of K KEY`
  (a composable continuation's). A mark is a frame of its own around the
  thunk's call; in tail position the native code pushes it and
  tail-calls the thunk, which returns through a stub that pops it
  (`mark_ret`); returning there already, under a mark of the same key,
  its value is replaced, so a loop that marks each round runs in
  constant space (`run/tail-marks.fx`).
- **Accomplice** (Larceny): `call/cc` by its stack cache; no marks.
- **Coopt**: none.
- **kayak**: C458's records and C459's copies, which take any record on
  the chain.

What changed:
- **A run's frame** (l0): each top-level form's expression, a
  definition's value too, is a call of a lambda of no parameters, so
  that its run is a call of its own from `kc_entry`'s frame, as Fixpt
  runs each form; `kc_run_base` (a cell) holds that frame's stack
  pointer, `kc_entry`'s, or `kc_call2`'s for a transformer's call.
- **`cwcc`** (a unary primitive): `kc_capture` with flag 32 and the run
  base for a tag; the copy is of the frames up to the run's return.
  Resumed (`kc_resume`, by the flag), the copy goes below the current
  run's return, whatever frames are there given up (the stack pointer
  moved below both it and them for kcrt's copy), its records the whole
  chain; so it returns where the current run does.
- **Marks**: a record of five words on the prompts' chain (the record
  before, the key, its own address, the value, a word for alignment);
  keys are boxes as tags are, so a walk for one never meets the other.
  `with-mark` (ternary) calls `kc_with_mark`, or, in tail position, gives
  its frame back and jumps to it: if its return is `kc_mark_ret` and that
  mark's key is this, the value replaced; else a record pushed with
  `kc_mark_ret` as the thunk's return, which takes it off. `first-mark`
  walks the chain (`kc_first_mark`); `current-marks` and `marks-of`
  (`kcrt_current_marks`, `kcrt_marks_of`) make the list, from the chain
  or from a continuation's copy, whose records C459 already keeps. An
  abort past a mark takes it off with the frames it cuts.
- **Tests** (tests/kc/control/, kc-steps' every way and Fixpt's answer):
  `marks.fx` (a loop marking each of 20000 rounds, its one mark; tail
  marks nested; marks not in tail position, each kept; `first-mark`
  outside, inside; a mark under an abort, gone; `marks-of` of a
  capture), `whole.fx` (Fixpt's `whole-captures.fx` on one machine:
  thrown through frames, from 200 deep, through a prompt; kept, called
  again after `cwcc` returned, three times round). Two of the values I
  first stated were wrong; Fixpt's are the ones stated, and kc gives
  them. By hand, the marking loop of 1000000 rounds: 0.07 s, 77 MB (a
  mark a round would take 48 MB of the 64 MB stack).
- **The first check failed** at kc-steps' cutoff: a program `42`, cold,
  ran `form` and `interface` too. The run's lambda was first written as a
  form, `((lambda () E))`, for `l0-of`, which looks a head up as a
  global's name before it sees `lambda` (so asks the database of a
  global `lambda`); now the term is made directly, `(app (lambda N () ()
  T))`, numbered after the form's binders. A lambda a program writes,
  and C459's prompt body, still make that look-up, as before.
- **Left**: a copy past 262140 words; kc's Rust back end's control;
  `letregion`; l0's look-up of `lambda` as a global's name.
- **The full check** (the second): all passed in 590 s; kc-steps 346 of
  346 (in process, and with a heap of 64 words); kc-objects 447, 0
  differ; nextest 5501 passed.

## C461. Tuples in kc, and KFX's tuples inside an unquote (2026-11-17)

The user: "Okay, sure, do boring old tuples" (TODO §54's survey: 1484
`tuple`s, the most of any gap). Reviewed first:
- **Fixpt**: `(tupleof T1 T2 ...)` is `(productof (1 T1) (2 T2) ...)` and
  `(tuple e1 e2 ...)` is `(product (1 e1) (2 e2) ...)` (`docs/fx26.md`,
  the table of types and their values); `extract` reads a field by its
  label, from 1.
- **Accomplice**, **Coopt**: nothing of their own; a record or a vector.
- **kayak**: kc's products (C423), kind 37, their fields by position;
  KFX's checker makes the same rewrite (`check-defs.kfx`'s `imports-in`,
  C393).

What changed:
- **kc's l0**: `(tuple E ...)` is `(product E ...)`, its fields' terms in
  order; `extract` and the types were kc's already.
- **KFX's checker**: `imports-in`, which writes `tuple` as `product`,
  `->` as `subr` and expands imports, returned a quasiquote whole, so a
  `tuple` an unquote held stayed, and was refused, "unbound variable
  `tuple`", where Fixpt takes it (`` `(,(extract (tuple 1 2) 1)) ``). It
  now goes into a quasiquote as the checker reads one (`quasi`): what
  an unquote at depth 1 holds is an expression, expanded; a nested
  quasiquote deeper. Found by this step's test; KFX takes more of what
  Fixpt takes, nothing Fixpt refuses.
- **Tests**: tests/kc/types/tuple.fx (fields of each kind, a tuple in a
  tuple, one made and taken apart by a caller, tuples in a list, a
  datatype's payload, the expander's `(tupleof datum string int)`, and
  20000 made and dropped past a heap of 64 words), kc-steps' every way
  and Fixpt's answer; built too.
- **The survey again** (a build of kc that goes on past `not-yet`, its
  generated Rust copied aside and put back): 110 items refused, were
  320; the counts of what is left (`reverse` 725, `append` 346,
  `int->string` 161, ...) grew, as more of each item is reached.
- **The full check**: all passed in 537.1 s; kc-steps 347 of 347 (in
  process, and with a heap of 64 words); kc-objects 449, 0 differ; the
  fixpoint as before (Fixpt makes the compiler with the checker's
  change); nextest 5501 passed.

## C462. Lists and numbers in kc: `reverse`, `append`, `length`, `int->string`, `i64` (2026-11-17)

The user: "Go ahead" (after C461: the survey's largest gaps then
`reverse` 725, `append` as a procedure 346, `int->string` 161, the
`i64` conversions 85). Reviewed first:
- **Fixpt**: `reverse`, `append` (any number of lists, the last shared)
  and `length` are its runtime's procedures (`prim.rs`); `int->string`
  is `number->string` (`standard.fx`), decimal. An `i64` is the exact
  integer it stands for, a fixnum where it fits (`docs/fx26.md`, the
  fixed widths), and its operations the machine's 64 bits, wrapped.
- **Accomplice**, **Coopt**: list procedures written in their own
  languages; nothing kc's convention can take.
- **kayak**: kc's binary `append` (C447, `,@`'s), and kcrt's routines
  that make things in kc's heap (C455's `x64-heap-routine`).

What changed:
- **l0**: `reverse`, `length` and `int->string` unary primitives;
  `(append L ...)` the binary `append` nested, `(append)` `nil` and
  `(append L)` `L`; `int->i64` and `i64->int` nothing, and `i64+`,
  `i64-`, `i64*`, the comparisons, `i64-quotient` and `i64-remainder`
  the `int`'s primitives. So an `i64` past a fixnum fails in kc, as an
  `int` there does, where Fixpt's wraps at 64 bits (the bit operations
  and the `u64`s not yet: the expander has none).
- **x64**: `length` a loop in line, a non-pair failing as `cdr` does;
  `reverse` and `int->string` kcrt's (`kcrt_reverse`,
  `kcrt_int_to_string`), each with its wrapper, so that the collector
  may move its argument, and its in-process twin.
- **Tests**: tests/kc/lists/procedures.fx (`reverse` of a list and of
  none; `append` of no lists, one, and four, its last list shared;
  `length`; `int->string` of each sign, 0 and the largest fixnum; an
  `i64`'s operations; 2000 rounds of each made past a heap of 64
  words), kc-steps' every way and Fixpt's answer; built too.
- **The survey again** (as C461): 13 items refused, were 110; `not yet`
  met 343 times, was 1701. Left: calls of more than six arguments (39)
  and procedures of more than six parameters (26); the type-level names
  reached as values (`expr` 27, `data` 26, ...); `char->string` 11,
  `datum-list->vector` 7, `string-hash` 6, `f64?` 6,
  `char-whitespace?` 6, `char-numeric?` 5, `symbol-name-hash` 4,
  `f64->string` 4.
- **The full check**: all passed in 637.9 s; kc-steps 348 of 348 (in
  process, and with a heap of 64 words); kc-objects 451, 0 differ; the
  fixpoint as before; nextest 5501 passed.

## C463. kc's incrementality kept checked, and its timings recorded each step (2026-11-18)

The user: "is kc still incremental? Do you have tests ensuring we
aren't undermining that goal for it?", then "Yes" to closing the gaps
found; and "are you measuring timing for these incremental steps? ...
I would like to have some idea of overall trends", and "Is there a
straightforward way to make gathering those timings part of the
default workflow?". Reviewed first:
- **kayak**: kc-steps' `cutoff()` and `expansions()` (C399, C407, C414,
  C445–C449) say exactly which query bodies run after each edit, and
  fail the check otherwise (C460's first check failed by them). But
  their programs are the early steps' (an integer, two procedures, a few
  macros); the sweep's count of bodies run is said, not checked; and
  kc-bench (C414), which times a session of edits against Fixpt, was
  last recorded at C418, 44 commits before: run by hand, and then not.
- **Fixpt**: a core form's keyword at a form's head is that form, a
  global of its name or not (`(define if ...)`, then `(if #t 1 2)` is
  1; `(define lambda ...)`, then `(lambda 4)` is refused as a lambda);
  a program may define `nil`, or `+`, which its uses then name.

What changed:
- **l0**: a head that is a core form's keyword (`lambda`, `let`, `if`,
  `prompt`, ...) is never looked up as a global's name, and nor are
  `#t`, `#f` and `#u`. Each such look-up made the item ask for the name
  (`kc-need`, a restart) and depend on it, so every `lambda` a program
  wrote did; and kc called a global of a keyword's name where Fixpt
  takes the form. `nil` and the primitives' names are still looked up,
  as Fixpt lets a program define them.
- **kc-steps**: after the sweep, each program built again with the
  database it left, nothing changed: no query body may run (348 of 348).
  `cutoff()` grows: a program of keywords and literals, cold, runs what
  `42` does; and four kinds of item since the early steps, each cold,
  the same, another procedure's body edited (that item alone), then the
  kind's own edit: a datatype's variants (a `tagcase` of it lowered
  again and cut off, the datatype's code made again), a module's value
  and a captured global's (that item alone), a prompt's body made an
  abort (that item, and the new name looked up).
- **The timings, each step**: `check-snapshot.sh`, when a full check
  passes, runs kc-bench on the snapshot (`--rows`, some 35 s; it gates
  nothing, a failure said), and when the next check begins appends the
  rows to bench/kc.tsv under the commit made of that snapshot
  (`--tag`), its hash final by then, after any amend. So each step's
  rows go in with the step after it, and every step has them.
- **The trend since C418** (kc-bench by hand at C462, recorded): the
  same bodies run at each step, none beyond the edit's need; at 1000
  procedures, `body` 405 → 301 ms, `add` 411 → 300 ms, `serve`'s `body`
  249 → 167 ms; cold 827 → 819 ms. And after this step's l0 change, a
  cold 100 procedures runs 314 bodies, was 316.
- **The full check** (the second; the first stopped at the warm pass,
  whose call of `ran` a counter of that name hid, now `bodies_ran`):
  all passed in 596.2 s; kc-steps 348 of 348, each built again warm, no
  body run (kc-steps 402 s, was 345: the warm pass and the new cases);
  kc-objects 451, 0 differ; nextest 5501 passed; kc-bench's rows kept.

## C464. More than six arguments in kc, and `string-append` of any count (2026-11-18)

The user: "Go ahead" (after C463: the survey's calls of more than six
arguments, 39, and procedures of more than six parameters, 26); and,
while it was made, "Is kc_args gc managed? Does it work in presence of
continuation capture? Does it handle var args?". Reviewed first:
- **Fixpt**: register code, Larceny's convention, passes REG1…REG7 and
  a list of the rest in REG8, which the callee takes apart, a `cons` per
  value past the seventh at each call (`docs/performance.md`, "More than
  8 values in register code"). FX-26's variadic procedures (`vsubr`,
  `vlambda`, `apply`; `docs/fx26.md`, "Variadic procedures") pass a
  count at every call, Larceny's way; `list`, `append` and
  `string-append` are standard `vsubr`s, a call of any count the
  runtime's primitive.
- **kayak**: kc's convention is its own (only formats must be Fixpt's):
  six registers; its entry's checks (the stack's limit, `poll`'s fuel,
  every register kept, built and in process).

What changed:
- **x64**: arguments past the sixth (to 64) in `kc_args`, a run of words
  of the image's, the seventh first: the caller stores them, after
  every argument is made, through the writable view; the callee, a
  procedure or a lambda, copies them into its frame's slots on entry,
  after its registers'. Not the collector's: nothing between the stores
  and the copy collects (the entry's checks fail or give fuel), so they
  are no roots, and what is left there after the copy is never read. No
  room on a caller's stack, so a tail call of more than six is as one of
  fewer. A capture is a call out of a body, after the copy, so a
  continuation's frames hold the arguments, and its call is of one
  value; an abort leaves from a body too. One image's run at a time, as
  its other cells (`kc_prompts`, the heap) are already.
- **l0**: `string-append` of any count, as Fixpt's `vsubr`: `""` of
  none, a fresh copy of one, more nested.
- **Not yet**: `vlambda`, `vsubr` and `apply`, the variadic procedures
  a program writes; kc refuses them (the expander has none). A `vsubr`
  is a type of its own, so each call of one is known to be one; a count
  there alone, or the list made by the caller, is to choose then.
- **Tests**: tests/kc/step9/many-args.fx (eight, called and in tail
  position; a loop of nine, each round a tail call; recursion of seven;
  a lambda of ten; a closure over the seventh; arguments that allocate,
  past a heap of 64 words; twenty); tests/kc/control/many-args-captured.fx
  (a continuation taken ten calls of eight deep, resumed twice, each
  frame's seventh and eighth read after; a continuation called by a
  lambda of seven); tests/kc/lists/string-append.fx. Fixpt's answers;
  kc's built, in process, and with a heap of 64 words.
- **The survey again**: no call or procedure past six left; `not yet`
  met 264 times, was 343; 13 items refused, as before (theirs are the
  type-level names, the strings' and characters' rest, `f64`, a vector).
- **The full check**: all passed in 626.1 s; kc-steps 351 of 351, each
  built again warm, no body run; kc-objects 457, 0 differ (the
  `kc_args+N(%rip)` references across objects as `as` makes them);
  nextest 5501 passed.
- **The timings**: C463's rows, the snapshot check's first, recorded at
  this check's start, were slower than C462's by hand (at 1000
  procedures `fix-forward` 509 ms, was 299; Fixpt's column 5% slower
  too); this check's, at C464, are C462's again (299 ms). One run, best
  of 3, can be that far off: a trend is to be read over several steps.

## C465. K8, the forward-thinking runtime; and the debts against it listed (2026-11-18)

The user, on C464's `kc_args` being the image's ("two threads running
the same image at once would clash"): "Hurm. I had hoped that Kayak and
kc were taking a more forward thinking approach to this. Can you make a
list of all the known places where we have opted for things that
actively work against a multi threaded program?"; then "Yes record this
list. But also figure out where to put the overall design objective:
Kayak is the forward thinking runtime for FX26, building in
incrementality, scalability, asynchrony, and parallelism from the
start".

Where: PLAN.md §3, as **K8**, beside K0–K7, the invariants each step is
held to (not a section of its own, which a step need not read); the
opening list's item 5, "async and incremental throughout", widened to
say parallel and scalable too, and to point at K8. What K8 asks: where a
way that works for many threads, runs and sizes costs about what one
for one does, that way; a choice against any of the four, taken all the
same, a debt written in TODO §55 when it is made and said in its
C-entry; each of the four checked where it can be. The list, TODO §55:
ten items, kc's native side all of them (kayak-rt was built for threads,
C101, C102), chosen in C448–C464, the worst `kayak:last-sealed/1`
(C456), which two threads sealing at once could make free each other's
code. And to-discuss 16 now asks the heap's design for parallelism too.

What I should have done: C464's `kc_args`, like C458's `kc_prompts` and
C460's `kc_run_base` before it, could have been a word of the `%r15`
state block, per run, at the cost of an image cell; I took the cell.
Items 1, 3 and 7 of §55 next, before kc grows more.

Then, the user, on symbols: "isn't the right answer there to have
thread-local portions of the heap, but: 1. objects that spans threads
have to live on a shared heap and 2. you unify the disparate symbols
when you promote from per-thread into the shared heap?" For objects,
yes, and §55 item 2 now says so. For symbols, unifying at promotion
leaves another thread's unpromoted symbol of the name apart (A's `foo`
sent to B; B's own never promoted; two, not `eq?`), which the user
accepted ("Your A sends to B counter example makes sense"); so symbols
are interned in the shared heap, one table, a cache per thread (item 3).
And: "try to avoid building in assumptions that the shared heap is
non-moving GC ... we will just have to ensure it remains incremental":
item 2 and to-discuss 16, which had said "a space that does not move",
now say the shared heap may move, and what that asks (pointers into it
updatable, no address an identity or a hash, the barrier named).

And, on my "an incremental moving collector needs either a read barrier
(Baker-style, or a Brooks forwarding pointer) or a way to update every
reference at once": the user, "see my PhD thesis. We can accomplish
this; the only question is whether we can bear the cost." The regional
collector moves incrementally with a write barrier and summaries, no
read barrier; Coopt's R5 note read the thesis, and its M12 built the
collector (`coopt-heap`), its pauses bounded and tested, its throughput
not yet measured. §55 item 2 now says so, and that `coopt-heap`, a fork
kayak may take, is where kc's shared heap starts; the cost the user asks
about is a measurement still to make.

## C466. A run's state in the state block, and what a thread sealed its own (2026-11-18)

The user: "Yes continue" (TODO §55 items 1 and 7, after C465). Reviewed
first:
- **Fixpt**: its native convention keeps a run's state in the block
  `%r15` (x24 on arm64) holds, one per native thread (kayak-jit's
  `state`: the stack's limit, fuel, the trap, the arguments past the
  registers, the code's distance, `poll`, ...), and its code no cells of
  its own but the globals'.
- **kayak**: kc's convention has had `%r15` since C451 (C452's runtime
  table, the stack's limit, fuel, `poll`, the code's distance), and
  kayak-jit's `LEAVE` (656) is a word of kayak's own past Fixpt's. In
  process, kayak-jit's state is a thread's (`STATE`, thread-local);
  built, kcrt's is the program's one. `kayak:last-sealed/1` (C239),
  used by the cells engine for its perf map and by kc since C456 to free
  an image, read a word the process shared.

What changed:
- **Item 1**: kc's run's words are the state block's, from 664, past
  `LEAVE`: `kc_prompts` (664), `kc_run_base` (672), `kc_heap_next`
  (680), the collector's block `kc_gc` (688, 7 words), the entry's
  stack pointer for a failure (744; it was the heap array's word 4), and
  the arguments past the sixth (752, 58 words). The block is 152 words,
  kayak-jit's and kcrt's alike, each asserting at compile time that
  they fit. The image keeps only the globals' cells, a program's
  definitions, which FX-26's processes share. Each access is shorter
  too: a store is `movq R, N(%r15)`, not a store through the code
  area's writable view, and the collector's block `leaq 688(%r15)`.
- **Item 7**: `last_sealed` is the thread's own (kayak-jit, a
  thread-local): each thread is told the code it sealed last, as every
  caller (kc, the cells engine) asks on the thread that sealed.
  kayak-jit's `code_free` test seals from eight threads at once, 200
  each, each told its own; with the process's word back, it fails.
- **Not yet, and why**: a test of two threads running one kc image at
  once. The heap is still the image's (in process, the code bloblet's
  field, a heap array shared by size, §55 item 6), so two runs would
  share it; that is §55's items 2 and 6.
- **Tests**: kc's control and argument tests (many-args, its capture,
  `whole.fx`, `marks.fx`), the 18 transformers, every way, before the
  check.
- **The full check**: all passed in 637.4 s; kc-steps 351 of 351, each
  built again warm, no body run; kc-objects 457, 0 differ; nextest 5501
  passed. kc-bench at 1000 procedures: `body` 300 ms, `fix-forward`
  289, `add` 303, as C464's; the same bodies run.

## C467. Symbols interned in one table the process shares (2026-11-18)

The user: "Sure. I don't have a solution to 'A sends to B' problem that
you posed, so for right now I'm fine with a shared synchronized intern
table for symbols" (TODO §55 item 3; and, an idea not to dig into now,
whether that problem has an encoding as N systems negotiating a shared
vocabulary incrementally, which §55 item 3 keeps). Reviewed first:
- **Fixpt**: `intern` makes a symbol, the one of its name, with a
  global binding's slot: process-wide by design.
- **kayak**: kcrt-core's `SYMBOLS` was a thread-local; kc's code caches
  each quoted symbol in a cell of the image (`kcy_…`, C421), which every
  thread running the image reads. So under threads a quoted `'foo` could
  be one thread's symbol and that thread's `(string->symbol "foo")`
  another's: wrong, not only unready.

What changed: `SYMBOLS` is a `Mutex` over the map, the process's; a
symbol is made under the lock, so two threads interning one name at
once get one. Interning is rare (once per quoted name per image, and
each `string->symbol`), so no cache per thread yet. Symbols stay in
kcrt's own memory, outside kc's heap, never moved; into the shared heap
with §55 item 2, hashed by name (FNV, Fixpt's) so that it may move them.
Test: kcrt-core's `a_name_is_one_symbol_across_threads`, eight threads
interning 64 names at once, each name one symbol; with the table keyed
by thread instead, it fails.
- **The full check**: all passed in 684.7 s; kc-steps 351 of 351, each
  built again warm, no body run; kc-objects 457, 0 differ; nextest 5501
  passed. But not the new test: kcrt-core's library says `test = false`
  (copied in C452 from kayak-kcrt, which defines a program's `main` and
  so cannot be a test harness; kcrt-core defines none), so nextest never
  ran its unit tests. The test passed by hand (`cargo test -p
  kayak-kcrt-core`); its tests turned on is C468.

## C468. kcrt-core's unit tests run in the check (2026-11-18)

Found at C467's check: nextest's count stayed 5501, the new
`a_name_is_one_symbol_across_threads` not among them. kcrt-core's
`Cargo.toml` said `test = false`, copied in C452 from kayak-kcrt, which
defines a built program's `main` and so cannot be a test harness;
kcrt-core defines none, and has nothing that keeps it from one. Its
`test = false` is gone, with a word why; the other crate that says it,
kayak-kcrt, keeps it for its reason. nextest now runs kcrt-core's test.
The full check: all passed in 632.0 s; nextest 5502 passed (was 5501:
kcrt-core's test among them); kc-steps 351 of 351, warm; kc-objects 457,
0 differ.

## C469. `case`'s data quoted, and strings compared by contents (2026-11-18)

The user: "Go back to expander survey" (TODO §54 step 2). The survey's
largest gap, "type-level names reached as values" (`expr` 26, `data` 25,
`bvar` 24, ... 228 in all), was to be diagnosed first. Reviewed first:
- **Fixpt**: `case` compares the key with each clause's data, which are
  literals all of one kind (integers, characters, strings, symbols or
  booleans; its checker refuses a mix); strings by contents.
- **kayak**: kc's `l0-case-as-if` (C405) made each clause an `or` of
  `(EQ key DATUM)`, `EQ` by the datum's kind (`=`, `char=?`, else
  `eq?`), the datum spliced in as it was written.

What it was: every one of those names came from two procedures, the
expander's `read-sty-name` (`(case n ((expr) ...) ((bvar) ...) ...)`)
and `mx-parser-forms-kind-form?` (`((region place effect type data
...) #t)`). A symbol datum spliced in unquoted is an expression naming a
variable: kc refused where no global had the name, and, where a local or
global did, compared the key with that variable's value, silently. Not
a gap but a miscompile; and its neighbour, a `case` on strings, compared
by `eq?`, so a string made at run time never matched (`(0 0 0)` where
Fixpt gives `(1 2 0)`).

What changed (kc/l0.kfx): a symbol's or boolean's datum quoted, a
string's compared by `string=?`. Tests (tests/kc/step6/, Fixpt's
answers): `case-symbol.fx` (data that name a global, a local, keywords,
a primitive and `nil`; a `case` of nine symbols, the expander's; one of
booleans); `case-string.fx` (strings made at run time). Each failed on
the old kc.

The survey again (as C464, the generated Rust copied aside and back):
11 items refused, were 13; `not yet` 46 times, was 264. Every remaining
refused item has a `not yet` before its failure. Left: `char->string`
11, `datum-list->vector` 7, `string-hash` 6, `f64?` 6,
`symbol-name-hash` 4, `f64->string` 4, `char-whitespace?` 4,
`char-numeric?` 3, `error` 1; then what the survey does not see (an
`int` past a fixnum, the hashes' values, the expander's host).
- **The full check**: all passed in 544.1 s; kc-steps 353 of 353, each
  built again warm, no body run (8777 bodies in its cold passes, kc's
  identity new); kc-objects 461, 0 differ; nextest 5502 passed.

## C470. The expander's rest: characters, hashes, `f64`, a vector, `error` (2026-11-18)

The user, on C469's proposal to take what the survey left as one step:
"Yep sounds good" (TODO §54 step 2). Reviewed first:
- **Fixpt**: `char->string` a string of one; `char-whitespace?` and
  `char-numeric?` Rust's Unicode `is_whitespace` and `is_numeric`
  (`prim.rs`), so `½` is numeric; `string-hash` FNV-1a of the code
  points, each a `u64`, shifted right 4 (`Heap::string_fnv`);
  `symbol-name-hash` the hash `intern` keeps in the symbol's field 1,
  FNV-1a of the name's UTF-8 bytes, its top bit cleared, wrapped to a
  fixnum; `f64?` a bloblet of kind 5 (`ObjType::Flonum`), its bits the
  suffix; `f64->string` `num.rs`'s `format_flonum`; `datum-list->vector`
  `list->vector`, a vector of kind 3 laid out by `alloc` (header,
  elements, a trailer; none, a header alone); `error` of a string, the
  evaluation failed with it as the message.
- **kayak**: kcrt-core's `kc_intern` (C421, C467) already kept Fixpt's
  hash in field 1; heap routines of C462's kind (`x64-heap-routine`,
  the arguments in a stack block the collector moves); kind tests by
  `kc_is_kind` (C448).

What changed:
- **kcrt-core**: `kcrt_char_to_string`, `kcrt_f64_to_string` and
  `kcrt_list_to_vector`, which make in kc's heap and may fail;
  `kc_char_whitespace`, `kc_char_numeric`, `kc_string_hash` (of the
  contents, never an address: the heap may move, TODO §55) and
  `kc_symbol_hash` (field 1); `kc_error`, which always fails; each with
  its in-process variant. The printer writes a vector `#(…)`.
- **l0**: the nine unary. **x64**: each a call; `f64?` `kc_is_kind` 5.
- **Tests** (tests/kc/lists/, Fixpt's answers, every way kc-steps runs):
  `chars.fx` (Unicode whitespace and digits, strings of characters made
  past a 64-word heap), `hashes.fx` (literal and made strings, symbols
  quoted and made), `vectors.fx` (empty, mixed, made in a loop past a
  64-word heap), `error.fx`; and kcrt-core's
  `flonums::written_as_fixpt_writes_them`, each string the pinned
  Fixpt's `f64->string` (kc's programs make no `f64` yet).
- **Found**: kfx's reader (`kfx/read.kfx`, kc's too) has no `|…|`
  symbols, so `'||` is the symbol of two bars, Fixpt's the empty name:
  TODO §54, to fix in the shared reader. `hashes.fx` takes the empty
  name by `string->symbol`.

The survey again (as C469): no item refused, no `not yet`. kc compiles
the whole expander; its 9.4 MB of assembly, 1583 procedures, accepted by
GNU `as`. What step 2 still holds: a file of a place (kc's checker and
`module-parameters`), an `int` past a fixnum, and the expander's host.
- **The full check**: all passed in 652.9 s; kc-steps 357 of 357, each
  built again warm, no body run; kc-objects 468, 0 differ; nextest 5503
  passed (kcrt-core's new test among them).

## C471. Loads in kc: a file of a place's instances, and the expander loaded (2026-11-18)

The user, on C470's next step, a file of a place: "Go ahead"; then, of
two routes (kc compiling a file of a place as its root, the place its
heap; or loads in general, a native host loading the expander as
`kc-instance` does): "General loads first". Reviewed first:
- **Fixpt**: a load of a file of no parameters is one module for all its
  loads; a file of `(module-parameters ((p place)))` is made again at
  each call with the place given, its state its own (C442, Fixpt's
  `docs/research/closed-code.md`). A module's items name each other as
  a program's globals do, a later one too.
- **kayak**: kfx reads what a program loads (`loads-read`) and splices it
  in before checking (`with-loads`, `check-defs.kfx`): each file's
  module a top-level `made--load--K`; a file of a place a maker,
  `(poly ((p place)) ...)` of type `MK--face`, its body `(module
  (%loaded ...) ITEM ...)`, called with the place. kc ran kfx's checker
  but never that splicing (`unbound variable load-module`); its modules
  were static, flattened to top-level definitions (C425).

What changed:
- **The driver** (`kc/main.kfx`'s `kc-read-file!`): what a file loads,
  and what those load, read once (`loads-read`, kfx's).
- **The inputs** (`kc/expand.kfx`'s `kc-load-file!`): the loaded files'
  texts an input, `loads PATH`, so that an edit of one sets the forms
  again; a program that loads has its loads spliced by kfx's
  `with-loads`, after its expansion, kept as an expansion's forms are
  (`expanded PATH`, which the check reads), each form an input of its
  own still. The forms before splicing kept too (`unspliced PATH`),
  which `--expanded-located` writes, as the engine does.
- **Modules** (`kc/front.kfx`): a load's mark, `(%loaded ...)`, nothing;
  `(include M)` of a module known, each of `M`'s values an alias.
- **A file of a place's maker**: its module made at each call as its
  items scope at the top level: each procedure (a value that is a
  `lambda`) in one `letrec`; each other value in a slot of an array of
  the instance's own, set in order, each use of its name reading the
  slot (substituted, binders respected); then a tuple of its interface's
  values in `MK--face`'s order. A first try bound each item by a `let`
  in order, which a procedure naming a later one (the expander's
  `mx-datum-dotted`) showed wrong. `(with x E)` of an instance (a local
  bound to a maker's call, or the call) a `let` of the names `E` uses,
  each its field; `(letrena r ...)` a `let` of `r` to unit (kc's code
  has one heap; a place is nothing at run time).
- **Tests** (tests/kc/loads/, Fixpt's answers; the loaded files under
  `files/`, which kc-steps does not run as programs): `with.fx` (a file
  loaded twice, one module), `place.fx` (instances' state their own),
  `include.fx` (a file of a place including a load, as the expander's
  includes its types file), `expander-facts.fx` (Fixpt's expander
  loaded under a `letrena`, a file's facts asked). kc-steps' cutoff: a
  loaded file's value edited, and a file of a place's procedure, each
  runs one item's five bodies, the program's text as it was.

The expander loaded (`expander-facts.fx`): kfx's checker takes the whole
program; kc builds it cold in 48 s (1058 query bodies; warm, none, 0.3
s); the executable answers in 4 ms, Fixpt's interpreter in 3.4 s; in
process, and with a heap of 64 words, the same answer.

Its cost, said (K8; TODO §55 item 11): a file of a place is one item,
its maker, so an edit of one of the expander's procedures lowers and
assembles all 1583 again. Left of loads (TODO §54): a loaded file's
macros not expanded (kfx's way), a program's top-level `include`
(kfx's checker refuses it, Fixpt accepts it), a `define-datatype` in a
file of a place.

Found at the first full check (stopped by the user's word at 25 minutes,
by mine at 18): kc-steps' shared database's payload grew to 43 MB once
`expander-facts.fx` was built in it, and kc reads it whole, and writes
it whole when changed, at every run: each of the ~1250 runs after it
0.94 s, where 0.012 s against a small one. kc-steps now gives a program
that loads the expander a database of its own (`db_of`); the cost
itself is TODO §55 item 12.
- **The full check**, again: all passed in 694.9 s; kc-steps 361 of 361
  in 512.1 s (its database cold, kc's identity new), each built again
  warm, no body run; kc-objects 478, 0 differ; kc-expanded 258 agree,
  0 differ; nextest 5503 passed.

## C472. The database in shards, and a file a value (`kayak-db 3`) (2026-11-18)

The user, after C471's first check ran past 18 minutes (kc-steps' shared
database's payload 43 MB, read and written whole at each of ~1250 runs):
"we gotta prioritize revisiting the database format now"; then, of three
designs (a file per value, the graph whole; segments compacted, which
needs a `delete-file` FX-26 lacks; a file per value and the graph split
too), "Graph first too". Reviewed first:
- **Fixpt**: no persisted query database; its files are read and written
  whole (`read-file`, `write-file`; FX-26 has no delete, rename, append
  or ranged read: `list-dir`, `file-info`, `read-symlink` besides).
- **Coopt**: `coopt-query`, whose design lib/db.kfx follows, keeps
  nothing on disk.
- **kayak**: C417's two files, the graph (keys read at once, each node's
  record when asked for, C418) and the payload, read whole by the driver
  and both written whole whenever anything changed; kfx's `--cache`, one
  file, likewise; kc's objects (C419), a file each by fingerprint.

What changed (`lib/db.kfx`, `kayak-db 3`):
- **Shards**: a key's group is its first argument (for kc, most often a
  program's path) or its query's name; a shard holds the keys and nodes
  of one group, so one program's are together and a run reads the
  shards of what it reads. A node's number is its shard's times 2^24
  and its place in it, so that a number a node read names its shard. A
  shard is read the first time one of its keys or nodes is asked for
  (its keys numbered, its nodes' records noted and read when asked for,
  as C417's graph was), and written when one of its nodes changed, to
  the other of its two slots.
- **The head**, written last: whose, the revision, the durabilities'
  revisions, and each shard's group's hash, slot and next number. A save
  cut short leaves the files the old head names as they were; a shard
  file not there, or not a shard's, has nothing, its numbers kept its
  own (its next number the head's) so that none is given another key.
- **Values**: each kept value a file of its own, `vN`, written when made,
  read when asked for, and believed only if its fingerprint is the
  graph's (else made again): a file cut short, or one written after the
  graph that names it, is not believed.
- **The interface**: `db-saved` (the files to write, values, shards, the
  head last) and `db-opened` (a reader of the files by name) for C417's
  `db-files` and `db-of-files`; kc writes them at DB and in DB.d/ (made
  by `mkdir` once, as kc's objects' directories are); the database's own
  tests (tests/kfx/db, db-verify) keep them in memory, a save's files
  over those before. `map`, kfx's, takes procedures of the `kfx` effect
  alone, so the database's loops are its own.

Measured: a small program beside the expander's (33 MB of values, 1680
files) runs in 0.012 s, as beside nothing; it took 0.94 s. The expander's
program built warm runs no body, in 0.25 s. kc-steps' one database for
the sweep again (C471's database of its own for the expander's program
taken away). Left (TODO §55 item 12): the head lists every shard, read
and written whole, a line each.

## C473. Small values in their shard's file; side values fingerprinted (2026-11-18)

Found at C472's commit: its kc-bench rows (1000 procedures) against
C471's, cold 6188 ms (809), a body edit 395 ms (304). FX-26's
`write-file` replaces a file "written beside it, flushed to the disk,
and renamed over it" (Fixpt's `docs/fx26.md`; kayak-rt's
`write_file_bytes`, `sync_all`): a file a value made a cold build pay a
flush for each of its 3014 nodes, about 1.8 ms each. The user, of three
ways (small values inline; a non-flushing write, Fixpt's first; a
shard's values packed): "All of these are worth discussing. First give
me the prompt to hand to Fixpt team for non-flushing writes. Then while
they work on that, you implement inline storage of small values." The
prompt asked Fixpt for an atomic `write-file` that does not flush
(`write-file-unflushed`, or any shape Fixpt prefers), for files whose
loss is harmless.

What changed (`lib/db.kfx`):
- **Inline values**: a value of `inline-most` (4096) characters or fewer
  is kept in its shard's file, after its first line, `(kayak-shard 3 NEXT
  LENGTH)`, the values one after another, each record's `(at OFFSET
  LENGTH)` (`atx`, escaped past ASCII, as C417's payload was); cut out
  when read. A longer one, a file of its own as in C472 (the expander's
  maker's code, a program's text, the checker's memos).
- **Side values fingerprinted**: `db-side-put!` made its node with
  fingerprint 0; C472 believes a value read from a file only if its
  fingerprint is the graph's, so the checker's memos (a side value, 392
  KB at 1000 procedures) were read, not believed, and every run checked
  the program whole again: a comment's edit 0.28 s, C471's kc 0.21 s, in
  turn on one machine. Now its text's. tests/kfx/db's program keeps a
  short side value and a long one across a save and a read back
  (`side=kept λ,5000`; before the fix, none).

Measured (one machine, C471's kc built beside, in turn): a comment's
edit, 0.25 s against C471's 0.21 s, the same CPU (439 samples against
447); the rest the flushes, five files against two (the program's text
and the memos, each changed by every edit, and two shards and the
head). kc-bench (the machine some 12% slower than at C471's rows, Fixpt's
column 730 ms, not 650): cold 917 ms, a body edit 352, a comment 286.
Left (TODO §55 item 12): Fixpt's non-flushing write, then each file but
the head written so.
- **The full check**: all passed in 376.2 s; kc-steps 361 of 361 in
  123.3 s (C472's 251.9, C471's 512.1 cold), each built again warm, no
  body run; kc-objects 478, 0 differ; corpus 107 agree; nextest 5503.

## C474. The pin moved to Fixpt 6f7bd73 (2026-11-18)

The user: "Also update to the latest Fixpt from GitHub". 0585488 to
6f7bd73, 30 commits: names under the heap's management (a heap's names,
interners of them, the symbols table weak; TODO §292), `gensym`,
module paths in `with` and `select` and `a::b::c` paths, local binders
shadowing macros, `import-with-syntax` of an instance's macros, ER's
`rename` given no expansion's tag, a form's key of every atom's value
(kayak's INCR_IMPED §5), the closed-code note (§4), and
`write-file-unflushed` (kayak's request, C473).

Carried:
- **The expander**, re-ported (`macros-port.py`; 2220 lines added, 765
  taken away). The driver (`fx26/macros-driver.kfx`) follows two of
  Fixpt's changes: a `with`'s uses carry a kind (0 a macro used by a
  name, 1 a template's `(with m name)`, 2 a path's), and kfx's
  `uses-wrong` reads it as Fixpt's `k-macro-clash` does, a path's name
  its last (`last-path-name`), in Fixpt's words; the type is the port's
  `mx-exports-uses`.
- **ER's `rename`** (Fixpt's 3a84cb5f, answering C446's measure): a
  transformer is called with `%er-rename`, `%~:x`, which the expander
  makes the calling expansion's alias, not `%er-renamer` of a tag
  (`fx26/macros-phase1.kfx`, `kc/phase1-read.kfx`): 27 of kc-expanded's
  28 new differences.
- **kfx's join of a `nat` and an `int`** (`nat-then-int?`): branches
  found by themselves, a `nat` first and an `int` after, refused (the
  rest's fitted to the first's); FX-26 joins them to the `int`, and
  Fixpt's expander now has one (`(if (< k 0) (string-length n) k)`,
  `path-from`). Fixpt took it at the old pin too: a gap of kfx's.
- **`string-search` in kc** (Fixpt's, by code points, -1 if none; the
  expander's paths use it): `kc_string_search`, kcrt-core's.
  `tests/kc/lists/string-search.fx`. x64's three-argument primitives no
  longer fall to `substring` whatever their name.
- **Paths kfx's checker refuses, as Fixpt does** (kayak-checks' K1:
  kayak may accept no program Fixpt refuses): a name `module::name`
  where a binder goes (`name-of`, in Fixpt's words; Fixpt's Rust expander
  refuses it, the FX-26 one kayak ports passes it on:
  `modules/colons-binder.fx`); and a `with` of a module no variable names,
  a path `(with (with m a) x)`: kfx opened the expression's module type,
  its abstract types not its base binding's, so it took what Fixpt's path
  rules refuse (`path-apart`, `path-escape`, `path-not-binding`). Refused
  now, as Fixpt refused any such `with` before; a path's types, to follow
  Fixpt's anchors, TODO §54.
  kfx finds a `::` by its own `colons-at`, not `string-search`, which
  tier 1 has not (selfhost runs the compiler there).
- **The expander program's answer** (`tests/kc/loads/expander-facts.fx`):
  a form's key is now of every atom's value (22aebcf), so its first
  form's is 23307562, not 22848371: Fixpt's.
- **Drift**: ten marks, each read against what was ported: `parser-forms`
  (with-module, path-written: kfx's port is the generative usage's),
  `check.rs` (no standard generative changed), `prim.rs` (two primitives
  added last, `gensym` and `write-file-unflushed`; lang items 562),
  `heap.rs` (the hashes as kayak copied them), `value.rs` (an interned
  symbol's name a text: a gap, TODO §54), `layout.rs` (none),
  `sched.rs` (a global reached otherwise), `print.rs` (a text printed),
  `parser-exps.fx` (a path in `with`: a gap). Built-in modules and
  Fixpt's test programs copied again.
- **kc-expanded**: `modules/colons-macro.fx` known to differ (PATHED):
  kc's expansion a form at a time knows not `m`'s shape at a path's
  `with`, and refuses as Fixpt would where it does not (TODO §54).

Tried and not taken (before the move): kc's link by lld (`-fuse-ld=lld`),
half the default's time alone (0.07 s against 0.12 s, 1000 procedures),
but an edit's build, in turn, 0.32 to 0.35 s by either: no gain, a
dependency more. Next, after C475: `kc build` with no linker at all.
- **The full check**: all passed in 416.3 s (its fourth run: the first
  stopped at kc-steps, the expander program's answer and `string-search`;
  the second at nextest, the four path programs; the third at selfhost,
  `string-search` in kfx); kc-steps 362 of 362; nextest 5595; selfhost
  147 s; cells-programs 652 agree on tier 1; of Fixpt's 615 refused
  programs kayak refuses 615.

## C475. The database's shards and values written unflushed (2026-11-18)

Fixpt's answer to C473's request, at the pin since C474:
`write-file-unflushed`, of `write-file`'s type, replaced whole and
atomically (written beside, renamed over), nothing flushed to the disk
first; after a crash the file may hold its old contents, its new,
nothing, or be missing (Fixpt's `docs/fx26.md`). Reviewed: Fixpt's
`world::replace_file`, one for both, a `Flush` of `Durable` or
`Unflushed`; kayak-rt's `write_file_bytes`, `sync_all` before the rename.

What changed:
- **kayak-rt**: `write_file_unflushed`, and its async twin, one
  `replace_file_bytes` for both, flushing if durable; its test,
  `a_file_replaced_unflushed`, as Fixpt's (written long then short, the
  short alone read back, nothing beside it; a missing directory's
  `not-found`).
- **kfx**: the name's type (Fixpt's), and among the runner's I/O, which
  an async build awaits; lang items: 216 runtime functions.
- **kc's database** (`save-db`): the head, written last, flushed
  (`write-file`); a shard's file and a value's not. The head names what
  is believed: a save cut short leaves the head before it, and the files
  it names, which no save writes (a shard's other slot, C472); a value is
  believed only if its fingerprint is the graph's (C473), so one lost is
  made again.

Measured: a comment's edit of the 1000-procedure bench flushes one file,
the head (C473: five). kc-bench: a comment's edit 252 ms, a body's 317,
cold 827 (the Fixpt column 840, not C473's 655: the new pin's Fixpt is
slower at this program, so that column is not comparable across C474).
Next: an edit's build measured part by part, then `kc build` with no
linker.
- **The full check**, first run: nextest stopped at
  `kayak/io/pselect-processes.fx` on tier 1 (`--regs --compiled
  --gc-stress`): `"0 one\nexited 4"`, the process's last line lost, under
  load; right alone six times in six. A race of tier 1's, not of this
  step's (nothing here reaches processes): TODO §54. kc-steps 149.7 s,
  C473's 123.3: the flushes saved (a few a run) are under the machine's
  noise.
- **The full check**, again: all passed in 283.9 s; kc-steps 362 of 362
  in 63.2 s, its database warm from the first run (no body ran);
  nextest 5596 passed, `pselect-processes` among them.

## C476. What a fresh `kc` costs: frame pointers, heap statistics, and two cuts (2026-11-18)

The user, after C475's measure of an edit's build at 1000 procedures (0.35
s, its linker's processes 0.030 s): "Park linker-less in a todo and focus
on these two" (a fresh process's cost, and the work `kc serve` does too);
on the profile's cut-short call chains, "You can just enable frame
pointer across the board; that's what we resorted to for Fixpt"; and, on
220 MB allocated by one edit's build, never freed, "what the hell".

Found:
- **The time**: an edit's build 0.33 s, user 0.15, system 0.16: the
  system's half 62,000 page faults, kc touching 240 MB, each page faulted
  once: kayak-rt keeps every value for the run (no collector), so every
  intermediate a run makes is memory the kernel gives it.
- **The allocations** (kayak-rt's `heap-stats`, now reported by kc's
  `main` too, and its samples, now naming kc's frames): 7.1 million, 220
  MB; strings 76 MB, characters' lists 45 MB (2.8 million cells), lists
  of strings 22 MB, the reader's 13 MB, the database's maps 11 MB. Of
  the strings, the database's save 50 MB, writing a shard of 265 KB:
  `string-list-append` joining its pieces pairwise, round on round, every
  round's strings kept. Of the characters, the reader's `only-of?`,
  which made `allowed`'s list again for each character it tested, and
  `read-all`, which makes a text's whole list first.

What changed:
- **Frame pointers in every build** (`.cargo/config.toml`, as Fixpt's):
  perf's call graphs walk kayak's Rust, the generated too; an edit's
  build as fast (0.33 to 0.34 s, in turn).
- **`string-list-append` joined at once** in Rust (kfx's emitter,
  `put-define`: its body `kayak_rt::strings_joined(xs)`, one new string,
  each copied once). Its KFX, pairwise (C261), stays the meaning, which
  Fixpt, tier 1 and the engine run; only the Rust differs.
- **`only-of?`** makes `allowed`'s characters once.
- **Lang items**: `strings_joined` an emitter call (62); the corpus's Rust
  of the database's tests blessed again (`base.rs`'s join, `read.rs`'s
  `only-of?`).

Measured (one machine, in turn): an edit's build 0.33 s to 0.28 s, its
memory 240 MB to 177 MB, its page faults 62,300 to 46,500; allocations
220 MB to 154 MB. Next: the reader by index, not a text's whole list of
characters (36 MB left), and the texts read more than once.
- **The full check**: all passed in 287.6 s (its third run: the first's
  corpus wanted its Rust blessed again, the second's lang items the new
  emitter call); kc-steps 362 of 362, warm; nextest 5596; selfhost 151 s.

## C477. kc atop kc: the survey (2026-11-18)

The user: "Let's tackle §54: kc's code on a collected heap"; of three
routes (kc compiling kc, natively on kcrt's collector; a collector for
kayak-rt; the survey first), "Survey first, then decide"; on its result,
"It sounds like a rabbit-hole. How hard is adding GC to kayak-rt?"; then,
"Recording the survey of kc atop kc is a fine idea". So recorded; no
route chosen. Reviewed first: C457's survey of the expander (its way,
kept: a build of kc whose `not yet` says its words and goes on, an item's
failure caught, the generated Rust copied aside and back).

How: kc's unit's files, one after another (32,723 lines; `kc/kc.unit`),
beside a link to `fx26/` for the expander's load. kc's checker allows
syscalls' declarations only in a runtime's own program (a `--syscalls`
unit, kfx's `check-program-as`), which a file cannot say: the survey's
build answered yes (`syscalls-allowed?`). kc's checker takes the program
whole, in 48 s (a first run, before, 2 min 39 s, most of it the kernel's
on memory). kc's code generator refuses 355 items; `not yet` 4956 times,
of 107 kinds: `i64` 3222 (KFX's names, and literals past a fixnum: kc's
`i64` a fixnum where its hashes wrap at 64 bits), modules' `define-rec`s
647 (C425's flattening renames `define`s alone), products by field name
543, strings and characters 264, the system 170, bytes 99, `attempt` 11.
The kinds, sized, are TODO §54's "kc atop kc".

Weighed beside it, as asked: a collector in kayak-rt. Its values are
`&'static` references into arenas of each type and each thread, strings
leaked chunks, passed through locals, closures, constants and kayak-rt's
own maps; no stack maps. So no precise or moving collector; a
conservative mark and sweep, not moving (Boehm's design), in kayak-sys
(`unsafe`, which kayak-rt has none of): a map from address to block,
roots the threads' stacks and registers and kayak-rt's statics and
thread locals, cells used again, the threads stopped; a stress mode
collecting at each allocation. Some 4 to 6 steps; it serves every KFX
program built to Rust (kfx, kc, the engine, tier 1; `kc serve`, which
grows without bound now), and is no road to K8's incremental, shared
heap.

## C478. Handles for kayak-rt's heap: `Handle<'r, T>` for `&'r T` (2026-11-18)

The user, after C477's weighing of a collector for kayak-rt (its values
`&'static` references, no stack maps): "Could we take an intermediate step
of replacing the current `&'static T` uses with Handle<T> that we borrow
from, just to try to contain this problem?"; then "Yep do this now".
Reviewed first: kayak-rt's references were written in few places: its
aliases (`ListAt`, `RefAt`), its allocations (`heap_alloc`, the regions'
arenas, `alloc`), and kfx's emitter's three datatype sites (`type-path`,
`put-type`'s datatype and family arms) and constructors (`put-define`'s
`-> &'at D`); the 6834 `&'static` of kc's Rust all theirs.

What changed:
- **`kayak_rt::Handle<'r, T: ?Sized>`** (`handle.rs`): a reference and
  nothing more (the same size, `Copy`, `Deref` to `T`), made only by the
  heap's and the regions' allocations (`made`, the crate's); `to_ref` the
  reference, `same` identity. Of a lifetime, so a region's (`'r`) and the
  heap's (`'static`) one type as before: code generic in `'r` serves both.
- **kayak-rt**: `ListAt` and `RefAt` hold handles; `heap_alloc`, an
  arena's `alloc` and `alloc` give them; what is kayak-rt's own
  (continuations, mailboxes, equality tables, closures, bloblets, sums)
  by `leak`, a reference, as before. A ref's identity, the equality
  table's key, its handle's (`Identity`, `ref_identity`).
- **kfx's emitter**: a datatype's value `kayak_rt::Handle<'static, D>` (a
  region's, `Handle<'r, D<'r, ...>>`); its constructors return one; a
  module maker's state a reference of its handle (`to_ref`). `match
  *tagcase_` reads through `Deref`, unchanged.
  A datatype of a region's (C270, C271), found by the check's first run
  (the corpus's `data-region.rs`, which no other Rust has): its heap twin
  and its `_in` maker return handles too, and its arena's element is the
  handle's datatype (`held-rust`, which had cut the `&'r ` off the
  reference's text, and so cut `kayak_rt::Handle`'s first four characters).
  And the engine's hand-written REPL front (`rust_repl.rs`, built with
  `rust-repl` alone), whose `Req` and `New` name generated datatypes:
  handles. Lang items: `to_ref` an emitter call (63). kayak-rt's traits a reference
  had, a handle has too: `Show` (Fixpt's programs' Rust shows a
  datatype's value), a ref's `ToValue` and `Shaped`.
- **The bootstrap**: `boot/src/kfx`, written by the old emitter, did not
  build on the new kayak-rt. The old kfx (a worktree at C477) made
  generation 1 from the new sources, still of references; generation 1,
  built there, made generation 2, of handles, which builds here; then
  `regen.sh --all`, at the fixed point at once. The corpus's Rust blessed
  again.

Measured: an edit's build of the 1000-procedure bench 0.27 to 0.28 s
(C476's 0.28), its memory as it was: a handle costs nothing. kc's Rust:
7929 type mentions handles, 345 references left (closures' `&'static dyn
Fn`, a bloblet's). Left (TODO §54): closures, bloblets, flat arrays;
then a generated `Trace`; then a collector.
- **The full check**: all passed in 394.2 s, its fifth run (the first
  four stopped at the corpus's region datatypes, the engine's REPL front,
  the lang items, Fixpt's programs' `Show`); Fixpt running the compiler
  makes the same Rust; selfhost on tier 1 147 s; the async build; nextest
  5596; kc-steps 362 of 362.

## C479. `Trace`: what a value reaches of kayak-rt's heap, generated (2026-11-18)

The user, after C478: "Okay ... keep going". C478's next steps were
closures, bloblets and flat arrays as handles, then a generated `Trace`.
Taken in the other order, said to the user: a closure's captures are
opaque to Rust whether it is a handle or a reference, so a collector
scans one whole either way; what it needs first is to know, precisely,
which parts of each heap value are the heap's. Reviewed first: Fixpt's
`dump-heap!` (FX-26's) dumps a heap to resume it, no census, so no name a
KFX program could call one by: the generated impls tested from Rust.

What changed:
- **kayak-rt's `trace.rs`**: `Trace<'a>`, a value's handles told to a
  `Tracer<'a>`, which keeps what it must still visit in a list (a long
  list traced in a loop), visits each value once, and counts what it
  reached by type, and strings' bytes (`census`); `traced(root)`. No
  `unsafe`: the list holds `&'a dyn Trace<'a>`. The lifetime the
  trait's, so a copy (a cell's value, read) traces what it reaches.
  Impls: scalars, `Str`, `Symbol` (the process's), `Int`, `Handle` (its
  value reached), `Option`, `Cons`, `Cell`, `Array`, `Value`, bytevectors,
  kayak-rt's three products; a plain reference opaque (a closure, a
  dynamic sum, a bloblet: a collector to scan such a block whole).
- **kfx's emitter**: each datatype's `impl Trace` (a region's lifetimes
  outliving the trace's, its type parameters traceable; each variant's
  fields traced), and each shape's (a product's fields, a sum's payload).
- **Tests**: kayak-rt's (a shared tail reached once; a million pairs);
  the corpus crate's `tests/trace.rs`, of `tests/kfx/data.rs`'s generated
  `Tree` (a shared subtree once; 200,000 deep, in a loop).

Measured: an edit's build 0.28 s, as before; kc's binary 7.06 MB.
- **The full check**: all passed in 442.6 s, its third run (the first
  stopped at the I/O programs' Rust, blessed again; the second at
  clippy's two findings in `trace.rs`); nextest 5600 (the four new
  tests); selfhost 155 s; the fixpoint with Fixpt.

## C480. A census of what the stack reaches of kayak-rt's heap (2026-11-18)

The user, on closures as handles: "To me, this is just as much about
getting a precise estimate of what's actually known to be reachable from
the stack as anything else", then approving closure conversion to FX-26's
heap grammar, its first step the measure. Taken here: the mark of
Bartlett's collector, as a measure, nothing freed, with kayak-rt's
`heap-stats` feature (C476) alone.

What changed:
- **kayak-rt's `census.rs`** (the feature's only): each heap arena's block
  noted as it is made, with how its cell is traced (`trace_cell::<T>`);
  each block `leak` makes noted opaque, its size alone; each constant
  noted when first made (`once`), its slot read again at each mark. A
  mark every `KAYAK_HEAP_MARK`th allocation: the run's stack (from
  `with_big_stack`'s top) read word by word, conservatively, a word in a
  noted block its cell's root; an opaque block so reached, and a plain
  reference a trace meets, read the same way; each root and constant
  traced precisely. Said on standard error: the stack's words, roots,
  values and strings reached, and the types reached the most. Its
  `unsafe` (a stack's words, a cell by its address) is the feature's
  only, never a run's.
- **`Trace` required** of what the heap holds: `heap_alloc`, `heap_cons`,
  `heap_new`, `once`, `sum_new`/`sum_of` and the functions building lists
  of a generic `T`; so the emitter bounds every type parameter and
  associated type `for<'t> kayak_rt::Trace<'t>`, and writes a generative
  type's `Trace` (its one field's). New impls: `SumOf<T>` (its payload),
  `world.rs`'s three products, `MarkKey`, `Tag` (numbers alone), `BlobRef`
  (its owner). A plain reference's `Trace` reports it to the tracer, for
  a census to read its block as opaque.
- **Bootstrapped** through two generations of the old kfx (the bounds
  name what the old kayak-rt lacks).

Measured, at an edit's build of the 1000-procedure bench (155 MB in 6.1
million allocations), a mark every 250,000 allocations: 2.5 to 8.3 MB of
values reachable, and 2.0 to 4.4 MB of strings, the most at the read's
deepest (142,547 stack words, 31,165 conservative roots), mostly the
database's maps and the reader's forms. Closures (opaque blocks) reached:
one or two, under 0.1 MB, at most one root through them. So a collector
keeps a twentieth of what kc allocates; and closures' captures are not
where the estimate is imprecise, so converting them to FX-26's grammar
is for a collector's precision and layout, not for this measure. Left
out of the measure: values in letregion arenas (not the heap's), strings
held only by the stack (a `Str`'s bytes `leak_string`'s). An edit's
build 0.27 to 0.28 s, as before.
- **The full check**: all passed in 398.6 s.

## C481. Closures as FX-26's heap grammar has them (2026-11-18)

The user, after C480's census: "Closure conversion. Hell, everything
conversion if you can swing it"; before it, on the grammar: "it doesn't
have to match Fixpt byte for byte, but the *recursive* structure has to
match the grammar too. If you need bytes (i64, etc), put them into the
suffix." Closures first, here; the rest of the heap is listed in TODO
§54 as next.

What changed:
- **`Trace` split in two** (kayak-rt's `trace.rs`): `Trace` of a value
  (every KFX value is `Copy`), traced by value, its lifetime the
  method's, not the trait's; and split by its type into `Field` (what a
  collector follows) and `Byte` (plain bytes), with `split` and `join`:
  tuples, products (`trace_product!`) and generative types part by part,
  `i64`, `f64`, characters, booleans, keys and tags bytes, the rest whole
  among the fields (`trace_whole!`). And `Visit`, of what a handle points
  to (a datatype's value, a pair, a cell, a sum's cell, a closure),
  object-safe. C479's `Trace<'a>` could not trace a closure behind `dyn`:
  its lifetime the trait's, a `dyn` of it invariant.
- **kayak-rt's `closure.rs`**: `Closure<E, C>`, `#[repr(C)]`, its
  captures' fields, then their bytes, then its code, a `fn` of the
  captures and the parameters; `Visit` its fields. `CallN` (0 to 6), and
  `ProcN`/`ProcNs` (`Send + Sync`) handles to `dyn CallN`; `closureN(env,
  code)`. A function as a value: `as_proc(f)`, `FnProc`, no bytes; the
  procedure type wanted chosen by the context, a `Send + Sync` handle
  made a plain one there. `with_mark` takes a `Thunk` (a Rust closure or
  a procedure value); `spawn_monitored`, `blocking1`, `readyN`,
  `native_procedureN`, `Native` and `Resume` take or give handles.
- **kfx's emitter**: a procedure type `kayak_rt::ProcN[s]<'r, A..., R>`;
  a call of a procedure value `.call(...)`. Each of the ten sites that
  made a closure writes it as before into text of its own, and
  `closured` makes that `closureN(env, |env, params| body)`, `env` the
  enclosing locals the body names (its words scanned: no string, number
  suffix, lifetime, path part or field), but the closure's own
  parameters; rustc refuses a `fn` that captures, so one missed is an
  error, not a bug. A `Send + Sync` closure through `as_proc`, but where
  a `let` binds it or a union boxes it; a `let` of a function no Rust
  `let`, its name the function's alias (`%fnalias`); a function called
  where a value is (`(with m f)`, `proj`, a lift's) called by its path.
- **Bootstrapped** through two generations of the old kfx, its regen at
  its fixed point.
- **Tests**: kayak-rt's closure tests (called and traced; bytes kept
  apart from fields).

Measured: an edit's build of the 1000-procedure bench 0.27 to 0.28 s, as
before; kc's binary 7.73 MB (C479: 7.06 MB; each closure its own type and
vtable). The census (C480) reaches each closure by its fields now, no
opaque block; the stack and reachable bytes as C480 found them.
- **Found by the full check**: Fixpt's programs needed rustc told which
  closure's type is wanted where nothing else says it: a poly call whose
  type binder is solved to a procedure's type now has its turbofish (the
  checker's `poly-settled`; before, only a binder of the result alone); a
  closure called at once is called as made; `attempt`'s handler that is no
  `lambda` is given as a function, or a closure's handle called. The
  trait method `into_proc` (clippy: an `as_` takes `self` by reference);
  `lang/items.sexp` gains `as_proc`. Beyond KFX, `native/crossing-
  overflow.fx` now differs: Fixpt overflows the Rust stack 3,000,000
  calls deep, as the program means it to, and the engine (its frames
  smaller under closures) runs it to its end; a resource's limit, not a
  meaning, so left as it is.- **Found by the async check** (the async mode's kfx and corpus): a
  function `proj` or `with` names, called, is called by its path in the
  synchronous mode only (in the async mode a procedure value's answer is
  a future, a leaf function's or a primitive's not); a lift's closure
  `Send + Sync` called at once as made; a word Rust calls (`f(`) is no
  capture, a local being called by `.call`; and a named procedure as a
  value in the async mode (C371's closure boxing its future) calls a
  local by `.call`, through `callee`.
- **The full check**: all passed in 316.1 s, its fifth run (the first
  four stopped at clippy, lang items, Fixpt's programs and the async
  check, each above); nextest 5603.

## C482. A string as Fixpt's object: one word, self-describing (2026-11-18)

The user, on C481's next step (strings, as FX-26's grammar has them):
"self-describing in an unsafe strings module. But it isn't just strings;
why not just handle all bloblets this way?" Asked first because safe Rust
allocates no object of a size known at run time whose bytes follow its
header, and kayak-rt had `unsafe` only in `sum.rs` (and C480's census).
Strings here; the other bloblet-shaped objects next, the same way.

What changed:
- **kayak-rt's `obj.rs`** (`unsafe` allowed, as `sum.rs`): an object as
  Fixpt's `fixpt-heap` lays it, a header word (`kayak_jit::layout::
  header`: kind, fields, suffix's bytes, frozen bits), its fields, a
  trailer word (its distance back), its suffix. `Str` is one word to a
  string's object (kind 1, no fields, frozen, the text the suffix),
  `Send + Sync` (never written after it is made, nor freed); its text
  checked UTF-8 when made. Made by bumping in chunks of words, as C364's
  strings were. Its trailer keeps whether the text is ASCII (bit 62):
  the trailer's payload is the runtime's (`docs/object-model.md`), and
  Fixpt's keeps the distance alone, so a gap, in TODO §54.
- **Literals**: `str_lit!("...")`, a `static` of the object's words made
  at compile time (`StrLit<W>`); the emitters (kfx's, and kc's Rust back
  end) write it. `str_lit(&str)`, a function still, makes a string now,
  for kayak-rt's hand-written Rust.
- **Tests**: a literal and a made string alike; one not UTF-8 refused.

Measured, the 1000-procedure bench: an edit's build 0.27 to 0.28 s, as
before; a cold build 0.63 s (C481: 0.69); an edit's build allocates
137 MB (155); at each census mark, values reachable 25% fewer bytes
(5.7 MB, not 7.8, at the second); the deepest stack 89,311 words
(142,547). kc's binary 8.91 MB (C481: 7.73): a `static` a literal.
- **The full check**: all passed in 521.5 s, its first run; nextest 5605.

## C483. A bytevector as Fixpt's object (2026-11-18)

The user, after C482: "Go for it" (bytevectors, then bloblets, then
arrays, each as C482's string is). A bytevector, `BytesAt<'r>`, is one
word to an object of Fixpt's layout (`obj.rs`): header (kind 4,
bytevector; no fields; its suffix not frozen), trailer, its bytes the
suffix. Its object's words are cells (`Cell<u64>`), so that its bytes,
viewed as `Cell<u8>`s, may be written. In the heap, kept; at a
`letrena`'s region, its object in the region's arena (`BytesCells` now
the object's words), freed with it, as C267's were. `BytesAt`'s field
was public; every use was kayak-rt's own (`world.rs`), now `cells()`.
No change to the Rust kfx writes.

Measured: nextest 5606 (a test of a bytevector written and read).
- **The full check**: all passed in 433.7 s, its first run.

## C484. A bloblet as Fixpt's object (2026-11-18)

After C483, the user's "Go for it" of bytevectors, bloblets and arrays.
A bloblet (`(bloblet (fields T …) R)` of an untyped suffix) is one word,
`BlobletAt<'r, Fs>`, to an object of Fixpt's layout (`obj.rs`): header
(kind 32; its fields counted as words to the trailer; its suffix frozen
for a sealed copy), its fields (a tuple of cells, typed, in place),
trailer, its suffix words of cells, one block. `Bloblet<Fs>` is the
object owned: a region's arena holds one, freed (its fields dropped)
with the region; the heap's is kept. A sealed copy's (C116) code, and
how a field written is written beside it, are in a table of the
thread's by its object's address (a bloblet is no `Send`), its suffix
frozen by its header's bit, as Fixpt's is.

The emitter: a bloblet's type `kayak_rt::BlobletAt<'r, (…)>`, not `&'r
Bloblet<…>`; a field read and written by `.fields()`, which a typed-suffix
bloblet (`TBloblet`, left as it is) has too. `native_procedureN` takes
the sealed copy by value (`impl Sealed`). Bootstrapped through two
generations of C483's kfx.

Left: a typed-suffix bloblet (`TBloblet`), its suffix a Rust value and
an array in it boxed apart (`FlatArr`); arrays; bloblets opaque to a
trace (their fields cells of any type).

Measured: nextest 5607 (a bloblet's fields and suffix); the async check
and Fixpt's programs as before.
- **The full check**: all passed in 456.1 s, its first run.

## C485. Typed-suffix bloblets and arrays as Fixpt's objects (2026-11-18)

The user, after C484: "Okay. Go for it" (the rest: typed-suffix
bloblets, arrays).

What changed:
- **`TBloblet`** (C32's typed suffix): `#[repr(C)]`, its header (kind 32,
  its suffix's bytes), its fields, its trailer, its suffix, one block; the
  byte count its header's, no field of its own. An array in its suffix
  (`FlatArr`) is boxed apart still: made in place, the object would need
  its flat valid as all zero bytes, an `unsafe` trait that the emitter's
  flat products would implement in generated Rust, which the workspace
  denies (`unsafe_code`); so left, in TODO §54.
- **`Array<T>`** (`(arrayof T R)`): one word to an object (`obj.rs`): its
  header (kind 3, vector; its elements the fields, counted), its
  elements each a cell, its trailer; past 2^18 - 1 elements, Fixpt's
  `large` header and its second word (kind 255, the count above bit 11).
  Its field was public; every use was kayak-rt's (`array.rs`, `trace.rs`,
  `eqtable.rs`). No change to the Rust kfx writes.
- **Tests**: an array of three, of 300,000 (large), of units.

Measured: nextest 5608; the async check and Fixpt's programs as before.
- **The full check**: all passed in 436.0 s, its first run.

## C486. Bloblets and arrays traced, not opaque (2026-11-19)

The user, after C485: "Keep going" (on "everything conversion"). Of
§54's list, what a collector needs first: a bloblet and an array, now
objects of Fixpt's layout, were opaque to a trace (a census read their
blocks word by word, conservatively); now precise.

What changed:
- **A bloblet's object visited** (`obj.rs`): `Visit` for its head
  (`BlobHead<Fs>`), its fields each visited, its size its head and its
  suffix as its header counts it; `Trace` for `BlobletAt` reaches its
  head, once. A bloblet's fields are a tuple of cells: `Visit` for
  tuples (to 16) of what is visited, and for `()`.
- **An array's object reached once** (`trace.rs`): before, each value
  holding an array traced its elements again, in a recursion as deep as
  arrays held in arrays. The tracer now keeps one-word values to visit
  (`reach_value`, the value boxed while it waits, as a reference to it
  would not outlive the value), visited in its loop (`visit_owned`); an
  array is counted once, its bytes its object's (`object_bytes`).
- **The census** (`heap-stats`): a bloblet's and an array's object noted
  as an object traced from its address (`census::object`), no longer an
  opaque block.
- **Bounds**: `make_array`, `list_to_array` and `array_made` want
  `T: Trace + 'static`; `make_bloblet`, `of_native`, `seal_code` and a
  sealed copy want their fields `Visit`; `Native` (what crosses to
  native code) is `Trace`. The emitter's Rust already bounds what these
  are called at; no change to the Rust kfx writes.
- **Tests**: an array and a bloblet reached once each, however often
  held, at their objects' sizes; a chain of 200,001 arrays each holding
  the next, in a loop.

Not done: a typed-suffix bloblet (`&TBloblet`) is a plain reference, so
opaque still (`Trace` for `&T`): it wants a one-word value of its own,
as `BlobletAt`, which with `FlatArr` waits on the user's word on
`unsafe` in generated Rust (C485).

Measured: at an edit's build of the 1000-procedure bench, no opaque
block left (were closures' and bloblets'); 2.2 to 5.9 MB of values and
2.6 to 3.3 MB of strings reached, as C482. An edit's build 0.26 to 0.27
s, as before. nextest 5610; the async check and Fixpt's programs as
before.
- **The full check**: all passed in 437.4 s, its first run.

## C487. Typed-suffix bloblets in one block, `unsafe` in generated Rust (2026-11-19)

The user, on C485's and C486's gap: "Yeah let's allow unsafe for
TBloblet". The workspace denies `unsafe_code`; generated Rust now has
one `unsafe` item, allowed where it stands.

What changed:
- **`Flat` an `unsafe` marker trait** (`obj.rs`): a flat whose all-zero
  bytes are a valid value and which needs no drop. Implemented by kayak-rt
  for a number's cell and for `FlatArr`, and by the emitter for each flat
  product (`emit-tops.kfx`): `#[allow(unsafe_code)] unsafe impl<A0: Flat,
  …> Flat for Product_…<A0, …> {}`, with its `SAFETY` comment; sound as
  each part is bounded `Flat`. Its `zeroed()` is gone.
- **`FlatArr<T, N>`** an array in place, `[T; N]`, `repr(transparent)`;
  no box apart.
- **`TBloblet`** made as `Bloblet` (C484): one block, `TBlobHead` (header,
  fields, trailer, suffix), allocated zeroed (`alloc_zeroed`), its
  header, fields and trailer written in it; the suffix never built on a
  stack, so a large one is made (a test: 16 MB). The value is one word,
  `TBlobletAt<'r, Fs, S>`; `TBloblet` is the owned object a region's
  arena holds, freeing its block (`Drop`). The emitter writes the value's
  type `kayak_rt::TBlobletAt<'r, Fs, S>`, not `&'r TBloblet<Fs, S>`, and a
  blobref at 0 `(b).suffix()`; `TBlobletAt` joins the names no KFX type
  may take (`check.kfx`).
- **Traced** (C486's way): its head visited, its fields; a census traces
  it from its address. So no bloblet is opaque now.
- **Lang items**: the emitter calls `suffix` (`lang/items.sexp`'s
  `emitter-call`), the first full check's stop.

Measured: nextest 5611; the corpus re-blessed (each flat product's
`Flat` impl alone changed); the async check and Fixpt's programs as
before.
- **The full check**: all passed in 307.0 s, its second run (the first stopped at lang-items, ~35 s in).

## C488. An integer one word, Fixpt's fixnum or bignum (2026-11-19)

The user, after C487: "Keep going". Of §54's list, `Int` next: two words
(a Rust enum, `Small(i64)` or `Big(&'static Big)`), a bignum's limbs a
`Vec` of their own, where FX-26's integer is one tagged word. Taken
before products held by value (which want a choice of when a product is
boxed, and the emitter's work), as kayak-rt's alone: the emitter never
looks inside an `Int`.

What changed:
- **`Int` one word** (`obj.rs`): Fixpt's fixnum, tag 000, 61 bits, its
  value the word shifted right three; or a bignum, tag 100, the address
  of its object's suffix (as Fixpt's bloblet tag says), its provenance the
  object's (`map_addr`; a fixnum's word has none, `without_provenance`).
  The object: a header (kind 6, bignum, no fields, both frozen) and a
  suffix of `[negative?, limb count, limbs…]`, limbs of 64 bits, least
  significant first, as Fixpt's runtime writes one (`num.rs`'s
  `write_bignum`); made in the bump chunks strings are made in.
- **One representation each**: a value is a fixnum whenever it fits 61
  bits (`Int::of_i64`, and `int.rs`'s `int`), else a bignum; so values
  between 2^60 and 2^63 are bignums now, where they were small. `int.rs`'s
  arithmetic computes as before (`Wide`, base 2^32), reading a bignum by
  `view()` (`IntView::Small` or `IntView::Big`) and writing one by
  `Int::bignum`; the fast paths (`small()`) are the fixnums'.
  `Int::fixnum` is `const`, for statics (`world.rs`'s counts).
- **Traced**: a bignum's object reached once (`reach_value`), nothing in
  it traced (Fixpt's kind 6 is untraced); a census notes it as an object.
  `Big` and its `Visit` are gone.
- **Tests**: the 61-bit bounds, a bignum past each, back to a fixnum,
  its object's words; the old ones as they were (one now asks for the
  low 64 bits, as `i64::MAX` is a bignum).

Measured: an edit's build 0.28 s (0.26 to 0.27 at C486), a cold one
0.64 s (0.63); nothing kfx writes changed.
- **The full check**: all passed in 458.4 s, its first run (a clippy lint, `manual_range_contains`, fixed before it).

## C489. A flat array one word (2026-11-19)

The user: "Keep going". A flat array (`(flatarrayof i64 R)`) already had
Fixpt's header, element code and trailer in the three words before its
first element (C126), but was reached by a plain reference, a slice: two
words, opaque to a trace.

What changed:
- **`FlatArray<T>` one word** (`obj.rs`): the address of its first
  element, as native code is given it and as Fixpt's bloblet tag points
  at a suffix; its length its header's suffix bytes; its block's
  provenance kept (`of_cells`, of the whole block, header and all). A big
  one of zeros still reserved, committed as touched (C139).
- **Traced**: its object reached once, nothing in it traced (its
  elements numbers); a census notes it as an object. So nothing is
  reached by a plain reference now but a continuation's and a knot's
  values and a dynamic `Value`'s.
- **kayak-cells' driver** (`main.rs`, hand-written) reads the flat
  arrays it makes for a native callback by `cells()`. Nothing kfx writes
  changed.
- **A test**: one of five and one of 2^17 reserved, each one word, read
  and written, reached once each.

Measured: an edit's build 0.27 s.
- **The full check**: all passed in 457.6 s, its first run.

## C490. A product held boxed, one word to Fixpt's product object (2026-11-19)

The user, asked how a product is held in kc's heap (boxed at each, boxed
where a heap object holds it, or its parts in line in the holder): "I
thought FX26 had support for flat data that needn't be separately heap
allocated; isn't that the appropriate analogy?" Reviewed first, Fixpt's
`docs/fx26.md`: a flat product is laid out in line only where a flat is
wanted (a bloblet's typed suffix, a flat array), of flats alone, and
reached by a blobref, nothing more than a word coming out as a value;
"Whether products and sums of flat data may be flattened too (unboxed
data types, as Rust has them) is a question for later" (and PLAN.md's
Q6). Said so: kayak's typed suffixes already hold flat products in line
(C487); kc's products hold references. The user: "I don't think I want
unboxed products holding references. I think that complicates the object
model too much." So a product is boxed where a heap object holds it, as
FX-26's heap grammar has it (kind 37, its own object); on the stack, its
parts, by value (kc's calling convention), which Fixpt's grammar does not
see.

What changed (kayak-rt):
- **`Prod<P>`** (`obj.rs`): one word, to a product's object: Fixpt's
  header (kind 37, its fields and its suffix both frozen), its parts'
  traced fields, a trailer, their plain bytes the suffix; a part that is
  itself a product held so too. Made by a product's `Trace::split`, so
  what lays out a value in a heap object (C481's split) boxes a product
  there and nowhere else: `trace_product!` gives a product `Field =
  Prod<Self>`, no bytes, and a `Product` impl (`Fields`, `Bytes`: what
  the object keeps). A closure's captures, laid out by `split`, hold a
  product so with no change of their own.
- **`Held<T>`** (`trace.rs`): a value as a heap object holds it, its
  `split`; `new`, `get`, and `const` makers for a `static`'s plain bytes
  or whole value. A pair's element (`Cons`, its `car` private, read by
  `car()`), a sum's payload (`SumOf`), and **`Slot<T>`**, a cell of
  KFX's, holding its value held, read and written as a `Cell`: a ref's
  cell (`RefAt` a handle of one) and an array's element.
- Bounds: what reads a list, a ref or an array asks `T: Trace` (every
  KFX value is), not `Copy`.

kfx's emitter: a region's arena of a ref's cells, and a place's bound,
`kayak_rt::Slot<T>`, not `std::cell::Cell<T>` (`held-rust`, `held-type`);
nothing else, since generated code reaches pairs, sums, refs and arrays
only by kayak-rt's functions and a cell's `get`/`set`. kc's Rust
unchanged; kayak-cells' and kayak-fx26's (their listings' regions) and
the corpus's region programs blessed again. The engine's REPL front
(`rust_repl.rs`) asks `Trace` of a list it walks.

Left of a product in line in its holder: a datatype's field of a
product type (none in kc; a generic datatype at a product, none), a
bloblet's field (its fields `Cell`s, `put-fields-tuple`), an `Option`
(kept whole), a process's message and an I-cell's value, a
continuation's and a knot's. Recorded in TODO §54.

- **A test**: a product of a product and a character held one word, its
  object kind 37 of one field and four bytes, frozen; in a pair, a ref,
  an array, a sum, each one word, read back.

Measured: an edit's build 0.26 to 0.27 s, as before: the benchmark's
build holds no product in a list (kc's lists of products are its
expander's, which it does not run), and its census has no product's
object.
- **Found by the checks**: Fixpt's programs show a typed suffix's flat
  cell through `Cell`'s `Show`, kept beside `Slot`'s; `Held`'s own
  tracing named `reach`, not `trace`, so that the lang items do not read
  the emitter's `Trace::trace(` as a call of a kayak-rt function.
- **The full check**: all passed in 462.0 s, its third run (the first two
  stopped at rustfmt and at the lang items, above); nextest 5614.

## C491. A dynamic value traced, its members their own objects (2026-11-20)

The user, after C490: "Go ahead" (TODO §54's next: a dynamic value,
`Value`). A `Value` held a pair as a plain reference and any other
member (a datatype's value, a sum, a product, a ref, a procedure, an
array) as a leaked copy of it behind `Any`, a box FX-26's grammar has
not, neither traced; a bignum in it not traced either. And a sum
(`Sum`, a reference to its cell's head) was traced as a plain
reference, opaque: its head did not say its payload's type.

What changed (kayak-rt):
- **A sum precise**: its head keeps its cell's type, as a function to the
  whole cell (`cell`: what a header's kind tells a collector); the head
  visited visits its cell. `Sum` a type of its own (a reference to the
  head, `Deref` to it), so that it is traced so and not by the plain
  references' fallback.
- **A union's other member its own object** (`value.rs`): `Value::Other`
  holds `&'static dyn Obj` (`Any` and `Visit`), the member's object, as
  FX-26's union holds a word to it: a handle's (a datatype's value, a
  ref's cell), a sum's cell, a product's object (kind 37, C490). A
  `Member` trait gives the object and finds the member again by its
  type. What has no object of its own yet is boxed, the box traced: a
  procedure (its closure behind `dyn CallN`, which is no `Any`), an
  array, a vector, a `lambdaN` procedure, an `f32` or a fixed-width
  integer (as Fixpt boxes a flonum).
- **A union's pair traced**: `PairLike` is `Visit`; a pair `cons`ed for
  a union (`ValuePair`) made in the heap's arena, not leaked.
- **`Value` traced**: its string, symbol, bignum, pair and member.
- Nothing kfx writes changed: the emitter's `other` and `unbox_other`
  ask `Member` of what they are given.

- **A test**: a sum given for a union is held as its cell, a ref as its
  cell, each found again; a datum of a string, the sum and the ref
  traced, each reached once, nothing opaque.

Found, not taken (recorded in TODO §54): what KFX's Rust still traces as
a plain reference, opaque, the fallback for `&T`: a place and a module's
state (`&Place_P`, `&...FaceState`), an I-cell, a mailbox, a knot, a
continuation, a blobref to a typed suffix's array (`&FlatArr`). And a
`Value` is three words, not Fixpt's one: one word needs every object it
may point at to say its own shape and type, as Fixpt's headers do (a
pair, a datatype's value, a sum, a ref, a closure have no header yet).

Measured: an edit's build 0.26 to 0.27 s, as before; its census the same
(the benchmark's build makes no datum's pair nor sum).
- **Found by Fixpt's programs**: an `f32` given for a union as another
  shape's member, `Member` now, boxed.
- **The full check**: all passed in 499.5 s, its first run; nextest 5615.

## C492. Plain references traced precisely: places, states, I-cells, mailboxes, knots (2026-11-20)

The user, after C491: "Okay keep going". C491 found what KFX's Rust
still traced as a plain reference, opaque (kayak-rt's fallback `Trace`
for any `&T`): a place and a module's state (which closures capture),
an I-cell, a mailbox, a knot, a continuation, a `lambdaN` procedure, a
datum's vector, a blobref to a typed suffix's array. Surveyed by turning
the fallback off: in kc, its expander's place (`&Place_Mxp`, 82 sites)
and its makers' states (84); in the engine and the corpus, the rest.

What changed:
- **The fallback precise** (`trace.rs`): a reference to what is `Visit`
  is reached, its object visited, as a handle's is; nothing else is
  traced through `&T`. What stays opaque says so by an impl of its own:
  a continuation (`&Cont`, its `Resume`'s values a Rust closure's
  captures), a `lambdaN` procedure (`&dyn Fn`, kc's `rust.kfx`), a
  blobref to a typed suffix's array (`&FlatArr`, a pointer into its
  owner's suffix, which a collector finds by the owner's trailer).
- **kayak-rt's `Visit`**: an I-cell (its value once written), a mailbox
  (its messages), a knot (its value once tied), an arena (each value it
  holds), a region's bytevector words (none), a bloblet and a
  typed-suffix bloblet a region owns (their objects); a datum's vector
  traced, each element, its block reached once.
- **kfx's emitter**: a `letrena`'s region struct `Visit`, each arena's
  values (`put-letrena`); a module maker's state `Visit`, each value it
  holds once made, and its face's `Trace` asks its place be `Visit`; a
  place's type parameter bound `kayak_rt::Visit` beside its `Holds`
  (`put-place-generics`). kc's `expand.rs` and the engine's `main.rs`
  changed so; the corpus blessed again.
- **Tests**: an I-cell, a mailbox, a knot and a region's arena, each
  reached once, precisely, nothing opaque; the heap's test's list of
  `&str` (no KFX value, and no longer `Trace`) a list of strings.

Left opaque (TODO §54): a continuation, a `lambdaN` procedure, a
blobref into a suffix (an interior pointer); and a `Value` one word.

Measured: an edit's build 0.27 s, as before; its census the same.
- **The full check**: all passed in 494.1 s, its first run; nextest 5616.

## C493. kc's Rust back end of closures, `lambdaN` gone; a continuation conservative (2026-11-20)

The user, after C492's list of what is left before a collector: "Go
ahead" (its first item: a continuation and a `lambdaN` procedure, the
last references traced as opaque, but a blobref into a suffix).

Reviewed first: who makes a `lambdaN` procedure. kfx's emitter none
since C481; kc's generated crates none; kc's Rust back end (`rust.kfx`,
C412) each lambda it compiles (`lambdaN(move |...| ...)`), a procedure's
type `&'static dyn Fn`, a call `(f)(...)`, a top-level procedure as a
value `lambdaN(kcp_f)`; and the corpus's programs of another machine
(`made-*.rs`, written by an older emitter, blessed again there).

What changed:
- **kc's Rust back end** (`kc/rust.kfx`): a lambda kayak-rt's closure,
  as kfx's emitter's are (C481): `closureN((free, ...), |(free, ...),
  params| body)`, its free variables the ones kc's L0 found (C412,
  `lambda-free`), so its captures traced; a procedure's type `ProcN<
  'static, ...>`; a call `.call(...)`; a top-level procedure as a value
  a closure of nothing that calls it. kc's 77 KFX programs, built by
  `kc --rust` and run, each its stated answer.
- **`lambdaN` gone from kayak-rt**: its makers (`lambda0` to
  `lambda5s`), and what a `&'static dyn Fn` had (its opaque `Trace`, its
  box as a union's member, its `ToValue`, `Shaped` and `Show`); the
  union test's procedure a closure. The checkers' reserved names keep
  `lambda0` to `lambda4` (harmless; a regen of kfx's own to drop).
- **A continuation conservative** (`suspend.rs`), decided, not left: its
  body a suspended Rust future, an `async fn`'s frames rustc lays out as
  it lays out the stack's, which no trait of ours can name; so a
  collector scans it as it scans a stack (Bartlett's mostly-copying),
  pinning what it may point to, and a trace marks it opaque. Found: the
  future's own block (`Box::pin`'s) is no block a census or collector
  knows, so a scan of the continuation finds only a pointer to it; to do
  with the collector (TODO §54).

So opaque to a trace now: a continuation (conservative, as a stack is)
and a blobref into a typed suffix (an interior pointer, its owner found
by its trailer). Nothing else.
- **The full check**: all passed in 516.4 s, its first run; nextest 5616;
  kc-steps' Rust pass 77 programs.

## C494. A closure's code no pointer: one of no captures no bytes (2026-11-20)

The user, after C493's list (a closure of no captures made at each
evaluation, about 9000 at an edit's build, where one made once would
do): "Go ahead".

Taken in kayak-rt, not the emitter: a closure's code is its own Rust
closure's type now, generic (`Code: Fn(E, ...) -> R`), not a `fn`
pointer. One that captures nothing is zero bytes, so a closure of no
captures is zero bytes whole, and a box of nothing allocates nothing:
made once or made at each evaluation, it costs no allocation. One of
captures is its captures alone, a word less. The emitter's text is the
same (`closureN(env, |env, params| body)`); rustc's check that the code
captures nothing (a `fn`'s, C481) is kept as a check when built: its
size zero, asserted in a `const` block. Considered, not taken: a static
per site, which a generic procedure's closure (its types its own) cannot
have.

- **Identity**: closures of no bytes share an address. KFX's Rust never
  asks a procedure's identity (`eq?` of immediates alone, the checker
  telling no pair apart); and a handle's `same` compares a `dyn`'s
  vtable too, so closures of two codes are never one. FX-26's `eq?` of
  procedures says `#t` only of equal ones, `#f` saying nothing.
- **A test**: a closure of no captures zero bytes, of an `i64` eight.
- A closure of no bytes is not noted among the heap's allocations.

Measured (a fresh build of the 1000-procedure bench, heap-stats): 18.22
million allocations to 16.73 million, the census's 377.1 MB to 354.2
MB (its size classes'). An edit's build 0.27 to 0.28 s, a cold one 0.64
s, as before: fewer allocations, no less time.
kc's binary 9.23 to 9.27 MB (a type per closure site).
- **The full check**: all passed in 509.9 s, its first run; nextest 5617.

## C495. Fixpt words in the heap: scalars, integers, strings (2026-11-20)

The user, after C494's next step (Fixpt's header on every object, pairs
first): "Go ahead". Reviewed first, Fixpt's `fixpt-heap` (`layout.rs`,
`value.rs`): a pair has no header, a two-word car/cdr cell whose words
are values, a walk telling it from an object by its first word, never a
header's tag. So the step was not a header but the words: said so, with
three ways (types by page, a big bag of pages; Fixpt words everywhere;
headers on all but pairs). The user asked what boxing the second meant
(only an `f64` always, an `i64` or `u64` past 60 bits, the rest a tag's
shift), and whether it behaves as Fixpt's native code does: it does,
`docs/performance.md`'s "`i64` and `u64` raw in native registers" (raw
in registers, boxed wherever a value leaves one: "nothing raw is stored
in a frame, passed, returned or kept in the heap"), kayak boxing less
(raw across calls and returns too, Rust's types static). The user: "Yes
I think Fixpt words everywhere is a better foundation."

So every value a heap object holds is to be a Fixpt word; this step the
values that have Fixpt's words now, the rest held as they are until
their objects are Fixpt's.

What changed (kayak-rt):
- **`HeapWord`** (`obj.rs`): a Fixpt word, a pointer so that one to an
  object keeps its provenance (as `Int`): a fixnum; an immediate
  (`#f`, `#t`, unit, a character, an `f32` its bits in the upper half,
  Fixpt's subtags); a tagged pointer, kayak's an address where Fixpt's an
  index, the tag the same three bits: a bignum's and a string's (tag
  `100`, its suffix's start: a string's text), a flonum's (kind 5,
  Fixpt's boxed `f64`, made in the words' chunk as a bignum is). An
  `i64` or `u64` the integer it stands for, a bignum past 60 bits.
  Named so beside a program's own `Word` (kayak-cells has a datatype of
  the name, its glob import meeting kayak-rt's).
- **`Trace::Stored`**, `store`, `load`: how a heap object holds a value;
  a scalar, an `Int`, a `Str` a `HeapWord`; a product its object, one
  word (C490); a tuple its parts'; the rest, as yet no object of Fixpt's
  layout, itself (`trace_whole!`; `trace_whole_fields!` for a word's).
  **`Held<T>`** holds `T::Stored`: so a pair's element, a cell's value
  (a ref's, an array's), a sum's payload are Fixpt words where the value
  has one. A `static`'s held byte, integer, unit by `const`s.
- kfx's emitter: a generative type's `Stored` its field's (`put-
  generative`). Nothing else it writes changed.
- **A test**: each scalar's word, Fixpt's bits (a fixnum shifted, `#t`
  `#f` unit Fixpt's immediates, a character's subtag), a flonum's and a
  string's tag, a bignum past 60 bits.

Left (TODO §54): what is held as itself (a handle's datatype value, a
sum, a ref, a list, an array, a bloblet, a procedure, a `Value`) until its
object is Fixpt's, a word then; a closure's captures and a product's
object, laid out by `split` (fields and bytes apart), to be words too; a
bloblet's fields (`Cell`s).

Measured: kc's build makes no bignum nor flonum (its `i64`s all
fixnums: none of the 64-bit hashes feared); the census
of a fresh build as before, 16.73 million allocations, 354.3 MB. An
edit's build 0.27 to 0.28 s, a cold one 0.64 to 0.65 s; peak memory
171 to 176 MB at an edit's, 417 to 443 MB cold, not yet explained.
- **The full check**: all passed in 484.0 s, its first run; nextest 5618.

## C496. A closure's captures and a product's object Fixpt words (2026-11-20)

The user, after C495's next steps (a closure's captures and a product's
object as words first, or the memory rise first): "Step 1 next".
Reviewed first, Fixpt's `fixpt-heap` (`layout.rs`, `heap.rs`): a
product (kind 37) is `make_frozen`'s bloblet, its fields the values'
words in order, no suffix; a native closure's free values a word each
(`CLOSURE_FREE0 + i`).

What changed (kayak-rt):
- **A product's object** (`ProdObj`): its header, its parts' words (each
  its type's `Stored`), its trailer, no suffix, as Fixpt's: `Product`'s
  `words` and `of_words`, not `apart` and `together`.
- **A closure**: its captures' words, then its code (its type's, no
  word, C494). Its captures by `Captures`, a tuple's parts' words; none,
  no words, not unit's word: a closure of no captures still no bytes
  (the first build of this said 8, unit's word, which the test caught).
  No header yet, as before.
- **`split`, `join`, `Field`, `Byte` gone** from `Trace`: nothing lays
  out fields apart from bytes now, every value a heap object holds its
  `Stored`. kfx's emitter writes them no more for a generative type
  (`put-generative`); boot/'s own Rust has none, so no bootstrap of two
  generations.
- An escape's tag and a mark's key, a `u64` each, Fixpt words (fixnums)
  now, not held as themselves.
- **Tests**: a product's header two fields and no suffix; a closure of an
  `i64`, a string, an `f64` three words; a tuple stored as words and
  loaded again.

Measured (the 1000-procedure bench):
- **The census**, a fresh database, kc's object cache warm: 16.73
  million allocations, 354.3 MB, each type's count as C495's.
- **A correction to C495**: its "no bignum" was of a build with kc's
  object cache (`exe.o.d`) warm. With it empty, a cold build makes
  36.0 million allocations, 81,686 of them bignums at C495 (2.6 MB, its
  64-bit integers past 60 bits in pairs), 82,546 now (860 more, from a
  closure's or a product's words); each other type's count the same.
  Where those integers are made is not yet traced (TODO §54).
- An edit's build 0.28 to 0.29 s, a cold one 0.65 to 0.66 s; peak memory
  174 to 175 MB and 443 MB, as C495's.
- **The full check**: all passed in 494.3 s, its second run (the first stopped at 106 s, an unused import in a test); nextest 5618.

## C497. A list one Fixpt word, a pair Fixpt's headerless cell (2026-11-20)

The user, after C496's next steps (pairs next, or first where the cold
build's bignums are made): "Go with direction of one word migration.
Don't worry about the bignums".

Reviewed first, Fixpt's `fixpt-heap` (`value.rs`, `layout.rs`): a
pair has no header, a two-word cell of its car's and cdr's words, a
word to it tagged `001`; the empty list an immediate (subtag 2).

Considered: a list kept `Option<Handle<Cons>>`, its word only where a
heap object holds it. Not taken: `Option` and `Handle` are `Trace`
generically, so a pair's tag (a handle to a pair is `001`, to an object
another) would need a trait of the pointee's for every type a handle
points to, and a way through `Option`. Taken: kfx's emitter reaches a
list only by kayak-rt's functions (`nil`, `heap_cons`, `car`, `cdr`,
`null_p`), never as an `Option`, so the list's type itself is a word.

What changed (kayak-rt):
- **`ListAt`** (`obj.rs`): one Fixpt word, on the stack as in the heap:
  `HeapWord::NULL`, or its first pair's address tagged `001`; made only
  of a pair's handle or the empty list (`of_pair`, `NIL`), so the word
  it reads a pair of (`pair`) is one it made. Its `Stored` its word.
- **A pair** (`Cons`): its car's word and its cdr's, Fixpt's
  headerless two-word cell, as it was in size; its cdr a word now, not
  an `Option`'s pointer, `0` no longer the empty list.
- kayak-rt's walks of a list by `pair()`; nothing outside kayak-rt
  treated a list as an `Option`, and the emitter's text is the same. A
  list crossing to native code is still copied, each way (C116).
- No list is `Flat` (a zero word no list now): only scalars' cells and
  products of flats are.
- **A test**: the empty list's word Fixpt's immediate, a list's tagged
  `001`, a pair two words, a list held as its word.

Measured (the 1000-procedure bench): the census as C496's, 16.73
million allocations, 354.3 MB; an edit's build 0.28 s, a cold one 0.64
to 0.65 s; peak memory 175 to 176 MB and 442 MB, as before. kc's binary
9.39 to 9.46 MB (each walk's test of a tag).
- **The full check**: all passed in 582.2 s, its first run; nextest 5619.

## C498. A ref one Fixpt word, to Fixpt's box; headers count the trailer (2026-11-20)

The user, after C497 (refs next): "Okay let's just go on to refs as
Fixpt boxes".

Reviewed first, Fixpt's `fixpt-heap` (`heap.rs`'s `alloc`, `make_box`,
`put_header`, `read_head`; `value.rs`'s tags): a box (kind 10) is an
object of one field, settable: its header, its field, its trailer; a
word to it tagged `100`, at its suffix's start (past its trailer), its
field two words back. And two things kayak-rt had not as Fixpt has
them, found reading `put_header`:
- **The header's count of fields counts the trailer** (`fields + 1`;
  `bloblet_len` and `obj_len` take it off again), as kayak-jit's
  objects count it (`object_with`, "the trailer counted"). kayak-rt's
  own did not: a bloblet's (C484), a typed-suffix bloblet's (C485), a
  product's (C490), an array's (its elements, not its words). Now each
  counts as Fixpt's: its fields' words and its trailer.
- **A large object's extension word comes before its main header**
  (`read_head` finds the count at `at - 1`). kayak-rt's large array had
  them the other way; now Fixpt's order, its length read from whichever
  its first word is (the extension's kind, 255).

What changed (kayak-rt):
- **`RefObj<T>`** (`obj.rs`): Fixpt's box, its header (kind 10, two
  fields: its value's cell and its trailer; neither frozen), its cell
  (a `Slot`, one word), its trailer. In the heap's arena and a region's.
- **`RefAt`**: one Fixpt word, its box's suffix's address tagged
  `100`, made only of a box (`of_obj`), so that the word it reads a box
  of is one it made. `get`, `set`, `same` (`eq?`: its word), its
  `Stored` its word; a union's member its box; an eqtable's identity its
  word.
- `Slot` stays, the one-word cell: a box's, an array's element.
- kfx's emitter: a region's arena of references `RefObj<T>`, not
  `Slot<T>` (`held-type`); the corpus's Rust of regions blessed again.
- **Tests**: a ref one word, tagged `100`, its header kind 10 and two
  fields; a product's header three fields for two words.

Measured (the 1000-procedure bench): the census as C497's (kc's build
reaches few references), 16.73 million allocations, 354.3 MB; an
edit's build 0.28 to 0.29 s, a cold one 0.65 to 0.66 s; peak memory 176
MB and 443 MB, as before. kc's binary 9.46 to 9.43 MB.
- **The full check**: all passed in 552.7 s, its first run (the machine loaded besides); nextest 5620.

## C499. A sum one Fixpt word, to Fixpt's sum; its tag Fixpt's symbol (2026-11-20)

The user, after C498 (sums next): "Go for it".

Reviewed first, Fixpt: a sum is an object of kind 36, frozen, two
fields, its tag and its value (`fixpt-runtime`'s `prim.rs`,
`make_frozen(SUM_KIND, &[tag, v])`; `fixpt-fx26`'s `typed_cells.rs`); its
tag a symbol, kind 2, interned, three fields: its name (a text of the
heap's), its hash (`symbol_hash_of`: FNV-1a of the name, its sign bit
cleared, a fixnum's 61 bits) and its global's place (`heap.rs`'s
`intern_text`). kayak-rt's sum was a reference to a cell of kayak's own:
its tag's SipHash, its name (a `&str`, two words), a function giving its
payload's type, one giving its cell's type to trace, then its payload.

What changed (kayak-rt's `sum.rs`):
- **`SymObj`**: Fixpt's symbol, kind 2, frozen: its name a string's word
  (where Fixpt's is a text of its heap's), its hash Fixpt's, its place
  −1 (kayak has no global by a symbol). Made as rustc builds,
  `tag_lit!("circle")`, a `static`, its name a string literal's
  (`HeapWord::of_str_lit`, `object_at`: pointer arithmetic, which a
  `const` may do, where `|` on an address it may not).
- **A tag's discriminant its symbol's hash**, Fixpt's, not SipHash:
  `tag_hash` FNV-1a as `symbol_hash_of` (checked against kayak-rt's
  `symbol_name_hash`, already Fixpt's). A collision is still a build's
  failure (`tags_apart_from_runtime`).
- **`SumOf<T>`**: Fixpt's sum, its header (kind 36, frozen, its fields
  counting its trailer), its tag (the symbol's word), its payload (its
  word, `Held`), and one field Fixpt's has not, before its trailer: what
  it keeps of its payload's type (`SumDesc`, a `static` of each payload
  type, by an associated `const`), which traces it and which every read
  of its payload checks, as the cell's two functions did. A `static`'s
  address, so read as a word it is a fixnum, nothing a walk of Fixpt's
  would follow. Gone once a payload's word says its type itself.
- **`Sum`**: its object's start (kayak's raw value, as `Str`'s); held
  as its word, tagged `100` at its suffix's start (`Stored` its word,
  where it was the reference itself); the word read back by its
  trailer's distance. `sum_tag` reads the tag's symbol's hash; a read of
  the payload finds the desc by the header's count, compared by address
  first, by `TypeId` if not the same `static`.
- **A union holds a sum as itself**, `Value::Sum` (`value.rs`), where it
  held its cell's head: a sum's object has no type of rustc's to be
  found again by (`Member`'s `into_value` and `of_value`, defaults for
  the rest). Tried first: a method of `Visit` to find a sum's object
  again. It cost each of kc's 1,558 vtables an entry and a relocation:
  kc's binary 9.43 to 9.64 MB. A union's sum is now written as Fixpt
  writes one, `#<sum TAG>`, not `#<value>`.
- kfx's emitter: a sum made `sum_tagged::<P>(kayak_rt::tag_lit!("t"),
  payload)`, not `sum_new::<P>(const { tag_hash("t") }, "t", payload)`;
  `sum_new` kept the while that `boot/`'s Rust still called it (regen's
  first generation), then removed. The corpus's Rust blessed again.
- **Tests**: a sum one word, tagged `100`, its header kind 36, frozen,
  three fields and its trailer; its tag kind 2; its payload a word; held
  in a pair; a tag's hash Fixpt's, worked by hand. SipHash's reference
  vector test gone with SipHash.

Measured (the 1000-procedure bench): the census as C498's, 16.73
million allocations, 354.3 MB; an edit's build 0.27 to 0.29 s, a cold
one 0.62 to 0.64 s; peak memory 176 MB and 443 MB, as before. kc's
binary 9.43 to 9.56 MB: of what is loaded, 61 KB (tag strings and
symbols, their relocations, the `desc`s, 12 KB of code); the rest
symbols' names.
- **The full check**: all passed in 479.3 s, its first run; nextest 5621.

## C500. A datatype's value one Fixpt word, to Fixpt's sum of a product (2026-11-20)

The user, after C499 (datatypes next): "Go ahead"; then, asked whether a
datatype's value should be Fixpt's two objects (estimated: allocations
+15%, the heap +27%) or one sum of its fields in line (+11%, a format
Fixpt's code could not read): "Fixpt's exactly".

Reviewed first, Fixpt: `define-datatype` is expanded as it is read
(`fixpt-fx26`'s `top.rs`, `expand_datatype`) into a `sumof` of
`productof`s, each variant's members labelled from 1, its constructor
`(sum tag (product (1 x1) ...))`: a value is Fixpt's sum (kind 36, its
tag and a word to its payload) of Fixpt's product (kind 37, its fields).
kayak's was a Rust enum in the heap's arena of its type (C478's
`Handle`): Rust's discriminant, its fields raw.

What changed:
- **kayak-rt's `data` module** (`sum.rs`, a module so that a program's
  glob of kayak-rt meets none of its own names, a datatype `variant`):
  - `DataObj<E>`: the two objects in one allocation of kayak's, Fixpt's
    product of the fields' words (`E`, a tuple, its `Captures` words),
    then Fixpt's sum, its tag the variant's symbol, its payload a word
    to the product (written once the object is where it stays). Its
    first word a `NonZeroU64`, so an arena's `OnceCell` of one needs no
    word of its own.
  - `DataAt<'r, D>`: one word, to the sum's suffix, tagged `100`.
  - `Variant<I>`, one impl per variant: its fields' tuple and its tag, a
    symbol of the impl's own (`tag_lit!`). A read of a variant's fields
    checks that the object's tag is that symbol: so an object is read
    only as the variant that made it, which is what makes the read
    sound (its fields' types the impl's).
  - `Datatype`: what kfx writes for each datatype, its value read, what
    it reaches, its object as a union's member, found again.
  - In the heap `data_heap`; at a lifetime, for the run, `data_alloc`; in
    a place, `data_in`, each value boxed (`DataBox`, of no lifetime, so
    that a place's arenas need not outlive the place under Rust's drop
    check; freeing one runs no code, asserted as it is built).
  - Tuples' `Trace` and `Captures` to 16 parts (a variant of fx26's has
    13 fields).
- **kfx's emitter**: a datatype's value `kayak_rt::data::DataAt`, not a
  handle; each variant's `Variant`, and `__is_V`/`__V` to read it; its
  `Datatype` where its `Visit` was; its constructors `data_heap::<D,
  I>((fields,))`; a `tagcase` on one an `if` per arm, its variant's
  tag compared, only that variant's fields read (first tried: the enum
  read whole and matched, `match w.get()`: kc's user-mode instructions
  +23% over C498's at an edit's build, the reads not inlined; now +2.8%).
  The enum stays, `Clone, Copy`, as the value taken apart.
- **The heap's arena cache** (`heap.rs`): 64 entries by the type's hash,
  where it kept the last type's: a datatype's values are of their
  variants' types now, so allocations go from arena to arena more (a
  HashMap's look-up each, 5% of the instructions, with one entry).
- The corpus's trace test counts a datatype's objects by their census
  name, their variant's fields' (`DataObj<...>`).
- kayak-fx26's `rust_repl.rs` (hand-written, its `rust-repl` feature):
  its requests and news `DataAt`s, not handles (found at the first full
  check, which stopped at `repl-sessions.py`, the engine not building).
- **`Show` for a datatype's value**, its value taken apart and written as
  its enum's (kfx's) `Show` writes it, where a handle's deref did (found
  at the second full check, at `run-fixpt-programs.py`: Fixpt's p159
  shows a list of a datatype's values).

Measured (the 1000-procedure bench), against C499:
- **The census**: 16.73 to 16.75 million allocations; the heap 354.3 to
  470.4 MB (+33%): 48 bytes more a value, of 2.48 million (the
  product's header and trailer, the sum's four words), as Fixpt's format
  has it.
- **An edit's build** 0.27–0.29 to 0.33–0.34 s (+20%), peak memory 176
  to 255 MB; **a cold build** 0.62–0.64 to 0.70–0.71 s (+11%), 443 to
  558 MB. Its instructions (user mode) +2.8% over C498's; its page faults
  +50%, its cycles +15%: the cost is the memory.
- **kc's binary** 9.56 to 10.47 MB (each variant's impls).
- **The full check**: all passed in 499.2 s, its third run (the first stopped at ~20 s, `rust_repl.rs`; the second at `run-fixpt-programs.py`, a datatype's `Show`); nextest 5623.

## C501. A procedure value one Fixpt word, to Fixpt's native closure (2026-11-20)

The user, after C500 (closures next): "Keep going".

Reviewed first, Fixpt: a closure of native code is kind 41
(`native-closure`, `layout.rs`), `[free…][code][trailer]`, its fields
numbered as slots back from the trailer (slot 1): its code at
`CLOSURE_WORD` (2), a word to its code bloblet, free value `i` at
`CLOSURE_FREE0 + i` (3 + i); made by `fixpt-native`'s `native_closure`.
kayak's was a handle to `dyn CallN`, two words (its object and Rust's
vtable), its object its captures' words alone (C496), none for a closure
of no captures (C494).

Measured before choosing how a closure of no captures is made (a
counter, put in and taken out again): at the 1000-procedure bench's cold
build, 253,575 made (59,426 at an edit's), against 202,392 of some. So
each is made as any other, an object of its own (24 bytes, +1.5%
allocations), rather than one of each type kept: a type of closure may
have lifetimes, so no `TypeId` to keep it by.

What changed (kayak-rt):
- **`obj.rs`'s `procs`** (where `unsafe` may be: a closure read again by
  its code's type):
  - `ClosureObj<E, C>`: Fixpt's native closure, its header (kind 41,
    frozen), its free values' words (`Free`: free value 0 last in memory,
    nearest its code, as Fixpt's slots count), its code's word, its
    trailer; its Rust code, of no bytes, after.
  - `ProcCode`: a closure type's code, a frozen bloblet, its suffix its
    call (erased, of its arity, given its type back by the procedure's
    type, the only reader) and its trace; one of each type, an
    associated `const`.
  - `ProcN`, `ProcNs`: one word, tagged `100`, called by its code's call
    (one load more than a vtable's); `Send + Sync` the second, as before;
    its `Stored` its word, where it was the handle.
- `trace.rs`'s `Free`: a tuple's words reversed, to 16 parts.
- `closure.rs`: what is safe of it, `as_proc` (a function a closure of
  nothing, its code the function, where it was `FnProc`).
- kfx's emitter unchanged: it names `ProcN`, `closureN`, `as_proc`,
  `.call`, all kept; nothing regenerated changed.
- `lang/items.sexp`: the emitter's `.call(` now a function of kayak-rt's
  own (the procedure's), where it was a trait's method, so listed (found
  at the first full check, which stopped at `lang-items.py --check`).
- **Tests**: a procedure one word, its object kind 41, frozen, two
  fields (its code, its trailer); of two free values, free value 0 at
  slot 3; a function as a procedure, and a `Send` one as one not.

Measured (the 1000-procedure bench), against C500: allocations 16.75 to
17.00 million, the heap 470.4 to 481.3 MB (closures 198,386, 5.8 MB);
an edit's build 0.33–0.35 s (0.33–0.34), peak 259 MB (255); a cold one
0.72–0.73 s (0.70–0.71), 570 MB (558). kc's binary 10.47 to 11.49 MB:
its loaded part ~500 KB (the call of each type a trampoline where it was
a vtable's entry, the same size; each type's code object beside its
`Visit` vtable, ~80 bytes and their relocations; 284 more type names),
the rest symbols' names.
- **The full check**: all passed in 377.4 s, its second run (the first stopped at ~40 s, `lang-items.py --check`); nextest 5623.

## C502. An array held as Fixpt's word to its vector (2026-11-21)

The user, after C501 (arrays next): "Go ahead".

Reviewed first, Fixpt: a vector (kind 3) is `[header][elements…][trailer]`,
its word tagged `100` one past its trailer (`make_vector`'s
`blob_v(main + 2 + len)`, `heap.rs`); a large one's extension word
(kind 255, its field count above bit 11) before its main header
(`put_header`), the trailer's distance counted to the main header
(`T_DISTANCE`, "back to the header"). kayak's `Array` was one word
already (C485), but its untagged pointer to its block's first word, held
in a heap object as itself (`trace_whole!`); and a large array's trailer
counted to its extension word, one word too many: a bug no reader met,
since nothing read an array's trailer.

What changed (kayak-rt):
- **`obj.rs`'s `Array`**: its pointer to its main header (after the
  extension word, if large), its elements always one word past it; its
  field count read from its header, or, `large`, from the word before.
  `word` (its trailer `fields` words past its main header, the word one
  past that, tagged `100`) and `of_word` (its trailer's distance back).
  Its trailer's distance its field count, from its main header.
- **`FlatArray`** (kind 42, C489): its word Fixpt's, tagged `100`, at its
  first element, its suffix's start, where it was the untagged pointer.
- **`trace.rs`**: each's `Stored` its word, where it was itself.
- Its identity (`addr`, `eq?` on arrays, a census's key) its main
  header's address; the census's block its allocation's start as before,
  its trace finding the main header past an extension word.
- kfx's emitter unchanged; nothing regenerated changed.
- **Tests**: an array's word tagged `100`, read back to the same array,
  small and large; a large one's trailer counting back to its main header.

Measured (the 1000-procedure bench's edit build, `perf stat`, twice each,
against C501 built from a worktree): instructions 1,493.45 million both
(to 0.0004%), page faults 66,842–66,847 both, task-clock 290–294 ms both.
kc holds few arrays in its heap objects; kc's binary 11.49 to 11.50 MB.
- **The full check**: all passed in 517.1 s, its first run; nextest 5623.

## C503. A bloblet's fields Fixpt words; a bloblet held as its word (2026-11-21)

The user, after C502 (bloblet fields next): "Go ahead".

Reviewed first, Fixpt: a bloblet (kind 32) is `[header][fields…][trailer][suffix]`,
its fields tagged words, its word tagged `100` (`TAG_BLOBLET`, "index of
the start of a bloblet's suffix", `layout.rs`), one past its trailer.
kayak's (C484) had that layout, but each field its value in line, a
`Cell<T>` (a product's parts, a closure's handle, an integer's own
representation), and was held in a heap object as its untagged pointer
to its header (`trace_whole!`).

What changed:
- **The emitter** (`kfx/emit.kfx`): a bloblet's fields `kayak_rt::Slot<T>`,
  made by `kayak_rt::Slot::new`, where they were `std::cell::Cell`'s;
  `Slot` holds its value's word (`Trace::Stored`, a product boxed, C490),
  read and written as a cell. `bloblet-ref`, `bloblet-set!` unchanged
  (`.get()`, `.set()`). A typed suffix's cells (its flats) stay `Cell`s:
  raw bytes, as Fixpt's suffix is. kfx's fixed point: one new generation.
- **kayak-rt**:
  - `BlobletAt`, `TBlobletAt`: `word` (Fixpt's, tagged `100`, at the
    suffix's start), `of_word` (back by the trailer's distance, a
    `HeapWord::header_ptr` of its own); `Stored` the word.
  - `native.rs`: a crossing bloblet's fields and `NativeFields` over
    `Slot`s.
  - `Slot: Clone` (a sealed copy's fields, C116), as `Cell`'s was.
- **Regenerated**: kc's `native.rs` (its bloblets of procedures), the
  corpus's 107 and io-programs' 9 blessed again. The corpus's
  other machine's (aarch64) `made-*.rs`, stale since before C490 (they
  still hold `&'static Bloblet`), left as TODO already says: blessed again
  on it.
- **Tests**: a bloblet's field a fixnum's word (7 << 3) in its object, its
  trailer at its place, its word tagged `100`, read back to the same
  bloblet.

Measured (the 1000-procedure bench's edit build, `perf stat`, twice each,
against C501's kc): instructions 1,493.46 to 1,493.51 million (+0.003%),
page faults 66,840–66,854 both; times not compared (a load average of 16).
kc's bloblets are its native code's few; kc's binary 11.50 MB, as C502.
- **The full check**: all passed in 560.0 s, its first run; nextest 5623.

## C504. A symbol one Fixpt word, to Fixpt's symbol (2026-11-21)

The user, after C503 (`Value` as one word next): "Go ahead". `Value` one
word needs every member's word to say what it is, as Fixpt's does;
kayak's `symbol` was a name's string (`Symbol(Str)`, compared by its
text), held as itself, which a string's word cannot be told apart from.
So first, a symbol as Fixpt's; then the datum's vector; then `Value`.

Reviewed first: Fixpt's symbol (kind 2, `[name, hash, place]`, frozen),
which kayak-rt already makes for a sum's tag (`SymObj`, C499, as rustc
builds, `tag_lit!`); kcrt-core's symbols interned in one table the
process shares (C467, TODO §55 item 3), which kayak-rt's never were: its
`eq?` of symbols compared their names.

What changed:
- **kayak-rt** (`sum.rs`, beside `SymObj`): `Symbol` one word, a pointer
  to its `SymObj`; its `Stored` Fixpt's word to it (tagged `100`, past its
  trailer); `Symbol::new` makes one in the heap (a heap arena's cell, its
  header a `NonZeroU64` so that the cell is no wider); `sym_lit!("name")`
  a quoted one, its object made as rustc builds, one of each in each
  crate. `eq?`: the same object, or the same hash and name (what
  interning makes it, and what kayak's comparison of names was), so not
  one object of each name: each `string->symbol` makes one (40 bytes),
  where it made none. `symbol-name-hash` its object's hash, where it was
  computed at each call. Traced: its object reached.
- **The emitter**: a quoted symbol `kayak_rt::sym_lit!("x")`, where it was
  `symbol_lit(str_lit!("x"))` (`symbol_lit` gone). kfx's fixed point: one
  new generation.
- **Regenerated**: cells', fx26's and kc's quoted symbols; the corpus's 107
  and io-programs' 9 blessed again.
- **Tests**: a symbol one word (and `Option<Symbol>`), its word tagged
  `100` to an object of kind 2, read back; a quoted one and a made one of
  one name `eq?`, of another not; its hash Fixpt's.

Measured (the 1000-procedure bench, `perf stat`, twice each, against
C501's kc; a load average of 450 from elsewhere, so times not compared):
instructions at a cold build 3,627.96 to 3,627.87 million, at an edit's
1,493.50 to 1,493.42 million (−0.002% both); page faults the same.
- **The full check**: a first run stopped at kc-steps (kc's Rust back end, `kc/rust.kfx`, still wrote `symbol_lit`: now `sym_lit!`); a second failed one test, `kayak/io/pselect-processes.fx` under cells timed out at 600 s (TODO's race in tier 1's `pselect`, the machine loaded; 30 runs alone each right).
  The third run: all passed in 383.6 s; nextest 5624.

## C505. A datum's vector Fixpt's vector (2026-11-21)

The second of `Value`'s three steps (C504). A datum's vector
(`datum-list->vector`'s, `#(...)`) was a slice of `Value`s of its own
(`&'static [Value]`), a union's member boxed, traced by a visitor of its
own (`VectorAt`, C492). Now `Vector` is `Array<Value>`: Fixpt's vector
(kind 3, C485, C502), its elements' cells, a union's member as any
array is (still boxed, until C506). `VectorAt` and the slice's `Trace`
gone. Nothing generated changed (the emitter names `value::Vector`).

Test: a datum's list made a vector, its length and first element, its
shape `vector`, written `#(1 "a")`.

Measured (the 1000-procedure bench's edit build, against C501's kc):
instructions 1,493.46 to 1,493.42 million, page faults the same.
- **The full check**: all passed in 507.2 s, its first run; nextest 5624.

## C506. A dynamic value one Fixpt word (2026-11-21)

The last of `Value`'s three steps (C504, C505); the user: "Go ahead".
`Value` was an enum of three words (a tag; a scalar, a pair's or a
member's trait object; a shape), a member boxed where it had no object
of its own, found again by its `TypeId`.

Reviewed first, Fixpt: a value is one word, its shape its tag's (fixnum,
pair, immediate, object) or its object's header's kind; a header found
from a word by the word before it (a header itself, for an object of no
fields, a flonum; else a trailer's distance back). `make-array` makes a
plain bloblet (kind 32, its elements its fields, no suffix:
`%make-bloblet-filled`), not a vector; `array?` asks for kind 32.

What changed (kayak-rt):
- **`value.rs`, rewritten**: `Value(HeapWord)`, `!Send + !Sync` as before
  (a word to a box another alias writes); `Value::Nil` an associated
  constant, so that kfx's `kayak_rt::value::Value::Nil` stands. Its shape
  read from its word (`Shaped`); a scalar given as its word and read back
  where its tag or kind says it is one (`FromValue`, else "a union
  narrowed to a member it does not hold"). `PairLike`, `ValuePair`,
  `Boxed` gone: `cons` makes Fixpt's pair cell of two values' words (a
  `Cons<Value>`, its tail any value's word, `obj.rs`'s `value_cons`).
- **`Trace::VALUE_WORD`**: whether a type's stored word is Fixpt's own for
  it, which a value holds as it is (scalars, strings, integers, symbols,
  lists of such, arrays, refs, sums, datatypes' values, products,
  procedures, bloblets). `list` of such a list is its word, no copy (it
  was a view, each element boxed as read); `unbox_list` the same word,
  where it was a downcast of the pair's type; `boxed_list` the list
  itself. Of any other, `list` copies, giving each element.
- **`Member`** (a union's member of another shape): `into_value`, its word;
  `of_value`, its kind checked (`word_member!`): a sum's (kind 36, four
  fields), a datatype's value's (kind 36, three fields, its tag one of its
  variants', `Datatype::has`, which the emitter writes in place of
  `object` and `of_object`), a ref's (10), a procedure's (41), a
  product's (37), an array's (32, or 3 for a datum's vector); an `f32`
  its immediate, a fixed-width integer its fixnum. So a member is found
  again by its kind, not its type (to-discuss 17).
- **An array Fixpt's plain bloblet** (kind 32), where it was a vector
  (kind 3, C485): so `array?` and `vector?` tell an array and a datum's
  vector apart by kind alone. A datum's vector stays kind 3
  (`array_made_of`).
- **A product held as its word** (`Prod::word`, tagged `100`, one past its
  trailer), where it was its object's untagged address.
- **`HeapWord`**: its tag, its immediate's subtag, its object's header and
  kind; a pair's words; `header_ptr` Fixpt's way (a header just before,
  or a trailer's distance, its bits above 61 not of it: a string's ASCII
  bit, C482). `Cons` `#[repr(C)]`, its head first, as Fixpt's pair.
- **Traced** (a value's own): a pair's parts, as values; a string's text;
  a symbol, a bignum, a `sumof` (its payload's type kept) and a closure
  (`reach_closure`, by its code's `reach`, whatever its type) precisely;
  an object whose fields' types only its type knows (an array, a box, a
  product, a bloblet, a datatype's value) told the tracer as opaque, by
  its header's address, which a census resolves to its block and traces
  as its type has it.
- **The emitter** (`emit-tops.kfx`): a datatype's `has`, its variants'
  tags tested, for `object` and `of_object`. kfx's bootstrap through a
  worktree of C505, as C478's (boot/'s Rust named the old methods):
  generation 1 by C505's kfx, generation 2 by it, then `regen.sh --all`,
  at the fixed point at once. Everything regenerated; the corpus's 107
  and io-programs' 9 blessed again.
- **Tests**: a value one word; a symbol, unit, a character, `#f`, an `f64`,
  a bignum each of its shape; a typed list's word the value's; a value's
  list narrowed to a list of `i64`s; an array of shape bloblet, not
  vector; a ref; a datum traced: its three pairs and its sum precisely,
  its box opaque.

Found on the way: `cargo build --workspace --all-targets` fails to link
kayak-kcrt's lib test (two `main`s), at C505 too; the check builds no such
target. In TODO.

Measured, against C505's kc (a worktree): the 1000-procedure bench makes
no value (no datum, no union on its path): its allocations the same to
the one (17,001,005, 481.3 MB); instructions −0.13% at a cold build,
+0.13% at an edit's; page faults and times the same. kc on
`tests/kc/step15/procedural-many.fx` (its 18 transformers, the expander
run): instructions +1.2–1.5% (19.68–19.75 to 19.99 billion), time +1%
(2.98–2.99 to 3.02 s), page faults the same; its heap 2,309.6 to 2,308.1
MB, allocations +0.02%: a value's pairs 46,932 of 48 bytes (2.3 MB) to
59,710 of 16 (1.0 MB). The instructions: a value's shape read from its
object's header, through its trailer, where the enum's tag was in hand.
- **The full check**: all passed in 520.3 s, its first run; nextest 5625.
