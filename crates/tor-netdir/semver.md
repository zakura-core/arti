
BREAKING: Remove geoip, PartialNetDir::new_with_geoip, testnet::construct_custom_netdir_with_geoip and HasCountryCode implementations on Relay/UncheckedRelay. Relay metadata no longer includes country annotations.

BREAKING: `DirEvent` retains `From<DirEvent> for u16` and `TryFrom<u16>`,
but the latter now returns `strum::ParseError`. The provider-specific
`num_enum::TryFromPrimitive` implementation and its associated items are removed.
