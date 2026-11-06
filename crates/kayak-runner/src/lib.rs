//! Runners (`PLAN.md` §3.3; `docs/research/runners.md`, R3): the program
//! performs operations, a runner answers them, and every answer reaches a
//! task through the scheduler's inbox, at a step.
//!
//! - [`io`]: operations as data, and [`io::Io::perform`];
//! - [`runner`]: the [`runner::Runner`] trait, [`runner::Sim`] and
//!   [`runner::Replay`], and [`runner::drive`], the one loop for all;
//! - [`log`]: the replay log, written by [`log::Recorder`], a sink of the
//!   event stream.
//!
//! This is the one runtime crate allowed to reach the outside world
//! (`coopt-tidy`, "outside-world"); the live runner (M8) will, through
//! `kayak-sys`.
// coopt-source: crates/coopt-runner/src/lib.rs@9283bba

pub mod io;
pub mod live;
pub mod log;
mod net;
pub mod pool;
pub mod runner;
pub mod values;

pub use io::{Answer, Fd, Io, IoHub, Op, STDIN, STDOUT};
pub use runner::{Faults, KeyFaults, Kill, Outcome, Replay, Runner, Sim, drive};
