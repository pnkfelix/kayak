# Places as lifetimes

Status: a design note (2026-10-11), before building; `TODO.md` §14 has
the rule, this the cost. The decision it asks for is in §4.

## 1. The rule, and where KFX is

`TODO.md` §14: a region's Rust lifetime is the lifetime of the place its
data lives in; a region variable is a lifetime parameter ranging over
places. Today every place is the heap, never freed, so every reference is
`'static` and every region erased (C10). `letregion`, which makes no
place, is in KFX (its data is the heap's); `letrena`, `letreap` and the
`r`-allocators (`rcons`, `rnew`, ...) are not.

## 2. What a place needs in Rust

A `letrena r` is an arena that lives for its body: `'r` is the borrow of
it, and everything allocated in it is `&'r`. That reaches every type
constructor that can hold such a reference:

- `List<T>` becomes `List<'a, T>` (`Option<&'a Cons<'a, T>>`), since a
  list of `&'r` data cannot be `'static`;
- `Ref<T>`, `FlatArray<T>`: `&'a Cell<T>`, `&'a [Cell<T>]`;
- a datatype holding any of them: `enum T<'a, ...>`, and its references
  `&'a T<'a>`;
- a procedure value: `&'a (dyn Fn(...) -> R + 'a)`, and the `lambdaN`
  helpers generic in `'a`, since a closure capturing `&'r` data is not
  `'static`;
- every generic `fn` that takes any of these: a lifetime parameter.

So the change is not local: every type the emitter writes gains a
lifetime, and rustc's lifetime inference must agree with FX-26's region
checking, which it will where the regions are places, and may not where
a region is only for typing (a `letregion`'s, now erased).

## 3. A cheaper first step

Most of the gain is memory reclaimed by a long computation that builds
and drops large data. One lifetime for "the run" (`'run`), passed as a
parameter, instead of `'static`, changes nothing yet but makes the types
ready; a `letrena` then takes a lifetime of its own only where it is
written. Even this touches every type.

## 4. To decide

1. Whether to do this at all now: the compiler written in KFX runs for
   10 ms and peaks at 30 MB on itself; nothing in kayak today needs
   memory reclaimed. Places matter for a compiler of FX-26 at large on
   large programs, or for long-running KFX programs.
2. If so, `'static` replaced by one lifetime throughout first (§3), or
   `letrena` alone, its regions' types lifetime-generic and the rest
   `'static`.
