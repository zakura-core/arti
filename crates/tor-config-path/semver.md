REMOVED: The implicit `shellexpand` dependency feature. Use the existing
`expand-paths` feature for configuration path interpolation. Variable, default,
dollar-escape and tilde expansion behavior is unchanged; inserted paths retain
their native OS representation. No Rust signature or visibility changes.


BREAKING: Remove the unused `address` feature and its implicit
`tor-general-addr` dependency feature. Defaults now select only `expand-paths`.
Remove public module `addr`, `CfgAddr`, its `new_unix`, `address` and
`substitutions_will_apply` methods, `Clone`, `Debug`, `Serialize`, `Deserialize`,
`Eq`, `PartialEq`, `FromStr`, `From<std::net::SocketAddr>` and
`TryFrom<unix::SocketAddr>` implementations.

Remove `CfgAddrError`, its `NoAfUnixSocketSupport`, `Path` and
`ConstructAfUnixAddress` variants/payloads and its `Clone`, `Debug`, `Display`,
`Error`, `From<unix::NoAfUnixSocketSupport>` and `From<CfgPathError>`
implementations. Remove `UnixAddrNotAPath` and its
`Clone`, `Debug`, `Default`, `Display` and `Error` implementations. Their methods
and associated types disappear with the implementations.

This deletes only the network-address/path-expansion adapter. Filesystem
`CfgPath`, path resolution/expansion, Serde representations and error behavior
remain unchanged; no runtime dependency replaces the removed address edge.
