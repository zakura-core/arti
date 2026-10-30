
BREAKING: Remove onion-service-cli-extra and the CLI-only InertTorClient::keymgr and TorClient::keymgr accessors.

BREAKING: Remove geoip, CountryCode re-export and StreamPrefs::{exit_country, any_exit_country}. Ordinary Tor and onion routing is unchanged.

BREAKING: Removing channel-manager's unused key-manager dependency means a
client built without `keymgr` no longer accidentally enables key stores.
Default key-store configuration is now disabled in that feature combination;
explicit `enabled = true` is rejected by the existing configuration validator.
Enable `keymgr` to use persistent or C Tor key stores. In-memory onion client
keys and cryptographic operations remain available without it.
The existing `ephemeral-keystore` feature now explicitly selects `keymgr`,
retaining its functional key-store behavior.
