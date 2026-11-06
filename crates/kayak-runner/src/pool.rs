//! Worker threads for jobs (`TODO.md` §48, stage 1): every runner runs a
//! task's [`Job`]s on them, so that a sim or a replay uses the machine's
//! cores as a live run does. When a job's completion is delivered is the
//! runner's to say, not the pool's: a live runner delivers it when it is
//! done, a sim at a step it chooses, waiting for it if need be.
//!
//! Threads are started as jobs wait with none free, up to a bound, so a
//! run with no jobs starts none.
// coopt-source: crates/coopt-runner/src/pool.rs@9283bba

use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

use kayak_sched::Job;
use kayak_sched::job::Ran;

/// What a job gave: its value's fingerprint, and its bytes if they were
/// kept; or what its panic said.
pub type Done = Ran;

pub struct Pool {
    shared: Arc<Shared>,
    /// Detached when the pool is dropped.
    threads: Vec<JoinHandle<()>>,
    max: usize,
}

struct Shared {
    state: Mutex<State>,
    /// A job came, or the pool is ending.
    work: Condvar,
    /// A job finished.
    finished: Condvar,
    /// Called on a worker as each job finishes: a live runner's waker.
    on_done: Option<Box<dyn Fn() + Send + Sync>>,
}

#[derive(Default)]
struct State {
    /// Each job, and whether to keep its value's bytes.
    queue: VecDeque<(u64, Job, bool)>,
    done: BTreeMap<u64, Done>,
    /// Submitted, and not yet taken.
    open: usize,
    idle: usize,
    ending: bool,
}

impl Pool {
    /// A pool of up to `max` threads.
    pub fn new(max: usize, on_done: Option<Box<dyn Fn() + Send + Sync>>) -> Pool {
        Pool {
            shared: Arc::new(Shared {
                state: Mutex::default(),
                work: Condvar::new(),
                finished: Condvar::new(),
                on_done,
            }),
            threads: Vec::new(),
            max: max.max(1),
        }
    }

    /// As many threads as the machine has cores.
    pub fn per_core(on_done: Option<Box<dyn Fn() + Send + Sync>>) -> Pool {
        let n = std::thread::available_parallelism().map_or(1, |n| n.get());
        Pool::new(n, on_done)
    }

    /// Runs `job` under `token`, keeping its value's bytes if `keep`.
    pub fn submit(&mut self, token: u64, job: Job, keep: bool) {
        let mut st = self.shared.state.lock().expect("the pool");
        st.queue.push_back((token, job, keep));
        st.open += 1;
        let start = st.queue.len() > st.idle && self.threads.len() < self.max;
        drop(st);
        self.shared.work.notify_one();
        if start {
            let shared = self.shared.clone();
            let n = self.threads.len();
            let t = std::thread::Builder::new()
                .name(format!("coopt-job-{n}"))
                .spawn(move || work(&shared))
                .expect("a worker thread");
            self.threads.push(t);
        }
    }

    /// Jobs submitted and not yet taken.
    pub fn open(&self) -> usize {
        self.shared.state.lock().expect("the pool").open
    }

    /// Job `token`'s result, waiting for it if it is still running.
    pub fn wait(&self, token: u64) -> Done {
        let mut st = self.shared.state.lock().expect("the pool");
        loop {
            if let Some(d) = st.done.remove(&token) {
                st.open -= 1;
                return d;
            }
            st = self.shared.finished.wait(st).expect("the pool");
        }
    }

    /// Every job finished since the last time, by token.
    pub fn take_done(&self) -> Vec<(u64, Done)> {
        let mut st = self.shared.state.lock().expect("the pool");
        let done = std::mem::take(&mut st.done);
        st.open -= done.len();
        done.into_iter().collect()
    }
}

fn work(shared: &Shared) {
    let mut st = shared.state.lock().expect("the pool");
    loop {
        if let Some((token, job, keep)) = st.queue.pop_front() {
            drop(st);
            let d = job.run_keeping(keep);
            shared.state.lock().expect("the pool").done.insert(token, d);
            shared.finished.notify_all();
            if let Some(f) = &shared.on_done {
                f();
            }
            st = shared.state.lock().expect("the pool");
        } else if st.ending {
            return;
        } else {
            st.idle += 1;
            st = shared.work.wait(st).expect("the pool");
            st.idle -= 1;
        }
    }
}

impl Drop for Pool {
    /// Jobs still queued are dropped, and those running left to finish on
    /// their own: no one will take their values, and a run that ends (a
    /// ^C, say) must not wait for them.
    fn drop(&mut self) {
        let mut st = self.shared.state.lock().expect("the pool");
        st.ending = true;
        st.queue.clear();
        drop(st);
        self.shared.work.notify_all();
    }
}
