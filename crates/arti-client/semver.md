
BREAKING: Remove onion-service-cli-extra and the CLI-only InertTorClient::keymgr and TorClient::keymgr accessors.

BREAKING: Remove geoip, CountryCode re-export and StreamPrefs::{exit_country, any_exit_country}. Ordinary Tor and onion routing is unchanged.

BREAKING: Remove the `native-tls` and `static-native-tls` Cargo features.
Default and `full` now select Rustls/Ring instead of native-TLS. The `static`
feature retains static SQLite linking without a native-TLS backend.

BREAKING: Remove the `load-config` feature, `config::default_config_files`,
`config::{ConfigurationSource, ConfigurationSources, ConfigError}` re-exports,
and the `load::TopLevel` implementation for `TorClientConfig`. Applications
supply Rust configuration values directly; directory builders and validation
retain their existing behavior.

REMOVED: The `accel-openssl` Cargo feature. Existing default RustCrypto
implementations remain in use.

BREAKING: Removing channel-manager's unused key-manager dependency means a
client built without `keymgr` no longer accidentally enables key stores.
Default key-store configuration is now disabled in that feature combination;
explicit `enabled = true` is rejected by the existing configuration validator.
Enable `keymgr` to use persistent or C Tor key stores. In-memory onion client
keys and cryptographic operations remain available without it.
The existing `ephemeral-keystore` feature now explicitly selects `keymgr`,
retaining its functional key-store behavior.
