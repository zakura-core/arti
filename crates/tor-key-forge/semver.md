
BREAKING: `ItemType` now extends `std::any::Any`, and `EncodableItem` inherits
that bound through `ItemType`, replacing the `downcast_rs::Downcast` supertrait.
The generated `is`, `downcast`, `downcast_rc`, `downcast_ref` and `downcast_mut`
methods on both trait objects, and inherited `into_any`, `into_any_rc`,
`as_any` and `as_any_mut`, are removed. Upcast the owned/reference trait object
to `dyn Any` and use its standard checked downcasts. `ErasedKey` remains
`Box<dyn ItemType>`.

BREAKING: Disable the `ssh-key-fork-arti` dependency's default ECDSA format
feature while retaining the explicit `std` and `rsa` selections. Supported
RSA, Ed25519, X25519 and expanded Ed25519 formats stay enabled. ECDSA was
already rejected by Arti's typed key conversion; `SshKeyAlgorithm` variants
and existing signatures remain unchanged.

This also reduces the conditional ECDSA types, enum variants, accessors,
`From` implementations and `sec1` re-export of the dependency exposed through
key-manager's public `ssh_key` re-export, unless another consumer enables
`ecdsa`. See `tor-keymgr/semver.md` for the affected API inventory.

Valid ECDSA input in a matching public/private keystore format can fail
before typed conversion with `SshKeyParse(AlgorithmUnknown)` instead of
`UnexpectedSshKeyType`. Both errors remain fail-closed `KeystoreCorrupted`;
the error variant, source and diagnostic text can change. No Arti public or
crate-visible item, signature, visibility or feature name is added.
