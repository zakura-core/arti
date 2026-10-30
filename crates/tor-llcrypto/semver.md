CHANGED: `util::x509_extract_rsa_subject_kludge` uses the existing RSA provider's
DER reader instead of materializing an ASN.1 tree. Its signature is unchanged.
It still checks the same field positions, RSA OID and PKCS#1 key encoding,
discards the BIT STRING unused-bit byte, and ignores outer trailing bytes.
The surrounding field headers now require canonical DER encodings. Inputs
exceeding the reader's 256 MiB bound are rejected (Tor CERTS lengths are at
most 65,535 bytes). Unrelated
field contents (including names, dates, extensions and certificate signatures)
are opaque and are no longer subject to incidental ASN.1 value validation.
This helper remains key extraction, not X.509 certificate authentication; relay
identity signatures, certificate timeliness and TLS bindings are checked by
the channel handshake.
