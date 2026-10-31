//! Runtime construction must work with the built-in default TLS provider.

#![cfg(all(feature = "tokio", feature = "rustls"))]

use tor_rtcompat::{PreferredRuntime, ToplevelBlockOn};

#[test]
fn preferred_runtime_initializes_tls_without_an_explicit_provider() {
    let runtime = PreferredRuntime::create().expect("construct the default runtime");
    assert!(rustls::crypto::CryptoProvider::get_default().is_some());
    assert_eq!(runtime.block_on(async { 42 }), 42);
}
