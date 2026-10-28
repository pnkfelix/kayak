//! What the tests under nextest found, counted and said, as the scripts
//! they were said it (C276); and the passing lists blessed from it.
//!
//! usage: checks-summary cells [--bless | --bless-drop] [--why] [--list]
//!                             [--regs-differ] [--regs-declined]
//!
//! `cells`: the tier 1 tests' findings (tests/cells.rs), in
//! target/kayak-checks/cells/, for Fixpt's words and kayak's own. A
//! program with no finding was not run: the summary fails, as the check
//! would have. --bless rewrites tests/cells-passing and
//! tests/cellgen-passing from what agrees; one that agreed and no longer
//! does fails it unless --bless-drop (C235). --why counts the reasons
//! programs are not run; --list says, of kayak's own words, which are
//! Fixpt's exactly; --regs-differ and --regs-declined name the register
//! words that differ from Fixpt's, and those Fixpt has and kayak declined
//! (C259).

use kayak_checks::{fixpt, root};
use std::collections::BTreeMap;

struct Found {
    kind: String,
    why: String,
    same: bool,
    regs: Option<(usize, usize, usize)>,
    regs_differ: Vec<String>,
    regs_declined: Vec<String>,
}

fn read(suite: &str, name: &str) -> Option<Found> {
    let path = root()
        .join("target")
        .join("kayak-checks")
        .join("cells")
        .join(suite)
        .join(name.replace('/', "%"));
    let text = std::fs::read_to_string(path).ok()?;
    let mut f = Found {
        kind: String::new(),
        why: String::new(),
        same: false,
        regs: None,
        regs_differ: vec![],
        regs_declined: vec![],
    };
    for l in text.lines() {
        let (k, v) = l.split_once(' ').unwrap_or((l, ""));
        let words = || v.split_whitespace().map(String::from).collect::<Vec<_>>();
        match k {
            "kind" => f.kind = v.to_string(),
            "why" => f.why = v.to_string(),
            "same" => f.same = v == "true",
            "regs" => {
                let n: Vec<usize> = v.split_whitespace().map(|x| x.parse().unwrap()).collect();
                f.regs = Some((n[0], n[1], n[2]));
            }
            "regs-differ" => f.regs_differ = words(),
            "regs-declined" => f.regs_declined = words(),
            _ => {}
        }
    }
    Some(f)
}

fn cells(flags: &[String]) -> i32 {
    let has = |f: &str| flags.iter().any(|x| x == f);
    let bless = has("--bless") || has("--bless-drop");
    let mut programs = fixpt::programs();
    programs.extend(fixpt::kayak_programs());
    let mut status = 0;
    for suite in ["fixpt", "ours"] {
        let ours = suite == "ours";
        let mut by: BTreeMap<String, Vec<String>> = BTreeMap::new();
        let mut found = Vec::new();
        let mut missing = Vec::new();
        for (name, _) in &programs {
            match read(suite, name) {
                Some(f) => {
                    by.entry(f.kind.clone()).or_default().push(name.clone());
                    found.push((name.clone(), f));
                }
                None => missing.push(name.clone()),
            }
        }
        if !missing.is_empty() {
            println!(
                "cells-programs: {} program(s) not run ({suite}), the first {}",
                missing.len(),
                missing[0]
            );
            status = 1;
            continue;
        }
        let count = |k: &str| by.get(k).map_or(0, Vec::len);
        let mut agree: Vec<String> = by.get("agree").cloned().unwrap_or_default();
        agree.sort();
        let list = root().join("tests").join(if ours {
            "cellgen-passing"
        } else {
            "cells-passing"
        });
        let before: Vec<String> = std::fs::read_to_string(&list)
            .map(|t| t.split_whitespace().map(String::from).collect())
            .unwrap_or_default();
        let lost: Vec<&String> = before.iter().filter(|n| !agree.contains(n)).collect();
        if bless {
            if !lost.is_empty() && !has("--bless-drop") {
                for n in &lost {
                    println!("cells-programs: WOULD DROP  {n}");
                }
                println!(
                    "cells-programs: --bless would drop {} program(s) from its passing list; nothing written. \
                     Fix them, or bless with --bless-drop to drop them.",
                    lost.len()
                );
                status = 1;
            } else {
                for n in &lost {
                    println!("cells-programs: DROPPED  {n}");
                }
                std::fs::write(&list, agree.join("\n") + "\n").unwrap();
            }
        }
        if has("--why") {
            let mut reasons: BTreeMap<(String, String), usize> = BTreeMap::new();
            for (_, f) in &found {
                if ["refused", "not-yet", "gen-refuses"].contains(&f.kind.as_str()) {
                    *reasons
                        .entry((f.kind.clone(), f.why.chars().take(150).collect()))
                        .or_default() += 1;
                }
            }
            let mut rs: Vec<_> = reasons.into_iter().collect();
            rs.sort_by_key(|a| std::cmp::Reverse(a.1));
            for ((k, w), n) in rs {
                println!("{n:5}  {k}: {w}");
            }
        }
        for (name, f) in &found {
            if f.kind == "differ" {
                println!("DIFFER  {name}: {}", f.why);
                status = 1;
            }
        }
        if !bless {
            for n in &lost {
                println!("NO LONGER AGREES  {n}");
                status = 1;
            }
        }
        if ours {
            let same_n = found
                .iter()
                .filter(|(n, f)| agree.contains(n) && f.same)
                .count();
            if has("--list") {
                for (n, f) in &found {
                    if agree.contains(n) {
                        println!("{} {n}", if f.same { "same " } else { "other" });
                    }
                }
            }
            println!(
                "cells-programs: kayak's own words: {} programs agree, {} differ, {} of them Fixpt's words exactly; \
                 {} the generator cannot yet",
                agree.len(),
                count("differ"),
                same_n,
                count("gen-refuses")
            );
            // Where the stack words are Fixpt's exactly, the register words
            // are named alike: there, each is compared.
            let rs: Vec<&Found> = found
                .iter()
                .filter(|(_, f)| f.same && f.regs.is_some())
                .map(|(_, f)| f)
                .collect();
            println!(
                "cells-programs: kayak's register words, where its stack words are Fixpt's: {} Fixpt's exactly, \
                 {} differ, {} Fixpt's only (declined), {} kayak's only",
                rs.iter().map(|f| f.regs.unwrap().0).sum::<usize>(),
                rs.iter().map(|f| f.regs_differ.len()).sum::<usize>(),
                rs.iter().map(|f| f.regs.unwrap().1).sum::<usize>(),
                rs.iter().map(|f| f.regs.unwrap().2).sum::<usize>()
            );
            for (n, f) in &found {
                if f.same && has("--regs-differ") && !f.regs_differ.is_empty() {
                    println!("REGS DIFFER  {n}: {}", f.regs_differ.join(" "));
                }
                if f.same && has("--regs-declined") && !f.regs_declined.is_empty() {
                    println!("REGS DECLINED  {n}: {}", f.regs_declined.join(" "));
                }
            }
        }
        println!(
            "cells-programs: {} of Fixpt's programs agree on tier 1, {} differ; not run: {} listings not exact, \
             {} need what tier 1 has not yet, {} leave no value, {} Fixpt refuses",
            agree.len(),
            count("differ"),
            count("refused"),
            count("not-yet"),
            count("no-value"),
            count("fixpt-refuses")
        );
    }
    status
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let status = match args.first().map(String::as_str) {
        Some("cells") => cells(&args[1..]),
        _ => {
            eprintln!(
                "usage: checks-summary cells [--bless | --bless-drop] [--why] [--list] [--regs-differ] [--regs-declined]"
            );
            2
        }
    };
    std::process::exit(status);
}
