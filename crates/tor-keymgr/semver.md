
BREAKING: Remove onion-service-cli-extra, RawEntryId re-export, raw_id getters, KeyMgr::{get_from, validate_entry_integrity, remove_unchecked, list_by_id, list, list_keystores}, and Keystore::{raw_entry_id, remove_unchecked}. Typed key operations remain.

REMOVED (crate-visible): ArtiNativeKeystoreError::UnsupportedRawEntry and ArtiEphemeralKeystoreError::NotSupported, used only by the removed raw CLI operations.

BREAKING: Remove the unused key-description registry and `KeyMgr::describe`.
The following public metadata APIs are removed:

- `KeyPathInfo`, its `builder`, `summary`, `role` and `extra_info` accessors,
  and its `Debug`, `Clone` and `PartialEq` implementations;
- the generated `KeyPathInfoBuilder` and builder error, including its
  `summary`, `role`, `extra_info`, `set_all_extra_info`, `build` and default
  construction APIs;
- `KeyPathInfoExtractor` and its `describe` trait method;
- `register_key_info_extractor!` and the doc-hidden `inventory` re-export;
- the doc-hidden `key_specifier_derive::describe_via_components` helper and
  `key_specifier_derive::KeyPathInfoBuilder` re-export.

The exported `KeySpecifier` derive-deftly template no longer generates or
registers a private `SomeKeySpecInfoExtractor`. Its formerly required
`#[deftly(summary = "...")]` metadata attribute is removed; callers must
remove that attribute. The generated `KeySpecifier`, `KeySpecifierPattern`,
`TryFrom<&KeyPath>` and `CTorKeySpecifier` implementations keep their existing
signatures and behavior. Manual `KeySpecifier` implementations, component
formatting methods, path parsing/errors and typed key operations are unchanged.
There are no added or changed `pub(crate)` interfaces.

BREAKING: Remove the `config::ConfigurationSource` re-export with the
configuration-file discovery API. Typed keystore configuration is unchanged.

ADDED: `key-encoding` feature, enabled by `keymgr` and therefore by default.
Disabling it gates key-forge re-exports, `Error::{KeyForge, InvalidCert}` and
their `From` implementations, and `raw::RawEntryId::Ephemeral` (crate-visible
normally, public under the existing experimental visibility selection).
The test-only crate-visible `DummyKey` helper is similarly gated.



BREAKING: Remove `From<derive_builder::UninitializedFieldError>` and
`From<derive_builder::SubfieldBuildError<E>>` (for `E: std::error::Error`)
for `KeyMgrBuilderError`, including their `from` methods. These conversions
existed in both the real `keymgr` configuration and the feature-disabled
implementation. Explicit builders construct the same existing error variants
directly. `From<String>`, `Debug`, `Display`, `Error`, both error variants and
their payloads remain unchanged.

The owned `KeyMgrBuilder` still implements only `Default`, retains its
consuming primary-store setter and `build`, and preserves all secondary-store
list accessors. Missing-primary errors still precede duplicate-ID validation;
the real key manager still rejects duplicate store IDs. The feature-disabled
key manager keeps its inert behavior. No public or crate-visible API is added,
and no retained signature or feature changes.
