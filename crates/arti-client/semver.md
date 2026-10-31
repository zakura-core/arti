
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

BREAKING: Remove the `rpc` and implicit `dyn-clone` Cargo features and public
`rpc` module. This removes `TorClient::rpc_methods`, `ClientStatusInfo` and its
`From<BootstrapStatus>` conversion, `IsolatedClient`, `ClientConnectionError`,
`ClientConnectionResult`, and `ConnectWithPrefs`, `ResolveWithPrefs` and
`ResolvePtrWithPrefs` with all their public fields and RPC trait implementations.
The RPC `Object` implementation for `TorClient` is removed. Native embedded
client operations and optional persistent key stores are unchanged.
