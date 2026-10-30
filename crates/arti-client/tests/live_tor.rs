//! End-to-end smoke test for the public Tor network.
//!
//! Ordinary tests stay offline; the shared Linux CI job explicitly runs this
//! ignored test with the wallet's Tokio/Rustls feature selection.

#![cfg(all(feature = "tokio", feature = "rustls"))]

use anyhow::{Context, Result, ensure};
use arti_client::{TorClient, config::TorClientConfigBuilder};
use futures::io::{AsyncReadExt, AsyncWriteExt};
use std::time::Duration;
use tor_rtcompat::{SleepProviderExt, ToplevelBlockOn, tokio::TokioRustlsRuntime};

const LIVE_TIMEOUT: Duration = Duration::from_secs(240);
const MAX_RESPONSE_BYTES: u64 = 16 * 1024;
const CHECK_HOST: &str = "check.torproject.org";

/// A fresh bootstrap authenticates real relay channels, then a Tor exit must
/// carry an HTTPS request to the Tor Project's own exit-check service.
#[test]
#[ignore = "uses the public Tor network and requires outbound Internet access"]
fn public_tor_bootstrap_and_https_request() -> Result<()> {
    rustls::crypto::ring::default_provider()
        .install_default()
        .map_err(|_| anyhow::anyhow!("live Tor test must select its own Ring provider"))?;

    let directories = tempfile::tempdir().context("create isolated Tor state/cache root")?;
    let config = TorClientConfigBuilder::from_directories(
        directories.path().join("state"),
        directories.path().join("cache"),
    )
    .build()
    .context("build wallet-style Tor configuration")?;
    let runtime = TokioRustlsRuntime::create().context("create Tokio/Rustls runtime")?;

    runtime
        .block_on(runtime.timeout(LIVE_TIMEOUT, async {
            let client = TorClient::with_runtime(runtime.clone())
                .config(config)
                .create_bootstrapped()
                .await
                .context("bootstrap and authenticate channels against public Tor")?;
            let stream = client
                .connect((CHECK_HOST, 443))
                .await
                .context("build an exit circuit and open the Tor stream")?;

            // This connector validates the destination certificate using system
            // roots. The only TCP stream it receives comes from TorClient;
            // it has no direct-network fallback.
            let mut tls = async_native_tls::TlsConnector::new()
                .connect(CHECK_HOST, stream)
                .await
                .context("authenticate the Tor check HTTPS endpoint over Tor")?;
            tls.write_all(
                b"GET /api/ip HTTP/1.0\r\nHost: check.torproject.org\r\nAccept: application/json\r\nConnection: close\r\n\r\n",
            )
            .await
            .context("write the HTTPS request through Tor")?;
            tls.flush().await.context("flush the Tor stream")?;

            let mut response = Vec::new();
            tls.take(MAX_RESPONSE_BYTES + 1)
                .read_to_end(&mut response)
                .await
                .context("read the Tor check HTTPS response")?;
            ensure!(
                response.len() <= MAX_RESPONSE_BYTES as usize,
                "Tor check response exceeded the test's size bound"
            );
            let boundary = response
                .windows(4)
                .position(|bytes| bytes == b"\r\n\r\n")
                .context("Tor check did not return complete HTTP headers")?;
            let headers = std::str::from_utf8(&response[..boundary])
                .context("Tor check returned non-UTF-8 HTTP headers")?;
            let mut status = headers.lines().next().unwrap_or("").split_whitespace();
            ensure!(
                matches!(status.next(), Some("HTTP/1.0" | "HTTP/1.1"))
                    && status.next() == Some("200"),
                "Tor check did not return HTTP success"
            );
            // HTTP/1.0 requests avoid chunked framing; parse the bounded body
            // without adding another HTTP client/parser dependency to the test.
            let body: serde_json::Value = serde_json::from_slice(&response[boundary + 4..])
                .context("Tor check did not return its expected JSON response")?;
            ensure!(
                body.get("IsTor").and_then(serde_json::Value::as_bool) == Some(true),
                "the authenticated Tor check service did not identify a Tor exit"
            );
            Ok::<_, anyhow::Error>(())
        }))
        .context("public Tor bootstrap/request exceeded its four-minute deadline")??;
    Ok(())
}
