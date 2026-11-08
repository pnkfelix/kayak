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

## 2. Evidence that can be inspected

This is the dynamic form. An event loop generic in its handlers' effect,
`(poly ((e effect)) …)`, cannot tell whether a handler may wait. Evidence
reflects the effect into a runtime object, `(effectrep e)`, but nothing
can look inside one: it is made only by `reify-effect` and used only by
`stage`.

We ask for a test of evidence against an effect that also narrows a
procedure's type when it passes, so that the loop can call a wait-free
handler directly and hand any other to a worker:

```
(effect-narrow f er (reify-effect E))
  ; f   : (subr e (A …) R)
  ; er  : (effectrep e)
  ; ->    (sumof (within (subr E (A …) R)) (beyond unit))
```

`within` if `e`, as `er` is evidence of it, is within `E`; else `beyond`.
A test that gives only a `bool` would not be enough, because the call
must still check against the loop's declared effect.

## 3. Bounded effect variables

This is the static form, and it makes (2) unnecessary where the bound is
known when the program is written:

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
