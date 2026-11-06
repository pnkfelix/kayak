//! The one event stream (`PLAN.md` §3.4).
//!
//! Every subsystem reports what it decided here: the scheduler which task
//! ran and why its slice ended, the runners what they delivered, later the
//! collector's phases, the query engine's runs and the tier swaps. A run is
//! reproducible exactly when its stream is (`PLAN.md` §1, I1), so the tests
//! compare streams, and the replay log will be a view of this one.
//!
//! An event is numbered by the scheduler **step** it happened in, never by
//! wall time: steps are the clock. The vocabulary is one enum, [`What`],
//! extended as subsystems arrive, so that the stream reads as one story.
// coopt-source: crates/coopt-trace/src/lib.rs@9283bba

use std::fmt;

/// One thing that happened, at a scheduler step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event {
    pub step: u64,
    pub what: What,
}

/// Why a task's slice ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliceEnd {
    /// Its fuel ran out. It is runnable, and between two of its suspension
    /// points.
    Preempted,
    /// It gave the scheduler back voluntarily, and is runnable.
    Yielded,
    /// It is waiting for a wake.
    Suspended,
    /// It returned.
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum What {
    /// A task was made.
    Spawn {
        task: u64,
        name: String,
        priority: u8,
    },
    /// A task's priority changed.
    Priority { task: u64, priority: u8 },
    /// A query was run, verified, cut off, or failed (`coopt-query`).
    Query {
        action: &'static str,
        query: &'static str,
        key: String,
    },
    /// A task was chosen to run, with this much fuel.
    Run { task: u64, budget: u64 },
    /// The slice of the task that was running ended, having used `used` fuel.
    SliceEnd { task: u64, used: u64, end: SliceEnd },
    /// A completion was taken from the inbox: an input of the run, which
    /// the replay log keeps (`docs/research/runners.md` §3).
    Arrive { token: u64, bytes: Vec<u8> },
    /// A task performed an operation, given to the runner under `token`.
    Perform { token: u64, op: String },
    /// A completion from outside was delivered, to whoever waits on `token`.
    Deliver { token: u64, len: u64 },
    /// A timer fired, waking a task.
    Timer { task: u64 },
    /// The virtual clock moved forward, because nothing could run before.
    Clock { now: u64 },
    /// A runner moved the clock: an input of the run, logged.
    ClockSet { now: u64 },
    /// The collector took a step: the `n`th, of a kind (a minor
    /// collection, an evacuation, a slice of marking or summarization, a
    /// full collection), taking `units` of work (R5 §4).
    Collect {
        n: u64,
        kind: &'static str,
        units: u64,
    },
    /// A label a test or tool put in the stream.
    Mark { label: String },
    /// A task declared its effect, at a suspension point: what it does from
    /// now on.
    Declare { task: u64 },
    /// A task's section has stayed open (preempted, not yet at a
    /// suspension point) for `slices` slices: a hog, reported at 8, 16, 32,
    /// … (R7, decision 5).
    Hog { task: u64, slices: u64 },
    /// A task whose open section blocks a more urgent one runs at that
    /// one's priority until its section closes (`None`: it closed).
    Inherit { task: u64, priority: Option<u8> },
    /// A demotable task fell to a lower class, having run long.
    Demote { task: u64, priority: u8 },
    /// A demoted task came back up, having waited.
    Promote { task: u64, priority: u8 },
}

impl fmt::Display for SliceEnd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            SliceEnd::Preempted => "preempted",
            SliceEnd::Yielded => "yielded",
            SliceEnd::Suspended => "suspended",
            SliceEnd::Finished => "finished",
        })
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:>6} ", self.step)?;
        match &self.what {
            What::Spawn {
                task,
                name,
                priority,
            } => {
                write!(f, "spawn t{task} {name:?} priority {priority}")
            }
            What::Priority { task, priority } => write!(f, "priority t{task} {priority}"),
            What::Query { action, query, key } => write!(f, "query {action} {query}({key})"),
            What::Run { task, budget } => write!(f, "run t{task} budget {budget}"),
            What::SliceEnd { task, used, end } => write!(f, "end t{task} used {used} {end}"),
            What::Arrive { token, bytes } => {
                write!(f, "arrive #{token} {:?}", String::from_utf8_lossy(bytes))
            }
            What::Perform { token, op } => write!(f, "perform #{token} {op}"),
            What::Deliver { token, len } => write!(f, "deliver #{token} ({len} bytes)"),
            What::Timer { task } => write!(f, "timer t{task}"),
            What::Clock { now } => write!(f, "clock {now}"),
            What::ClockSet { now } => write!(f, "clock set {now}"),
            What::Collect { n, kind, units } => write!(f, "gc #{n} {kind} {units} units"),
            What::Mark { label } => write!(f, "mark {label}"),
            What::Declare { task } => write!(f, "declare t{task}"),
            What::Hog { task, slices } => write!(f, "hog t{task} open {slices} slices"),
            What::Inherit {
                task,
                priority: Some(p),
            } => write!(f, "inherit t{task} priority {p}"),
            What::Inherit {
                task,
                priority: None,
            } => write!(f, "inherit t{task} ends"),
            What::Demote { task, priority } => write!(f, "demote t{task} priority {priority}"),
            What::Promote { task, priority } => write!(f, "promote t{task} priority {priority}"),
        }
    }
}

/// Where events go.
pub trait Sink {
    fn record(&mut self, event: &Event);
}

/// Keeps every event, for tests and for comparing two runs.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Memory {
    pub events: Vec<Event>,
}

impl Sink for Memory {
    fn record(&mut self, event: &Event) {
        self.events.push(event.clone());
    }
}

impl Memory {
    /// The stream as text, one event to a line.
    pub fn text(&self) -> String {
        let mut s = String::new();
        for e in &self.events {
            s.push_str(&e.to_string());
            s.push('\n');
        }
        s
    }
}

/// Writes each event as a line of text.
pub struct Text<W: std::io::Write> {
    out: W,
}

impl<W: std::io::Write> Text<W> {
    pub fn new(out: W) -> Text<W> {
        Text { out }
    }

    pub fn into_inner(self) -> W {
        self.out
    }
}

impl<W: std::io::Write> Sink for Text<W> {
    fn record(&mut self, event: &Event) {
        // A trace that cannot be written must not change the run.
        let _ = writeln!(self.out, "{event}");
    }
}

/// Drops every event.
#[derive(Debug, Default)]
pub struct Discard;

impl Sink for Discard {
    fn record(&mut self, _: &Event) {}
}
