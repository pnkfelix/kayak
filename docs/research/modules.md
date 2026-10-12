# FX-26's modules in KFX: what maps cleanly to Rust, and in what order

Status: design, 2026-10-12, the user's: "tackle modules, I'm eager to see
how much of that you can get into KFX cleanly." TODO §19 and §25 (46+
programs). Builds on C19 (top-level modules as Rust `mod`s), C24 (module
parameters as traits), and FX-26's own account (Fixpt's `docs/fx26.md`,
"Modules").

## What FX-26 has, and what KFX has

FX-26's modules are first-class values (Sheldon's, as FX-91 built them):
a `module` is a product of its values at run time, its type a `moduleof`;
`with` opens one, `select` names its types; `include`, `hide`, `extend`
combine them; `load-module` makes a file one; a procedure may take one and
name its types in the types after (dependent, ML's functors a case of it).

KFX has the static part: a module defined at the top level is a Rust
`mod` (C19), its generative types newtypes whose privacy is Rust's; a
top-level procedure's module parameter is a type parameter bounded by a
trait the `moduleof` is (C24); `include` and `hide` of top-level modules.

## The principle

A module whose identity is known where it is used is a choice of code,
which Rust's items (`mod`, `pub use`, generics) express with nothing at
run time and with Rust's privacy for abstraction. KFX takes those first,
since they are clean: the Rust says what the FX-26 says. A module that is
a value made at run time (in a procedure, from a parameter, in a list)
needs a representation of its own: a struct of its values, its abstract
types generic. That is later, and only if programs want it.

## Stages, by cleanliness and count of Fixpt's programs

1. **Items as FX-26's `letrec*`** (`forward`, `mutual`, `include`,
   `hide`): every typed item bound before any is checked; checked in an
   order its references allow, mutually recursive procedures as one
   `define-rec` group (so `spin` and termination are FX-26's as now); a
   value made from an item not made before it refused, as FX-26 refuses
   it. Rust needs nothing: a `mod`'s items are in no order.
2. **Names in a module** (`type-and-value`, `include-same-type`,
   `hide-include`, `transparent-as-abstract`): FX-26's rules for a name
   two `include`s give, a value named as a type, a hidden name given again.
3. **`extend` and re-exports** of top-level modules: a `mod` of `pub use`s,
   `e1`'s where both give a name; `(define x (with m x))` a `pub use`.
4. **`load-module`**: a file, a module that sees only the standard
   environment, loaded once per path (FX-26's): a Rust `mod` per file.
5. **A module's types with parameters** (`datatype-family`,
   `family-alias`, `param-define-type`): Rust's generic types.
6. **`moduleof` beyond C24** (`dependent-later`, `effect-moduleof`,
   functors over abstract types): traits with associated types, used where
   the module is known statically.
7. **Modules as values made at run time** (`local-module`, `nested`,
   `made-in-procedure`, `stack-by-poly`): designed when 1–6 are done.

Each stage records its choices (C85, …), and is checked against Fixpt's
`modules/` and `module-files/` programs, both ways (`refused-fixpt-programs`
too).
