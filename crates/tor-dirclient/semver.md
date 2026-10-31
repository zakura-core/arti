BREAKING: Renamed `check_circuit` to `check_circuit_skip_optimization`

BREAKING: Remove the `hs-service` feature and `request::HsDescUploadRequest`,
its constructor, `Clone`/`Debug`, and request trait implementations. Descriptor
GET requests remain available with `hs-client`. The two
`should_report_as_suspicious_if_anon` methods now require `hs-client` rather
than either onion client or service support.
