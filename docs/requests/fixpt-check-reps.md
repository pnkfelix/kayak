# A request to the Fixpt team: `fixpt check --reps`, each definition's rep

Status: proposed 2026-11-05 by kayak (C346-C349), on Fixpt 8616f2fe
(kayak's pin). Not sent yet.

## What we ask

A flag to `fixpt check` that prints, for each top-level definition of the
input, the rep of its type as Fixpt's checker exports it
(`docs/research/heap-types.md`), one line each:

```
NAME HEX
NAME declined
```

`HEX` is the rep's binary form (`fixpt_reps::binary::encode`) in
lowercase hexadecimal; `declined` where the export takes no rep of it
(`Checker::export_ty` fails: an unresolved `select`, say). A `define-rec`
gives a line for each of its bindings, in order. A form that is no
definition gives none; a form refused ends the output there, as `fixpt
check` stops.

Either checker's rep would do, as both make one rep of a type (your
heap-types step 2 checks so in one process, `tests/reps.rs`). If the two
differ for a definition, a second line saying so (`NAME differs HEX2`,
the FX-26 checker's) would tell us which checker we match, and would
tell you of a disagreement.

A readable form as well (`--reps=text`, `fixpt_reps::text::write`) would
help people reading a difference; the hexadecimal one alone is enough for
a test.

## Why

kayak's checker exports types as reps by Fixpt's table of heads and
Fixpt's `fixpt-reps` crate, copied whole, so that a rep kayak makes and one
Fixpt makes are the same bytes and can be compared, as stages and evidence
assume. Until now nothing compared them: each side's reps were checked
only against themselves. When we did compare (2026-11-05), only 800 of the
1280 definitions of your `tests/programs` had Fixpt's rep. Most of the
differences were kayak's export (a type variable not a `var` node, binder
names, generative types without their `gendef`, datatypes as constants
rather than the sums they are). Some were kayak's checker, which agreed
with Fixpt's printing but not its types: for example, kayak read a written
`heap` as `@heap`, and so took a program your checker refuses. Now 1262
match. kayak's own `kayak-fx26 --reps FILE` prints the same two fields,
then whether its rep comes back as itself.

What we compared with was a scratch program linked against your
`fixpt-fx26` crate (below). We would rather not keep a check that links
your crates: kayak's checks talk to Fixpt through the `fixpt` binary
alone, built from a pinned clone. With `fixpt check --reps`, kayak's
agreement test could compare reps rather than printed lines, which tells
a type that differs from a type printed differently (88 programs differed
in printing alone when we last counted, C299).

## The scratch program, for reference

What kayak used, run on each file. Its `main` is the whole of the
proposal, as far as the Rust checker goes:

```rust
use fixpt_fx26::{Checker, Top};
use fixpt_read::FileId;

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let program = std::fs::read_to_string(&path).unwrap();
    let mut c = Checker::new();
    c.base_dir = std::path::Path::new(&path).parent().map(|d| d.to_path_buf());
    let Ok(forms) = c.read_in(FileId(0), &program) else { return };
    for f in &forms {
        let tys = match c.top(f) {
            Ok(Top::Define { name: n, ty, .. }) => vec![(n, ty)],
            Ok(Top::DefineRec { bindings, .. }) => bindings.iter().map(|(n, t, _)| (*n, *t)).collect(),
            Ok(_) => continue,
            Err(_) => break,
        };
        for (n, t) in tys {
            let name = c.interner.name(n).to_string();
            match c.export_ty(t) {
                Ok(ex) => {
                    let b = fixpt_reps::binary::encode(&ex.rep);
                    println!("{name} {}", b.iter().map(|x| format!("{x:02x}")).collect::<String>());
                }
                Err(_) => println!("{name} declined"),
            }
        }
    }
}
```

## Where kayak and Fixpt still differ, at the pin

Here so you can see what the flag would show us, not as requests. Each is
kayak's to settle (C347, C349):
- nested `poly`s, which kayak's checker merges into one
  (`bidirectional/twice.fx`, `run/conditions.fx`);
- a module's abstract type seen outside it, which Fixpt writes as a hole
  (`?0:counter..t`) and kayak as a constant; in a functor, `param-sel`;
  an abstract type of arrow kind;
- the order of a module's descriptions, a loaded module's global's
  name, a region bound within a place, one inferred effect;
- three kayak declines (`convof`, an effect family applied, `~!`).
