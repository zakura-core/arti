# Arti 0.35 SSH key-encoding compatibility source

This source snapshot layers the released `tor-keymgr` and `tor-key-forge`
0.35.0 packages on the existing standalone `tor-rtcompat` compatibility
commit `69d025f04aa67454160a8d3e0d1706366faf99ea`. The 32 existing runtime
source carriers remain byte-identical. This is a standalone compatibility
source layout, not the full Arti workspace or an Arti version upgrade.

The two key packages retain all 58 published files. Their only changes are
`default-features = false` on each `ssh-key` dependency in `Cargo.toml` and
`Cargo.toml.orig`. The active normalized manifests retain registry dependency
specifications. The original manifests retain their historical workspace
paths as provenance; they are not the active manifests. No new workspace,
path rewrite, dependency version, generated checksum, Rust body, compiler
profile, assembly, crypto algorithm or inline attribute is introduced.
Published package lockfiles are unchanged.

## Selected features and API differences

The selected SSH package remains `ssh-key` 0.6.7. `std` is retained on both
edges, and `rsa` is retained on `tor-key-forge`. These continue to enable
`alloc` and `rand_core`. The default `ecdsa` parsing feature is no longer
implicitly enabled by either Arti package. If no other parent enables it,
`sec1` and its sole child `base16ct` disappear. This is a dependency reduction;
no build-time or runtime speed improvement is claimed here.

This narrows the selected public SSH API, including the API reachable through
`tor_keymgr::ssh_key`: the `sec1` re-export, ECDSA key types, their inherent and
trait APIs, `KeyData` and `KeypairData` ECDSA variants and accessors, and their
`From` conversions are unavailable without that feature. The `Algorithm`
ECDSA variants and `EcdsaCurve` names remain. No Arti or application public or
crate-visible declaration, constructor signature, alias, trait method or
visibility is changed by these manifest edits. The selected SSH enum variant
set and possible Rust layout change; these Rust enums have no stable C ABI.
No application wire or native ABI change is introduced.

## Parsing, diagnostics and preserved behavior

The 0.35 Arti key parser continues to accept its existing Ed25519, X25519 and
expanded Ed25519 formats. A valid ECDSA or security-key ECDSA key in one of
those expected key slots remains rejected. Its native diagnostic changes
from `UnexpectedSshKeyType` (wanted/found algorithm) to `SshKeyParse` with
`ssh_key::Error::AlgorithmUnknown` as its source. Both retain the outer
`Error::Keystore` classification and `ErrorKind::KeystoreCorrupted`. This
intentional diagnostic difference is part of the compatibility contract.
Keystore listing is based on filename, extension and path syntax; no key
content parsing or listing behavior is changed.

RSA support in the underlying SSH package is retained, including public and
private SSH parse/encode roundtrips. This does not add RSA acceptance to the
0.35 Arti native keystore: its expected-key dispatch currently handles only
Ed25519, X25519 and expanded Ed25519. Newer Main source has additional typed
RSA handling, which remains that source line's separate contract. This
snapshot does not backport or remove that behavior.

## Provenance and validation scope

Both key packages come from the checksummed published 0.35.0 crate archives,
with source VCS revision `0b636d9d8d3309663de0fb5554255f5f8f82544d` and original
paths `crates/tor-keymgr` and `crates/tor-key-forge`. The release `.cargo_vcs_info`
files are preserved verbatim. The surrounding Git commit is a delivery
container; it does not replace the published-source provenance.

Focused validation uses the seven existing parser tests, nineteen published
ECDSA wrong-slot cases checking the exact old/new errors and unchanged kind,
and two published RSA SSH roundtrips. Separate actual consumer metadata and
normal release artifacts must establish the effective feature/dependency
closure. Validation is not implied by constructing this source snapshot.
Publication and application adoption remain subject to the owner's native
platform and Main-source gates. No push, PR or network operation was used to
prepare this private export.
