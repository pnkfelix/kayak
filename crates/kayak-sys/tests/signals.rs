//! A signal is an event, with no handler (R4 §3). Its own test binary, so
//! its own process: watching SIGINT ignores it in this process. Without
//! libtest's harness, so that `watch` comes before any other thread, as it
//! must on Linux.
// coopt-source: crates/coopt-sys/tests/signals.rs@9283bba

use std::time::Duration;

use kayak_sys::{Event, Poller, Sig};

fn main() {
    let mut p = Poller::new().unwrap();
    p.watch(&[Sig::Int]).unwrap();
    // Sent by the system's `kill` command, so the test needs no `unsafe`.
    let pid = std::process::id().to_string();
    let status = std::process::Command::new("kill")
        .args(["-INT", &pid])
        .status()
        .unwrap();
    assert!(status.success());
    let mut out = Vec::new();
    for _ in 0..50 {
        p.wait(Some(Duration::from_millis(100)), &mut out).unwrap();
        if !out.is_empty() {
            break;
        }
    }
    assert!(
        matches!(
            out[..],
            [Event::Signal {
                sig: Sig::Int,
                count: 1
            }]
        ),
        "{out:?}"
    );
}
