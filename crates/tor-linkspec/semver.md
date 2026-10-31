BREAKING: Replace the three owned-target builder derives with explicit Rust.
`RelayIdsBuilder::build(&self)` now returns `RelayIds` directly and
`OwnedChanTargetBuilder::build(&self)` returns `OwnedChanTarget` directly.
`OwnedCircTargetBuilder::build(&self)` now returns
`Result<OwnedCircTarget, tor_config::ConfigBuildError>`; missing fields use
`ConfigBuildError::MissingField { field: String }` in the same ntor-key then
protocols order, with the shared configuration-error display text.

BREAKING: Remove the public generated `RelayIdsBuilderError`,
`OwnedChanTargetBuilderError`, and `OwnedCircTargetBuilderError` types from
the private owned module. Each loses `UninitializedField(&'static str)` and
`ValidationError(String)`, `Debug`, `Display`, `std::error::Error`,
`From<String>`, `From<derive_builder::UninitializedFieldError>`, and generic
`From<derive_builder::SubfieldBuildError<E>>` for `E: std::error::Error`.
These types were exposed by the previous build return signatures.

The three builder types, `Clone`/`Debug`/`Default`, target `builder()`
constructors, existing setters, nested accessors and conversion helpers remain.
All fields stay private. Relay-identity builder Debug output now shows its
flattened optional identities, without the redundant nested `Some` wrappers.
Target serialization, target Debug formatting, ID matching and link-specifier
validation are unchanged. No new public or crate-visible names or features.
