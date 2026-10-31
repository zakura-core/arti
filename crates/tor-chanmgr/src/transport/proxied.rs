//! Connect to relays via a proxy.
//!
//! Supports SOCKS4/4a/5 and HTTP CONNECT proxies.
#![allow(dead_code)]

use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use base64ct::{Base64, Encoding};
use futures::io::{AsyncBufReadExt, BufReader};
use futures::{AsyncReadExt, AsyncWriteExt};
use httparse;
use safelog::Sensitive;
use tor_rtcompat::NetStreamProvider;
use tor_socksproto::{
    Handshake as _, SocksAddr, SocksAuth, SocksClientHandshake, SocksCmd, SocksRequest,
    SocksStatus, SocksVersion,
};
use tracing::trace;

/// Information about what proxy protocol to use, and how to use it.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Protocol {
    /// Connect via SOCKS 4, SOCKS 4a, or SOCKS 5.
    Socks(SocksVersion, SocksAuth),
    /// Connect via HTTP CONNECT proxy (RFC 7231).
    HttpConnect {
        /// Optional Basic auth credentials (username, password) for Proxy-Authorization header.
        auth: Option<(Sensitive<String>, Sensitive<String>)>,
    },
}

/// Maximum number of bytes allowed in HTTP response headers from proxy.
const MAX_HTTP_HEADER_BYTES: usize = 16 * 1024;

/// Open a connection to `target` via the proxy at `proxy`, using the protocol
/// at `protocol`.
///
/// # Limitations
///
/// We will give an error if the proxy sends us any data on the connection along
/// with its final handshake: due to our implementation, any such data will be
/// discarded, and so we give an error rather than fail silently.
///
/// This limitation doesn't matter when the underlying protocol is Tor, or
/// anything else where the initiator is expected to speak before the responder
/// says anything.  To lift it, we would have to make this function's return
/// type become something buffered.
//
// TODO: Perhaps we should refactor this someday so it can be a general-purpose
// proxy function, not only for Arti.
pub(crate) async fn connect_via_proxy<R: NetStreamProvider + Send + Sync>(
    runtime: &R,
    proxy: &SocketAddr,
    protocol: &Protocol,
    target: &SocketAddr,
) -> Result<R::Stream, ProxyError> {
    trace!(
        "Launching a proxied connection to {} via proxy at {} using {:?}",
        target, proxy, protocol
    );
    // We don't (yet) use any custom options on the socket.
    let connect_options = Default::default();
    let stream = runtime
        .connect(proxy, &connect_options)
        .await
        .map_err(|e| ProxyError::ProxyConnect(Arc::new(e)))?;

    match protocol {
        Protocol::Socks(version, auth) => {
            do_socks_handshake::<R>(stream, version, auth, target).await
        }
        Protocol::HttpConnect { auth } => {
            do_http_connect_handshake::<R>(stream, auth, target).await
        }
    }
}

/// Perform SOCKS proxy handshake.
async fn do_socks_handshake<R: NetStreamProvider + Send + Sync>(
    mut stream: R::Stream,
    version: &SocksVersion,
    auth: &SocksAuth,
    target: &SocketAddr,
) -> Result<R::Stream, ProxyError> {
    let target_addr = SocksAddr::Ip(target.ip());
    let target_port = target.port();

    let request = SocksRequest::new(
        *version,
        SocksCmd::CONNECT,
        target_addr,
        target_port,
        auth.clone(),
    )
    .map_err(ProxyError::InvalidSocksRequest)?;
    let mut handshake = SocksClientHandshake::new(request);

    let mut buf = tor_socksproto::Buffer::new();
    let reply = loop {
        use tor_socksproto::NextStep as NS;
        match handshake.step(&mut buf).map_err(ProxyError::SocksProto)? {
            NS::Send(send) => {
                stream.write_all(&send).await?;
                stream.flush().await?;
            }
            NS::Finished(fin) => {
                break fin
                    .into_output_forbid_pipelining()
                    .map_err(ProxyError::SocksProto)?;
            }
            NS::Recv(mut recv) => {
                let n = stream.read(recv.buf()).await?;
                recv.note_received(n).map_err(ProxyError::SocksProto)?;
            }
        }
    };

    let status = reply.status();
    trace!("SOCKS handshake succeeded, status {:?}", status);

    if status != SocksStatus::SUCCEEDED {
        return Err(ProxyError::SocksError(status));
    }

    Ok(stream)
}

/// Format target address for HTTP CONNECT request line and Host header.
fn format_connect_target(target: &SocketAddr) -> String {
    let host = match target.ip() {
        IpAddr::V4(ip) => ip.to_string(),
        IpAddr::V6(ip) => format!("[{ip}]"),
    };
    format!("{}:{}", host, target.port())
}

/// Build HTTP CONNECT request string with optional Basic auth.
fn build_http_connect_request(
    target_str: &str,
    auth: &Option<(Sensitive<String>, Sensitive<String>)>,
) -> String {
    // Build CONNECT request: CONNECT host:port HTTP/1.1\r\nHost: host:port\r\n[...]\r\n
    let mut request = format!(
        "CONNECT {} HTTP/1.1\r\nHost: {}\r\n",
        target_str, target_str
    );

    if let Some((user, pass)) = auth {
        // Proxy-Authorization: Basic base64(username:password) per RFC 7617
        let credentials = format!("{}:{}", user.as_ref(), pass.as_ref());
        let encoded = Base64::encode_string(credentials.as_bytes());
        request.push_str(&format!("Proxy-Authorization: Basic {}\r\n", encoded));
    }

    request.push_str("\r\n");
    request
}

/// Parse HTTP CONNECT response and extract status code.
///
/// Uses httparse for spec-compliant parsing. Rejects:
/// - Responses larger than 16KB (prevents header bomb attacks)
/// - Pipelined data after headers (connection is dedicated to tunnel)
fn parse_http_connect_response(response_bytes: &[u8]) -> Result<u16, ProxyError> {
    if response_bytes.len() > MAX_HTTP_HEADER_BYTES {
        return Err(ProxyError::HttpConnectMalformed);
    }

    let mut headers = [httparse::EMPTY_HEADER; 64];
    let mut resp = httparse::Response::new(&mut headers);

    match resp.parse(response_bytes) {
        Ok(httparse::Status::Complete(header_end)) => {
            let status = resp.code.ok_or(ProxyError::HttpConnectMalformed)?;

            if !(200..300).contains(&status) {
                return Err(ProxyError::HttpConnectError(status));
            }

            // Reject any pipelined data after headers
            if header_end < response_bytes.len() {
                return Err(ProxyError::UnexpectedData);
            }

            trace!("HTTP CONNECT successful, status {}", status);
            Ok(status)
        }
        Ok(httparse::Status::Partial) => Err(ProxyError::HttpConnectMalformed),
        Err(_) => Err(ProxyError::HttpConnectMalformed),
    }
}

/// Send HTTP CONNECT request to proxy.
async fn send_http_connect_request<R: NetStreamProvider + Send + Sync>(
    stream: &mut R::Stream,
    auth: &Option<(Sensitive<String>, Sensitive<String>)>,
    target_str: &str,
) -> Result<(), ProxyError> {
    let request = build_http_connect_request(target_str, auth);
    trace!("Sending HTTP CONNECT request for {}", target_str);
    stream.write_all(request.as_bytes()).await?;
    stream.flush().await?;
    Ok(())
}

/// Perform HTTP CONNECT proxy handshake (RFC 7231, RFC 7617 for Basic auth).
async fn do_http_connect_handshake<R: NetStreamProvider + Send + Sync>(
    mut stream: R::Stream,
    auth: &Option<(Sensitive<String>, Sensitive<String>)>,
    target: &SocketAddr,
) -> Result<R::Stream, ProxyError> {
    let target_str = format_connect_target(target);
    send_http_connect_request::<R>(&mut stream, auth, &target_str).await?;

    // Read response until we see the double CRLF that terminates headers,
    // OR we reach MAX_HTTP_HEADER_BYTES.
    let mut response_buffer = Vec::new();
    let limit = MAX_HTTP_HEADER_BYTES as u64;
    let mut reader = BufReader::new(stream.take(limit));
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            // We got an EOF (either from take(), or from the underlying stream),
            // before we could finish reading the headers.
            return Err(ProxyError::HttpConnectMalformed);
        }

        response_buffer.extend_from_slice(line.as_bytes());

        // Check for blank line (end of headers)
        if line == "\r\n" || line == "\n" {
            break;
        }
    }

    if !reader.buffer().is_empty() {
        // We have extraneous data after the HTTP response header; it is about to be discarded,
        // which would be an error.
        //
        // LIMITATION: Technically, this might be data from the Tor relay we're contacting.
        // However, we only use this method for negotiating Tor channels, which use TLS.
        // In TLS, the responder is supposed to be silent until the initiator says something,
        // so if we do have data here, then it is a TLS protocol violation.
        return Err(ProxyError::HttpConnectMalformed);
    }

    // Parse and validate response
    let _status_code = parse_http_connect_response(&response_buffer)?;

    // Return the underlying stream (BufReader is dropped, Take is dropped, stream continues)
    Ok(reader.into_inner().into_inner())
}

/// An error that occurs while negotiating a connection with a proxy.
#[derive(Clone, Debug, thiserror::Error)]
#[non_exhaustive]
pub enum ProxyError {
    /// We had an IO error while trying to open a connection to the proxy.
    #[error("Problem while connecting to proxy")]
    ProxyConnect(#[source] Arc<std::io::Error>),

    /// We had an IO error while talking to the proxy.
    #[error("Problem while communicating with proxy")]
    ProxyIo(#[source] Arc<std::io::Error>),

    /// Our SOCKS implementation told us that this request cannot be encoded.
    #[error("Tried to make an invalid SOCKS request")]
    InvalidSocksRequest(#[source] tor_socksproto::Error),

    /// The peer refused our request, or spoke SOCKS incorrectly.
    #[error("Protocol error while communicating with SOCKS proxy")]
    SocksProto(#[source] tor_socksproto::Error),

    /// We encountered an internal programming error.
    #[error("Internal error")]
    Bug(#[from] tor_error::Bug),

    /// We got extra data immediately after our handshake, before we actually
    /// sent anything.
    ///
    /// This is not a bug in the calling code or in the peer protocol: it just
    /// means that the remote peer sent us data before we actually sent it any
    /// data. Unfortunately, there's a limitation in our code that makes it
    /// discard any such data, and therefore we have to give this error to
    /// prevent bugs.
    ///
    /// We could someday remove this limitation.
    #[error("Received unexpected early data from peer")]
    UnexpectedData,

    /// The proxy told us that our attempt failed.
    #[error("SOCKS proxy reported an error: {0}")]
    SocksError(SocksStatus),

    /// HTTP CONNECT proxy returned a non-2xx status code.
    #[error("HTTP CONNECT proxy returned status: {0}")]
    HttpConnectError(u16),

    /// HTTP CONNECT proxy returned a malformed response.
    #[error("HTTP CONNECT proxy returned invalid response")]
    HttpConnectMalformed,
}

impl From<std::io::Error> for ProxyError {
    fn from(e: std::io::Error) -> Self {
        ProxyError::ProxyIo(Arc::new(e))
    }
}

impl From<ProxyError> for std::io::Error {
    fn from(e: ProxyError) -> Self {
        std::io::Error::other(e)
    }
}

impl tor_error::HasKind for ProxyError {
    fn kind(&self) -> tor_error::ErrorKind {
        use ProxyError as E;
        use tor_error::ErrorKind as EK;
        match self {
            E::ProxyConnect(_) | E::ProxyIo(_) => EK::LocalNetworkError,
            E::InvalidSocksRequest(_) => EK::BadApiUsage,
            E::SocksProto(_) => EK::LocalProtocolViolation,
            E::Bug(e) => e.kind(),
            E::UnexpectedData => EK::NotImplemented,
            E::SocksError(_) => EK::LocalProtocolViolation,
            E::HttpConnectError(_) | E::HttpConnectMalformed => EK::LocalProtocolViolation,
        }
    }
}

impl tor_error::HasRetryTime for ProxyError {
    fn retry_time(&self) -> tor_error::RetryTime {
        use ProxyError as E;
        use SocksStatus as S;
        use tor_error::RetryTime as RT;
        match self {
            E::ProxyConnect(_) | E::ProxyIo(_) => RT::AfterWaiting,
            E::InvalidSocksRequest(_) => RT::Never,
            E::SocksProto(_) => RT::AfterWaiting,
            E::Bug(_) => RT::Never,
            E::UnexpectedData => RT::Never,
            E::SocksError(e) => match *e {
                S::CONNECTION_REFUSED
                | S::GENERAL_FAILURE
                | S::HOST_UNREACHABLE
                | S::NETWORK_UNREACHABLE
                | S::TTL_EXPIRED => RT::AfterWaiting,
                _ => RT::Never,
            },
            E::HttpConnectError(code) => {
                // 502/503/504 may be transient; auth and policy errors are not.
                if *code == 502 || *code == 503 || *code == 504 {
                    RT::AfterWaiting
                } else {
                    RT::Never
                }
            }
            E::HttpConnectMalformed => RT::Never,
        }
    }
}

#[cfg(test)]
mod test {
    // @@ begin test lint list maintained by maint/add_warning @@
    #![allow(clippy::bool_assert_comparison)]
    #![allow(clippy::clone_on_copy)]
    #![allow(clippy::dbg_macro)]
    #![allow(clippy::mixed_attributes_style)]
    #![allow(clippy::print_stderr)]
    #![allow(clippy::print_stdout)]
    #![allow(clippy::single_char_pattern)]
    #![allow(clippy::unwrap_used)]
    #![allow(clippy::unchecked_time_subtraction)]
    #![allow(clippy::useless_vec)]
    #![allow(clippy::needless_pass_by_value)]
    #![allow(clippy::string_slice)] // See arti#2571
    //! <!-- @@ end test lint list maintained by maint/add_warning @@ -->
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn protocol_debug_redacts_http_connect_auth() {
        let proto = Protocol::HttpConnect {
            auth: Some((
                Sensitive::new("user_name".to_owned()),
                Sensitive::new("pass_word".to_owned()),
            )),
        };

        let formatted = format!("{proto:?}");
        assert!(formatted.contains("HttpConnect"));
        assert!(!formatted.contains("user_name"));
        assert!(!formatted.contains("pass_word"));
    }

    #[test]
    fn connect_target_uses_numeric_relay_address() {
        for (address, expected) in [
            ("192.0.2.1:9001", "192.0.2.1:9001"),
            ("[2001:db8::1]:443", "[2001:db8::1]:443"),
            ("[fe80::1%4]:443", "[fe80::1]:443"),
        ] {
            let target = address.parse().unwrap();
            assert_eq!(format_connect_target(&target), expected);
        }
    }

    #[test]
    fn parse_http_connect_200_ok() {
        let response = b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";
        assert_eq!(parse_http_connect_response(response).unwrap(), 200);
    }

    #[test]
    fn parse_http_connect_407_auth_required() {
        let response = b"HTTP/1.1 407 Proxy Authentication Required\r\n\r\n";
        match parse_http_connect_response(response) {
            Err(ProxyError::HttpConnectError(407)) => (), // Expected
            other => panic!("Expected 407 error, got {:?}", other),
        }
    }

    #[test]
    fn parse_http_connect_malformed_no_status() {
        let response = b"INVALID HTTP";
        assert!(matches!(
            parse_http_connect_response(response),
            Err(ProxyError::HttpConnectMalformed)
        ));
    }

    #[test]
    fn parse_http_connect_with_headers() {
        let response = b"HTTP/1.1 200 Connection Established\r\nConnection: close\r\nProxy-Agent: Proxy/1.0\r\n\r\n";
        assert_eq!(parse_http_connect_response(response).unwrap(), 200);
    }

    #[test]
    fn parse_http_connect_rejects_pipelined_data() {
        let response = b"HTTP/1.1 200 OK\r\n\r\nEXTRA_DATA";
        assert!(matches!(
            parse_http_connect_response(response),
            Err(ProxyError::UnexpectedData)
        ));
    }

    #[test]
    fn parse_http_connect_oversized_headers() {
        let huge_header = vec![b'X'; MAX_HTTP_HEADER_BYTES + 1];
        assert!(matches!(
            parse_http_connect_response(&huge_header),
            Err(ProxyError::HttpConnectMalformed)
        ));
    }
}
