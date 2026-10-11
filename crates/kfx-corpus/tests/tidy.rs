//! The repository's own rules, as tests (`TODO.md` §9). K4, that generated
//! Rust is safe, is the workspace's `unsafe_code = "deny"`: every generated
//! module is compiled in `boot` or here.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn files(dir: &str, ext: &str) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(root().join(dir))
        .unwrap_or_else(|e| panic!("{dir}: {e}"))
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == ext))
        .collect();
    found.sort();
    found
}

/// KFX's strings are ASCII (`CLAUDES_CHOICES.md`, C4): `string-ref-i64` is a
/// byte index, so a source the compiler reads must be ASCII too.
#[test]
fn kfx_sources_are_ascii() {
    let mut sources = files("kfx", "kfx");
    sources.extend(files("kfx", "fx"));
    sources.extend(files("tests/kfx", "kfx"));
    sources.extend(files("tests/kfx/two", "kfx"));
    sources.extend(files("tests/kfx-refused", "kfx"));
    sources.extend(files("tests/kfx-stricter", "kfx"));
    assert!(!sources.is_empty());
    let mut wrong = Vec::new();
    for path in sources {
        let text = std::fs::read_to_string(&path).unwrap();
        for (i, line) in text.lines().enumerate() {
            if !line.is_ascii() {
                wrong.push(format!("{}:{}: {line}", path.display(), i + 1));
            }
        }
    }
    assert!(wrong.is_empty(), "not ASCII:\n{}", wrong.join("\n"));
}
