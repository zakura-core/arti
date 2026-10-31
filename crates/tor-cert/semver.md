BREAKING: Remove the unused macro-generated `Ed25519CertBuilderError`
enum, its `UninitializedField` and `ValidationError` variants and payloads,
and its `Debug`, `Display`, `Error`, `From<String>`,
`From<derive_builder::UninitializedFieldError>`, and
`From<derive_builder::SubfieldBuildError<E>>` implementations.

The certificate builder has no `build` method and uses `CertEncodeError`
from its existing handwritten signer. The explicit builder retains every
builder method, `Clone`, and `Default`, without adding any public or
crate-visible API.
