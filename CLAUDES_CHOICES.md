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
  with `the`), `load-module`, `include`, `extend`, `hide`, functors
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
