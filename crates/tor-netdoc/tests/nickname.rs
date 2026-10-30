//! Relay nickname parsing, formatting, ordering and hashing contracts.
use std::hash::{Hash, Hasher};

use tor_netdoc::types::Nickname;

#[test]
fn nickname_values() {
    for input in [
        "a",
        "0",
        "Z",
        "Foo",
        "Untr1gonometr1cally",
        "0123456789abcdefghi",
    ] {
        let nickname: Nickname = input.parse().unwrap();
        assert_eq!(nickname.as_str(), input);
        assert_eq!(nickname.as_ref(), input);
        assert_eq!(nickname.to_string(), input);
        assert_eq!(format!("{nickname:?}"), format!("Nickname({input:?})"));
        let encoded = serde_json::to_string(input).unwrap();
        assert_eq!(serde_json::to_string(&nickname).unwrap(), encoded);
        assert_eq!(
            serde_json::from_str::<Nickname>(&encoded).unwrap(),
            nickname
        );
    }
    for input in [
        "",
        "abcdefghijklmnopqrst",
        "a b",
        " foo",
        "foo ",
        "a-b",
        "a_b",
        "a.b",
        "a\0b",
        "\0",
        "é",
        "🦀",
        "a\u{301}",
        "０",
        "Α",
    ] {
        assert!(input.parse::<Nickname>().is_err(), "{input:?}");
        let encoded = serde_json::to_string(input).unwrap();
        assert!(
            serde_json::from_str::<Nickname>(&encoded).is_err(),
            "{input:?}"
        );
    }
}

#[test]
fn nickname_order() {
    let mut names: Vec<Nickname> = ["z", "A0", "Z", "a", "Aa", "Foo", "B", "A", "9", "AZ", "0"]
        .into_iter()
        .map(|name| name.parse().unwrap())
        .collect();
    names.sort();
    assert_eq!(
        names.iter().map(Nickname::as_str).collect::<Vec<_>>(),
        ["0", "9", "A", "A0", "AZ", "Aa", "B", "Foo", "Z", "a", "z"]
    );
    assert_eq!(names[2], "A".parse().unwrap());
    assert_ne!(names[2], "a".parse().unwrap());
}

#[derive(Debug, PartialEq, Eq)]
enum HashWrite {
    Bytes(Vec<u8>),
    Byte(u8),
    Length(usize),
}

#[derive(Default)]
struct HashTrace(Vec<HashWrite>);

impl Hasher for HashTrace {
    fn finish(&self) -> u64 {
        0
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0.push(HashWrite::Bytes(bytes.to_vec()));
    }

    fn write_u8(&mut self, byte: u8) {
        self.0.push(HashWrite::Byte(byte));
    }

    fn write_usize(&mut self, length: usize) {
        self.0.push(HashWrite::Length(length));
    }
}

#[test]
fn nickname_hash_vector() {
    use HashWrite::{Byte, Length};
    // Retain the padded ASCII-array hash contract, including its byte writes
    // for custom hashers. No dependency on the former storage provider is needed.
    let nickname: Nickname = "Foo".parse().unwrap();
    let mut trace = HashTrace::default();
    nickname.hash(&mut trace);
    assert_eq!(
        trace.0,
        [
            Length(19),
            Byte(70),
            Byte(111),
            Byte(111),
            Byte(0),
            Byte(0),
            Byte(0),
            Byte(0),
            Byte(0),
            Byte(0),
            Byte(0),
            Byte(0),
            Byte(0),
            Byte(0),
            Byte(0),
            Byte(0),
            Byte(0),
            Byte(0),
            Byte(0),
            Byte(0),
        ]
    );
}
