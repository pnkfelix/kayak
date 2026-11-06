//! Work a task hands off (`TODO.md` §48, stage 1).
//!
//! A [`Job`] is a function of what it captured and nothing else: no task,
//! no heap, no clock. Whoever runs a task's operations runs it too (a
//! runner, on a worker thread in a live run, or the task itself if no one
//! does), and its value comes back through a [`Slot`], with a completion
//! that says it is there. The completion is all a run sees of the job's
//! running: the step it comes at is recorded like any arrival, and its
//! bytes are the value's fingerprint, so a replay that runs the job again
//! finds out if it gives something else.
//!
//! A job made by [`Job::stored`] can also give its value's bytes (its
//! stable encoding, which its fingerprint is of) as it runs, and can be
//! given bytes in place of running: so a run's job values may be kept
//! beside its log, and a replay may go on with a recorded value where the
//! job, run again by another build, gives another.
// coopt-source: crates/coopt-sched/src/job.rs@9283bba

use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{Arc, Mutex};

use kayak_base::fingerprint::Encoder;
use kayak_base::stored::{Load, load};
use kayak_base::{Fingerprint, StableEncode};

/// What running a job gave: its value's fingerprint and, if asked for,
/// the value's bytes; or what its panic said.
pub type Ran = Result<(Fingerprint, Option<Vec<u8>>), String>;

type Work = Box<dyn FnOnce(bool) -> Ran + Send>;
type Fill = Box<dyn Fn(&[u8]) -> Result<(), String> + Send + Sync>;

/// A job, before it has run. Clones are the same job: whichever runs it
/// first runs it.
#[derive(Clone)]
pub struct Job {
    name: Arc<str>,
    work: Arc<Mutex<Option<Work>>>,
    /// Puts a value, from its bytes, in the slot: for a stored job.
    fill: Option<Arc<Fill>>,
}

/// Where a job's value goes: the value, or what its panic said.
pub struct Slot<T>(Arc<Mutex<Option<Result<T, String>>>>);

impl Job {
    /// A job computing `f()`, named `name` in the event stream (so the
    /// name must be the same in every run), and the slot its value goes to.
    pub fn new<T, F>(name: impl Into<String>, f: F) -> (Job, Slot<T>)
    where
        T: StableEncode + Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        let slot = Arc::new(Mutex::new(None));
        let out = slot.clone();
        let work: Work = Box::new(move |keep| {
            let r = catch_unwind(AssertUnwindSafe(f)).map_err(|p| panic_message(&*p));
            let ran = r.as_ref().map(|v| encoded(v, keep)).map_err(Clone::clone);
            *out.lock().expect("a job's slot") = Some(r);
            ran
        });
        let job = Job {
            name: name.into().into(),
            work: Arc::new(Mutex::new(Some(work))),
            fill: None,
        };
        (job, Slot(slot))
    }

    /// As [`Job::new`], for a value that can be loaded from its bytes: the
    /// job may then be given its value in place of running.
    pub fn stored<T, F>(name: impl Into<String>, f: F) -> (Job, Slot<T>)
    where
        T: StableEncode + Load + Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        let (mut job, slot) = Job::new(name, f);
        let into = slot.0.clone();
        job.fill = Some(Arc::new(Box::new(move |bytes: &[u8]| {
            let v = load::<T>(bytes).map_err(|b| b.0)?;
            *into.lock().expect("a job's slot") = Some(Ok(v));
            Ok(())
        })));
        (job, slot)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    /// Runs the job, here: its value's fingerprint, or what its panic said.
    /// A job runs once.
    pub fn run(&self) -> Result<Fingerprint, String> {
        self.run_keeping(false).map(|(f, _)| f)
    }

    /// Runs the job, here, and if `keep`, keeps its value's bytes too, if
    /// the value can be loaded from them ([`Job::stored`]).
    pub fn run_keeping(&self, keep: bool) -> Ran {
        let work = self.work.lock().expect("a job").take();
        match work {
            Some(w) => w(keep && self.fill.is_some()),
            None => panic!("the job {:?} has run already", self.name),
        }
    }

    /// Whether the job's value can be given it from bytes.
    pub fn can_fill(&self) -> bool {
        self.fill.is_some()
    }

    /// Gives the job its value from `bytes`, in place of whatever running
    /// it gave or would give: its slot then holds that value. The job need
    /// not have run.
    pub fn fill(&self, bytes: &[u8]) -> Result<(), String> {
        match &self.fill {
            Some(f) => f(bytes),
            None => Err(format!("the job {:?}'s value cannot be loaded", self.name)),
        }
    }
}

impl<T> Slot<T> {
    /// The value, once the job has run; it panics where the job did.
    pub fn take(&self) -> Option<T> {
        match self.0.lock().expect("a job's slot").take() {
            Some(Ok(v)) => Some(v),
            Some(Err(m)) => panic!("a job panicked: {m}"),
            None => None,
        }
    }
}

impl fmt::Debug for Job {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Job({:?})", self.name)
    }
}

impl PartialEq for Job {
    fn eq(&self, other: &Job) -> bool {
        Arc::ptr_eq(&self.work, &other.work)
    }
}

impl Eq for Job {}

/// A value's fingerprint, and its bytes if `keep`.
fn encoded<T: StableEncode>(v: &T, keep: bool) -> (Fingerprint, Option<Vec<u8>>) {
    let mut e = if keep {
        Encoder::recording()
    } else {
        Encoder::default()
    };
    v.encode(&mut e);
    e.into_parts()
}

fn panic_message(p: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = p.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = p.downcast_ref::<String>() {
        s.clone()
    } else {
        "a panic with no message".into()
    }
}
