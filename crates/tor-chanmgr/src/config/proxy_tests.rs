//! Fixed vectors for proxy URI parsing and credential formatting.

use super::{
    HTTP_SCHEME, HttpConnectAuth, ProxyProtocol, ProxyProtocolParseError, SOCKS4_SCHEME,
    SOCKS4A_SCHEME, SOCKS5_SCHEME, SOCKS5H_SCHEME,
};
use std::net::{IpAddr, Ipv6Addr, SocketAddr};
use tor_socksproto::{SocksAuth, SocksVersion};

/// Input spellings and addresses, independent of proxy URI parsing.
fn proxy_uri_forms(scheme: &str, userinfo: &str) -> Vec<(String, SocketAddr)> {
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
    let mut forms = Vec::new();
    for spelling in [scheme.to_owned(), scheme.to_ascii_uppercase(), mixed] {
        for (host, ip) in [
            ("127.0.0.1", IpAddr::from([127, 0, 0, 1])),
            ("192.0.2.10", IpAddr::from([192, 0, 2, 10])),
            ("[::1]", IpAddr::V6(Ipv6Addr::LOCALHOST)),
            (
                "[2001:db8::8]",
                IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 8)),
            ),
        ] {
            for port in [0, 1, 80, 1080, u16::MAX] {
                let uri = format!("{spelling}://{userinfo}{host}:{port}");
                let addr = SocketAddr::new(ip, port);
                forms.push((format!("{uri}/"), addr));
                forms.push((uri, addr));
            }
        }
    }
    forms
}

#[test]
fn documented_socks_forms_have_expected_values() {
    // Each row fixes the SOCKS4 and SOCKS5 authentication values.
    // None in the SOCKS4 column means a password must be rejected.
    let cases = [
        ("", Some(SocksAuth::NoAuth), SocksAuth::NoAuth),
        ("@", Some(SocksAuth::NoAuth), SocksAuth::NoAuth),
        (":@", Some(SocksAuth::NoAuth), SocksAuth::NoAuth),
        (
            "user@",
            Some(SocksAuth::Socks4(b"user".to_vec())),
            SocksAuth::Username(b"user".to_vec(), Vec::new()),
        ),
        (
            "user:@",
            Some(SocksAuth::Socks4(b"user".to_vec())),
            SocksAuth::Username(b"user".to_vec(), Vec::new()),
        ),
        (
            "user:pass@",
            None,
            SocksAuth::Username(b"user".to_vec(), b"pass".to_vec()),
        ),
        (
            ":pass@",
            None,
            SocksAuth::Username(Vec::new(), b"pass".to_vec()),
        ),
        (
            "us%65r:pass@",
            None,
            SocksAuth::Username(b"user".to_vec(), b"pass".to_vec()),
        ),
        (
            "user%40domain:pass%3Aword@",
            None,
            SocksAuth::Username(b"user@domain".to_vec(), b"pass:word".to_vec()),
        ),
        (
            "ümlaut:påss@",
            None,
            SocksAuth::Username("ümlaut".as_bytes().to_vec(), "påss".as_bytes().to_vec()),
        ),
        (
            "user:p@ss@",
            None,
            SocksAuth::Username(b"user".to_vec(), b"p@ss".to_vec()),
        ),
        (
            "u%zz:p%2@",
            None,
            SocksAuth::Username(b"u%zz".to_vec(), b"p%2".to_vec()),
        ),
    ];
    let mut checked = 0;
    for (scheme, version) in [
        (SOCKS4_SCHEME, SocksVersion::V4),
        (SOCKS4A_SCHEME, SocksVersion::V4),
        (SOCKS5_SCHEME, SocksVersion::V5),
        (SOCKS5H_SCHEME, SocksVersion::V5),
    ] {
        for (userinfo, socks4, socks5) in &cases {
            let auth = match version {
                SocksVersion::V4 => socks4.as_ref(),
                SocksVersion::V5 => Some(socks5),
                _ => unreachable!(),
            };
            for (uri, addr) in proxy_uri_forms(scheme, userinfo) {
                let actual = uri.parse::<ProxyProtocol>();
                if let Some(auth) = auth {
                    assert_eq!(
                        actual.expect(&uri),
                        ProxyProtocol::Socks {
                            version,
                            auth: auth.clone(),
                            addr,
                        },
                        "{uri:?}"
                    );
                } else {
                    assert!(
                        matches!(actual,
                        Err(ProxyProtocolParseError::UnsupportedPassword(s)) if s == scheme),
                        "expected unsupported password for {uri:?}"
                    );
                }
                checked += 1;
            }
        }
    }
    assert_eq!(checked, 5760);
}

#[test]
fn documented_http_forms_have_expected_values() {
    let auth = |username: &str, password: Option<&str>| {
        Ok(Some(HttpConnectAuth {
            username: username.to_owned(),
            password: password.map(str::to_owned),
        }))
    };
    let cases = [
        ("", Ok(None)),
        ("@", Ok(None)),
        (":@", Ok(None)),
        ("user@", auth("user", None)),
        ("user:@", auth("user", None)),
        ("user:pass@", auth("user", Some("pass"))),
        (":pass@", Err(())),
        ("us%65r:pass@", auth("user", Some("pass"))),
        (
            "user%40domain:pass%3Aword@",
            auth("user@domain", Some("pass:word")),
        ),
        ("ümlaut:påss@", auth("ümlaut", Some("påss"))),
        ("user:p@ss@", auth("user", Some("p@ss"))),
        ("u%zz:p%2@", auth("u%zz", Some("p%2"))),
    ];
    let mut checked = 0;
    for (userinfo, credentials) in cases {
        for (uri, addr) in proxy_uri_forms(HTTP_SCHEME, userinfo) {
            let actual = uri.parse::<ProxyProtocol>();
            if let Ok(credentials) = &credentials {
                assert_eq!(
                    actual.expect(&uri),
                    ProxyProtocol::HttpConnect {
                        addr,
                        credentials: credentials.clone(),
                    },
                    "{uri:?}"
                );
            } else {
                assert!(
                    matches!(actual, Err(ProxyProtocolParseError::InvalidFormat(s))
                    if s == "password without username not supported"),
                    "expected password-without-username error for {uri:?}"
                );
            }
            checked += 1;
        }
    }
    assert_eq!(checked, 1440);
}

#[test]
fn malformed_proxy_forms_fail() {
    for scheme in [HTTP_SCHEME, SOCKS4_SCHEME, SOCKS5_SCHEME] {
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
            assert!(uri.parse::<ProxyProtocol>().is_err(), "accepted {uri:?}");
        }
    }
}

#[test]
fn numeric_addresses_and_paths_are_not_normalized() {
    for uri in [
        "http://127.1:1080",
        "http://0x7f000001:1080",
        "http://127.0.0.1.:1080",
        "http://127.0.0.1:1080/a/..",
        "http:////127.0.0.1:1080",
        " http://127.0.0.1:1080 ",
        "http://us\ter@127.0.0.1:1080",
    ] {
        assert!(uri.parse::<ProxyProtocol>().is_err(), "accepted {uri:?}");
    }
}

#[test]
fn explicit_http_default_port_is_preserved() {
    let uri = "http://user:pass@127.0.0.1:80";
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

#[test]
fn credential_formatting_has_expected_encoding() {
    // Fixed WHATWG userinfo encodings for every ASCII byte, including controls.
    const ASCII: [&str; 128] = [
        "%00", "%01", "%02", "%03", "%04", "%05", "%06", "%07", "%08", "%09", "%0A", "%0B", "%0C",
        "%0D", "%0E", "%0F", "%10", "%11", "%12", "%13", "%14", "%15", "%16", "%17", "%18", "%19",
        "%1A", "%1B", "%1C", "%1D", "%1E", "%1F", "%20", "!", "%22", "%23", "$", "%", "&", "'",
        "(", ")", "*", "+", ",", "-", ".", "%2F", "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
        "%3A", "%3B", "%3C", "%3D", "%3E", "%3F", "%40", "A", "B", "C", "D", "E", "F", "G", "H",
        "I", "J", "K", "L", "M", "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z",
        "%5B", "%5C", "%5D", "%5E", "_", "%60", "a", "b", "c", "d", "e", "f", "g", "h", "i", "j",
        "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z", "%7B",
        "%7C", "%7D", "~", "%7F",
    ];
    let mut values = (0_u8..=127)
        .zip(ASCII)
        .map(|(byte, encoded)| {
            (
                format!("pre{}post", char::from(byte)),
                format!("pre{encoded}post"),
            )
        })
        .collect::<Vec<_>>();
    values.extend([
        ("".to_owned(), "".to_owned()),
        (
            "ümlaut🦀%2f".to_owned(),
            "%C3%BCmlaut%F0%9F%A6%80%2f".to_owned(),
        ),
        (
            String::from_utf8_lossy(&[0xff, 0x80]).into_owned(),
            "%EF%BF%BD%EF%BF%BD".to_owned(),
        ),
    ]);
    let addr = "[2001:db8::1]:1080".parse().expect("address");
    for (value, encoded) in values {
        let proxy = ProxyProtocol::Socks {
            version: SocksVersion::V4,
            auth: SocksAuth::Socks4(value.as_bytes().to_vec()),
            addr,
        };
        assert_eq!(proxy.to_string(), format!("socks4://{encoded}@{addr}"));
        for (pass, password) in [
            (None, None),
            (Some(""), None),
            (
                Some("p:ass@+%=;?#/\\🦀"),
                Some("p%3Aass%40+%%3D%3B%3F%23%2F%5C%F0%9F%A6%80"),
            ),
        ] {
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

#[test]
fn scoped_address_formatting_has_expected_encoding() {
    let addr = SocketAddr::V6(std::net::SocketAddrV6::new(
        "fe80::1".parse().expect("IPv6 address"),
        1080,
        0,
        3,
    ));
    for (proxy, expected) in [
        (
            ProxyProtocol::HttpConnect {
                addr,
                credentials: Some(HttpConnectAuth {
                    username: "u@name".to_owned(),
                    password: Some("p:word".to_owned()),
                }),
            },
            "http://u%40name:p%3Aword@[fe80::1%3]:1080",
        ),
        (
            ProxyProtocol::Socks {
                version: SocksVersion::V5,
                auth: SocksAuth::Username(b"u@name".to_vec(), b"p:word".to_vec()),
                addr,
            },
            "socks5://u%40name:p%3Aword@[fe80::1%3]:1080",
        ),
    ] {
        assert_eq!(proxy.to_string(), expected);
        // Scoped addresses can be formatted but remain invalid URI inputs.
        assert!(expected.parse::<ProxyProtocol>().is_err());
    }
}
