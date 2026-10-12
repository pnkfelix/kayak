"""usage: python3 -I scripts/bench.py [--runs N] [--compiler-runs N]

Times Fixpt's benchmarks (its crates/fixpt-fx26/tests/programs/bench) and
KFX's ports of them (bench/ports, tailored to KFX's paths: i64 for int,
an arena for a round's list), each two ways: as kfx's Rust, built
--release, and on Fixpt's machines (`fixpt bench`). A benchmark kfx
refuses is a row of dashes. Times are the run alone, best of N, in ms;
an answer that differs from Fixpt's is marked `✗`.

Then the largest program there is: the compiler compiling itself, kfx/
on kfx/kfx.unit, as kfx's Rust (target/release/kfx, the whole process)
and on Fixpt's machines (the prelude and kfx/ as FX-26, stage 0's way,
the run alone). Fixpt takes minutes a round, so its best is of
--compiler-runs (default 1); 0 skips the row.
"""

import os
import re
import shutil
import subprocess
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PORTS = os.path.join(ROOT, "bench", "ports")
WORK = os.path.join(ROOT, "target", "kfx-bench")
KFX = os.path.join(ROOT, "target", "release", "kfx")


def run(args, **kw):
    return subprocess.run(args, capture_output=True, text=True, **kw)


def fixpt_binary():
    r = run([os.path.join(ROOT, "scripts", "fixpt.sh")])
    if r.returncode != 0:
        sys.exit("bench: " + r.stderr.strip())
    return r.stdout.strip()


def fixpt_times(fixpt, files, runs):
    """Each file's answer and its best time on each of Fixpt's machines."""
    r = run([fixpt, "bench", "--runs", str(runs), "--tables", "run"] + files, timeout=3600)
    rows = {}
    head = None
    for line in r.stdout.splitlines():
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if cells and cells[0] == "program":
            head = cells
        elif head and len(cells) == len(head) and not cells[0].startswith("-"):
            rows[cells[0]] = dict(zip(head, cells))
    return head, rows


def rust_name(name):
    return "p_" + re.sub(r"[^a-z0-9]", "_", name)


def build_rust(programs, runs):
    """Each program's answer and best time as kfx's Rust, or None if kfx
    refuses it."""
    src = os.path.join(WORK, "runner", "src")
    shutil.rmtree(os.path.join(WORK, "runner"), ignore_errors=True)
    os.makedirs(src)
    accepted = []
    for name, path in programs:
        r = run([KFX, "--toplevel", path])
        if r.returncode != 0:
            continue
        with open(os.path.join(src, rust_name(name) + ".rs"), "w") as g:
            g.write(r.stdout)
        accepted.append(name)
    with open(os.path.join(WORK, "runner", "Cargo.toml"), "w") as g:
        g.write('[package]\nname = "kfx-bench"\nversion = "0.1.0"\nedition = "2024"\n\n'
                '[dependencies]\nkayak-rt = { path = "%s" }\n\n[workspace]\n'
                % os.path.join(ROOT, "crates", "kayak-rt"))
    with open(os.path.join(src, "main.rs"), "w") as g:
        for name in accepted:
            g.write("#[allow(dead_code, unused, non_snake_case, non_camel_case_types, clippy::all)]\nmod %s;\n"
                    % rust_name(name))
        g.write("""
fn time(name: &str, f: fn() -> kayak_rt::Str) {
    let mut best = f64::MAX;
    let mut answer = String::new();
    for _ in 0..%d {
        let t = std::time::Instant::now();
        let s = kayak_rt::with_big_stack(f);
        best = best.min(t.elapsed().as_secs_f64() * 1000.0);
        answer = s.as_str().trim().lines().last().unwrap_or("").to_string();
    }
    println!("{name}\\t{answer}\\t{best:.1}");
}

fn main() {
""" % runs)
        for name in accepted:
            g.write('    time("%s", %s::kfx_toplevel);\n' % (name, rust_name(name)))
        g.write("}\n")
    built = run(["cargo", "build", "--release", "--quiet"], cwd=os.path.join(WORK, "runner"))
    if built.returncode != 0:
        sys.stderr.write(built.stderr[-4000:])
        sys.exit("bench: the benchmarks' Rust does not build")
    out = run([os.path.join(WORK, "runner", "target", "release", "kfx-bench")], timeout=3600)
    times = {}
    for line in out.stdout.splitlines():
        name, answer, ms = line.split("\t")
        times[name] = (answer, ms)
    return times


def fx_string(text):
    return '"' + text.replace("\\", "\\\\").replace('"', '\\"') + '"'


def compiler_program():
    """The compiler compiling itself, as one FX-26 program (scripts/stage0.py)."""
    kfx = os.path.join(ROOT, "kfx")
    with open(os.path.join(kfx, "kfx.unit")) as f:
        files = [line.strip() for line in f if line.strip() and not line.strip().startswith(";")]
    texts = []
    for name in files:
        with open(os.path.join(kfx, name)) as f:
            texts.append((os.path.splitext(os.path.basename(name))[0], f.read()))
    with open(os.path.join(kfx, "prelude.fx")) as f:
        text = f.read() + "\n"
    text += "".join(t + "\n" for _, t in texts)
    sources = " ".join("(source %s %s)" % (fx_string(n), fx_string(t)) for n, t in texts)
    path = os.path.join(WORK, "compiler.fx")
    with open(path, "w") as f:
        f.write(text + "(bundle (compile-units (list " + sources + ")))\n")
    return path


def bench_compiler(fixpt, runs, fixpt_runs):
    out = os.path.join(WORK, "compiler-out")
    best = float("inf")
    for _ in range(runs):
        shutil.rmtree(out, ignore_errors=True)
        os.makedirs(out)  # kfx makes no directories (C50)
        t = time.perf_counter()
        r = run([KFX, os.path.join(ROOT, "kfx", "kfx.unit"), out])
        best = min(best, (time.perf_counter() - t) * 1000.0)
        if r.returncode != 0:
            sys.exit("bench: kfx does not compile itself: " + r.stderr.strip())
    head, rows = fixpt_times(fixpt, [compiler_program()], fixpt_runs)
    machines = [m for m in head[2:] if m not in ("M words", "GCs")]
    row = rows.get("compiler", {})
    print("\n| the compiler on itself | kfx Rust | " + " | ".join(machines) + " |")
    print("|---|" + "---:|" * (1 + len(machines)))
    print("| kfx/kfx.unit | %.1f | %s |" % (best, " | ".join(row.get(m, "—") for m in machines)))
    print("\nkfx Rust: target/release/kfx, the whole process, best of %d; Fixpt: the run alone, best of %d;"
          " in ms. That the two make the same Rust is scripts/fixpoint.sh's check." % (runs, fixpt_runs))


def main():
    runs = 3
    compiler_runs = 1
    args = sys.argv[1:]
    while args:
        if len(args) >= 2 and args[0] == "--runs":
            runs = int(args[1])
        elif len(args) >= 2 and args[0] == "--compiler-runs":
            compiler_runs = int(args[1])
        else:
            sys.exit(__doc__)
        args = args[2:]
    fixpt = fixpt_binary()
    clone = os.path.dirname(os.path.dirname(os.path.dirname(fixpt)))
    originals = os.path.join(clone, "crates", "fixpt-fx26", "tests", "programs", "bench")
    names = sorted(f[:-3] for f in os.listdir(originals) if f.endswith(".fx"))
    os.makedirs(WORK, exist_ok=True)
    # Fixpt reads `.fx`: the ports copied so, apart from the originals.
    ported = os.path.join(WORK, "ports")
    shutil.rmtree(ported, ignore_errors=True)
    os.makedirs(ported)
    ports = sorted(f[:-4] for f in os.listdir(PORTS) if f.endswith(".kfx"))
    for p in ports:
        shutil.copy(os.path.join(PORTS, p + ".kfx"), os.path.join(ported, p + ".fx"))

    programs = [(n, os.path.join(originals, n + ".fx")) for n in names]
    programs += [("port-" + p, os.path.join(ported, p + ".fx")) for p in ports]
    rust = build_rust(programs, runs)
    head, fx_orig = fixpt_times(fixpt, [path for n, path in programs if not n.startswith("port-")], runs)
    _, fx_port = fixpt_times(fixpt, [path for n, path in programs if n.startswith("port-")], runs)
    machines = [m for m in head[2:] if m not in ("M words", "GCs")]

    print("| program | kfx Rust | kfx Rust, port | " + " | ".join(machines) + " | " +
          " | ".join(m + ", port" for m in machines) + " |")
    print("|---|" + "---:|" * (2 + 2 * len(machines)))
    for n in names:
        fo = fx_orig.get(n, {})
        fp = fx_port.get(n, {})
        answer = fo.get("answer", "")

        def mark(cell, got):
            return cell if got is None or got == answer else cell + " ✗"
        orig = rust.get(n)
        port = rust.get("port-" + n)
        cells = [mark(orig[1], orig[0]) if orig else "—",
                 mark(port[1], port[0]) if port else "—"]
        cells += [fo.get(m, "—") for m in machines]
        cells += [mark(fp.get(m, "—"), fp.get("answer")) if fp else "—" for m in machines]
        print("| %s | %s |" % (n, " | ".join(cells)))
    print("\nkfx Rust: built --release, the run alone, best of %d, in ms; — where kfx refuses a program"
          " or there is no port. Fixpt's machines as `fixpt bench` names them." % runs)
    if compiler_runs > 0:
        bench_compiler(fixpt, runs, compiler_runs)


if __name__ == "__main__":
    main()
