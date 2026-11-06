//! Runners, and the one loop that drives the scheduler with any of them
//! (`docs/research/runners.md` §5).
//!
//! - [`Sim`]: scripted input at virtual times, output captured.
//! - [`Replay`]: answers from a log, and the first divergence reported.
//! - Live: M8.
//!
//! A runner never blocks the scheduler and never reaches a task but
//! through the inbox, at a step. Ahman and Bauer's signals are [`Kill`]:
//! the run cannot go on, and no task sees it.
// coopt-source: crates/coopt-runner/src/runner.rs@9283bba

use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::rc::Rc;

use kayak_base::Fingerprint;
use kayak_sched::{Inbox, Scheduler, Step};
use kayak_trace::{Event, Sink, What};

use crate::io::{Answer, IoHub, Op, STDIN};
use crate::log::{Log, Record, StreamHash};
use crate::net::{Net, Wait};
use crate::pool::Pool;
use crate::values::{UseValues, Values};
use kayak_sched::Job;

/// What the scheduler's being idle means to a runner.
#[derive(Debug, PartialEq, Eq)]
pub enum Wake {
    /// Go on: something was posted, or will be at a later step.
    Posted,
    /// Move the clock here, then go on.
    Clock(u64),
    /// Nothing more will come: the run is over.
    Done,
}

/// The run diverged from its log: where, and how.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Divergence {
    pub step: u64,
    pub expected: String,
    pub found: String,
}

/// A signal: the run ends, and no task sees why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kill {
    Diverged(Divergence),
    /// A signal that ends the process (`TERM`, `HUP`): the run stopped
    /// between steps, so its log is written and its terminal restored.
    Signal(String),
    /// The log was used up while the run still wanted answers.
    LogEnded {
        step: u64,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Nothing more to do.
    Returned,
    Failed(String),
    Killed(Kill),
}

pub trait Runner {
    /// An operation, performed under `token`. Never blocks.
    fn submit(&mut self, token: u64, op: Op);
    /// Before step `next`: post whatever is due; move the clock, if this
    /// runner keeps it (`ClockMode::Runner`), by returning where to.
    /// `deadline` is the earliest timer's.
    fn poll(&mut self, next: u64, now: u64, deadline: Option<u64>, inbox: &Inbox) -> Option<u64>;
    /// Nothing can run, after step `step`.
    fn idle(&mut self, step: u64, now: u64, deadline: Option<u64>, inbox: &Inbox) -> Wake;
    /// When this runner will next post something of its own accord, if it
    /// knows: the scheduler's autojumps go no further, so that it does
    /// not come late.
    fn next_event(&self) -> Option<u64> {
        None
    }
    /// A signal, if one was raised.
    fn kill(&mut self) -> Option<Kill> {
        None
    }
    /// Called once, however the run ended.
    fn finish(&mut self, _outcome: &Outcome) {}
}

/// Drives `sched` with `runner` until the runner says the run is over,
/// a signal ends it, or `max_steps` steps have passed.
pub fn drive<S: Sink>(
    sched: &mut Scheduler<S>,
    hub: &IoHub,
    runner: &mut dyn Runner,
    max_steps: u64,
) -> Outcome {
    let inbox = sched.inbox();
    let outcome = loop {
        if let Some(k) = runner.kill() {
            break Outcome::Killed(k);
        }
        for (token, op) in hub.take() {
            sched.note(What::Perform {
                token,
                op: op.to_string(),
            });
            runner.submit(token, op);
        }
        if let Some(k) = runner.kill() {
            break Outcome::Killed(k);
        }
        if sched.steps() >= max_steps {
            break Outcome::Failed(format!("still running after {max_steps} steps"));
        }
        if let Some(t) = runner.poll(
            sched.steps() + 1,
            sched.now(),
            sched.next_deadline(),
            &inbox,
        ) {
            sched.advance_to(t);
        }
        sched.limit_jumps(runner.next_event());
        if let Step::Idle = sched.step() {
            match runner.idle(sched.steps(), sched.now(), sched.next_deadline(), &inbox) {
                Wake::Posted => {}
                Wake::Clock(t) => sched.advance_to(t),
                Wake::Done => {
                    if let Some(k) = runner.kill() {
                        break Outcome::Killed(k);
                    }
                    break Outcome::Returned;
                }
            }
        }
    };
    runner.finish(&outcome);
    outcome
}

// ---------------------------------------------------------------- sim

/// Faults a sim injects (M14), by seed, so that a run with them is the
/// same run every time: each file operation fails at the rate `fail`; an
/// append is torn at the rate `torn`, a prefix of its bytes written and an
/// error answered, as a `write_all` that fails partway; and a file
/// operation's answer comes up to `delay` ticks late, so that answers
/// overtake one another. Rates are per thousand. The terminal is spared.
///
/// On sockets: a connect is refused at the rate `fail`, and a write resets
/// its connection at that rate (both ends' reads and writes are errors
/// from then on); a read takes fewer bytes than are there at the rate
/// `torn`, as a stream may; and every answer may come late.
#[derive(Clone, Copy, Debug, Default)]
pub struct Faults {
    pub seed: u64,
    pub fail: u32,
    pub torn: u32,
    pub delay: u64,
}

/// Faults a sim injects in its keys (R3 §2.1), by a seed of their own, so
/// that they shift no other fault's draws: each key comes up to `jitter`
/// ticks late, never before the key before it; and a read of standard
/// input takes fewer bytes than are there at the rate `split` per
/// thousand, as a pty's read may end anywhere, within a character too.
/// Writes are not cut short: a write is answered once all of it is out,
/// so a short one is the runner's, never the program's (R4 §6).
#[derive(Clone, Copy, Debug, Default)]
pub struct KeyFaults {
    pub seed: u64,
    pub jitter: u64,
    pub split: u32,
}

/// Input at given virtual times; what is written to fd 1 or 2, kept.
#[derive(Default)]
pub struct Sim {
    script: VecDeque<(u64, Vec<u8>)>,
    /// Signals, by name, at their times.
    signals: VecDeque<(u64, String)>,
    reads: VecDeque<(u64, u32)>,
    /// Who waits for the next signal.
    signal_waiters: VecDeque<u64>,
    ready: Vec<(u64, Answer)>,
    pub stdout: Rc<RefCell<Vec<u8>>>,
    /// Files, by path, in memory: read and appended as a live runner's.
    pub files: Rc<RefCell<std::collections::BTreeMap<String, Vec<u8>>>>,
    faults: Option<Faults>,
    /// The faults' generator (splitmix64), from the seed.
    rng: u64,
    /// Answers held back: how many ticks late, once a poll says when now
    /// is.
    late: Vec<(u64, u64, Answer)>,
    /// Answers held back, by when they come.
    due: Vec<(u64, u64, Answer)>,
    /// What was injected, in order: `fail read-file "h"`, say.
    pub injected: Vec<String>,
    /// The terminal's width, if output is one.
    width: Option<u32>,
    /// The clock, as the last poll or wait said: a step's time, since the
    /// clock does not move during one.
    now: u64,
    /// Localhost.
    net: Net,
    /// Reads of standard input cut short, per thousand, and the
    /// generator that says which.
    split: u32,
    key_rng: u64,
    /// The jobs' threads, started with the first job.
    pool: Option<Pool>,
    /// Jobs not yet delivered: by the step they are due, and token.
    jobs: Vec<(u64, u64)>,
    /// How many steps after its submission a job is delivered at most, if
    /// not [`JOB_STEPS`].
    job_steps: Option<u64>,
    /// The step the run is at, as the last poll said.
    step: u64,
    /// Where job values are kept, if they are (`Sim::record_values`).
    values: Option<Rc<RefCell<Values>>>,
}

/// A sim delivers a job when the run is idle, or this many steps after it
/// was submitted, whichever comes first: in between, the job runs on a
/// worker while tasks run, and the sim waits for it only when it delivers
/// it. Jobs take no virtual time.
pub const JOB_STEPS: u64 = 100;

impl Sim {
    /// Standard input: each item, its bytes at its time, in order.
    pub fn new(script: Vec<(u64, Vec<u8>)>) -> Sim {
        Sim {
            script: script.into(),
            ..Sim::default()
        }
    }

    /// A typist: each line's characters a key `gap` ticks apart, then
    /// Enter, starting at `gap`.
    pub fn typing(lines: &[&str], gap: u64) -> Sim {
        let mut t = 0;
        let mut script = Vec::new();
        for line in lines {
            for c in line.chars().chain(['\n']) {
                t += gap;
                script.push((t, c.to_string().into_bytes()));
            }
        }
        Sim::new(script)
    }

    /// A file there from the start.
    pub fn file(self, path: &str, bytes: &[u8]) -> Sim {
        self.files.borrow_mut().insert(path.into(), bytes.to_vec());
        self
    }

    /// The wall clock's answer: this, plus the run's clock. A fixed
    /// start, so that a sim run is its seed's and its script's alone.
    pub const EPOCH: u64 = 1_767_225_600_000_000_000; // 2026-01-01, UTC

    /// Output to a terminal this wide.
    pub fn width(mut self, columns: u32) -> Sim {
        self.width = Some(columns);
        self
    }

    /// Faults injected from now on (`Faults`).
    pub fn faults(mut self, f: Faults) -> Sim {
        self.rng = f.seed;
        self.faults = Some(f);
        self
    }

    /// Faults in the keys (`KeyFaults`): the script's times moved now, its
    /// reads cut short as they come.
    pub fn key_faults(mut self, f: KeyFaults) -> Sim {
        self.key_rng = f.seed;
        self.split = f.split;
        if f.jitter > 0 {
            let mut last = 0;
            for (at, _) in self.script.iter_mut() {
                let late = splitmix(&mut self.key_rng) % (f.jitter + 1);
                // The script is read in order, so a late key holds up the
                // rest; their times say so, as when each comes.
                last = (*at + late).max(last);
                if last > *at {
                    self.injected.push(format!("key at {at} late, at {last}"));
                }
                *at = last;
            }
        }
        self
    }

    /// The next number from the faults' generator.
    fn next(&mut self) -> u64 {
        splitmix(&mut self.rng)
    }

    /// Whether a fault at `rate` per thousand happens now.
    fn roll(&mut self, rate: u32) -> bool {
        rate > 0 && self.next() % 1000 < u64::from(rate)
    }

    /// A file operation's answer, as the faults make it: on time, or late.
    fn answer_file(&mut self, token: u64, a: Answer) {
        let delay = self.faults.map_or(0, |f| f.delay);
        let late = if delay > 0 {
            self.next() % (delay + 1)
        } else {
            0
        };
        if late == 0 {
            self.ready.push((token, a));
        } else {
            self.late.push((late, token, a));
        }
    }

    /// The answers held back, given the times they are due, now that the
    /// time is known.
    fn settle(&mut self, now: u64) {
        for (late, t, a) in self.late.drain(..) {
            self.due.push((now + late, t, a));
        }
        self.due.sort_by_key(|(at, t, _)| (*at, *t));
    }

    /// The time of the first answer held back, if any.
    fn first_due(&self) -> Option<u64> {
        self.due.iter().map(|(at, _, _)| *at).min()
    }

    /// Answers whatever waits on a socket and can be answered now.
    fn serve_net(&mut self) {
        let torn = self.faults.map_or(0, |f| f.torn);
        let (rng, injected) = (&mut self.rng, &mut self.injected);
        let mut take = |n: usize| {
            if n > 1 && torn > 0 && splitmix(rng) % 1000 < u64::from(torn) {
                let k = 1 + (splitmix(rng) % (n as u64 - 1)) as usize;
                injected.push(format!("short read, {k} of {n}"));
                k
            } else {
                n
            }
        };
        for (t, a) in self.net.serve(&mut take) {
            self.answer_file(t, a);
        }
    }

    /// Jobs delivered at most `n` steps after they are submitted (as soon
    /// as the next step, if 0), rather than [`JOB_STEPS`]. Under faults,
    /// each job's is drawn from 0 to twice that.
    pub fn job_steps(mut self, n: u64) -> Sim {
        self.job_steps = Some(n);
        self
    }

    /// Each job's value kept in `values`, as it is delivered.
    pub fn record_values(mut self, values: Rc<RefCell<Values>>) -> Sim {
        self.values = Some(values);
        self
    }

    /// Delivers job `token`, waiting for it if it is still running.
    fn deliver_job(&mut self, token: u64, inbox: &Inbox) {
        let pool = self.pool.as_ref().expect("a job was submitted");
        let done = pool.wait(token);
        keep_value(self.values.as_ref(), &done);
        inbox.post(token, Answer::of_job(&done).encode());
    }

    /// A signal, `INT` say, raised at virtual time `at`.
    pub fn signal(mut self, at: u64, name: &str) -> Sim {
        self.signals.push_back((at, name.into()));
        self
    }

    /// The time at which the `n`th key of the script arrives, from 0.
    pub fn key_time(&self, n: usize) -> u64 {
        self.script[n].0
    }

    pub fn output(&self) -> String {
        String::from_utf8_lossy(&self.stdout.borrow()).into()
    }
}

impl Runner for Sim {
    /// An answer held back, a key for a read waiting, or a signal for
    /// someone waiting for one: the first of their times.
    fn next_event(&self) -> Option<u64> {
        let key = self
            .script
            .front()
            .filter(|_| !self.reads.is_empty())
            .map(|(at, _)| *at);
        let signal = self
            .signals
            .front()
            .filter(|_| !self.signal_waiters.is_empty())
            .map(|(at, _)| *at);
        [self.first_due(), key, signal].into_iter().flatten().min()
    }

    fn submit(&mut self, token: u64, op: Op) {
        match op {
            Op::NextSignal => self.signal_waiters.push_back(token),
            Op::Read { fd, max } if fd == STDIN => self.reads.push_back((token, max)),
            Op::Write { fd, bytes } if fd.0 == 1 || fd.0 == 2 => {
                let n = bytes.len() as u32;
                self.stdout.borrow_mut().extend_from_slice(&bytes);
                self.ready.push((token, Answer::Count(n)));
            }
            Op::TerminalWidth => {
                let a = self
                    .width
                    .map_or_else(|| Answer::Err("not a terminal".into()), Answer::Count);
                self.ready.push((token, a));
            }
            Op::WallClock => self
                .ready
                .push((token, Answer::Time(Sim::EPOCH + self.now))),
            Op::ReadFile { path } => {
                let f = self.faults.unwrap_or_default();
                let a = if self.roll(f.fail) {
                    self.injected.push(format!("fail read-file {path:?}"));
                    Answer::Err(format!("{path}: an injected fault"))
                } else {
                    match self.files.borrow().get(&path) {
                        Some(b) => Answer::Bytes(b.clone()),
                        None => Answer::Err(format!("{path}: no such file")),
                    }
                };
                self.answer_file(token, a);
            }
            Op::AppendFile { path, mut bytes } => {
                let f = self.faults.unwrap_or_default();
                let n = bytes.len() as u32;
                let a = if self.roll(f.fail) {
                    self.injected.push(format!("fail append-file {path:?}"));
                    bytes.clear();
                    Answer::Err(format!("{path}: an injected fault"))
                } else if !bytes.is_empty() && self.roll(f.torn) {
                    let kept = (self.next() % bytes.len() as u64) as usize;
                    self.injected
                        .push(format!("torn append-file {path:?} at {kept} of {n}"));
                    bytes.truncate(kept);
                    Answer::Err(format!(
                        "{path}: an injected fault, {kept} of {n} bytes written"
                    ))
                } else {
                    Answer::Count(n)
                };
                self.files
                    .borrow_mut()
                    .entry(path)
                    .or_default()
                    .extend(bytes);
                self.answer_file(token, a);
            }
            Op::Listen { port } => {
                let a = self.net.listen(port);
                self.answer_file(token, a);
            }
            Op::Port { fd } => {
                let a = self.net.port(fd);
                self.answer_file(token, a);
            }
            Op::Connect { port } => {
                let f = self.faults.unwrap_or_default();
                let a = if self.roll(f.fail) {
                    self.injected.push(format!("refuse connect {port}"));
                    Answer::Err(format!(
                        "port {port}: connection refused (an injected fault)"
                    ))
                } else {
                    self.net.connect(port)
                };
                self.answer_file(token, a);
            }
            Op::Accept { fd } => self.net.wait(token, Wait::Accept(fd)),
            Op::Read { fd, max } if Net::owns(fd) => self.net.wait(token, Wait::Read(fd, max)),
            Op::Write { fd, bytes } if Net::owns(fd) => {
                let f = self.faults.unwrap_or_default();
                let a = if self.roll(f.fail) {
                    self.injected.push(format!("reset fd{}", fd.0));
                    self.net.reset(fd);
                    Answer::Err(format!("fd {}: connection reset (an injected fault)", fd.0))
                } else {
                    self.net.write(fd, bytes)
                };
                self.answer_file(token, a);
            }
            Op::Close { fd } => {
                let a = self.net.close(fd);
                self.answer_file(token, a);
            }
            Op::Shutdown { fd } => {
                let a = self.net.shutdown(fd);
                self.answer_file(token, a);
            }
            Op::Cancel { token: t } => {
                if self.net.cancel(t) {
                    self.ready.push((t, Answer::Err("cancelled".into())));
                }
            }
            Op::Job(job) => {
                let n = self.job_steps.unwrap_or(JOB_STEPS);
                let within = match self.faults {
                    Some(_) => self.next() % (2 * n + 1),
                    None => n,
                };
                let keep = self.values.is_some();
                self.pool
                    .get_or_insert_with(|| Pool::per_core(None))
                    .submit(token, job, keep);
                self.jobs.push((self.step + 1 + within, token));
                self.jobs.sort_unstable();
            }
            op => self
                .ready
                .push((token, Answer::Err(format!("no such file: {op}")))),
        }
        self.serve_net();
    }

    fn poll(&mut self, next: u64, now: u64, _deadline: Option<u64>, inbox: &Inbox) -> Option<u64> {
        self.now = now;
        self.step = next.saturating_sub(1);
        for (t, a) in self.ready.drain(..) {
            inbox.post(t, a.encode());
        }
        let n = self.jobs.iter().take_while(|(due, _)| *due <= next).count();
        for (_, t) in self.jobs.drain(..n).collect::<Vec<_>>() {
            self.deliver_job(t, inbox);
        }
        // Late answers come when they are due, in the order they fall due
        // (by token among those due at once).
        self.settle(now);
        let n = self.due.iter().take_while(|(at, _, _)| *at <= now).count();
        for (_, t, a) in self.due.drain(..n) {
            inbox.post(t, a.encode());
        }
        while self.signals.front().is_some_and(|(at, _)| *at <= now)
            && let Some(&token) = self.signal_waiters.front()
        {
            let (_, name) = self.signals.pop_front().expect("checked");
            self.signal_waiters.pop_front();
            inbox.post(token, Answer::Signal(name).encode());
        }
        while let Some(&(token, max)) = self.reads.front() {
            let Some((at, bytes)) = self.script.front_mut() else {
                break;
            };
            if *at > now {
                break;
            }
            let mut n = (max as usize).min(bytes.len()).max(1);
            if n > 1 && self.split > 0 && splitmix(&mut self.key_rng) % 1000 < u64::from(self.split)
            {
                let k = 1 + (splitmix(&mut self.key_rng) % (n as u64 - 1)) as usize;
                self.injected.push(format!("split read, {k} of {n}"));
                n = k;
            }
            let taken: Vec<u8> = bytes.drain(..n.min(bytes.len())).collect();
            if bytes.is_empty() {
                self.script.pop_front();
            }
            self.reads.pop_front();
            inbox.post(token, Answer::Bytes(taken).encode());
        }
        None
    }

    fn idle(&mut self, step: u64, now: u64, deadline: Option<u64>, inbox: &Inbox) -> Wake {
        self.now = now;
        // Nothing else can run: the first job due, now, in no time.
        if !self.jobs.is_empty() {
            let (_, t) = self.jobs.remove(0);
            self.deliver_job(t, inbox);
            return Wake::Posted;
        }
        // An answer held back comes in its time, or sooner if something
        // else wakes the run first.
        self.settle(now);
        let wake = self.idle_on_time(step, now, deadline, inbox);
        match (self.first_due(), wake) {
            (None, w) => w,
            (Some(_), Wake::Posted) => Wake::Posted,
            (Some(at), _) if at <= now => Wake::Posted,
            (Some(at), Wake::Clock(c)) => Wake::Clock(c.min(at)),
            (Some(at), Wake::Done) => Wake::Clock(at),
        }
    }
}

impl Sim {
    fn idle_on_time(&mut self, _step: u64, now: u64, deadline: Option<u64>, inbox: &Inbox) -> Wake {
        // A signal still to come, for someone waiting for one.
        if let Some(&(at, _)) = self.signals.front()
            && !self.signal_waiters.is_empty()
            && (self.reads.is_empty() || self.script.front().is_none_or(|(k, _)| *k > at))
        {
            if at > now {
                return Wake::Clock(at);
            }
            return Wake::Posted;
        }
        // Under the runner's clock, timers are this runner's to bring.
        if let Some(d) = deadline
            && d > now
            && self.script.front().is_none_or(|(at, _)| *at > d)
        {
            return Wake::Clock(d);
        }
        if let Some(&(token, _)) = self.reads.front() {
            return match self.script.front() {
                Some((at, _)) if *at > now => Wake::Clock(*at),
                Some(_) => Wake::Posted,
                None => {
                    // The end of the input.
                    self.reads.pop_front();
                    inbox.post(token, Answer::Bytes(Vec::new()).encode());
                    Wake::Posted
                }
            };
        }
        Wake::Done
    }
}

// ------------------------------------------------------------- replay

/// What replay compares the run against, shared with its [`Checker`].
#[derive(Default)]
pub struct Expect {
    /// Every event, if the log is full.
    events: Vec<String>,
    checkpoints: BTreeMap<u64, Fingerprint>,
    pub hash: StreamHash,
    pub diverged: Option<Divergence>,
}

/// A sink that checks the run's events against the log's as they come,
/// and passes them on.
pub struct Checker<S: Sink> {
    pub inner: S,
    expect: Rc<RefCell<Expect>>,
}

impl<S: Sink> Checker<S> {
    pub fn new(inner: S, expect: Rc<RefCell<Expect>>) -> Checker<S> {
        Checker { inner, expect }
    }
}

impl<S: Sink> Sink for Checker<S> {
    fn record(&mut self, e: &Event) {
        self.inner.record(e);
        let mut x = self.expect.borrow_mut();
        x.hash.update(e);
        if x.diverged.is_some() {
            return;
        }
        let n = x.hash.events;
        let text = e.to_string();
        if !x.events.is_empty() {
            let want = x.events.get(n as usize - 1).cloned();
            if want.as_deref() != Some(text.as_str()) {
                x.diverged = Some(Divergence {
                    step: e.step,
                    expected: want.unwrap_or_else(|| "the end of the log".into()),
                    found: text,
                });
                return;
            }
        }
        if let Some(h) = x.checkpoints.get(&n).copied()
            && h != x.hash.hash
        {
            x.diverged = Some(Divergence {
                step: e.step,
                expected: format!("the stream hash of the log at event {n}"),
                found: format!("a different one, ending at {text}"),
            });
        }
    }
}

/// Answers from a log.
pub struct Replay {
    arrivals: BTreeMap<u64, Vec<(u64, Vec<u8>)>>,
    kinds: BTreeMap<u64, String>,
    /// Clock moves by the step they were recorded after, in order: one
    /// step may see two (from `idle`, then from the next `poll`).
    clocks: BTreeMap<u64, VecDeque<u64>>,
    last: u64,
    pending: BTreeMap<u64, Op>,
    expect: Rc<RefCell<Expect>>,
    kill: Option<Kill>,
    /// The step the run is at.
    step: u64,
    /// What was written to fd 1 or 2, as the recorded run wrote it.
    pub stdout: Rc<RefCell<Vec<u8>>>,
    /// Jobs are run again, on threads as the recording's were, and each
    /// answer is checked against the recorded one.
    pool: Option<Pool>,
    /// The recorded job values, if the run kept them, and what to do with
    /// them.
    values: Option<(Values, UseValues)>,
    /// Where a job gave another value than the recorded one, and the
    /// replay went on with the recorded value.
    pub warnings: Vec<String>,
}

impl Replay {
    /// A replay of `log`, and what its [`Checker`] must share.
    pub fn new(log: &Log) -> (Replay, Rc<RefCell<Expect>>) {
        let mut r = Replay {
            arrivals: BTreeMap::new(),
            kinds: BTreeMap::new(),
            clocks: BTreeMap::new(),
            last: 0,
            pending: BTreeMap::new(),
            expect: Rc::default(),
            kill: None,
            step: 0,
            stdout: Rc::default(),
            pool: None,
            values: None,
            warnings: Vec::new(),
        };
        for rec in &log.records {
            match rec {
                Record::Arrive {
                    step,
                    token,
                    kind,
                    bytes,
                } => {
                    r.arrivals
                        .entry(*step)
                        .or_default()
                        .push((*token, bytes.clone()));
                    r.kinds.insert(*token, kind.clone());
                    r.last = r.last.max(*step);
                }
                Record::ClockSet { step, now } => {
                    r.clocks.entry(*step).or_default().push_back(*now);
                    r.last = r.last.max(*step);
                }
                Record::Checkpoint { events, hash, .. } => {
                    r.expect.borrow_mut().checkpoints.insert(*events, *hash);
                }
                Record::Event { text, .. } => r.expect.borrow_mut().events.push(text.clone()),
                Record::End { .. } => {}
            }
        }
        let expect = r.expect.clone();
        (r, expect)
    }
}

impl Replay {
    /// The recorded run's job values, used as `how` says.
    pub fn with_values(mut self, values: Values, how: UseValues) -> Replay {
        self.values = Some((values, how));
        self
    }

    /// Job `token`'s answer, recorded as `bytes`, at step `next`: the job
    /// run again gives the same, or the run has gone another way; or its
    /// value is taken from the recorded values, as they are used.
    fn answer_job(&mut self, next: u64, token: u64, job: &Job, bytes: &[u8]) {
        if let Some(v) = self.recorded_value(job, bytes, UseValues::Trust) {
            let v = v.to_vec();
            if let Err(e) = job.fill(&v) {
                self.diverge(
                    next,
                    token,
                    job,
                    bytes,
                    format!("a recorded value not loaded: {e}"),
                );
            }
            return;
        }
        // Trusting the values, a job that could be given one was not run;
        // with none kept for it, it runs here.
        let submitted = !(matches!(self.values, Some((_, UseValues::Trust))) && job.can_fill());
        let found = if submitted {
            let pool = self.pool.as_ref().expect("a job was submitted");
            Answer::of_job(&pool.wait(token))
        } else {
            Answer::of_job(&job.run_keeping(false))
        };
        if found.encode() == bytes {
            return;
        }
        if let Some(v) = self.recorded_value(job, bytes, UseValues::Fallback) {
            let v = v.to_vec();
            match job.fill(&v) {
                Ok(()) => self.warnings.push(format!(
                    "step {next}: the job {} gives {found:?}, recorded {:?}; going on with the recorded value",
                    job.name(),
                    Answer::decode(bytes)
                )),
                Err(e) => {
                    self.diverge(next, token, job, bytes, format!("a recorded value not loaded: {e}"))
                }
            }
            return;
        }
        self.diverge(next, token, job, bytes, format!("{found:?}"));
    }

    fn diverge(&mut self, step: u64, token: u64, job: &Job, bytes: &[u8], found: String) {
        if self.kill.is_none() {
            self.kill = Some(Kill::Diverged(Divergence {
                step,
                expected: format!(
                    "#{token} job {} answered {:?}",
                    job.name(),
                    Answer::decode(bytes)
                ),
                found,
            }));
        }
    }

    /// A recorded value for `job`, if this replay takes recorded values
    /// as `how` says and one is kept for the recorded answer `bytes`.
    fn recorded_value(&self, job: &Job, bytes: &[u8], how: UseValues) -> Option<&[u8]> {
        let (values, h) = self.values.as_ref()?;
        if *h != how || !job.can_fill() {
            return None;
        }
        values.get(&fingerprint_of(bytes)?)
    }

    fn next_clock(&mut self, step: u64) -> Option<u64> {
        let q = self.clocks.get_mut(&step)?;
        let now = q.pop_front();
        if q.is_empty() {
            self.clocks.remove(&step);
        }
        now
    }
}

impl Runner for Replay {
    /// The clock move recorded after the coming step: the recorded run
    /// went no further by itself, as its runner moved the clock there.
    fn next_event(&self) -> Option<u64> {
        self.clocks
            .get(&(self.step + 1))
            .and_then(|q| q.front())
            .copied()
    }

    fn submit(&mut self, token: u64, op: Op) {
        if let Some(kind) = self.kinds.get(&token)
            && kind != op.kind()
            && self.kill.is_none()
        {
            self.kill = Some(Kill::Diverged(Divergence {
                step: self.step,
                expected: format!("#{token} a {kind}"),
                found: format!("#{token} {op}"),
            }));
        }
        if let Op::Write { fd, bytes } = &op
            && (fd.0 == 1 || fd.0 == 2)
        {
            self.stdout.borrow_mut().extend_from_slice(bytes);
        }
        if let Op::Job(job) = &op {
            let trusted = matches!(self.values, Some((_, UseValues::Trust))) && job.can_fill();
            if !trusted {
                self.pool
                    .get_or_insert_with(|| Pool::per_core(None))
                    .submit(token, job.clone(), false);
            }
        }
        self.pending.insert(token, op);
    }

    fn poll(&mut self, next: u64, _now: u64, _deadline: Option<u64>, inbox: &Inbox) -> Option<u64> {
        self.step = next.saturating_sub(1);
        if let Some(items) = self.arrivals.remove(&next) {
            for (token, bytes) in items {
                if let Some(Op::Job(job)) = self.pending.remove(&token) {
                    self.answer_job(next, token, &job, &bytes);
                }
                inbox.post(token, bytes);
            }
        }
        // A clock move recorded after the step just ended, by whichever
        // path the recording runner took.
        self.next_clock(self.step)
    }

    fn idle(&mut self, step: u64, _now: u64, _deadline: Option<u64>, _inbox: &Inbox) -> Wake {
        if let Some(now) = self.next_clock(step) {
            return Wake::Clock(now);
        }
        if step < self.last {
            return Wake::Posted;
        }
        if self
            .pending
            .values()
            .any(|op| matches!(op, Op::Read { fd, .. } if *fd == STDIN))
            && self.kill.is_none()
        {
            // A read the log never answered: the log was cut short, or
            // this run wants more than the recorded one did.
            self.kill = Some(Kill::LogEnded { step });
        }
        Wake::Done
    }

    fn kill(&mut self) -> Option<Kill> {
        if self.kill.is_none()
            && let Some(d) = self.expect.borrow().diverged.clone()
        {
            self.kill = Some(Kill::Diverged(d));
        }
        self.kill.clone()
    }
}

/// Keeps a job's value in `values`, if they are kept and it gave its bytes.
fn keep_value(values: Option<&Rc<RefCell<Values>>>, done: &kayak_sched::job::Ran) {
    if let (Some(v), Ok((fp, Some(bytes)))) = (values, done) {
        v.borrow_mut().insert(*fp, bytes.clone());
    }
}

/// The fingerprint a job's recorded answer carries, if it gave one.
fn fingerprint_of(answer: &[u8]) -> Option<Fingerprint> {
    match Answer::decode(answer) {
        Answer::Bytes(b) if b.len() == 16 => Some(Fingerprint(
            u64::from_le_bytes(b[..8].try_into().expect("eight")),
            u64::from_le_bytes(b[8..].try_into().expect("eight")),
        )),
        _ => None,
    }
}

/// splitmix64: the faults' generator, a step of it.
fn splitmix(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}
