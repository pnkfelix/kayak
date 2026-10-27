"""Tier 1 (C122): kayak-cells, Fixpt's cellular machine in KFX, run on the
words Fixpt's own compiler makes, its answers compared with Fixpt's.

usage: python3 -I scripts/cells-programs.py [--bless | --bless-drop] [--why]
       [--ours [--list] [--regs-differ] [--regs-declined]]

--why: the reasons programs are not run, counted. With --ours,
--regs-differ names the register words that differ from Fixpt's, and
--regs-declined those Fixpt has and kayak declined (C259).

kfx compiles cells/cells.unit; its Rust is kept in
crates/kayak-cells/src/cells (--bless rewrites it; else it must be what
kfx makes). Then, for each of Fixpt's FX-26 test programs, as kept in
tests/fx26/programs/, and Fixpt's answers, as kept in tests/fixpt-answers/
(C132; asked of the pinned Fixpt when there is a clone and they are another
pin's): `fixpt --fx26-machine fx-cells-rsinterp compile` lists its words,
kayak-cells runs them, and `fixpt ... eval` says what the answer is. Each
program agrees (the same value, or the same failure), differs (the check
fails), or is not run: its listing is not exact (a literal cut short,
...), it needs what tier 1 has not yet (C122), it leaves no value (a
program of definitions), or Fixpt refuses it. The programs that agree are
kept in tests/cells-passing (--bless rewrites it); one that agreed and
no longer does fails the check. Each but Fixpt's benchmarks is run again
with a collection at every safepoint, or, once much is live, at about
one in every so many, jittered (`--gc-stress`, C142), to the same end;
each by tier 2's machine (`--hand`, C126), with its words compiled
(`--compiled`, C128), these on arm64 macOS and x86_64 Linux (C136,
C137), with register code as native procedures too (`--direct`, tier 6,
C138, C210; Fixpt's calls for tier 5, `--fixpt-calls`, C140, on x86_64
Linux alone), and with register code
(`--regs`, C129), its answer
the same; and each that runs tasks (processes, `par`, `race`, `pselect`), again
with a time slice of two polls (`--slice 2`, as Fixpt's `FIXPT_SLICE`,
C125), preempted at nearly every poll; with no step limit, as Fixpt's
limit is each form's and tier 1 runs the program as one word. (A slice of
one poll leaves a task none of its own after its grace: Fixpt's run of
`monitors/one-message.fx` so does not end.)
"""

import os
import re
import shutil
import subprocess
import sys
import tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import fixptcache  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
GENERATED = os.path.join(ROOT, "crates", "kayak-cells", "src", "cells")
PASSING = os.path.join(ROOT, "tests", "cells-passing")
# With --ours (C133): the words kayak's own code generator makes of each
# program (`kayak-fx26 --cells`), run the same ways; those that agree kept
# here, and those whose words are Fixpt's, word for word, counted.
OURS = "--ours" in sys.argv[1:]
OURS_PASSING = os.path.join(ROOT, "tests", "cellgen-passing")
ENGINE = os.path.join(ROOT, "target", "release", "kayak-fx26")
MACHINE = ["--fx26-machine", "fx-cells-rsinterp"]
# Fixpt's benchmarks run past FX-26's default step limit: run without one.
UNLIMITED = ("bench/",)


def run(args, **kw):
    # The standard input empty, as Fixpt's runs are given it (fixptcache):
    # a program that reads it gets the end, whatever the check was started
    # with (C233).
    return subprocess.run(args, capture_output=True, text=True, stdin=subprocess.DEVNULL, **kw)


def generated():
    """kfx's Rust of cells/cells.unit: the files' texts, by name."""
    with tempfile.TemporaryDirectory() as out:
        r = run([os.path.join(ROOT, "target", "release", "kfx"), os.path.join(ROOT, "cells", "cells.unit"), out])
        if r.returncode != 0:
            sys.exit("cells-programs: kfx refuses cells/: " + r.stderr.strip()[-400:])
        return {f: open(os.path.join(out, f), encoding="utf-8").read() for f in sorted(os.listdir(out))}


def failure(text):
    """What a failed run says, past Fixpt's and kayak's framing."""
    for l in text.splitlines():
        if "evaluation failed: " in l:
            m = l.split("evaluation failed: ", 1)[1].strip()
            return m[len("cellular word: "):] if m.startswith("cellular word: ") else m
    return None


DEFINING = ("(define", "(define*", "(define-rec", "(define-type", "(define-datatype", "(define-effect",
            "(define-generative", "(define-convention", "(import", "(module-parameters")


def last_defines(path):
    """Whether a program's last top-level form defines (it leaves no value)."""
    text = open(path, encoding="utf-8").read()
    depth, start, last, i = 0, None, "", 0
    while i < len(text):
        c = text[i]
        if c == ";":
            while i < len(text) and text[i] != "\n":
                i += 1
            continue
        if c == '"':
            i += 1
            while i < len(text) and text[i] != '"':
                i += 2 if text[i] == "\\" else 1
            if depth == 0:
                last = '"'
        elif text.startswith("#\\", i):
            i += 3
            continue
        elif c == "(":
            if depth == 0:
                start = i
            depth += 1
        elif c == ")":
            depth -= 1
            if depth == 0:
                last = text[start:i + 1]
        elif depth == 0 and not c.isspace():
            last = c
        i += 1
    return last.startswith(DEFINING)


# Whether each program's words, with --ours, are Fixpt's.
SAME = {}


def stack_words(listing):
    """A listing's words, by name: each its stack code's lines, its register
    code (which kayak's generator does not make yet) and comments left out;
    the words of one name (a global defined again) sorted."""
    words, name, lines, skipping = {}, None, [], False
    for l in listing.splitlines():
        if l.startswith("word "):
            if name is not None:
                words.setdefault(name, []).append(lines)
            name, lines, skipping = l.split(" (", 1)[0][5:], [l.split(", ", 1)[0]], False
        elif l.startswith("  its register code"):
            skipping = True
        elif l.startswith(";") or not l.strip():
            continue
        elif not skipping:
            lines.append(l)
    if name is not None:
        words.setdefault(name, []).append(lines)
    return {n: sorted(ws) for n, ws in words.items()}


# Of each program's register words, with --ours, how many are Fixpt's
# exactly, differ, are Fixpt's only (kayak's compiler declined) or kayak's
# only (C248); and the differing words' names.
REGS = {}


def reg_words(listing):
    """A listing's words' register code, by name: each its lines, the words
    of one name sorted; a word with none left out."""
    words, name, lines, inside = {}, None, None, False
    for l in listing.splitlines():
        if l.startswith("word "):
            if name is not None and lines is not None:
                words.setdefault(name, []).append(lines)
            name, lines, inside = l.split(" (", 1)[0][5:], None, False
        elif l.startswith("  its register code"):
            lines, inside = [l.split(", ", 1)[0]], True
        elif l.startswith(";") or not l.strip() or l.startswith("  its "):
            inside = False if l.startswith("  its ") else inside
        elif inside:
            lines.append(l)
    if name is not None and lines is not None:
        words.setdefault(name, []).append(lines)
    return {n: sorted(ws) for n, ws in words.items()}


def compare_regs(ours, theirs):
    a, b = reg_words(ours), reg_words(theirs)
    same = [n for n in a if n in b and a[n] == b[n]]
    differ = [n for n in a if n in b and a[n] != b[n]]
    return (len(same), differ, len([n for n in b if n not in a]), len([n for n in a if n not in b]),
            [n for n in b if n not in a])


def one(binary, cells, name, path):
    limit = ["--step-limit", "none"] if name.startswith(UNLIMITED) else []
    ev = fixptcache.run(binary, MACHINE + limit + ["eval", path], [path])
    want_fail = failure(ev.stdout + "\n" + ev.stderr)
    if ev.returncode != 0 and want_fail is None:
        return "fixpt-refuses", ""
    lines = [l for l in ev.stdout.splitlines() if l.strip() and not l.startswith(";")]
    if want_fail is None and (not lines or last_defines(path)):
        return "no-value", ""
    want = None if want_fail is not None else lines[-1].rsplit(" ! ", 1)[0].rsplit(" : ", 1)[0]
    co = fixptcache.run(binary, MACHINE + ["compile", "--full-literals", path], [path])
    text = co.stdout
    if co.returncode != 0 and "; the compilers disagree" in co.stdout:
        # Fixpt 1925bca's `--full-literals` compares its two compilers'
        # listings as text, and one labels literals the other makes twice
        # (flonums): they disagree only so, when the plain listing says both
        # made the same. Then the first, the FX-26 compiler's (C123).
        plain = fixptcache.run(binary, MACHINE + ["compile", path], [path])
        if plain.returncode == 0 and "; both compilers made this" in plain.stdout:
            text = co.stdout.split("; the Rust compiler:", 1)[0]
        else:
            return "fixpt-refuses", ""
    elif co.returncode != 0:
        return "fixpt-refuses", ""
    if OURS:
        theirs = text
        made = run([ENGINE, "--cells", path], timeout=600)
        if made.returncode != 0:
            said = [l for l in made.stderr.splitlines() if "kayak-fx26" in l or "kfx:" in l]
            return "gen-refuses", (said or made.stderr.strip().splitlines()[-1:] or ["?"])[0]
        text = made.stdout
        SAME[name] = stack_words(text) == stack_words(theirs)
        REGS[name] = compare_regs(text, theirs)
    with tempfile.NamedTemporaryFile("w", suffix=".cells", delete=False) as f:
        f.write(text)
        listing = f.name
    try:
        got = run([cells, listing] + (["--step-limit", "none"] if limit else []), timeout=600)
    finally:
        os.unlink(listing)
    if "the listing: " in got.stderr:
        return "refused", got.stderr.split("the listing: ", 1)[1].strip().splitlines()[0]
    got_fail = failure(got.stderr)
    if got_fail is not None and "(C122)" in got_fail:
        return "not-yet", got_fail
    if got_fail == "the word left nothing":
        return "no-value", ""
    if got.returncode != 0 and got_fail is None:
        return "differ", "kayak-cells failed: " + got.stderr.strip()[-200:]
    if want_fail is not None or got_fail is not None:
        if want_fail != got_fail:
            return "differ", "fixpt: %r, kayak: %r" % (want_fail, got_fail)
        return stressed(cells, name, path, text, limit, "", got_fail)
    have = got.stdout.strip()
    if have != want:
        return "differ", "fixpt: %r, kayak: %r" % (want, have)
    return stressed(cells, name, path, text, limit, have, None)


# Tiers 2, 4 and 5 run where native code does (kayak-jit's): arm64 macOS,
# and x86_64 Linux (C136, C137).
HAND = (sys.platform == "darwin" and os.uname().machine == "arm64") or (sys.platform == "linux" and os.uname().machine == "x86_64")
COMPILED = HAND
# Tier 6, register code as native procedures (C138, C210), likewise.
DIRECT = HAND
# Fixpt's calling convention for tier 5 (C140): x86_64 Linux alone.
FIXPT_CALLS = sys.platform == "linux" and os.uname().machine == "x86_64"
TASKS = re.compile(r"\((spawn|spawn-monitored|par|race|pselect|send|receive|icell-get|icell-put!)\b")


def stressed(cells, name, path, text, limit, have, fail):
    """The ways a program's answer must be the same: by each tier, with a
    collection at every safepoint (C122, Coopt's stress policy; spaced out
    as more lives, C142), and, for a
    program that runs tasks, preempted at every poll (C125); Fixpt's
    benchmarks, long, are not run so. Run by `ways_run`, all at once."""
    # Tier 2, the hand machine (C126), each program, where native code runs.
    # Tier 4, the words compiled (C128), likewise.
    ways = [[t] + (["--step-limit", "none"] if limit else []) for t in ("--hand", "--compiled") if t == "--hand" or COMPILED] if HAND else []
    # Register code (C129), interpreted by tier 1; compiled (tier 5, C130).
    ways.append(["--regs"] + (["--step-limit", "none"] if limit else []))
    if COMPILED:
        ways.append(["--regs", "--compiled"] + (["--step-limit", "none"] if limit else []))
    if DIRECT:
        ways.append(["--regs", "--direct"] + (["--step-limit", "none"] if limit else []))
    if FIXPT_CALLS:
        # Fixpt's calling convention for tier 5 (C140), with tier 6 too.
        ways.append(["--regs", "--compiled", "--fixpt-calls"] + (["--step-limit", "none"] if limit else []))
        ways.append(["--regs", "--direct", "--fixpt-calls"] + (["--step-limit", "none"] if limit else []))
    if not name.startswith(UNLIMITED):
        ways.append(["--gc-stress"])
        ways.append(["--regs", "--gc-stress"])
        if COMPILED:
            ways.append(["--regs", "--compiled", "--gc-stress"])
        if DIRECT:
            ways.append(["--regs", "--direct", "--gc-stress"])
        if FIXPT_CALLS:
            ways.append(["--regs", "--compiled", "--fixpt-calls", "--gc-stress"])
        if HAND:
            ways.append(["--hand", "--gc-stress"])
        if COMPILED:
            ways.append(["--compiled", "--gc-stress"])
        if TASKS.search(open(path, encoding="utf-8").read()):
            ways.append(["--slice", "2", "--step-limit", "none"])
            # Register code at a slice of two never runs on: the
            # trampoline's way back spends it (C129). Three.
            ways.append(["--regs", "--slice", "3", "--step-limit", "none"])
            if COMPILED:
                ways.append(["--regs", "--compiled", "--slice", "3", "--step-limit", "none"])
            if DIRECT:
                ways.append(["--regs", "--direct", "--slice", "3", "--step-limit", "none"])
            if FIXPT_CALLS:
                ways.append(["--regs", "--direct", "--fixpt-calls", "--slice", "3", "--step-limit", "none"])
            if COMPILED:
                ways.append(["--compiled", "--slice", "2", "--step-limit", "none"])
    return "ways", (ways, text, have, fail)


def ways_run(cells, ps, ran):
    """Each program's ways (`stressed`'s), every program's and way's run a
    job of its own, the benchmarks' first, being the longest: so as many
    run at once as there are processors to the last, where by program a few
    benchmarks' ways ran one after another while the rest stood idle. A
    program agrees if every way gives its answer, else differs at the first
    that does not."""
    listings, jobs = {}, []
    for i, ((name, _), (kind, why)) in enumerate(zip(ps, ran)):
        if kind != "ways":
            continue
        ways, text, have, fail = why
        with tempfile.NamedTemporaryFile("w", suffix=".cells", delete=False) as f:
            f.write(text)
            listings[i] = f.name
        jobs += [(not name.startswith(UNLIMITED), i, j, way, have, fail) for j, way in enumerate(ways)]
    jobs.sort(key=lambda job: job[0])

    def way_one(job):
        _, i, _, way, have, fail = job
        again = run([cells, listings[i]] + way, timeout=600)
        if failure(again.stderr) != fail or (fail is None and again.stdout.strip() != have):
            return "under %s: %r %r" % (" ".join(way), again.stdout.strip(), failure(again.stderr))
        return None

    try:
        said = fixptcache.parallel(way_one, jobs)
    finally:
        for listing in listings.values():
            os.unlink(listing)
    first = {}
    for (_, i, j, _, _, _), why in zip(jobs, said):
        if why is not None and (i not in first or j < first[i][0]):
            first[i] = (j, why)
    return [(("differ", first[i][1]) if i in first else ("agree", "")) if kind == "ways" else (kind, why) for i, (kind, why) in enumerate(ran)]


def main():
    bless = fixptcache.blessing()
    made = generated()
    kept = {f: open(os.path.join(GENERATED, f), encoding="utf-8").read() for f in os.listdir(GENERATED)} if os.path.isdir(GENERATED) else {}
    if made != kept:
        if not bless:
            sys.exit("cells-programs: kfx makes other Rust of cells/ (--bless, if it should)")
        shutil.rmtree(GENERATED, ignore_errors=True)
        os.makedirs(GENERATED)
        for f, text in made.items():
            with open(os.path.join(GENERATED, f), "w", encoding="utf-8") as g:
                g.write(text)
    r = run(["cargo", "build", "--release", "--quiet", "-p", "kayak-cells"], cwd=ROOT)
    if r.returncode != 0:
        sys.exit("cells-programs: kayak-cells does not build:\n" + r.stderr[-2000:])
    cells = os.path.join(ROOT, "target", "release", "kayak-cells")
    if OURS and run(["cargo", "build", "--release", "--quiet", "-p", "kayak-fx26"], cwd=ROOT).returncode != 0:
        sys.exit("cells-programs: kayak-fx26 does not build")
    binary = fixptcache.binary()
    ps = fixptcache.programs() + fixptcache.kayak_programs()
    ran = ways_run(cells, ps, fixptcache.parallel(lambda p: one(binary, cells, p[0], p[1]), ps))
    by = {}
    wrong = []
    for (name, _), (kind, why) in zip(ps, ran):
        by.setdefault(kind, []).append(name)
        if kind == "differ":
            wrong.append("%s: %s" % (name, why))
    agree = sorted(by.get("agree", []))
    passing = OURS_PASSING if OURS else PASSING
    before = open(passing).read().split() if os.path.exists(passing) else []
    lost = [n for n in before if n not in agree]
    if bless and fixptcache.drop_refused(lost, "cells-programs"):
        sys.exit(1)
    if bless:
        with open(passing, "w") as f:
            f.write("\n".join(agree) + "\n")
    if "--why" in sys.argv[1:]:
        reasons = {}
        for (name, _), (kind, why) in zip(ps, ran):
            if kind in ("refused", "not-yet", "gen-refuses"):
                reasons[(kind, why)] = reasons.get((kind, why), 0) + 1
        for (kind, why), n in sorted(reasons.items(), key=lambda x: -x[1]):
            print("%5d  %s: %s" % (n, kind, why[:150]))
    for w in wrong:
        print("DIFFER  " + w)
    if lost and not bless:
        for n in lost:
            print("NO LONGER AGREES  " + n)
    if OURS:
        same = sorted(n for n in agree if SAME.get(n))
        if "--list" in sys.argv[1:]:
            for n in agree:
                print("%s %s" % ("same " if SAME.get(n) else "other", n))
        print("cells-programs: kayak's own words: %d programs agree, %d differ, %d of them Fixpt's words exactly; "
              "%d the generator cannot yet" % (len(agree), len(wrong), len(same), len(by.get("gen-refuses", []))))
        # Where the stack words are Fixpt's exactly, the register words are
        # named alike: there, each is compared.
        rs = [r for n, r in REGS.items() if SAME.get(n)]
        print("cells-programs: kayak's register words, where its stack words are Fixpt's: %d Fixpt's exactly, %d differ, "
              "%d Fixpt's only (declined), %d kayak's only"
              % (sum(r[0] for r in rs), sum(len(r[1]) for r in rs), sum(r[2] for r in rs), sum(r[3] for r in rs)))
        if "--regs-differ" in sys.argv[1:]:
            for n, r in sorted(REGS.items()):
                if r[1] and SAME.get(n):
                    print("REGS DIFFER  %s: %s" % (n, " ".join(r[1])))
        if "--regs-declined" in sys.argv[1:]:
            for n, r in sorted(REGS.items()):
                if r[4] and SAME.get(n):
                    print("REGS DECLINED  %s: %s" % (n, " ".join(r[4])))
    print("cells-programs: %d of Fixpt's programs agree on tier 1, %d differ; not run: %d listings not exact, %d need what tier 1 has not yet, %d leave no value, %d Fixpt refuses"
          % (len(agree), len(wrong), len(by.get("refused", [])), len(by.get("not-yet", [])), len(by.get("no-value", [])), len(by.get("fixpt-refuses", []))))
    if wrong or (lost and not bless):
        sys.exit(1)


if __name__ == "__main__":
    main()
