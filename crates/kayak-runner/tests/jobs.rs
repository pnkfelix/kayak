//! Jobs (`TODO.md` §48, stage 1): pure work on worker threads, in every
//! runner, its completion an arrival like any other. A sim run is the same
//! run every time, however the threads go; a replay runs the jobs again
//! and finds a job that answers otherwise.
// coopt-source: crates/coopt-runner/tests/jobs.rs@9283bba

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use kayak_runner::live::Live;
use kayak_runner::log::{Recorder, header, read};
use kayak_runner::runner::Checker;
use kayak_runner::values::{UseValues, Values};
use kayak_runner::{IoHub, Kill, Op, Outcome, Replay, Runner, Sim, drive};
use kayak_sched::{Config, Effect, Job, Scheduler, Spawn, Stress};
use kayak_trace::{Memory, Sink};

type Seen = Rc<RefCell<Vec<String>>>;

/// Work for a job: `rounds` steps of splitmix64 from `seed`.
fn mix(seed: u64, rounds: u64) -> u64 {
    let mut x = seed;
    for _ in 0..rounds {
        x = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = x;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        x = z ^ (z >> 31);
    }
    x
}

const TASKS: u64 = 8;
const JOBS: u64 = 4;
const ROUNDS: u64 = 200_000;

/// `TASKS` tasks, each running `JOBS` jobs one after another, the next
/// one's seed the last one's value; and a task that reads the clock as
/// they go.
/// `skew` is added to one job's value: a job that answers otherwise.
fn program<S: Sink>(s: &mut Scheduler<S>, seen: &Seen, skew: u64) -> IoHub {
    program_counting(s, seen, skew, &Arc::default())
}

/// As `program`, counting in `runs` the jobs that run.
fn program_counting<S: Sink>(
    s: &mut Scheduler<S>,
    seen: &Seen,
    skew: u64,
    runs: &Arc<AtomicUsize>,
) -> IoHub {
    let hub = IoHub::new();
    for t in 0..TASKS {
        let (h, w, runs) = (hub.clone(), seen.clone(), runs.clone());
        s.spawn(
            Spawn::new(format!("t{t}")).effect(Effect::pure()),
            move |cx| async move {
                let mut x = t;
                for j in 0..JOBS {
                    let extra = if t == 3 && j == 2 { skew } else { 0 };
                    let runs = runs.clone();
                    x = h
                        .io(&cx)
                        .job(format!("mix t{t} j{j}"), move || {
                            runs.fetch_add(1, Ordering::SeqCst);
                            mix(x, ROUNDS).wrapping_add(extra)
                        })
                        .await;
                    w.borrow_mut().push(format!("t{t} j{j}: {x:x}"));
                }
            },
        );
    }
    let h = hub.clone();
    s.spawn(
        Spawn::new("clock").effect(Effect::pure()),
        move |cx| async move {
            for _ in 0..5 {
                h.io(&cx).perform(Op::WallClock).await;
            }
        },
    );
    hub
}

/// What each task's jobs give, computed here.
fn expected() -> Vec<String> {
    let mut out = Vec::new();
    for t in 0..TASKS {
        let mut x = t;
        for j in 0..JOBS {
            x = mix(x, ROUNDS);
            out.push(format!("t{t} j{j}: {x:x}"));
        }
    }
    out.sort();
    out
}

fn sorted(mut v: Vec<String>) -> Vec<String> {
    v.sort();
    v
}

fn config(seed: Option<u64>) -> Config {
    match seed {
        Some(seed) => Config {
            seed,
            stress: Some(Stress::heavy()),
            ..Config::default()
        },
        None => Config::default(),
    }
}

/// A sim run: what the program saw, and the event stream.
fn sim(seed: Option<u64>, runner: Sim) -> (Vec<String>, Memory) {
    let seen = Seen::default();
    let mut s = Scheduler::new(config(seed), Memory::default());
    let hub = program(&mut s, &seen, 0);
    let mut runner = runner;
    assert_eq!(
        drive(&mut s, &hub, &mut runner, 1_000_000),
        Outcome::Returned
    );
    (seen.take(), s.sink().clone())
}

#[test]
fn jobs_give_their_values_in_the_sim() {
    let (seen, _) = sim(None, Sim::new(Vec::new()));
    assert_eq!(sorted(seen), expected());
}

/// However the threads go, a seed is one run: the same stream, event for
/// event. With stress, and with jobs delivered at once or late.
#[test]
fn a_sim_with_jobs_is_one_run() {
    for seed in [None, Some(1), Some(2), Some(3)] {
        for steps in [0, 3, 100] {
            let a = sim(seed, Sim::new(Vec::new()).job_steps(steps));
            let b = sim(seed, Sim::new(Vec::new()).job_steps(steps));
            assert_eq!(a.1, b.1, "seed {seed:?}, within {steps} steps");
            assert_eq!(a.0, b.0);
            assert_eq!(sorted(a.0), expected());
        }
    }
}

/// Jobs run at once, on as many threads as wait: two jobs, each waiting
/// (up to ten seconds, on a loaded machine) for the other to start.
#[test]
fn jobs_run_in_parallel() {
    if std::thread::available_parallelism().map_or(1, |n| n.get()) < 2 {
        return;
    }
    let running = Arc::new(AtomicUsize::new(0));
    let started = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));
    let mut s = Scheduler::new(Config::default(), Memory::default());
    let hub = IoHub::new();
    for t in 0..2u64 {
        let (h, peak) = (hub.clone(), peak.clone());
        let (running, started) = (running.clone(), started.clone());
        s.spawn(
            Spawn::new(format!("t{t}")).effect(Effect::pure()),
            move |cx| async move {
                h.io(&cx)
                    .job(format!("wait t{t}"), move || {
                        started.fetch_add(1, Ordering::SeqCst);
                        let n = running.fetch_add(1, Ordering::SeqCst) + 1;
                        peak.fetch_max(n, Ordering::SeqCst);
                        let start = Instant::now();
                        while started.load(Ordering::SeqCst) < 2
                            && start.elapsed() < Duration::from_secs(10)
                        {
                            std::thread::yield_now();
                        }
                        peak.fetch_max(running.load(Ordering::SeqCst), Ordering::SeqCst);
                        running.fetch_sub(1, Ordering::SeqCst);
                        t
                    })
                    .await;
            },
        );
    }
    let mut runner = Sim::new(Vec::new());
    assert_eq!(drive(&mut s, &hub, &mut runner, 1_000), Outcome::Returned);
    assert_eq!(peak.load(Ordering::SeqCst), 2);
}

/// A job that panics is answered with what it said; its slot says so.
#[test]
fn a_panicking_job_is_answered_with_its_panic() {
    let seen = Seen::default();
    let mut s = Scheduler::new(Config::default(), Memory::default());
    let hub = IoHub::new();
    let (h, w) = (hub.clone(), seen.clone());
    s.spawn(
        Spawn::new("t").effect(Effect::pure()),
        move |cx| async move {
            let (job, slot) = Job::new("boom", || -> u64 { panic!("boom") });
            let a = h.io(&cx).perform(Op::Job(job)).await;
            let taken = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| slot.take()));
            w.borrow_mut()
                .push(format!("{a:?}; taken: {}", taken.is_err()));
        },
    );
    let mut runner = Sim::new(Vec::new());
    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let outcome = drive(&mut s, &hub, &mut runner, 1_000);
    std::panic::set_hook(prev);
    assert_eq!(outcome, Outcome::Returned);
    assert_eq!(
        seen.take(),
        ["Err(\"the job panicked: boom\"); taken: true"]
    );
}

/// A run recorded, and its replay with the program `skew`ed.
fn record_then_replay(runner: &mut dyn Runner, skew: u64) -> (Vec<String>, Outcome) {
    let seen = Seen::default();
    let log = Rc::new(RefCell::new(header(&[])));
    let mut s = Scheduler::new(
        Config::default(),
        Recorder::new(Memory::default(), log.clone(), true, 0),
    );
    let hub = program(&mut s, &seen, 0);
    assert_eq!(drive(&mut s, &hub, runner, 1_000_000), Outcome::Returned);
    let recorded = s.sink().inner.clone();
    let first = seen.take();
    assert_eq!(sorted(first.clone()), expected());

    let l = read(&log.borrow()).expect("a log");
    let (mut replay, expect) = Replay::new(&l);
    let mut s = Scheduler::new(Config::default(), Checker::new(Memory::default(), expect));
    let hub = program(&mut s, &seen, skew);
    let outcome = drive(&mut s, &hub, &mut replay, 1_000_000);
    if outcome == Outcome::Returned {
        assert_eq!(s.sink().inner, recorded);
        assert_eq!(seen.take(), first);
    }
    (first, outcome)
}

#[test]
fn a_sim_with_jobs_replays() {
    let (_, outcome) = record_then_replay(&mut Sim::new(Vec::new()).job_steps(5), 0);
    assert_eq!(outcome, Outcome::Returned);
}

/// Live, jobs come back when they are done, at whatever step: the replay
/// delivers each at the step it came, so it is the same run.
#[test]
fn a_live_run_with_jobs_replays() {
    let (_, outcome) = record_then_replay(&mut Live::without_input().unwrap(), 0);
    assert_eq!(outcome, Outcome::Returned);
}

/// A job that gives another value on replay is found, at its arrival.
#[test]
fn a_job_answering_otherwise_diverges() {
    let (_, outcome) = record_then_replay(&mut Sim::new(Vec::new()), 1);
    match outcome {
        Outcome::Killed(Kill::Diverged(d)) => {
            assert!(d.expected.contains("job mix t3 j2"), "{d:?}");
        }
        o => panic!("{o:?}"),
    }
}

// ---------------------------------------------------------- job values

/// A run recorded with its job values kept; then replayed, the program
/// `skew`ed, using the values as `how` says. What the replay gave, how
/// many jobs it ran, and the values kept.
fn replay_with_values(
    runner: &mut dyn Runner,
    values: Rc<RefCell<Values>>,
    skew: u64,
    how: UseValues,
) -> (Outcome, Vec<String>, usize, Replay) {
    let seen = Seen::default();
    let log = Rc::new(RefCell::new(header(&[])));
    let mut s = Scheduler::new(
        Config::default(),
        Recorder::new(Memory::default(), log.clone(), true, 0),
    );
    let hub = program(&mut s, &seen, 0);
    assert_eq!(drive(&mut s, &hub, runner, 1_000_000), Outcome::Returned);
    let recorded = s.sink().inner.clone();
    let first = seen.take();

    let kept = values.borrow().clone();
    let l = read(&log.borrow()).expect("a log");
    let (replay, expect) = Replay::new(&l);
    let mut replay = replay.with_values(kept, how);
    let runs = Arc::new(AtomicUsize::new(0));
    let mut s = Scheduler::new(Config::default(), Checker::new(Memory::default(), expect));
    let hub = program_counting(&mut s, &seen, skew, &runs);
    let outcome = drive(&mut s, &hub, &mut replay, 1_000_000);
    if outcome == Outcome::Returned {
        assert_eq!(s.sink().inner, recorded, "the same stream");
        assert_eq!(seen.take(), first, "the same values");
    }
    let warnings = replay.warnings.clone();
    (outcome, warnings, runs.load(Ordering::SeqCst), replay)
}

/// Kept as the jobs are delivered: one value for each distinct value.
#[test]
fn a_run_keeps_its_job_values() {
    let values = Rc::new(RefCell::new(Values::default()));
    let mut sim = Sim::new(Vec::new()).record_values(values.clone());
    let (_, outcome) = record_then_replay(&mut sim, 0);
    assert_eq!(outcome, Outcome::Returned);
    assert_eq!(values.borrow().len(), (TASKS * JOBS) as usize);
    let file = values.borrow().encode();
    assert_eq!(Values::read(&file).as_ref(), Ok(&*values.borrow()));
}

/// A job answering otherwise, its value kept: the replay warns and goes
/// on with the recorded value, to the recorded run's end.
#[test]
fn a_replay_goes_on_with_a_recorded_value() {
    let values = Rc::new(RefCell::new(Values::default()));
    let mut sim = Sim::new(Vec::new()).record_values(values.clone());
    let (outcome, warnings, runs, _) = replay_with_values(&mut sim, values, 1, UseValues::Fallback);
    assert_eq!(outcome, Outcome::Returned);
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    assert!(warnings[0].contains("job mix t3 j2"), "{warnings:?}");
    assert_eq!(runs, (TASKS * JOBS) as usize, "every job runs again");
}

/// Strict, the values are no help: a divergence, as with none.
#[test]
fn a_strict_replay_diverges_at_a_job_answering_otherwise() {
    let values = Rc::new(RefCell::new(Values::default()));
    let mut sim = Sim::new(Vec::new()).record_values(values.clone());
    let (outcome, ..) = replay_with_values(&mut sim, values, 1, UseValues::Strict);
    assert!(
        matches!(outcome, Outcome::Killed(Kill::Diverged(_))),
        "{outcome:?}"
    );
}

/// Trusting the values, no job runs, and the replay is the recorded run,
/// though the program's jobs now answer otherwise; from a live run too.
#[test]
fn a_replay_trusting_the_values_runs_no_job() {
    for live in [false, true] {
        let values = Rc::new(RefCell::new(Values::default()));
        let (outcome, warnings, runs, _) = if live {
            let mut l = Live::without_input().unwrap();
            l.record_values(values.clone());
            replay_with_values(&mut l, values, 1, UseValues::Trust)
        } else {
            let mut sim = Sim::new(Vec::new()).record_values(values.clone());
            replay_with_values(&mut sim, values, 1, UseValues::Trust)
        };
        assert_eq!(outcome, Outcome::Returned, "live {live}");
        assert!(warnings.is_empty());
        assert_eq!(runs, 0, "live {live}");
    }
}
