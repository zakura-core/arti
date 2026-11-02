//! Module exposing tunnel-related types shared by clients and relays.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::circuit::UniqId;

/// The unique identifier of a tunnel.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "relay", visibility::make(pub))]
#[allow(unreachable_pub)] // TODO(#1447): use in ChanMgr's ChannelProvider impl
pub(crate) struct TunnelId(u64);

impl core::fmt::Display for TunnelId {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let _0 = &self.0;
        formatter.write_fmt(format_args!("Tunnel {0}", _0))
    }
}

impl TunnelId {
    /// Create a new TunnelId.
    ///
    /// # Panics
    ///
    /// Panics if we have exhausted the possible space of u64 IDs.
    pub(crate) fn next() -> TunnelId {
        /// The next unique tunnel ID.
        static NEXT_TUNNEL_ID: AtomicU64 = AtomicU64::new(1);
        let id = NEXT_TUNNEL_ID.fetch_add(1, Ordering::Relaxed);
        assert!(id != 0, "Exhausted Tunnel ID space?!");
        TunnelId(id)
    }
}

/// The identifier of a circuit [`UniqId`] within a tunnel.
///
/// This type is only needed for logging purposes: a circuit's [`UniqId`] is
/// process-unique, but in the logs it's often useful to display the
/// owning tunnel's ID alongside the circuit identifier.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]

pub(crate) struct TunnelScopedCircId {
    /// The identifier of the owning tunnel
    tunnel_id: TunnelId,
    /// The process-unique identifier of the circuit
    circ_uniq_id: UniqId,
}

impl core::fmt::Display for TunnelScopedCircId {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let tunnel_id = &self.tunnel_id;
        let circ_uniq_id = &self.circ_uniq_id;
        formatter.write_fmt(format_args!("{0} ({1})", circ_uniq_id, tunnel_id))
    }
}

impl TunnelScopedCircId {
    /// Create a new [`TunnelScopedCircId`] from the specified identifiers.
    pub(crate) fn new(tunnel_id: TunnelId, circ_uniq_id: UniqId) -> Self {
        Self {
            tunnel_id,
            circ_uniq_id,
        }
    }

    /// Return the [`UniqId`].
    pub(crate) fn unique_id(&self) -> UniqId {
        self.circ_uniq_id
    }
}
