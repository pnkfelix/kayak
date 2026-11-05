# What staged code and sessions keep, and giving it back

Status: plan (2026-11-05), the user's ("a session once unreachable could be
GC'ed and its code eventually collected"). Not started; type reps first
(the user's, the same day: parity with Fixpt).

## What Fixpt does

Two paths (checked at Fixpt c3dc2406):
- **Its compiled machines** (`sessions26.rs`, since 6ddd924d, 2026-10-31,
  in kayak's pin): a session is all in the heap. Its front end, the
  conductor's `make-session-compiler`, is the session's third field; its
  globals are cells of that front end's compiler (`wglobal`,
  `compile-new-global`, `compile-programs.fx`); its code is its words',
  each word's code a bloblet its entry keeps (516abc8e), and the machine
  keeps nothing of a word. An unreachable session is collected, code and
  all. A plain `stage`'s words likewise: reached from the thunk's closure
  alone.
- **Its Rust checker's path** (lowered, `rs-scheme-vm`; evaluated): the
  session's checker state is in a thread-local `SESSIONS` vector, by
  index, never shrunk. This grows, as kayak does.

So nothing in FX-26's API keeps a session alive; kayak copied the path
that grows (C311, the engine's sessions, from Fixpt's lowered path), and
added one of its own (every word named).

## What kayak keeps

1. **Every word a listing reads, named** (`listing-reader.kfx`'s
   `words-made!`, `named!`): `named-values` is a root of every collection
   (`named-forwarded!`), so a staged word, and its machine code (C344), is
   never freed, a plain `stage`'s too, its thunk long dead.
2. **A session's globals, the machine's** (tier 1's, and the engine's):
   `fx26-session-K:NAME`, in the one table of globals, never let go.
3. **A session's checker state and its pendings** (`sessions`, `pendings`
   in `staging.kfx`): every session's, every definition checked.

## The plan

1. **A stage's words unnamed once read.** `listing-added!` resolves the
   listing's references by name while it reads; then the words it made
   reach one another by their cells and the program reaches them by the
   thunk's closure. Name them in a table of the read alone, not
   `named!`; what the program's listing named stays named. Test: a
   program staging in a loop under `--gc-stress` keeps a bounded heap and
   code area (`--gc-stats`), on every tier; `made-code-x86_64/code-freed.fx`'s
   kind of test for a staged word's code.
2. **A session's globals the session's.** Cells the session value holds
   (as Fixpt's `wglobal`s are its front end's), which its words' code
   reads through, not tier 1's table; the engine's likewise.
3. **Its checker state let go with it.** kayak's front end in tier 1 is
   KFX compiled to Rust (C335), so its checker state is not in the heap
   tier 1 collects, unlike Fixpt's (FX-26 code in the same heap). The
   session value keeps an index of it; a weak reference to the session
   (tier 1's collector has weak handling, `weak-seen`) says when it died,
   and its state and pendings are dropped then.
4. Measure a session-heavy loop's memory before and after.

The user's tree of environment frames (2026-11-03), each session a leaf
whose frame goes when it is unreachable, is what Fixpt's heap session
amounts to: its globals the session's, found through it.
