BREAKING: Removing channel-manager's unused key-manager dependency means a
client built without `keymgr` no longer accidentally enables key stores.
Default key-store configuration is now disabled in that feature combination;
explicit `enabled = true` is rejected by the existing configuration validator.
Enable `keymgr` to use persistent or C Tor key stores. In-memory onion client
keys and cryptographic operations remain available without it.
The existing `ephemeral-keystore` and `onion-service-cli-extra` features now
explicitly select `keymgr`, retaining their functional key-store behavior.
The Arti command-line application also selects it explicitly.
