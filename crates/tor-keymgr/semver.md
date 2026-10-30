ADDED: `key-encoding` feature, enabled by `keymgr` and therefore by default.
Disabling it gates key-forge re-exports, `Error::{KeyForge, InvalidCert}` and
their `From` implementations, and `raw::RawEntryId::Ephemeral` (crate-visible
normally, public under the existing experimental visibility selection).
The test-only crate-visible `DummyKey` helper is similarly gated.
