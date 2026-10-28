# Syscalls: one FX-26 extension through which each runtime extends itself

Status: implemented by Fixpt at `f03cabe2` (its copy of this note, in its
`docs/research/`, records what it decided and changed: a declaration
expands into two definitions, the check at load is their run, `(io sys)`
one atom, declarations per program), and by kayak's checker, engine and
compiler to cells at that pin (C224). Kayak's request follows, as sent. Agreed with
the user in conversation; the details below are this note's proposal, for
the Fixpt team to adjust as they implement it (and to say so here, or in
their copy).

## Why

Every capability a runtime written in FX-26 (or KFX) has needed so far has
been an extension of the language, implemented by Fixpt first: `%sched`
for the scheduler, `seal-code` and `native-procedure` for code a program
makes, and next an executable view of a range of a flat array, which
kayak's tier 1 (Fixpt's cellular machine, ported to KFX) needs in order to
run code a program makes the way Fixpt's heap does (`exec_view` of the
code area's range of `mem`, `fixpt-memmgmt/src/exec.rs`).

Instead: FX-26 gains one extension, a **syscall interface**, through which
a program calls a capability its runtime provides by name, at a type both
sides state. Capabilities are then each runtime's own, added without a
change to the language. Fixpt-first applies once, to the interface.

The rule that keeps this from becoming a back door: **what a portable
program needs in order to mean something stays in FX-26; syscalls are for
building runtimes.** Ordinary programs cannot declare them (see
"Containment").

## The form

    (define-syscall name "host-name" type)
    (define-optional-syscall name "host-name" type)

- `name` is bound, in the scope a `define` would bind it, to a procedure
  of `type`.
- `"host-name"` is the runtime's name for the capability: a namespace, a
  colon, a name, a slash and a version, e.g. `"code:exec-view/1"`. A
  capability's type never changes under a name; a new type is a new
  version (`/2`). Any runtime may implement any name, provided it gives
  it exactly the type the name has: so a capability several runtimes
  share (like `code:exec-view/1`) has a neutral namespace, and one only
  one runtime has, that runtime's (`kayak:`, `fixpt:`).
- `type` is a `subr` type, `(subr E (T …) R)`, or a `poly` over regions
  only of one, `(poly ((r region) …) (subr E (T …) R))`. No type
  binders (no `(t type)`) for now.
- `E` must include the syscall effect, `(io sys)` (a new `io` atom), so
  that every use of a syscall shows in the types of what uses it.
- Each `T` and `R` must be a type that crosses (below).
- `(syscall-available? name)`, `name` a syscall the program declared,
  is a `bool`: whether the runtime gave it at load (always `#t` for a
  required one, since the program would not have loaded otherwise).

### Types that cross

As `native-procedure`'s do, kept small: `int` (a fixnum's range; a
syscall given or giving more fails), `i64`, `u64`, `bool`, `char`,
`unit`, `string` (copied), `(flatarrayof i64 r)` and `(bytevectorof r)`
(passed in place, not copied). More only when a capability needs them.

## The canonical form of a type

The program's declared type and the runtime's are compared as text: each
side prints the type in one canonical form, and the two strings must be
equal. This makes the runtime's check a string comparison, so no runtime
needs a type parser or a type checker, and the form must therefore be
part of FX-26's definition, printed alike by every implementation.

The canonical form of a type:

1. Every name a `define-type` (or a module's type) introduced, expanded
   to what it names, recursively. A generative type, a datatype or an
   abstract type cannot appear (it crosses no runtime boundary).
2. Effects: a `maxeff` flattened (nested `maxeff`s and named effects
   expanded), its atoms deduplicated and sorted by their canonical text
   (byte order); one atom printed alone, not in a `maxeff`; none printed
   `pure`.
3. Region binders renamed in the order of the `poly`'s binder list:
   `r1`, `r2`, …, and every use with them.
4. Printed with one space between items, no other whitespace, and the
   type syntax's own keywords (`subr`, `poly`, `region`, `flatarrayof`,
   …), as FX-26's checker prints types elsewhere.

For example, `(poly ((buf region)) (subr (maxeff (io sys) (write @buf) (read @buf)) ((flatarrayof i64 buf) int int) i64))`
is, canonically,
`(poly ((r1 region)) (subr (maxeff (io sys) (read r1) (write r1)) ((flatarrayof i64 r1) int int) i64))`
(the exact spelling of a region variable in an effect, `r1` or `@r1`, is
whatever FX-26's printer already uses; the point is that it is fixed).

Equality only, no subtyping: a program that wants a looser type wraps
the syscall in a procedure of that type.

## The check, at load

The compiler checks the declaration alone, never against a runtime: the
form, the type's shape, the effect, the types that cross, and where the
declaration is (below). It then records, in what it compiles, for each
declaration: the host name, the canonical type, and whether it is
required. A program is compiled once and may run on any runtime, so
whether a runtime has a syscall is a question for when it runs.

When a runtime loads a program (for KFX compiled to Rust, when the
program starts), for each declaration:

- **Required** (`define-syscall`): the runtime has the host name at the
  same canonical type, or the load fails, naming both:
  - `syscall NAME ("HOST"): this runtime has none`
  - `syscall NAME ("HOST"): declared TYPE; this runtime's is TYPE`
- **Optional** (`define-optional-syscall`): the load succeeds whatever
  the runtime has; where it has none at that type, `syscall-available?`
  is `#f`, and a call fails with
  `syscall NAME ("HOST"): not available on this runtime`.

Every runtime does exactly this, so a program behaves alike on all of
them. A runtime keeps, for each syscall it provides, its host name, its
canonical type (as text, written by hand or by the implementation's own
printer at build time) and its implementation.

## Containment

A syscall can do what the language cannot check (the first hands out a
view of memory to execute), so ordinary programs must not declare them.
A `define-syscall` or `define-optional-syscall` is accepted only:

- in a module file built into the implementation (as `%sched` is
  accepted only in `fx26:` files today: Fixpt's `built_in_only`,
  `check.rs`), or
- in a program its driver was told is a runtime's, by an explicit flag
  (e.g. `--syscalls`); kayak's tier 1 is compiled so.

Anywhere else: refused, `syscalls are declared only by a runtime's own
code`. The `(io sys)` effect then makes every procedure that reaches a
syscall say so in its type.

## The first syscalls: an executable view, and flushing it

What kayak's tier 1 needs, so that its code area is a range of its heap's
own flat array, as Fixpt's is a range of `mem` (`native.md`, decision 3):
code bloblets that the machine's ordinary accessors and collector see as
heap objects, their fields written through the array, their code run
from a second, read+execute mapping of the same pages.

    (define-optional-syscall exec-view "code:exec-view/1"
      (poly ((r region)) (subr (maxeff (io sys) (write r)) ((flatarrayof i64 r) int int) int)))

`(exec-view a start n)`: words `start` to `start + n` of `a` mapped a
second time, read+execute; the result is the distance in bytes from the
array's address to the view's (the same for every word of the range).
Fixpt's `ExecView`, `exec_view` (`fixpt-memmgmt/src/exec.rs`). Fails
(the result -1) unless:

- the runtime keeps `a` at an address that never changes (kayak-rt's
  reserved flat arrays; a Fixpt heap's flat array, which a collection may
  move, is not one: Fixpt may give -1 there, or provide it only for
  arrays it pins);
- `start` and `n` cover whole pages (a page's words: `code:page-words/1`
  below);
- no word of the range was written yet: the view replaces the range's
  pages with shared ones, zero, as they were (as Fixpt's code area does,
  `code.rs`, "before anything is written there").

    (define-optional-syscall page-words "code:page-words/1"
      (subr (io sys) () int))

The words in one page of this machine (512 on x86_64 Linux, 2048 on arm64
macOS).

    (define-optional-syscall flush-code "code:flush/1"
      (poly ((r region)) (subr (maxeff (io sys) (read r)) ((flatarrayof i64 r) int int) unit)))

`(flush-code a start n)`: the instruction cache made to see words
`start` to `start + n` of `a`'s view as written (Fixpt's `flush_code`,
`ExecView::flush`). A no-op where the machine needs none.

How tier 1 calls code there (a native procedure kayak already makes, a
stub entered with the machine's registers in a flat array), and how its
collector marks code bloblets in place, are kayak's, not part of this.

## What each side does

**Fixpt** (first):
- the forms `define-syscall`, `define-optional-syscall`,
  `syscall-available?`; the `(io sys)` atom; the containment rule;
- the canonical printer, as above (and its definition written into
  FX-26's definition wherever Fixpt keeps that);
- the record of declarations in what it compiles, and the check at load,
  with the messages above, on every one of its machines;
- its own syscall table: `code:exec-view/1`, `code:page-words/1`,
  `code:flush/1` where it can give them (its heap's flat arrays move:
  -1, or pinned arrays, as it chooses), and none other needed;
- tests: a program declaring a required syscall its runtime lacks (load
  fails, both messages), one at another type, an optional one absent
  (`syscall-available?` `#f`, the call's failure), a declaration in an
  ordinary program (refused), and the canonical form's cases (aliases,
  effect order, region renaming).

**Kayak** (after the pin moves):
- the same in kayak's checker (`kfx/`), which is shared by KFX and the
  engine, and in KFX (restricted to what KFX can emit);
- kayak-rt's syscall table, and the check at program start;
- the engine's table (none at first; later, perhaps, the terminal and
  the standard input, out of the engine's core);
- tier 1's code area on `code:exec-view/1`: then `seal-code`,
  `native-procedure`, `native-code?` as Fixpt's `fx-cells-rsinterp` has
  them, and Fixpt's `made-code-x86_64/*.fx` programs on tier 1.

## Amendment proposed: no effect required (kayak's request, 2026-10-27)

For Fixpt first, as the design was. **Drop the rule that a syscall's
effect must hold `(io sys)`.** A syscall's effect becomes whatever its
declaration says, within the atoms already allowed (reads, writes and
allocations of the `poly`'s regions, `io` effects, `spin`, `nondet`), and
none, `pure`, among them. Everything else stays: the crossing types, the
region-only `poly`, the canonical form (which prints no atoms as `pure`
already), the check at load, containment.

**Why.** The rule was this design's own (its "so that every use of a
syscall shows in the types of what uses it"), and it buys less than it
costs:

- *It adds no safety.* A syscall is trusted already, wholly: one declared
  `(maxeff (io sys) (read r1))` is trusted not to write `r1`, not to
  allocate, and to give what its type says. A host that breaks that
  breaks the program whether or not `(io sys)` is there. The program and
  the runtime coordinate on the type, by its canonical text, effect
  included; a host function that does I/O says so, with `(io sys)` or
  another `io` atom, as any other part of its type.
- *Visibility it does not need to give*: only a runtime's own code may
  declare a syscall, so each one is found by its declaration.
- *It makes a pure host function impure to every caller.* The effect then
  spreads through every procedure that reaches it, up to procedures that
  are pure and used everywhere, which FX-26 gives no way to wrap.

**The case that asks for it.** Kayak's compiler joins its listing's
pieces into one string: a buffer sized once (`make-bytevector`), each
piece's bytes copied in, `utf8->string` once. Sizing it wants each
string's length in bytes, which FX-26 does not give (`string-length`
counts characters): `string->utf8` gives it only by copying. A runtime
whose strings are UTF-8 (kayak-rt's) has the length at hand; one whose
strings are characters (Fixpt's) counts each one's width, without
copying; so `kayak:string-utf8-length/1`, `(subr pure (string) int)`,
would be the natural syscall. (Corrected after: this note had Fixpt's
strings as UTF-8, which they are not.) Under the rule, it is `(subr (io sys) (string) int)`, and
the join, and every procedure that joins strings, all `pure` today, would
say `(io sys)`.

**What changes.**
- In both checkers, the check of a declaration's effect no longer
  requires `(io sys)`; the other atoms' rule is as it was. The message
  that names it goes.
- A call of a syscall has its declared type's effect, as now; a `pure`
  one's call is pure to the compilers, which may move, share or drop it as
  they do any pure call. The host must then be so (the coordination any
  syscall's type asks for).
- Tests: a `pure` syscall declared and called from a `pure` procedure
  (accepted, and a runtime that gives it at that type loads); the same
  syscall declared with `(io sys)` against a runtime that gives it `pure`
  (load fails: the types differ, as any other difference).
- `fx26.md`'s "its effect holds `(io sys)`, and only …" becomes "its
  effect only …".

**Not asked**: a `string-utf8-length` in FX-26 itself, a standard pure
operation, which would serve every program, not only a runtime's; worth
asking separately.

## Open questions

- Whether `(io sys)` should be one atom or one per namespace
  (`(io sys code)`), so that a type says which capabilities it reaches.
- Whether the record of declarations is per program or per module file
  (a runtime's library declaring them once, its users importing them).
- Whether `syscall-available?` should also take a host name, for a
  program that asks without declaring (likely not: declaring is the
  point).
