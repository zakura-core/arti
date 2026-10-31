BREAKING: `ProxyProtocol::from_str` now requires standard numeric IPv4 or bracketed IPv6 proxy addresses and rejects URL host/path normalization and unencoded control characters. Error payloads identify the invalid address component rather than retaining the full URI.
FIXED: An explicitly specified HTTP proxy port 80 is accepted rather than normalized away and rejected as missing.
FIXED: Credential formatting for programmatically constructed scoped IPv6 proxies no longer fails through a general URL parser. Scoped addresses remain unsupported in configuration URI strings.
BREAKING: With `metrics` enabled, remove the crate-visible `ChanMgrMetrics`
fields `inbound_channels_built_failure_no_such_transport` and
`inbound_channels_built_failure_pt`, and their obsolete transport-error labels.
FIXED: The `metrics` feature builds after removing bridge and transport errors.
