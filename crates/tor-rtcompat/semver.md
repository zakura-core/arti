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

The runtime-handle refactor above originally retained its Cargo feature names
and the async-std implementation, while removing Tokio's `async_executors`
feature activation. The later Tokio-only cleanup below removes the remaining
`async_executors` dependency together with async-std and Smol.

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

BREAKING: Retain only the built-in Tokio/Rustls runtime for this wallet fork.
Remove `async-std` and `smol` from `tor-rtcompat` and `arti-client`, and remove
`tor-rtcompat`'s implicit `async-std-crate`, `async-io`, and `blocking` Cargo
features. `async_executors` already used `dep:` and had no implicit feature.
`full` no longer selects async-std, and `experimental` no longer selects Smol.
Default Tokio features and runtime behavior are unchanged.

Removed public modules and Rust items:
- `async_std` and `smol`, their `PreferredRuntime` aliases and
  `create_runtime_impl` reexports.
- `async_std::AsyncStdRustlsRuntime::{create, current, run_test}` and
  `smol::SmolRustlsRuntime::{create, run_test}`, including the opaque runtime
  types themselves, `Clone`, `Debug`, and runtime trait implementations.
- The hidden public `cond` module and its reexports, and exported macros
  `if_tokio_rustls_present!`, `if_async_std_rustls_present!`, and
  `if_smol_rustls_present!`. The retained `test_with_all_runtimes!` now runs
  the closure once on Tokio, retaining its existing outcome/error check.
  `test_with_one_runtime!` retains the preferred wrapper and return behavior.

Removed crate-visible or associated-type-exposed implementation items:
- `impls::{async_std, smol}`, `impls::smol::net`, each backend's
  `create_runtime`, `impls::smol::SmolRuntime` and its `Clone` implementation.
- Both backends' generated `IncomingTcpStreams` and Unix-only
  `IncomingUnixStreams`, their `from_listener` constructors and
  `Stream::{Item, poll_next}` implementations, and `UdpSocket` wrappers.
- `NetStreamListener::{Stream, Incoming, incoming, local_addr}` for the
  backends' TCP/Unix listeners; `UdpSocket::{recv, send, local_addr}` for
  their UDP wrappers; `StreamOps::{set_tcp_notsent_lowat, new_handle}` on
  their TCP streams and Unix-stream `set_tcp_notsent_lowat` implementations.
- `NetStreamProvider::{Stream, Listener, ConnectOptions, ListenOptions,
  connect, listen}` for IP and Unix addresses, `UdpProvider::{UdpSocket,
  bind}`, `SleepProvider::{SleepFuture, sleep}`,
  `ToplevelBlockOn::block_on`, `Blocking::{ThreadHandle, spawn_blocking,
  reenter_block_on}`, and `JoinSetProvider::{JoinSet, new_join_set}` on
  `async_executors::AsyncStd` and `SmolRuntime`; also Smol's
  `futures::task::Spawn::spawn_obj` implementation.
- Both platform-gated definitions of the crate-visible async function
  `tcp_async_io_connect(&SocketAddr, &TcpConnectOptions) ->
  io::Result<std::net::TcpStream>`.

The removed opaque runtime wrappers also supplied `Spawn::spawn_obj`,
`CoarseTimeProvider::now_coarse`, TLS associated types
`{Connector, TlsStream, Acceptor, TlsServerStream}` and methods
`{tls_connector, tls_acceptor, supports_keying_material_export}`, and all
runtime-trait methods/associated types listed above. Their associated blanket
`Runtime` and `ToplevelRuntime` implementations disappear with the types.
Tokio's opaque types, preferred wrapper identity, constructors, associated
types, and trait implementations remain unchanged.

No new public or `pub(crate)` Rust items are introduced. Keep all generic
runtime/transport traits, `CompoundRuntime`, `RuntimeSubstExt`,
`FuturesJoinSet`, and the mock-runtime-facing interfaces unchanged. Only
stale trait documentation is adjusted; the trait definitions and default
implementations retain their behavior.

The existing `PreferredRuntime::{current, create, run_test}` wrapper and
`tls::{RustlsProvider, RustlsAcceptor, RustlsServerStream, UnimplementedTls}`
exports now require the retained `tokio` runtime feature (and their existing
Rustls/server feature gates), rather than accepting a removed backend feature.
The public test macros likewise require Tokio/Rustls. Provided methods of the
removed trait implementations disappear with those types; their generic trait
definitions and default bodies remain unchanged.
