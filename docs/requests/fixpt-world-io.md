# A request to the Fixpt team: operations on the world, for FX-26

Status: drafted 2026-10-12 for the user to take to the Fixpt team
(`PLAN.md` §8, "I/O, for a live runner", option (a)).

---

We'd like FX-26 programs to be able to do I/O: read and write files,
the terminal, and the clock. We'd like that to keep a program's meaning
deterministic, so that a run can be simulated, recorded, and replayed.
We propose no side-effecting primitives. Instead, a program performs an
operation by suspending to a prompt tag it is given, and the host
answers. Programs can already be written this way in FX-26 today. What
is missing is a standard set of operations, and a host that answers
them.

**1. A standard operation type and answer type**, as a built-in module
(for example `fx26:world`):

```scheme
(define-type op
  (sumof (read-line unit)            ; stdin, one line
         (write-out string)          ; stdout
         (write-err string)          ; stderr
         (read-file string)          ; path -> contents
         (write-file (productof (path string) (text string)))  ; replace whole, atomically
         (list-dir string)           ; path -> names, sorted
         (args unit)                 ; command-line arguments
         (clock unit)                ; monotonic, nanoseconds
         (sleep-until i64)))         ; ends at the clock's time given
(define-type answer
  (sumof (line string) (eof unit) (done unit) (text string)
         (names (listof string)) (time i64)
         (failed (productof (kind string) (detail string)))))  ; not-found, permission, ...
```

I/O errors are answers that the program sees, not traps. Each
operation is answered once.

**2. A convention for a program's entry point.** `main` takes the
world's prompt tag:

```scheme
(poly ((w region))
  (subr (maxeff (goto w) (comefrom w) ...) ((prompt-tag A request E w)) A))
```

Here a `request` is an `op` paired with the rest of the program, which
is resumed once with the `answer`. The effect system then shows which
procedures can touch the world: exactly those with control on `w`.

**3. A host that answers, chosen in Fixpt's CLI when the program is
run:**
- `--live` answers from the OS.
- `--sim FILE` answers from a script, with a virtual clock.
- `--record LOG` wraps another runner and writes each step, its
  operation, and its answer.
- `--replay LOG` answers from a log. It fails at the first step whose
  operation differs from the log's, and says which step.

**4. A specified log format**, so that other implementations can
replay Fixpt's logs and Fixpt can replay theirs. One s-expression per
step, `(step op answer)`, after a header that gives the arguments.

**Questions:**
- Would you rather have a distinguished effect such as `(io)` than a
  tag passed to `main`?
- Should file contents be bytes rather than strings? FX-26 has no bytes
  type yet. Until then, invalid UTF-8 could be a `failed` answer.
- What should the exit status be?

**Later:** subprocesses, sockets, signals, and terminal modes. These
would be more operations of the same shape.
