# A request to the Fixpt team: operations on the world, for FX-26

Status: drafted 2026-10-12 for the user to take to the Fixpt team
(`PLAN.md` §8, "I/O, for a live runner", option (a)).

---

**Context.** kayak is a compiler written in KFX, a sublanguage of FX-26
that transpiles to Rust. Every KFX program must give the same answer on
Fixpt as on its Rust translation. We check that on every commit, with
over 50 programs and Fixpt's own examples. We now have record and
replay, written in FX-26 as it is today
(kayak's `lib/runner.kfx`, `tests/kfx/replay/app.kfx`; they run on
Fixpt at 7851654). What we cannot write is a program that talks to the
real world: a terminal, files, the clock. FX-26 has no I/O, so such a
program has no FX-26 meaning, and its two implementations cannot be
checked against each other. The first program we need it for is
kayak's own compiler driver, which reads `kfx/*.kfx` and writes Rust.
Today that driver is hand-written Rust.

**What we ask for.** We ask for something small: a standard *world
interface*, plus a host runner in Fixpt that answers it. We do not ask
for side-effecting primitives. A program performs an operation by
suspending to a prompt tag that it is given, as Ahman and Bauer's
runners work and as Coopt's do. Everything below is already expressible
in FX-26. The new part is the host, which answers from the OS at the
top level.

1. **A standard operation type and answer type**, in Fixpt's prelude or
   a built-in module (`fx26:world`), so that every implementation means
   the same thing by them. A first set, enough for a compiler driver and
   a REPL:

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

   Errors are answers that the program sees, not traps. Only a runner's
   own failures end a run: a replay that diverges, or a log that ends
   early. Each operation is answered once, at one step.

2. **A typed shape for a program that uses the world.** We suggest a
   `main` that takes the world's tag:
   `(poly ((w region)) (subr (maxeff (goto w) (comefrom w) ...) ((prompt-tag A request E w)) A))`,
   where a `request` is an `op` paired with its rest, resumable once.
   This is what our `lib/runner.kfx` does now. The effect system then
   says which procedures can touch the world: exactly those with
   control on `w`. Pure code provably cannot.

3. **Runners in the Fixpt CLI**, chosen when the program is run:
   - `--live` answers from the OS;
   - `--sim FILE` answers from a script of answers, with a virtual clock;
   - `--record LOG` wraps another runner and writes each step, operation
     and answer;
   - `--replay LOG` answers from a log, and fails at the first step whose
     operation differs from the log's, saying which step.

4. **A log format both implementations share**, so that a run recorded
   by kayak's Rust replays on Fixpt and the reverse. This is how we would
   check a live program. Each line is an s-expression of what Fixpt
   prints and reads back: `(step op answer)`. A header gives the
   program's arguments. Writes are recorded so that replay can check
   them, but they are not inputs.

**Why this shape.** A program's meaning stays a function from the
answers it receives to the operations it performs. FX-26 stays
deterministic, Fixpt's checker needs no new effect form, and record,
replay and simulated testing come with it. kayak's implementation would
then be the same code we have now, plus a live runner in a small Rust
crate.

**Questions for you.**
- Would you rather give the world a distinguished region and effect,
  such as `(io)`, than pass a tag?
- Should file contents be bytes rather than strings? FX-26 has no bytes
  type, so for now we would treat invalid UTF-8 as a `failed` answer.
- Is `fx26:world` the right home for these types?
- What should `main`'s convention be, and its exit status?

**Later, not now:** running a subprocess (we need to run `rustc` and
`cargo`), TCP, signals, and the terminal's mode. These are the same
shape, with more operations.
