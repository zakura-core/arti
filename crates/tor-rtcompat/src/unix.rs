//! Definitions related to unix domain socket support.
//!
//! To avoid confusion, don't import `SocketAddr` from this module directly;
//! instead, import the module and refer to `unix::SocketAddr`.

#[cfg(not(unix))]
use std::path::Path;

/// A replacement/re-export [`std::os::unix::net::SocketAddr`].
///
/// On Unix platforms, this is just a re-export of [`std::os::unix::net::SocketAddr`].
///
/// On non-Unix platforms, this type is an uninhabited placeholder that can never be instantiated.
#[cfg(unix)]
pub use std::os::unix::net::SocketAddr;

// Void is used only as a stand-in for unix::SocketAddr.  Avoid unused crate warning.
//
// (We can't express the right dependency condition in Cargo.toml,
// and anyway it's not worth trying to conditionally suppress the dependency.)
#[cfg(unix)]
use void as _;

/// Address for an AF_UNIX socket.
///
/// (This is an uninhabited placeholder implementations for platforms without AF_UNIX support.)
///
/// Note that we currently include Windows on platforms without AF_UNIX support:
/// When we use unix domain sockets in Arti, we rely on their filesystem-based security properties,
/// which we haven't yet had a chance to fully analyze on non-Unix platforms.
#[cfg(not(unix))]
#[derive(Debug, Clone)]
pub struct SocketAddr(void::Void);

#[cfg(not(unix))]
impl SocketAddr {
    /// Return true if this is an "unnamed" socket address.
    ///
    /// (Because this type is uninhabited, this method cannot actually be called.)
    pub fn is_unnamed(&self) -> bool {
        void::unreachable(self.0)
    }
    /// Return the pathname for this socket address, if it is "named".
    ///
    /// (Because this type is uninhabited, this method cannot actually be called.)
    pub fn as_pathname(&self) -> Option<&Path> {
        void::unreachable(self.0)
    }
    /// Attempt to construct an AF_UNIX socket address from the provided `path`.
    ///
    /// (Because this platform lacks AF_UNIX support, this method will always return an error.)
    pub fn from_pathname<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let _ = path;
        Err(NoAfUnixSocketSupport.into())
    }
}

/// Error: Unix domain sockets are not supported on this platform.
#[derive(Clone, Debug, Default, thiserror::Error)]
#[error("No support for unix domain sockets on this platform")]
#[non_exhaustive]
pub struct NoAfUnixSocketSupport;

/// Deprecated name for `NoAfUnixSocketSupport`
#[deprecated]
pub type NoUnixAddressSupport = NoAfUnixSocketSupport;

impl From<NoAfUnixSocketSupport> for std::io::Error {
    fn from(value: NoAfUnixSocketSupport) -> Self {
        std::io::Error::new(std::io::ErrorKind::Unsupported, value)
    }
}

#[cfg(unix)]
use crate::unix;

/// Helper: construct an unnamed SocketAddr.
#[cfg(unix)]
pub(crate) fn new_unnamed_socketaddr() -> std::io::Result<unix::SocketAddr> {
    // There SHOULD be a better way to do this in legitimate Rust!
    // But sadly, there isn't.
    unix::SocketAddr::from_pathname("")
}

/// Error: Tried to perform an operation on an unsupported kind of AF\_UNIX address.
///
/// (For example, you can't bind or connect to an unnamed address.)
#[derive(Clone, Debug, thiserror::Error)]
#[error("Operation not supported on this kind of AF_UNIX address")]
#[non_exhaustive]
pub struct UnsupportedAfUnixAddressType;

/// Deprecated name for `UnsupportedAfUnixAddressType`
#[deprecated]
pub type UnsupportedUnixAddressType = UnsupportedAfUnixAddressType;

impl From<UnsupportedAfUnixAddressType> for std::io::Error {
    fn from(value: UnsupportedAfUnixAddressType) -> Self {
        std::io::Error::new(std::io::ErrorKind::Unsupported, value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[cfg(not(unix))]
    fn unsupported_address_preserves_io_error_kind_and_source() {
        let error = SocketAddr::from_pathname("a-socket")
            .expect_err("this platform does not support Unix sockets");
        assert_eq!(error.kind(), std::io::ErrorKind::Unsupported);
        assert!(
            error
                .get_ref()
                .is_some_and(|source| source.downcast_ref::<NoAfUnixSocketSupport>().is_some())
        );
    }

    #[test]
    #[cfg(unix)]
    fn unnamed() {
        let u = new_unnamed_socketaddr().expect("couldn't construct unnamed AF_UNIX socketaddr");
        assert!(u.is_unnamed());
        assert!(u.as_pathname().is_none());

        let n = unix::SocketAddr::from_pathname("/home/arachnidsGrip/.arti/SOCKET")
            .expect("Couldn't construct named socketaddr");
        assert!(!n.is_unnamed());
    }
}
