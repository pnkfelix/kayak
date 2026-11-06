//! The wrappers, against the system: pipes and wakes. Signals are in
//! their own test binary (`signals.rs`), since watching one changes the
//! process's dispositions.
// coopt-source: crates/coopt-sys/tests/sys.rs@9283bba

use std::os::fd::AsFd;
use std::time::Duration;

use kayak_sys::{Event, Interest, Kind, Poller, kind, pipe, raise_fd_limit, read, write};

#[test]
fn a_pipe_with_data_is_ready_once() {
    let (r, w) = pipe().unwrap();
    let mut p = Poller::new().unwrap();
    p.arm(r.as_fd(), Interest::Read, 7).unwrap();
    let mut out = Vec::new();
    p.wait(Some(Duration::ZERO), &mut out).unwrap();
    assert!(out.is_empty(), "nothing written yet: {out:?}");
    write(w.as_fd(), b"hello").unwrap();
    p.wait(Some(Duration::from_secs(5)), &mut out).unwrap();
    assert!(
        matches!(
            out[..],
            [Event::Io {
                key: 7,
                interest: Interest::Read,
                ..
            }]
        ),
        "{out:?}"
    );
    // One-shot: not reported again until armed again.
    out.clear();
    p.wait(Some(Duration::ZERO), &mut out).unwrap();
    assert!(out.is_empty(), "{out:?}");
    let mut buf = [0u8; 16];
    assert_eq!(read(r.as_fd(), &mut buf).unwrap(), 5);
    assert_eq!(&buf[..5], b"hello");
    assert_eq!(kind(r.as_fd()).unwrap(), Kind::Pipe);
}

#[test]
fn a_closed_writer_is_eof() {
    let (r, w) = pipe().unwrap();
    drop(w);
    let mut p = Poller::new().unwrap();
    p.arm(r.as_fd(), Interest::Read, 1).unwrap();
    let mut out = Vec::new();
    p.wait(Some(Duration::from_secs(5)), &mut out).unwrap();
    assert!(
        matches!(
            out[..],
            [Event::Io {
                key: 1,
                eof: true,
                ..
            }]
        ),
        "{out:?}"
    );
    let mut buf = [0u8; 4];
    assert_eq!(read(r.as_fd(), &mut buf).unwrap(), 0);
}

#[test]
fn a_waker_wakes_from_another_thread() {
    let mut p = Poller::new().unwrap();
    let w = p.waker();
    let t = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(20));
        w.wake().unwrap();
    });
    let mut out = Vec::new();
    p.wait(Some(Duration::from_secs(5)), &mut out).unwrap();
    t.join().unwrap();
    assert_eq!(out, [Event::Wake]);
}

#[test]
fn a_regular_file_is_one() {
    let f = std::fs::File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")).unwrap();
    assert_eq!(kind(f.as_fd()).unwrap(), Kind::Regular);
}

#[test]
fn the_fd_limit_goes_up_or_stays() {
    let n = raise_fd_limit().unwrap();
    assert!(n >= 256, "{n}");
    assert_eq!(
        raise_fd_limit().unwrap(),
        n,
        "raising again changes nothing"
    );
}
