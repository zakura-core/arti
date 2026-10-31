
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

BREAKING: Remove the `async-std` and `smol` Cargo features. `full` and
`experimental` no longer enable these backends. The default Tokio/Rustls
runtime and `TorClient<PreferredRuntime>` APIs retain their signatures,
identity and behavior; `TorClient::with_runtime` remains generic.
The preferred `TorClient::{create_bootstrapped, builder}` implementations now
require `tokio` with `rustls`, replacing the former Tokio-or-async-std gate.
