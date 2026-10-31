BREAKING: Remove the `rpc` Cargo feature and public `client::rpc` module,
including `GetTunnel`, `DescribePath`, `PathEntry::{VirtualHop, KnownRelay}`,
`RelayIds`, `PathDescription` and their RPC trait implementations. Remove the
RPC `Object` implementations for `ClientTunnel` and `ClientDataStreamCtrl`.
The independent `stream-ctrl` feature remains available.

Remove the RPC-only crate-visible `ClientTunnel::tagged_paths` and
parent-visible `TunnelMutableState::tagged_paths` helpers. Native tunnel, relay
handshake, stream and onion-service operations are unchanged.

BREAKING: Remove `From<derive_builder::UninitializedFieldError>` for
`CircuitPadderConfigError`, including its `from` method. This implementation
was public with `circ-padding-manual`, and applied to the crate-visible error
with `circ-padding` alone. Explicit builders now construct the same existing
`UninitializedField` variant directly.

The seven parameter builders preserve their public/scoped types, methods,
setters, Clone/Default, constructors, required-field order, configuration
Builder trait implementations and feature-dependent visibility. No signatures,
variants or payloads change. Channel padding bounds, circuit-padding fraction
validation and defaults remain unchanged, including the distinction between
omitted fraction limits and explicit `1.0`.
