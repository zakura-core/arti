
BREAKING: Remove onion-service-cli-extra, RawEntryId re-export, raw_id getters, KeyMgr::{get_from, validate_entry_integrity, remove_unchecked, list_by_id, list, list_keystores}, and Keystore::{raw_entry_id, remove_unchecked}. Typed key operations remain.

REMOVED (crate-visible): ArtiNativeKeystoreError::UnsupportedRawEntry and ArtiEphemeralKeystoreError::NotSupported, used only by the removed raw CLI operations.
