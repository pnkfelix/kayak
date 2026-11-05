//! kayak's checker's types as reps (C301; `docs/research/type-reps.md`):
//! each program's definitions, by `kayak-fx26 --reps`, each exported,
//! imported and exported again. A test each, `reps/NAME`; it fails if a
//! type comes back as another rep (`differs`). What each found is kept
//! for counting: each definition `same`, or `declined` (a part a rep does
//! not take yet).
//!
//! And against Fixpt's (C352): `fixpt check --reps` (kayak's request,
//! landed in Fixpt 13114974), on a cellular machine, whose plain `subr` is
//! `cellular` as kayak's checker's is. Each name's rep, kayak's against
//! Fixpt's Rust checker's, in order: the program is `same` if every one
//! is the same bytes (or both declined), else `differ`; `refused` if kayak
//! refuses what Fixpt takes, `both` if both refuse it, `fixpt-refuses` if
//! Fixpt alone does (the checkers test's to judge). Kept in
//! target/kayak-checks/reps/NAME.fixpt, the kind first, for
//! `checks-summary reps`; it fails if a program in tests/reps-passing is
//! no longer `same`.

use kayak_checks::fixpt;
use kayak_checks::{Test, release_bin, root};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

const LIMIT: Duration = Duration::from_secs(60);

/// The machine Fixpt is asked on: a cellular one, as kayak's plain `subr`.
const MACHINE: &str = "fx-cells-rsinterp";

/// Each name and its rep, `HEX` or `declined`, from `--reps` lines (kayak's
/// `NAME HEX same`, `NAME declined: WHY`; Fixpt's `NAME HEX`, `NAME
/// declined`, and its `NAME differs REP`, which is no name's of its own).
fn reps_of(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|l| {
            let mut ws = l.split_whitespace();
            let n = ws.next()?;
            let r = ws.next()?;
            match r {
                "differs" | "none" => None,
                r if r.starts_with("declined") => Some((n.to_string(), "declined".to_string())),
                r => Some((n.to_string(), r.to_string())),
            }
        })
        .collect()
}

/// What kayak's reps of `path` are against Fixpt's: the kind, then why.
fn against_fixpt(path: &Path, ours: &fixpt::Ran) -> String {
    let args = vec![
        "--dialect".into(),
        "fx26".into(),
        "--machine".into(),
        MACHINE.into(),
        "check".into(),
        "--reps".into(),
        path.to_string_lossy().into_owned(),
    ];
    let theirs = match fixpt::ask(
        fixpt::binary().as_deref(),
        &args,
        &[path.to_path_buf()],
        Some(LIMIT),
    ) {
        Ok(r) => r,
        Err(e) => return format!("unasked\n{e}\n"),
    };
    match (ours.ok(), theirs.ok()) {
        (false, false) => "both\n".into(),
        (false, true) => format!("refused\n{}", ours.stderr),
        (true, false) => format!("fixpt-refuses\n{}", theirs.stderr),
        (true, true) => {
            let (k, f) = (reps_of(&ours.stdout), reps_of(&theirs.stdout));
            if k == f {
                return "same\n".into();
            }
            let at = k
                .iter()
                .zip(f.iter())
                .position(|(a, b)| a != b)
                .unwrap_or(k.len().min(f.len()));
            let show = |v: &Vec<(String, String)>| {
                v.get(at)
                    .map_or("(none)".to_string(), |(n, r)| format!("{n} {r}"))
            };
            format!("differ\nkayak {}\nfixpt {}\n", show(&k), show(&f))
        }
    }
}

fn main() {
    let dir = root().join("target").join("kayak-checks").join("reps");
    let passing: Vec<String> = std::fs::read_to_string(root().join("tests").join("reps-passing"))
        .map(|t| t.split_whitespace().map(String::from).collect())
        .unwrap_or_default();
    let mut programs = fixpt::programs();
    programs.extend(fixpt::kayak_programs());
    let tests = programs
        .into_iter()
        .map(|(label, path)| {
            let dir = dir.clone();
            let was = passing.contains(&label);
            Test::new(format!("reps/{label}"), move || {
                let r = fixpt::run(
                    Command::new(release_bin("kayak-fx26"))
                        .arg("--reps")
                        .arg(&path),
                    Some(LIMIT),
                );
                std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
                let file = label.replace('/', "%");
                std::fs::write(dir.join(&file), &r.stdout).map_err(|e| e.to_string())?;
                let found = against_fixpt(&path, &r);
                std::fs::write(dir.join(format!("{file}.fixpt")), &found)
                    .map_err(|e| e.to_string())?;
                if let Some(l) = r.stdout.lines().find(|l| l.contains(" differs")) {
                    return Err(format!("a type comes back as another rep: {l}"));
                }
                if was && !found.starts_with("same\n") {
                    return Err(format!("{label}'s reps are no longer Fixpt's:\n{found}"));
                }
                Ok(())
            })
        })
        .collect();
    kayak_checks::run(tests)
}
