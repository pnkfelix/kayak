# A request to the Fixpt team: a checker panic on a reshaped module in `list`

Status: found 2026-11-04 by kayak (C328), on Fixpt 19b31f10 (kayak's pin);
landed in Fixpt the same day (8616f2fe, its test `modules/reshape-in-list.fx`,
which found a silent case too: the same values in another order, the wrong one
taken); kayak's pin since C341.

## The panic

A module passed where a module type names fewer of its values, or orders
them differently, is reshaped (`k-reshape-at`). As an element of `list`, the
same call stops Fixpt's checker:

```scheme
(define-type ab (moduleof (val b int) (val a int)))
(define m (module (define a int 2) (define c int 4) (define b int 7)))
(define first-b (subr pure (ab) int) (lambda (x) (with x b)))
(list (first-b m))
```

```
$ fixpt eval min.fx
thread 'fixpt' panicked at crates/fixpt-fx26/src/infer.rs:420:55:
index out of bounds: the len is 2 but the index is 2
```

The backtrace runs through `Checker::expect` (building its `Vec<Pick>`),
`check_node`, `check`, `synth_app_of`. We'd expect `(7) : (listof int
acyclic)`.

The same call elsewhere is fine: `(first-b m)` gives `7`, `(cons (first-b
m) nil)` gives `(7)`, and `(let ((x (first-b m))) x)` gives `7`. So does
`(list (first-b m))` when `m` has exactly the type's two values, in either
order. The index (2) is `b`'s place in `m`, so it looks as if the picks
made for `m`'s reshape (three values to two) are applied again where
`list`'s element is checked, to a value that already has two.

kayak gives `(7)`, on its engine and its compiled tier.
`tests/fx26-kayak/modules/values-in-order.fx` keeps each reshape out of
`list` for now, so that Fixpt's answer can be recorded for it.
