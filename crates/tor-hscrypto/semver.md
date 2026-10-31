BREAKING: Remove the `ope` feature and public `ope` module. This deletes
`AesOpeKey`, its `from_secret`/`encrypt` methods and `Clone`/`Debug`, and
`SrvPeriodOffset`, its crate-visible `u32` payload, `From<u32>`, and
`Copy`/`Clone`/`Debug`/`PartialEq`. These existed only to generate revision
counters for onion descriptor publication. The implicit `cipher` and `zeroize`
features and their optional direct dependencies are also removed.
