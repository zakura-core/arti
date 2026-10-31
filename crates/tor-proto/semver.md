BREAKING: Remove the `rpc` Cargo feature and public `client::rpc` module,
including `GetTunnel`, `DescribePath`, `PathEntry::{VirtualHop, KnownRelay}`,
`RelayIds`, `PathDescription` and their RPC trait implementations. Remove the
RPC `Object` implementations for `ClientTunnel` and `ClientDataStreamCtrl`.
The independent `stream-ctrl` feature remains available.

Remove the RPC-only crate-visible `ClientTunnel::tagged_paths` and
parent-visible `TunnelMutableState::tagged_paths` helpers. Native tunnel, relay
handshake, stream and onion-service operations are unchanged.
