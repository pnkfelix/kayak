# A request to the Fixpt team: effects for code that must stay responsive

Status: proposed 2026-11-07 by kayak (C388), on Fixpt 13114974.

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

## 1. Sleeping apart from reading the clock

`clock` and `sleep-until` are both `(io clock)`. So a handler that must
read the time is also allowed to sleep, which a responsive one must not
do. We ask for a separate effect for sleeping, say `(io sleep)`, for
`sleep-until` and for a `pselect`'s `sleep-until` arm, leaving `(io clock)`
for reading the clock. A program that writes `(io clock)` where it sleeps
would need the new effect too.

## 2. A supertype of evidence that a syscall may take

This is the dynamic form. An event loop generic in its handlers' effect,
`(poly ((e effect)) …)`, cannot tell whether a handler may wait. Evidence
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
effect given as text. A program would ask `syscall-available?` and fall
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
