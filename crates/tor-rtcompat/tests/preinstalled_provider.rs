//! The testing fallback must preserve the application's provider selection.

#![cfg(all(
    feature = "testing",
    feature = "rustls",
    any(feature = "tokio", feature = "async-std", feature = "smol")
))]

use futures_rustls::rustls::crypto::{CryptoProvider, ring};
use std::sync::Arc;
use tor_rtcompat::tls::RustlsProvider;

#[test]
fn preserves_preinstalled_provider() {
    // Customize an existing provider's configuration without adding a backend.
    let mut provider = ring::default_provider();
    provider.cipher_suites.truncate(1);
    provider
        .install_default()
        .expect("install the application's provider before runtime construction");
    let original = CryptoProvider::get_default()
        .expect("provider was installed")
        .clone();

    let _tls = RustlsProvider::default();
    let selected = CryptoProvider::get_default().expect("provider remains installed");
    assert!(Arc::ptr_eq(&original, selected));
    assert_eq!(selected.cipher_suites.len(), 1);
}
