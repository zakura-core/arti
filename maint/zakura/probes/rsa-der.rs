//! Compare the public DER encoder against its pinned previous provider.

use der_parser::ber::BerObject;
use rsa::traits::PublicKeyParts;
use rsa::{BigUint, RsaPublicKey};
use sha1::{Digest, Sha1};

/// Preserve the old production implementation, independently of the candidate.
fn previous_der(key: &RsaPublicKey) -> Vec<u8> {
    let positive_integer = |bytes: Vec<u8>| {
        if bytes[0] & 0x80 != 0 {
            [vec![0], bytes].concat()
        } else {
            bytes
        }
    };
    let n = positive_integer(key.n().to_bytes_be());
    let e = positive_integer(key.e().to_bytes_be());
    BerObject::from_seq(vec![
        BerObject::from_int_slice(&n),
        BerObject::from_int_slice(&e),
    ])
    .to_vec()
    .unwrap()
}

fn compare(key: &RsaPublicKey) {
    let previous = previous_der(key);
    let candidate = subject::pk::rsa::PublicKey::from_der(&previous).unwrap();
    assert_eq!(candidate.to_der(), previous);
    assert_eq!(candidate.as_key(), key);
    assert_eq!(
        candidate.to_rsa_identity().as_bytes(),
        Sha1::digest(&previous).as_slice()
    );
}

#[test]
fn rsa_der_length_sign_and_exponent_boundaries() {
    let mut compared = 0;
    // Cover both integer sign encodings and every DER length boundary through
    // RSA's supported 4096-bit key size. Tiny moduli are encoding tests only.
    for length in 1..=512 {
        for high in [0x01, 0x7f, 0x80, 0xff] {
            let mut modulus = vec![0x5a; length];
            modulus[0] = high;
            modulus[length - 1] |= 1;
            for exponent in [
                3_u64,
                17,
                127,
                129,
                255,
                257,
                65537,
                0x7fff_ffff,
                0x8000_0001,
            ] {
                let Ok(key) =
                    RsaPublicKey::new(BigUint::from_bytes_be(&modulus), BigUint::from(exponent))
                else {
                    continue;
                };
                compare(&key);
                compared += 1;
            }
        }
    }
    assert!(
        compared > 18_000,
        "unexpectedly empty or truncated key corpus"
    );
    println!("{compared} key encodings and Tor identity digests matched");
}

#[test]
fn rsa_der_certificate_fixtures() {
    for name in ["tor.der", "tpo.der"] {
        let der = std::fs::read(std::path::Path::new(env!("RSA_DER_TESTDATA")).join(name)).unwrap();
        let key = subject::util::x509_extract_rsa_subject_kludge(&der).unwrap();
        compare(key.as_key());
    }
}
