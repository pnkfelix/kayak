"""kc's assembler against the GNU assembler (C450): each assembly file kc's
native back end made (the objects `kc build` assembles, under
target/kc-steps/, made by scripts/kc-steps.py first), made an object by kc
(`kc --object FILE OUT`, kc/asm.kfx) and by `as`, the two compared: each
section's bytes (`.text`, `.data`, `.rodata`), `.bss`'s size, each
section's relocations (offset, type, symbol or section, addend), and the
symbols (name, binding, type, section, value, size), but the sections'
own. The first difference of each that differs is said.

usage: python3 -I scripts/kc-objects.py [-v] [FILE.s ...]
"""

import concurrent.futures
import glob
import os
import struct
import subprocess
import sys
import tempfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
KC = os.path.join(ROOT, "target", "release", "kc")
WORK = os.path.join(ROOT, "target", "kc-steps")


def elf(path):
    """An ELF64 relocatable's sections, relocations and symbols."""
    b = open(path, "rb").read()
    shoff, = struct.unpack_from("<Q", b, 0x28)
    shentsize, shnum, shstrndx = struct.unpack_from("<HHH", b, 0x3A)
    secs = []
    for i in range(shnum):
        name, typ, flags, addr, off, size, link, info, align, entsize = struct.unpack_from("<IIQQQQIIQQ", b, shoff + i * shentsize)
        secs.append(dict(name=name, type=typ, off=off, size=size, link=link, info=info))
    shstr = secs[shstrndx]

    def cstr(base, k):
        e = b.index(b"\0", base + k)
        return b[base + k:e].decode()

    for s in secs:
        s["name"] = cstr(shstr["off"], s["name"])
    symtab = next(s for s in secs if s["type"] == 2)
    strtab = secs[symtab["link"]]
    syms = []
    for k in range(symtab["size"] // 24):
        name, info, other, shndx, value, size = struct.unpack_from("<IBBHQQ", b, symtab["off"] + 24 * k)
        syms.append(dict(name=cstr(strtab["off"], name), bind=info >> 4, type=info & 15, shndx=shndx, value=value, size=size))

    def sec_name(n):
        return {0: "UND", 0xFFF1: "ABS", 0xFFF2: "COM"}.get(n, secs[n]["name"] if n < len(secs) else "?")

    def sym_name(k):
        s = syms[k]
        return sec_name(s["shndx"]) if s["type"] == 3 else s["name"]

    out = {"sections": {}, "relocs": {}, "symbols": set()}
    for s in secs:
        if s["name"] in (".text", ".data", ".rodata"):
            out["sections"][s["name"]] = b[s["off"]:s["off"] + s["size"]]
        elif s["name"] == ".bss":
            out["sections"][".bss"] = s["size"]
        elif s["type"] == 4:
            target = secs[s["info"]]["name"]
            rs = []
            for k in range(s["size"] // 24):
                off, info, addend = struct.unpack_from("<QQq", b, s["off"] + 24 * k)
                rs.append((off, info & 0xFFFFFFFF, sym_name(info >> 32), addend))
            out["relocs"][target] = sorted(rs)
    for s in syms:
        if s["name"] and s["type"] not in (3, 4):
            out["symbols"].add((s["name"], s["bind"], s["type"], sec_name(s["shndx"]), s["value"], s["size"]))
    for name in (".text", ".data", ".rodata"):
        out["sections"].setdefault(name, b"")
    out["sections"].setdefault(".bss", 0)
    return out


def first_difference(mine, theirs):
    for name in (".text", ".data", ".rodata"):
        a, t = mine["sections"][name], theirs["sections"][name]
        if a != t:
            k = next((i for i in range(min(len(a), len(t))) if a[i] != t[i]), min(len(a), len(t)))
            return "%s differs at byte %d (kc %d bytes, as %d): kc %s, as %s" % (
                name, k, len(a), len(t), a[max(0, k - 8):k + 8].hex(), t[max(0, k - 8):k + 8].hex())
    if mine["sections"][".bss"] != theirs["sections"][".bss"]:
        return ".bss: kc %d bytes, as %d" % (mine["sections"][".bss"], theirs["sections"][".bss"])
    for target in sorted(set(mine["relocs"]) | set(theirs["relocs"])):
        a, t = mine["relocs"].get(target, []), theirs["relocs"].get(target, [])
        if a != t:
            only_a = [r for r in a if r not in t][:3]
            only_t = [r for r in t if r not in a][:3]
            return "relocations of %s: kc's alone %s; as's alone %s" % (target, only_a, only_t)
    if mine["symbols"] != theirs["symbols"]:
        return "symbols: kc's alone %s; as's alone %s" % (
            sorted(mine["symbols"] - theirs["symbols"])[:3], sorted(theirs["symbols"] - mine["symbols"])[:3])
    return None


def compare(path):
    with tempfile.TemporaryDirectory() as d:
        mine, theirs = os.path.join(d, "kc.o"), os.path.join(d, "as.o")
        r = subprocess.run([KC, "--object", path, mine], capture_output=True, text=True)
        if r.returncode != 0:
            return path, "kc --object failed: " + (r.stderr.strip().splitlines() or [""])[-1]
        r = subprocess.run(["as", "-o", theirs, path], capture_output=True, text=True)
        if r.returncode != 0:
            return path, "as failed: " + r.stderr.strip()[-300:]
        return path, first_difference(elf(mine), elf(theirs))


def main():
    verbose = "-v" in sys.argv[1:]
    given = [a for a in sys.argv[1:] if a != "-v"]
    files = given or sorted(glob.glob(os.path.join(WORK, "*.o.d", "*.s")) + glob.glob(os.path.join(WORK, "*.phase1", "*.o.d", "*.s"))
                            + glob.glob(os.path.join(ROOT, "target", "kc-steps.db.phase1", "*.o.d", "*.s")))
    if not files:
        sys.exit("kc-objects: no assembly under %s: run scripts/kc-steps.py first" % WORK)
    with concurrent.futures.ThreadPoolExecutor(max_workers=os.cpu_count() or 4) as ex:
        results = list(ex.map(compare, files))
    bad = [(p, why) for p, why in results if why]
    for p, why in bad[: (len(bad) if verbose else 20)]:
        print("DIFFERS: %s: %s" % (os.path.relpath(p, ROOT), why))
    print("kc-objects: %d objects, %d as `as` makes them, %d differ" % (len(results), len(results) - len(bad), len(bad)))
    sys.exit(1 if bad else 0)


main()
