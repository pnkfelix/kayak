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
- **Not chosen:** a Rust function for each colour an instance needs
  (monomorphised by kfx), quicker and larger, later if it is measured to
  matter; effect parameters in families (`(loop u l)` of Coopt's
  library): next, when the library of tasks needs them.
