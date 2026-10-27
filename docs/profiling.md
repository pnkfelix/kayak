# Profiling FX-26 programs on kayak

What kayak gives a profiler of an FX-26 program, by tier (C239, C240,
C245-C247). Each listing below is `kayak-fx26 --cells prog.fx >
prog.cells`; its line tables name `prog.fx` as given, so run from the
directory that path is relative to.

## Tier 1: `--profile`

    kayak-cells prog.cells --profile

Counts every cell tier 1 runs, and writes to the standard error the
source lines, the expressions (LINE:COLUMN and text) and the words that
ran the most. Exact counts, no sampling; tier 1 only.

## Tiers 2-6: `perf`, names only

    KAYAK_PERF_MAP=1 perf record kayak-cells prog.cells --compiled
    perf report

`/tmp/perf-PID.map` names each compiled word's code (`fx26 NAME`, `fx26
NAME (tier 6)`) and the machine's (`kayak tier 2: the machine`).

## Tiers 2-6: `perf annotate`, with source lines (tiers 4-6)

    KAYAK_JITDUMP=1 perf record -k 1 kayak-cells prog.cells --compiled
    perf inject --jit -i perf.data -o perf.jit.data
    perf annotate -i perf.jit.data -l -s "fx26 NAME"
    perf report -i perf.jit.data --sort srcline

The jitdump gives perf each word's code, and for words with a line table
each instruction's source line: tier 4's from its cells' table, tiers 5
and 6's from their register code's (`; regs at`, C249), from kayak's
listings. A register word's entry (its frame made, its parameters
stored, its guards) is the lambda's own line: the call protocol's cost,
in `expr` mode the lambda's text.

## Per expression: `KAYAK_JITDUMP=expr`

    KAYAK_JITDUMP=expr perf record -k 1 kayak-cells prog.cells --compiled
    perf inject --jit -i perf.data -o perf.jit.data
    python3 -I scripts/perf-expr.py perf.jit.data            # by expression
    python3 -I scripts/perf-expr.py perf.jit.data --lines    # summed by line
    perf annotate -i perf.jit.data -l -s "fx26 NAME"         # expressions shown inline

perf keeps lines, not columns, so this mode gives each expression a line
of its own, in a side file `/tmp/jit-PID.NAME.expr`: its first N lines
are the source's, then a `;;; kayak expr` line saying so, then line
N + 2 + P for character position P, `LINE:COLUMN  TEXT`. Any line past
N + 1 is an expression; `perf annotate` shows its text over its code
unaided, and `scripts/perf-expr.py` reads perf's lines back.

## Not yet

- Fixpt's listings have no line tables: names only there.
- A word from a module file of its own is given the program's file.
