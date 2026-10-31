//! Owned variants of [`ChanTarget`] and [`CircTarget`].

use safelog::Redactable;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};
use std::net::SocketAddr;
use tor_config::ConfigBuildError;
use tor_llcrypto::pk;

use crate::{
    ChanTarget, ChannelMethod, CircTarget, HasAddrs, HasChanMethod, HasRelayIds, RelayIdRef,
    RelayIdType,
};

/// RelayIds is an owned copy of the set of known identities of a relay.
///
/// Note that an object of this type will not necessarily have every type of
/// identity: it's possible that we don't know all the identities, or that one
/// of the identity types has become optional.
#[derive(Debug, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct RelayIds {
    /// Copy of the ed25519 id from the underlying ChanTarget.
    //
    // NOTE: we added serde(default) for ed25519 and rsa around April 2026.
    // Do not assume that they are present in older versions!
    #[serde(rename = "ed25519", default)]
    ed_identity: Option<pk::ed25519::Ed25519Identity>,

    /// Copy of the rsa id from the underlying ChanTarget.
    #[serde(rename = "rsa", default)]
    rsa_identity: Option<pk::rsa::RsaIdentity>,
}

/// Builder for [`RelayIds`].
#[derive(Debug, Clone, Default)]
pub struct RelayIdsBuilder {
    /// The configured Ed25519 identity, if any.
    ed_identity: Option<pk::ed25519::Ed25519Identity>,
    /// The configured RSA identity, if any.
    rsa_identity: Option<pk::rsa::RsaIdentity>,
}

impl HasRelayIds for RelayIds {
    fn identity(&self, key_type: RelayIdType) -> Option<crate::RelayIdRef<'_>> {
        match key_type {
            RelayIdType::Ed25519 => self.ed_identity.as_ref().map(RelayIdRef::from),
            RelayIdType::Rsa => self.rsa_identity.as_ref().map(RelayIdRef::from),
        }
    }
}

impl RelayIds {
    /// Return a fresh, default builder.
    pub fn builder() -> RelayIdsBuilder {
        RelayIdsBuilder::default()
    }

    /// Return an empty set of identities.
    ///
    /// This is _not_ a `Default` method, since this is not what you should
    /// usually construct.
    pub const fn empty() -> Self {
        Self {
            ed_identity: None,
            rsa_identity: None,
        }
    }

    /// Construct a new `RelayIds` object from another object that implements
    /// [`HasRelayIds`].
    ///
    /// Note that it is possible to construct an _empty_ `RelayIds` object if
    /// the input does not contain any recognized identity type.
    pub fn from_relay_ids<T: HasRelayIds + ?Sized>(other: &T) -> Self {
        Self {
            ed_identity: other
                .identity(RelayIdType::Ed25519)
                .map(|r| *r.unwrap_ed25519()),
            rsa_identity: other.identity(RelayIdType::Rsa).map(|r| *r.unwrap_rsa()),
        }
    }
}

impl std::fmt::Display for RelayIds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_relay_ids())
    }
}
impl Redactable for RelayIds {
    fn display_redacted(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_relay_ids().redacted())
    }
}

impl RelayIdsBuilder {
    /// Set the Ed25519 identity.
    pub fn ed_identity(&mut self, value: pk::ed25519::Ed25519Identity) -> &mut Self {
        self.ed_identity = Some(value);
        self
    }

    /// Set the RSA identity.
    pub fn rsa_identity(&mut self, value: pk::rsa::RsaIdentity) -> &mut Self {
        self.rsa_identity = Some(value);
        self
    }

    /// Build a set of relay identities.
    pub fn build(&self) -> RelayIds {
        RelayIds {
            ed_identity: self.ed_identity,
            rsa_identity: self.rsa_identity,
        }
    }

    /// Construct a new `RelayIdsBuilder` object from an object implementing
    /// [`HasRelayIds`].
    ///
    /// Note that it is possible to construct an _empty_ `RelayIds` object if
    /// the input does not contain any recognized identity type.
    pub fn from_relay_ids<T: HasRelayIds + ?Sized>(ids: &T) -> Self {
        let mut builder = Self::default();
        if let Some(ed_id) = ids.ed_identity() {
            builder.ed_identity(*ed_id);
        }
        if let Some(rsa_id) = ids.rsa_identity() {
            builder.rsa_identity(*rsa_id);
        }
        builder
    }
}

/// OwnedChanTarget is a summary of a [`ChanTarget`] that owns all of its
/// members.
#[derive(Debug, Clone)]
pub struct OwnedChanTarget {
    /// Copy of the addresses from the underlying ChanTarget.
    addrs: Vec<SocketAddr>,
    /// Copy of the channel methods from the underlying ChanTarget.
    //
    // TODO: in many cases this will be redundant with addrs; if we allocate a
    // lot of these objects, we might want to handle that.
    method: ChannelMethod,
    /// Identities that this relay provides.
    ids: RelayIds,
}

/// Builder for [`OwnedChanTarget`].
#[derive(Debug, Clone, Default)]
pub struct OwnedChanTargetBuilder {
    /// The configured addresses, if any.
    addrs: Option<Vec<SocketAddr>>,
    /// The configured channel method, if any.
    method: Option<ChannelMethod>,
    /// The relay identities being built.
    ids: RelayIdsBuilder,
}

impl OwnedChanTargetBuilder {
    /// Set the addresses for this target.
    pub fn addrs(&mut self, value: Vec<SocketAddr>) -> &mut Self {
        self.addrs = Some(value);
        self
    }

    /// Set the channel method for this target.
    pub fn method(&mut self, value: ChannelMethod) -> &mut Self {
        self.method = Some(value);
        self
    }

    /// Access the builder for this target's relay identities.
    pub fn ids(&mut self) -> &mut RelayIdsBuilder {
        &mut self.ids
    }

    /// Build an owned channel target.
    ///
    /// If no channel method was set, derive a direct method from the current
    /// addresses on each call.
    pub fn build(&self) -> OwnedChanTarget {
        OwnedChanTarget {
            addrs: self.addrs.clone().unwrap_or_default(),
            method: self.method.clone().unwrap_or_else(|| self.make_method()),
            ids: self.ids.build(),
        }
    }

    /// Set the ed25519 identity in this builder to `id`.
    pub fn ed_identity(&mut self, id: pk::ed25519::Ed25519Identity) -> &mut Self {
        self.ids().ed_identity(id);
        self
    }

    /// Set the RSA identity in this builder to `id`.
    pub fn rsa_identity(&mut self, id: pk::rsa::RsaIdentity) -> &mut Self {
        self.ids().rsa_identity(id);
        self
    }

    /// Helper: make a channel method if none was specified.
    fn make_method(&self) -> ChannelMethod {
        ChannelMethod::Direct(self.addrs.clone().unwrap_or_default())
    }
}

impl HasAddrs for OwnedChanTarget {
    fn addrs(&self) -> impl Iterator<Item = SocketAddr> {
        self.addrs.iter().copied()
    }
}

impl HasChanMethod for OwnedChanTarget {
    fn chan_method(&self) -> ChannelMethod {
        self.method.clone()
    }
}

impl HasRelayIds for OwnedChanTarget {
    fn identity(&self, key_type: RelayIdType) -> Option<RelayIdRef<'_>> {
        self.ids.identity(key_type)
    }
}

impl ChanTarget for OwnedChanTarget {}

impl OwnedChanTarget {
    /// Return a fresh, default builder.
    pub fn builder() -> OwnedChanTargetBuilder {
        OwnedChanTargetBuilder::default()
    }

    /// Construct a OwnedChanTarget from a given ChanTarget.
    pub fn from_chan_target<C>(target: &C) -> Self
    where
        C: ChanTarget + ?Sized,
    {
        OwnedChanTarget {
            addrs: target.addrs().collect(),
            method: target.chan_method(),
            ids: RelayIds::from_relay_ids(target),
        }
    }

    /// Return a mutable reference to this [`OwnedChanTarget`]'s [`ChannelMethod`]
    ///
    pub fn chan_method_mut(&mut self) -> &mut ChannelMethod {
        &mut self.method
    }
}

/// Primarily for error reporting and logging
impl Display for OwnedChanTarget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.display_chan_target())
    }
}

impl Redactable for OwnedChanTarget {
    fn display_redacted(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display_chan_target().display_redacted(f)
    }

    fn debug_redacted(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display_chan_target().debug_redacted(f)
    }
}

/// OwnedCircTarget is a summary of a [`CircTarget`] that owns all its
/// members.
#[derive(Debug, Clone)]
pub struct OwnedCircTarget {
    /// The fields from this object when considered as a ChanTarget.
    chan_target: OwnedChanTarget,
    /// The ntor key to use when extending to this CircTarget
    ntor_onion_key: pk::curve25519::PublicKey,
    /// The subprotocol versions that this CircTarget supports.
    protocols: tor_protover::Protocols,
}

/// Builder for [`OwnedCircTarget`].
#[derive(Debug, Clone, Default)]
pub struct OwnedCircTargetBuilder {
    /// The channel target being built.
    chan_target: OwnedChanTargetBuilder,
    /// The configured ntor onion key, if any.
    ntor_onion_key: Option<pk::curve25519::PublicKey>,
    /// The configured subprotocol versions, if any.
    protocols: Option<tor_protover::Protocols>,
}

impl OwnedCircTargetBuilder {
    /// Access the builder for this target's channel target.
    pub fn chan_target(&mut self) -> &mut OwnedChanTargetBuilder {
        &mut self.chan_target
    }

    /// Set the ntor onion key for this target.
    pub fn ntor_onion_key(&mut self, value: pk::curve25519::PublicKey) -> &mut Self {
        self.ntor_onion_key = Some(value);
        self
    }

    /// Set the subprotocol versions for this target.
    pub fn protocols(&mut self, value: tor_protover::Protocols) -> &mut Self {
        self.protocols = Some(value);
        self
    }

    /// Build an owned circuit target, requiring an ntor key and protocols.
    pub fn build(&self) -> Result<OwnedCircTarget, ConfigBuildError> {
        Ok(OwnedCircTarget {
            chan_target: self.chan_target.build(),
            ntor_onion_key: self
                .ntor_onion_key
                .ok_or_else(|| ConfigBuildError::MissingField {
                    field: "ntor_onion_key".into(),
                })?,
            protocols: self
                .protocols
                .clone()
                .ok_or_else(|| ConfigBuildError::MissingField {
                    field: "protocols".into(),
                })?,
        })
    }
}

impl OwnedCircTarget {
    /// Return a fresh, default builder.
    pub fn builder() -> OwnedCircTargetBuilder {
        OwnedCircTargetBuilder::default()
    }

    /// Construct an OwnedCircTarget from a given CircTarget.
    pub fn from_circ_target<C>(target: &C) -> Self
    where
        C: CircTarget + ?Sized,
    {
        OwnedCircTarget {
            chan_target: OwnedChanTarget::from_chan_target(target),
            ntor_onion_key: *target.ntor_onion_key(),
            protocols: target.protovers().clone(),
        }
    }

    /// Return a mutable view of this OwnedCircTarget as an [`OwnedChanTarget`].
    pub fn chan_target_mut(&mut self) -> &mut OwnedChanTarget {
        &mut self.chan_target
    }

    /// Return a  view of this OwnedCircTarget as an [`OwnedChanTarget`].
    pub fn chan_target(&self) -> &OwnedChanTarget {
        &self.chan_target
    }
}

impl HasAddrs for OwnedCircTarget {
    fn addrs(&self) -> impl Iterator<Item = SocketAddr> {
        self.chan_target.addrs()
    }
}

impl HasRelayIds for OwnedCircTarget {
    fn identity(&self, key_type: RelayIdType) -> Option<RelayIdRef<'_>> {
        self.chan_target.identity(key_type)
    }
}
impl HasChanMethod for OwnedCircTarget {
    fn chan_method(&self) -> ChannelMethod {
        self.chan_target.chan_method()
    }
}

impl ChanTarget for OwnedCircTarget {}

impl CircTarget for OwnedCircTarget {
    fn ntor_onion_key(&self) -> &pk::curve25519::PublicKey {
        &self.ntor_onion_key
    }
    fn protovers(&self) -> &tor_protover::Protocols {
        &self.protocols
    }
}

/// A value that can be converted into an OwnedChanTarget.
pub trait IntoOwnedChanTarget {
    /// Convert this value into an [`OwnedChanTarget`].
    fn to_owned(self) -> OwnedChanTarget;

    /// Convert this value into an [`LoggedChanTarget`].
    fn to_logged(self) -> LoggedChanTarget
    where
        Self: Sized,
    {
        self.to_owned().into()
    }
}

impl<'a, T: ChanTarget + ?Sized> IntoOwnedChanTarget for &'a T {
    fn to_owned(self) -> OwnedChanTarget {
        OwnedChanTarget::from_chan_target(self)
    }
}

impl IntoOwnedChanTarget for OwnedChanTarget {
    fn to_owned(self) -> OwnedChanTarget {
        self
    }
}

/// An `OwnedChanTarget` suitable for logging and including in errors
pub type LoggedChanTarget = safelog::BoxSensitive<OwnedChanTarget>;

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
    use itertools::Itertools;

    /// Subprotocols used by the circuit-target fixtures.
    const TEST_PROTOCOLS: &str = "FlowCtrl=7";

    #[test]
    fn relay_id_builder_reuse_and_clone() {
        let mut builder = RelayIds::builder();
        assert_eq!(builder.build(), RelayIds::empty());

        let ed = [42; 32].into();
        let rsa = [45; 20].into();
        builder.ed_identity(ed);
        let saved = builder.clone();
        let ed_only = builder.build();
        assert_eq!(ed_only.ed_identity(), Some(&ed));
        assert_eq!(ed_only.rsa_identity(), None);

        builder.rsa_identity(rsa);
        assert_eq!(builder.build().rsa_identity(), Some(&rsa));
        assert_eq!(saved.build(), ed_only);
        assert_eq!(RelayIdsBuilder::from_relay_ids(&ed_only).build(), ed_only);
    }

    #[test]
    fn chan_builder_uses_latest_addresses_without_mutating_clones() {
        let mut builder = OwnedChanTarget::builder();
        let empty = builder.build();
        assert_eq!(empty.addrs().count(), 0);
        assert_eq!(empty.chan_method(), ChannelMethod::Direct(Vec::new()));

        let first: SocketAddr = "127.0.0.1:11".parse().unwrap();
        let second: SocketAddr = "127.0.0.2:12".parse().unwrap();
        let explicit: SocketAddr = "127.0.0.3:13".parse().unwrap();
        builder.addrs(vec![first]);
        let saved = builder.clone();
        for _ in 0..2 {
            assert_eq!(
                builder.build().chan_method(),
                ChannelMethod::Direct(vec![first])
            );
        }

        builder.addrs(vec![second]);
        builder.ids().ed_identity([42; 32].into());
        let updated = builder.build();
        assert_eq!(updated.addrs().collect_vec(), vec![second]);
        assert_eq!(updated.chan_method(), ChannelMethod::Direct(vec![second]));
        assert!(updated.ed_identity().is_some());
        assert_eq!(
            saved.build().chan_method(),
            ChannelMethod::Direct(vec![first])
        );
        assert!(saved.build().ed_identity().is_none());

        // An explicit method is independent of later address changes.
        builder.method(ChannelMethod::Direct(vec![explicit]));
        builder.addrs(vec![first]);
        assert_eq!(builder.build().addrs().collect_vec(), vec![first]);
        assert_eq!(
            builder.build().chan_method(),
            ChannelMethod::Direct(vec![explicit])
        );
    }

    #[test]
    fn circ_builder_requires_fields_in_order_and_keeps_clones_independent() {
        let mut builder = OwnedCircTarget::builder();
        assert!(
            matches!(builder.build(), Err(ConfigBuildError::MissingField { field })
            if field == "ntor_onion_key")
        );

        let key: pk::curve25519::PublicKey = [99; 32].into();
        builder.ntor_onion_key(key);
        assert!(
            matches!(builder.build(), Err(ConfigBuildError::MissingField { field })
            if field == "protocols")
        );

        builder.protocols(TEST_PROTOCOLS.parse().unwrap());
        let saved = builder.clone();
        builder.chan_target().ids().ed_identity([42; 32].into());
        for _ in 0..2 {
            let target = builder.build().unwrap();
            assert_eq!(target.ntor_onion_key(), &key);
            assert_eq!(target.protovers().to_string(), TEST_PROTOCOLS);
            assert!(target.ed_identity().is_some());
            assert!(saved.build().unwrap().ed_identity().is_none());
        }
    }

    #[test]
    #[allow(clippy::redundant_clone)]
    fn chan_target() {
        let ti = OwnedChanTarget::builder()
            .addrs(vec!["127.0.0.1:11".parse().unwrap()])
            .ed_identity([42; 32].into())
            .rsa_identity([45; 20].into())
            .build();

        let ti2 = OwnedChanTarget::from_chan_target(&ti);
        assert_eq!(ti.addrs().collect_vec(), ti2.addrs().collect_vec());
        assert!(ti.same_relay_ids(&ti2));

        assert_eq!(format!("{:?}", ti), format!("{:?}", ti2));
        assert_eq!(format!("{:?}", ti), format!("{:?}", ti.clone()));
    }

    #[test]
    #[allow(clippy::redundant_clone)]
    fn circ_target() {
        let mut builder = OwnedCircTarget::builder();
        builder
            .chan_target()
            .addrs(vec!["127.0.0.1:11".parse().unwrap()])
            .ed_identity([42; 32].into())
            .rsa_identity([45; 20].into());
        let ct = builder
            .ntor_onion_key([99; 32].into())
            .protocols(TEST_PROTOCOLS.parse().unwrap())
            .build()
            .unwrap();
        let ch = ct.chan_target.clone();

        assert_eq!(ct.addrs().collect_vec(), ch.addrs().collect_vec());
        assert!(ct.same_relay_ids(&ch));
        assert_eq!(ct.ntor_onion_key().as_bytes(), &[99; 32]);
        assert_eq!(ct.protovers().to_string(), TEST_PROTOCOLS);
        let ct2 = OwnedCircTarget::from_circ_target(&ct);
        assert_eq!(format!("{:?}", ct), format!("{:?}", ct2));
        assert_eq!(format!("{:?}", ct), format!("{:?}", ct.clone()));
    }

    #[test]
    fn format_relay_ids() {
        let mut builder = RelayIds::builder();
        builder
            .ed_identity([42; 32].into())
            .rsa_identity([45; 20].into());
        let ids = builder.build();
        assert_eq!(
            format!("{}", ids),
            "ed25519:KioqKioqKioqKioqKioqKioqKioqKioqKioqKioqKio $2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d"
        );
        assert_eq!(format!("{}", ids.redacted()), "ed25519:Ki…");
    }
}
