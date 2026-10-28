//! What Fixpt says of a file, asked once (C132; `scripts/fixptcache.py`,
//! ported, C276): the same keys, the same stores, the same files written,
//! so that the Python checks not yet ported and these share them. See that
//! script's documentation for the two stores (tests/fixpt-answers/, kept in
//! git, and target/fixpt-cache/, every run on this machine) and the keys
//! (`PLATFORM@PIN`) an answer is held by.

use crate::json::{self, Json};
use crate::root;
use crate::sha256::Sha256;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

/// A run's status and output. `code` is `None` for a run stopped at its
/// time limit.
#[derive(Clone, Debug)]
pub struct Ran {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

impl Ran {
    pub fn ok(&self) -> bool {
        self.code == Some(0)
    }
}

/// `cmd` run, its standard input empty, as Fixpt's runs are given it (a
/// program that reads it gets the end, C233); stopped after `timeout`.
pub fn run(cmd: &mut Command, timeout: Option<Duration>) -> Ran {
    let mut child = cmd
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("{cmd:?}: {e}"));
    // The output read as it comes, so that a full pipe never stops the run.
    let mut out = child.stdout.take().unwrap();
    let mut err = child.stderr.take().unwrap();
    let o = std::thread::spawn(move || {
        let mut v = Vec::new();
        let _ = std::io::Read::read_to_end(&mut out, &mut v);
        v
    });
    let e = std::thread::spawn(move || {
        let mut v = Vec::new();
        let _ = std::io::Read::read_to_end(&mut err, &mut v);
        v
    });
    let start = Instant::now();
    let code = loop {
        match child.try_wait() {
            Ok(Some(s)) => break Some(s.code().unwrap_or(-1)),
            Ok(None) => {}
            Err(_) => break Some(-1),
        }
        if timeout.is_some_and(|t| start.elapsed() > t) {
            let _ = child.kill();
            let _ = child.wait();
            break None;
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    Ran {
        code,
        stdout: String::from_utf8_lossy(&o.join().unwrap()).into_owned(),
        stderr: String::from_utf8_lossy(&e.join().unwrap()).into_owned(),
    }
}

pub fn rev() -> &'static str {
    static REV: OnceLock<String> = OnceLock::new();
    REV.get_or_init(|| {
        std::fs::read_to_string(root().join("FIXPT_REV"))
            .unwrap()
            .trim()
            .to_string()
    })
}

/// The Fixpt of the pin (`scripts/fixpt.sh`), or none when there is no
/// clone at the pin: then the answers kept are all there is.
pub fn binary() -> Option<PathBuf> {
    static BIN: OnceLock<Option<PathBuf>> = OnceLock::new();
    BIN.get_or_init(|| {
        let out = Command::new(root().join("scripts").join("fixpt.sh"))
            .stderr(Stdio::null())
            .output()
            .ok()?;
        out.status
            .success()
            .then(|| PathBuf::from(String::from_utf8_lossy(&out.stdout).trim()))
    })
    .clone()
}

/// This machine, as Rust's `std::env::consts` names it, `ARCH-OS`.
pub fn platform() -> String {
    format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS)
}

fn me() -> String {
    format!("{}@{}", platform(), rev())
}

pub fn programs_dir() -> PathBuf {
    root().join("tests").join("fx26").join("programs")
}

pub fn kayak_programs_dir() -> PathBuf {
    root().join("tests").join("fx26-kayak")
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().is_some_and(|x| x == "fx") {
            out.push(p);
        }
    }
}

fn rel(p: &Path, base: &Path) -> String {
    p.strip_prefix(base).unwrap().to_string_lossy().into_owned()
}

/// Fixpt's programs kept, each as `DIR/NAME.fx`, and its path, sorted.
pub fn programs() -> Vec<(String, PathBuf)> {
    let base = programs_dir();
    let mut ps = Vec::new();
    walk(&base, &mut ps);
    let mut out: Vec<_> = ps.into_iter().map(|p| (rel(&p, &base), p)).collect();
    out.sort();
    out
}

/// Kayak's own programs (C229), each as `kayak/DIR/NAME.fx`, sorted.
pub fn kayak_programs() -> Vec<(String, PathBuf)> {
    let base = kayak_programs_dir();
    let mut ps = Vec::new();
    walk(&base, &mut ps);
    let mut out: Vec<_> = ps
        .into_iter()
        .map(|p| (format!("kayak/{}", rel(&p, &base)), p))
        .collect();
    out.sort();
    out
}

fn answers_dir() -> PathBuf {
    root().join("tests").join("fixpt-answers")
}

fn cache_dir() -> PathBuf {
    root().join("target").join("fixpt-cache")
}

/// The questions kept, by key: each file NAME.KEY.json.
fn index() -> &'static Mutex<HashMap<String, PathBuf>> {
    static INDEX: OnceLock<Mutex<HashMap<String, PathBuf>>> = OnceLock::new();
    INDEX.get_or_init(|| {
        let mut m = HashMap::new();
        if let Ok(rd) = std::fs::read_dir(answers_dir()) {
            for e in rd.flatten() {
                let f = e.file_name().to_string_lossy().into_owned();
                if let Some(stem) = f.strip_suffix(".json") {
                    let key = stem.rsplit('.').next().unwrap_or("");
                    if key.len() == 20
                        && key
                            .bytes()
                            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                    {
                        m.insert(key.to_string(), e.path());
                    }
                }
            }
        }
        Mutex::new(m)
    })
}

/// A question asked, noted for `fx26-tests.py --prune`.
fn note_used(key: &str) {
    let dir = root().join("target").join("fixpt-answers-used");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join(format!("{}.txt", std::process::id()));
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = std::io::Write::write_all(&mut f, format!("{key}\n").as_bytes());
    }
}

const HERE: &str = "<kayak>";
const THERE: &str = "<programs>";

/// A temporary directory a script made, `/tmp/tmpXXXXXXXX` or macOS's
/// `/var/folders/…/T/tmpXXXXXXXX`, as `<tmp>`.
fn temporaries(text: &str) -> String {
    let b = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    let name_ok = |c: u8| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_';
    let seg_ok = |c: u8| c.is_ascii_alphanumeric() || c == b'_' || c == b'+' || c == b'-';
    let tail = |j: usize| -> Option<usize> {
        // `/tmp` and 8 name characters.
        if b[j..].starts_with(b"/tmp")
            && b.len() >= j + 12
            && b[j + 4..j + 12].iter().all(|&c| name_ok(c))
        {
            Some(j + 12)
        } else {
            None
        }
    };
    while i < b.len() {
        let mut j = i;
        if b[j..].starts_with(b"/private") {
            j += 8;
        }
        let mut end = None;
        if b[j..].starts_with(b"/tmp") {
            end = tail(j + 4);
        } else if b[j..].starts_with(b"/var/folders/") {
            let mut k = j + 13;
            let s1 = k;
            while k < b.len() && seg_ok(b[k]) {
                k += 1;
            }
            if k > s1 && b.get(k) == Some(&b'/') {
                k += 1;
                let s2 = k;
                while k < b.len() && seg_ok(b[k]) {
                    k += 1;
                }
                if k > s2 && b[k..].starts_with(b"/T") {
                    end = tail(k + 2);
                }
            }
        }
        match end {
            Some(e) => {
                out.push_str("<tmp>");
                i = e;
            }
            None => {
                let c = text[i..].chars().next().unwrap();
                out.push(c);
                i += c.len_utf8();
            }
        }
    }
    out
}

fn root_text() -> String {
    // As Python's `os.path.abspath`: the root, its `..` resolved.
    std::fs::canonicalize(root())
        .unwrap()
        .to_string_lossy()
        .into_owned()
}

fn placeless(text: &str) -> String {
    let t = temporaries(&text.replace(&root_text(), HERE));
    t.replace(
        &format!("{HERE}/../kayak-refs/fixpt/crates/fixpt-fx26/tests/programs"),
        THERE,
    )
    .replace(&format!("{HERE}/tests/fx26/programs"), THERE)
}

fn placed(lines: &[String]) -> String {
    lines
        .join("\n")
        .replace(THERE, &programs_dir_text())
        .replace(HERE, &root_text())
}

fn programs_dir_text() -> String {
    format!("{}/tests/fx26/programs", root_text())
}

fn answer_of(r: &Ran, timeout: Option<Duration>) -> Json {
    match r.code {
        Some(c) => Json::Obj(vec![
            ("code".into(), Json::Int(c as i64)),
            (
                "out".into(),
                Json::Arr(
                    placeless(&r.stdout)
                        .split('\n')
                        .map(|l| Json::Str(l.into()))
                        .collect(),
                ),
            ),
            (
                "err".into(),
                Json::Arr(
                    placeless(&r.stderr)
                        .split('\n')
                        .map(|l| Json::Str(l.into()))
                        .collect(),
                ),
            ),
        ]),
        None => Json::Obj(vec![
            ("code".into(), Json::Str("timeout".into())),
            (
                "out".into(),
                Json::Arr(vec![Json::Str(
                    timeout.map_or(String::new(), |t| t.as_secs().to_string()),
                )]),
            ),
            ("err".into(), Json::Arr(vec![Json::Str(String::new())])),
        ]),
    }
}

fn ran_of(a: &Json) -> Ran {
    let lines = |k: &str| placed(&a.get(k).map(Json::strs).unwrap_or_default());
    match a.get("code") {
        Some(Json::Int(c)) => Ran {
            code: Some(*c as i32),
            stdout: lines("out"),
            stderr: lines("err"),
        },
        _ => Ran {
            code: None,
            stdout: String::new(),
            stderr: String::new(),
        },
    }
}

fn same(a: &Json, b: &Json) -> bool {
    a.get("code") == b.get("code") && a.get("out") == b.get("out") && a.get("err") == b.get("err")
}

fn by(a: &Json) -> Vec<String> {
    a.get("by").map(Json::strs).unwrap_or_default()
}

fn set_by(a: &mut Json, keys: Vec<String>) {
    let v = Json::Arr(keys.into_iter().map(Json::Str).collect());
    match a.get_mut("by") {
        Some(x) => *x = v,
        None => {
            if let Json::Obj(ms) = a {
                ms.push(("by".into(), v));
            }
        }
    }
}

fn store(path: &Path, d: &mut Json) {
    if let Some(Json::Arr(answers)) = d.get_mut("answers") {
        answers.retain(|a| !by(a).is_empty());
        for a in answers.iter_mut() {
            let mut keys = by(a);
            keys.sort();
            keys.dedup();
            set_by(a, keys);
        }
        if answers.is_empty() {
            let _ = std::fs::remove_file(path);
            return;
        }
    }
    let text = json::dumps_indent0(d) + "\n";
    if std::fs::read_to_string(path).ok().as_deref() == Some(text.as_str()) {
        return;
    }
    let _ = std::fs::create_dir_all(answers_dir());
    let tmp = path.with_extension(format!(
        "json.{}.{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::write(&tmp, text).unwrap();
    std::fs::rename(&tmp, path).unwrap();
}

/// Fixpt asked, through target/fixpt-cache/ (its key the pin's).
fn asked(binary: &Path, args: &[String], old: &str, timeout: Option<Duration>) -> Ran {
    let path = cache_dir().join(format!("{old}.json"));
    if let Some(d) = std::fs::read_to_string(&path)
        .ok()
        .and_then(|t| json::parse(&t).ok())
    {
        let s = |k: &str| d.get(k).and_then(Json::str).unwrap_or("").to_string();
        if let Some(Json::Int(c)) = d.get("code") {
            return Ran {
                code: Some(*c as i32),
                stdout: s("out"),
                stderr: s("err"),
            };
        }
    }
    let r = run(Command::new(binary).args(args), timeout);
    if let Some(c) = r.code {
        let _ = std::fs::create_dir_all(cache_dir());
        let d = Json::Obj(vec![
            ("code".into(), Json::Int(c as i64)),
            ("out".into(), Json::Str(r.stdout.clone())),
            ("err".into(), Json::Str(r.stderr.clone())),
        ]);
        let tmp = path.with_extension(format!(
            "json.{}.{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        std::fs::write(&tmp, json::dumps(&d)).unwrap();
        std::fs::rename(&tmp, &path).unwrap();
    }
    r
}

static LOCK: Mutex<()> = Mutex::new(());

/// Fixpt (`binary`, none with no clone at the pin) given `args`, which name
/// `files` (read for the key): its status and output, as kept, or asked.
/// `Err` when there is no clone and nothing is kept. As `fixptcache.run`.
pub fn ask(
    binary: Option<&Path>,
    args: &[String],
    files: &[PathBuf],
    timeout: Option<Duration>,
) -> Result<Ran, String> {
    let mut h = Sha256::new();
    let mut old = Sha256::new();
    old.update(rev().as_bytes());
    let file_strs: Vec<String> = files
        .iter()
        .map(|f| f.to_string_lossy().into_owned())
        .collect();
    let shown: Vec<String> = args
        .iter()
        .map(|a| {
            if file_strs.contains(a) {
                Path::new(a)
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned()
            } else {
                a.clone()
            }
        })
        .collect();
    for s in &shown {
        let mut part = vec![0u8];
        part.extend_from_slice(s.as_bytes());
        h.update(&part);
        old.update(&part);
    }
    for f in files {
        let mut part = vec![1u8];
        part.extend(std::fs::read(f).map_err(|e| format!("{}: {e}", f.display()))?);
        h.update(&part);
        old.update(&part);
    }
    let key = h.hex()[..20].to_string();
    let old = old.hex();
    note_used(&key);
    let kept = {
        let _g = LOCK.lock().unwrap();
        index().lock().unwrap().get(&key).cloned()
    }
    .and_then(|p| std::fs::read_to_string(&p).ok())
    .and_then(|t| json::parse(&t).ok());
    if let Some(k) = &kept {
        let answers = k
            .get("answers")
            .and_then(Json::arr)
            .cloned()
            .unwrap_or_default();
        if let Some(a) = answers.iter().find(|a| by(a).contains(&me())) {
            return Ok(ran_of(a));
        }
        if binary.is_none() {
            let standing = [format!("{}@", platform()), "unknown@".to_string()]
                .iter()
                .find_map(|pre| {
                    answers
                        .iter()
                        .find(|a| by(a).iter().any(|k| k.starts_with(pre.as_str())))
                })
                .or(answers.first());
            if let Some(a) = standing {
                return Ok(ran_of(a));
            }
        }
    }
    let Some(binary) = binary else {
        return Err(format!(
            "no answer of Fixpt's kept for {} (a clone at the pin makes it)",
            shown.join(" ")
        ));
    };
    let r = asked(binary, args, &old, timeout);
    let progs = programs_dir();
    let stem = match files.first() {
        Some(f) if f.starts_with(&progs) => {
            let r = rel(f, &progs);
            r.strip_suffix(".fx").unwrap_or(&r).replace('/', "--")
        }
        Some(f) => f.file_stem().unwrap().to_string_lossy().into_owned(),
        None => "run".into(),
    };
    let mut new = answer_of(&r, timeout);
    let _g = LOCK.lock().unwrap();
    let path = index()
        .lock()
        .unwrap()
        .get(&key)
        .cloned()
        .unwrap_or_else(|| answers_dir().join(format!("{stem}.{key}.json")));
    let mut d = std::fs::read_to_string(&path)
        .ok()
        .and_then(|t| json::parse(&t).ok())
        .unwrap_or_else(|| {
            Json::Obj(vec![
                (
                    "args".into(),
                    Json::Arr(shown.iter().cloned().map(Json::Str).collect()),
                ),
                ("answers".into(), Json::Arr(vec![])),
            ])
        });
    let me = me();
    if let Some(Json::Arr(answers)) = d.get_mut("answers") {
        for a in answers.iter_mut() {
            let keys = by(a).into_iter().filter(|k| *k != me).collect();
            set_by(a, keys);
        }
        if let Some(a) = answers.iter_mut().find(|a| same(a, &new)) {
            let mut keys = by(a);
            keys.push(me.clone());
            set_by(a, keys);
        } else {
            set_by(&mut new, vec![me.clone()]);
            answers.push(new.clone());
        }
    }
    store(&path, &mut d);
    index().lock().unwrap().insert(key, path);
    Ok(ran_of(&new))
}

#[cfg(test)]
mod tests {
    use super::temporaries;

    #[test]
    fn temporary_directories_named_so() {
        assert_eq!(temporaries("at /tmp/tmpab12_x9z/f.fx:3"), "at <tmp>/f.fx:3");
        assert_eq!(
            temporaries("/private/var/folders/ab/c_d/T/tmpqwertyui/x"),
            "<tmp>/x"
        );
        assert_eq!(temporaries("/tmp/tmpAB/x"), "/tmp/tmpAB/x");
    }
}
