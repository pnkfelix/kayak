//! The replay log (`docs/research/runners.md` §3).
//!
//! A run's only inputs are what the scheduler takes from its inbox, and
//! the clock moves a runner makes; everything else follows from them and
//! the configuration. So the log holds those, and checkpoints of a hash of
//! the event stream; outputs are not kept, they are checked through the
//! hash. A "full" log keeps every event too, for exact comparison.
//!
//! ```text
//! header:  b"coopt-log\n" · version · (key, value)… · empty key
//! record:  kind (u8) · body length · body · check (u32)
//!   1 Arrive      step · token · op kind · bytes
//!   2 ClockSet    step · now
//!   3 Checkpoint  step · events so far · stream hash (two u64)
//!   4 Event       step · the event, as text
//!   5 End         how the run ended, as text
//! ```
//!
//! Numbers are LEB128 varints; strings and byte strings are length
//! prefixed. Steps are absolute, not the deltas R3 suggested: a byte or
//! two more a record, for records that are each readable alone. The check
//! is the low half of a fingerprint of the kind and the body: a reader
//! stops at the first record that fails it, or that is cut short, so the
//! log of a run that crashed is usable up to its last whole record.
//! Records are written as the events happen, an arrival before it is
//! delivered.
// coopt-source: crates/coopt-runner/src/log.rs@9283bba

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use kayak_base::{Fingerprint, fingerprint};
use kayak_sched::{ClockMode, Config, Stress};
use kayak_trace::{Event, Sink, What};

pub const MAGIC: &[u8] = b"coopt-log\n";
pub const VERSION: u64 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Record {
    Arrive {
        step: u64,
        token: u64,
        kind: String,
        bytes: Vec<u8>,
    },
    ClockSet {
        step: u64,
        now: u64,
    },
    Checkpoint {
        step: u64,
        events: u64,
        hash: Fingerprint,
    },
    Event {
        step: u64,
        text: String,
    },
    End {
        outcome: String,
    },
}

fn put_varint(out: &mut Vec<u8>, mut n: u64) {
    loop {
        let b = (n & 0x7f) as u8;
        n >>= 7;
        if n == 0 {
            out.push(b);
            return;
        }
        out.push(b | 0x80);
    }
}

fn put_bytes(out: &mut Vec<u8>, b: &[u8]) {
    put_varint(out, b.len() as u64);
    out.extend_from_slice(b);
}

struct Cursor<'a> {
    b: &'a [u8],
    at: usize,
}

impl Cursor<'_> {
    fn varint(&mut self) -> Option<u64> {
        let mut n = 0u64;
        for shift in (0..64).step_by(7) {
            let b = *self.b.get(self.at)?;
            self.at += 1;
            n |= u64::from(b & 0x7f) << shift;
            if b & 0x80 == 0 {
                return Some(n);
            }
        }
        None
    }
    fn bytes(&mut self) -> Option<&[u8]> {
        let n = self.varint()? as usize;
        let s = self.b.get(self.at..self.at + n)?;
        self.at += n;
        Some(s)
    }
    fn string(&mut self) -> Option<String> {
        self.bytes().map(|b| String::from_utf8_lossy(b).into())
    }
    fn u64(&mut self) -> Option<u64> {
        let s = self.b.get(self.at..self.at + 8)?;
        self.at += 8;
        Some(u64::from_le_bytes(s.try_into().ok()?))
    }
}

fn check(kind: u8, body: &[u8]) -> u32 {
    fingerprint(&(u64::from(kind), body.to_vec())).0 as u32
}

impl Record {
    /// The record, framed.
    pub fn encode(&self) -> Vec<u8> {
        let mut body = Vec::new();
        let kind = match self {
            Record::Arrive {
                step,
                token,
                kind,
                bytes,
            } => {
                put_varint(&mut body, *step);
                put_varint(&mut body, *token);
                put_bytes(&mut body, kind.as_bytes());
                put_bytes(&mut body, bytes);
                1
            }
            Record::ClockSet { step, now } => {
                put_varint(&mut body, *step);
                put_varint(&mut body, *now);
                2
            }
            Record::Checkpoint { step, events, hash } => {
                put_varint(&mut body, *step);
                put_varint(&mut body, *events);
                body.extend_from_slice(&hash.0.to_le_bytes());
                body.extend_from_slice(&hash.1.to_le_bytes());
                3
            }
            Record::Event { step, text } => {
                put_varint(&mut body, *step);
                put_bytes(&mut body, text.as_bytes());
                4
            }
            Record::End { outcome } => {
                put_bytes(&mut body, outcome.as_bytes());
                5
            }
        };
        let mut out = vec![kind];
        put_bytes(&mut out, &body);
        out.extend_from_slice(&check(kind, &body).to_le_bytes());
        out
    }

    fn decode(kind: u8, body: &[u8]) -> Option<Record> {
        let mut c = Cursor { b: body, at: 0 };
        let r = match kind {
            1 => Record::Arrive {
                step: c.varint()?,
                token: c.varint()?,
                kind: c.string()?,
                bytes: c.bytes()?.to_vec(),
            },
            2 => Record::ClockSet {
                step: c.varint()?,
                now: c.varint()?,
            },
            3 => Record::Checkpoint {
                step: c.varint()?,
                events: c.varint()?,
                hash: Fingerprint(c.u64()?, c.u64()?),
            },
            4 => Record::Event {
                step: c.varint()?,
                text: c.string()?,
            },
            5 => Record::End {
                outcome: c.string()?,
            },
            _ => return None,
        };
        (c.at == body.len()).then_some(r)
    }
}

/// A log, read.
#[derive(Clone, Debug, Default)]
pub struct Log {
    pub header: BTreeMap<String, String>,
    pub records: Vec<Record>,
    /// Where a torn or corrupt record stopped the reading, if one did.
    pub torn_at: Option<usize>,
}

/// The header: magic, version, and what the run was started with.
pub fn header(fields: &[(&str, &str)]) -> Vec<u8> {
    let mut out = MAGIC.to_vec();
    put_varint(&mut out, VERSION);
    for (k, v) in fields {
        put_bytes(&mut out, k.as_bytes());
        put_bytes(&mut out, v.as_bytes());
    }
    put_bytes(&mut out, b"");
    out
}

/// The scheduler's settings, as header fields, so that a replay runs with
/// the ones the run had: `seed`, `quantum`, `clock` (`auto` or `runner`),
/// and `stress`, `none` or `MAX-QUANTUM N/D MAX-DELAY` (`16 1/4 8`).
pub fn settings(c: &Config) -> Vec<(&'static str, String)> {
    let stress = match &c.stress {
        None => "none".to_string(),
        Some(s) => format!(
            "{} {}/{} {}",
            s.max_quantum, s.ignore_priority.0, s.ignore_priority.1, s.max_delivery_delay
        ),
    };
    let clock = match c.clock {
        ClockMode::Auto => "auto",
        ClockMode::Runner => "runner",
    };
    vec![
        ("seed", c.seed.to_string()),
        ("quantum", c.quantum.to_string()),
        ("stress", stress),
        ("clock", clock.to_string()),
        ("demote", c.demote_after.to_string()),
    ]
}

/// The settings a header gives ([`settings`]); a field it lacks, as an
/// older log's do, is the default.
pub fn config_of(header: &BTreeMap<String, String>) -> Result<Config, String> {
    let mut c = Config::default();
    let bad = |k: &str, v: &str| format!("the log's {k} is {v:?}");
    let num = |k: &str, v: &str| v.parse::<u64>().map_err(|_| bad(k, v));
    if let Some(v) = header.get("seed") {
        c.seed = num("seed", v)?;
    }
    if let Some(v) = header.get("quantum") {
        c.quantum = num("quantum", v)?;
    }
    if let Some(v) = header.get("clock") {
        c.clock = match v.as_str() {
            "auto" => ClockMode::Auto,
            "runner" => ClockMode::Runner,
            _ => return Err(bad("clock", v)),
        };
    }
    // A log from before demotion ran without it.
    c.demote_after = match header.get("demote") {
        Some(v) => num("demote", v)?,
        None => 0,
    };
    if let Some(v) = header.get("stress").filter(|v| *v != "none") {
        let parts: Vec<&str> = v.split([' ', '/']).collect();
        let [q, n, d, delay] = parts[..] else {
            return Err(bad("stress", v));
        };
        c.stress = Some(Stress {
            max_quantum: num("stress", q)?,
            ignore_priority: (num("stress", n)?, num("stress", d)?),
            max_delivery_delay: num("stress", delay)?,
        });
    }
    Ok(c)
}

/// Reads a log, as far as its records are whole and check.
pub fn read(b: &[u8]) -> Result<Log, String> {
    let rest = b.strip_prefix(MAGIC).ok_or("not a coopt log")?;
    let mut c = Cursor { b: rest, at: 0 };
    let version = c.varint().ok_or("a log with no version")?;
    if version != VERSION {
        return Err(format!(
            "a log of format version {version}; this build reads {VERSION}"
        ));
    }
    let mut log = Log::default();
    loop {
        let k = c.string().ok_or("a torn header")?;
        if k.is_empty() {
            break;
        }
        let v = c.string().ok_or("a torn header")?;
        log.header.insert(k, v);
    }
    while c.at < rest.len() {
        let start = c.at;
        let whole = (|| {
            let kind = *c.b.get(c.at)?;
            c.at += 1;
            let body = c.bytes()?.to_vec();
            let sum = c.b.get(c.at..c.at + 4)?;
            c.at += 4;
            let sum = u32::from_le_bytes(sum.try_into().ok()?);
            if sum != check(kind, &body) {
                return None;
            }
            Record::decode(kind, &body)
        })();
        match whole {
            Some(r) => log.records.push(r),
            None => {
                log.torn_at = Some(MAGIC.len() + start);
                break;
            }
        }
    }
    Ok(log)
}

pub use kayak_sched::StreamHash;

/// A sink that writes the log as the run goes, and passes every event on
/// to `inner`.
pub struct Recorder<S: Sink> {
    pub inner: S,
    out: Rc<RefCell<Vec<u8>>>,
    /// Keep every event, not only the inputs and checkpoints.
    full: bool,
    /// A checkpoint after this many events; 0 for none.
    every: u64,
    pub hash: StreamHash,
    /// The kind of each operation performed, by token, for its arrival.
    kinds: BTreeMap<u64, String>,
}

impl<S: Sink> Recorder<S> {
    /// A recorder appending to `out`, which should hold a [`header`].
    pub fn new(inner: S, out: Rc<RefCell<Vec<u8>>>, full: bool, every: u64) -> Recorder<S> {
        Recorder {
            inner,
            out,
            full,
            every,
            hash: StreamHash::default(),
            kinds: BTreeMap::new(),
        }
    }

    fn write(&self, r: Record) {
        self.out.borrow_mut().extend_from_slice(&r.encode());
    }
}

impl<S: Sink> Sink for Recorder<S> {
    fn record(&mut self, e: &Event) {
        self.inner.record(e);
        self.hash.update(e);
        match &e.what {
            What::Perform { token, op } => {
                let kind = op.split(' ').next().unwrap_or("").to_string();
                self.kinds.insert(*token, kind);
            }
            What::Arrive { token, bytes } => {
                let kind = self.kinds.remove(token).unwrap_or_default();
                self.write(Record::Arrive {
                    step: e.step,
                    token: *token,
                    kind,
                    bytes: bytes.clone(),
                })
            }
            What::ClockSet { now } => self.write(Record::ClockSet {
                step: e.step,
                now: *now,
            }),
            _ => {}
        }
        if self.full {
            self.write(Record::Event {
                step: e.step,
                text: e.to_string(),
            });
        }
        if self.every > 0 && self.hash.events.is_multiple_of(self.every) {
            self.write(Record::Checkpoint {
                step: e.step,
                events: self.hash.events,
                hash: self.hash.hash,
            });
        }
    }
}
