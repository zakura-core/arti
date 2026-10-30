//! Compare the retained nickname contract with its former storage provider.
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};

use tor_netdoc::types::Nickname;

const MAX_NICKNAME_LEN: usize = 19;

mod previous {
    use super::*;
    use std::str::FromStr;
    use tinystr::TinyAsciiStr;

    #[derive(Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
    pub(super) struct Nickname(TinyAsciiStr<MAX_NICKNAME_LEN>);

    impl Nickname {
        pub(super) fn parse(s: &str) -> Result<Self, ()> {
            let tiny = TinyAsciiStr::from_str(s).map_err(|_| ())?;
            if tiny.is_ascii_alphanumeric() && !tiny.is_empty() {
                Ok(Self(tiny))
            } else {
                Err(())
            }
        }

        pub(super) fn as_str(&self) -> &str {
            self.0.as_str()
        }
    }

    impl fmt::Display for Nickname {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.as_str().fmt(f)
        }
    }

    impl serde::Serialize for Nickname {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.collect_str(self)
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum HashWrite {
    Bytes(Vec<u8>),
    Byte(u8),
    Length(usize),
}

#[derive(Default, Debug, PartialEq, Eq)]
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

fn hash_trace(value: &impl Hash) -> HashTrace {
    let mut trace = HashTrace::default();
    value.hash(&mut trace);
    trace
}

fn hash_value(value: &impl Hash) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn previous_provider_contract() {
    let mut cases = vec![String::new()];
    let alphabet = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    for &first in alphabet {
        cases.push(char::from(first).to_string());
        for &second in alphabet {
            cases.push(String::from_utf8(vec![first, second]).unwrap());
        }
    }
    // Exercise every ASCII byte at every legal position and both length bounds.
    for len in 1..=MAX_NICKNAME_LEN + 1 {
        for byte in 0..=127 {
            for position in 0..len {
                let mut input = vec![b'a'; len];
                input[position] = byte;
                cases.push(String::from_utf8(input).unwrap());
            }
        }
    }
    for input in ["ö", "é", "🦀", "a\u{301}", "０", "Α", "a\0z"] {
        cases.push(input.to_owned());
        cases.push(format!("{input}{}", "z".repeat(MAX_NICKNAME_LEN)));
    }
    for seed in 0..4096 {
        let len = seed % MAX_NICKNAME_LEN + 1;
        let mut value = seed as u64;
        let mut input = String::new();
        for _ in 0..len {
            value = value.wrapping_mul(6364136223846793005).wrapping_add(1);
            input.push(char::from(
                alphabet[(value >> 32) as usize % alphabet.len()],
            ));
        }
        cases.push(input);
    }
    assert_eq!(cases.len(), 34_897);

    let mut accepted = Vec::new();
    for input in &cases {
        let previous = previous::Nickname::parse(input);
        let candidate = input.parse::<Nickname>();
        assert_eq!(previous.is_ok(), candidate.is_ok(), "{input:?}");
        let encoded = serde_json::to_string(input).unwrap();
        assert_eq!(
            serde_json::from_str::<Nickname>(&encoded).is_ok(),
            previous.is_ok(),
            "JSON {input:?}"
        );
        if let (Ok(previous), Ok(candidate)) = (previous, candidate) {
            assert_eq!(previous.as_str(), candidate.as_str());
            assert_eq!(previous.to_string(), candidate.to_string());
            assert_eq!(format!("{previous:?}"), format!("{candidate:?}"));
            assert_eq!(format!("{previous:#?}"), format!("{candidate:#?}"));
            assert_eq!(hash_value(&previous), hash_value(&candidate));
            assert_eq!(hash_trace(&previous), hash_trace(&candidate));
            assert_eq!(
                serde_json::to_string(&previous).unwrap(),
                serde_json::to_string(&candidate).unwrap()
            );
            let decoded: Nickname = serde_json::from_str(&encoded).unwrap();
            assert_eq!(decoded, candidate);
            accepted.push((previous, candidate));
        }
    }
    assert!(accepted.len() > 10_000);
    // Compare adjacent and distant pairs as well as the complete sorted order.
    for (left, right) in accepted.iter().zip(accepted.iter().rev()) {
        assert_eq!(left.0.cmp(&right.0), left.1.cmp(&right.1));
        assert_eq!(left.0 == right.0, left.1 == right.1);
    }
    accepted.sort_by(|left, right| left.0.cmp(&right.0));
    let mut candidates: Vec<_> = accepted.iter().map(|(_, candidate)| candidate).collect();
    candidates.sort();
    for ((previous, _), candidate) in accepted.iter().zip(candidates) {
        assert_eq!(previous.as_str(), candidate.as_str());
    }
}
