//! Ciphers used to implement the Tor protocols.
//!
//! Fortunately, Tor has managed not to proliferate ciphers.  It only
//! uses AES, and (so far) only uses AES in counter mode.

/// Re-exports implementations of counter-mode AES.
///
/// These ciphers implement the `cipher::StreamCipher` trait, so use
/// the [`cipher`](https://docs.rs/cipher) crate to access them.
#[cfg_attr(docsrs, doc(cfg(true)))]
pub mod aes {
    // These implement StreamCipher.
    /// AES128 in counter mode as used by Tor.
    pub type Aes128Ctr = ctr::Ctr128BE<aes::Aes128>;

    /// AES256 in counter mode as used by Tor.  
    pub type Aes256Ctr = ctr::Ctr128BE<aes::Aes256>;
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]

    use super::aes::{Aes128Ctr, Aes256Ctr};
    use cipher::{KeyIvInit, StreamCipher};
    use hex_literal::hex;

    /// NIST SP 800-38A, F.5.1 and F.5.5, including counter carry.
    const COUNTER: [u8; 16] = hex!("f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff");
    /// Four plaintext blocks shared by both NIST CTR examples.
    const PLAINTEXT: [u8; 64] = hex!(
        "6bc1bee22e409f96e93d7e117393172a
         ae2d8a571e03ac9c9eb76fac45af8e51
         30c81c46a35ce411e5fbc1191a0a52ef
         f69f2445df4f9b17ad2b417be66c3710"
    );

    /// Verify known ciphertext and continuity across partial-block calls.
    fn assert_ctr_vector<C: KeyIvInit + StreamCipher>(key: &[u8], ciphertext: [u8; 64]) {
        let mut encrypted = PLAINTEXT;
        C::new_from_slices(key, &COUNTER)
            .unwrap()
            .apply_keystream(&mut encrypted);
        assert_eq!(encrypted, ciphertext);

        let mut cipher = C::new_from_slices(key, &COUNTER).unwrap();
        let mut chunked = PLAINTEXT;
        let mut offset = 0;
        for length in [1, 15, 17, 31] {
            cipher.apply_keystream(&mut chunked[offset..offset + length]);
            offset += length;
        }
        assert_eq!(chunked, ciphertext);

        let mut decrypted = ciphertext;
        C::new_from_slices(key, &COUNTER)
            .unwrap()
            .apply_keystream(&mut decrypted);
        assert_eq!(decrypted, PLAINTEXT);
    }

    // https://doi.org/10.6028/NIST.SP.800-38A, sections F.5.1 and F.5.5.
    #[test]
    fn aes128_ctr_nist() {
        assert_ctr_vector::<Aes128Ctr>(
            &hex!("2b7e151628aed2a6abf7158809cf4f3c"),
            hex!(
                "874d6191b620e3261bef6864990db6ce
                 9806f66b7970fdff8617187bb9fffdff
                 5ae4df3edbd5d35e5b4f09020db03eab
                 1e031dda2fbe03d1792170a0f3009cee"
            ),
        );
    }

    #[test]
    fn aes256_ctr_nist() {
        assert_ctr_vector::<Aes256Ctr>(
            &hex!(
                "603deb1015ca71be2b73aef0857d7781
                 1f352c073b6108d72d9810a30914dff4"
            ),
            hex!(
                "601ec313775789a5b7a7f504bbf3d228
                 f443e3ca4d62b59aca84e990cacaf5c5
                 2b0930daa23de94ce87017ba2d84988d
                 dfc9c58db67aada613c2dd08457941a6"
            ),
        );
    }
}
