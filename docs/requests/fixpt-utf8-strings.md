# A request to the Fixpt team: strings in UTF-8

Status: drafted 2026-11-22 for the user to take to the Fixpt team
(`TODO.md` §54, step 2's string gap; the user's choice of the three
options, "propose UTF-8 to Fixpt", with the argument that UTF-8 is table
stakes for interchange).

---

We'd like FX-26's frozen strings to hold UTF-8 text in the heap, in place
of UTF-32. What a program sees would not change: strings are still indexed
by character (code point), and `string-ref`, `string-length`, `substring`
and the rest mean what they mean now. Only the layout and the cost model
change, and we propose a layout that keeps the common cases O(1).

## Why

**UTF-8 is what everything outside the heap speaks.** Files, terminals,
pipes, sockets, JSON, source text, command-line arguments and environment
variables are UTF-8 on every platform FX-26 targets. So are Rust's `&str`
and C's `char *` in practice. With UTF-32 in the heap, every crossing is a
conversion:
- `utf8->string` and the reader decode and re-encode every character;
- `string->utf8`, every write to a port and every printed message encode
  every character again;
- a runtime written in Rust (kayak's, Fixpt's) can't hand a string's text
  to a Rust function without first building a copy.

With UTF-8 in the heap, `utf8->string` is a validation and a copy of
bytes, `string->utf8` is a copy of bytes, and a runtime can lend the text
as a `&str` with no copy at all.

**Text is mostly ASCII, and UTF-32 spends four bytes on each character.**
Source code, symbols' names, identifiers, messages and most data are ASCII,
or nearly so. In UTF-32 their text takes four times the bytes it does in
UTF-8, and a string's text is a large share of what many programs hold:
kayak-rt's strings held 125 MB on kc's expander path (C508's census).
Four times the bytes is also four times the memory bandwidth for
comparing, hashing, searching and copying.

**Places are already converted at the edges.** Fixpt's front end counts
places in characters against texts in UTF-8 (`fixpt_fx26::chars`), and
`docs/research/heap-syntax.md` says a place is also an index into the text
read. That stays true under this proposal, since strings are still indexed
by character; what changes is that the heap's text and the file's text
would be the same bytes.

## What we propose

**1. A frozen string's suffix is UTF-8.** `string`, that is `(stringof
const)`, and what `letfreeze` makes of a `(stringof r)`.

**2. Its one field is its length in characters**, as a fixnum. Fixpt's
table already allows a string "a length field". The header already gives
the length in bytes. So:
- `string-length` is a field read, O(1), as now.
- A string is ASCII exactly when its length in characters equals its
  length in bytes. No flag is needed, and a writer can't make the two
  disagree.
- `string-ref` of an ASCII string is a byte read, O(1).

**3. A non-ASCII string past a threshold carries breadcrumbs**: the byte
offset of every 64th character, made once when the string is made, kept
after the text in the suffix (or as a second field, a bytevector). Then:
- `string-ref` of any string is O(1): find the breadcrumb, then step at
  most 63 characters.
- The obvious loop `(string-ref s i)` for `i` from 0 to `n` is O(n), not
  quadratic. That answers the concern in `fixpt-heap/src/heap.rs`, "UTF-8
  would make the obvious character loop quadratic".
- The breadcrumbs cost 4 bytes per 64 characters, under 2% of the text in
  the worst case. Short strings (under the threshold, say 64 characters)
  need none, since stepping through them is already bounded.
- They are made with the string and never written after, so a frozen
  string stays immutable: no lazily built index, no cache to invalidate,
  nothing for two threads to race on.

**4. Mutable strings, `(stringof r)` at a region, keep a fixed-width
layout**: UTF-32 as now, under a kind of their own. `string-set!` stays
O(1) and never has to resize text when a character's width changes.
- We see two ways to freeze one:
  - **(a)** `letfreeze` re-encodes it as UTF-8. This costs a copy, which
    the language today promises not to make ("frozen by `letfreeze`, a
    `string`, nothing copied", `tests/programs/run/stringof.fx`).
  - **(b)** A frozen string may be in either layout, told apart by its
    kind; readers check the kind, one compare of the header's kind bits.
    Nothing is copied, but every string operation takes the check.
- We lean to (b) as the smaller change to the language, and would follow
  Fixpt either way.

**5. Everything else is the same.** `char` is still a code point, string
literals mean what they mean now, and the checker's types are unchanged.

## What it costs Fixpt

We've looked for every place in the pinned Fixpt (6f7bd739) that depends
on UTF-32 strings:
- `fixpt-heap`'s `make_string`, `string_with` and the string readers;
- the native layouts, `native-direct-x64-layout.fx` and
  `native-direct-arm64-layout.fx`, whose string primitives read and
  write the UTF-32 suffix;
- `eval-prims.fx`'s string primitives, the printer and the reader;
- `object-model.md`'s table ("UTF-32 suffix") and its settled question,
  and `PLAN.md`'s decision 5, "Strings stay UTF-32".

kayak would carry the same change in kcrt-core and kayak-jit, which lay
strings out as Fixpt does.

## What kayak has found running UTF-8

kayak-rt has kept strings in UTF-8 since C482, with an ASCII bit and,
for non-ASCII strings, a cursor at the last character found rather than
breadcrumbs. Every program of the corpus, of Fixpt's programs that kfx
accepts, and of kc's own build gives Fixpt's answers on it. The cursor is a thread's cache,
which is why we propose breadcrumbs for Fixpt instead: they're immutable
and so safe to share between threads.

## Precedents

- **Swift 5** (2019) moved its native strings from UTF-16 to UTF-8, for
  interoperability with C and the system and for memory, and kept O(1)
  UTF-16 offsets for Objective-C with breadcrumbs, as here.
- **PyPy** (2019) moved its strings to UTF-8 under Python's code-point
  indexing, with an index of character offsets beside the text.
- **Rust, Go and Julia** keep strings as UTF-8 throughout, indexed by
  byte.
- On the other side, Chez Scheme and Racket CS keep UTF-32, which is
  Fixpt's choice now.

## Questions

- Breadcrumbs in the suffix, or a second field? And is 64 the right
  stride and threshold?
- (a) or (b) for freezing a mutable string?
- Should the UTF-8 string keep kind 1, with the mutable one given a new
  kind, or the other way round?
