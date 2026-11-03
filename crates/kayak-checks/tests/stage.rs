//! `stage` on the engine (C305, C306): each of Fixpt's CLI's stage
//! programs (tests/fx26/stage), run by Fixpt (`fixpt run`, its `main`)
//! and by the engine (`kayak-fx26 --run`, the same, its `main`'s value the
//! exit status, C315), or both refusing it as it is checked, in the same
//! words (C312). A test each, `stage/NAME`; it fails if a program in
//! tests/stage-passing no longer prints what Fixpt prints. What each found
//! is kept in target/kayak-checks/stage: `same`, or `differ` and both.

use kayak_checks::fixpt;
use kayak_checks::{Test, release_bin, root};
use std::process::Command;
use std::time::Duration;

const LIMIT: Duration = Duration::from_secs(60);

fn main() {
    let out = root().join("target").join("kayak-checks").join("stage");
    let passing: Vec<String> = std::fs::read_to_string(root().join("tests").join("stage-passing"))
        .map(|t| t.split_whitespace().map(String::from).collect())
        .unwrap_or_default();
    let tests = fixpt::stage_programs()
        .into_iter()
        .map(|(label, path)| {
            let out = out.clone();
            let was = passing.contains(&label);
            Test::new(format!("stage/{label}"), move || {
                let args = vec![
                    "--dialect".into(),
                    "fx26".into(),
                    "run".into(),
                    path.to_string_lossy().into_owned(),
                ];
                let asked = fixpt::ask(
                    fixpt::binary().as_deref(),
                    &args,
                    std::slice::from_ref(&path),
                    Some(LIMIT),
                )
                .ok();
                let theirs = asked.as_ref().map(|r| r.stdout.clone()).unwrap_or_default();
                // Refused as it is checked (C312): its words, past `fixpt: ` and
                // its place.
                let refused = asked
                    .as_ref()
                    .filter(|r| !r.ok() && r.stdout.is_empty())
                    .and_then(|r| {
                        r.stderr
                            .lines()
                            .find_map(|l| l.strip_prefix("fixpt: "))
                            .map(|l| fixpt::after_place(l, 3))
                    });
                std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
                // Run as Fixpt's `run` runs it (C315): the program, then its
                // `main`, if it has one, whose value is the exit status.
                let r = fixpt::run(
                    Command::new(release_bin("kayak-fx26"))
                        .arg("--run")
                        .arg(&path),
                    Some(LIMIT),
                );
                let ours = r.stdout.clone();
                let theirs_code = asked.as_ref().and_then(|r| r.code);
                let same = match &refused {
                    Some(words) => {
                        r.code != Some(0)
                            && r.stderr
                                .lines()
                                .find(|l| l.starts_with("kayak-fx26:"))
                                .map(fixpt::kayak_words)
                                .as_deref()
                                == Some(words.as_str())
                    }
                    None => r.code == theirs_code && ours == theirs,
                };
                std::fs::write(
                    out.join(label.replace('/', "%")),
                    if same {
                        "same\n".to_string()
                    } else {
                        format!(
                            "differ\nFixpt:\n{theirs}{}kayak:\n{}{}",
                            refused
                                .map(|w| format!("refused: {w}\n"))
                                .unwrap_or_default(),
                            r.stdout,
                            r.stderr
                        )
                    },
                )
                .map_err(|e| e.to_string())?;
                if was && !same {
                    Err(format!("{label} no longer runs as Fixpt runs it"))
                } else {
                    Ok(())
                }
            })
        })
        .collect();
    kayak_checks::run(tests)
}
