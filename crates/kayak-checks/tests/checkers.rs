//! kayak's checker against Fixpt's, program by program (TODO §30, PLAN §9;
//! was `scripts/compare-checkers.py`, C276): each of Fixpt's FX-26
//! programs, its ports of the R7RS benchmarks (`scheme-bench/`, from the
//! clone, when there is one, C256), and kayak's own (C257), given to
//! `fixpt check` and to `kayak-fx26 --check`, which writes the same lines
//! (`define NAME : TYPE ! EFFECT`, an expression's `TYPE ! EFFECT`). A
//! test each, `checkers/NAME`; it fails only if kayak accepts a program
//! Fixpt refuses (K1). What each found is kept for `checks-summary
//! checkers`, which counts it:
//!   same       both accept, every line the same;
//!   alike      both accept, the lines the same once each effect's atoms and
//!              each `(globals ...)`'s names are sorted (Fixpt sorts them);
//!   differ     both accept, a line differs even so;
//!   refused    Fixpt accepts, kayak refuses: what kayak's checker lacks;
//!   ACCEPTED   Fixpt refuses, kayak accepts: a K1 breach;
//!   both       both refuse (their messages not compared yet).

use kayak_checks::fixpt;
use kayak_checks::{Test, release_bin, root};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

const LIMIT: Duration = Duration::from_secs(60);

/// Fixpt's lines, or none if it refuses (or fails, or takes too long).
fn fixpt_says(path: &Path) -> Option<Vec<String>> {
    let args = vec![
        "--dialect".into(),
        "fx26".into(),
        "check".into(),
        path.to_string_lossy().into_owned(),
    ];
    let r = fixpt::ask(
        fixpt::binary().as_deref(),
        &args,
        &[path.to_path_buf()],
        Some(LIMIT),
    )
    .ok()?;
    let lines: Vec<&str> = r.stdout.lines().collect();
    if !r.ok() || lines.iter().any(|l| l.starts_with("! ")) {
        return None;
    }
    Some(
        lines
            .into_iter()
            .filter(|l| !l.is_empty() && !l.starts_with(';'))
            .map(String::from)
            .collect(),
    )
}

/// kayak's lines, or its refusal.
fn kayak_says(path: &Path) -> Result<Vec<String>, String> {
    let r = fixpt::run(
        Command::new(release_bin("kayak-fx26"))
            .arg("--check")
            .arg(path),
        Some(LIMIT),
    );
    match r.code {
        None => Err("takes more than 60 s".into()),
        Some(0) => Ok(r
            .stdout
            .lines()
            .filter(|l| !l.is_empty())
            .map(String::from)
            .collect()),
        Some(_) => Err(r
            .stderr
            .lines()
            .find(|l| l.starts_with("kfx:") || l.starts_with("kayak-fx26:"))
            .or(r.stderr.trim().lines().last())
            .unwrap_or("fails")
            .to_string()),
    }
}

/// A list's parts at its top: each parenthesized part, and each word.
fn atoms(text: &str) -> Vec<String> {
    let b: Vec<char> = text.chars().collect();
    let (mut out, mut depth, mut start): (Vec<String>, i64, Option<usize>) = (vec![], 0, None);
    let s = |a: usize, z: usize| b[a..z].iter().collect::<String>();
    for (i, &c) in b.iter().enumerate() {
        if c == '(' {
            if depth == 0 {
                start = Some(i);
            }
            depth += 1;
        } else if c == ')' {
            depth -= 1;
            if depth == 0 {
                out.push(s(start.unwrap(), i + 1));
            }
        } else if c == ' ' && depth == 0 {
            if let Some(st) = start
                && st < i
                && b[st] != '('
            {
                out.push(s(st, i));
            }
            start = Some(i + 1);
        } else if depth == 0 && start.is_none() {
            start = Some(i);
        }
    }
    if let Some(st) = start
        && st < b.len()
        && b[st] != '('
    {
        let t = s(st, b.len());
        if !t.trim().is_empty() {
            out.push(t.trim().to_string());
        }
    }
    out
}

/// `line` with every `(maxeff ...)`'s atoms and `(globals ...)`'s names
/// sorted, as Fixpt writes them.
fn sorted_effects(text: &str) -> String {
    let b: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    'outer: while i < b.len() {
        for head in ["(maxeff ", "(globals "] {
            let h: Vec<char> = head.chars().collect();
            if b[i..].starts_with(&h) {
                let (mut depth, mut j) = (0i64, i);
                loop {
                    if b[j] == '(' {
                        depth += 1;
                    } else if b[j] == ')' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    j += 1;
                }
                let inner = sorted_effects(&b[i + h.len()..j].iter().collect::<String>());
                let mut parts = atoms(&inner);
                parts.sort();
                out.push_str(head);
                out.push_str(&parts.join(" "));
                out.push(')');
                i = j + 1;
                continue 'outer;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    out
}

fn compare(path: &Path) -> (&'static str, String) {
    let f = fixpt_says(path);
    let k = kayak_says(path);
    let Some(f) = f else {
        return match k {
            Err(_) => ("both", String::new()),
            Ok(_) => ("ACCEPTED", "kayak accepts what Fixpt refuses".into()),
        };
    };
    let k = match k {
        Err(why) => return ("refused", why),
        Ok(k) => k,
    };
    if f == k {
        return ("same", String::new());
    }
    let sorted = |ls: &[String]| ls.iter().map(|l| sorted_effects(l)).collect::<Vec<_>>();
    if sorted(&f) == sorted(&k) {
        return ("alike", String::new());
    }
    for i in 0..f.len().max(k.len()) {
        let a = f.get(i).map_or("(nothing)", String::as_str);
        let b = k.get(i).map_or("(nothing)", String::as_str);
        if sorted_effects(a) != sorted_effects(b) {
            return ("differ", format!("Fixpt: {a}\n      kayak: {b}"));
        }
    }
    ("differ", "the lines differ".into())
}

fn main() {
    let dir = root().join("target").join("kayak-checks").join("checkers");
    let tests = fixpt::checker_programs()
        .into_iter()
        .map(|(label, path)| {
            let dir = dir.clone();
            Test::new(format!("checkers/{label}"), move || {
                let (kind, detail) = compare(&path);
                std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
                std::fs::write(
                    dir.join(label.replace('/', "%")),
                    format!("{kind}\n{detail}"),
                )
                .map_err(|e| e.to_string())?;
                if kind == "ACCEPTED" {
                    Err(format!("K1: kayak accepts {label}, which Fixpt refuses"))
                } else {
                    Ok(())
                }
            })
        })
        .collect();
    kayak_checks::run(tests)
}
