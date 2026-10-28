//! The incremental compile checked against a compile from scratch (C54;
//! was `scripts/incremental.py`, C276). A copy of the compiler's sources
//! (kfx/, and lib/ beside it) is edited, an edit at a time, and after each
//! the copy is compiled twice: with the memos kept from the compiles
//! before (`kfx --cache FILE`), and with none. The two must be the same,
//! file for file. Each edit keeps the program one kfx accepts; together
//! they move forms (an insertion before them), change a form's text and its
//! facts, add definitions, a datatype (the context), and take them away
//! again; and across files, a form whose text is the same but whose type is
//! found from a definition that changes.
//!
//! The edits are chains, each a test of its own, each from its own copy
//! and its own memos: a chain's compiles are in order, each with the memos
//! of the one before, and the chains at once. A file's edits, for three
//! files (`KAYAK_CHECK_ALL`: every file); a module's items; and across
//! files. With `KAYAK_CHECK_QUICK`, one file's first edits and the edits
//! across files.

use kayak_checks::{TempDir, Test, copy_tree, quick, release_bin, root};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

type Edit = (&'static str, fn(&str) -> String);

const EDITS: [Edit; 6] = [
    ("a comment first, moving every form after it", |s| {
        format!("\"An edit, first.\"\n{s}")
    }),
    ("a definition last", |s| {
        format!(
            "{s}\n(define incr-probe (subr (maxeff (read @globals) spin) () i64) (lambda () (i64+ 1 2)))\n"
        )
    }),
    ("its body changed, and its facts", |s| {
        s.replace("(i64+ 1 2)", "(i64* 1 (i64+ 2 3))")
    }),
    ("a datatype last, in the context", |s| {
        format!("{s}\n(define-datatype incr-shape (incr-round i64) (incr-square i64 i64))\n")
    }),
    ("the comment first taken away", |s| {
        s.replacen("\"An edit, first.\"\n", "", 1)
    }),
    ("the definitions taken away", |s| {
        format!("{}\n", s.split("\n(define incr-probe").next().unwrap_or(s))
    }),
];

/// An edit of one file of the copy, `kfx/FILE`.
struct Step {
    file: String,
    what: &'static str,
    edit: fn(&str) -> String,
    /// Whether the edit must change the file (a module's edits find text).
    must_change: bool,
}

fn step(file: &str, (what, edit): Edit) -> Step {
    Step {
        file: file.to_string(),
        what,
        edit,
        must_change: false,
    }
}

fn found(file: &str, (what, edit): Edit) -> Step {
    Step {
        file: file.to_string(),
        what,
        edit,
        must_change: true,
    }
}

fn files(d: &Path) -> Result<BTreeMap<String, String>, String> {
    let mut m = BTreeMap::new();
    for e in std::fs::read_dir(d).map_err(|e| format!("{}: {e}", d.display()))? {
        let e = e.map_err(|e| e.to_string())?;
        let text = std::fs::read_to_string(e.path()).map_err(|e| e.to_string())?;
        m.insert(e.file_name().to_string_lossy().into_owned(), text);
    }
    Ok(m)
}

/// A chain: a copy compiled, then each step's edit and the copy compiled
/// again, each time with memos and without, the two the same.
fn chain(label: &str, steps: &[Step]) -> Result<(), String> {
    let tmp = TempDir::new(label);
    let t = &tmp.0;
    copy_tree(&root().join("kfx"), &t.join("kfx")).map_err(|e| e.to_string())?;
    copy_tree(&root().join("lib"), &t.join("lib")).map_err(|e| e.to_string())?;
    let unit = t.join("kfx").join("kfx.unit");
    let cache = t.join("cache");
    let kfx = release_bin("kfx");
    let compare = |what: &str| -> Result<(), String> {
        let (warm, cold) = (t.join("warm"), t.join("cold"));
        for d in [&warm, &cold] {
            let _ = std::fs::remove_dir_all(d);
            std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
        }
        // The two at once: neither reads what the other writes.
        let spawn = |args: Vec<&std::ffi::OsStr>| {
            Command::new(&kfx)
                .args(args)
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
        };
        let w = spawn(vec![
            "--cache".as_ref(),
            cache.as_os_str(),
            unit.as_os_str(),
            warm.as_os_str(),
        ])
        .map_err(|e| e.to_string())?;
        let c = spawn(vec![unit.as_os_str(), cold.as_os_str()]).map_err(|e| e.to_string())?;
        for p in [w, c] {
            let out = p.wait_with_output().map_err(|e| e.to_string())?;
            if !out.status.success() {
                return Err(format!(
                    "after {what}, kfx fails:\n{}",
                    String::from_utf8_lossy(&out.stderr)
                ));
            }
        }
        let (wf, cf) = (files(&warm)?, files(&cold)?);
        if wf != cf {
            let differ: Vec<&String> = cf.keys().filter(|f| wf.get(*f) != cf.get(*f)).collect();
            return Err(format!(
                "after {what}, the compile with memos differs in {differ:?}"
            ));
        }
        Ok(())
    };
    compare("nothing")?;
    for s in steps {
        let path: PathBuf = t.join("kfx").join(&s.file);
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let changed = (s.edit)(&text);
        if s.must_change && changed == text {
            return Err(format!("an edit found nothing to change: {}", s.what));
        }
        std::fs::write(&path, changed).map_err(|e| e.to_string())?;
        compare(&format!("{}: {}", s.file, s.what))?;
    }
    Ok(())
}

fn unit_names() -> Vec<String> {
    std::fs::read_to_string(root().join("kfx").join("kfx.unit"))
        .expect("kfx/kfx.unit")
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with(';') && !l.starts_with("--"))
        .map(String::from)
        .collect()
}

fn main() {
    let names = unit_names();
    let every = std::env::var_os("KAYAK_CHECK_ALL").is_some();
    let chosen: Vec<String> = if every {
        names.clone()
    } else if quick() {
        vec![names[names.len() / 2].clone()]
    } else {
        vec![
            names[0].clone(),
            names[names.len() / 2].clone(),
            names[names.len() - 1].clone(),
        ]
    };
    let mut tests = Vec::new();
    for name in chosen {
        let edits: Vec<Edit> = if quick() {
            EDITS[..3].iter().chain(&EDITS[5..]).copied().collect()
        } else {
            EDITS.to_vec()
        };
        let label = format!("edits-of-{}", name.trim_end_matches(".kfx"));
        let file = name.clone();
        tests.push(Test::new(label.clone(), move || {
            chain(
                &label,
                &edits.iter().map(|e| step(&file, *e)).collect::<Vec<_>>(),
            )
        }));
    }
    if !quick() {
        // Inside a module (C91): an item's body, an item added, and back; a
        // string among its items.
        tests.push(Test::new("module-items", || {
            let t = "terminate.kfx";
            chain(
                "module-items",
                &[
                    found(
                        t,
                        ("a module item's body changed", |s| {
                            s.replace(
                                "(define most-graphs i64 500)",
                                "(define most-graphs i64 501)",
                            )
                        }),
                    ),
                    found(
                        t,
                        ("an item added in a module", |s| {
                            s.replacen(
                                "    (hide\n",
                                "    (hide\n      (define incr-inner i64 3)\n",
                                1,
                            )
                        }),
                    ),
                    found(
                        t,
                        ("the module's item taken away", |s| {
                            s.replacen("      (define incr-inner i64 3)\n", "", 1)
                        }),
                    ),
                    found(
                        t,
                        ("the module item's body back", |s| {
                            s.replace(
                                "(define most-graphs i64 501)",
                                "(define most-graphs i64 500)",
                            )
                        }),
                    ),
                    found(
                        t,
                        ("a string among a module's items", |s| {
                            s.replacen(
                                "    (hide\n",
                                "    (hide\n      \"An edit, a comment.\"\n",
                                1,
                            )
                        }),
                    ),
                    found(
                        t,
                        ("the string taken away", |s| {
                            s.replacen("      \"An edit, a comment.\"\n", "", 1)
                        }),
                    ),
                ],
            )
        }));
        // A procedure the order checks in one `define-rec` with others, not
        // first among them: its text, after its first, moved.
        tests.push(Test::new("define-rec-item", || {
            let x = "check-exprs.kfx";
            chain(
                "define-rec-item",
                &[
                    found(
                        x,
                        ("a later item of a define-rec group changed", |s| {
                            s.replacen(
                                "\"a blobref made where none is expected\"",
                                "\"a blobref made where no blobref is expected\"",
                                1,
                            )
                        }),
                    ),
                    found(
                        x,
                        ("that item back", |s| {
                            s.replacen(
                                "\"a blobref made where no blobref is expected\"",
                                "\"a blobref made where none is expected\"",
                                1,
                            )
                        }),
                    ),
                ],
            )
        }));
    }
    // Across files: a later form's text the same, what it reads changed.
    let (first, last) = (names[0].clone(), names[names.len() - 1].clone());
    tests.push(Test::new("across-files", move || {
        chain(
            "across-files",
            &[
                step(&first, ("a definition, read later", |s| {
                    format!("{s}\n(define incr-given (subr (maxeff (read @globals) spin) () i64) (lambda () 7))\n")
                })),
                step(&last, ("a definition whose type is found from it", |s| {
                    format!("{s}\n(define incr-use (lambda () (incr-given)))\n")
                })),
                step(&first, ("the first's type changed", |s| {
                    s.replace("() i64) (lambda () 7))", "() i32) (lambda () 7))")
                })),
                step(&first, ("its body changed, its type the same", |s| s.replace("(lambda () 7))", "(lambda () 8))"))),
            ],
        )
    }));
    kayak_checks::run(tests)
}
