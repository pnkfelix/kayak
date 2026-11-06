//! Raw mode on a pseudo-terminal: entered, and restored by a panic
//! (R4, recommendation 5), not only by the guard's drop.
// coopt-source: crates/coopt-sys/tests/raw.rs@9283bba

use std::os::fd::AsFd;

use kayak_sys::{RawMode, canonical, openpty};

#[test]
fn a_panic_restores_the_terminal() {
    let (_leader, follower) = openpty().expect("a pty");
    assert!(canonical(follower.as_fd()).unwrap());
    let raw = RawMode::enter(follower.as_fd()).expect("raw mode");
    assert!(!canonical(follower.as_fd()).unwrap(), "raw");
    // A panic elsewhere, the guard still held: the hook restores it.
    let panicked = std::thread::spawn(|| panic!("a panic, on purpose")).join();
    assert!(panicked.is_err());
    assert!(
        canonical(follower.as_fd()).unwrap(),
        "restored by the panic"
    );
    std::mem::forget(raw);
}
