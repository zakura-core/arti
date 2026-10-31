# tor-config

Typed configuration helpers shared by the embedded Arti client.

Applications configure the client through Rust values and builders. This crate
provides builder validation, defaults, collection helpers and reconfiguration
policies used by the internal Tor components.

Configuration-file discovery, configuration trees and file-format resolution
are not provided by this fork. Wallet callers supply their state/cache paths
and filesystem policy directly.

The remaining optional `file-watcher` feature supports the onion-service
hosting code pending its separate removal; embedded wallet clients do not
enable it.
