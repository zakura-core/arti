
BREAKING: Remove onion-service-cli-extra and the CLI-only InertTorClient::keymgr and TorClient::keymgr accessors.

BREAKING: Remove geoip, CountryCode re-export and StreamPrefs::{exit_country, any_exit_country}. Ordinary Tor and onion routing is unchanged.

BREAKING: Remove the `native-tls` and `static-native-tls` Cargo features.
Default and `full` now select Rustls/Ring instead of native-TLS. The `static`
feature retains static SQLite linking without a native-TLS backend.
