# The engine's heap and its collector

Status: design, 2026-10-12. `docs/research/engine.md`, step 5; C63 (the
compiler runs on the engine, but the self-compile runs out of memory: the
engine allocates about 4 GB a second in kayak-rt's heap, which is never
freed, C43). The user (2026-10-12): the collector first.

## Findings first

1. **A collected heap of its own is not enough by itself.** Anything the
   engine allocates as KFX data (a `val`, a frame, a `ctl`, a string made
   by `string-append`) is kayak-rt's, and never freed. So the engine's
   loop must allocate no KFX data at all, in its steady state: its
   values, environments, continuation frames and strings must all be
   words in its own heap. KFX data is only what is made once: the lowered
   program (the IR, a table), the interned names (symbols, tags, labels),
   and the heap's two spaces.
2. **So the engine becomes a VM written in KFX over flat arrays of `i64`**
   (`make-flatarray-i64`, `C12`): Fixpt's and Coopt's arrangement, values
   as words, pointers as indices, which needs no `unsafe` (Q7) and is K7
   at its fullest: the collector is KFX.
3. **When to collect**: only at the top of the machine's loop, where its
   registers are all the roots. A step allocates a bounded amount (a frame,
   an environment, a pair), which a reserve covers; a primitive that
   allocates in proportion to its input (`string-append`, `string->list`,
   `make-array`, `read-file`) first says how much it needs, and the loop
   collects (and grows the heap) before it runs if need be. No pointer is
   ever held across a collection outside the roots.

## Decisions

- **Words** (`i64`), tagged in their low two bits:
  - `00` a fixnum, its value the word shifted right by 2 (62 bits); an
    `int` past it is a boxed bignum;
  - `01` a pointer: an object at the index the word shifted right by 2;
  - `10` an immediate of a kind in bits 2 to 4: a character, a boolean,
    unit, `nil`, a symbol (an interned name's number), a primitive, a
    prompt tag, the unbound marker;
  - `11` an IR node, by its number in the IR's table.
- **Objects**: a header word (kind in the low 8 bits, payload size above)
  then the payload: a pair (car, cdr); a reference; a sum (tag, value); a
  product (labels' number, fields); a datatype's value (tag, fields); an
  array; a flat array; a bytevector (bytes, packed); a string (its
  characters, a word each); an `f64` (its bits); a bignum (sign, 32-bit
  limbs); a procedure (code, environment); a continuation (frames, used,
  once); an environment frame (parent, slots); a continuation frame (its
  kind, its fields).
- **The heap**: two spaces, fixed, swapped at each collection (none
  allocated anew, which would leak the old one); grown, both, when what
  survives is more than half: the old pair leaked once, a geometric cost.
- **The collector**: Cheney's, copying, breadth-first; a forwarding
  pointer in the header of an object moved.
- **Roots**: the machine's registers (the control, the environment, the
  continuation, the value, the arguments being gathered), the globals (a
  flat array of words of their own), and a short stack the driver uses.
- **The IR as a table**: `lower.kfx` is unchanged; a pass numbers its
  nodes into an array of `ir`s whose children are numbers, so a frame
  holds a node as a word.
- **Interned**: symbols, sums' and datatypes' tags, products' label lists:
  each a number in a KFX table, made once per name, never freed (bounded
  by the program's text and the symbols it makes).
- **Strings** in the heap, a character a word; one becomes a KFX string
  only for I/O, in proportion to what is read and written.
- **Bignums**: computed with KFX's `int` (which allocates for those alone)
  and stored back: rare enough to leak in.

## Steps

1. `fx26/heap.kfx`: words, objects, allocation, the collector, interning.
2. `fx26/table.kfx`: the IR as a table.
3. `fx26/vm.kfx`: the machine over words; the primitives over words; the
   printer. The old machine kept until the new one passes all it did.
4. The self-compile on the engine, its memory measured.

## Can it be KFX alone? (the user, 2026-10-12)

Yes, on one condition, which is checked, not trusted. KFX has what a
heap and a Cheney collector need: flat arrays of `i64`, written in place
(the two spaces), `i64` arithmetic and shifts (tags, headers), a
character's and an `f64`'s bits. The condition: the engine's loop
allocates nothing of kayak-rt's in its steady state, since that heap is
never freed. Reading a datatype and calling a procedure allocate nothing;
making a datatype's value, a list, a closure (a `lambda`: so no `map` in
the loop) or a string do. The check: kayak-rt's own allocation counts
(`heap_stats_report`) for runs of different lengths, which must not grow
with the run. What does grow, boundedly: the old spaces left when the
heap grows (geometric, about twice the final heap in all), and I/O's
strings (as much as is read and written).

## Rust or KFX for the collector (the user, 2026-10-12)

The user: Coopt's collector is Rust (`coopt-heap`, some 3,500 lines, no
`unsafe`, indices into words `coopt-mem` reserves, values branded so that
an async task cannot hold one across `.await`); it may be better in pure
async Rust. What either way: the engine's values, environments, frames and
strings are words in a heap of its own, the machine and the primitives
over them KFX (Q8). What differs is behind a small interface (allocate,
read and write a word, collect given the roots), the collector's 150 lines
or so: in Rust, faster (no `Cell` or `ref` between the loop and the words),
and Coopt's incremental collection, a task on the scheduler, within reach;
in KFX, K7's, and the engine runs on Fixpt unchanged (a Rust heap's
primitives would need an FX-26 meaning in the prelude, which is the KFX
collector again). Proposed: the interface fixed; the KFX collector first,
as the reference; its share of the self-compile measured; a Rust one
behind the same interface, checked against it, if it matters or when
incremental collection does. The user's call.
