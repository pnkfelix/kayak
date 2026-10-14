# Coopt's heap, and porting it to the engine in KFX

Status: research, 2026-10-14. Coopt read at `c366857` (the clone brought
forward on purpose for this, from `8d4fb96`): `crates/coopt-heap`,
`crates/coopt-mem`, `docs/research/heap-base.md` (R1),
`heap-regions.md` (R5), `heap-bounds.md` (R8). The user (2026-10-14):
"take a look at the Coopt code to learn about how that system did memory
management; maybe we can reimplement its memory management in KFX".

## Findings first

1. **Kayak has one collector, the engine's** (C64, `fx26/heap.kfx`): a
   stop-the-world Cheney copy between two flat arrays of `i64`, both
   grown when what survives fills half, in KFX. Compiled KFX programs
   have none: kayak-rt's heap is bump arenas never freed (C43), and only
   `letrena`'s regions are freed, when their bodies end.
2. **Coopt's collector is the regional collector of the user's thesis**
   (Klock 2011), with R8's tighter parameters, built in steps (M11, M12,
   M13): a nursery; an old generation in regions of R words (a power of
   two), filled a to-region at a time, fillers keeping each region
   parseable; a card barrier with a crossing map and a dirty-card list;
   large objects (over θ·R, θ = (√5 − 1)/2) in never-moved spans;
   summaries ("points-into" locations, a sparse matrix by source and
   target region) built by a linear scan through the last snapshot,
   popular regions waved off past S·R entries; one ready region evacuated
   at a time; snapshot-at-the-beginning marking, a few objects a step,
   per-region frontier stacks as roots; pacing by counts (words promoted
   and assigned), never time; a verifier; a stress policy whose fillers
   change size so no object keeps its offset; heap images; freed regions'
   pages given back (§44).
3. **All of it is safe Rust over words, but for the memory under it.**
   `coopt-heap` never dereferences a Value: a Value is an offset, and the
   heap indexes a `&[Cell<u64>]`. The `unsafe` is all in `coopt-mem`:
   `mmap` to reserve the range (at 32 TiB if the system allows, R1),
   `mprotect` to commit a growing prefix, `madvise` to give pages back,
   and the one `from_raw_parts` that makes the slice.
4. **So it is the engine's shape already.** The engine's heap is words in
   flat arrays, pointers as indices, its objects a header and a payload,
   collected only at the top of a machine step where the registers are
   the roots. Coopt's collector ports to KFX almost piece for piece; what
   has no FX-26 counterpart is `coopt-mem`, and the engine can do without
   it (below).
5. **What it would buy the engine** is not mainly throughput: the
   collector is about 5% of a self-compile on the engine (C66). It is
   bounded pauses (today a collection copies everything live, inside one
   machine slice, so a large heap stalls the REPL however short the
   slice), and footprint (a semispace pair is twice the live data before
   growth; C66's self-compile peaked at 783 MB).
6. **A collector for compiled KFX programs is a different project.** Their
   values are Rust references (`&'a T`, `Copy`), which K5 wants readable;
   a moving collector would make them indices into a heap of words and
   every access a lookup, the engine's way. C64 already called that a
   separate question. Not proposed here.

## The port, piece by piece

| Coopt | In the engine, in KFX |
|---|---|
| `coopt-mem`'s reservation, commit, release | A flat array of `i64` for the nursery and one for each region, made when first wanted. Kayak-rt never frees one, so an emptied region's array goes to a pool and is reused, never dropped: the footprint is the peak's, and §44 (pages given back) has no counterpart. |
| A Value's payload as an offset; `rgnof(o)` a shift | The same: a pointer word's index is region × R + offset (the nursery numbered apart), so `rgnof` is a shift and the tags (`fx26/heap.kfx`) do not change. |
| `'gc`-branded Values, `Mutation`, `Root` | Not needed: the engine collects only at a step's top, its registers then the roots (C64), which already keeps any word from being held across a collection. |
| The barrier in `Mutation`'s stores | In `obj-set!`, the one store into an existing object (53 calls, most in `vm.kfx` and `vwords.kfx`): the card, the dirty list, the assignment count, and while marking the old word to the SATB log. Frames are never written once made, so few stores are old to young. |
| Crossing map, fillers, the region's parse | The same, as flat arrays; a filler is an object of kind `ob-raw`. |
| `FixedMap`, deterministic iteration | KFX has no hashed iteration; the summary matrix is flat arrays and lists in insertion order. |
| Pacing by counts, steps as scheduler slices | The machine already runs in slices (`m-slice-ns`); a collector step runs at a step's top when debt is owed, its units charged to the slice. |
| Verifier, stress policy | The same, behind an engine flag; the corpus, Fixpt's programs and the self-compile run under stress. |
| Heap images (M13) | Not wanted yet: the engine loads no images. |

## Steps, in coopt's order

Each step one commit, the check green, the corpus and Fixpt's programs
run under the stress policy, and measured (pauses in words, the
self-compile's time and peak) against the step before:

1. **Generational.** A nursery and an old space; minor collections from
   the roots and the dirty cards; the barrier in `obj-set!`; the
   crossing map; the major collection still a full Cheney copy; the
   verifier (every old word into the nursery on a dirty card).
2. **Regions.** The old space as regions with fillers and a to-region;
   large objects in spans; the major collection a full evacuation into
   fresh regions.
3. **Summaries** by linear scan, a step at a time; wave-off.
4. **One region evacuated** at a time, from its summary.
5. **Snapshot marking**: bitmaps, per-region frontier stacks, the SATB
   log; the basis filtering summarization, so dead cycles across regions
   are freed.
6. **Pacing by counts**, R8's parameters (S = S′ = 3, F1 = F3 = 1,
   L_soft = 1.5, θ golden); a test of the pause bound on an adversary.
