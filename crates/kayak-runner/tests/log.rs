//! The log's format: every record reads back as written, and a record
//! changed in a way its framing does not show is caught by its check.
// coopt-source: crates/coopt-runner/tests/log.rs@9283bba

use kayak_base::Fingerprint;
use kayak_runner::log::{Record, config_of, header, read, settings};
use kayak_sched::{ClockMode, Config, Stress};

fn records() -> Vec<Record> {
    vec![
        Record::Arrive {
            step: 3,
            token: 1,
            kind: "read".into(),
            bytes: b"x".to_vec(),
        },
        Record::ClockSet {
            step: 300,
            now: u64::MAX,
        },
        Record::Checkpoint {
            step: 1 << 40,
            events: 128,
            hash: Fingerprint(1, u64::MAX),
        },
        Record::Event {
            step: 0,
            text: "     7 mark prompt".into(),
        },
        Record::End {
            outcome: "Returned".into(),
        },
    ]
}

fn log_of(rs: &[Record]) -> Vec<u8> {
    let mut b = header(&[("seed", "0"), ("commit", "abc")]);
    for r in rs {
        b.extend(r.encode());
    }
    b
}

#[test]
fn every_record_reads_back() {
    let l = read(&log_of(&records())).expect("a log");
    assert_eq!(l.records, records());
    assert_eq!(l.torn_at, None);
    assert_eq!(l.header.get("commit").map(String::as_str), Some("abc"));
}

/// The key `x` turned into `y`: the record still parses, and only its
/// check says it changed.
#[test]
fn a_changed_payload_fails_its_check() {
    let mut b = log_of(&records()[..1]);
    let at = b.len() - 5;
    assert_eq!(b[at], b'x');
    b[at] = b'y';
    let l = read(&b).expect("a log");
    assert!(l.records.is_empty(), "{:?}", l.records);
    assert!(l.torn_at.is_some());
}

#[test]
fn another_version_is_refused() {
    let mut b = log_of(&[]);
    b[b"coopt-log\n".len()] = 9;
    assert!(read(&b).unwrap_err().contains("version 9"));
    assert!(read(b"not a log").is_err());
}

/// The scheduler's settings read back from a header as written; a field
/// missing, as from an older log, is the default (but demotion, which an
/// older log's run did without); one garbled is refused.
#[test]
fn settings_read_back() {
    let fields = |c: &Config| -> std::collections::BTreeMap<String, String> {
        settings(c)
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect()
    };
    for c in [
        Config::default(),
        Config {
            seed: 7,
            quantum: 3,
            stress: Some(Stress::heavy()),
            clock: ClockMode::Runner,
            demote_after: 7,
        },
    ] {
        let back = config_of(&fields(&c)).unwrap();
        assert_eq!(settings(&back), settings(&c));
    }
    // A log from before demotion ran without it; else, the defaults.
    let none = config_of(&Default::default()).unwrap();
    assert_eq!(none.demote_after, 0);
    assert_eq!(
        settings(&none),
        settings(&Config {
            demote_after: 0,
            ..Config::default()
        })
    );
    let mut garbled = fields(&Config::default());
    garbled.insert("stress".into(), "16 1/4".into());
    assert!(config_of(&garbled).is_err());
}
