//! `stage` on the engine (C305, C306): each of Fixpt's CLI's stage
//! programs (tests/fx26/stage), run by Fixpt (`fixpt run`, its `main`)
//! and by the engine (the program, then `(main)`, whose value the engine
//! writes last). A test each, `stage/NAME`; it fails if a program in
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
                let theirs = fixpt::ask(
                    fixpt::binary().as_deref(),
                    &args,
                    std::slice::from_ref(&path),
                    Some(LIMIT),
                )
                .map(|r| r.stdout)
                .unwrap_or_default();
                std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
                let with_main = out.join(format!("{}.fx", label.replace('/', "%")));
                let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
                std::fs::write(&with_main, format!("{text}\n(main)\n"))
                    .map_err(|e| e.to_string())?;
                let r = fixpt::run(
                    Command::new(release_bin("kayak-fx26")).arg(&with_main),
                    Some(LIMIT),
                );
                let mut lines: Vec<&str> = r.stdout.lines().collect();
                lines.pop();
                let ours = lines.iter().map(|l| format!("{l}\n")).collect::<String>();
                let same = r.code == Some(0) && ours == theirs;
                std::fs::write(
                    out.join(label.replace('/', "%")),
                    if same {
                        "same\n".to_string()
                    } else {
                        format!("differ\nFixpt:\n{theirs}kayak:\n{}{}", r.stdout, r.stderr)
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
