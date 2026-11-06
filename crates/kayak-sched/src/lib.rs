//! The one scheduler (`PLAN.md` §1, I1 and I3).
//!
//! Every piece of work in a run is a task here: user tasks, collector
//! slices, compiler queries, deliveries from runners. A task is a Rust
//! future, polled for one **slice** at a time, with a budget of **fuel**
//! that it spends through [`Cx::charge`]. There are no threads and no
//! timers: a slice ends because the task's fuel ran out, because it
//! yielded, or because it waits for something, and nothing reaches a task
//! except through the scheduler, between slices.
//!
//! Choosing the next task:
//! - **Priority first.** The highest [`Priority`] class with a task that may
//!   run wins; within a class, the task runnable longest.
//! - **Effects decide which tasks may run.** A task whose fuel ran out is
//!   between two of its suspension points: its section is *open*. While it
//!   is, no task whose [`Effect`] interferes with its effect runs. Such a
//!   task waits until the open one reaches a suspension point (a yield, or a
//!   wait). So an open section is never interleaved with anything that could
//!   observe it or change what it reads, and preemption by fuel is invisible
//!   to the program, with no API for the program to call.
//! - **Seeded.** Under [`Stress`], fuel quanta, the choice among runnable
//!   tasks, and the delays of deliveries are drawn from the seed, to explore
//!   other orders. The same seed always gives the same run.
//!
//! Time is virtual, and work takes it: the clock moves by the fuel spent,
//! one tick a unit, so a timer can come due while tasks work, as a
//! keystroke does while the compiler runs. When nothing can run, the clock
//! jumps to the next timer (trio's `MockClock` with autojump). A live
//! clock will come from a runner (M8), as a recorded event like any other.
//!
//! Everything the scheduler decides goes to the event stream
//! ([`kayak_trace`]), numbered by step.
//!
//! A task returns nothing ([`Scheduler::spawn`]), or a value its spawner
//! takes, or another task awaits, through a [`JoinHandle`]
//! ([`Scheduler::spawn_join`]). Dropping the handle cancels nothing.
//!
//! A task may hand pure work to a [`Job`], which a runner runs where it
//! likes, a worker thread among them; the scheduler sees only its
//! completion ([`job`]).
// coopt-source: crates/coopt-sched/src/lib.rs@9283bba

pub mod effect;
pub mod job;

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::future::Future;
use std::io::Write;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};

use kayak_base::{Fingerprint, FixedMap, Rng, fingerprint};
use kayak_trace::{Event, Sink, SliceEnd, What};

pub use effect::{Effect, Region};
pub use job::{Job, Slot};

/// A demotable task falls a class for each this much fuel counted against
/// it (spent, less the time it waited on the world), by default: two
/// slices' worth.
pub const DEMOTE_AFTER: u64 = 20_000;

/// The lowest class demotion takes a task to: below it are only idle
/// work, and nothing falls there that was not put there.
const DEMOTE_TO: Priority = Priority::Background;

/// How many priority classes there are.
const CLASSES: usize = Priority::Idle as usize + 1;

/// A section open this many slices is reported as a hog, and again at each
/// doubling (R7, decision 5).
const HOG_SLICES: u64 = 8;

/// Priority classes, most urgent first (`PLAN.md` §1, I3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority {
    /// Handling what the user types.
    Input = 0,
    /// The work the user is waiting on.
    UserWaiting = 1,
    /// Collection the mutator is in debt for.
    GcDebt = 2,
    /// Compilation something runnable waits on.
    CompileNeeded = 3,
    /// Compilation of what the user is typing, before they press Enter.
    Typing = 4,
    /// Background compilation and speculation.
    Background = 5,
    /// Idle work.
    Idle = 6,
}

impl Priority {
    /// Every class, most urgent first: by its number.
    const ALL: [Priority; CLASSES] = [
        Priority::Input,
        Priority::UserWaiting,
        Priority::GcDebt,
        Priority::CompileNeeded,
        Priority::Typing,
        Priority::Background,
        Priority::Idle,
    ];
}

/// A task, by number, in order of spawning from 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskId(pub u64);

/// How a run explores other schedules.
#[derive(Debug, Clone)]
pub struct Stress {
    /// Each slice's fuel is drawn from `1..=max_quantum`.
    pub max_quantum: u64,
    /// With this chance (numerator, denominator), priority is ignored for a
    /// step and any task that may run is chosen; otherwise a random one in
    /// the most urgent class.
    pub ignore_priority: (u64, u64),
    /// Each delivery is held back by `0..=max_delivery_delay` steps.
    pub max_delivery_delay: u64,
}

impl Stress {
    /// A policy that perturbs everything it can.
    pub fn heavy() -> Stress {
        Stress {
            max_quantum: 16,
            ignore_priority: (1, 4),
            max_delivery_delay: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub seed: u64,
    /// The fuel each slice gets, unless stress says otherwise.
    pub quantum: u64,
    pub stress: Option<Stress>,
    /// Who moves the clock when nothing can run.
    pub clock: ClockMode,
    /// A demotable task falls a class for each this much fuel counted
    /// against it: spent, less the time it waited on the world
    /// (`Spawn::demotable`); 0, never.
    pub demote_after: u64,
}

/// Who moves the clock when nothing can run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ClockMode {
    /// The scheduler jumps to the next timer (autojump): virtual time,
    /// for tests and the simulated runner.
    #[default]
    Auto,
    /// Only a runner does ([`Scheduler::advance_to`]): wall time, which
    /// the live runner reads, and records.
    Runner,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            seed: 0,
            quantum: 10_000,
            stress: None,
            clock: ClockMode::Auto,
            demote_after: DEMOTE_AFTER,
        }
    }
}

/// How a task is to be spawned.
#[derive(Debug, Clone)]
pub struct Spawn {
    pub name: String,
    pub priority: Priority,
    pub effect: Effect,
    /// Whether it falls a class as it runs long, and comes back when it
    /// waits (`TODO.md` §48: a multilevel feedback queue).
    pub demotable: bool,
}

impl Spawn {
    /// A background task with an unknown effect, which is safe but
    /// interferes with everything that is not pure.
    pub fn new(name: impl Into<String>) -> Spawn {
        Spawn {
            name: name.into(),
            priority: Priority::Background,
            effect: Effect::any(),
            demotable: false,
        }
    }

    pub fn priority(mut self, p: Priority) -> Spawn {
        self.priority = p;
        self
    }

    pub fn effect(mut self, e: Effect) -> Spawn {
        self.effect = e;
        self
    }

    /// A task that falls a class for each `Config::demote_after` fuel it
    /// spends, down to `Background`, and rises back as it waits on the
    /// world, by the time it waits: so that work which runs long (a REPL
    /// entry that loops) leaves quick work (what is typed) quick, and work
    /// that waits as much as it computes is left where it is. By fuel and
    /// the run's clock, so a run demotes the same task at the same step
    /// every time.
    pub fn demotable(mut self) -> Spawn {
        self.demotable = true;
        self
    }
}

type TaskFuture = Pin<Box<dyn Future<Output = ()>>>;

/// A task's value, once its body has returned it: taken by whoever
/// spawned it ([`Scheduler::spawn_join`], [`Cx::spawn_join`]), or awaited
/// by another task. Dropping the handle leaves the task running, and its
/// value is dropped when it comes.
pub struct JoinHandle<T> {
    id: TaskId,
    name: String,
    joined: Rc<RefCell<Joined<T>>>,
}

struct Joined<T> {
    value: Option<T>,
    finished: bool,
    /// The task awaiting the handle, if one is.
    waker: Option<Waker>,
}

impl<T> JoinHandle<T> {
    pub fn id(&self) -> TaskId {
        self.id
    }

    /// The task's name, as spawned.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Whether the task has returned.
    pub fn is_finished(&self) -> bool {
        self.joined.borrow().finished
    }

    /// The value, if the task has returned and it is not taken already.
    pub fn try_take(&mut self) -> Option<T> {
        self.joined.borrow_mut().value.take()
    }

    /// The value, for a test once the scheduler has run: a task that has
    /// not returned is a failure, named, rather than a result half made.
    #[track_caller]
    pub fn take(mut self) -> T {
        // Not in a closure, which `track_caller` does not see through:
        // the panic is at the test's line.
        let finished = self.is_finished();
        match self.try_take() {
            Some(v) => v,
            None => panic!(
                "the task {:?} (t{}) {}",
                self.name,
                self.id.0,
                if finished {
                    "gave its value already"
                } else {
                    "has not finished"
                }
            ),
        }
    }

    /// A body whose value goes to `joined`, waking whoever awaits it.
    async fn body<Fut: Future<Output = T>>(fut: Fut, joined: Rc<RefCell<Joined<T>>>) {
        let v = fut.await;
        let waker = {
            let mut j = joined.borrow_mut();
            j.value = Some(v);
            j.finished = true;
            j.waker.take()
        };
        if let Some(w) = waker {
            w.wake();
        }
    }

    fn new(id: TaskId, name: String, joined: Rc<RefCell<Joined<T>>>) -> JoinHandle<T> {
        JoinHandle { id, name, joined }
    }
}

fn joined<T>() -> Rc<RefCell<Joined<T>>> {
    Rc::new(RefCell::new(Joined {
        value: None,
        finished: false,
        waker: None,
    }))
}

/// Awaited from another task: the value, once the task returns.
impl<T> Future for JoinHandle<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        let mut j = self.joined.borrow_mut();
        if let Some(v) = j.value.take() {
            return Poll::Ready(v);
        }
        assert!(
            !j.finished,
            "the task {:?} (t{}) gave its value already",
            self.name, self.id.0
        );
        j.waker = Some(cx.waker().clone());
        Poll::Pending
    }
}

struct Task {
    name: String,
    priority: Priority,
    effect: Effect,
    future: TaskFuture,
    /// Between two suspension points, having been preempted.
    open: bool,
    /// The slices its section has stayed open, for hog reports.
    open_slices: u64,
    /// The priority of a more urgent task its open section blocks, which
    /// it runs at until the section closes.
    inherited: Option<Priority>,
    /// Its place in the runnable queue, if it is there: when it became
    /// runnable, as a sequence number.
    queued: Option<u64>,
    /// Whether it falls a class as it runs long (`Spawn::demotable`), the
    /// fuel counted against it (what it spent, less the time it waited on
    /// the world), how many classes it has fallen, and since when it has
    /// waited on the world, if it is waiting.
    demotable: bool,
    running: u64,
    fallen: u8,
    waiting_since: Option<u64>,
}

/// The live tasks, by id: ids are numbered from 0 as tasks are spawned, so
/// a task is found by indexing, not searching, and in id order when all are
/// visited. Slots before the oldest live task are let go, so a long session
/// keeps a slot for each task from its oldest live one on.
#[derive(Default)]
struct Tasks {
    slots: VecDeque<Option<Task>>,
    /// The id of `slots[0]`.
    base: u64,
    live: usize,
}

impl Tasks {
    fn slot(&self, id: &TaskId) -> Option<usize> {
        id.0.checked_sub(self.base).map(|i| i as usize)
    }

    fn insert(&mut self, id: TaskId, task: Task) {
        if self.live == 0 {
            self.slots.clear();
            self.base = id.0;
        }
        let i = self.slot(&id).expect("ids grow");
        if self.slots.len() <= i {
            self.slots.resize_with(i + 1, || None);
        }
        assert!(
            self.slots[i].replace(task).is_none(),
            "{id:?} spawned twice"
        );
        self.live += 1;
    }

    fn remove(&mut self, id: &TaskId) -> Option<Task> {
        let i = self.slot(id)?;
        let t = self.slots.get_mut(i)?.take();
        self.live -= usize::from(t.is_some());
        while let Some(None) = self.slots.front() {
            self.slots.pop_front();
            self.base += 1;
        }
        t
    }

    fn get(&self, id: &TaskId) -> Option<&Task> {
        self.slots.get(self.slot(id)?)?.as_ref()
    }

    fn get_mut(&mut self, id: &TaskId) -> Option<&mut Task> {
        let i = self.slot(id)?;
        self.slots.get_mut(i)?.as_mut()
    }

    fn len(&self) -> usize {
        self.live
    }

    fn values(&self) -> impl Iterator<Item = &Task> {
        self.slots.iter().flatten()
    }
}

impl std::ops::Index<&TaskId> for Tasks {
    type Output = Task;
    fn index(&self, id: &TaskId) -> &Task {
        self.get(id).expect("a live task")
    }
}

impl Task {
    /// The priority it runs at: its own, fallen as it has run long, or one
    /// it inherited, if more urgent.
    fn effective(&self) -> Priority {
        let own = self.own();
        match self.inherited {
            Some(p) => p.min(own),
            None => own,
        }
    }

    /// Its own priority, fallen as it has run long.
    fn own(&self) -> Priority {
        if self.priority >= DEMOTE_TO {
            return self.priority;
        }
        let at = (self.priority as u8 + self.fallen).min(DEMOTE_TO as u8);
        Priority::ALL[at as usize]
    }
}

/// What tasks reach through their [`Cx`] while they run.
struct Shared {
    current: Option<TaskId>,
    /// Fuel left in the current slice, and used so far.
    budget: u64,
    used: u64,
    /// Set by `charge` when the fuel runs out, and by `yield_now`.
    preempted: bool,
    yielded: bool,
    /// The slice ends waiting on the world: a completion or a timer, not
    /// another task. Only such a wait brings a demoted task back up.
    waits_out: bool,
    now: u64,
    /// Whether spending fuel moves the clock: virtual time
    /// (`ClockMode::Auto`), not wall time, which only the runner moves.
    fuel_is_time: bool,
    timers: BTreeMap<(u64, u64), TaskId>,
    timer_seq: u64,
    /// Delivered completions nobody has taken yet, and who waits for which.
    completions: FixedMap<u64, Vec<u8>>,
    waiters: FixedMap<u64, TaskId>,
    /// Completions no one will take ([`Cx::abandon`]), not yet delivered:
    /// dropped when they are.
    abandoned: std::collections::BTreeSet<u64>,
    spawns: Vec<(TaskId, Spawn, TaskFuture)>,
    /// Events recorded by tasks, and priority changes asked for, during a
    /// slice; both take effect when it ends.
    events: Vec<What>,
    priorities: Vec<(TaskId, Priority)>,
    /// Effects declared during a slice, taken on when it ends.
    declared: Vec<(TaskId, Effect)>,
    next_task: u64,
}

impl Shared {
    fn fresh_id(&mut self) -> TaskId {
        let id = TaskId(self.next_task);
        self.next_task += 1;
        id
    }
}

/// A task's handle on the scheduler.
#[derive(Clone)]
pub struct Cx {
    shared: Rc<RefCell<Shared>>,
}

/// Where completions from outside are posted. Posting may happen from any
/// thread; delivery happens only at a step, in the order the scheduler
/// takes them, and is recorded.
#[derive(Clone, Default)]
pub struct Inbox {
    queue: Arc<Mutex<VecDeque<Posted>>>,
}

/// A completion as posted: its token and its bytes.
type Posted = (u64, Vec<u8>);

impl Inbox {
    pub fn post(&self, token: u64, bytes: Vec<u8>) {
        self.queue
            .lock()
            .expect("inbox lock")
            .push_back((token, bytes));
    }

    fn take_all(&self) -> Vec<Posted> {
        self.queue.lock().expect("inbox lock").drain(..).collect()
    }
}

/// A delivery held back by stress until step `due`; `seq` keeps the order
/// deliveries were taken in among those due together.
struct Held {
    due: u64,
    seq: u64,
    token: u64,
    bytes: Vec<u8>,
}

struct TaskWaker {
    task: u64,
    woken: Arc<Mutex<Vec<u64>>>,
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.woken.lock().expect("wake lock").push(self.task);
    }
}

/// What one call of [`Scheduler::step`] did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// A task ran a slice.
    Ran(TaskId),
    /// Nothing could run, and no timer is set: every task left is waiting
    /// for a completion from outside, or there are none.
    Idle,
}

/// A running hash of the event stream, over each event's text: the
/// run's "unseed" (FoundationDB's, R3 §2.1), cheap enough to keep always.
/// Two runs of one seed have the same; a replay checks it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StreamHash {
    pub events: u64,
    pub hash: Fingerprint,
}

impl Default for StreamHash {
    fn default() -> StreamHash {
        StreamHash {
            events: 0,
            hash: Fingerprint(0, 0),
        }
    }
}

impl StreamHash {
    pub fn update(&mut self, e: &Event) {
        self.events += 1;
        self.hash = fingerprint(&(self.hash, e.to_string()));
    }
}

pub struct Scheduler<S: Sink> {
    config: Config,
    /// The stream so far, said if a test fails while this runs.
    report: Report,
    rng: Rng,
    sink: S,
    step: u64,
    tasks: Tasks,
    /// The tasks whose sections are open: few, so choosing a task scans
    /// these, not every task (`TODO.md` §28).
    open: BTreeSet<TaskId>,
    /// Runnable tasks: for each priority class they run at, by when they
    /// became runnable (a sequence number). So the most urgent class's
    /// first is found at once, and the order across classes is the
    /// sequence numbers' (`TODO.md` §28).
    runnable: [BTreeMap<u64, TaskId>; CLASSES],
    next_seq: u64,
    woken: Arc<Mutex<Vec<u64>>>,
    inbox: Inbox,
    /// Deliveries held back by stress.
    held: Vec<Held>,
    held_seq: u64,
    /// How far an autojump may go ([`Scheduler::limit_jumps`]).
    jump_limit: Option<u64>,
    shared: Rc<RefCell<Shared>>,
}

/// What a test that fails while a scheduler runs says (R3 §2.1,
/// FoundationDB's habit): what it would take to see the same run again,
/// the seed, the stress, how far it got and the stream's hash so far; the
/// test is deterministic, so running it again runs the same. Said once a
/// scheduler, when it is dropped in the panic, on stderr, which the test
/// harness shows with the failure, and its name.
struct Report {
    seed: u64,
    quantum: u64,
    stress: Option<Stress>,
    /// The step of the last event.
    step: u64,
    hash: StreamHash,
}

impl Drop for Report {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            return;
        }
        let stress = match &self.stress {
            Some(s) => format!("{s:?}"),
            None => "none".into(),
        };
        // The harness names the test, and how to run it again. Not
        // `eprintln!`: a panic while unwinding aborts, losing the first.
        let _ = writeln!(
            std::io::stderr(),
            "; the scheduler of seed {:#x} (stress: {stress}; quantum {}) was at step {}, \
             stream hash {:016x}{:016x} after {} events: the test, run again, runs it again",
            self.seed,
            self.quantum,
            self.step,
            self.hash.hash.0,
            self.hash.hash.1,
            self.hash.events,
        );
    }
}

impl<S: Sink> Scheduler<S> {
    pub fn new(config: Config, sink: S) -> Scheduler<S> {
        let rng = Rng::new(config.seed).split("sched");
        let fuel_is_time = config.clock == ClockMode::Auto;
        Scheduler {
            report: Report {
                seed: config.seed,
                quantum: config.quantum,
                stress: config.stress.clone(),
                step: 0,
                hash: StreamHash::default(),
            },
            config,
            rng,
            sink,
            step: 0,
            tasks: Tasks::default(),
            open: BTreeSet::new(),
            runnable: Default::default(),
            next_seq: 0,
            woken: Arc::default(),
            inbox: Inbox::default(),
            held: Vec::new(),
            held_seq: 0,
            jump_limit: None,
            shared: Rc::new(RefCell::new(Shared {
                current: None,
                budget: 0,
                used: 0,
                preempted: false,
                yielded: false,
                waits_out: false,
                now: 0,
                fuel_is_time,
                timers: BTreeMap::new(),
                timer_seq: 0,
                completions: FixedMap::default(),
                abandoned: Default::default(),
                waiters: FixedMap::default(),
                spawns: Vec::new(),
                events: Vec::new(),
                priorities: Vec::new(),
                declared: Vec::new(),
                next_task: 0,
            })),
        }
    }

    pub fn sink(&self) -> &S {
        &self.sink
    }

    pub fn into_sink(self) -> S {
        self.sink
    }

    pub fn inbox(&self) -> Inbox {
        self.inbox.clone()
    }

    /// The step counter: how many times a task has been chosen, or the
    /// scheduler found nothing to run.
    pub fn steps(&self) -> u64 {
        self.step
    }

    pub fn now(&self) -> u64 {
        self.shared.borrow().now
    }

    /// The tasks not yet finished.
    pub fn live_tasks(&self) -> usize {
        self.tasks.len()
    }

    /// How many completions are delivered and not yet taken.
    pub fn completions_held(&self) -> usize {
        self.shared.borrow().completions.len()
    }

    /// Spawns a task, from outside any task.
    pub fn spawn<F, Fut>(&mut self, spawn: Spawn, body: F) -> TaskId
    where
        F: FnOnce(Cx) -> Fut,
        Fut: Future<Output = ()> + 'static,
    {
        let cx = Cx {
            shared: self.shared.clone(),
        };
        let future: TaskFuture = Box::pin(body(cx));
        let id = self.shared.borrow_mut().fresh_id();
        self.add_task(id, spawn, future);
        id
    }

    /// Spawns a task whose value is wanted: its handle gives it.
    pub fn spawn_join<T, F, Fut>(&mut self, spawn: Spawn, body: F) -> JoinHandle<T>
    where
        T: 'static,
        F: FnOnce(Cx) -> Fut,
        Fut: Future<Output = T> + 'static,
    {
        let (name, j) = (spawn.name.clone(), joined());
        let into = j.clone();
        let id = self.spawn(spawn, move |cx| JoinHandle::body(body(cx), into));
        JoinHandle::new(id, name, j)
    }

    /// Changes a live task's priority.
    pub fn set_priority(&mut self, t: TaskId, p: Priority) {
        if let Some(task) = self.tasks.get_mut(&t)
            && task.priority != p
        {
            let was = task.effective();
            task.priority = p;
            self.reclass(t, was);
            self.record(What::Priority {
                task: t.0,
                priority: p as u8,
            });
        }
    }

    fn add_task(&mut self, id: TaskId, spawn: Spawn, future: TaskFuture) {
        self.record(What::Spawn {
            task: id.0,
            name: spawn.name.clone(),
            priority: spawn.priority as u8,
        });
        self.tasks.insert(
            id,
            Task {
                name: spawn.name,
                priority: spawn.priority,
                effect: spawn.effect,
                future,
                open: false,
                open_slices: 0,
                inherited: None,
                queued: None,
                demotable: spawn.demotable,
                running: 0,
                fallen: 0,
                waiting_since: None,
            },
        );
        self.enqueue(id);
    }

    /// The name a task was spawned with, while it lives.
    pub fn task_name(&self, id: TaskId) -> Option<&str> {
        self.tasks.get(&id).map(|t| t.name.as_str())
    }

    /// Puts a label in the event stream at the current step.
    /// Puts an event in the stream from outside the tasks: a runner's.
    pub fn note(&mut self, what: What) {
        self.record(what);
    }

    /// The earliest timer's deadline, if any.
    /// Autojumps (`ClockMode::Auto`) go no further than `at`, if given:
    /// where a runner will post something of its own, which a jump past
    /// would make come late.
    pub fn limit_jumps(&mut self, at: Option<u64>) {
        self.jump_limit = at;
    }

    pub fn next_deadline(&self) -> Option<u64> {
        self.shared
            .borrow()
            .timers
            .first_key_value()
            .map(|(&(d, _), _)| d)
    }

    /// Moves the clock forward to `now`, as a runner says: an input of the
    /// run, recorded. Never backward.
    pub fn advance_to(&mut self, now: u64) {
        let moved = {
            let mut sh = self.shared.borrow_mut();
            if now > sh.now {
                sh.now = now;
                true
            } else {
                false
            }
        };
        if moved {
            self.record(What::ClockSet { now });
        }
    }

    pub fn mark(&mut self, label: impl Into<String>) {
        self.record(What::Mark {
            label: label.into(),
        });
    }

    fn record(&mut self, what: What) {
        let e = Event {
            step: self.step,
            what,
        };
        self.report.hash.update(&e);
        self.report.step = e.step;
        self.sink.record(&e);
    }

    /// The hash of the event stream so far.
    pub fn stream_hash(&self) -> StreamHash {
        self.report.hash
    }

    /// Runs steps until [`Step::Idle`], or until `max_steps` have run.
    /// Returns how many slices ran.
    pub fn run(&mut self, max_steps: u64) -> u64 {
        let mut ran = 0;
        while ran < max_steps {
            match self.step() {
                Step::Ran(_) => ran += 1,
                Step::Idle => break,
            }
        }
        ran
    }

    /// One scheduling decision: deliver what is due, then choose a task and
    /// run one slice of it; or, if none may run, advance the clock to the
    /// next timer and try again.
    pub fn step(&mut self) -> Step {
        self.step += 1;
        self.deliver();
        self.fire_timers(false);
        self.collect_wakes();
        let chosen = match self.choose() {
            Some(t) => t,
            None => {
                let jump = self.config.clock == ClockMode::Auto;
                if !self.fire_timers(jump) && !self.release_held_now() {
                    return Step::Idle;
                }
                self.collect_wakes();
                match self.choose() {
                    Some(t) => t,
                    None => return Step::Idle,
                }
            }
        };
        self.run_slice(chosen);
        Step::Ran(chosen)
    }

    /// Takes what was posted to the inbox, holds it back under stress, and
    /// delivers what is due.
    fn deliver(&mut self) {
        for (token, bytes) in self.inbox.take_all() {
            self.record(What::Arrive {
                token,
                bytes: bytes.clone(),
            });
            let delay = match &self.config.stress {
                Some(s) => self.rng.below(s.max_delivery_delay + 1),
                None => 0,
            };
            self.held.push(Held {
                due: self.step + delay,
                seq: self.held_seq,
                token,
                bytes,
            });
            self.held_seq += 1;
        }
        self.held.sort_by_key(|h| (h.due, h.seq));
        while self.held.first().is_some_and(|h| h.due <= self.step) {
            let h = self.held.remove(0);
            self.complete(h.token, h.bytes);
        }
    }

    /// When nothing else can happen, held deliveries are not made to wait
    /// for steps that would only spin: they are released at once, in
    /// order, until one wakes a task. Releasing only the first would leave
    /// the scheduler idle, if no one waited for it, with an awaited one
    /// still held, which a runner cannot know of.
    fn release_held_now(&mut self) -> bool {
        if self.held.is_empty() {
            return false;
        }
        let before = self.runnable_len();
        while !self.held.is_empty() {
            let h = self.held.remove(0);
            self.complete(h.token, h.bytes);
            if self.runnable_len() > before {
                break;
            }
        }
        true
    }

    fn complete(&mut self, token: u64, bytes: Vec<u8>) {
        self.record(What::Deliver {
            token,
            len: bytes.len() as u64,
        });
        let waiter = {
            let mut sh = self.shared.borrow_mut();
            if !sh.abandoned.remove(&token) {
                sh.completions.insert(token, bytes);
            }
            sh.waiters.remove(&token)
        };
        if let Some(t) = waiter {
            self.make_runnable(t);
        }
    }

    fn collect_wakes(&mut self) {
        let woken: Vec<u64> = std::mem::take(&mut *self.woken.lock().expect("wake lock"));
        for t in woken {
            self.make_runnable(TaskId(t));
        }
    }

    fn make_runnable(&mut self, t: TaskId) {
        if self.tasks.get(&t).is_some_and(|task| task.queued.is_none()) {
            if let Some(what) = self.woke(t) {
                self.record(what);
            }
            self.enqueue(t);
        }
    }

    /// A demotable task waking from a wait on the world: the fuel counted
    /// against it falls by the time it waited, and it may rise back (what
    /// changed, to record). So a task that waits about as long as it
    /// computes stays near its own class, and one that computes far more
    /// falls, however often it waits: a loop that prints as it goes is
    /// answered at once, and waits for next to nothing.
    fn woke(&mut self, t: TaskId) -> Option<What> {
        let now = self.shared.borrow().now;
        let after = self.config.demote_after;
        let task = self.tasks.get_mut(&t).expect("live");
        let since = task.waiting_since.take()?;
        let own = task.own();
        task.running = task.running.saturating_sub(now.saturating_sub(since));
        task.fallen = (task.running / after).min(DEMOTE_TO as u64) as u8;
        let risen = task.own();
        (risen < own).then_some(What::Promote {
            task: t.0,
            priority: risen as u8,
        })
    }

    /// A demotable task's class, after a slice that `used` fuel and ended
    /// `end` (waiting on the world, if `waits_out`): one lower for each
    /// `demote_after` fuel counted against it. A wait on the world is
    /// counted when the task wakes (`woke`); a wait for another task (the
    /// collector, a query, a join) is part of running long, not a pause
    /// in it. What changed, to record.
    fn demote(&mut self, t: TaskId, end: SliceEnd, used: u64, waits_out: bool) -> Option<What> {
        let after = self.config.demote_after;
        let now = self.shared.borrow().now;
        let task = self.tasks.get_mut(&t).expect("still live");
        if !task.demotable || after == 0 {
            return None;
        }
        let (was, own) = (task.effective(), task.own());
        task.running += used;
        task.fallen = (task.running / after).min(DEMOTE_TO as u64) as u8;
        if end == SliceEnd::Suspended && waits_out {
            task.waiting_since = Some(now);
        }
        let fallen = task.own();
        self.reclass(t, was);
        (fallen > own).then_some(What::Demote {
            task: t.0,
            priority: fallen as u8,
        })
    }

    /// Puts live task `t` at the end of its class's queue.
    fn enqueue(&mut self, t: TaskId) {
        let seq = self.next_seq;
        self.next_seq += 1;
        let task = self.tasks.get_mut(&t).expect("a live task");
        task.queued = Some(seq);
        self.runnable[task.effective() as usize].insert(seq, t);
    }

    /// Takes queued task `t` out of the queue.
    fn dequeue(&mut self, t: TaskId) {
        let task = self.tasks.get_mut(&t).expect("a live task");
        let seq = task.queued.take().expect("queued");
        let was = self.runnable[task.effective() as usize].remove(&seq);
        assert_eq!(was, Some(t), "queued in its class");
    }

    /// Moves `t`, if it is queued, from class `was` to the one it runs at
    /// now, keeping its place in the order.
    fn reclass(&mut self, t: TaskId, was: Priority) {
        let Some(task) = self.tasks.get(&t) else {
            return;
        };
        let now = task.effective();
        if let Some(seq) = task.queued
            && now != was
        {
            let moved = self.runnable[was as usize].remove(&seq);
            assert_eq!(moved, Some(t), "queued in its class");
            self.runnable[now as usize].insert(seq, t);
        }
    }

    fn runnable_len(&self) -> usize {
        self.runnable.iter().map(BTreeMap::len).sum()
    }

    /// Every runnable task, in the order they became runnable.
    fn queue_order(&self) -> Vec<TaskId> {
        let mut all: Vec<(u64, TaskId)> = self
            .runnable
            .iter()
            .flat_map(|q| q.iter().map(|(&s, &t)| (s, t)))
            .collect();
        all.sort_unstable();
        all.into_iter().map(|(_, t)| t).collect()
    }

    /// Fires every timer that is due. If `jump`, and none is, first moves
    /// the clock to the earliest deadline (autojump, when nothing else can
    /// happen).
    fn fire_timers(&mut self, jump: bool) -> bool {
        let fired: Vec<TaskId> = {
            let mut sh = self.shared.borrow_mut();
            let Some((&(deadline, _), _)) = sh.timers.first_key_value() else {
                return false;
            };
            if deadline > sh.now {
                if !jump || self.jump_limit.is_some_and(|l| deadline > l) {
                    return false;
                }
                sh.now = deadline;
            }
            let now = sh.now;
            let mut fired = Vec::new();
            while let Some(entry) = sh.timers.first_entry() {
                if entry.key().0 > now {
                    break;
                }
                fired.push(entry.remove());
            }
            fired
        };
        let now = self.now();
        self.record(What::Clock { now });
        for t in fired {
            self.record(What::Timer { task: t.0 });
            self.make_runnable(t);
        }
        true
    }

    /// Priority inheritance (R7 §7, recommendation 6): each open section
    /// that keeps a runnable task from running, whose own priority is less
    /// urgent, runs at that task's priority, so that it reaches its next
    /// suspension point before less urgent work is chosen.
    fn inherit(&mut self) {
        let mut raise: Vec<(TaskId, Priority)> = Vec::new();
        for t in self.queue_order() {
            let (p, effect) = (self.tasks[&t].effective(), &self.tasks[&t].effect);
            for &u in &self.open {
                let task = &self.tasks[&u];
                if u != t && task.effective() > p && task.effect.interferes(effect) {
                    raise.push((u, p));
                }
            }
        }
        for (u, p) in raise {
            let task = self.tasks.get_mut(&u).expect("open");
            let was = task.effective();
            if was > p {
                task.inherited = Some(p);
                self.reclass(u, was);
                self.record(What::Inherit {
                    task: u.0,
                    priority: Some(p as u8),
                });
            }
        }
    }

    /// Whether `t` may run now: no other task's open section has an effect
    /// that interferes with `t`'s.
    fn may_run(&self, t: TaskId) -> bool {
        let effect = &self.tasks[&t].effect;
        self.open
            .iter()
            .all(|u| *u == t || !self.tasks[u].effect.interferes(effect))
    }

    fn choose(&mut self) -> Option<TaskId> {
        if !self.open.is_empty() {
            self.inherit();
        }
        if self.config.stress.is_none() {
            // The first task runnable of the most urgent class that may
            // run: with no section open, the first of the first class.
            let t = self.runnable.iter().find_map(|q| {
                q.values()
                    .copied()
                    .find(|&t| self.open.is_empty() || self.may_run(t))
            })?;
            self.dequeue(t);
            return Some(t);
        }
        let eligible: Vec<(usize, TaskId)> = self
            .queue_order()
            .into_iter()
            .enumerate()
            .filter(|&(_, t)| self.may_run(t))
            .collect();
        let best = eligible
            .iter()
            .map(|&(_, t)| self.tasks[&t].effective())
            .min()?;
        let pick = match self.config.stress.clone() {
            None => eligible
                .iter()
                .find(|&&(_, t)| self.tasks[&t].effective() == best)
                .copied(),
            Some(s) => {
                let pool: Vec<(usize, TaskId)> =
                    if self.rng.chance(s.ignore_priority.0, s.ignore_priority.1) {
                        eligible
                    } else {
                        eligible
                            .into_iter()
                            .filter(|&(_, t)| self.tasks[&t].effective() == best)
                            .collect()
                    };
                Some(pool[self.rng.below(pool.len() as u64) as usize])
            }
        };
        let (_, t) = pick?;
        self.dequeue(t);
        Some(t)
    }

    fn run_slice(&mut self, t: TaskId) {
        let budget = match &self.config.stress {
            Some(s) => 1 + self.rng.below(s.max_quantum),
            None => self.config.quantum,
        };
        {
            let mut sh = self.shared.borrow_mut();
            sh.current = Some(t);
            sh.budget = budget;
            sh.used = 0;
            sh.preempted = false;
            sh.yielded = false;
            sh.waits_out = false;
        }
        self.record(What::Run { task: t.0, budget });
        let waker = Waker::from(Arc::new(TaskWaker {
            task: t.0,
            woken: self.woken.clone(),
        }));
        let mut cx = Context::from_waker(&waker);
        let task = self.tasks.get_mut(&t).expect("chosen task exists");
        let poll = task.future.as_mut().poll(&mut cx);
        let (used, preempted, yielded, waits_out, spawns, events, priorities) = {
            let mut sh = self.shared.borrow_mut();
            sh.current = None;
            (
                sh.used,
                sh.preempted,
                sh.yielded,
                sh.waits_out,
                std::mem::take(&mut sh.spawns),
                std::mem::take(&mut sh.events),
                std::mem::take(&mut sh.priorities),
            )
        };
        let declared = std::mem::take(&mut self.shared.borrow_mut().declared);
        for (task, e) in declared {
            if let Some(tk) = self.tasks.get_mut(&task) {
                tk.effect = e;
                self.record(What::Declare { task: task.0 });
            }
        }
        for what in events {
            self.record(what);
        }
        let end = match poll {
            Poll::Ready(()) => SliceEnd::Finished,
            Poll::Pending if preempted => SliceEnd::Preempted,
            Poll::Pending if yielded => SliceEnd::Yielded,
            Poll::Pending => SliceEnd::Suspended,
        };
        let mut hog = None;
        let mut ended_inheritance = false;
        let mut rank = None;
        match end {
            SliceEnd::Finished => {
                self.tasks.remove(&t);
                self.open.remove(&t);
            }
            _ => {
                let task = self.tasks.get_mut(&t).expect("still live");
                task.open = end == SliceEnd::Preempted;
                if task.open {
                    self.open.insert(t);
                    task.open_slices += 1;
                    let n = task.open_slices;
                    if n >= HOG_SLICES && n.is_power_of_two() {
                        hog = Some(n);
                    }
                } else {
                    self.open.remove(&t);
                    task.open_slices = 0;
                    let was = task.effective();
                    ended_inheritance = task.inherited.take().is_some();
                    self.reclass(t, was);
                }
                rank = self.demote(t, end, used, waits_out);
            }
        }
        self.record(What::SliceEnd {
            task: t.0,
            used,
            end,
        });
        if let Some(slices) = hog {
            self.record(What::Hog { task: t.0, slices });
        }
        if ended_inheritance {
            self.record(What::Inherit {
                task: t.0,
                priority: None,
            });
        }
        if let Some(what) = rank {
            self.record(what);
        }
        for (id, spawn, future) in spawns {
            self.add_task(id, spawn, future);
        }
        for (task, priority) in priorities {
            self.set_priority(task, priority);
        }
        debug_assert!(self.open_set_does_not_interfere());
    }

    /// The scheduler's own invariant: no two open sections interfere.
    /// It also checks that the set of open tasks the scheduler chooses by
    /// is every task whose section is open, by visiting every task.
    pub fn open_sections_do_not_interfere(&self) -> bool {
        let flagged = self.tasks.values().filter(|t| t.open).count();
        flagged == self.open.len()
            && self.open.iter().all(|u| self.tasks[u].open)
            && self.open_set_does_not_interfere()
    }

    /// No two tasks of the open set interfere: after every slice, in a
    /// debug build, in time for the open tasks alone.
    fn open_set_does_not_interfere(&self) -> bool {
        let open: Vec<&Task> = self.open.iter().map(|u| &self.tasks[u]).collect();
        open.iter().enumerate().all(|(i, a)| {
            open[i + 1..]
                .iter()
                .all(|b| !a.effect.interferes(&b.effect))
        })
    }
}

impl Cx {
    /// Spends `n` fuel. If the slice's fuel runs out, the task is preempted
    /// here, and goes on paying in its next slices until `n` is spent.
    pub fn charge(&self, n: u64) -> Charge {
        Charge {
            cx: self.clone(),
            n,
        }
    }

    /// Gives the scheduler back: a suspension point, after which the task is
    /// runnable again.
    pub fn yield_now(&self) -> YieldNow {
        YieldNow {
            cx: self.clone(),
            done: false,
        }
    }

    /// The virtual clock.
    pub fn now(&self) -> u64 {
        self.shared.borrow().now
    }

    /// Waits until the clock reaches `deadline`: a suspension point.
    pub fn sleep_until(&self, deadline: u64) -> Sleep {
        Sleep {
            cx: self.clone(),
            deadline,
            timer: None,
        }
    }

    pub fn sleep(&self, ticks: u64) -> Sleep {
        let now = self.now();
        self.sleep_until(now + ticks)
    }

    /// Waits for the completion posted under `token`: a suspension point,
    /// unless it has been delivered already.
    pub fn completion(&self, token: u64) -> Completion {
        Completion {
            cx: self.clone(),
            token,
        }
    }

    /// Waits for the first of `tokens`' completions, or the clock to reach
    /// `deadline` if there is one: a suspension point, unless one has been
    /// delivered already, or the deadline has passed. The completion
    /// taken, with its token; none at the deadline. The rest are left for
    /// later, and the wait's registrations are taken back.
    pub fn first_completion(&self, tokens: Vec<u64>, deadline: Option<u64>) -> FirstCompletion {
        FirstCompletion {
            cx: self.clone(),
            tokens,
            deadline,
            timer: None,
            task: None,
        }
    }

    /// The completion under `token` will be taken by no one (a run gave
    /// up on its operation): dropped, now if it was delivered, or when it
    /// is, so that it is not kept for the rest of the session.
    pub fn abandon(&self, token: u64) {
        let mut sh = self.shared.borrow_mut();
        if sh.completions.remove(&token).is_none() {
            sh.abandoned.insert(token);
        }
    }

    /// Puts a label in the event stream.
    pub fn mark(&self, label: impl Into<String>) {
        self.record(What::Mark {
            label: label.into(),
        });
    }

    /// Puts an event in the stream, at the end of the current slice.
    pub fn record(&self, what: What) {
        self.shared.borrow_mut().events.push(what);
    }

    /// Spawns a task, which becomes runnable after the current slice.
    pub fn spawn<F, Fut>(&self, spawn: Spawn, body: F) -> TaskId
    where
        F: FnOnce(Cx) -> Fut,
        Fut: Future<Output = ()> + 'static,
    {
        let future: TaskFuture = Box::pin(body(self.clone()));
        let mut sh = self.shared.borrow_mut();
        let id = sh.fresh_id();
        sh.spawns.push((id, spawn, future));
        id
    }

    /// Spawns a task whose value is wanted, after the current slice: its
    /// handle gives it, or is awaited.
    pub fn spawn_join<T, F, Fut>(&self, spawn: Spawn, body: F) -> JoinHandle<T>
    where
        T: 'static,
        F: FnOnce(Cx) -> Fut,
        Fut: Future<Output = T> + 'static,
    {
        let (name, j) = (spawn.name.clone(), joined());
        let into = j.clone();
        let id = self.spawn(spawn, move |cx| JoinHandle::body(body(cx), into));
        JoinHandle::new(id, name, j)
    }

    /// Changes a task's priority, when the current slice ends.
    pub fn set_priority(&self, task: TaskId, priority: Priority) {
        self.shared.borrow_mut().priorities.push((task, priority));
    }

    /// The task this handle's slice is running.
    /// This task's effect, from now on, is `e`: a suspension point, at
    /// which the effect changes. While it is suspended no section of its
    /// own is open, so it may take on any effect; it resumes only once no
    /// open section interferes with `e`. So a task that first only asks
    /// queries (a REPL entry, before its forms are checked) runs as `pure`,
    /// and says what it does once it knows, before it does it.
    pub fn declare_effect(&self, e: Effect) -> YieldNow {
        {
            let mut sh = self.shared.borrow_mut();
            let t = sh.current.expect("a task is running");
            sh.declared.push((t, e));
        }
        self.yield_now()
    }

    pub fn current_task(&self) -> TaskId {
        self.current()
    }

    fn current(&self) -> TaskId {
        self.shared
            .borrow()
            .current
            .expect("a task's handle used outside its slice")
    }
}

/// Spends fuel; see [`Cx::charge`]. A charge larger than what is left in
/// the slice pays what it can, is preempted, and pays the rest in later
/// slices, so `charge(n)` always spends exactly `n`.
pub struct Charge {
    cx: Cx,
    /// What is still owed.
    n: u64,
}

impl Future for Charge {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let mut sh = self.cx.shared.borrow_mut();
        let pay = self.n.min(sh.budget);
        sh.budget -= pay;
        sh.used += pay;
        // Work takes time: the clock moves by the fuel spent, if the clock
        // is virtual.
        if sh.fuel_is_time {
            sh.now += pay;
        }
        let owed = self.n - pay;
        if owed == 0 {
            return Poll::Ready(());
        }
        sh.preempted = true;
        drop(sh);
        self.n = owed;
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

pub struct YieldNow {
    cx: Cx,
    done: bool,
}

impl Future for YieldNow {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.done {
            return Poll::Ready(());
        }
        self.done = true;
        self.cx.shared.borrow_mut().yielded = true;
        cx.waker().wake_by_ref();
        Poll::Pending
    }
}

/// A sleep, dropped before its deadline (an interrupt cut it short),
/// takes back its timer: none fires for a task that waits for it no
/// longer, nor moves the clock there.
pub struct Sleep {
    cx: Cx,
    deadline: u64,
    /// The timer set, by its key.
    timer: Option<(u64, u64)>,
}

impl Drop for Sleep {
    fn drop(&mut self) {
        if let Some(k) = self.timer.take() {
            self.cx.shared.borrow_mut().timers.remove(&k);
        }
    }
}

impl Future for Sleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<()> {
        let task = self.cx.current();
        let mut sh = self.cx.shared.borrow_mut();
        if sh.now >= self.deadline {
            return Poll::Ready(());
        }
        sh.waits_out = true;
        if self.timer.is_none() {
            let seq = sh.timer_seq;
            sh.timer_seq += 1;
            sh.timers.insert((self.deadline, seq), task);
            drop(sh);
            self.timer = Some((self.deadline, seq));
        }
        Poll::Pending
    }
}

pub struct FirstCompletion {
    cx: Cx,
    tokens: Vec<u64>,
    deadline: Option<u64>,
    /// The timer set, by its key.
    timer: Option<(u64, u64)>,
    /// The task registered, while it is.
    task: Option<TaskId>,
}

/// Dropped before it is done, a wait takes back its registrations.
impl Drop for FirstCompletion {
    fn drop(&mut self) {
        if let Some(task) = self.task.take() {
            let cx = self.cx.clone();
            let mut sh = cx.shared.borrow_mut();
            self.unregister(&mut sh, task);
        }
    }
}

impl FirstCompletion {
    /// The wait's registrations, taken back: a token's waiter, if it is
    /// still this task, and the timer.
    fn unregister(&mut self, sh: &mut Shared, task: TaskId) {
        for t in &self.tokens {
            if sh.waiters.get(t) == Some(&task) {
                sh.waiters.remove(t);
            }
        }
        if let Some(k) = self.timer.take() {
            sh.timers.remove(&k);
        }
    }
}

impl Future for FirstCompletion {
    type Output = Option<(u64, Vec<u8>)>;

    fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        let task = self.cx.current();
        let cx = self.cx.clone();
        let mut sh = cx.shared.borrow_mut();
        let found = self
            .tokens
            .iter()
            .find_map(|t| sh.completions.remove(t).map(|b| (*t, b)));
        if found.is_some() || self.deadline.is_some_and(|d| sh.now >= d) {
            self.unregister(&mut sh, task);
            self.task = None;
            return Poll::Ready(found);
        }
        for t in &self.tokens {
            sh.waiters.insert(*t, task);
        }
        sh.waits_out = true;
        self.task = Some(task);
        if let (Some(d), None) = (self.deadline, self.timer) {
            let seq = sh.timer_seq;
            sh.timer_seq += 1;
            sh.timers.insert((d, seq), task);
            self.timer = Some((d, seq));
        }
        Poll::Pending
    }
}

pub struct Completion {
    cx: Cx,
    token: u64,
}

impl Future for Completion {
    type Output = Vec<u8>;

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Vec<u8>> {
        let task = self.cx.current();
        let mut sh = self.cx.shared.borrow_mut();
        if let Some(bytes) = sh.completions.remove(&self.token) {
            return Poll::Ready(bytes);
        }
        sh.waiters.insert(self.token, task);
        sh.waits_out = true;
        Poll::Pending
    }
}
