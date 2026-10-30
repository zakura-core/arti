
BREAKING: Remove onion-service-cli-extra, RawEntryId re-export, raw_id getters, KeyMgr::{get_from, validate_entry_integrity, remove_unchecked, list_by_id, list, list_keystores}, and Keystore::{raw_entry_id, remove_unchecked}. Typed key operations remain.

REMOVED (crate-visible): ArtiNativeKeystoreError::UnsupportedRawEntry and ArtiEphemeralKeystoreError::NotSupported, used only by the removed raw CLI operations.

ADDED: `key-encoding` feature, enabled by `keymgr` and therefore by default.
Disabling it gates key-forge re-exports, `Error::{KeyForge, InvalidCert}` and
their `From` implementations, and `raw::RawEntryId::Ephemeral` (crate-visible
normally, public under the existing experimental visibility selection).
The test-only crate-visible `DummyKey` helper is similarly gated.
