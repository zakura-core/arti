
BREAKING: Remove geoip, PartialNetDir::new_with_geoip, testnet::construct_custom_netdir_with_geoip and HasCountryCode implementations on Relay/UncheckedRelay. Relay metadata no longer includes country annotations.

BREAKING: `DirEvent` retains `From<DirEvent> for u16` and `TryFrom<u16>`,
but the latter now returns `strum::ParseError`. The provider-specific
`num_enum::TryFromPrimitive` implementation and its associated items are removed.

CHANGED (crate-visible): `ConsensusRelays::c_relays` returns a borrowed
`ConsensusRelaySlice<'_>` by value instead of
`&TiSlice<RouterStatusIdx, MdRouterStatus>`.
The new borrowed view has a private slice field, `Clone`/`Copy`, typed
`Index<RouterStatusIdx>`, and private length, iteration, indexed-iteration and
optional-lookup methods. It does not expose the former generic `TiSlice` API.
`RouterStatusIdx`, `HsDirPos`, and all public signatures and visibility are
unchanged. Owned microdescriptor/ring containers are private, retain distinct
index types and bounds checks, and preserve their index-keyed `Debug` output.
