
BREAKING: Remove geoip, PartialNetDir::new_with_geoip, testnet::construct_custom_netdir_with_geoip and HasCountryCode implementations on Relay/UncheckedRelay. Relay metadata no longer includes country annotations.

BREAKING: `DirEvent` retains `From<DirEvent> for u16` and `TryFrom<u16>`,
but the latter now returns `strum::ParseError`. The provider-specific
`num_enum::TryFromPrimitive` implementation and its associated items are removed.

CHANGED (crate-visible): `ConsensusRelays::c_relays` returns a borrowed
`ConsensusRelaySlice<'_>` by value instead of
`&TiSlice<RouterStatusIdx, MdRouterStatus>`.
The new private view has a private slice field, `Clone`/`Copy`, typed
`Index<RouterStatusIdx>`, and private length, iteration, indexed-iteration and
optional-lookup methods. It does not expose the former generic `TiSlice` API.
`RouterStatusIdx`, `HsDirPos`, and all public signatures and visibility are
unchanged. Owned microdescriptor/ring containers are private, retain distinct
index types and bounds checks, and preserve their index-keyed `Debug` output.

BREAKING: Remove the `hs-service` feature, `NetDir::hs_dirs_upload`,
`NetDir::hs_all_time_periods`, deprecated `NetDir::hs_dirs`, `HsDirOp` and its
`Upload`/`Download` variants, and `NetParameters::hsdir_spread_store`. The
fork retains descriptor download selection through `NetDir::hs_dirs_download`.

BREAKING: Remove the public `HsDirParams` re-export and its `time_period`,
`start_of_shard_rand_period`, and `offset_within_srv_period` methods. The
parameter type remains private to the crate, with its existing derived traits.
Its fields and `compute` are now `pub(super)`; `compute` returns the current
ring's parameters directly instead of `HsDirs<HsDirParams>`.

REMOVED (crate-visible): `HsDirs<D>` and its `Clone`/`Debug` implementations,
`map`, `iter`, and `iter_for_op`. The directory now stores one descriptor
fetch ring, preserving its shared-random-value cache comparison.
