//! Methods for opening a channel to a relay.

use std::net::SocketAddr;

use crate::HasAddrs;

/// The way to approach a single relay in order to open a channel.
///
/// Connect to one of the relay's advertised socket addresses.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum ChannelMethod {
    /// Connect to the relay directly at one of several addresses.
    Direct(Vec<std::net::SocketAddr>),
}

impl ChannelMethod {
    /// Return all the advertized socket addresses to which this method may connect.
    ///
    /// Returns `Some(&[])` if there is no way to connect to this target.
    pub fn socket_addrs(&self) -> Option<&[std::net::SocketAddr]> {
        match self {
            ChannelMethod::Direct(addr) => Some(addr.as_ref()),
        }
    }

    /// Return the [`SocketAddr`] if this method is [`ChannelMethod::Direct`] and there is one and
    /// only one available address.
    ///
    /// This is single address requirement is very important as this call is used during the
    /// channel handshake on which we need the actual peer address we are connected to and not all
    /// the possibilities.
    ///
    /// When connecting or accepting, that address is put in the OwnedChanTarget and so this
    /// extracts it.
    pub fn unique_direct_addr(&self) -> Option<SocketAddr> {
        match self {
            Self::Direct(addrs) => match addrs.as_slice() {
                [addr] => Some(*addr),
                // Remember, more than one, we don't have a single address.
                _ => None,
            },
        }
    }

    /// Return true if this is a method for a direct connection.
    pub fn is_direct(&self) -> bool {
        matches!(self, ChannelMethod::Direct(_))
    }

    ///
    /// Change this `ChannelMethod` by removing every socket address that
    /// does not satisfy `pred`.
    ///
    /// Return an error if we have removed every address.
    pub fn retain_addrs<P>(&mut self, pred: P) -> Result<(), RetainAddrsError>
    where
        P: Fn(&std::net::SocketAddr) -> bool,
    {
        match self {
            ChannelMethod::Direct(d) if d.is_empty() => {}
            ChannelMethod::Direct(d) => {
                d.retain(pred);
                if d.is_empty() {
                    return Err(RetainAddrsError::NoAddrsLeft);
                }
            }
        }
        Ok(())
    }

    /// Return true if every method to contact `self` is also a method to
    /// contact `other`.
    pub fn contained_by(&self, other: &ChannelMethod) -> bool {
        use ChannelMethod as CM;
        match (self, other) {
            (CM::Direct(our_addrs), CM::Direct(their_addrs)) => {
                our_addrs.iter().all(|a| their_addrs.contains(a))
            }
        }
    }
}

/// An error that occurred while filtering addresses from a ChanMethod.
#[derive(Clone, Debug, thiserror::Error)]
pub enum RetainAddrsError {
    /// We removed all of the addresses from this method.
    #[error("All addresses were removed.")]
    NoAddrsLeft,
}

impl HasAddrs for ChannelMethod {
    fn addrs(&self) -> impl Iterator<Item = SocketAddr> {
        match self {
            ChannelMethod::Direct(addrs) => addrs.iter().copied(),
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
    use super::*;

    #[test]
    fn chanmethod_direct() {
        let a1 = "127.0.0.1:8080".parse().unwrap();
        let a2 = "127.0.0.2:8181".parse().unwrap();
        let a3 = "127.0.0.3:8282".parse().unwrap();

        let m = ChannelMethod::Direct(vec![a1, a2]);
        assert_eq!(m.socket_addrs(), Some(&[a1, a2][..]));
        assert!(m.is_direct());

        let m2 = ChannelMethod::Direct(vec![a1, a2, a3]);
        assert!(m.contained_by(&m));
        assert!(m.contained_by(&m2));
        assert!(!m2.contained_by(&m));

        let mut m3 = m2.clone();
        m3.retain_addrs(|a| a.port() != 8282).unwrap();
        assert_eq!(m3, m);
        assert_ne!(m3, m2);
    }
}
