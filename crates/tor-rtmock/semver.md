BREAKING: Unix mock-provider implementations now use
`tor_rtcompat::unix::SocketAddr` and `NoAfUnixSocketSupport`; the direct normal
and development dependencies on `tor-general-addr` are removed. Unix addresses
remain the same standard-library type on Unix. The non-Unix placeholder and
unsupported-address error instead belong to the runtime crate; provider
associated types use those moved identities. Mock behavior is unchanged.
