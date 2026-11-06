//! A runner that keeps the clock (`ClockMode::Runner`), as the live
//! runner keeps wall time: the scheduler never jumps it; the runner moves
//! it, in `poll` while things run and in `idle` when nothing does; every
//! move is recorded, and replays.
// coopt-source: crates/coopt-runner/tests/clock.rs@9283bba

use std::cell::RefCell;
use std::rc::Rc;

use kayak_runner::log::{Recorder, header, read};
use kayak_runner::runner::{Checker, Wake};
use kayak_runner::{Answer, IoHub, Op, Outcome, Replay, Runner, STDOUT, drive};
use kayak_sched::{ClockMode, Config, Effect, Inbox, Scheduler, Spawn};
use kayak_trace::{Memory, Sink};

/// Time passes by `tick` at every step, as a wall clock would while the
/// program works; when nothing can run, it moves to the next timer.
struct Ticking {
    tick: u64,
    ready: Vec<(u64, Answer)>,
    out: Vec<u8>,
}

impl Runner for Ticking {
    fn submit(&mut self, token: u64, op: Op) {
        if let Op::Write { bytes, .. } = op {
            self.out.extend_from_slice(&bytes);
            self.ready.push((token, Answer::Count(bytes.len() as u32)));
        }
    }
    fn poll(&mut self, _next: u64, now: u64, _d: Option<u64>, inbox: &Inbox) -> Option<u64> {
        for (t, a) in self.ready.drain(..) {
            inbox.post(t, a.encode());
        }
        Some(now + self.tick)
    }
    fn idle(&mut self, _step: u64, now: u64, deadline: Option<u64>, _inbox: &Inbox) -> Wake {
        match deadline {
            Some(d) if d > now => Wake::Clock(d),
            Some(_) => Wake::Posted,
            None => Wake::Done,
        }
    }
}

fn config() -> Config {
    Config {
        clock: ClockMode::Runner,
        ..Config::default()
    }
}

/// A task that sleeps, then writes the time it woke at; another that
/// works meanwhile.
fn program<S: Sink>(s: &mut Scheduler<S>) -> IoHub {
    let hub = IoHub::new();
    let h = hub.clone();
    s.spawn(
        Spawn::new("sleeper").effect(Effect::pure()),
        move |cx| async move {
            cx.sleep(100).await;
            let now = cx.now();
            h.io(&cx)
                .perform(Op::Write {
                    fd: STDOUT,
                    bytes: format!("woke at {now}\n").into_bytes(),
                })
                .await;
        },
    );
    s.spawn(
        Spawn::new("worker").effect(Effect::pure()),
        move |cx| async move {
            for _ in 0..30 {
                cx.yield_now().await;
            }
        },
    );
    hub
}

#[test]
fn the_runners_clock_wakes_sleepers_and_replays() {
    let log = Rc::new(RefCell::new(header(&[("clock", "runner")])));
    let mut s = Scheduler::new(
        config(),
        Recorder::new(Memory::default(), log.clone(), true, 0),
    );
    let hub = program(&mut s);
    let mut r = Ticking {
        tick: 3,
        ready: Vec::new(),
        out: Vec::new(),
    };
    assert_eq!(drive(&mut s, &hub, &mut r, 100_000), Outcome::Returned);
    let text = String::from_utf8(r.out.clone()).unwrap();
    // It woke when the clock, moved only by the runner, passed 100.
    let woke: u64 = text.trim().trim_start_matches("woke at ").parse().unwrap();
    assert!((100..110).contains(&woke), "{text}");
    let recorded = s.sink().inner.clone();

    let l = read(&log.borrow()).expect("a log");
    let (mut replay, expect) = Replay::new(&l);
    let mut s = Scheduler::new(config(), Checker::new(Memory::default(), expect));
    let hub = program(&mut s);
    assert_eq!(drive(&mut s, &hub, &mut replay, 100_000), Outcome::Returned);
    assert_eq!(s.sink().inner, recorded);
}

/// Under the runner's clock, the scheduler does not jump to a timer: with
/// a runner that never moves the clock, a sleeper never wakes.
#[test]
fn the_scheduler_does_not_jump_the_runners_clock() {
    let mut s = Scheduler::new(config(), Memory::default());
    let woke = Rc::new(RefCell::new(false));
    let w = woke.clone();
    s.spawn(Spawn::new("sleeper"), move |cx| async move {
        cx.sleep(10).await;
        *w.borrow_mut() = true;
    });
    s.run(1000);
    assert!(!*woke.borrow());
    s.advance_to(10);
    s.run(1000);
    assert!(*woke.borrow());
}
