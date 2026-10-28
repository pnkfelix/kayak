# The checker's work in regions that end

Status: A done (C270, 2026-10-28), B's groundwork (C271); B by a module
made per form (below), whose modules made in procedures of places are
KFX's now (C274). After C261-C269. The user: "go for the larger
refactor. The reason to *have* arenas is to enable architectural choices
like this."

## Where things stand

kayak-rt has no collector: what a KFX program makes in the heap is kept
for the run. A `letrena`'s place is the one thing that ends: its pairs,
references, bloblets (C30, C34) and now bytevectors (C267) are freed when
its body does. C268 put the listing in places, and with C261-C269 kayak's
FX-26 compiler compiling the KFX compiler to cells went from 45 s and 55
GB to 2.57 s and 1.14 GB (Fixpt: 4.0 s, 0.49 GB). What is left is mostly
the checker's: the work of checking each top-level form, kept for the run
though only its result is needed after the form.

## What FX-26 allows, and what kayak lacks

FX-26 has datatypes with region parameters (`(define-datatype (rose (t
type) (r region)) …)`), whose values may hold that region's data; a value
whose type mentions `r` cannot leave `r`'s `letrena` (the `letrena` refuses
a value type mentioning its region). Products and sums are in no region in
FX-26 ("making one is pure"): where they live is the implementation's to
choose. So kayak may make a value of `(rose t r)` in `r`'s place, and free
it with the place, with nothing asked of Fixpt.

KFX erases a datatype's regions in Rust (C17): every value is a
`&'static` enum in the heap, its fields the heap's lists. That is the gap.

## The plan

### A. Datatypes in places (KFX, kayak-rt)

1. A datatype with region parameters is a Rust enum with a lifetime for
   each (`Rose<'r, T>`); its fields at those regions are at those
   lifetimes (`ListAt<'r, &'r Rose<'r, T>>`). A use at a region with no
   lifetime (`acyclic`, `@g`, a region binder not within a place) is at
   `'static`, as now.
2. A constructor call whose result is at a place's region, the place named
   as its region (as C267's bytevectors), is made in the place: the checker
   notes `rheld-data@'p T` (the place holds that enum, and a procedure given
   the place says so, C34) and `rdata-in p`; the emitter calls the
   constructor's `_in` twin, `leaf_in(p, …)`, which allocates in the
   place's arena. Elsewhere, the heap, as now (a `&'static` value fits
   where an `&'r` one is wanted: the enums are covariant).
3. Tests: a rose tree built in a `letrena`, summed, many times; a
   procedure given a place that builds one; Fixpt's answers.

### B. The checker's work per form in a place (the checker)

1. The checker's structures take a region parameter: its types (`ty`,
   `scheme`, `variant`, …), its bindings, its results (`res`), its
   substitution (`st`), its effect lists, its facts. Then each procedure
   over them is polymorphic in that region. Done a structure at a time,
   from the leaves (`ty`) up, each step checked whole (`scripts/check.sh`)
   with the region always `acyclic` at first: no behaviour changes until
   the last step.
2. `check-top` checks each top-level form in a `letrena` of its own: the
   scope's bindings read from the heap (a binding's type instantiated is
   copied into the form's region, as instantiation copies already), the
   form's work in the region, and what outlives the form copied out to the
   heap: the binding it defines (its scheme, zonked), its datatypes and
   aliases, its facts, its elaborated types.
3. Measured at each step against the KFX compiler compiled to cells, and
   the KFX compiler compiling itself (the checker is shared with it).

### Size

The checker is some 15,500 lines of KFX (`kfx/check.kfx`,
`check-exprs.kfx`, `check-defs.kfx`, `terminate.kfx`), 79 datatypes and
some 1,300 procedures. A is a contained change to the emitter and checker
of KFX; B is a long one, done in steps that each keep every check
passing.

## B by a module made per form (the user's, 2026-10-28)

The user: "Can we use module parameters to abstract over the place for a
group of items?" Threading the place through each of the checker's
procedures, and each call, is B's bulk. Instead: the checker one module
made in a procedure of the form's place, its items closing over it, so
that inside it `cons`, `list` and constructors find the place by name
(C270, C271) and nothing gains a parameter. KFX has no such modules yet
(its modules are Rust's `mod`s, C19, C24): a module made in a procedure,
in Rust a generic struct holding what it closes over, its items methods.
A module is one form, and the checker is four files: the user, "We may
want to just allowlist putting it all in one file for now" — one file,
over the size the others keep, by that decision.
