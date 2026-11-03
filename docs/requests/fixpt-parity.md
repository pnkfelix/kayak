# A request to the Fixpt team: two small fixes, found comparing with kayak

Status: drafted 2026-11-03 for the user to take to the Fixpt team.
Checked against Fixpt 63b95ef (kayak's pin) and again against
`origin/main` at dd369934: neither is fixed there.

---

kayak compares its checker and runtime with Fixpt's on Fixpt's own test
programs. Where they differed, kayak mostly changed to match Fixpt. These
two are the cases where we think Fixpt should change instead: each is a
plain improvement, and each comes with a minimal program.

## 1. `+` and `-` should name the argument that is wrong

Every call that Fixpt checks by its general rule (`k-app-arg` in
`check-rules.fx`) says which argument is wrong. `+` and `-` don't, because
they are typed by the rule for sizes instead:

```scheme
(+ 1 "x")
(* 1 "x")
```

```
$ fixpt --dialect fx26 check plus.fx
! plus.fx:1:6: a int is expected here, and this is a string
$ fixpt --dialect fx26 check times.fx
! times.fx:1:6: argument 2 is a string, where a int is expected
```

`(- 1 "x")` gives the same message as `+`. `*`, `quotient`, `=`, `<`,
`max` and the rest all say "argument N". We'd like `+` and `-` to say it
too, through the same rewording (`k-argument-error`) the general rule
uses:

```
! plus.fx:1:6: argument 2 is a string, where a int is expected
```

kayak already says this for `+` and `-`. Fixpt's expected outputs that
would change include `stage/session.fx` and `stage/kept.fx`, whose staged
`(+ 1 "x")` and `(+ y "a")` are refused this way.

## 2. `process/basic.fx` can lose the child's last output

`crates/fixpt-fx26/tests/programs/process/basic.fx` reads a child's output
until the child ends:

```scheme
(let ((more (string-append acc (process-output p))))
  (tagcase (process-status p)
    (exited c (string-append more "exited " (int->string c)))
    ...
    (running u (begin (sleep-until ...) (wait-for p more)))))
```

`process-status` says `exited` once the child has exited and its reader
has reached the end of the output. That can be after the program's last
`process-output` call. Whatever the child wrote in between is then never
taken. `world.rs` documents the protocol: "a last `process_output` after
gives the rest". `wait-for` never makes that last call.

We saw it once in kayak's checks under load: the result lost its
`got hi`. This program widens the window so it happens on every run:

```scheme
(define* wait-for (subr (maxeff (io process) (io clock) spin) (int string) string)
  (lambda (p acc)
    (let ((more (string-append acc (process-output p))))
      (begin
        (sleep-until (i64+ (clock) (int->i64 200000000)))
        (tagcase (process-status p)
          (exited c (string-append more (string-append "|exited " (int->string c))))
          (signalled s "signalled")
          (unknown u "unknown")
          (running u (wait-for p more)))))))
(tagcase (process-start (list "/bin/sh" "-c" "echo last; exit 3") "")
  (started p (wait-for p ""))
  (failed e "failed"))
```

```
$ fixpt --dialect fx26 eval race.fx
"|exited 3" : string ! ...
```

The fix is in the program: once the child has ended, take the output one
more time.

```scheme
(exited c (string-append more (string-append (process-output p) (string-append "exited " (int->string c)))))
```

With that change the reproduction gives `"last\n|exited 3"`.

`fixpt-cli/tests/programs/io/process-news.fx` has the same shape. Its
`until-ended` marks a process ended without taking its output once more.
We have not reproduced a loss there.
