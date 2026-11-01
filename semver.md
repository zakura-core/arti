Compatibility notes for the unpublished 0.35.0 source candidate

BREAKING: The [`Blocking::ThreadHandle<T>`] associated types of
`tokio::{TokioNativeTlsRuntime, TokioRustlsRuntime}` change from
`async_executors::BlockingHandle<T>` to an opaque Tokio blocking-task future.
They retain `Future<Output = T>`, detached-on-drop behavior, panic and
cancellation reporting, and the existing `Debug` bound and representation.
Explicit downstream type annotations and provider-specific constructors such
as `BlockingHandle::tokio` are no longer compatible with these returned types.

ADDED (associated-type exposure): `TokioBlockingHandle<T>` implements `Future`
and, for `T: Debug`, `Debug`. It is declared `pub` inside the crate-private
Tokio implementation module to satisfy the public associated type. Its field
is private; it adds no public constructor, raw task controls or re-export.

REMOVED (crate-visible): `From<async_executors::TokioTp>` for the hidden
`TokioRuntimeHandle`, including the corresponding blanket `Into` conversion.
The existing `From<tokio::runtime::Handle>` conversion, public runtime
constructor signatures and `is_owned` behavior are unchanged.

CHANGED: Runtime construction still wraps failure as `io::ErrorKind::Other`,
but the boxed error is the original Tokio builder `io::Error` rather than
`async_executors::TokioTpErr`. Downcasting, source details and messages can
therefore change. The hidden handle's derived `Debug` ownership field also
changes shape; the public opaque runtime `Debug` implementation is unchanged.
Owned runtimes still shut down on their last owner's drop, including the
existing panic when dropped in an async context and waiting for started
blocking tasks. No background shutdown or leaked-runtime policy is added.

Cargo feature names and existing defaults remain unchanged. `async-std`
explicitly activates the now-optional `async_executors` dependency. Tokio uses
weak optional forwarding for that provider's `tokio_tp`, `tokio_timer` and
`tokio_io` features: this preserves their previous mixed-runtime feature union
when `async-std` activates the provider, without adding it to Tokio-only
builds. This weak forwarding is the intentional manifest deviation from the
merged upstream implementation. Async-std's runtime implementation and
associated handle type are unchanged.

This standalone candidate starts from the checksummed published crate.
`Cargo.toml` uses registry dependencies for `tor-error` and `tor-general-addr`;
it contains no workspace path or Git dependencies. `Cargo.toml.orig` retains
the released source manifest's historical path annotations and receives the
same feature/dependency changes, but it is not Cargo's active manifest.
The version, released lockfile, TLS, protocol and crypto files are unchanged.

[`Blocking::ThreadHandle<T>`]: https://docs.rs/tor-rtcompat/0.35.0/tor_rtcompat/trait.Blocking.html
