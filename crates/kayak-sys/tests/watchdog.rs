//! The ^C watchdog (`watch_int`). Its own test binary, so its own process:
//! it watches SIGINT, which every test here would see. Without libtest's
//! harness, so that SIGINT is watched before any other thread, as it must
//! be on Linux.
// coopt-source: crates/coopt-sys/tests/watchdog.rs@9283bba

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::time::Duration;

use kayak_sys::{Event, Poller, Sig};

fn sigint() {
    let pid = std::process::id().to_string();
    let status = std::process::Command::new("kill")
        .args(["-INT", &pid])
        .status()
        .unwrap();
    assert!(status.success());
}

/// Waits on `p`, the caller's poller, for one SIGINT.
fn take(p: &mut Poller) {
    let mut out = Vec::new();
    for _ in 0..100 {
        p.wait(Some(Duration::from_millis(100)), &mut out).unwrap();
        if out.iter().any(|e| {
            matches!(
                e,
                Event::Signal {
                    sig: Sig::Int,
                    count: 1
                }
            )
        }) {
            return;
        }
    }
    panic!("the caller saw no SIGINT: {out:?}");
}

/// The caller still sees each ^C (on Linux, handed on by the watchdog);
/// a ^C the caller took, then another, is no reason to force; a second
/// one the caller did not take is.
fn main() {
    // As the live runner: SIGINT watched (and so ignored) first.
    let mut p = Poller::new().unwrap();
    p.watch(&[Sig::Int, Sig::Term]).unwrap();
    let taken = Arc::new(AtomicU64::new(0));
    let (tx, rx) = mpsc::channel();
    kayak_sys::watch_int(&mut p, taken.clone(), move || tx.send(()).unwrap()).unwrap();
    sigint();
    take(&mut p);
    taken.fetch_add(1, Ordering::SeqCst);
    sigint();
    assert!(
        rx.recv_timeout(Duration::from_millis(500)).is_err(),
        "forced with every ^C but one taken"
    );
    sigint();
    assert!(
        rx.recv_timeout(Duration::from_secs(10)).is_ok(),
        "two ^Cs not taken, and not forced"
    );
}
