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
