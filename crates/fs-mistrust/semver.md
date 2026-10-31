BREAKING: Remove the unreachable `Error::MissingField` variant and its
`From<derive_builder::UninitializedFieldError>` implementation. The explicit
`MistrustBuilder` supplies every field and retains the existing builder
methods, defaults, Serde representation, and permission validation errors.
No public or crate-visible Rust item is added.
