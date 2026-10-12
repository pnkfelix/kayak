# Suspension in KFX, as Rust futures

Status: research, 2026-10-12, with a prototype built
(`crates/kayak-rt/src/suspend.rs`, its tests Fixpt's examples written by
hand as the emitter is to write them). `PLAN.md` §6.5, step A1. Decisions:
`CLAUDES_CHOICES.md` C45.

Read: Fixpt's `docs/research/async.md` and its examples
(`docs/research/examples/async/`), `docs/fx26.md` "Control, typed", at the
pinned `7851654`; Coopt's `PLAN.md` (I1–I5, §3.3, §3.5),
`crates/coopt-fx26/lib/async.fx` and `docs/research/runners.md`, at
`cce7e87`.

## Findings first

1. **A suspension is a capture followed at once by an abort to the same
   tag**, and that is a Rust future returning `Pending`. Fixpt's examples
   suspend a task so: `(call-with-composable-continuation (lambda (k)
   (abort-current-continuation sched (file k))) sched)`. The continuation
   up to the prompt is then exactly the prompt's body, suspended at the
   capture: the body's future. The prompt takes it as `k`; resuming `k`
   polls it again. rustc writes the state machine; nothing is interpreted.
2. **The scheduler is the program's own, and KFX.** In Fixpt's examples
   the run queue, `spawn!`, `run!`, promises, are FX-26 over prompts: no
   executor in Rust at all. The prototype runs `ping-pong.fx` and
   `promise.fx`, written as KFX's Rust would be, with Fixpt's answers.
   What Rust must provide is about 150 lines: the capture, the prompt that
   polls, and the top level. That is K7 at its best: the policy all KFX,
   the mechanism a page of Rust.
3. **A Rust future resumes once; FX-26's continuation, any number of
   times.** Resumed twice, FX-26 runs the rest twice (Fixpt: 32, below),
   and the future cannot. So KFX does not take
   `call-with-composable-continuation` as it is: K1 forbids a construct
   that means something else. It takes a form of its own,
   `abort-with-continuation`, defined in `kfx/prelude.fx` in FX-26 with a
   `k` that fails at a second resume, as kayak-rt's does. Checked on
   Fixpt: one resume answers, two fail with the same message.
4. **Effects decide the code, as for escapes (C18).** A procedure whose
   effect has `(comefrom r)` is an `async fn`; its calls are `.await`ed.
   One with `(goto r)` alone stays a plain `fn` returning `Result`; one
   with neither, a plain `fn`. A function's colour is its effect, which
   the checker already knows.
5. **The `Waker` has nothing to do until the outside world is waited
   on.** A capture with no prompt is an error, as in FX-26. When runners
   come (record and replay, `PLAN.md` §6.1), an operation on the outside
   is a capture to the *runner's* tag, whose prompt is the top level, in
   Rust: it performs the operation, logs its answer at the step, and
   resumes. Record and replay live there, in one place.
6. **Coopt's library cannot be Fixpt's.** It uses Coopt's own primitives
   (`effect-handle`, `%one-shot!`, `%preempt-key`, `%no-preempt`,
   `%section-close!`, `clock-now`, `sleep-until`), which Fixpt has not
   (continuation marks Fixpt has). So kayak's library is its own, in KFX,
   smaller, after Fixpt's examples, and runs unchanged on Fixpt: Q6 is
   settled without async operations in the prelude, beyond the one form.

## 1. What FX-26 has

From `fixpt/docs/fx26.md`, "Control, typed": `(prompt-tag A H D R)`;
`(prompt tag body handler)`, syntax, masking `(goto R)` and
`(comefrom R)`; `abort-current-continuation`, `(goto R)`;
`call-with-composable-continuation`, `(comefrom R)`, giving a
`(composable T A D R)`, which is a `(subr (maxeff D (goto R) (comefrom R))
(T) A)` whose marks can also be read.

Fixpt's async note builds tasks on them (its §3.1: suspension is a
control effect on the loop's region; §6.1, the operations). Its examples
run on Fixpt today: `ping-pong.fx`, `promise.fx`, `cancel.fx`, and others.

## 2. The mapping

`suspend.rs`:

| FX-26 | Rust |
|---|---|
| a procedure of effect `(comefrom r)` | `async fn … -> Result<T, Unwind>` |
| `(abort-with-continuation tag f)` | `abort_with_continuation(tag, f).await?`: registers `(tag, f)` in a thread-local, returns `Pending` once; resumed, the value given |
| `(prompt tag body handler)`, body may capture | `match prompt(tag, Box::pin(body)).await? { Done(v) => v, Aborted(h) => handler(h) }`: polls the body; on `Pending` with a capture to its tag, takes the body as `k` and makes the payload `f k` |
| `k`, `(composable V A D r)` | `&'static Cont<V, A>`: the body's future, taken at its one resume |
| `(k v)` | `k.resume(v).await?`: the value given to the capture, the body polled until it ends or suspends again |
| the top level | `run(Box::pin(main))`: one poll with `Waker::noop()` |

A capture to another tag passes a prompt by (`Pending`, outward), and
takes the inner prompt with it: resumed, an abort to the inner tag finds
it. The test `inner_prompt_kept` is that case; Fixpt gives the same 701:

```scheme
(prompt outer
  (prompt inner
    (begin (call-with-composable-continuation
             (lambda ((k (composable unit int E @p))) (the unit (abort-current-continuation outer k)))
             outer)
           (abort-current-continuation inner 7))
    (lambda (v) (* v 100)))
  (lambda (k) (+ (k #u) 1)))     ; 701
```

Plain aborts are still `Result` (C18), and pass through `async fn`s as
they are, `.await?`.

## 3. Once, and K1

```scheme
(prompt tag
  (+ (call-with-composable-continuation
       (lambda ((k (composable int int E @p))) (the int (abort-current-continuation tag k)))
       tag)
     1)
  (lambda (k) (+ (k 10) (k 20))))     ; Fixpt: 32
```

The future of the body can be polled to its end once. So KFX's form,
`kfx/prelude.fx`'s meaning of it:

```scheme
(define abort-with-continuation
  (poly ((v type) (a type) (h type) (d effect) (r region) (e effect))
    (subr (maxeff e (goto r) (comefrom r) (alloc r))
          ((prompt-tag a h d r)
           (subr e ((subr (maxeff d (goto r) (comefrom r) (read r) (write r)) (v) a)) h))
          v))
  (plambda ((v type) (a type) (h type) (d effect) (r region) (e effect))
    (lambda (tag f)
      (call-with-composable-continuation
        (lambda ((k (composable v a d r)))
          (let ((used (the (ref bool r) (new #f))))
            (the v (abort-current-continuation tag
              (f (lambda ((x v))
                   (if (get used)
                       (begin (error "a continuation resumed twice") (k x))
                       (begin (set used #t) (k x)))))))))
        tag))))
```

Fixpt checks it, and on the case above fails, "a continuation resumed
twice"; resumed once (`(+ (k 10) 100)`), 111, as the Rust. `k` is a
procedure here, not a `composable`: its marks cannot be read, which KFX
does not do. Its effect says what resuming does: the body's, the tag's
control, and the flag, at the tag's region.

KFX's checker gives the form its type by a rule of its own, as it does
for `prompt`: KFX has no polymorphism over effects, and the prelude's is
FX-26's.

## 4. What is KFX, and what Rust (K7)

KFX: the scheduler (run queue, `spawn`, `run`, `yield`, `sleep` on a
virtual clock, promises, channels, nurseries); the runners' logic where
it is policy (the sim's clock and delays). Rust: `suspend.rs` (capture,
prompt, top level), and later the runners' mechanism (live I/O; the log's
bytes). The prototype's tests are the KFX side written by hand; the
emitter will write them from KFX.

## 5. Costs, and what is open

- **A box per prompt body, and per suspending procedure value** (a
  thunk spawned is a `Pin<Box<dyn Future>>`). A non-tail recursive
  `async fn` needs a `Box::pin` per call; tail calls are loops already.
- **A capture leaks** its `Cont` and its value's cell (two small
  allocations), as the heap does everything (C43); a body's future,
  boxed, is freed when it ends. A long-running server would grow: the
  same question as the heap's (`PLAN.md` §6.3, 5).
- **Regions.** `Body` is `'static`: a suspension inside a `letrena` is
  refused, which is Fixpt's note's own rule (finding 4 there); a
  `letrena` around the loop is a later relaxation (`Body<'r>`).
- **Marks.** Coopt's cancel scopes and task-local context are
  continuation marks, which Fixpt has and KFX refuses (C18). Marks travel
  with a captured continuation; in Rust a prompt would save and restore
  the mark stack around each poll of its body. Raised with the user when
  cancel scopes need it (`PLAN.md` §8).
- **Preemption.** None: a task runs to its next suspension. Fuel would
  be a capture at loop back-edges, an `async fn` for every `spin`
  procedure.
- **Engine slices** (phase C): the FX-26 engine's task is a future that
  returns `Pending` when its fuel is spent, under the same top level.

## 6. Next

1. `CLAUDES_CHOICES.md` C45, and `abort-with-continuation` in
   `kfx/prelude.fx` and kayak-rt.
2. The checker: the form's rule, `(comefrom r)` in effects, prompts whose
   body may capture, suspension refused inside a `letrena`.
3. The emitter: `async fn` by effect, `.await` at calls, `prompt` as
   above, suspending procedure values, `run` at the top.
4. The corpus: Fixpt's `ping-pong.fx` and `promise.fx` as KFX, compared
   with Fixpt; then a library of tasks in KFX.
5. Runners, record and replay, at the top level.
