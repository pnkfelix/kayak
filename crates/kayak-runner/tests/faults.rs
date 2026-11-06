//! Faults a sim injects (M14): by seed, so a run with them is one run,
//! every time; and recorded as answers, so its replay needs no faults.
// coopt-source: crates/coopt-runner/tests/faults.rs@9283bba

use std::cell::RefCell;
use std::rc::Rc;

use kayak_runner::log::{Recorder, header, read};
use kayak_runner::runner::Checker;
use kayak_runner::{Answer, Faults, IoHub, Op, Outcome, Replay, Sim, drive};
use kayak_sched::{Config, Effect, Scheduler, Spawn};
use kayak_trace::{Memory, Sink};

/// What the program saw: each task's answers, in the order they came.
type Seen = Rc<RefCell<Vec<String>>>;

/// Two tasks: one appends ten lines to a file, each awaited; the other
/// reads the file five times.
fn program<S: Sink>(s: &mut Scheduler<S>, seen: &Seen) -> IoHub {
    let hub = IoHub::new();
    let (h, w) = (hub.clone(), seen.clone());
    s.spawn(
        Spawn::new("appender").effect(Effect::pure()),
        move |cx| async move {
            for i in 0..10 {
                let a = h
                    .io(&cx)
                    .perform(Op::AppendFile {
                        path: "f".into(),
                        bytes: format!("line {i}\n").into_bytes(),
                    })
                    .await;
                w.borrow_mut().push(format!("append {i}: {a:?}"));
            }
        },
    );
    let (h, w) = (hub.clone(), seen.clone());
    s.spawn(
        Spawn::new("reader").effect(Effect::pure()),
        move |cx| async move {
            for i in 0..5 {
                let a = h.io(&cx).perform(Op::ReadFile { path: "f".into() }).await;
                let said = match a {
                    Answer::Bytes(b) => format!("{} bytes", b.len()),
                    other => format!("{other:?}"),
                };
                w.borrow_mut().push(format!("read {i}: {said}"));
            }
        },
    );
    hub
}

struct Run {
    seen: Vec<String>,
    file: Vec<u8>,
    injected: Vec<String>,
}

fn run(faults: Option<Faults>) -> Run {
    let seen = Seen::default();
    let mut s = Scheduler::new(Config::default(), Memory::default());
    let hub = program(&mut s, &seen);
    let mut sim = Sim::new(Vec::new());
    if let Some(f) = faults {
        sim = sim.faults(f);
    }
    assert_eq!(drive(&mut s, &hub, &mut sim, 1_000_000), Outcome::Returned);
    let file = sim.files.borrow().get("f").cloned().unwrap_or_default();
    Run {
        seen: seen.take(),
        file,
        injected: sim.injected,
    }
}

fn some(seed: u64) -> Faults {
    Faults {
        seed,
        fail: 200,
        torn: 200,
        delay: 20,
    }
}

#[test]
fn a_seed_is_one_run() {
    for seed in 0..20 {
        let (a, b) = (run(Some(some(seed))), run(Some(some(seed))));
        assert_eq!(a.seen, b.seen, "seed {seed}");
        assert_eq!(a.file, b.file, "seed {seed}");
        assert_eq!(a.injected, b.injected, "seed {seed}");
    }
    // And seeds differ: not all twenty runs alike.
    let runs: Vec<_> = (0..20).map(|s| run(Some(some(s))).injected).collect();
    assert!(runs.iter().any(|r| *r != runs[0]), "{runs:?}");
    assert!(runs.iter().any(|r| !r.is_empty()), "{runs:?}");
}

#[test]
fn no_faults_at_rate_zero() {
    let plain = run(None);
    let zero = run(Some(Faults {
        seed: 7,
        ..Faults::default()
    }));
    assert_eq!(plain.seen, zero.seen);
    assert_eq!(plain.file, zero.file);
    assert!(zero.injected.is_empty());
    assert_eq!(plain.file.len(), "line 0\n".len() * 10);
}

#[test]
fn every_operation_fails_at_rate_a_thousand() {
    let r = run(Some(Faults {
        seed: 1,
        fail: 1000,
        ..Faults::default()
    }));
    assert!(r.file.is_empty());
    assert!(
        r.seen.iter().all(|s| s.contains("injected fault")),
        "{:?}",
        r.seen
    );
    assert_eq!(r.injected.len(), 15);
}

/// A torn append writes a strict prefix of its bytes and says so; the file
/// is what was written, in order.
#[test]
fn a_torn_append_writes_a_prefix() {
    let r = run(Some(Faults {
        seed: 3,
        torn: 1000,
        ..Faults::default()
    }));
    assert_eq!(r.injected.len(), 10, "{:?}", r.injected);
    let mut want = Vec::new();
    for (i, inj) in r.injected.iter().enumerate() {
        let kept: usize = inj
            .split(" at ")
            .nth(1)
            .and_then(|t| t.split(' ').next())
            .and_then(|k| k.parse().ok())
            .expect("a count");
        let line = format!("line {i}\n");
        assert!(kept < line.len(), "{inj}");
        want.extend_from_slice(&line.as_bytes()[..kept]);
    }
    assert_eq!(r.file, want);
}

/// Late answers overtake one another: the reader's and the appender's
/// interleave differently with delays than without, and all still come.
#[test]
fn late_answers_reorder_completions() {
    let plain = run(None);
    let orders: Vec<Vec<String>> = (0..20)
        .map(|seed| {
            run(Some(Faults {
                seed,
                delay: 50,
                ..Faults::default()
            }))
            .seen
        })
        .collect();
    for o in &orders {
        assert_eq!(o.len(), 15, "every answer came: {o:?}");
    }
    assert!(orders.iter().any(|o| *o != plain.seen), "{orders:?}");
}

/// A run with faults is recorded as its answers: replayed without them,
/// it is the same run.
#[test]
fn a_faulty_run_replays() {
    let seen = Seen::default();
    let log = Rc::new(RefCell::new(header(&[])));
    let mut s = Scheduler::new(
        Config::default(),
        Recorder::new(Memory::default(), log.clone(), true, 0),
    );
    let hub = program(&mut s, &seen);
    let mut sim = Sim::new(Vec::new()).faults(some(1));
    assert_eq!(drive(&mut s, &hub, &mut sim, 1_000_000), Outcome::Returned);
    assert!(!sim.injected.is_empty());
    let recorded = s.sink().inner.clone();
    let first = seen.take();

    let l = read(&log.borrow()).expect("a log");
    let (mut replay, expect) = Replay::new(&l);
    let mut s = Scheduler::new(Config::default(), Checker::new(Memory::default(), expect));
    let hub = program(&mut s, &seen);
    assert_eq!(
        drive(&mut s, &hub, &mut replay, 1_000_000),
        Outcome::Returned
    );
    assert_eq!(s.sink().inner, recorded);
    assert_eq!(seen.take(), first);
}
