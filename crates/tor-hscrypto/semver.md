ADDED: Default-enabled `key-encoding` feature. Disabling it gates the existing
`ToEncodableKey` implementations (both conversion methods and associated types)
for `HsClientDescEncKeypair`, `HsBlindIdKeypair`, `HsBlindIdKey`, `HsIdKeypair`,
`HsIdKey`, `HsDescSigningKeypair`, `HsIntroPtSessionIdKeypair` and
`HsSvcNtorKeypair`. In-memory key types and cryptographic operations are unchanged.
