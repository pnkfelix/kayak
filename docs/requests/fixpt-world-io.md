# A request to the Fixpt team: I/O for FX-26

Status: drafted 2026-10-12 for the user to take to the Fixpt team
(`PLAN.md` §8, "I/O, for a live runner", option (a)).

---

We'd like FX-26 programs to be able to do I/O: files, the terminal, the
clock, and command-line arguments. We propose primitives that perform
it, each with an effect from a fixed family of I/O effects, so that the
checker shows which procedures can touch the world, and in what way.

**1. A fixed family of I/O effects**, as effect atoms beside `read`,
`write` and `alloc`:

| effect | covers |
|---|---|
| `(io stdin)` | reading standard input |
| `(io stdout)` | writing standard output |
| `(io stderr)` | writing standard error |
| `(io fs-read)` | reading files and directories |
| `(io fs-write)` | writing files |
| `(io clock)` | reading the clock, and sleeping |
| `(io args)` | the command-line arguments |

These would combine with `maxeff` and appear in `subr` types, as other
atoms do. A procedure whose effect has none of them can do no I/O. We
picked this granularity for the operations below; a coarser or finer
family would do as well, so long as it is fixed.

**2. Primitives, each with its effect:**

```scheme
(read-line)              ; (io stdin)    -> (sumof (line string) (eof unit) (failed io-error))
(write-out s)            ; (io stdout)   -> (sumof (done unit) (failed io-error))
(write-err s)            ; (io stderr)   -> (sumof (done unit) (failed io-error))
(read-file path)         ; (io fs-read)  -> (sumof (text string) (failed io-error))
(list-dir path)          ; (io fs-read)  -> (sumof (names (listof string)) (failed io-error))
(write-file path text)   ; (io fs-write) -> (sumof (done unit) (failed io-error))   ; replaces whole, atomically
(clock)                  ; (io clock)    -> i64, monotonic nanoseconds
(sleep-until t)          ; (io clock)    -> unit
(args)                   ; (io args)     -> (listof string)

(define-type io-error (productof (kind string) (detail string)))   ; not-found, permission, ...
```

I/O errors are values that the program sees, not traps. `list-dir`
returns names sorted, so that its answer does not depend on the file
system's order.

**3. Running them for real**: `fixpt eval` and `fixpt run` would perform
these primitives against the OS. Nothing more is needed; we will build
simulation, recording, and replay ourselves, as FX-26 libraries.

**Questions:**
- Is this granularity right? A family indexed by more than a name (say
  `(io fs-read "dir")`) is more than we need.
- Should file contents be bytes rather than strings? FX-26 has no bytes
  type yet. Until then, invalid UTF-8 could be a `failed` answer.
- What should the exit status be, and should a program be able to set
  it?

**Later:** subprocesses, sockets, signals, and terminal modes, as more
members of the family.
