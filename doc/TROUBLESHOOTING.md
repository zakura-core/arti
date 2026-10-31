# Fixing some common (and not-so-common) problems

Here's an infrequently-asked-questions list about compiling or embedding this
Arti library fork.

## Linking with SQLite

Arti uses SQLite via `rusqlite`. Install the SQLite development libraries and
`pkg-config` when linking against the system library. Alternatively, enable
`arti-client/static-sqlite` (also selected by `arti-client/static`) to build
SQLite from source and link it statically.

See [rusqlite's build notes](https://github.com/rusqlite/rusqlite#notes-on-building-rusqlite-and-libsqlite3-sys)
for platform-specific configuration.

The TLS backend is Rustls with Ring; it does not link the platform's TLS
library.

## Selecting a runtime

The `arti-client` defaults select Tokio and Rustls. When disabling defaults,
select `rustls` and an asynchronous runtime such as `tokio` or `async-std`.
The experimental `smol` feature is also retained.

Cargo unifies features across dependencies. If both `tokio` and `async-std`
are enabled, `tor_rtcompat::PreferredRuntime` prefers Tokio. To choose a
runtime explicitly, use `tor_rtcompat::tokio::TokioRustlsRuntime`,
`tor_rtcompat::async_std::AsyncStdRustlsRuntime`, or
`tor_rtcompat::smol::SmolRustlsRuntime` with the corresponding features.
Applications may also supply a custom implementation of the runtime traits.

Rustls enables its Ring cryptographic provider. Applications that install a
custom Rustls provider must do so before constructing a runtime.

## Running the public Tor network test

The ignored `arti-client` `live_tor` test uses the wallet's Tokio/Rustls
selection and explicitly installs Ring. Its destination HTTPS connection
validates the hostname and certificate chain using a system PEM CA bundle.
Set `SSL_CERT_FILE` to a bundle path if it is outside the standard Linux/macOS
locations. Unsupported locations and invalid or empty bundles are test setup
errors; the test never disables certificate validation. This only configures
the test's destination HTTPS trust store, not Tor relay authentication.
