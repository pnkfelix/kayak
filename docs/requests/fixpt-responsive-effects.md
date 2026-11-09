# A request to the Fixpt team: effects for code that must stay responsive

Status: proposed 2026-11-07 by kayak (C388-C391), on Fixpt 13114974.
Answered 2026-11-09 by Fixpt's design note `docs/research/effect-markers.md`
(Fixpt a95025ad), nothing built yet: properties of effects as markers
derived from their atoms by a fixed table (`nonblocking`, `terminating`,
`deterministic`, `wait-free`), not an atom `wait`; `(io clock)`, `(io
stdin)`, `(io process)` split into their waiting and non-waiting parts;
masking a wait leaves `spin`, so a private wait is not `wait-free`;
bounded effect variables with markers as a second kind of bound; and
`effect-case`, a test of evidence that narrows `e`, in place of `rep`.
kayak's answers to its questions (C392): the standard streams' writes
honest, waiting; `rep` not needed; `comefrom` no wait; `nonblocking` wanted
for code run to its end, a private wait there failing as it runs.

## What we want to say

A procedure that must stay responsive (a key handler, an event loop's
step) should never wait itself: what would wait goes to a process of its
own, and the answer comes back as a message or an I-cell, taken at one
place. FX-26 can say most of this already. Waiting is an effect
(`(receive r)`, `(await r)`, `(io stdin)`, a `comefrom`), a procedure's
declared effect bounds what it calls, and `spawn` keeps a process's waits
out of its spawner's effect. So a handler of effect `(maxeff (send @w)
(alloc @r))` cannot call anything that waits, and your checker says so.
kayak's `tests/kfx/responsive.kfx` and
`tests/kfx-refused/responsive-handler-waits.kfx` are a small example of
each.

Three things are missing. Each is a change to FX-26, so we ask before
kayak uses any of them.

## 1. An effect `wait`

Waiting is in FX-26's effects already, but by kind: `(receive r)`,
`(await r)`, `(io stdin)`, and `(io clock)`, which `sleep-until` shares
with `clock`. So a handler that must not wait has to leave out each kind
by name, and one that must read the time is also allowed to sleep.

We ask for one effect atom, `wait`, beside `spin` and `nondet`, which
every operation that may block until something else happens has as well
as its own atoms: `receive`, `icell-get`, a `pselect`, `read-line`,
`read-byte`, `read-bytes`, `sleep-until`, `process-exit`. What has it is
not wait-free, in the sense of wait-free algorithms: it may not finish
until another party acts, or time passes. A procedure that must stay
responsive is then one of an effect without `wait`, and `clock`, which
does not wait, no longer goes with `sleep-until`. A write that blocks
until the other end takes it we would count too: `process-write`, and
`write-out` and `write-err`, which wait when what they write to is a pipe
no one is reading (as they do in kayak's async mode, where such a write
is a task waiting on its runner). File operations, bounded by the machine
rather than another party, we would not. That is your call. It is a bare
atom, not an `(io …)` one, as most waits are on other processes, not on
the world.

With it, wait-freedom in the usual sense (every call ends in a finite
number of its own steps, whatever others do) is an effect without `wait`
and without `spin`. Without `wait`, a procedure blocks on no one; but
it may still loop until another party moves (polling `stdin-ready?`, say),
which never blocks and still depends on others. Without `spin` too, your
termination check has found that it ends; and as FX-26 has no mutable
memory processes share (what crosses is transmissible and never written;
mailboxes and I-cells are reached only by what waits; `par`'s branches'
writes are checked for interference), how long it takes depends on no
one else. That is wait-freedom, not a bound on its steps, which no effect
gives.

## 2. A supertype of evidence that a syscall may take

This is the dynamic form. An event loop generic in its handlers' effect,
`(poly ((e effect)) …)`, cannot tell whether a handler may wait, whether
`e` holds `wait`. Evidence
reflects the effect into a runtime object, `(effectrep e)`, but nothing
can look inside one: it is made only by `reify-effect` and used only by
`stage`. Syscalls are FX-26's way for a runtime to add a capability
without a change to the language, but none can take evidence. A
syscall's type may be polymorphic over regions alone, so it cannot name
`(effectrep e)` for any `e`; and its arguments are plain data, compared as
canonical text.

We ask for one standard type, say `rep`, that all evidence is a subtype
of: `(effectrep e)`, `(typerep t)`, `(regionrep r)`, `(effectbound e)`
and `(typebound t)`. `rep` has no parameter, so a syscall's type may name
it and still be compared as text, as `(subr (io sys) (rep string) bool)`.
It would cross to the runtime as what evidence already is at run time,
the canonical text of what it is evidence of. A `rep` can be inspected
but never made evidence again: there is no conversion back, so a syscall
can learn what evidence says but never forge it.

With it, a runtime could offer an optional syscall such as
`kayak:effect-within/1`, which says whether evidence's effect is within an
effect given as text: whether a handler's holds `wait`, say. A program would ask `syscall-available?` and fall
back where a runtime has none, so nothing about Fixpt's runtime need
change unless you want it to. kayak would offer one only once FX-26 has
`rep`.

What a syscall cannot do is narrow a procedure's static type: telling
the program that a handler may not wait still leaves its type at `e`, so
the checker still refuses calling it from code of a wait-free effect. For
that there is `stage`, which checks code against a type made of evidence
as the program runs and gives the code at that type (kayak's engine runs
it; its Rust build has no checker at run time), and the static form,
which follows.

## 3. Bounded effect variables

This is the static form. It narrows where the bound is known when the
program is written:

```
(poly ((e effect (within (maxeff (send @w) (alloc @r))))) …)
```

An instantiation beyond the bound is refused as the program is checked,
and in the body `e` stands where its bound is wanted. Evidence bounds
(`effectbound`) are bounds from above at run time; this would be the same
at check time.

## What kayak does meanwhile

Without asking anything of FX-26:
- The places where kayak's Rust runs code to its end where nothing may
  wait are a constant's value, a native procedure's callback, and a
  monitor's `on-failure`. The first is already only reads of globals and
  allocation in KFX, and the third already `pure`. For the second, KFX now
  refuses a native procedure type whose procedure parameters may wait
  (`tests/kfx-stricter/native-callback-waits.kfx`, which your checker
  accepts).
- At run time, kayak-rt's `block` now says "a wait where nothing may wait"
  rather than speaking of a missing prompt, if one ever gets there.
