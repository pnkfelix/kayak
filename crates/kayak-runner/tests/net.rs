//! Localhost in the sim (M14): listeners, connections, the end of a
//! stream; faults on them by seed; and a run with faults replayed.
// coopt-source: crates/coopt-runner/tests/net.rs@9283bba

use std::cell::RefCell;
use std::rc::Rc;

use kayak_runner::live::Live;
use kayak_runner::log::{Recorder, header, read};
use kayak_runner::runner::Checker;
use kayak_runner::{Answer, Faults, Fd, IoHub, Op, Outcome, Replay, Sim, drive};
use kayak_sched::{Config, Effect, Scheduler, Spawn};
use kayak_trace::{Memory, Sink};

type Seen = Rc<RefCell<Vec<String>>>;

const PORT: u16 = 7000;

fn fd(a: &Answer) -> Option<Fd> {
    match a {
        Answer::Count(n) => Some(Fd(*n)),
        _ => None,
    }
}

/// A server, which accepts one connection and echoes what it reads, in
/// capitals, until the end; and a client, which writes three lines, reads
/// until it has all three back, and closes. Each stops at its first error.
fn program<S: Sink>(s: &mut Scheduler<S>, seen: &Seen, port: u16) -> IoHub {
    let hub = IoHub::new();
    let (h, w) = (hub.clone(), seen.clone());
    s.spawn(
        Spawn::new("server").effect(Effect::pure()),
        move |cx| async move {
            let io = h.io(&cx);
            let say = |m: String| w.borrow_mut().push(format!("server: {m}"));
            let Some(l) = fd(&io.perform(Op::Listen { port }).await) else {
                return say("no listener".into());
            };
            let a = io.perform(Op::Accept { fd: l }).await;
            let Some(c) = fd(&a) else {
                return say(format!("accept: {a:?}"));
            };
            loop {
                match io.perform(Op::Read { fd: c, max: 64 }).await {
                    Answer::Bytes(b) if b.is_empty() => {
                        say("the end".into());
                        break;
                    }
                    Answer::Bytes(b) => {
                        let up = b.to_ascii_uppercase();
                        let a = io.perform(Op::Write { fd: c, bytes: up }).await;
                        if fd(&a).is_none() {
                            say(format!("write: {a:?}"));
                            break;
                        }
                    }
                    a => {
                        say(format!("read: {a:?}"));
                        break;
                    }
                }
            }
            io.perform(Op::Close { fd: c }).await;
            io.perform(Op::Close { fd: l }).await;
        },
    );
    let (h, w) = (hub.clone(), seen.clone());
    s.spawn(
        Spawn::new("client").effect(Effect::pure()),
        move |cx| async move {
            let io = h.io(&cx);
            let say = |m: String| w.borrow_mut().push(format!("client: {m}"));
            let a = io.perform(Op::Connect { port }).await;
            let Some(c) = fd(&a) else {
                return say(format!("connect: {a:?}"));
            };
            let sent = "one\ntwo\nthree\n";
            for line in sent.split_inclusive('\n') {
                let bytes = line.as_bytes().to_vec();
                let a = io.perform(Op::Write { fd: c, bytes }).await;
                if fd(&a).is_none() {
                    say(format!("write: {a:?}"));
                    break;
                }
            }
            let mut back = Vec::new();
            while back.len() < sent.len() {
                match io.perform(Op::Read { fd: c, max: 5 }).await {
                    Answer::Bytes(b) if b.is_empty() => break,
                    Answer::Bytes(b) => back.extend(b),
                    a => {
                        say(format!("read: {a:?}"));
                        break;
                    }
                }
            }
            say(format!("back {:?}", String::from_utf8_lossy(&back)));
            io.perform(Op::Close { fd: c }).await;
        },
    );
    hub
}

fn run(faults: Option<Faults>) -> (Vec<String>, Vec<String>) {
    let seen = Seen::default();
    let mut s = Scheduler::new(Config::default(), Memory::default());
    let hub = program(&mut s, &seen, PORT);
    let mut sim = Sim::new(Vec::new());
    if let Some(f) = faults {
        sim = sim.faults(f);
    }
    assert_eq!(drive(&mut s, &hub, &mut sim, 1_000_000), Outcome::Returned);
    (seen.take(), sim.injected)
}

#[test]
fn an_echo_over_localhost() {
    let (seen, injected) = run(None);
    assert_eq!(
        seen,
        ["client: back \"ONE\\nTWO\\nTHREE\\n\"", "server: the end"],
    );
    assert!(injected.is_empty());
}

/// A port no one listens at refuses; a port in use is refused; port 0 is
/// a free one, which a connection then reaches; a closed fd is not open.
#[test]
fn ports_refusals_and_closes() {
    let seen = Seen::default();
    let mut s = Scheduler::new(Config::default(), Memory::default());
    let hub = IoHub::new();
    let (h, w) = (hub.clone(), seen.clone());
    s.spawn(
        Spawn::new("t").effect(Effect::pure()),
        move |cx| async move {
            let io = h.io(&cx);
            let say = |a: Answer| w.borrow_mut().push(format!("{a:?}"));
            say(io.perform(Op::Connect { port: 80 }).await);
            let l = io.perform(Op::Listen { port: 0 }).await;
            let lfd = fd(&l).unwrap();
            say(l);
            let port = match io.perform(Op::Port { fd: lfd }).await {
                Answer::Count(p) => p as u16,
                a => panic!("{a:?}"),
            };
            say(Answer::Count(u32::from(port)));
            say(io.perform(Op::Listen { port }).await);
            let c = io.perform(Op::Connect { port }).await;
            let cfd = fd(&c).unwrap();
            let sfd = fd(&io.perform(Op::Accept { fd: lfd }).await).unwrap();
            say(io.perform(Op::Close { fd: cfd }).await);
            // The server's end reads the end; the client's is gone.
            say(io.perform(Op::Read { fd: sfd, max: 9 }).await);
            say(io
                .perform(Op::Write {
                    fd: sfd,
                    bytes: b"x".to_vec(),
                })
                .await);
            say(io.perform(Op::Read { fd: cfd, max: 9 }).await);
            say(io.perform(Op::Close { fd: cfd }).await);
        },
    );
    let mut sim = Sim::new(Vec::new());
    assert_eq!(drive(&mut s, &hub, &mut sim, 1_000_000), Outcome::Returned);
    assert_eq!(
        seen.take(),
        [
            "Err(\"port 80: connection refused\")",
            "Count(3)",
            "Count(49152)",
            "Err(\"port 49152: address in use\")",
            "Count(0)",
            "Bytes([])",
            "Err(\"fd 5: broken pipe\")",
            "Err(\"fd 4: closed\")",
            "Err(\"fd 4: not open\")",
        ]
    );
}

/// An accept waits for a connection, and a read for bytes, while the
/// other task goes on: the client connects only after the server waits.
#[test]
fn an_accept_waits_for_its_connection() {
    let seen = Seen::default();
    let mut s = Scheduler::new(Config::default(), Memory::default());
    let hub = IoHub::new();
    let (h, w) = (hub.clone(), seen.clone());
    s.spawn(
        Spawn::new("server").effect(Effect::pure()),
        move |cx| async move {
            let io = h.io(&cx);
            let l = fd(&io.perform(Op::Listen { port: PORT }).await).unwrap();
            w.borrow_mut().push("accepting".into());
            let c = fd(&io.perform(Op::Accept { fd: l }).await).unwrap();
            w.borrow_mut().push("accepted".into());
            let a = io.perform(Op::Read { fd: c, max: 64 }).await;
            w.borrow_mut().push(format!("read {a:?}"));
        },
    );
    let (h, w) = (hub.clone(), seen.clone());
    s.spawn(
        Spawn::new("client").effect(Effect::pure()),
        move |cx| async move {
            let io = h.io(&cx);
            cx.sleep_until(100).await;
            w.borrow_mut().push("connecting".into());
            let c = fd(&io.perform(Op::Connect { port: PORT }).await).unwrap();
            cx.sleep_until(200).await;
            w.borrow_mut().push("writing".into());
            io.perform(Op::Write {
                fd: c,
                bytes: b"hi".to_vec(),
            })
            .await;
        },
    );
    let mut sim = Sim::new(Vec::new());
    assert_eq!(drive(&mut s, &hub, &mut sim, 1_000_000), Outcome::Returned);
    assert_eq!(
        seen.take(),
        [
            "accepting",
            "connecting",
            "accepted",
            "writing",
            "read Bytes([104, 105])"
        ]
    );
}

fn some(seed: u64) -> Faults {
    Faults {
        seed,
        fail: 100,
        torn: 300,
        delay: 20,
    }
}

#[test]
fn a_seed_is_one_run_on_sockets_too() {
    let runs: Vec<_> = (0..30).map(|seed| run(Some(some(seed)))).collect();
    for (seed, r) in runs.iter().enumerate() {
        assert_eq!(*r, run(Some(some(seed as u64))), "seed {seed}");
    }
    let has = |what: &str| {
        runs.iter()
            .any(|(_, inj)| inj.iter().any(|i| i.starts_with(what)))
    };
    assert!(
        has("short read") && has("reset") && has("refuse"),
        "{runs:?}"
    );
    // Short reads alone change nothing the program sees: it reads on.
    for (seen, inj) in &runs {
        if inj.iter().all(|i| i.starts_with("short read")) {
            assert_eq!(seen[0], "client: back \"ONE\\nTWO\\nTHREE\\n\"", "{inj:?}");
        }
    }
}

/// A reset is an error at both ends, and neither waits forever.
#[test]
fn a_reset_is_an_error_at_both_ends() {
    let (seen, injected) = run(Some(Faults {
        seed: 5,
        fail: 1000,
        ..Faults::default()
    }));
    // Every connect is refused at this rate.
    assert_eq!(injected, ["refuse connect 7000"]);
    assert!(seen[0].starts_with("client: connect: Err"), "{seen:?}");
    let resets: Vec<_> = (0..40)
        .map(|seed| run(Some(some(seed))))
        .filter(|(_, inj)| inj.iter().any(|i| i.starts_with("reset")))
        .collect();
    assert!(!resets.is_empty());
    for (seen, _) in resets {
        assert!(
            seen.iter().any(|s| s.contains("connection reset")),
            "{seen:?}"
        );
    }
}

#[test]
fn a_faulty_run_on_sockets_replays() {
    let seen = Seen::default();
    let log = Rc::new(RefCell::new(header(&[])));
    let mut s = Scheduler::new(
        Config::default(),
        Recorder::new(Memory::default(), log.clone(), true, 0),
    );
    let hub = program(&mut s, &seen, PORT);
    let mut sim = Sim::new(Vec::new()).faults(some(2));
    assert_eq!(drive(&mut s, &hub, &mut sim, 1_000_000), Outcome::Returned);
    assert!(!sim.injected.is_empty());
    let recorded = s.sink().inner.clone();
    let first = seen.take();

    let l = read(&log.borrow()).expect("a log");
    let (mut replay, expect) = Replay::new(&l);
    let mut s = Scheduler::new(Config::default(), Checker::new(Memory::default(), expect));
    let hub = program(&mut s, &seen, PORT);
    assert_eq!(
        drive(&mut s, &hub, &mut replay, 1_000_000),
        Outcome::Returned
    );
    assert_eq!(s.sink().inner, recorded);
    assert_eq!(seen.take(), first);
}

/// The same echo on the live runner, through the system's localhost: a
/// port that was free a moment ago, and the reactor to wait on.
/// Runs `f` on a thread of its own and gives what it saw, or fails after
/// ten seconds: a live run that waits for an answer never coming would
/// otherwise wait forever.
fn within(f: impl FnOnce() -> Vec<String> + Send + 'static) -> Vec<String> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || tx.send(f()));
    rx.recv_timeout(std::time::Duration::from_secs(10))
        .expect("the live run ended in time")
}

#[test]
fn an_echo_over_the_real_localhost() {
    let seen = within(echo_live);
    assert_eq!(
        seen,
        ["client: back \"ONE\\nTWO\\nTHREE\\n\"", "server: the end"],
    );
}

fn echo_live() -> Vec<String> {
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .and_then(|l| l.local_addr())
        .unwrap()
        .port();
    let seen = Seen::default();
    let mut s = Scheduler::new(Config::default(), Memory::default());
    let hub = program(&mut s, &seen, port);
    let mut live = Live::without_input().unwrap();
    assert_eq!(drive(&mut s, &hub, &mut live, 1_000_000), Outcome::Returned);
    seen.take()
}

/// Live: a refusal, a port in use, a closed fd; and a big write, more
/// than a socket's buffer, answered once all of it is written, while the
/// reader takes it in pieces.
#[test]
fn live_refusals_and_a_big_write() {
    assert_eq!(
        within(big_write),
        [
            "again: true".to_string(),
            "refused: true".into(),
            format!("wrote Count({BIG}), read {BIG} in many: true"),
            "end Bytes([])".into(),
            "closed Err(\"fd 5: closed\")".into(),
        ]
    );
}

const BIG: usize = 4 << 20;

fn big_write() -> Vec<String> {
    let seen = Seen::default();
    let mut s = Scheduler::new(Config::default(), Memory::default());
    let hub = IoHub::new();
    let (h, w) = (hub.clone(), seen.clone());
    s.spawn(
        Spawn::new("t").effect(Effect::pure()),
        move |cx| async move {
            let io = h.io(&cx);
            let l = fd(&io.perform(Op::Listen { port: 0 }).await).unwrap();
            let Answer::Count(port) = io.perform(Op::Port { fd: l }).await else {
                panic!("no port");
            };
            let port = port as u16;
            let again = io.perform(Op::Listen { port }).await;
            w.borrow_mut()
                .push(format!("again: {}", matches!(again, Answer::Err(_))));
            let c = fd(&io.perform(Op::Connect { port }).await).unwrap();
            let sv = fd(&io.perform(Op::Accept { fd: l }).await).unwrap();
            io.perform(Op::Close { fd: l }).await;
            let refused = io.perform(Op::Connect { port }).await;
            w.borrow_mut()
                .push(format!("refused: {}", matches!(refused, Answer::Err(_))));
            // The writer and the reader at once: two tasks' worth, as two
            // operations out together.
            let wt = h.submit(Op::Write {
                fd: c,
                bytes: vec![7; BIG],
            });
            let mut got = 0;
            let mut reads = 0;
            while got < BIG {
                match io
                    .perform(Op::Read {
                        fd: sv,
                        max: 1 << 16,
                    })
                    .await
                {
                    Answer::Bytes(b) if !b.is_empty() => {
                        got += b.len();
                        reads += 1;
                    }
                    a => panic!("{a:?}"),
                }
            }
            let wrote = Answer::decode(&cx.completion(wt).await);
            w.borrow_mut().push(format!(
                "wrote {wrote:?}, read {got} in many: {}",
                reads > 1
            ));
            io.perform(Op::Close { fd: c }).await;
            let end = io.perform(Op::Read { fd: sv, max: 9 }).await;
            w.borrow_mut().push(format!("end {end:?}"));
            io.perform(Op::Close { fd: sv }).await;
            let closed = io.perform(Op::Read { fd: sv, max: 9 }).await;
            w.borrow_mut().push(format!("closed {closed:?}"));
        },
    );
    let mut live = Live::without_input().unwrap();
    assert_eq!(
        drive(&mut s, &hub, &mut live, 10_000_000),
        Outcome::Returned
    );
    seen.take()
}

/// An accept no one connects to, and a read with nothing to read,
/// cancelled: each answered "cancelled", and the run ends rather than
/// waits; cancelling what was answered already does nothing. In the sim
/// and live.
fn cancels(runner: &mut dyn kayak_runner::Runner) -> Vec<String> {
    let seen = Seen::default();
    let mut s = Scheduler::new(Config::default(), Memory::default());
    let hub = IoHub::new();
    let (h, w) = (hub.clone(), seen.clone());
    s.spawn(
        Spawn::new("t").effect(Effect::pure()),
        move |cx| async move {
            let io = h.io(&cx);
            let l = fd(&io.perform(Op::Listen { port: 0 }).await).unwrap();
            let Answer::Count(port) = io.perform(Op::Port { fd: l }).await else {
                panic!("no port");
            };
            let c = fd(&io.perform(Op::Connect { port: port as u16 }).await).unwrap();
            let sv = fd(&io.perform(Op::Accept { fd: l }).await).unwrap();
            let accept = h.submit(Op::Accept { fd: l });
            let read = h.submit(Op::Read { fd: sv, max: 9 });
            h.submit(Op::Cancel { token: accept });
            h.submit(Op::Cancel { token: read });
            for t in [accept, read] {
                let a = Answer::decode(&cx.completion(t).await);
                w.borrow_mut().push(format!("{a:?}"));
            }
            // Answered already: nothing.
            let wrote = h.submit(Op::Write {
                fd: c,
                bytes: b"x".to_vec(),
            });
            let a = Answer::decode(&cx.completion(wrote).await);
            h.submit(Op::Cancel { token: wrote });
            w.borrow_mut().push(format!("{a:?}"));
            let a = io.perform(Op::Read { fd: sv, max: 9 }).await;
            w.borrow_mut().push(format!("{a:?}"));
        },
    );
    assert_eq!(drive(&mut s, &hub, runner, 1_000_000), Outcome::Returned);
    seen.take()
}

#[test]
fn a_cancelled_wait_is_answered_cancelled() {
    let want = [
        "Err(\"cancelled\")",
        "Err(\"cancelled\")",
        "Count(1)",
        "Bytes([120])",
    ];
    assert_eq!(cancels(&mut Sim::new(Vec::new())), want);
    assert_eq!(
        within(|| cancels(&mut Live::without_input().unwrap())),
        want
    );
}

/// A half-close: the client writes, then shuts down its writing; the
/// server reads what came, then the end, and still writes back, which the
/// client still reads. Writing after the shutdown is an error, as is
/// shutting down a listener. In the sim and live.
fn half_close(runner: &mut dyn kayak_runner::Runner) -> Vec<String> {
    let seen = Seen::default();
    let mut s = Scheduler::new(Config::default(), Memory::default());
    let hub = IoHub::new();
    let (h, w) = (hub.clone(), seen.clone());
    s.spawn(
        Spawn::new("t").effect(Effect::pure()),
        move |cx| async move {
            let io = h.io(&cx);
            let say = |a: Answer| w.borrow_mut().push(format!("{a:?}"));
            let l = fd(&io.perform(Op::Listen { port: 0 }).await).unwrap();
            let Answer::Count(port) = io.perform(Op::Port { fd: l }).await else {
                panic!("no port");
            };
            let c = fd(&io.perform(Op::Connect { port: port as u16 }).await).unwrap();
            let sv = fd(&io.perform(Op::Accept { fd: l }).await).unwrap();
            let bytes = b"ping".to_vec();
            say(io.perform(Op::Write { fd: c, bytes }).await);
            say(io.perform(Op::Shutdown { fd: c }).await);
            say(io.perform(Op::Read { fd: sv, max: 64 }).await);
            say(io.perform(Op::Read { fd: sv, max: 64 }).await);
            let bytes = b"pong".to_vec();
            say(io.perform(Op::Write { fd: sv, bytes }).await);
            say(io.perform(Op::Read { fd: c, max: 64 }).await);
            let bytes = b"late".to_vec();
            let late = io.perform(Op::Write { fd: c, bytes }).await;
            w.borrow_mut()
                .push(format!("late: {}", matches!(late, Answer::Err(_))));
            let l = io.perform(Op::Shutdown { fd: l }).await;
            w.borrow_mut()
                .push(format!("listener: {}", matches!(l, Answer::Err(_))));
        },
    );
    assert_eq!(drive(&mut s, &hub, runner, 1_000_000), Outcome::Returned);
    seen.take()
}

#[test]
fn a_half_close_ends_one_direction() {
    let want = [
        "Count(4)",
        "Count(0)",
        "Bytes([112, 105, 110, 103])",
        "Bytes([])",
        "Count(4)",
        "Bytes([112, 111, 110, 103])",
        "late: true",
        "listener: true",
    ];
    assert_eq!(half_close(&mut Sim::new(Vec::new())), want);
    assert_eq!(
        within(|| half_close(&mut Live::without_input().unwrap())),
        want
    );
}
