//! `kfx FILE.kfx`: a program of one file, as Rust, on standard output.
//! `kfx PROGRAM.unit DIR`: a program of the files the manifest lists, in
//! order, as `DIR/mod.rs` and a module each.
//!
//! The compiler is generated from `kfx/` (`PLAN.md` §1); this file is the
//! only hand-written part of it.

#[allow(warnings, clippy::all)]
#[rustfmt::skip]
mod kfx;

use kayak_rt::{List, Str, cons, leak_string, nil, with_big_stack};
use std::path::Path;

fn read(path: &Path) -> Str {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("kfx: cannot read {}: {e}", path.display());
        std::process::exit(1);
    });
    // KFX's strings are ASCII, a byte a character (CLAUDES_CHOICES.md, C4),
    // and so is its source: said here, where the line can be, rather than
    // where the reader meets the byte.
    if let Some(n) = text.lines().position(|l| !l.is_ascii()) {
        eprintln!("kfx: {}:{}: not ASCII: KFX's sources are", path.display(), n + 1);
        std::process::exit(1);
    }
    leak_string(text)
}

/// A manifest's files, in order: a line each, relative to the manifest;
/// blank lines and `;` comments are skipped.
fn sources(manifest: &Path) -> List<&'static kfx::Source> {
    let dir = manifest.parent().unwrap_or(Path::new("."));
    let mut files: Vec<&'static kfx::Source> = Vec::new();
    for line in read(manifest).lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }
        let path = dir.join(line);
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_else(|| {
                eprintln!("kfx: {line}: not a file name");
                std::process::exit(2);
            });
        files.push(kfx::source(leak_string(name.to_owned()), read(&path)));
    }
    files.into_iter().rev().fold(nil(), |rest, s| cons(s, rest))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match &args[1..] {
        [file] => {
            let src = read(Path::new(file));
            let name = leak_string(file.clone());
            print!("{}", with_big_stack(move || kfx::compile_named(name, src)));
        }
        [manifest, dir] => {
            let program = sources(Path::new(manifest));
            let mut outputs = with_big_stack(move || kfx::compile_units(program));
            std::fs::create_dir_all(dir).unwrap_or_else(|e| {
                eprintln!("kfx: cannot make {dir}: {e}");
                std::process::exit(1);
            });
            while let Some(c) = outputs {
                let kfx::Output::Output(name, text) = *c.car;
                let path = Path::new(dir).join(name);
                std::fs::write(&path, text).unwrap_or_else(|e| {
                    eprintln!("kfx: cannot write {}: {e}", path.display());
                    std::process::exit(1);
                });
                outputs = c.cdr;
            }
        }
        _ => {
            eprintln!("usage: kfx FILE.kfx | kfx PROGRAM.unit DIR");
            std::process::exit(2);
        }
    }
}
