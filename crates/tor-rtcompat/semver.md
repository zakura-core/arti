ADDED: `SleepProviderExt` now has a `sleep_until()` method
ADDED: `JoinSet`, `JoinSetProvider`
BREAKING: `Runtime` now requires `JoinSetProvider`

BREAKING: The `Blocking::ThreadHandle<T>` associated type of
`tokio::TokioRustlsRuntime` is now an opaque Tokio
blocking-task future instead of `async_executors::BlockingHandle<T>`. It retains
`Future<Output = T>`, detached-on-drop behavior, panic/cancellation reporting,
and the existing `Debug` behavior and bound. Provider-specific constructors
such as `BlockingHandle::tokio` are no longer available on these futures.

ADDED (associated-type exposure): `TokioBlockingHandle<T>` implements `Future`
and, for `T: Debug`, `Debug`. It lives in the crate-private Tokio implementation
module, has a private field, and adds no public constructor or raw task controls.
The type is declared `pub` because it implements the existing public associated
type; it is not re-exported as a new top-level API.

REMOVED (crate-visible): `From<async_executors::TokioTp>` for the internal
`TokioRuntimeHandle`, including the corresponding blanket `Into` conversion.
The existing `From<tokio::runtime::Handle>` conversion, runtime constructors,
and `is_owned` behavior are unchanged.

CHANGED: Runtime construction still returns `io::ErrorKind::Other` on failure,
but its error payload is the original Tokio builder `io::Error` rather than
`async_executors::TokioTpErr`. Owned runtimes still shut down on their last
owner's drop, including Tokio's existing panic when dropped in an async context.

The runtime-handle refactor above retains its Cargo feature names.
`async_executors` is needed only with
`async-std`; Tokio-only builds no longer enable its unused process support.
Its Tokio features are no longer incidentally enabled in mixed-runtime builds
either. Callers using that provider's `TokioTp` or `BlockingHandle::tokio`
directly must enable its Tokio features explicitly. Async-std's runtime
implementations and associated handle type are unchanged.

BREAKING: Remove the native-TLS backend and its runtime/provider APIs. The
`native-tls`, `static`, `async-native-tls` and `native-tls-crate` Cargo features
are removed. `full` selects Rustls instead of native-TLS.

Removed Rust items:
- `tls::NativeTlsProvider` (also the implementation's `NativeTlsProvider`
  and `NativeTlsConnector<S>` within the `pub(crate)` `impls::native_tls` module).
- `tokio::TokioNativeTlsRuntime`, including `From<tokio::runtime::Handle>`,
  `create`, `current`, `run_test` and its runtime trait implementations.
- `async_std::AsyncStdNativeTlsRuntime`, including `create`, `current`,
  `run_test` and its runtime trait implementations.
- `smol::SmolNativeTlsRuntime`, including `create`, `run_test` and its
  runtime trait implementations.
- Hidden exported macros `if_tokio_native_tls_present!`,
  `if_async_std_native_tls_present!`, `if_smol_native_tls_present!`, and
  their `cond` reexports.
- `CertifiedConn` and `StreamOps` implementations on
  `async_native_tls::TlsStream<S>`.

`PreferredRuntime` and each runtime module's `PreferredRuntime` alias now
always use Rustls when their runtime and `rustls` features are enabled.
Their existing functions and runtime traits remain. Rustls enables its
existing Ring provider so default consumers need no explicit provider install;
applications can still install a provider before constructing a runtime.
Consumers also enabling another built-in Rustls provider must explicitly
install their chosen provider before runtime construction.
`test_with_all_runtimes!` now visits only the retained Rustls runtimes.

For previously native-TLS-selected preferred runtimes, `TlsProvider`'s
`Connector` and `TlsStream` associated types now use the existing Rustls
implementations. `Acceptor` and `TlsServerStream` use Rustls server types when
`tls-server` is enabled, or the existing unimplemented types otherwise.
`supports_keying_material_export` now returns true; the native backend did not
support it. Enabling `tls-server` now provides its existing Rustls behavior on
these preferred runtimes. The default TLS handshake fingerprint consequently
changes from native-TLS to Rustls/Ring. Explicit Rustls wallet runtimes already
use these implementations.

The removed runtime trait implementations include `Clone`, `Debug`,
`futures::task::Spawn::spawn_obj`, `Blocking::{ThreadHandle, spawn_blocking,
reenter_block_on}`, `ToplevelBlockOn::block_on`, `SleepProvider::{SleepFuture,
sleep}`, `CoarseTimeProvider::now_coarse`, `NetStreamProvider::{Stream,
Listener, ConnectOptions, ListenOptions, connect, listen}` for IP and Unix
addresses, `TlsProvider::{Connector, TlsStream, Acceptor, TlsServerStream,
tls_connector, tls_acceptor, supports_keying_material_export}`,
`UdpProvider::{UdpSocket, bind}`, and `JoinSetProvider::{JoinSet, new_join_set}`.
The associated blanket `Runtime` and `ToplevelRuntime` implementations also
cease to exist for the removed runtime types.

The removed native provider implements `Clone`, `Default` and `TlsProvider`
with the associated types and methods above. Its connector implements
`TlsConnector::{Conn, negotiate_unvalidated}`. The removed native stream
implementations provide `CertifiedConn::{peer_certificate,
export_keying_material, own_certificate}` and
`StreamOps::{set_tcp_notsent_lowat, new_handle}`. These traits themselves and
their Rustls implementations remain unchanged. No new public or `pub(crate)`
Rust items are introduced.

CHANGED: The experimental `testing` feature no longer enables AWS-LC through
`futures-rustls`. Its automatic provider fallback now uses the already enabled
Ring implementation. Applications that need AWS-LC must request that provider
on their own dependency. An explicitly installed process-wide provider remains
unchanged. Normal non-testing runtime behavior is unchanged.

No public or `pub(crate)` Rust items, signatures, aliases, methods, fields,
constructors, variants, constants, or Cargo feature names change. The private
`ensure_testing_provider_installed` helper retains its signature and the guard
that respects an existing provider. Remove the development self-dependency
that existed to supply AWS-LC to tests: unit tests use `cfg(test)`, and tests of
the explicit `testing` feature still request it normally.
BREAKING: The six socket-option builders now build infallibly. For
`CommonListenOptionsBuilder`, `TcpListenOptionsBuilder`,
`UnixListenOptionsBuilder`, `CommonConnectOptionsBuilder`,
`TcpConnectOptionsBuilder` and `UnixConnectOptionsBuilder`, `build(&self)`
returns the corresponding options value directly instead of a `Result`.
Remove their six generated `*BuilderError` enums, `UninitializedField` and
`ValidationError` variants and payloads, and `Debug`, `Display`, `Error` and
`From` implementations. The builders remain available through the existing
options' `builder()` methods; they retain `Clone`, `Default`, buffer setters
and nested `common()` accessors. No builder re-exports or visibility are added.


BREAKING: Own generalized and Unix address primitives in `tor-rtcompat` rather
than the separate `tor-general-addr` package. The existing `general::SocketAddr`
and `general::AddrParseError` re-exports now name types defined by this crate.
`general::DisplayLossy` is also re-exported. The original methods, variants,
payloads and trait implementations move with those types. The existing
`arbitrary` feature now selects the moved `SocketAddr` implementation.

Move `unix::SocketAddr`, `NoAfUnixSocketSupport` and the deprecated
`NoUnixAddressSupport` alias to the existing public `unix` facade. On Unix,
`SocketAddr` remains the identical standard-library type. On other targets,
its uninhabited placeholder type now belongs to this crate. Consequently,
`NetStreamProvider`/`NetStreamListener` implementations, runtime supertraits,
listener stream items and their address-dependent associated types refer to
the runtime-owned address types. The corresponding mock-provider types follow
the same identity change. No socket/TLS operation implementation changes.

Remove `general::UnsupportedAddress` and its `Clone`, `Debug`, `Display` and
`Error` implementations. It was used only by two wildcard arms that become
unreachable when the address enum and its provider share an owner. The existing
`Inet` and `Unix` provider branches are unchanged.

The standalone address package and its `default`, `full` and `arbitrary`
features are removed. Its existing parser/roundtrip fuzz package keeps the
same package identity and targets under `tor-rtcompat/fuzz`. No new crate,
feature, compatibility wrapper or public address behavior is introduced.
