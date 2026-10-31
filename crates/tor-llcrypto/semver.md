CHANGED: `util::x509_extract_rsa_subject_kludge` uses the existing RSA provider's
DER reader instead of materializing an ASN.1 tree. Its signature is unchanged.
It still checks the same field positions, RSA OID and PKCS#1 key encoding,
discards the BIT STRING unused-bit byte, and ignores outer trailing bytes.
The surrounding field headers now require canonical DER encodings and the
reader's supported tags (including its limit to tag numbers 0 through 30).
Inputs exceeding the reader's 256 MiB bound are rejected (Tor CERTS lengths
are at most 65,535 bytes). Unrelated field contents (including names, dates,
extensions and certificate signatures) are opaque and are no longer subject
to incidental ASN.1 value validation.
This helper remains key extraction, not X.509 certificate authentication; relay
identity signatures, certificate timeliness and TLS bindings are checked by
the channel handshake.

REMOVED: The `with-openssl` Cargo feature and its implicit `openssl`,
`typenum`, and `cipher` features. The existing RustCrypto implementations of
AES-128-CTR, AES-256-CTR, and SHA-1 are now used in every feature combination.
Default builds already use these exact implementations; their behavior and
backend performance selection are unchanged.

The public paths `cipher::aes::{Aes128Ctr, Aes256Ctr}` and `d::Sha1` remain.
For former OpenSSL builds these now identify the existing RustCrypto aliases
and reexport, rather than OpenSSL wrapper structs. The OpenSSL-only
`cipher::aes::{Aes128Key, Aes256Key}` structs are removed. Their private tuple
fields were never public constructors. Remove the OpenSSL key structs'
`KeySizeUser::KeySize`, `KeyInit::new`, `Zeroize::zeroize`, `ZeroizeOnDrop`,
and `Drop` implementations, and the OpenSSL cipher structs'
`InnerUser::Inner`, `IvSizeUser::IvSize`, `InnerIvInit::inner_iv_init`, and
`StreamCipher::{check_remaining, unchecked_apply_keystream_inout,
unchecked_write_keystream}` implementations. The old SHA-1 wrapper's `Clone`,
`Default`, `Update::update`, `OutputSizeUser::OutputSize`,
`FixedOutput::finalize_into`, and `HashMarker` implementations disappear with
that wrapper; the retained RustCrypto type supplies its existing digest APIs.
Derived and blanket trait methods follow the retained provider types.

No new public or `pub(crate)` Rust items, signatures, constructors, fields,
enum variants, aliases, constants, or exported templates are introduced.
RSA, Ed25519, X25519, SHA-2, SHA-3, SHAKE, onion-service authentication, random
number generation, and the optional SHA-1 assembly selection are unchanged.
