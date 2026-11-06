// coopt-source: crates/coopt-sched/tests/sched.rs@9283bba
use std::cell::RefCell;
use std::rc::Rc;

use kayak_sched::{Config, Effect, Priority, Scheduler, Spawn, Step, Stress};
use kayak_trace::{Memory, SliceEnd, What};

type Log = Rc<RefCell<Vec<String>>>;

fn log() -> Log {
    Rc::new(RefCell::new(Vec::new()))
}

fn note(log: &Log, s: impl Into<String>) {
    log.borrow_mut().push(s.into());
}

fn stressed(seed: u64) -> Config {
    Config {
        seed,
        quantum: 10_000,
        stress: Some(Stress::heavy()),
        ..Config::default()
    }
}

/// A mixed workload: tasks that spend fuel, yield, sleep, wait for
/// completions, and spawn. Each task's own log is deterministic whatever the
/// schedule; the interleaving of the shared one is not.
fn workload(config: Config) -> (Memory, Vec<String>, Vec<u64>) {
    let mut s = Scheduler::new(config, Memory::default());
    let shared = log();
    let totals = Rc::new(RefCell::new(vec![0u64; 4]));
    for i in 0..4u32 {
        let (shared, totals) = (shared.clone(), totals.clone());
        let effect = Effect::pure().write(i % 2);
        s.spawn(
            Spawn::new(format!("worker{i}")).effect(effect),
            move |cx| async move {
                for k in 0..20u64 {
                    cx.charge(3).await;
                    totals.borrow_mut()[i as usize] += k;
                    if k % 7 == 0 {
                        cx.yield_now().await;
                    }
                    if k % 9 == 0 {
                        cx.sleep(5 + u64::from(i)).await;
                    }
                }
                note(&shared, format!("worker{i} done"));
            },
        );
    }
    let inbox = s.inbox();
    {
        let shared = shared.clone();
        s.spawn(
            Spawn::new("input")
                .priority(Priority::Input)
                .effect(Effect::pure()),
            move |cx| async move {
                let a = cx.completion(1).await;
                let b = cx.completion(2).await;
                note(&shared, format!("input {} {}", a.len(), b.len()));
                cx.spawn(
                    Spawn::new("child").effect(Effect::pure().read(0)),
                    move |cx| async move {
                        cx.charge(50).await;
                        note(&shared, "child done");
                    },
                );
            },
        );
    }
    inbox.post(2, vec![0; 2]);
    inbox.post(1, vec![0; 1]);
    let ran = s.run(100_000);
    assert!(ran < 100_000, "the workload ends");
    assert_eq!(s.live_tasks(), 0);
    let totals = totals.borrow().clone();
    let mut done = shared.borrow().clone();
    done.sort();
    (s.into_sink(), done, totals)
}

#[test]
fn the_same_seed_gives_the_same_stream() {
    for seed in [0, 1, 17, 0xdead_beef] {
        let (a, _, _) = workload(stressed(seed));
        let (b, _, _) = workload(stressed(seed));
        assert_eq!(a.text(), b.text(), "seed {seed}");
    }
    let (a, _, _) = workload(Config::default());
    let (b, _, _) = workload(Config::default());
    assert_eq!(a, b);
}

#[test]
fn different_seeds_explore_different_schedules_with_the_same_results() {
    let (reference, done, totals) = workload(Config::default());
    let mut streams = std::collections::BTreeSet::new();
    streams.insert(reference.text());
    for seed in 0..40 {
        let (stream, d, t) = workload(stressed(seed));
        assert_eq!(d, done, "seed {seed}");
        assert_eq!(t, totals, "seed {seed}");
        streams.insert(stream.text());
    }
    assert!(
        streams.len() > 30,
        "only {} distinct schedules",
        streams.len()
    );
}

#[test]
fn fuel_preempts_and_the_charge_is_paid_on_resumption() {
    let config = Config {
        quantum: 10,
        ..Config::default()
    };
    let mut s = Scheduler::new(config, Memory::default());
    s.spawn(Spawn::new("spender"), |cx| async move {
        for _ in 0..100 {
            cx.charge(1).await;
        }
    });
    s.run(1000);
    let ends: Vec<SliceEnd> = s
        .sink()
        .events
        .iter()
        .filter_map(|e| match e.what {
            What::SliceEnd { end, used, .. } => {
                assert!(used <= 10);
                Some(end)
            }
            _ => None,
        })
        .collect();
    // 100 units at 10 a slice. The charge that finds a slice's fuel gone
    // is preempted, and paid first in the next slice; so ten slices, the
    // last of which spends its final unit and returns.
    assert_eq!(ends.len(), 10, "{ends:?}");
    assert!(ends[..9].iter().all(|&e| e == SliceEnd::Preempted));
    assert_eq!(ends[9], SliceEnd::Finished);
}

#[test]
fn a_charge_larger_than_a_slice_spends_exactly_what_it_says() {
    let config = Config {
        quantum: 7,
        ..Config::default()
    };
    let mut s = Scheduler::new(config, Memory::default());
    s.spawn(Spawn::new("big"), |cx| async move {
        cx.charge(100).await;
    });
    s.run(1000);
    let used: Vec<u64> = s
        .sink()
        .events
        .iter()
        .filter_map(|e| match e.what {
            What::SliceEnd { used, .. } => Some(used),
            _ => None,
        })
        .collect();
    assert_eq!(used.iter().sum::<u64>(), 100);
    assert_eq!(used.len(), 15, "{used:?}");
}

#[test]
fn input_is_served_at_the_next_step_however_busy_the_rest_is() {
    let mut s = Scheduler::new(
        Config {
            quantum: 5,
            ..Config::default()
        },
        Memory::default(),
    );
    for i in 0..3 {
        s.spawn(
            Spawn::new(format!("spin{i}")).effect(Effect::pure()),
            |cx| async move {
                loop {
                    cx.charge(1).await;
                }
            },
        );
    }
    let served = log();
    let input = {
        let served = served.clone();
        s.spawn(
            Spawn::new("input")
                .priority(Priority::Input)
                .effect(Effect::pure()),
            move |cx| async move {
                for token in 0..=u64::MAX {
                    cx.completion(token).await;
                    note(&served, format!("key {token}"));
                }
            },
        )
    };
    let inbox = s.inbox();
    for token in 0..10 {
        s.run(37);
        inbox.post(token, vec![]);
        assert_eq!(s.step(), Step::Ran(input), "key {token} is handled at once");
        assert_eq!(served.borrow().len() as u64, token + 1);
    }
}

/// Each task logs when it enters and leaves a section between suspension
/// points, spending fuel inside it; `regions` gives each task's written
/// region. Under stress, by `seed`.
fn sections(seed: u64, regions: [u32; 3]) -> Vec<String> {
    sections_in(
        Config {
            seed,
            quantum: 0,
            stress: Some(Stress {
                max_quantum: 4,
                ignore_priority: (1, 2),
                max_delivery_delay: 0,
            }),
            ..Config::default()
        },
        regions,
    )
}

fn sections_in(config: Config, regions: [u32; 3]) -> Vec<String> {
    let mut s = Scheduler::new(config, Memory::default());
    let trail = log();
    for (i, r) in regions.into_iter().enumerate() {
        let trail = trail.clone();
        s.spawn(
            Spawn::new(format!("t{i}")).effect(Effect::pure().write(r)),
            move |cx| async move {
                for _ in 0..5 {
                    note(&trail, format!("enter {i}"));
                    for _ in 0..6 {
                        cx.charge(1).await;
                    }
                    note(&trail, format!("leave {i}"));
                    cx.yield_now().await;
                }
            },
        );
    }
    s.run(10_000);
    assert!(s.open_sections_do_not_interfere());
    trail.take()
}

/// Whether some section was entered while another was open.
fn interleaved(trail: &[String]) -> bool {
    let mut open = 0;
    for entry in trail {
        if entry.starts_with("enter") {
            open += 1;
            if open > 1 {
                return true;
            }
        } else {
            open -= 1;
        }
    }
    false
}

#[test]
fn interfering_sections_never_interleave() {
    for seed in 0..100 {
        let trail = sections(seed, [7, 7, 7]);
        assert!(!interleaved(&trail), "seed {seed}: {trail:?}");
    }
}

/// Without stress too, where the scheduler chooses in one pass: slices of
/// 4 fuel preempt each section, and a task that interferes waits.
#[test]
fn without_stress_interfering_sections_never_interleave() {
    let config = Config {
        quantum: 4,
        ..Config::default()
    };
    let trail = sections_in(config.clone(), [7, 7, 7]);
    assert!(!interleaved(&trail), "{trail:?}");
    assert!(interleaved(&sections_in(config, [1, 2, 3])));
}

#[test]
fn sections_that_do_not_interfere_are_preempted_freely() {
    let some = (0..100).any(|seed| interleaved(&sections(seed, [1, 2, 3])));
    assert!(some, "fuel never interleaved independent sections");
}

#[test]
fn the_clock_jumps_to_each_timer_in_turn() {
    let mut s = Scheduler::new(Config::default(), Memory::default());
    let woke = log();
    for d in [30, 10, 20] {
        let woke = woke.clone();
        s.spawn(Spawn::new(format!("sleep{d}")), move |cx| async move {
            cx.sleep(d).await;
            note(&woke, format!("{d} at {}", cx.now()));
        });
    }
    s.run(100);
    assert_eq!(*woke.borrow(), ["10 at 10", "20 at 20", "30 at 30"]);
    assert_eq!(s.now(), 30);
    let clocks: Vec<u64> = s
        .sink()
        .events
        .iter()
        .filter_map(|e| match e.what {
            What::Clock { now } => Some(now),
            _ => None,
        })
        .collect();
    assert_eq!(clocks, [10, 20, 30]);
}

#[test]
fn a_completion_delivered_before_it_is_awaited_is_kept() {
    let mut s = Scheduler::new(Config::default(), Memory::default());
    let got = log();
    s.inbox().post(5, b"hello".to_vec());
    {
        let got = got.clone();
        s.spawn(Spawn::new("late"), move |cx| async move {
            cx.sleep(100).await;
            let bytes = cx.completion(5).await;
            note(&got, String::from_utf8(bytes).unwrap());
        });
    }
    s.run(100);
    assert_eq!(*got.borrow(), ["hello"]);
}

#[test]
fn a_task_waiting_for_the_outside_leaves_the_scheduler_idle() {
    let mut s = Scheduler::new(Config::default(), Memory::default());
    s.spawn(Spawn::new("waiter"), |cx| async move {
        cx.completion(9).await;
    });
    assert!(matches!(s.step(), Step::Ran(_)));
    assert_eq!(s.step(), Step::Idle);
    assert_eq!(s.live_tasks(), 1);
    s.inbox().post(9, vec![]);
    assert!(matches!(s.step(), Step::Ran(_)));
    assert_eq!(s.live_tasks(), 0);
}

/// Under stress, deliveries are held back; when nothing can run, they
/// are released, until one wakes a task: answers no one waits for (a
/// write's) do not leave the scheduler idle with the awaited one held.
#[test]
fn the_scheduler_is_never_idle_while_an_awaited_answer_is_held() {
    for seed in 0..32 {
        let mut s = Scheduler::new(stressed(seed), Memory::default());
        let got = log();
        {
            let got = got.clone();
            s.spawn(Spawn::new("reader"), move |cx| async move {
                cx.completion(9).await;
                note(&got, "read");
            });
        }
        assert!(matches!(s.step(), Step::Ran(_)));
        for token in 1..=9 {
            s.inbox().post(token, vec![]);
        }
        while let Step::Ran(_) = s.step() {}
        assert_eq!(*got.borrow(), ["read"], "seed {seed}");
    }
}

#[test]
fn effects_interfere_as_the_par_table_says() {
    let r = |n| Effect::pure().read(n);
    let w = |n| Effect::pure().write(n);
    let c = |n| Effect::pure().control(n);
    let cases = [
        (r(1), r(1), false),
        (r(1), w(1), true),
        (w(1), w(1), true),
        (w(1), w(2), false),
        (c(1), r(1), true),
        (c(1), r(2), false),
        (Effect::any(), r(9), true),
        (Effect::any(), Effect::pure(), false),
        (Effect::pure(), Effect::pure(), false),
    ];
    for (a, b, expected) in cases {
        assert_eq!(a.interferes(&b), expected, "{a:?} vs {b:?}");
        assert_eq!(b.interferes(&a), expected, "symmetric: {b:?} vs {a:?}");
    }
}

/// Work takes time: a sleeper's deadline comes while another task works,
/// and it runs then, not only once everything else is idle (their effects
/// do not interfere, so the worker's open section does not hold it back). A typist's
/// next key, in the simulated runner, arrives so while the compiler runs.
#[test]
fn a_timer_comes_due_while_others_work() {
    let mut s = Scheduler::new(
        Config {
            quantum: 10,
            ..Config::default()
        },
        Memory::default(),
    );
    let log = log();
    let l = log.clone();
    s.spawn(
        Spawn::new("worker")
            .priority(Priority::Background)
            .effect(Effect::pure()),
        move |cx| async move {
            for i in 0..20 {
                cx.charge(10).await;
                note(&l, format!("work {i} at {}", cx.now()));
            }
        },
    );
    let l = log.clone();
    s.spawn(
        Spawn::new("sleeper")
            .priority(Priority::Input)
            .effect(Effect::pure()),
        move |cx| async move {
            cx.sleep(55).await;
            note(&l, format!("woke at {}", cx.now()));
        },
    );
    s.run(1000);
    let trail = log.borrow().clone();
    let woke = trail
        .iter()
        .position(|t| t.starts_with("woke"))
        .expect("woke");
    assert!(
        woke < trail.len() - 1,
        "it waited for the worker: {trail:#?}"
    );
    assert_eq!(trail[woke], "woke at 60", "{trail:#?}");
}

/// A section left open slice after slice is a hog, reported at 8 slices
/// and at each doubling; one that suspends often is not.
#[test]
fn a_section_left_open_is_reported_as_a_hog() {
    let mut s = Scheduler::new(
        Config {
            quantum: 10,
            ..Config::default()
        },
        Memory::default(),
    );
    let hog = s.spawn(
        Spawn::new("hog").effect(Effect::pure().write(1)),
        move |cx| async move {
            for _ in 0..200 {
                cx.charge(1).await;
            }
        },
    );
    s.spawn(
        Spawn::new("polite").effect(Effect::pure().write(2)),
        move |cx| async move {
            for _ in 0..20 {
                cx.charge(5).await;
                cx.yield_now().await;
            }
        },
    );
    s.run(10_000);
    let hogs: Vec<(u64, u64)> = s
        .sink()
        .events
        .iter()
        .filter_map(|e| match e.what {
            What::Hog { task, slices } => Some((task, slices)),
            _ => None,
        })
        .collect();
    assert_eq!(hogs, [(hog.0, 8), (hog.0, 16)]);
}

/// A background task's open section blocks an urgent one with which it
/// interferes: the background task runs at the urgent one's priority until
/// its section closes, ahead of work of a middle priority.
#[test]
fn an_open_section_blocking_urgent_work_inherits_its_priority() {
    let mut s = Scheduler::new(
        Config {
            quantum: 10,
            ..Config::default()
        },
        Memory::default(),
    );
    let order = log();
    let o = order.clone();
    let bg = s.spawn(
        Spawn::new("background")
            .priority(Priority::Background)
            .effect(Effect::pure().write(1)),
        move |cx| async move {
            for _ in 0..3 {
                cx.charge(10).await;
            }
            note(&o, "background section closed");
            cx.yield_now().await;
        },
    );
    // The urgent task and the middle one arrive once the background
    // task's section is open.
    let o = order.clone();
    s.spawn(
        Spawn::new("starter").priority(Priority::Input),
        move |cx| async move {
            cx.sleep(15).await;
            let o2 = o.clone();
            cx.spawn(
                Spawn::new("middle")
                    .priority(Priority::CompileNeeded)
                    .effect(Effect::pure().write(2)),
                move |cx| async move {
                    for _ in 0..5 {
                        cx.charge(10).await;
                    }
                    note(&o2, "middle done");
                },
            );
            cx.spawn(
                Spawn::new("urgent")
                    .priority(Priority::UserWaiting)
                    .effect(Effect::pure().read(1)),
                move |_cx| async move {
                    note(&o, "urgent ran");
                },
            );
        },
    );
    s.run(10_000);
    assert_eq!(
        *order.borrow(),
        ["background section closed", "urgent ran", "middle done"]
    );
    assert!(s.sink().events.iter().any(|e| matches!(
        e.what,
        What::Inherit { task, priority: Some(_) } if task == bg.0
    )));
}

/// The first of several completions, or the clock (what an in-language
/// loop waits on when none of its fibers is ready): one delivered taken,
/// the others left for later; at the deadline, none; and the wait leaves
/// no registration behind, so no stale timer moves the clock.
#[test]
fn the_first_of_several_completions_or_the_clock() {
    let mut s = Scheduler::new(Config::default(), Memory::default());
    let got: Rc<RefCell<Vec<String>>> = Rc::default();
    let g = got.clone();
    s.spawn(
        Spawn::new("waiter").effect(Effect::pure()),
        move |cx| async move {
            let first = cx.first_completion(vec![1, 2], Some(1000)).await;
            g.borrow_mut().push(format!("{first:?} at {}", cx.now()));
            // Token 1's, delivered later, is there to take.
            let one = cx.completion(1).await;
            g.borrow_mut().push(format!("{one:?} at {}", cx.now()));
            let none = cx.first_completion(vec![3], Some(cx.now() + 50)).await;
            g.borrow_mut().push(format!("{none:?} at {}", cx.now()));
        },
    );
    let inbox = s.inbox();
    s.spawn(
        Spawn::new("poster").effect(Effect::pure()),
        move |cx| async move {
            cx.sleep(10).await;
            inbox.post(2, b"b".to_vec());
            cx.sleep(10).await;
            inbox.post(1, b"a".to_vec());
        },
    );
    s.run(1000);
    assert_eq!(
        *got.borrow(),
        ["Some((2, [98])) at 10", "[97] at 20", "None at 70"]
    );
    // The first wait's timer, at 1000, was taken back: nothing is left.
    assert_eq!(s.next_deadline(), None);
    assert_eq!(s.now(), 70);
}

/// Polls a future once, then drops it.
struct Once<F: std::future::Future>(Option<std::pin::Pin<Box<F>>>);

impl<F: std::future::Future> std::future::Future for Once<F> {
    type Output = ();
    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<()> {
        if let Some(f) = self.0.as_mut() {
            let _ = f.as_mut().poll(cx);
        }
        self.0 = None;
        std::task::Poll::Ready(())
    }
}

/// A sleep, or a wait for a completion or the clock, dropped before it is
/// done (as an interrupt drops a run's) takes back its timer and its
/// waiter: the clock does not jump to the deadline, and the completion
/// wakes no one.
#[test]
fn a_wait_dropped_takes_back_its_registrations() {
    let mut s = Scheduler::new(Config::default(), Memory::default());
    s.spawn(Spawn::new("waiter"), |cx| async move {
        Once(Some(Box::pin(cx.sleep(1000)))).await;
        Once(Some(Box::pin(cx.first_completion(vec![7], Some(2000))))).await;
        cx.completion(8).await;
    });
    while let Step::Ran(_) = s.step() {}
    assert_eq!(s.next_deadline(), None);
    assert_eq!(s.now(), 0);
    s.inbox().post(7, vec![]);
    assert_eq!(s.step(), Step::Idle);
    s.inbox().post(8, vec![]);
    assert!(matches!(s.step(), Step::Ran(_)));
    assert_eq!(s.live_tasks(), 0);
}

/// A completion abandoned is dropped: at once if it was delivered, or
/// when it is; others are kept.
#[test]
fn an_abandoned_completion_is_dropped() {
    let mut s = Scheduler::new(Config::default(), Memory::default());
    s.inbox().post(1, vec![1]);
    s.spawn(Spawn::new("quitter"), |cx| async move {
        cx.abandon(1);
        cx.abandon(2);
    });
    while let Step::Ran(_) = s.step() {}
    assert_eq!(s.completions_held(), 0);
    s.inbox().post(2, vec![2]);
    s.inbox().post(3, vec![3]);
    while let Step::Ran(_) = s.step() {}
    assert_eq!(s.completions_held(), 1);
}

/// A task's value, taken by its spawner once the scheduler has run.
#[test]
fn a_task_gives_its_value_to_its_handle() {
    let mut s = Scheduler::new(Config::default(), Memory::default());
    let h = s.spawn_join(Spawn::new("sum"), |cx| async move {
        let mut total = 0u64;
        for k in 1..=10 {
            cx.charge(1).await;
            total += k;
        }
        total
    });
    assert!(!h.is_finished());
    s.run(1000);
    assert!(h.is_finished());
    assert_eq!(h.take(), 55);
}

/// A task that has not returned is a failure that names it, not a value
/// half made.
#[test]
#[should_panic(expected = "the task \"waiter\" (t0) has not finished")]
fn taking_from_an_unfinished_task_names_it() {
    let mut s = Scheduler::new(Config::default(), Memory::default());
    let h = s.spawn_join(Spawn::new("waiter"), |cx| async move {
        cx.completion(1).await;
    });
    s.run(1000);
    // Gone first, so that it does not report a run that did not fail.
    drop(s);
    h.take();
}

/// One task awaits another's handle: whether the value is there before
/// it asks (`early`) or comes after (`late`), it gets it, under any
/// schedule; and the same seed gives the same stream.
#[test]
fn a_task_awaits_another_through_its_handle() {
    let run = |config: Config| {
        let mut s = Scheduler::new(config, Memory::default());
        let joiner = s.spawn_join(Spawn::new("joiner"), |cx| async move {
            let early = cx.spawn_join(Spawn::new("early"), |_| async { 1u64 });
            let late = cx.spawn_join(Spawn::new("late"), |cx| async move {
                cx.sleep(50).await;
                2u64
            });
            cx.sleep(10).await;
            assert!(early.is_finished() && !late.is_finished());
            early.await * 10 + late.await
        });
        s.run(10_000);
        (joiner.take(), s.into_sink().text())
    };
    assert_eq!(run(Config::default()).0, 12);
    for seed in 0..20 {
        let (v, stream) = run(stressed(seed));
        assert_eq!(v, 12, "seed {seed}");
        assert_eq!(run(stressed(seed)).1, stream, "seed {seed}");
    }
}

/// Dropping a handle cancels nothing: the task runs to its end.
#[test]
fn a_dropped_handle_leaves_its_task_running() {
    let mut s = Scheduler::new(Config::default(), Memory::default());
    let done = log();
    let d = done.clone();
    drop(s.spawn_join(Spawn::new("orphan"), move |cx| async move {
        cx.sleep(5).await;
        note(&d, "ran to its end");
        7
    }));
    s.run(1000);
    assert_eq!(*done.borrow(), ["ran to its end"]);
}

/// A program that exercises every choice the scheduler makes: classes,
/// interfering and independent sections, sleeps, spawns, and priorities
/// changed while tasks wait. Its stream's FNV-1a hash, by seed.
fn pinned_run(stress: Option<u64>) -> u64 {
    let config = match stress {
        Some(seed) => Config {
            seed,
            quantum: 0,
            stress: Some(Stress::heavy()),
            demote_after: 12,
            ..Config::default()
        },
        None => Config {
            quantum: 3,
            demote_after: 12,
            ..Config::default()
        },
    };
    let mut s = Scheduler::new(config, Memory::default());
    let classes = [
        Priority::Background,
        Priority::Typing,
        Priority::UserWaiting,
        Priority::Background,
        Priority::Idle,
        Priority::CompileNeeded,
    ];
    let mut ids = Vec::new();
    for (i, p) in classes.into_iter().enumerate() {
        let region = [1, 1, 2, 3, 2, 4][i];
        let mut spawn = Spawn::new(format!("t{i}"))
            .priority(p)
            .effect(Effect::pure().write(region));
        // Two that fall as they run long, and come back when they sleep.
        if i == 2 || i == 5 {
            spawn = spawn.demotable();
        }
        ids.push(s.spawn(spawn, move |cx| async move {
            for k in 0..4u64 {
                // Sections of 8 fuel or more: preempted twice at
                // least, so a section that inherits is queued again
                // while it has.
                for _ in 0..(i as u64 % 3 + 4) {
                    cx.charge(2).await;
                }
                if k == 1 && i % 2 == 0 {
                    cx.spawn(
                        Spawn::new(format!("t{i}'")).effect(Effect::pure().write(5)),
                        move |cx| async move {
                            cx.charge(5).await;
                            cx.sleep(7).await;
                            cx.charge(1).await;
                        },
                    );
                }
                if k == 2 {
                    cx.sleep(i as u64 * 3 + 1).await;
                }
                cx.yield_now().await;
            }
        }));
    }
    let (last, first) = (ids[5], ids[0]);
    s.spawn(
        Spawn::new("boss").priority(Priority::Input),
        move |cx| async move {
            cx.sleep(4).await;
            cx.set_priority(last, Priority::Idle);
            cx.set_priority(first, Priority::UserWaiting);
        },
    );
    s.run(100_000);
    let text = s.sink().text();
    assert!(text.contains("inherit"), "no section inherited");
    if stress.is_none() {
        assert!(
            text.contains("demote") && text.contains("promote"),
            "no demotion"
        );
    }
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in text.bytes() {
        h = (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// The scheduler's decisions do not change unless meant to: each run's
/// stream is pinned, with and without stress. A change that keeps every
/// decision (a faster queue, say) keeps these; one that does not must
/// say why, and pin the new values.
#[test]
fn the_schedulers_decisions_are_pinned() {
    let got: Vec<u64> = [None, Some(1), Some(2), Some(3)]
        .into_iter()
        .map(pinned_run)
        .collect();
    assert_eq!(got, PINNED, "{got:#x?}");
}

/// As at `72997b3`, before the open set, the slots, and the queues by
/// class, which each kept every decision; re-pinned when demotion came
/// (`TODO.md` §48), which changes decisions by design, and which the
/// program uses since; and again when a wait came to take the time it
/// waited off a task's fuel, rather than all of it.
const PINNED: [u64; 4] = [
    0xd181_14b4_d2b1_69e5,
    0x166e_9432_bdef_fbac,
    0x5ce7_1e11_d672_271f,
    0x33fe_4d7e_e5ad_cd9e,
];

// ------------------------------------------------------------ demotion

fn demoting(after: u64) -> Config {
    Config {
        quantum: 3,
        demote_after: after,
        ..Config::default()
    }
}

/// The ranks a run's tasks went through: "demote t0 priority 2", ….
fn ranks(s: &Scheduler<Memory>) -> Vec<String> {
    s.sink()
        .text()
        .lines()
        .filter(|l| l.contains("demote") || l.contains("promote"))
        .map(|l| l.split_whitespace().skip(1).collect::<Vec<_>>().join(" "))
        .collect()
}

/// A demotable task that runs long falls a class for each 10 fuel, down
/// to `Background`, and rises back as it waits, by the time it waits:
/// part way after a short sleep, to its own class after a long one. One
/// that is not demotable stays where it is.
#[test]
fn a_task_that_runs_long_falls_and_rises_as_it_waits() {
    let mut s = Scheduler::new(demoting(10), Memory::default());
    for (name, demotable) in [("long", true), ("steady", false)] {
        let mut spawn = Spawn::new(name)
            .priority(Priority::UserWaiting)
            .effect(Effect::pure());
        if demotable {
            spawn = spawn.demotable();
        }
        s.spawn(spawn, move |cx| async move {
            for _ in 0..60 {
                cx.charge(1).await;
            }
            // 60 counted, less 25: 35, three classes down.
            cx.sleep(25).await;
            cx.charge(1).await;
            cx.sleep(100).await;
            cx.charge(1).await;
        });
    }
    s.run(10_000);
    assert_eq!(
        ranks(&s),
        [
            "demote t0 priority 2",
            "demote t0 priority 3",
            "demote t0 priority 4",
            "demote t0 priority 5",
            "promote t0 priority 4",
            "promote t0 priority 1",
        ]
    );
}

/// A loop that waits on the world as it goes, but for next to nothing (a
/// print, answered at once): it waits a tick for each 5 fuel it spends,
/// so it still falls, as a loop that never waits does.
#[test]
fn a_loop_that_waits_briefly_as_it_goes_still_falls() {
    let mut s = Scheduler::new(demoting(10), Memory::default());
    s.spawn(
        Spawn::new("printing")
            .priority(Priority::UserWaiting)
            .effect(Effect::pure())
            .demotable(),
        move |cx| async move {
            for _ in 0..40 {
                cx.charge(5).await;
                cx.sleep(1).await;
            }
        },
    );
    s.run(10_000);
    assert_eq!(
        ranks(&s),
        [
            "demote t0 priority 2",
            "demote t0 priority 3",
            "demote t0 priority 4",
            "demote t0 priority 5",
        ]
    );
}

/// A task waking at `Typing` while a demotable `UserWaiting` task runs
/// long: with demotion, it runs before the long one ends; without, after.
#[test]
fn demotion_lets_quick_work_past_work_that_runs_long() {
    let order = |after: u64| {
        let mut s = Scheduler::new(demoting(after), Memory::default());
        let trail = log();
        let t = trail.clone();
        s.spawn(
            Spawn::new("entry")
                .priority(Priority::UserWaiting)
                .effect(Effect::pure())
                .demotable(),
            move |cx| async move {
                for _ in 0..300 {
                    cx.charge(1).await;
                }
                note(&t, "entry done");
            },
        );
        let t = trail.clone();
        s.spawn(
            Spawn::new("typing")
                .priority(Priority::Typing)
                .effect(Effect::pure()),
            move |cx| async move {
                cx.sleep(20).await;
                note(&t, "typing ran");
            },
        );
        s.run(10_000);
        trail.take()
    };
    assert_eq!(order(10), ["typing ran", "entry done"]);
    assert_eq!(order(0), ["entry done", "typing ran"]);
}

/// Inheritance still wins: a demoted task whose open section blocks an
/// urgent one that interferes runs at the urgent one's priority.
#[test]
fn a_demoted_section_blocking_urgent_work_inherits() {
    let mut s = Scheduler::new(demoting(10), Memory::default());
    let trail = log();
    let t = trail.clone();
    s.spawn(
        Spawn::new("long")
            .priority(Priority::UserWaiting)
            .effect(Effect::pure().write(1))
            .demotable(),
        move |cx| async move {
            // One section, never suspending: open across many slices.
            for _ in 0..100 {
                cx.charge(1).await;
            }
            note(&t, "long done");
        },
    );
    let t = trail.clone();
    s.spawn(
        Spawn::new("urgent")
            .priority(Priority::Input)
            .effect(Effect::pure().write(1)),
        move |cx| async move {
            cx.sleep(40).await;
            note(&t, "urgent ran");
        },
    );
    s.run(10_000);
    let text = s.sink().text();
    let demoted = text.find("demote t0 priority 5").expect("demoted");
    let inherited = text.find("inherit t0 priority 0").expect("inherited");
    assert!(demoted < inherited, "{text}");
    assert_eq!(trail.take(), ["long done", "urgent ran"]);
}
