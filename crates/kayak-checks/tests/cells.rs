//! Tier 1 (C122): kayak-cells, Fixpt's cellular machine in KFX, run on the
//! words Fixpt's own compiler makes, its answers compared with Fixpt's (was
//! `scripts/cells-programs.py`, C276). A test for each of Fixpt's programs
//! kept (tests/fx26/programs/) and kayak's own (tests/fx26-kayak/), twice:
//! `fixpt/NAME`, on Fixpt's words (`fixpt … compile`), and `ours/NAME`, on
//! the words kayak's own code generator makes (`kayak-fx26 --cells`, C133).
//! And `generated`: the Rust kfx makes of cells/cells.unit is the Rust
//! kept in crates/kayak-cells/src/cells (`scripts/regen.sh --all` makes it).
//!
//! Each program agrees (the same value, or the same failure), differs (the
//! test fails), or is not run: its listing is not exact, it needs what tier
//! 1 has not yet, it leaves no value, Fixpt refuses it, or (ours) kayak's
//! generator cannot yet. One kept as agreeing (tests/cells-passing,
//! tests/cellgen-passing) that no longer does fails too. Each test writes
//! what it found to target/kayak-checks/cells/, which `checks-summary
//! cells` counts, says, and blesses the passing lists from.
//!
//! Each that agrees runs again: by each tier where native code runs
//! (`--hand`, `--compiled`, `--direct`, `--fixpt-calls`), with register
//! code (`--regs`), and, but for Fixpt's benchmarks, with a collection at
//! every safepoint (`--gc-stress`, C142), and, for a program that runs
//! tasks, preempted at nearly every poll (`--slice`, C125); every way at
//! once, each giving the same answer.

use kayak_checks::fixpt::{self, Ran};
use kayak_checks::{TempDir, Test, release_bin, root};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const MACHINE: [&str; 2] = ["--fx26-machine", "fx-cells-rsinterp"];
const LONG: Duration = Duration::from_secs(600);

fn hand() -> bool {
    matches!(
        (std::env::consts::ARCH, std::env::consts::OS),
        ("aarch64", "macos") | ("x86_64", "linux")
    )
}

fn fixpt_calls() -> bool {
    (std::env::consts::ARCH, std::env::consts::OS) == ("x86_64", "linux")
}

/// What a failed run says, past Fixpt's and kayak's framing.
fn failure(text: &str) -> Option<String> {
    text.lines().find_map(|l| {
        let m = l.split_once("evaluation failed: ")?.1.trim();
        Some(m.strip_prefix("cellular word: ").unwrap_or(m).to_string())
    })
}

const DEFINING: [&str; 10] = [
    "(define",
    "(define*",
    "(define-rec",
    "(define-type",
    "(define-datatype",
    "(define-effect",
    "(define-generative",
    "(define-convention",
    "(import",
    "(module-parameters",
];

/// Whether a program's last top-level form defines (it leaves no value).
fn last_defines(path: &Path) -> bool {
    let text = std::fs::read_to_string(path).unwrap();
    let t: Vec<char> = text.chars().collect();
    let (mut depth, mut start, mut last, mut i) = (0i64, 0usize, String::new(), 0usize);
    while i < t.len() {
        let c = t[i];
        if c == ';' {
            while i < t.len() && t[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c == '"' {
            i += 1;
            while i < t.len() && t[i] != '"' {
                i += if t[i] == '\\' { 2 } else { 1 };
            }
            if depth == 0 {
                last = "\"".into();
            }
        } else if c == '#' && t.get(i + 1) == Some(&'\\') {
            i += 3;
            continue;
        } else if c == '(' {
            if depth == 0 {
                start = i;
            }
            depth += 1;
        } else if c == ')' {
            depth -= 1;
            if depth == 0 {
                last = t[start..=i].iter().collect();
            }
        } else if depth == 0 && !c.is_whitespace() {
            last = c.to_string();
        }
        i += 1;
    }
    DEFINING.iter().any(|d| last.starts_with(d))
}

type Words = std::collections::BTreeMap<String, Vec<Vec<String>>>;

/// Register words compared: the same, those that differ, Fixpt's only and
/// kayak's only, and Fixpt's only named.
type Regs = (usize, Vec<String>, usize, usize, Vec<String>);

fn word_name(l: &str) -> String {
    l.split(" (").next().unwrap()[5..].to_string()
}

/// A listing's words, by name: each its stack code's lines, its register
/// code and comments left out; the words of one name sorted.
fn stack_words(listing: &str) -> Words {
    let mut words = Words::new();
    let (mut name, mut lines, mut skipping): (Option<String>, Vec<String>, bool) =
        (None, vec![], false);
    for l in listing.lines() {
        if l.starts_with("word ") {
            if let Some(n) = name.take() {
                words.entry(n).or_default().push(std::mem::take(&mut lines));
            }
            name = Some(word_name(l));
            lines = vec![l.split(", ").next().unwrap().to_string()];
            skipping = false;
        } else if l.starts_with("  its register code") {
            skipping = true;
        } else if l.starts_with(';') || l.trim().is_empty() {
            continue;
        } else if !skipping {
            lines.push(l.to_string());
        }
    }
    if let Some(n) = name {
        words.entry(n).or_default().push(lines);
    }
    for ws in words.values_mut() {
        ws.sort();
    }
    words
}

/// A listing's words' register code, by name; a word with none left out.
fn reg_words(listing: &str) -> Words {
    let mut words = Words::new();
    let (mut name, mut lines, mut inside): (Option<String>, Option<Vec<String>>, bool) =
        (None, None, false);
    for l in listing.lines() {
        if l.starts_with("word ") {
            if let (Some(n), Some(ls)) = (name.take(), lines.take()) {
                words.entry(n).or_default().push(ls);
            }
            name = Some(word_name(l));
            lines = None;
            inside = false;
        } else if l.starts_with("  its register code") {
            lines = Some(vec![l.split(", ").next().unwrap().to_string()]);
            inside = true;
        } else if l.starts_with(';') || l.trim().is_empty() || l.starts_with("  its ") {
            if l.starts_with("  its ") {
                inside = false;
            }
        } else if inside && let Some(ls) = lines.as_mut() {
            ls.push(l.to_string());
        }
    }
    if let (Some(n), Some(ls)) = (name, lines) {
        words.entry(n).or_default().push(ls);
    }
    for ws in words.values_mut() {
        ws.sort();
    }
    words
}

/// Kayak's register words against Fixpt's: how many the same, those that
/// differ, how many Fixpt's only and kayak's only, and Fixpt's only named.
fn compare_regs(ours: &str, theirs: &str) -> Regs {
    let (a, b) = (reg_words(ours), reg_words(theirs));
    let same = a.iter().filter(|(n, w)| b.get(*n) == Some(w)).count();
    let differ = a
        .iter()
        .filter(|(n, w)| b.get(*n).is_some_and(|x| x != *w))
        .map(|(n, _)| n.clone())
        .collect();
    let only_b: Vec<String> = b.keys().filter(|n| !a.contains_key(*n)).cloned().collect();
    (
        same,
        differ,
        only_b.len(),
        a.keys().filter(|n| !b.contains_key(*n)).count(),
        only_b,
    )
}

fn tasks(path: &Path) -> bool {
    let text = std::fs::read_to_string(path).unwrap();
    [
        "spawn",
        "spawn-monitored",
        "par",
        "race",
        "pselect",
        "send",
        "receive",
        "icell-get",
        "icell-put!",
    ]
    .iter()
    .any(|w| {
        let pat = format!("({w}");
        text.match_indices(&pat).any(|(i, _)| {
            text[i + pat.len()..]
                .chars()
                .next()
                .is_none_or(|c| !(c.is_alphanumeric() || c == '_'))
        })
    })
}

/// What a test found: its kind, why, and, ours, how its words compare.
struct Found {
    kind: &'static str,
    why: String,
    same_words: Option<bool>,
    regs: Option<Regs>,
}

fn found(kind: &'static str, why: impl Into<String>) -> Found {
    Found {
        kind,
        why: why.into(),
        same_words: None,
        regs: None,
    }
}

fn strs(xs: &[&str]) -> Vec<String> {
    xs.iter().map(|s| s.to_string()).collect()
}

/// A program, its words found and run, then its ways.
fn one(name: &str, path: &Path, ours: bool) -> Found {
    let binary = fixpt::binary();
    let unlimited = name.starts_with("bench/");
    let limit: Vec<String> = if unlimited {
        strs(&["--step-limit", "none"])
    } else {
        vec![]
    };
    let p = path.to_string_lossy().into_owned();
    let files = [path.to_path_buf()];
    let ask = |args: Vec<String>| fixpt::ask(binary.as_deref(), &args, &files, None);
    let mut args = strs(&MACHINE);
    args.extend(limit.clone());
    args.extend(["eval".to_string(), p.clone()]);
    let ev = match ask(args) {
        Ok(r) => r,
        Err(e) => return found("fixpt-refuses", e),
    };
    let want_fail = failure(&format!("{}\n{}", ev.stdout, ev.stderr));
    if !ev.ok() && want_fail.is_none() {
        return found("fixpt-refuses", "");
    }
    let lines: Vec<&str> = ev
        .stdout
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with(';'))
        .collect();
    if want_fail.is_none() && (lines.is_empty() || last_defines(path)) {
        return found("no-value", "");
    }
    let want = want_fail.is_none().then(|| {
        let l = lines[lines.len() - 1];
        let l = l.rsplit_once(" ! ").map_or(l, |x| x.0);
        l.rsplit_once(" : ").map_or(l, |x| x.0).to_string()
    });
    let mut args = strs(&MACHINE);
    args.extend(strs(&["compile", "--full-literals"]));
    args.push(p.clone());
    let Ok(co) = ask(args) else {
        return found("fixpt-refuses", "");
    };
    let mut text = co.stdout.clone();
    if !co.ok() && co.stdout.contains("; the compilers disagree") {
        // Fixpt 1925bca's `--full-literals` compares its two compilers'
        // listings as text, and they differ only so when the plain listing
        // says both made the same: then the FX-26 compiler's (C123).
        let mut args = strs(&MACHINE);
        args.extend(["compile".to_string(), p.clone()]);
        match ask(args) {
            Ok(plain) if plain.ok() && plain.stdout.contains("; both compilers made this") => {
                text = co
                    .stdout
                    .split("; the Rust compiler:")
                    .next()
                    .unwrap()
                    .to_string();
            }
            _ => return found("fixpt-refuses", ""),
        }
    } else if !co.ok() {
        return found("fixpt-refuses", "");
    }
    let (mut same_words, mut regs) = (None, None);
    if ours {
        let theirs = text;
        let made = fixpt::run(
            Command::new(release_bin("kayak-fx26"))
                .arg("--cells")
                .arg(path),
            Some(LONG),
        );
        if !made.ok() {
            let said = made
                .stderr
                .lines()
                .find(|l| l.contains("kayak-fx26") || l.contains("kfx:"))
                .or(made.stderr.trim().lines().last())
                .unwrap_or("?");
            return found("gen-refuses", said);
        }
        text = made.stdout;
        same_words = Some(stack_words(&text) == stack_words(&theirs));
        regs = Some(compare_regs(&text, &theirs));
    }
    let tmp = TempDir::new("cells");
    let listing = tmp.0.join("words.cells");
    std::fs::write(&listing, &text).unwrap();
    let cells = release_bin("kayak-cells");
    let got = fixpt::run(Command::new(&cells).arg(&listing).args(&limit), Some(LONG));
    let mut f = (|| {
        if let Some((_, rest)) = got.stderr.split_once("the listing: ") {
            return found("refused", rest.trim().lines().next().unwrap_or(""));
        }
        let got_fail = failure(&got.stderr);
        if got_fail.as_deref().is_some_and(|g| g.contains("(C122)")) {
            return found("not-yet", got_fail.unwrap());
        }
        if got_fail.as_deref() == Some("the word left nothing") {
            return found("no-value", "");
        }
        if !got.ok() && got_fail.is_none() {
            let e = got.stderr.trim();
            return found(
                "differ",
                format!("kayak-cells failed: {}", &e[e.len().saturating_sub(200)..]),
            );
        }
        if want_fail.is_some() || got_fail.is_some() {
            if want_fail != got_fail {
                return found(
                    "differ",
                    format!("fixpt: {want_fail:?}, kayak: {got_fail:?}"),
                );
            }
            return ways(
                &cells,
                name,
                path,
                &listing,
                unlimited,
                "",
                got_fail.as_deref(),
            );
        }
        let have = got.stdout.trim();
        if Some(have) != want.as_deref() {
            return found("differ", format!("fixpt: {want:?}, kayak: {have:?}"));
        }
        ways(&cells, name, path, &listing, unlimited, have, None)
    })();
    f.same_words = same_words;
    f.regs = regs;
    f
}

/// The ways a program's answer must be the same, all run at once: agree,
/// or differ at the first that does not.
fn ways(
    cells: &Path,
    name: &str,
    path: &Path,
    listing: &Path,
    unlimited: bool,
    have: &str,
    fail: Option<&str>,
) -> Found {
    let lim = |mut w: Vec<String>| {
        if unlimited {
            w.extend(strs(&["--step-limit", "none"]));
        }
        w
    };
    let (hand, compiled, direct, fcalls) = (hand(), hand(), hand(), fixpt_calls());
    let mut ways: Vec<Vec<String>> = Vec::new();
    if hand {
        ways.push(lim(strs(&["--hand"])));
        if compiled {
            ways.push(lim(strs(&["--compiled"])));
        }
    }
    ways.push(lim(strs(&["--regs"])));
    if compiled {
        ways.push(lim(strs(&["--regs", "--compiled"])));
    }
    if direct {
        ways.push(lim(strs(&["--regs", "--direct"])));
    }
    if fcalls {
        ways.push(lim(strs(&["--regs", "--compiled", "--fixpt-calls"])));
        ways.push(lim(strs(&["--regs", "--direct", "--fixpt-calls"])));
    }
    if !name.starts_with("bench/") {
        ways.push(strs(&["--gc-stress"]));
        ways.push(strs(&["--regs", "--gc-stress"]));
        if compiled {
            ways.push(strs(&["--regs", "--compiled", "--gc-stress"]));
        }
        if direct {
            ways.push(strs(&["--regs", "--direct", "--gc-stress"]));
        }
        if fcalls {
            ways.push(strs(&[
                "--regs",
                "--compiled",
                "--fixpt-calls",
                "--gc-stress",
            ]));
        }
        if hand {
            ways.push(strs(&["--hand", "--gc-stress"]));
        }
        if compiled {
            ways.push(strs(&["--compiled", "--gc-stress"]));
        }
        if tasks(path) {
            ways.push(strs(&["--slice", "2", "--step-limit", "none"]));
            // Register code at a slice of two never runs on: the
            // trampoline's way back spends it (C129). Three.
            ways.push(strs(&["--regs", "--slice", "3", "--step-limit", "none"]));
            if compiled {
                ways.push(strs(&[
                    "--regs",
                    "--compiled",
                    "--slice",
                    "3",
                    "--step-limit",
                    "none",
                ]));
            }
            if direct {
                ways.push(strs(&[
                    "--regs",
                    "--direct",
                    "--slice",
                    "3",
                    "--step-limit",
                    "none",
                ]));
            }
            if fcalls {
                ways.push(strs(&[
                    "--regs",
                    "--direct",
                    "--fixpt-calls",
                    "--slice",
                    "3",
                    "--step-limit",
                    "none",
                ]));
            }
            if compiled {
                ways.push(strs(&[
                    "--compiled",
                    "--slice",
                    "2",
                    "--step-limit",
                    "none",
                ]));
            }
        }
    }
    let results: Vec<Option<String>> = std::thread::scope(|s| {
        let hs: Vec<_> = ways
            .iter()
            .map(|w| {
                s.spawn(move || {
                    let again: Ran =
                        fixpt::run(Command::new(cells).arg(listing).args(w), Some(LONG));
                    let f = failure(&again.stderr);
                    if f.as_deref() != fail || (fail.is_none() && again.stdout.trim() != have) {
                        Some(format!(
                            "under {}: {:?} {:?}",
                            w.join(" "),
                            again.stdout.trim(),
                            f
                        ))
                    } else {
                        None
                    }
                })
            })
            .collect();
        hs.into_iter().map(|h| h.join().unwrap()).collect()
    });
    match results.into_iter().flatten().next() {
        Some(why) => found("differ", why),
        None => found("agree", ""),
    }
}

fn results_dir(suite: &str) -> PathBuf {
    root()
        .join("target")
        .join("kayak-checks")
        .join("cells")
        .join(suite)
}

fn passing(suite: &str) -> Vec<String> {
    let f = if suite == "ours" {
        "cellgen-passing"
    } else {
        "cells-passing"
    };
    std::fs::read_to_string(root().join("tests").join(f))
        .map(|t| t.split_whitespace().map(String::from).collect())
        .unwrap_or_default()
}

/// A test's finding, kept for `checks-summary cells`: lines of `key value`.
fn keep(suite: &str, name: &str, f: &Found) {
    let dir = results_dir(suite);
    std::fs::create_dir_all(&dir).unwrap();
    let mut text = format!("kind {}\nwhy {}\n", f.kind, f.why.replace('\n', " "));
    if let Some(s) = f.same_words {
        text += &format!("same {s}\n");
    }
    if let Some((same, differ, theirs, mine, declined)) = &f.regs {
        text += &format!(
            "regs {same} {theirs} {mine}\nregs-differ {}\nregs-declined {}\n",
            differ.join(" "),
            declined.join(" ")
        );
    }
    std::fs::write(dir.join(name.replace('/', "%")), text).unwrap();
}

fn generated() -> Result<(), String> {
    let tmp = TempDir::new("cells-rust");
    kayak_checks::ran(
        Command::new(release_bin("kfx"))
            .arg(root().join("cells").join("cells.unit"))
            .arg(&tmp.0),
    )?;
    let kept = root()
        .join("crates")
        .join("kayak-cells")
        .join("src")
        .join("cells");
    let names = |d: &Path| -> Vec<String> {
        let mut v: Vec<String> = std::fs::read_dir(d)
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        v.sort();
        v
    };
    if names(&tmp.0) != names(&kept) {
        return Err("kfx makes other files of cells/ than crates/kayak-cells/src/cells (scripts/regen.sh --all)".into());
    }
    for n in names(&tmp.0) {
        if std::fs::read(tmp.0.join(&n)).ok() != std::fs::read(kept.join(&n)).ok() {
            return Err(format!(
                "kfx makes other Rust of cells/ in {n} (scripts/regen.sh --all, if it should)"
            ));
        }
    }
    Ok(())
}

fn main() {
    let mut programs = fixpt::programs();
    programs.extend(fixpt::kayak_programs());
    let mut tests = vec![Test::new("generated", generated)];
    for suite in ["fixpt", "ours"] {
        let kept = passing(suite);
        for (name, path) in &programs {
            let (name, path) = (name.clone(), path.clone());
            let was = kept.contains(&name);
            tests.push(Test::new(format!("{suite}/{name}"), move || {
                let f = one(&name, &path, suite == "ours");
                keep(suite, &name, &f);
                match f.kind {
                    "differ" => Err(format!("differs: {}", f.why)),
                    "agree" => Ok(()),
                    _ if was => Err(format!("no longer agrees: {} {}", f.kind, f.why)),
                    _ => Ok(()),
                }
            }));
        }
    }
    kayak_checks::run(tests)
}
