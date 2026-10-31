
BREAKING: `HsClientDescEncKeypairSpecifier`, its constructor and trait
implementations, and its crate-private `hs_id` field require the existing
`keymgr` feature. `tor-keymgr` is now optional. In-memory onion client
secret keys and ordinary onion service connections remain available
without that feature.
