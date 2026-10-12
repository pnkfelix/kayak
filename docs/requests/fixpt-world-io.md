# A request to the Fixpt team: I/O and bytes for FX-26

Status: drafted 2026-10-12 for the user to take to the Fixpt team
(`PLAN.md` §8, "I/O, for a live runner", option (a)).

---

We'd like FX-26 programs to be able to do I/O: files, the terminal, the
clock, and command-line arguments. That needs bytes too (section 3). We propose primitives that perform
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
(read-file path)         ; (io fs-read)  -> (sumof (data bytevector) (failed io-error))
(list-dir path)          ; (io fs-read)  -> (sumof (names (listof string)) (failed io-error))
(write-file path data)   ; (io fs-write) -> (sumof (done unit) (failed io-error))   ; data a bytevector; replaces whole, atomically
(clock)                  ; (io clock)    -> i64, monotonic nanoseconds
(sleep-until t)          ; (io clock)    -> unit
(args)                   ; (io args)     -> (listof string)

(define-type io-error (productof (kind string) (detail string)))   ; not-found, permission, ...
```

I/O errors are values that the program sees, not traps. `list-dir`
returns names sorted, so that its answer does not depend on the file
system's order.

**3. Bytes: `u8` and `i8`, and bytevectors as a full type.** Files are
bytes, so the file primitives above take and give bytevectors.
- `u8` and `i8` join `i32`, `u32`, `i64` and `u64`, with the same table
  of wrapping operations, `int->u8`, `u8->int`, and the rest. We need
  `u8`; `i8` is for symmetry.
- A `bytevector` is today opaque, made only by the reader. We would need
  to make one and read it: `(make-bytevector n u8)`, `bytevector-length`,
  `(bytevector-u8-ref bv i)` giving a `u8`, `bytevector-copy` of a range,
  `bytevector-append`, and `bytevector=?`.
- A way to build one: either `bytevector-u8-set!` on a bytevector at a
  region, with `(write r)`, or a builder type. We have no preference.
- Conversions: `(utf8->string bv)`, giving
  `(sumof (text string) (invalid i64))` where `invalid` is the offset of
  the first bad byte; and `(string->utf8 s)`.

**4. Running them for real**: `fixpt eval` and `fixpt run` would perform
these primitives against the OS. Nothing more is needed; we will build
simulation, recording, and replay ourselves, as FX-26 libraries.

**Questions:**
- Is this granularity right? A family indexed by more than a name (say
  `(io fs-read "dir")`) is more than we need.
- What should the exit status be, and should a program be able to set
  it?

**Later:** subprocesses, sockets, signals, and terminal modes, as more
members of the family.
