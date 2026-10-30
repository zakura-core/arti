//! Oracle copied from Arti commit 4a3cee32e, before removing URL parsing.

use super::{HttpConnectAuth, ProxyProtocol, ProxyProtocolParseError};
use percent_encoding::percent_decode_str;
use std::net::{IpAddr, SocketAddr};
use tor_socksproto::{SocksAuth, SocksVersion};
use url::{Host, Url};

fn previous(s: &str) -> Result<ProxyProtocol, ProxyProtocolParseError> {
    let url = Url::parse(s).map_err(|e| match e {
        url::ParseError::InvalidPort => ProxyProtocolParseError::InvalidPort,
        url::ParseError::InvalidIpv4Address
        | url::ParseError::InvalidIpv6Address
        | url::ParseError::EmptyHost
        | url::ParseError::InvalidDomainCharacter
        | url::ParseError::IdnaError => ProxyProtocolParseError::InvalidAddress(s.to_string()),
        _ => ProxyProtocolParseError::InvalidFormat(s.to_string()),
    })?;

    let scheme_lower = url.scheme().to_ascii_lowercase();

    if url.query().is_some() || url.fragment().is_some() {
        return Err(ProxyProtocolParseError::InvalidFormat(s.to_string()));
    }

    let path = url.path();
    if !path.is_empty() && path != "/" {
        return Err(ProxyProtocolParseError::InvalidFormat(s.to_string()));
    }

    let port = url.port().ok_or(ProxyProtocolParseError::InvalidPort)?;
    let host = url
        .host()
        .ok_or_else(|| ProxyProtocolParseError::InvalidAddress(s.to_string()))?;
    let ip = match host {
        Host::Ipv4(ip) => IpAddr::V4(ip),
        Host::Ipv6(ip) => IpAddr::V6(ip),
        Host::Domain(domain) => domain
            .parse::<IpAddr>()
            .map_err(|_| ProxyProtocolParseError::InvalidAddress(domain.to_string()))?,
    };
    let addr = SocketAddr::new(ip, port);

    match scheme_lower.as_str() {
        "http" => {
            // HTTP CONNECT: optional Basic auth via user:pass@host:port
            let user = url.username();
            let pass = url.password();
            // Reject password-only auth (http://:pass@host:port) - username is required
            if user.is_empty() && pass.is_some() {
                return Err(ProxyProtocolParseError::InvalidFormat(
                    "password without username not supported".to_string(),
                ));
            }
            let credentials = if user.is_empty() {
                None
            } else {
                let username = percent_decode_str(user)
                    .decode_utf8()
                    .map_err(|_| {
                        ProxyProtocolParseError::InvalidFormat(
                            "invalid UTF-8 in username".to_string(),
                        )
                    })?
                    .into_owned();
                let password = pass
                    .map(|p| {
                        percent_decode_str(p).decode_utf8().map_err(|_| {
                            ProxyProtocolParseError::InvalidFormat(
                                "invalid UTF-8 in password".to_string(),
                            )
                        })
                    })
                    .transpose()?
                    .map(|s| s.into_owned());
                Some(HttpConnectAuth { username, password })
            };
            Ok(ProxyProtocol::HttpConnect { addr, credentials })
        }
        "socks4" | "socks4a" | "socks5" | "socks5h" => {
            let version = match scheme_lower.as_str() {
                "socks4" | "socks4a" => SocksVersion::V4,
                "socks5" | "socks5h" => SocksVersion::V5,
                _ => unreachable!(),
            };
            // Check for authentication credentials (user:pass@host:port or user@host:port).
            let user = url.username();
            let pass = url.password();
            if version == SocksVersion::V4 && pass.is_some() {
                return Err(ProxyProtocolParseError::UnsupportedPassword(
                    url.scheme().to_string(),
                ));
            }
            let user_decoded = percent_decode_str(user).decode_utf8().map_err(|_| {
                ProxyProtocolParseError::InvalidFormat("invalid UTF-8 in username".to_string())
            })?;
            let pass_decoded = pass
                .map(|p| {
                    percent_decode_str(p).decode_utf8().map_err(|_| {
                        ProxyProtocolParseError::InvalidFormat(
                            "invalid UTF-8 in password".to_string(),
                        )
                    })
                })
                .transpose()?;
            let auth = if user.is_empty() && pass.is_none() {
                SocksAuth::NoAuth
            } else {
                match version {
                    SocksVersion::V4 => SocksAuth::Socks4(user_decoded.as_bytes().to_vec()),
                    SocksVersion::V5 => {
                        let pass = pass_decoded.as_deref().unwrap_or("");
                        SocksAuth::Username(
                            user_decoded.as_bytes().to_vec(),
                            pass.as_bytes().to_vec(),
                        )
                    }
                    _ => SocksAuth::NoAuth,
                }
            };
            Ok(ProxyProtocol::Socks {
                version,
                auth,
                addr,
            })
        }
        _ => Err(ProxyProtocolParseError::UnsupportedScheme(
            url.scheme().to_string(),
        )),
    }
}

fn assert_matches_previous(uri: &str) {
    match (previous(uri), uri.parse::<ProxyProtocol>()) {
        (Ok(previous), Ok(actual)) => assert_eq!(actual, previous, "{uri:?}"),
        (Err(_), Err(_)) => {}
        (previous, actual) => panic!("parser mismatch for {uri:?}: {previous:?} / {actual:?}"),
    }
}

#[test]
fn documented_proxy_forms_match_previous() {
    let mut comparisons = 0;
    for scheme in ["socks4", "socks4a", "socks5", "socks5h", "http"] {
        let mixed = scheme
            .chars()
            .enumerate()
            .map(|(index, c)| {
                if index % 2 == 0 {
                    c.to_ascii_uppercase()
                } else {
                    c
                }
            })
            .collect::<String>();
        for spelling in [scheme.to_owned(), scheme.to_ascii_uppercase(), mixed] {
            for host in ["127.0.0.1", "192.0.2.10", "[::1]", "[2001:db8::8]"] {
                for port in [0, 1, 80, 1080, 65535] {
                    // URL normalizes HTTP's explicit default port away. The new
                    // parser deliberately fixes that rejection, tested below.
                    if scheme == "http" && port == 80 {
                        continue;
                    }
                    for userinfo in [
                        "",
                        "@",
                        ":@",
                        "user@",
                        "user:@",
                        "user:pass@",
                        ":pass@",
                        "us%65r:pass@",
                        "user%40domain:pass%3Aword@",
                        "ümlaut:påss@",
                        "user:p@ss@",
                        "u%zz:p%2@",
                    ] {
                        let uri = format!("{spelling}://{userinfo}{host}:{port}");
                        assert_matches_previous(&uri);
                        assert_matches_previous(&format!("{uri}/"));
                        comparisons += 2;
                    }
                }
            }
        }
    }
    assert_eq!(comparisons, 6912);
}

#[test]
fn malformed_proxy_forms_still_fail() {
    for scheme in ["http", "socks4", "socks5"] {
        for suffix in [
            "",
            "127.0.0.1",
            "127.0.0.1:",
            "127.0.0.1:-1",
            "127.0.0.1:+1",
            "127.0.0.1:65536",
            "127.0.0.1:1?",
            "127.0.0.1:1#",
            "127.0.0.1:1/path",
            "localhost:1",
            "éxample.org:1",
            "[::1]:",
            "[::1]:65536",
            "[::1:1",
            "[::1%1]:1",
            "%ff@127.0.0.1:1",
            "user:%c0%80@127.0.0.1:1",
        ] {
            let uri = format!("{scheme}://{suffix}");
            assert!(
                previous(&uri).is_err(),
                "legacy unexpectedly accepts {uri:?}"
            );
            assert!(uri.parse::<ProxyProtocol>().is_err(), "accepted {uri:?}");
        }
    }
}

#[test]
fn numeric_address_parser_rejects_url_normalization() {
    for uri in [
        "http://127.1:1080",
        "http://0x7f000001:1080",
        "http://127.0.0.1.:1080",
        "http://127.0.0.1:1080/a/..",
        "http:////127.0.0.1:1080",
        " http://127.0.0.1:1080 ",
        "http://us\ter@127.0.0.1:1080",
    ] {
        assert!(previous(uri).is_ok(), "legacy unexpectedly rejects {uri:?}");
        assert!(uri.parse::<ProxyProtocol>().is_err(), "accepted {uri:?}");
    }
}

#[test]
fn explicit_http_default_port_is_preserved() {
    let uri = "http://user:pass@127.0.0.1:80";
    assert!(previous(uri).is_err());
    assert_eq!(
        uri.parse::<ProxyProtocol>().expect("explicit port"),
        ProxyProtocol::HttpConnect {
            addr: "127.0.0.1:80".parse().expect("address"),
            credentials: Some(HttpConnectAuth {
                username: "user".to_owned(),
                password: Some("pass".to_owned()),
            }),
        },
    );
}

fn previous_userinfo(scheme: &str, user: &str, pass: Option<&str>) -> (String, Option<String>) {
    let mut url = Url::parse(&format!("{scheme}://127.0.0.1:1080")).expect("fixed address");
    url.set_username(user).expect("username");
    url.set_password(pass).expect("password");
    (url.username().to_owned(), url.password().map(str::to_owned))
}

#[test]
fn credential_formatting_matches_previous() {
    let mut values = (0_u8..=127)
        .map(|byte| format!("pre{}post", char::from(byte)))
        .collect::<Vec<_>>();
    values.extend([
        "".to_owned(),
        "ümlaut🦀%2f".to_owned(),
        String::from_utf8_lossy(&[0xff, 0x80]).into_owned(),
    ]);
    let addr = "[2001:db8::1]:1080".parse().expect("address");
    for value in &values {
        let (encoded, _) = previous_userinfo("socks4", value, None);
        let proxy = ProxyProtocol::Socks {
            version: SocksVersion::V4,
            auth: SocksAuth::Socks4(value.as_bytes().to_vec()),
            addr,
        };
        assert_eq!(proxy.to_string(), format!("socks4://{encoded}@{addr}"));
        for pass in [None, Some(""), Some("p:ass@+%=;?#/\\🦀")] {
            let (encoded, password) = previous_userinfo("http", value, pass);
            let proxy = ProxyProtocol::HttpConnect {
                addr,
                credentials: Some(HttpConnectAuth {
                    username: value.clone(),
                    password: pass.map(str::to_owned),
                }),
            };
            let expected = match password {
                Some(password) => format!("http://{encoded}:{password}@{addr}"),
                None => format!("http://{encoded}@{addr}"),
            };
            assert_eq!(proxy.to_string(), expected);
            let (encoded, password) = previous_userinfo("socks5", value, pass);
            let proxy = ProxyProtocol::Socks {
                version: SocksVersion::V5,
                auth: SocksAuth::Username(
                    value.as_bytes().to_vec(),
                    pass.unwrap_or("").as_bytes().to_vec(),
                ),
                addr,
            };
            assert_eq!(
                proxy.to_string(),
                format!("socks5://{encoded}:{}@{addr}", password.unwrap_or_default())
            );
        }
    }
}
