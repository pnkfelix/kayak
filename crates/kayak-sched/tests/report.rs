//! A test that fails while a scheduler runs says the seed, the stress,
//! the step and the stream's hash so far (R3 §2.1): what it takes to see
//! the same run again. The failing test is run as a child of this one,
//! as its output is the thing checked.
// coopt-source: crates/coopt-sched/tests/report.rs@9283bba

use kayak_sched::{Config, Effect, Scheduler, Spawn, StreamHash, Stress};
use kayak_trace::Memory;

const SEED: u64 = 0x2a;

/// Some tasks, run under stress for `steps` steps.
fn run(steps: u64) -> Scheduler<Memory> {
    let config = Config {
        seed: SEED,
        stress: Some(Stress::heavy()),
        ..Config::default()
    };
    let mut s = Scheduler::new(config, Memory::default());
    for i in 0..3u64 {
        s.spawn(
            Spawn::new(format!("t{i}")).effect(Effect::pure()),
            move |cx| async move {
                for _ in 0..50 {
                    cx.charge(7).await;
                    cx.sleep(1 + i).await;
                }
            },
        );
    }
    s.run(steps);
    s
}

/// Fails on purpose, 40 steps into the run; run only by the test below.
#[test]
#[ignore = "fails on purpose: run by a_failure_says_its_seed_and_stream_hash"]
fn a_seeded_run_that_fails() {
    let _s = run(40);
    panic!("a planted failure");
}

#[test]
fn a_failure_says_its_seed_and_stream_hash() {
    let out = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "a_seeded_run_that_fails",
            "--nocapture",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    // The run again, here: the same stream, whose hash is the one said.
    let mut want_hash = StreamHash::default();
    for e in &run(40).sink().events {
        want_hash.update(e);
    }
    let StreamHash { events, hash } = want_hash;
    assert!(events > 40, "{events} events");
    let want = format!("seed {SEED:#x} (stress: {:?}", Stress::heavy());
    let said = format!(
        "stream hash {:016x}{:016x} after {events} events",
        hash.0, hash.1
    );
    assert!(err.contains("a planted failure"), "{err}");
    assert!(err.contains(&want), "{want} in {err}");
    assert!(err.contains(&said), "{said} in {err}");
}

/// No failure, nothing said.
#[test]
fn a_run_that_passes_says_nothing() {
    let out = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "a_seeded_run_that_passes", "--nocapture"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(!String::from_utf8_lossy(&out.stderr).contains("stream hash"));
}

#[test]
fn a_seeded_run_that_passes() {
    let _s = run(40);
}
