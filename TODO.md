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
bootstrap in two steps: teach `kfx.kfx` the feature and regenerate
`boot/`, and only then use it in `kfx.kfx` (`README.md`).

---

## 7. Errors that point at KFX source

A Rust error in generated code names a line of `kfx.rs`, and a run-time
`error` names nothing. Start with a `// kfx: line` comment per item: the
reader would keep each datum's line, which `sexp` does not carry yet.

## 10. Self tail calls as loops

Rust has no tail calls, so a walk of a list is as deep as the list.
`with_big_stack` gives the run 1 GiB, which is a workaround. A self tail
call in a defined procedure can be its body in a `loop`, the call a
reassignment and `continue`: local, and still a trivial mapping.

## 11. First-class procedures

A call's operator is always a global's name. A `lambda` as a value would
be `&'static dyn Fn(…) -> T`, allocated in the arena: `Copy`, with its
captures `Copy` and moved.

## 12. `poly` as generics, rank 1

`poly` only at the top of a definition's type: `fn f<T: Copy + 'static>`.
`proj` as a turbofish, its effect and region arguments dropped. Today the
only polymorphism is the runtime's generics.

## 13. A KFX checker, in KFX

Fixpt checks types and effects in stage 0. A checker of KFX's own would
let `kfx` build without Fixpt and still refuse what is not KFX: today it
refuses `int` and a few shapes, and translates anything else it reads.
The verdicts to agree with are Fixpt's.

## 14. Places as lifetimes

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

## 15. Flat data and `blobref`

Flat products, sums and arrays, and `(blobref B F)`, as `#[repr(C)]`
structs, `#[repr(u8)]` enums, `[F; n]`, and references into them. Keyed
flat sums already store their tag "as Rust stores a discriminant"
(`fixpt/docs/fx26.md`, "Flat sums").

## 16. Escapes, then FX-26 at large

- Prompts with only `(goto r)` as `Result` and `?`: the effect says which
  calls may abort, and so which `fn`s return a `Result`. The first place
  effects decide code generation.
- Then the broader FX-26: a runtime with a collector, bignums, unions,
  delimited control, modules. Whether that keeps targeting Rust or
  something else is open (`PLAN.md` §5).

## 17. Small things noticed

- **Strings are ASCII.** `string-ref-i64` is a byte index, and a byte past
  ASCII is an error where it is read.
- **A number literal past `i64` wraps** as the reader reads it
  (`digits-value`), and the Rust gets the wrapped value. It should be
  refused.
- **A local named like a constant** is taken for the constant, and
  written as a call (`x()`), since the emitter does not track locals.
- **`tagcase_`**, the name a `tagcase` binds, could be a KFX name's
  mangling (`tagcase-`).
- **`(define x e)` with no type** is refused; FX-26 allows it.
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
- **A literal with nothing expected is an `int`** in FX-26, so `(let ((n
  1)) …)` binds an `int`, which KFX does not have: `(the i64 1)`. The
  compiler does not see this (it writes `1i64` regardless); Fixpt does.
- **FX-26's keywords** (`sum`, `product`, `extract`, …) cannot name a
  procedure; the compiler does not know them, and Fixpt refuses them.
- **Shifts**: `i64-shl` and `i64-shr` take their count as an `int`; KFX
  has none, so they are not in the corpus. A prelude `i64-shl-i64`, when
  wanted.

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

The hard part either way is effect polymorphism: a latent effect that is
a variable, or a `maxeff` over regions, needs a procedure generic over
sets of tokens. A research note first: which of FX's rules carry over,
and what the generated code looks like for the compiler's own source.
