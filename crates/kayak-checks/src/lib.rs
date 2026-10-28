//! A test harness for checks that are many tests found at run time (C276):
//! a program each, a chain of edits each. Under `cargo nextest`, every one
//! is a test of its own, scheduled with every other check's, so the
//! longest start first and no check waits for another's slowest. No
//! dependency: the part of libtest's command line nextest uses, and
//! `cargo test`'s plain run.
//!
//! nextest lists with `--list --format terse` (and `--ignored`, of which
//! there are none) and runs one with `NAME --exact`; `cargo test` runs the
//! binary with no arguments, or a filter, and gets every test, at once.

use std::path::PathBuf;
use std::process::{Command, Output};

/// A test: its name, and what it does, `Err` saying why it fails.
pub struct Test {
    pub name: String,
    pub run: Box<dyn Fn() -> Result<(), String> + Send + Sync>,
}

impl Test {
    pub fn new(
        name: impl Into<String>,
        run: impl Fn() -> Result<(), String> + Send + Sync + 'static,
    ) -> Test {
        Test {
            name: name.into(),
            run: Box::new(run),
        }
    }
}

/// The harness: list, or run those asked for, and exit as libtest does.
pub fn run(tests: Vec<Test>) -> ! {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let flag = |f: &str| args.iter().any(|a| a == f);
    let exact = flag("--exact");
    // libtest's options that take a value, which is no filter.
    const VALUED: [&str; 6] = [
        "--format",
        "--test-threads",
        "--color",
        "--logfile",
        "--skip",
        "-Z",
    ];
    let filters: Vec<&String> = args
        .iter()
        .enumerate()
        .filter(|(i, a)| {
            !a.starts_with('-') && (*i == 0 || !VALUED.contains(&args[i - 1].as_str()))
        })
        .map(|(_, a)| a)
        .collect();
    let wanted = |t: &Test| {
        filters.is_empty()
            || filters.iter().any(|f| {
                if exact {
                    t.name == **f
                } else {
                    t.name.contains(f.as_str())
                }
            })
    };
    if flag("--list") {
        if !flag("--ignored") {
            for t in tests.iter().filter(|t| wanted(t)) {
                println!("{}: test", t.name);
            }
        }
        std::process::exit(0);
    }
    if flag("--ignored") {
        std::process::exit(0);
    }
    let chosen: Vec<&Test> = tests.iter().filter(|t| wanted(t)).collect();
    let failed: Vec<(String, String)> = std::thread::scope(|s| {
        let running: Vec<_> = chosen
            .iter()
            .map(|t| (t.name.clone(), s.spawn(|| (t.run)())))
            .collect();
        running
            .into_iter()
            .filter_map(|(name, h)| match h.join() {
                Ok(Ok(())) => {
                    println!("test {name} ... ok");
                    None
                }
                Ok(Err(why)) => Some((name, why)),
                Err(_) => Some((name, "panicked".to_string())),
            })
            .collect()
    });
    for (name, why) in &failed {
        println!("test {name} ... FAILED\n{why}");
    }
    println!(
        "\ntest result: {}. {} passed; {} failed",
        if failed.is_empty() { "ok" } else { "FAILED" },
        chosen.len() - failed.len(),
        failed.len()
    );
    std::process::exit(if failed.is_empty() { 0 } else { 101 });
}

/// The repository's root.
pub fn root() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

/// A program built by `cargo build --release`, as `scripts/check.sh` has
/// it before the tests run.
pub fn release_bin(name: &str) -> PathBuf {
    root().join("target").join("release").join(name)
}

/// Whether `scripts/check.sh --quick` asked for fewer cases
/// (`KAYAK_CHECK_QUICK`).
pub fn quick() -> bool {
    std::env::var_os("KAYAK_CHECK_QUICK").is_some()
}

/// A command run to its end; `Err` with its standard error if it fails.
pub fn ran(cmd: &mut Command) -> Result<Output, String> {
    let out = cmd.output().map_err(|e| format!("{cmd:?}: {e}"))?;
    if out.status.success() {
        Ok(out)
    } else {
        Err(format!(
            "{cmd:?} failed:\n{}",
            String::from_utf8_lossy(&out.stderr)
        ))
    }
}

/// A directory of its own, removed when dropped.
pub struct TempDir(pub PathBuf);

impl TempDir {
    pub fn new(label: &str) -> TempDir {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let base = std::env::var_os("TMPDIR")
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        let dir = base.join(format!(
            "kayak-checks-{}-{}-{}",
            label,
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).expect("a temporary directory");
        TempDir(dir)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// `from`'s files and directories, copied under `to`.
pub fn copy_tree(from: &std::path::Path, to: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for e in std::fs::read_dir(from)? {
        let e = e?;
        let target = to.join(e.file_name());
        if e.file_type()?.is_dir() {
            copy_tree(&e.path(), &target)?;
        } else {
            std::fs::copy(e.path(), target)?;
        }
    }
    Ok(())
}
