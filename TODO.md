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

## 8. A corpus of KFX programs

K1 is checked only on the compiler itself. Wanted: small KFX programs,
each run by Fixpt (with the prelude) and by its Rust, answers compared,
and golden Rust for a few, read and approved. This needs a Rust driver
that runs a compiled program and prints its value: so far `boot/` is the
only program built.

## 9. Guardrails: `scripts/check.sh` and tidy tests

The siblings' `scripts/check.sh` runs `rustfmt`, `clippy` and the tests,
and their tidy crates make the repository's rules tests. kayak's rules:
- K3, no hash-map order, addresses or clocks in the compiler;
- K4, no `unsafe` in generated code;
- KFX source is ASCII (§17).

`boot/src/main.rs` allows every warning on the generated module. Which
warnings the generated code should be free of is part of this: unused
bindings, from a `tagcase` arm that ignores fields, are the common ones.

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

`'static` is one region, the whole run. `letrena r` as an arena scoped to
the body, `r` a lifetime `'r`, and `rcons p x y` as `p.alloc(…)`; the
runtime's types gain the lifetime, `List<'r, T> = Option<&'r Cons<'r, T>>`.
FX-26's escape check on `letrena` is what makes `'r` well-formed. This is
what makes memory reclaimable without a collector (§16).

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
- **`(the T e)`** is written as `e`. Rust could check `T` too:
  `{ let v: T = e; v }`.
- **Allocation** is `Box::leak`, one allocation per object; an arena
  later, and §14.
