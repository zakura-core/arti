//! Digests and XOFs used to implement the Tor protocol.
//!
//! In various places, for legacy reasons, Tor uses SHA1, SHA2, SHA3,
//! and SHAKE.  We re-export them all here, in forms implementing the
//! the [`digest::Digest`] traits.
//!
//! Other code should access these digests via the traits in the
//! [`digest`] crate.

pub use sha1::Sha1;

pub use sha2::{Sha256, Sha512};
pub use sha3::{Sha3_256, Shake128, Shake256, Shake256Reader};

#[cfg(test)]
mod test {
    use super::Sha1;
    use digest::Digest as _;
    use hex_literal::hex;

    /// NIST's one- and two-block SHA-1 message samples.
    #[test]
    fn sha1_nist() {
        // NIST CSRC example values, SHA1.pdf (FIPS 180-2 samples).
        let cases: &[(&[u8], [u8; 20])] = &[
            (b"abc", hex!("a9993e364706816aba3e25717850c26c9cd0d89d")),
            (
                b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
                hex!("84983e441c3bd26ebaae4aa1f95129e5e54670f1"),
            ),
        ];
        for (message, expected) in cases {
            assert_eq!(&Sha1::digest(message)[..], expected);
            let mut digest = Sha1::new();
            for byte in *message {
                digest.update([*byte]);
            }
            assert_eq!(&digest.finalize()[..], expected);
        }
    }
}
