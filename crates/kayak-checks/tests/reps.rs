//! kayak's checker's types as reps (C301; `docs/research/type-reps.md`):
//! each program's definitions, by `kayak-fx26 --reps`, each exported,
//! imported and exported again. A test each, `reps/NAME`; it fails if a
//! type comes back as another rep (`differs`). What each found is kept
//! for counting: each definition `same`, or `declined` (a part a rep does
//! not take yet).

use kayak_checks::fixpt;
use kayak_checks::{Test, release_bin, root};
use std::process::Command;
use std::time::Duration;

fn main() {
    let dir = root().join("target").join("kayak-checks").join("reps");
    let mut programs = fixpt::programs();
    programs.extend(fixpt::kayak_programs());
    let tests = programs
        .into_iter()
        .map(|(label, path)| {
            let dir = dir.clone();
            Test::new(format!("reps/{label}"), move || {
                let r = fixpt::run(
                    Command::new(release_bin("kayak-fx26"))
                        .arg("--reps")
                        .arg(&path),
                    Some(Duration::from_secs(60)),
                );
                std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
                std::fs::write(dir.join(label.replace('/', "%")), &r.stdout)
                    .map_err(|e| e.to_string())?;
                match r.stdout.lines().find(|l| l.contains(" differs")) {
                    Some(l) => Err(format!("a type comes back as another rep: {l}")),
                    None => Ok(()),
                }
            })
        })
        .collect();
    kayak_checks::run(tests)
}
