"""usage: python3 -I scripts/bench.py [--runs N]

Times Fixpt's benchmarks (its crates/fixpt-fx26/tests/programs/bench) and
KFX's ports of them (bench/ports, tailored to KFX's paths: i64 for int,
an arena for a round's list), each two ways: as kfx's Rust, built
--release, and on Fixpt's machines (`fixpt bench`). A benchmark kfx
refuses is a row of dashes. Times are the run alone, best of N, in ms;
an answer that differs from Fixpt's is marked `✗`.
"""

import os
import re
import shutil
import subprocess
import sys

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


def main():
    runs = 3
    if len(sys.argv) == 3 and sys.argv[1] == "--runs":
        runs = int(sys.argv[2])
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


if __name__ == "__main__":
    main()
