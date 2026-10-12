# Memory for every KFX program: a collector for kayak-rt?

Status: research, 2026-10-12. PLAN §6.3, 5 ("a long-running task needs its
data freed"); the user (2026-10-12): "maybe kayak-rt should grow an
incremental GC"; set beside the engine's own collector (C64), which is a
different thing: that one collects the interpreter's words, this one would
collect what kfx's Rust allocates for any KFX program. No code yet.

## Where things stand

- **C43**: a KFX value is `Copy`: a scalar, a `Str` (a leaked `&'static
  str`), or a `&'static` reference into a typed arena (pairs, references,
  sums, datatypes), bumped and never freed; a closure is a leaked
  `Box<dyn Fn>`; a suspended computation a `Pin<Box<dyn Future>>` (C45).
  Regions (C38): a `letrena`'s arena is freed when its body ends, and rustc
  checks that nothing of it escapes, by its lifetime.
- **Measured**: the compiler on itself, 0.9 s, 586 MB resident (it was
  7.8 GB before C44's cuts). For a batch compiler, never freeing is
  cheap and fast. A REPL, a server, a long compile of many files, or a
  program of many tasks (C47's) grows without end.

## What a collector of Rust-compiled KFX must find

The roots: every live reference, which is in a Rust stack frame (a local,
a temporary), in a closure's captures (in a `Box`), in a suspended
future's state (in a `Box`, its locals across an `.await`), and in the
globals (`thread_local!` constants, C21). Rust does not say where they
are: there are no stack maps.

## The options

1. **Conservative, non-moving, Boehm's way**, in one runtime crate with
   `unsafe` (Q7 allows it there): every allocation, closures' and futures'
   too, from a heap kayak-rt owns (as the process's global allocator, or
   the emitter's calls); at a collection, the stacks and the registers
   scanned for words that look like pointers into it, then a mark through
   the heap, conservatively too, and a sweep.
   - No change to the emitter; futures and closures covered for free.
   - `unsafe`, per platform (stack bounds, registers spilled); a word that
     looks like a pointer keeps garbage; no moving, so fragmentation.
   - Incremental needs write barriers (every `set` and `Cell` write, which
     the emitter would add) or page protection, as Boehm's does.
2. **Precise, roots the emitter keeps**: the emitter knows every local's
   type, so it could push the references a frame holds on a shadow stack,
   one per task (tasks interleave), and generate `Trace` for every type;
   closures as types the emitter writes, not Rust's, so their captures can
   be traced.
   - Safe, exact, could move.
   - The emitter rewritten throughout; every call pays; a future's locals
     across `.await` need their own frames, close to CPS by hand.
3. **Regions, as FX-26 has them** (FX-87's lineage): memory freed by scope,
   statically checked, which KFX has already (`letrena`, C38, its arenas
   freed at the end, rustc proving nothing escapes). No collector: a
   program says where its data dies. Grown into:
   - **a region per task or per query** (Erlang's heap per process; Coopt's
     queries): a task's data in its own arena, freed when it ends; what it
     gives back copied out (a deep copy the emitter generates, or text, as
     C54's memos already are);
   - the compiler's own phases in regions: a form's checking and emitting
     in a region freed after it, only its Rust (a string) kept.
   - Safe, no `unsafe`, no barriers, no pauses; FX-26's own discipline,
     so K1 holds by construction.
   - Only what a program scopes is freed: data that lives long and dies
     unpredictably (a cache, a graph edited in a REPL) needs more.
4. **Reference counting** (`Rc`): every value `Clone` not `Copy`, the
   emitter's whole output changed, a count at every copy, and cycles (FX-26
   makes them with references and `set-cdr!`) leaked anyway. Not
   recommended.

## Recommendation

**Regions first (3)**, since they need no `unsafe`, no barrier and no
pause, are FX-26's own semantics, and have the safety proof (rustc's) for
free:

1. Measure where the compiler's memory goes by phase (C44's `heap-stats`).
2. A region per form for its checking and emitting, the result kept as a
   string (which C54's memos are already): the compiler's peak should fall
   from the whole program's to a form's.
3. A region per task in `lib/tasks.kfx`, its result copied out: the shape
   a long-running program needs (a server's request, a REPL's input).

**A tracing collector (1) later, if regions prove not enough**, in one
runtime crate with its `unsafe`, conservative and non-moving first, Boehm
as the model, with an incremental mode by write barriers the emitter adds
at `set`. Precise collection (2) is the larger rewrite, and buys moving,
which the engine's collector (C64) already has where it matters most.

Questions for the user: whether a REPL of KFX itself (not the engine's,
C65) or a long-running KFX service is near, which decides how soon (1) is
wanted; and whether regions per task match how you want tasks used.
