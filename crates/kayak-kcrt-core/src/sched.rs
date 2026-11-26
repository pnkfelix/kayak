//! `%sched` (C544): what FX-26's processes and `par` need of the runtime,
//! as Fixpt's runtime has it (`fixpt_runtime::sched`, ported; tier 1's
//! `cells/sched.kfx` is the same in KFX). The scheduler itself is FX-26,
//! `fx26:processes.fx`, which kc compiles with the program: a task is a
//! thunk run under the scheduler's prompt, which waits by capturing the
//! rest of itself (a one-shot continuation, C459) and aborting. Here is
//! only what the tasks share: the run queue, the I-cells' and mailboxes'
//! waiters, timers, the tasks cancelled, monitored groups, and messages
//! copied.
//!
//! Fixpt keeps its state in a vector in its heap; here it is Rust's, a
//! thread's, each value it holds (a task's thunk, a waker, a handler, the
//! prompt's tag) a word the collector takes as an exact root (`slots`,
//! `kayak_heap::roots::runtime_slots`) and rewrites where its object
//! moves. What is in the heap is Fixpt's: an I-cell (kind `icell`, its
//! fields whether it is full, its value, its waiters) and a mailbox (kind
//! `mailbox`, its messages oldest first then newest first, its waiter),
//! each a bloblet with a trailer, their lists kc's pairs.
//!
//! Each operation that makes something in the heap takes its room first,
//! all it needs at once, and reads its operands after, from the stack
//! words the collection rewrote (C455), as the other routines do; the
//! state is never borrowed across that.

use std::cell::RefCell;
use std::collections::VecDeque;

use super::{
    KIND_BLOBLET, KIND_PRODUCT, KIND_STRING, KIND_SUM, TAG_BLOBLET, TAG_HEADER, TAG_MASK, TAG_PAIR,
    TAG_TRAILER, arg, boolean, fixnum, heap, kind, null, pair, set, word,
};

const KIND_ICELL: u64 = kayak_layout::kind("icell") as u64;
const KIND_MAILBOX: u64 = kayak_layout::kind("mailbox") as u64;
const KIND_BYTEVECTOR: u64 = kayak_layout::kind("bytevector") as u64;
const KIND_FLAT_ARRAY: u64 = kayak_layout::kind("flat-array") as u64;

/// An I-cell's fields, and a mailbox's, by Fixpt's numbering: field `k`
/// the word `k` before the value's suffix (field 1 the trailer).
const ICELL_FULL: u64 = 2;
const ICELL_VALUE: u64 = 3;
const ICELL_WAITERS: u64 = 4;
const MB_FRONT: u64 = 2;
const MB_BACK: u64 = 3;
const MB_WAITER: u64 = 4;

/// kc's own operation, past Fixpt's: `(make-icell)`, an I-cell, empty
/// (Fixpt's is a primitive of its own, `%fx26-make-icell`).
pub(crate) const MAKE_ICELL: i64 = 100;
/// And `(clock)`: nanoseconds since the run began (Fixpt's `clock_ns`).
pub(crate) const CLOCK: i64 = 101;
/// And `(sleep-until t)`: until the clock reaches `t`; unit.
pub(crate) const SLEEP_UNTIL: i64 = 102;

pub(crate) fn unit() -> u64 {
    (super::IMM_UNIT << super::IMM_SHIFT) | super::TAG_IMMEDIATE
}

fn falsity() -> u64 {
    boolean(false)
}

#[derive(Default)]
struct Sched {
    /// Tasks to run, oldest first.
    queue: VecDeque<u64>,
    /// The scheduler's prompt tag, once made; 0 before.
    tag: u64,
    /// Waker run when the clock reaches the time (`clock_ns`), in the
    /// order they were set.
    timers: Vec<(i64, u64)>,
    cancelled: Vec<i64>,
    /// The scheduler's `preempt`; 0 if none.
    preempt: u64,
    /// Tasks stopped at a poll, oldest first.
    parked: Vec<u64>,
    running: i64,
    /// Each task of a monitored group, and its group.
    groups: Vec<(i64, i64)>,
    /// Each group's handler.
    monitors: Vec<(i64, u64)>,
    /// Each `race` branch's group, and the group it is inside.
    parents: Vec<(i64, i64)>,
    /// Each task that waits, and how it stops waiting.
    waiting: Vec<(i64, u64)>,
    /// Which tasks live (task 0, the first, always).
    live: Vec<bool>,
    current: usize,
    /// Each task's quiet count (how deep it is in what may not be
    /// preempted) and, bit 7, its grace (Fixpt's `Regions::quiet`).
    quiet: Vec<u8>,
    /// Words copied into messages.
    copied: i64,
    /// A failure caught (`task_failed`): its message, and its group's
    /// handler, for the scheduler to call (`TAKE_FAILURE`).
    failure: Option<(String, u64)>,
}

impl Sched {
    /// Each word of the heap it holds, by its slot's address.
    fn slots(&self, f: &mut dyn FnMut(usize)) {
        let (a, b) = self.queue.as_slices();
        for w in a.iter().chain(b) {
            f(std::ptr::from_ref(w).addr());
        }
        for w in [&self.tag, &self.preempt] {
            f(std::ptr::from_ref(w).addr());
        }
        for (_, w) in &self.timers {
            f(std::ptr::from_ref(w).addr());
        }
        for w in &self.parked {
            f(std::ptr::from_ref(w).addr());
        }
        for (_, w) in &self.monitors {
            f(std::ptr::from_ref(w).addr());
        }
        for (_, w) in &self.waiting {
            f(std::ptr::from_ref(w).addr());
        }
        if let Some((_, h)) = &self.failure {
            f(std::ptr::from_ref(h).addr());
        }
    }

    fn task_live(&self, t: i64) -> bool {
        t == 0 || (t > 0 && self.live.get(t as usize).copied().unwrap_or(false))
    }

    fn quiet_mut(&mut self) -> &mut u8 {
        let t = self.current;
        if self.quiet.len() <= t {
            self.quiet.resize(t + 1, 0);
        }
        &mut self.quiet[t]
    }

    fn quiet_enter(&mut self) {
        let q = self.quiet_mut();
        *q = (*q & 0x80) | ((*q & 0x7f) + 1).min(0x7f);
    }

    /// Out of what may not be preempted; out of the last, a grace begins.
    fn quiet_leave(&mut self) {
        let q = self.quiet_mut();
        let d = (*q & 0x7f).saturating_sub(1);
        *q = d | if d == 0 { 0x80 } else { *q & 0x80 };
    }

    fn task_quiet(&self) -> bool {
        self.quiet.get(self.current).is_some_and(|q| q & 0x7f != 0)
    }

    fn take_grace(&mut self) -> bool {
        let q = self.quiet_mut();
        let g = *q & 0x80 != 0;
        *q &= 0x7f;
        g
    }

    fn group_of(&self, t: i64) -> Option<i64> {
        self.groups.iter().find(|&&(u, _)| u == t).map(|&(_, g)| g)
    }

    fn forget(&mut self, t: i64) {
        self.cancelled.retain(|&c| c != t);
        self.groups.retain(|&(u, _)| u != t);
        self.monitors.retain(|&(u, _)| u != t);
        self.parents.retain(|&(a, b)| a != t && b != t);
        self.waiting.retain(|&(u, _)| u != t);
    }

    /// Task `t` ended, not the first; if it is current, the first is
    /// (Fixpt's `Heap::task_end`).
    fn end(&mut self, t: i64) {
        if t > 0 && self.task_live(t) {
            self.live[t as usize] = false;
            if self.current == t as usize {
                self.current = 0;
            }
            self.forget(t);
        }
    }
}

std::thread_local! {
    static SCHED: RefCell<Sched> = RefCell::new(Sched::default());
}

/// The state's values, the collector's exact roots (registered with the
/// heap at a run's start, `reset`).
fn slots(f: &mut dyn FnMut(usize)) {
    SCHED.with(|s| {
        if let Ok(s) = s.try_borrow() {
            s.slots(f);
        } else {
            panic!("kc: a collection while the scheduler's state is borrowed");
        }
    });
}

/// A run's start: no task but the first, nothing queued (Fixpt's
/// `sched::reset`); the state's values roots of this thread's heap.
pub(crate) fn reset() {
    SCHED.with(|s| *s.borrow_mut() = Sched::default());
    kayak_heap::roots::runtime_slots(Some(slots));
}

fn with<R>(f: impl FnOnce(&mut Sched) -> R) -> R {
    SCHED.with(|s| f(&mut s.borrow_mut()))
}

/// Nanoseconds since the run began, Fixpt's `clock_ns` (which counts from
/// its process's start too), a fixnum's for years.
fn clock_ns() -> i64 {
    static START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    START
        .get_or_init(std::time::Instant::now)
        .elapsed()
        .as_nanos() as i64
}

fn is_kind(v: u64, k: u64) -> bool {
    v & TAG_MASK == TAG_BLOBLET && kind(v) == k
}

/// Field `k` of bloblet `v` (Fixpt's `bloblet_slot`), and its address.
fn slot_at(v: u64, k: u64) -> u64 {
    v - TAG_BLOBLET - 8 * k
}

fn slot(v: u64, k: u64) -> u64 {
    word(slot_at(v, k))
}

/// Field `k` of `v` set, through the heap's barrier: `v` may be old.
fn set_slot(v: u64, k: u64, x: u64) {
    let a = slot_at(v, k);
    set(a, x);
    heap::remember(a);
}

/// Room taken at once, made into objects from its start.
struct Room(u64);

impl Room {
    fn get(block: *mut u64, words: u64) -> Result<Room, String> {
        Ok(Room(if words == 0 {
            0
        } else {
            heap::words(block, words)?
        }))
    }

    fn take(&mut self, n: u64) -> u64 {
        let at = self.0;
        self.0 += 8 * n;
        at
    }

    fn cons(&mut self, a: u64, d: u64) -> u64 {
        let at = self.take(2);
        set(at, a);
        set(at + 8, d);
        at + TAG_PAIR
    }

    /// A bloblet of kind `k` with a trailer, its fields `fs` from field 2
    /// on (as kc's code makes one, x64.kfx's `x64-emit-bloblet`).
    fn bloblet(&mut self, k: u64, fs: &[u64]) -> u64 {
        let n = fs.len() as u64;
        let at = self.take(n + 2);
        set(at, TAG_HEADER | (k << 3) | ((n + 1) << 14));
        for (i, &f) in fs.iter().enumerate() {
            set(at + 8 * (n - i as u64), f);
        }
        set(at + 8 * (n + 1), TAG_TRAILER | ((n + 1) << 3));
        at + 8 * (n + 2) + TAG_BLOBLET
    }

    fn list(&mut self, xs: &[u64]) -> u64 {
        let mut l = null();
        for &x in xs.iter().rev() {
            l = self.cons(x, l);
        }
        l
    }
}

fn list_items(mut l: u64) -> Vec<u64> {
    let mut v = Vec::new();
    while l & TAG_MASK == TAG_PAIR {
        let (a, d) = pair(l);
        v.push(a);
        l = d;
    }
    v
}

fn symbol(name: &str) -> u64 {
    super::kc_intern(super::make_string(name))
}

/// `%sched`'s operation `n` on `a` and `b`, its operands the stack words
/// `args` (`n`, `a`, `b`), in kc's heap where it makes something.
pub(crate) fn op(args: *const u64, block: *mut u64) -> Result<u64, String> {
    kayak_heap::roots::runtime_slots(Some(slots));
    let n = (arg(args, 0) as i64) >> 3;
    let a = arg(args, 1);
    let b = arg(args, 2);
    let task = (a as i64) >> 3;
    match n {
        1 | 2 | 24 | 35 | 36 | 39 | 40 if a & TAG_MASK != 0 || !with(|s| s.task_live(task)) => {
            return Err(format!("%fx26-sched: no such task (op {n})"));
        }
        6..=9 | 17 | 23 if n != 6 && !is_kind(a, KIND_MAILBOX) => {
            return Err("expected a mailbox".to_string());
        }
        10 | 11 | 14 | 15 | 22 | 34 if !is_kind(a, KIND_ICELL) => {
            return Err("expected an i-cell".to_string());
        }
        _ => {}
    }
    Ok(match n {
        // TASK_NEW: the lowest number free.
        0 => with(|s| {
            let t = (1..s.live.len())
                .find(|&i| !s.live[i])
                .unwrap_or(s.live.len().max(1));
            if t >= s.live.len() {
                s.live.resize(t + 1, false);
            }
            s.live[t] = true;
            if s.quiet.len() <= t {
                s.quiet.resize(t + 1, 0);
            }
            s.quiet[t] = 0;
            fixnum(t as i64)
        }),
        // TASK_SWITCH: the task before.
        1 => with(|s| {
            let before = s.current;
            s.current = task as usize;
            fixnum(before as i64)
        }),
        2 => {
            with(|s| s.end(task));
            unit()
        }
        3 => fixnum(with(|s| s.current) as i64),
        4 => {
            with(|s| s.queue.push_back(a));
            unit()
        }
        5 => with(|s| s.queue.pop_front()).unwrap_or_else(falsity),
        6 => {
            let mut r = Room::get(block, 5)?;
            r.bloblet(KIND_MAILBOX, &[null(), null(), falsity()])
        }
        7 => mailbox_put(args, block)?,
        8 => mailbox_take(args, block)?,
        // MAILBOX_WAIT: run at once if a message is there.
        9 => {
            if slot(a, MB_FRONT) & TAG_MASK == TAG_PAIR || slot(a, MB_BACK) & TAG_MASK == TAG_PAIR {
                with(|s| s.queue.push_back(b));
            } else {
                set_slot(a, MB_WAITER, b);
            }
            unit()
        }
        10 => slot(a, ICELL_FULL),
        11 => {
            if slot(a, ICELL_FULL) != falsity() {
                with(|s| s.queue.push_back(b));
            } else {
                let mut r = Room::get(block, 2)?;
                let (c, w) = (arg(args, 1), arg(args, 2));
                let ws = r.cons(w, slot(c, ICELL_WAITERS));
                set_slot(c, ICELL_WAITERS, ws);
            }
            unit()
        }
        // TAG: `(sum some tag)`, or `(sum none #u)` before one is made.
        12 => {
            let (some, none) = (symbol("some"), symbol("none"));
            let mut r = Room::get(block, 4)?;
            let t = with(|s| s.tag);
            if t == 0 {
                r.bloblet(KIND_SUM, &[none, unit()])
            } else {
                r.bloblet(KIND_SUM, &[some, t])
            }
        }
        13 => {
            with(|s| s.tag = a);
            unit()
        }
        14 => icell_put(args, block)?,
        15 => slot(a, ICELL_VALUE),
        // REAP_ENTER: the task's place, which kc's code has none of at run
        // time (a place is unit, `kc/front.kfx`'s `letrena`).
        16 => unit(),
        17 => boolean(
            slot(a, MB_FRONT) & TAG_MASK == TAG_PAIR || slot(a, MB_BACK) & TAG_MASK == TAG_PAIR,
        ),
        18 => {
            let t = (a as i64) >> 3;
            if clock_ns() >= t {
                with(|s| s.queue.push_back(b));
            } else {
                with(|s| s.timers.push((t, b)));
            }
            unit()
        }
        19 => boolean(clock_ns() >= (a as i64) >> 3),
        20 => {
            with(|s| s.timers.retain(|&(_, w)| w != a));
            unit()
        }
        21 => timers_wait(true),
        22 => {
            let ws = list_items(slot(a, ICELL_WAITERS));
            if ws.contains(&b) {
                let mut r = Room::get(block, 2 * (ws.len() as u64 - 1))?;
                let (c, w) = (arg(args, 1), arg(args, 2));
                let kept: Vec<u64> = list_items(slot(c, ICELL_WAITERS))
                    .into_iter()
                    .filter(|&x| x != w)
                    .collect();
                let l = r.list(&kept);
                set_slot(c, ICELL_WAITERS, l);
            }
            unit()
        }
        23 => {
            if slot(a, MB_WAITER) == b {
                set_slot(a, MB_WAITER, falsity());
            }
            unit()
        }
        24 => {
            if task != 0 {
                with(|s| s.cancelled.push(task));
            }
            unit()
        }
        25 => boolean(with(|s| s.cancelled.contains(&task))),
        26 => {
            with(|s| s.preempt = a);
            unit()
        }
        27 => {
            with(|s| s.parked.push(a));
            unit()
        }
        28 => with(|s| {
            s.queue
                .pop_front()
                .or_else(|| (!s.parked.is_empty()).then(|| s.parked.remove(0)))
        })
        .unwrap_or_else(falsity),
        29 => {
            with(|s| s.running = (s.running + if a & TAG_MASK == 0 { task } else { 0 }).max(0));
            unit()
        }
        30 => timers_wait(false),
        // PREEMPT_WANTED: kc's code is not preempted yet.
        31 => falsity(),
        32 => {
            with(|s| s.quiet_leave());
            unit()
        }
        33 => {
            with(|s| s.quiet_enter());
            unit()
        }
        34 => {
            if slot(a, ICELL_FULL) != falsity() {
                unit()
            } else {
                icell_put(args, block)?
            }
        }
        35 | 39 => {
            with(|s| {
                if n == 39
                    && let Some(p) = s.group_of(s.current as i64)
                {
                    s.parents.push((task, p));
                }
                s.groups.push((task, task));
                s.monitors.push((task, b));
            });
            unit()
        }
        36 => {
            with(|s| {
                if let Some(g) = s.group_of(s.current as i64) {
                    s.groups.push((task, g));
                }
            });
            unit()
        }
        // TAKE_FAILURE: the failure `task_failed` caught, if any, `(message
        // . handler)`, the current task, which failed, ended; else `#f`.
        37 => {
            let n = with(|s| s.failure.as_ref().map(|(m, _)| m.chars().count()));
            match n {
                None => falsity(),
                Some(n) => {
                    let mut r = Room::get(block, kayak_heap::text::string_words(n) as u64 + 2)?;
                    let (m, h) = with(|s| {
                        let f = s.failure.take().expect("a failure");
                        let t = s.current as i64;
                        s.end(t);
                        f
                    });
                    let ps: Vec<u32> = m.chars().map(|c| c as u32).collect();
                    let at = r.take(kayak_heap::text::string_words(n) as u64);
                    let m = super::write_string(at, &ps);
                    r.cons(m, h)
                }
            }
        }
        38 => return Err("kc: not yet: a task's failure caught by its group".to_string()),
        40 => {
            with(|s| s.waiting.push((task, b)));
            unit()
        }
        41 => {
            with(|s| s.waiting.retain(|&(t, _)| t != task));
            unit()
        }
        // REAP: each cancelled task that waits, `(task . stop)`, taken.
        42 => {
            let k = with(|s| {
                s.waiting
                    .iter()
                    .filter(|(t, _)| s.cancelled.contains(t))
                    .count()
            }) as u64;
            let mut r = Room::get(block, 4 * k)?;
            let reaped: Vec<(i64, u64)> = with(|s| {
                let (out, keep): (Vec<_>, Vec<_>) =
                    s.waiting.iter().partition(|(t, _)| s.cancelled.contains(t));
                s.waiting = keep;
                out
            });
            let es: Vec<u64> = reaped.iter().map(|&(t, w)| r.cons(fixnum(t), w)).collect();
            r.list(&es)
        }
        43..=45 => return Err("kc: not yet: a task that waits for the standard input".to_string()),
        46..=54 => return Err("kc: not yet: a task that waits for a process".to_string()),
        55 => {
            let ms = ((a as i64) >> 3).max(0);
            fixnum(clock_ns().saturating_add(ms.saturating_mul(1_000_000)))
        }
        CLOCK => fixnum(clock_ns()),
        SLEEP_UNTIL => {
            let t = (a as i64) >> 3;
            let now = clock_ns();
            if t > now {
                std::thread::sleep(std::time::Duration::from_nanos((t - now) as u64));
            }
            unit()
        }
        MAKE_ICELL => {
            let mut r = Room::get(block, 5)?;
            r.bloblet(KIND_ICELL, &[falsity(), falsity(), null()])
        }
        _ => return Err(format!("%fx26-sched: no such operation {n}")),
    })
}

/// I-cell `c` given `v`, its one write; the tasks waiting for it run, in
/// the order they began to wait.
fn icell_put(args: *const u64, _block: *mut u64) -> Result<u64, String> {
    let (c, v) = (arg(args, 1), arg(args, 2));
    if slot(c, ICELL_FULL) != falsity() {
        return Err("an i-cell written twice".to_string());
    }
    set_slot(c, ICELL_VALUE, v);
    set_slot(c, ICELL_FULL, boolean(true));
    let ws = list_items(slot(c, ICELL_WAITERS));
    set_slot(c, ICELL_WAITERS, null());
    with(|s| {
        for &w in ws.iter().rev() {
            s.queue.push_back(w);
        }
    });
    Ok(unit())
}

/// When no task can run and the first waits: `#f` if no timer is set;
/// else, having slept until the earliest (if `sleep`), each due taken, its
/// waker run, in the order of their times, and `#t`.
fn timers_wait(sleep: bool) -> u64 {
    let earliest = with(|s| s.timers.iter().map(|&(t, _)| t).min());
    let Some(t) = earliest else {
        return falsity();
    };
    if sleep {
        let now = clock_ns();
        if t > now {
            std::thread::sleep(std::time::Duration::from_nanos((t - now) as u64));
        }
    }
    let now = clock_ns();
    with(|s| {
        let (mut due, keep): (Vec<_>, Vec<_>) = s.timers.iter().partition(|&&(t, _)| t <= now);
        s.timers = keep;
        due.sort_by_key(|&(t, _)| t);
        for (_, w) in due {
            s.queue.push_back(w);
        }
    });
    boolean(true)
}

/// Whether a message shares `v` with its sender rather than copies it:
/// what is never written, procedures, and the capabilities (Fixpt's
/// `shared`).
fn shared(v: u64) -> bool {
    if v & TAG_MASK == TAG_PAIR {
        return false;
    }
    if v & TAG_MASK != TAG_BLOBLET {
        return true;
    }
    let k = kind(v);
    ![
        KIND_STRING,
        KIND_BYTEVECTOR,
        KIND_BLOBLET,
        KIND_SUM,
        KIND_PRODUCT,
        KIND_FLAT_ARRAY,
    ]
    .contains(&k)
}

/// The words of `v`'s copy, and of what it holds that is copied too, each
/// object once (`seen`).
fn copy_words(v: u64, seen: &mut std::collections::HashSet<u64>) -> u64 {
    if shared(v) || !seen.insert(v) {
        return 0;
    }
    if v & TAG_MASK == TAG_PAIR {
        let (a, d) = pair(v);
        return 2 + copy_words(a, seen) + copy_words(d, seen);
    }
    let (h, o) = object_of(v);
    let mut n = (o.end - o.main) as u64 / 8;
    for k in 1..=o.traced as u64 {
        n += copy_words(word(h + 8 * k), seen);
    }
    n
}

/// A bloblet's first word (its header's address) and its extent.
fn object_of(v: u64) -> (u64, kayak_heap::Object) {
    let p = v - TAG_BLOBLET;
    let before = word(p - 8);
    let h = if before & TAG_MASK == TAG_TRAILER {
        p - 8 - 8 * (before >> 3)
    } else {
        let mut h = p - 8;
        while word(h) & TAG_MASK != TAG_HEADER {
            h -= 8;
        }
        h
    };
    (h, kayak_heap::object_at(h as usize, |a| word(a as u64)))
}

/// `v` copied into room `r`, what it holds that is not shared copied too,
/// each object once (`done`).
fn copied(v: u64, r: &mut Room, done: &mut std::collections::HashMap<u64, u64>) -> u64 {
    if shared(v) {
        return v;
    }
    if let Some(&c) = done.get(&v) {
        return c;
    }
    if v & TAG_MASK == TAG_PAIR {
        let at = r.take(2);
        done.insert(v, at + TAG_PAIR);
        let (a, d) = pair(v);
        let (a, d) = (copied(a, r, done), copied(d, r, done));
        set(at, a);
        set(at + 8, d);
        return at + TAG_PAIR;
    }
    let (h, o) = object_of(v);
    let n = (o.end - o.main) as u64 / 8;
    let at = r.take(n);
    for k in 0..n {
        set(at + 8 * k, word(h + 8 * k));
    }
    let c = at + (v - h);
    done.insert(v, c);
    for k in 1..=o.traced as u64 {
        let f = copied(word(h + 8 * k), r, done);
        set(at + 8 * k, f);
    }
    c
}

/// MAILBOX_PUT: the message copied, at the back; the receiver, if it
/// waits, run.
fn mailbox_put(args: *const u64, block: *mut u64) -> Result<u64, String> {
    let words = copy_words(arg(args, 2), &mut std::collections::HashSet::new());
    let mut r = Room::get(block, words + 2)?;
    let (mb, msg) = (arg(args, 1), arg(args, 2));
    let msg = copied(msg, &mut r, &mut std::collections::HashMap::new());
    with(|s| s.copied += words as i64);
    let back = r.cons(msg, slot(mb, MB_BACK));
    set_slot(mb, MB_BACK, back);
    let w = slot(mb, MB_WAITER);
    if w != falsity() {
        set_slot(mb, MB_WAITER, falsity());
        with(|s| s.queue.push_back(w));
    }
    Ok(unit())
}

/// MAILBOX_TAKE: the oldest message, taken; the back turned to the front
/// first if the front is empty.
fn mailbox_take(args: *const u64, block: *mut u64) -> Result<u64, String> {
    let a = arg(args, 1);
    if slot(a, MB_FRONT) & TAG_MASK != TAG_PAIR {
        let back = list_items(slot(a, MB_BACK));
        if back.is_empty() {
            return Err("%fx26-sched: an empty mailbox taken from".to_string());
        }
        let mut r = Room::get(block, 2 * back.len() as u64)?;
        let a = arg(args, 1);
        let mut back = list_items(slot(a, MB_BACK));
        back.reverse();
        let front = r.list(&back);
        set_slot(a, MB_FRONT, front);
        set_slot(a, MB_BACK, null());
    }
    let a = arg(args, 1);
    let (m, rest) = pair(slot(a, MB_FRONT));
    set_slot(a, MB_FRONT, rest);
    Ok(m)
}

/// The current task failed with `message` (C544; Fixpt's
/// `sched::task_failed`): if it is of a monitored group (a monitored
/// process's, a `race` branch's, an `attempt`'s), and the scheduler's
/// prompt is on kc's stack (its prompts' chain, the state block's word
/// 664, has its tag), the failure is the group's, not the run's: the rest
/// of the group cancelled, and every group inside it; the failure kept for
/// the scheduler to give the group's handler (`TAKE_FAILURE`); and the
/// prompt's record given back (its tag the scheduler's), for kc's code to
/// abort to with unit, as a task that waits does. Else none, and the run
/// fails.
pub(crate) fn task_failed(message: &str, state: *const u64) -> Option<u64> {
    SCHED.with(|s| {
        let mut s = s.try_borrow_mut().ok()?;
        let cur = s.current as i64;
        if cur == 0 || s.tag == 0 || s.failure.is_some() {
            return None;
        }
        let g = s.group_of(cur)?;
        let h = s
            .monitors
            .iter()
            .rev()
            .find(|&&(t, _)| t == g)
            .map(|&(_, h)| h)?;
        let at = prompted(s.tag, state)?;
        let mut groups = vec![g];
        loop {
            let before = groups.len();
            for &(a, b) in &s.parents {
                if groups.contains(&b) && !groups.contains(&a) {
                    groups.push(a);
                }
            }
            if groups.len() == before {
                break;
            }
        }
        let others: Vec<i64> = s
            .groups
            .iter()
            .filter(|&&(t, g)| groups.contains(&g) && t != cur)
            .map(|&(t, _)| t)
            .collect();
        s.cancelled.extend(others);
        s.failure = Some((message.to_string(), h));
        while s.task_quiet() {
            s.quiet_leave();
        }
        Some(at)
    })
}

/// The innermost record of a prompt of `tag` on kc's stack, if any: its
/// prompts' chain, from the state block's word 664, each record its tag at
/// 8 (x64.kfx's `x64-emit-prompt`; `R_TAG`).
fn prompted(tag: u64, state: *const u64) -> Option<u64> {
    if state.is_null() {
        return None;
    }
    // SAFETY: the state kc's code runs on, given by its shim, its word 664
    // the innermost prompt's record or 0.
    let mut r = unsafe { *state.add(664 / 8) };
    while r != 0 {
        if word(r + super::R_TAG) == tag {
            return Some(r);
        }
        r = word(r + super::R_PREV);
    }
    None
}

/// At a poll whose slice is spent (C544; Fixpt's `sched::preempt_due`):
/// the scheduler's `preempt`, for kc's code to call there and go on, if
/// another task could run and preempting is safe here; else 0. The first
/// task is preempted if another is queued or parked, or a deadline is due,
/// and it is not running one already; another task, if the scheduler's
/// prompt is on its stack, which `preempt` captures up to. Marked quiet
/// until `preempt_done`, so that the scheduler's own code is not
/// preempted meanwhile, nor at the next slice's end.
pub(crate) fn preempt_due(state: *const u64) -> u64 {
    SCHED.with(|s| {
        let Ok(mut s) = s.try_borrow_mut() else {
            return 0;
        };
        if s.preempt == 0 || s.task_quiet() || s.take_grace() {
            return 0;
        }
        let wanted = if s.current == 0 {
            let now = clock_ns();
            let runnable = !s.queue.is_empty()
                || !s.parked.is_empty()
                || s.timers.iter().any(|&(t, _)| t <= now);
            runnable && s.running == 0
        } else {
            s.tag != 0 && prompted(s.tag, state).is_some()
        };
        if !wanted {
            return 0;
        }
        s.quiet_enter();
        s.preempt
    })
}

/// A call of `preempt` at a poll returned: the task may be preempted again,
/// after a slice of its own.
pub(crate) fn preempt_done() {
    with(|s| s.quiet_leave());
}
