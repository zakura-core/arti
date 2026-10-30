
BREAKING: `BridgeDescEvent` retains `From<BridgeDescEvent> for u16` and `TryFrom<u16>`,
but the latter now returns `strum::ParseError`. The provider-specific
`num_enum::TryFromPrimitive` implementation and its associated items are removed.
