//! Types and functions to configure a Tor client.
//!
//! Some of these are re-exported from lower-level crates.

use crate::err::ErrorDetail;
use derive_deftly::Deftly;
use derive_more::AsRef;
use fs_mistrust::{Mistrust, MistrustBuilder};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::result::Result as StdResult;
use std::time::Duration;

pub use tor_chanmgr::{ChannelConfig, ChannelConfigBuilder};
pub use tor_config::BoolOrAuto;
pub use tor_config::convert_helper_via_multi_line_list_builder;
use tor_config::derive::prelude::*;
use tor_config::extend_builder::extend_with_replace;
pub use tor_config::impl_standard_builder;
pub use tor_config::list_builder::{MultilineListBuilder, MultilineListBuilderError};
pub use tor_config::mistrust::BuilderExt as _;
pub use tor_config::{ConfigBuildError, Reconfigure};
pub use tor_config::{define_list_builder_accessors, define_list_builder_helper};
pub use tor_config_path::{CfgPath, CfgPathError, CfgPathResolver};
pub use tor_linkspec::{ChannelMethod, HasChanMethod};

use tor_keymgr::config::{ArtiKeystoreConfig, ArtiKeystoreConfigBuilder};

/// Types for configuring how Tor circuits are built.
pub mod circ {
    pub use tor_circmgr::{
        CircMgrConfig, CircuitTiming, CircuitTimingBuilder, PathConfig, PathConfigBuilder,
        PreemptiveCircuitConfig, PreemptiveCircuitConfigBuilder,
    };
}

/// Types for configuring how Tor accesses its directory information.
pub mod dir {
    pub use tor_dircommon::authority::{AuthorityContacts, AuthorityContactsBuilder};
    pub use tor_dircommon::config::{
        DirTolerance, DirToleranceBuilder, DownloadScheduleConfig, DownloadScheduleConfigBuilder,
        NetworkConfig, NetworkConfigBuilder,
    };
    pub use tor_dircommon::retry::{DownloadSchedule, DownloadScheduleBuilder};
    pub use tor_dirmgr::{DirMgrConfig, FallbackDir, FallbackDirBuilder};
}

/// Types for configuring vanguards.
pub mod vanguards {
    pub use tor_guardmgr::{VanguardConfig, VanguardConfigBuilder};
}

#[cfg(not(all(feature = "vanguards", feature = "onion-service-client")))]
use {
    std::sync::LazyLock,
    tor_config::ExplicitOrAuto,
    tor_guardmgr::{VanguardConfig, VanguardConfigBuilder, VanguardMode},
};

/// A [`VanguardConfig`] which is disabled.
// It would be nice if the builder were const, but this is the best we can do.
// Boxed so that this is guaranteed to use very little space if it's unused.
#[cfg(not(all(feature = "vanguards", feature = "onion-service-client")))]
static DISABLED_VANGUARDS: LazyLock<Box<VanguardConfig>> = LazyLock::new(|| {
    Box::new(
        VanguardConfigBuilder::default()
            .mode(ExplicitOrAuto::Explicit(VanguardMode::Disabled))
            .build()
            .expect("Could not build a disabled `VanguardConfig`"),
    )
});

/// Configuration for client behavior relating to addresses.
///
/// This type is immutable once constructed. To create an object of this type,
/// use [`ClientAddrConfigBuilder`].
///
/// You can replace this configuration on a running Arti client.  Doing so will
/// affect new streams and requests, but will have no effect on existing streams
/// and requests.
#[derive(Debug, Clone, Deftly, Eq, PartialEq)]
#[derive_deftly(TorConfig)]
pub struct ClientAddrConfig {
    /// Should we allow attempts to make Tor connections to local addresses?
    ///
    /// This option is off by default, since (by default) Tor exits will
    /// always reject connections to such addresses.
    #[deftly(tor_config(default))]
    pub(crate) allow_local_addrs: bool,

    /// Should we accept and return local addresses in anonymously retrieved DNS answers?
    ///
    /// This option is off by default, since (by default) Tor exits will
    /// always reject connections to such addresses.
    #[deftly(tor_config(default))]
    pub(crate) allow_resolving_local_addrs: bool,

    /// Should we allow attempts to connect to hidden services (`.onion` services)?
    ///
    /// This option is on by default.
    //
    // NOTE: This could use tor_config(cfg) instead, but that would change the API.
    #[cfg(feature = "onion-service-client")]
    #[deftly(tor_config(default = "true"))]
    pub(crate) allow_onion_addrs: bool,
}

/// Configuration for client behavior relating to stream connection timeouts
///
/// This type is immutable once constructed. To create an object of this type,
/// use [`StreamTimeoutConfigBuilder`].
///
/// You can replace this configuration on a running Arti client.  Doing so will
/// affect new streams and requests, but will have no effect on existing streams
/// and requests—even those that are currently waiting.
#[derive(Debug, Clone, Deftly, Eq, PartialEq)]
#[derive_deftly(TorConfig)]
#[non_exhaustive]
pub struct StreamTimeoutConfig {
    /// How long should we wait before timing out a stream when connecting
    /// to a host?
    #[deftly(tor_config(default = "default_connect_timeout()"))]
    pub(crate) connect_timeout: Duration,

    /// How long should we wait before timing out when resolving a DNS record?
    #[deftly(tor_config(default = "default_dns_resolve_timeout()"))]
    pub(crate) resolve_timeout: Duration,

    /// How long should we wait before timing out when resolving a DNS
    /// PTR record?
    #[deftly(tor_config(default = "default_dns_resolve_ptr_timeout()"))]
    pub(crate) resolve_ptr_timeout: Duration,
}

/// Return the default stream timeout
fn default_connect_timeout() -> Duration {
    Duration::new(10, 0)
}

/// Return the default resolve timeout
fn default_dns_resolve_timeout() -> Duration {
    Duration::new(10, 0)
}

/// Return the default PTR resolve timeout
fn default_dns_resolve_ptr_timeout() -> Duration {
    Duration::new(10, 0)
}

/// Configuration for where information should be stored on disk.
///
/// By default, cache information will be stored in `${ARTI_CACHE}`, and
/// persistent state will be stored in `${ARTI_LOCAL_DATA}`.  That means that
/// _all_ programs using these defaults will share their cache and state data.
/// If that isn't what you want,  you'll need to override these directories.
///
/// On unix, the default directories will typically expand to `~/.cache/arti`
/// and `~/.local/share/arti/` respectively, depending on the user's
/// environment. Other platforms will also use suitable defaults. For more
/// information, see the documentation for [`CfgPath`].
///
/// This section is for read/write storage.
///
/// You cannot change this section on a running Arti client.
#[derive(Debug, Clone, Deftly, Eq, PartialEq)]
#[derive_deftly(TorConfig)]
pub struct StorageConfig {
    /// Location on disk for cached information.
    ///
    /// This follows the rules for `/var/cache`: "sufficiently old" filesystem objects
    /// in it may be deleted outside of the control of Arti,
    /// and Arti will continue to function properly.
    /// It is also fine to delete the directory as a whole, while Arti is not running.
    //
    // Usage note, for implementations of Arti components:
    //
    // When files in this directory are to be used by a component, the cache_dir
    // value should be passed through to the component as-is, and the component is
    // then responsible for constructing an appropriate sub-path (for example,
    // tor-dirmgr receives cache_dir, and appends components such as "dir_blobs".
    //
    // (This consistency rule is not current always followed by every component.)
    #[deftly(tor_config(default = "default_cache_dir()", setter(into)))]
    pub(crate) cache_dir: CfgPath,

    /// Location on disk for less-sensitive persistent state information.
    // Usage note: see the note for `cache_dir`, above.
    #[deftly(tor_config(default = "default_state_dir()", setter(into)))]
    state_dir: CfgPath,

    /// Location on disk for the Arti keystore.
    //
    // NOTE: This could use tor_config(cfg) instead, but that would change the API.
    #[cfg(feature = "keymgr")]
    #[deftly(tor_config(sub_builder))]
    keystore: ArtiKeystoreConfig,

    /// Configuration about which permissions we want to enforce on our files.
    #[deftly(tor_config(
        sub_builder(build_fn = "build_for_arti"),
        extend_with = "extend_with_replace"
    ))]
    permissions: Mistrust,
}

/// Return the default cache directory.
fn default_cache_dir() -> CfgPath {
    CfgPath::new("${ARTI_CACHE}".to_owned())
}

/// Return the default state directory.
fn default_state_dir() -> CfgPath {
    CfgPath::new("${ARTI_LOCAL_DATA}".to_owned())
}

/// Macro to avoid repeating code for `expand_*_dir` functions on StorageConfig
// TODO: generate the expand_*_dir functions using d-a instead
macro_rules! expand_dir {
    ($self:ident, $dirname:ident, $dircfg:ident) => {
        $self
            .$dirname
            .path($dircfg)
            .map_err(|e| ConfigBuildError::Invalid {
                field: stringify!($dirname).to_owned(),
                problem: e.to_string(),
            })
    };
}

impl StorageConfig {
    /// Try to expand `state_dir` to be a path buffer.
    pub(crate) fn expand_state_dir(
        &self,
        path_resolver: &CfgPathResolver,
    ) -> Result<PathBuf, ConfigBuildError> {
        expand_dir!(self, state_dir, path_resolver)
    }
    /// Try to expand `cache_dir` to be a path buffer.
    pub(crate) fn expand_cache_dir(
        &self,
        path_resolver: &CfgPathResolver,
    ) -> Result<PathBuf, ConfigBuildError> {
        expand_dir!(self, cache_dir, path_resolver)
    }
    /// Return the keystore config
    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn keystore(&self) -> ArtiKeystoreConfig {
        cfg_if::cfg_if! {
            if #[cfg(feature="keymgr")] {
                self.keystore.clone()
            } else {
                Default::default()
            }
        }
    }
    /// Return the FS permissions to use for state and cache directories.
    pub(crate) fn permissions(&self) -> &Mistrust {
        &self.permissions
    }
}

/// A configuration used to bootstrap a [`TorClient`](crate::TorClient).
///
/// In order to connect to the Tor network, Arti needs to know a few
/// well-known directory caches on the network, and the public keys of the
/// network's directory authorities.  It also needs a place on disk to
/// store persistent state and cached directory information. (See [`StorageConfig`]
/// for default directories.)
///
/// Most users will create a TorClientConfig by running
/// [`TorClientConfig::default`].
///
/// If you need to override the locations where Arti stores its
/// information, you can make a TorClientConfig with
/// [`TorClientConfigBuilder::from_directories`].
///
/// Finally, you can get fine-grained control over the members of a
/// TorClientConfig using [`TorClientConfigBuilder`].
///
/// Deserializing the builder rejects unknown top-level settings. This includes
/// removed bridge and pluggable-transport configuration; it cannot silently
/// fall back to direct Tor connections.
#[derive(Clone, Deftly, Debug, AsRef, educe::Educe)]
#[educe(PartialEq, Eq)]
#[derive_deftly(TorConfig)]
#[deftly(tor_config(attr = { serde(deny_unknown_fields) }))]
#[non_exhaustive]
pub struct TorClientConfig {
    /// Information about the Tor network we want to connect to.
    #[deftly(tor_config(sub_builder))]
    tor_network: dir::NetworkConfig,

    /// Directories for storing information on disk
    #[deftly(tor_config(sub_builder))]
    pub(crate) storage: StorageConfig,

    /// Information about when and how often to download directory information
    #[deftly(tor_config(sub_builder))]
    download_schedule: dir::DownloadScheduleConfig,

    /// Information about how premature or expired our directories are allowed
    /// to be.
    ///
    /// These options help us tolerate clock skew, and help survive the case
    /// where the directory authorities are unable to reach consensus for a
    /// while.
    #[deftly(tor_config(sub_builder))]
    directory_tolerance: dir::DirTolerance,

    /// Facility to override network parameters from the values set in the
    /// consensus.
    #[deftly(tor_config(
        setter(skip), // See note on accessor. This isn't the best way to do this.
        field(ty = "HashMap<String, i32>"),
        build = "|this: &Self| default_extend(this.override_net_params.clone())",
        extend_with = "extend_with_replace"
    ))]
    pub(crate) override_net_params: tor_netdoc::doc::netstatus::NetParams<i32>,

    /// Information about how to build paths through the network.
    #[deftly(tor_config(sub_builder))]
    pub(crate) channel: ChannelConfig,

    /// Configuration for system resources used by Arti
    ///
    /// Note that there are other settings in this section,
    /// in `arti::cfg::SystemConfig` -
    /// these two structs overlay here.
    #[deftly(tor_config(sub_builder))]
    pub(crate) system: SystemConfig,

    /// Information about how to build paths through the network.
    #[as_ref]
    #[deftly(tor_config(sub_builder))]
    path_rules: circ::PathConfig,

    /// Information about preemptive circuits.
    #[as_ref]
    #[deftly(tor_config(sub_builder))]
    preemptive_circuits: circ::PreemptiveCircuitConfig,

    /// Information about how to retry and expire circuits and request for circuits.
    #[as_ref]
    #[deftly(tor_config(sub_builder))]
    circuit_timing: circ::CircuitTiming,

    /// Rules about which addresses the client is willing to connect to.
    #[deftly(tor_config(sub_builder))]
    pub(crate) address_filter: ClientAddrConfig,

    /// Information about timing out client requests.
    #[deftly(tor_config(sub_builder))]
    pub(crate) stream_timeouts: StreamTimeoutConfig,

    /// Information about vanguards.
    // NOTE: Don't use `#[as_ref]` below, since we provide our own AsRef impl to handle when
    // vanguards are disabled.
    #[deftly(tor_config(sub_builder))]
    pub(crate) vanguards: vanguards::VanguardConfig,

    /// Resolves paths in this configuration.
    ///
    /// This is not [reconfigurable](crate::TorClient::reconfigure).
    // We don't accept this from the builder/serde, and don't inspect it when comparing configs.
    // This should be considered as ancillary data rather than a configuration option.
    // TorClientConfig maybe isn't the best place for this, but this is where it needs to go to not
    // require public API changes.
    #[as_ref]
    #[deftly(tor_config(skip, build = "|_| tor_config_path::arti_client_base_resolver()"))]
    #[educe(PartialEq(ignore), Eq(ignore))]
    pub(crate) path_resolver: CfgPathResolver,
}

/// Helper to add overrides to a default collection.
fn default_extend<T: Default + Extend<X>, X>(to_add: impl IntoIterator<Item = X>) -> T {
    let mut collection = T::default();
    collection.extend(to_add);
    collection
}

/// Configuration for system resources used by Tor.
///
/// You cannot change this section on a running Arti client.
///
/// Note that there are other settings in this section,
/// in `arti_client::config::SystemConfig`.
#[derive(Debug, Clone, Deftly, Eq, PartialEq)]
#[derive_deftly(TorConfig)]
#[non_exhaustive]
pub struct SystemConfig {
    /// Memory limits (approximate)
    #[deftly(tor_config(sub_builder))]
    pub(crate) memory: tor_memquota::Config,
}

impl AsRef<tor_guardmgr::VanguardConfig> for TorClientConfig {
    fn as_ref(&self) -> &tor_guardmgr::VanguardConfig {
        cfg_if::cfg_if! {
            if #[cfg(all(
                feature = "vanguards",
                feature = "onion-service-client",
            ))]
            {
                &self.vanguards
            } else {
                &DISABLED_VANGUARDS
            }
        }
    }
}

impl tor_guardmgr::GuardMgrConfig for TorClientConfig {}

impl tor_circmgr::CircMgrConfig for TorClientConfig {}

#[cfg(feature = "onion-service-client")]
impl tor_hsclient::HsClientConnectorConfig for TorClientConfig {}

#[cfg(feature = "onion-service-client")]
impl tor_circmgr::hspool::HsCircPoolConfig for TorClientConfig {
    #[cfg(all(feature = "vanguards", feature = "onion-service-client"))]
    fn vanguard_config(&self) -> &tor_guardmgr::VanguardConfig {
        &self.vanguards
    }
}

impl AsRef<tor_dircommon::fallback::FallbackList> for TorClientConfig {
    fn as_ref(&self) -> &tor_dircommon::fallback::FallbackList {
        self.tor_network.fallback_caches()
    }
}

impl TorClientConfig {
    /// Try to create a DirMgrConfig corresponding to this object.
    #[rustfmt::skip]
    pub fn dir_mgr_config(&self) -> Result<dir::DirMgrConfig, ConfigBuildError> {
        Ok(dir::DirMgrConfig {
            network:             self.tor_network        .clone(),
            schedule:            self.download_schedule  .clone(),
            tolerance:           self.directory_tolerance.clone(),
            cache_dir:           self.storage.expand_cache_dir(&self.path_resolver)?,
            cache_trust:         self.storage.permissions.clone(),
            override_net_params: self.override_net_params.clone(),
            extensions:          Default::default(),
        })
    }

    /// Return a reference to the [`fs_mistrust::Mistrust`] object that we'll
    /// use to check permissions on files and directories by default.
    ///
    /// # Usage notes
    ///
    /// In the future, specific files or directories may have stricter or looser
    /// permissions checks applied to them than this default.  Callers shouldn't
    /// use this [`Mistrust`] to predict what Arti will accept for a specific
    /// file or directory.  Rather, you should use this if you have some file or
    /// directory of your own on which you'd like to enforce the same rules as
    /// Arti uses.
    //
    // NOTE: The presence of this accessor is _NOT_ in any form a commitment to
    // expose every field from the configuration as an accessor.  We explicitly
    // reject that slippery slope argument.
    pub fn fs_mistrust(&self) -> &Mistrust {
        self.storage.permissions()
    }

    /// Return the keystore config
    pub fn keystore(&self) -> ArtiKeystoreConfig {
        self.storage.keystore()
    }

    /// Get the state directory and its corresponding
    /// [`Mistrust`] configuration.
    pub(crate) fn state_dir(&self) -> StdResult<(PathBuf, &fs_mistrust::Mistrust), ErrorDetail> {
        let state_dir = self
            .storage
            .expand_state_dir(&self.path_resolver)
            .map_err(ErrorDetail::Configuration)?;
        let mistrust = self.storage.permissions();

        Ok((state_dir, mistrust))
    }

    /// Access the `tor_memquota` configuration
    ///
    /// Ad-hoc accessor for testing purposes.
    /// (ideally we'd use `visibility` to make fields `pub`, but that doesn't work.)
    #[cfg(feature = "testing")]
    pub fn system_memory(&self) -> &tor_memquota::Config {
        &self.system.memory
    }
}

impl TorClientConfigBuilder {
    /// Returns a `TorClientConfigBuilder` using the specified state and cache directories.
    ///
    /// All other configuration options are set to their defaults, except `storage.keystore.path`,
    /// which is derived from the specified state directory.
    pub fn from_directories<P, Q>(state_dir: P, cache_dir: Q) -> Self
    where
        P: AsRef<Path>,
        Q: AsRef<Path>,
    {
        let mut builder = Self::default();

        builder
            .storage()
            .cache_dir(CfgPath::new_literal(cache_dir.as_ref()))
            .state_dir(CfgPath::new_literal(state_dir.as_ref()));

        builder
    }

    /// Return a mutable reference to a HashMap of `override_net_params`
    ///
    /// These parameters, if set, replace those that arrive in the network consensus document.
    //
    // NOTE: This is necessary for now because sub_builder isn't compatible with build().
    pub fn override_net_params(&mut self) -> &mut HashMap<String, i32> {
        &mut self.override_net_params
    }
}

/// The environment variable we look at when deciding whether to disable FS permissions checking.
#[deprecated = "use tor-config::mistrust::ARTI_FS_DISABLE_PERMISSION_CHECKS instead"]
pub const FS_PERMISSIONS_CHECKS_DISABLE_VAR: &str = "ARTI_FS_DISABLE_PERMISSION_CHECKS";

/// Return true if the environment has been set up to disable FS permissions
/// checking.
///
/// This function is exposed so that other tools can use the same checking rules
/// as `arti-client`.  For more information, see
/// [`TorClientBuilder`](crate::TorClientBuilder).
#[deprecated(since = "0.5.0")]
#[allow(deprecated)]
pub fn fs_permissions_checks_disabled_via_env() -> bool {
    std::env::var_os(FS_PERMISSIONS_CHECKS_DISABLE_VAR).is_some()
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
    use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

    use super::*;

    #[test]
    fn defaults() {
        let dflt = TorClientConfig::default();
        let b2 = TorClientConfigBuilder::default();
        let dflt2 = b2.build().unwrap();
        assert_eq!(&dflt, &dflt2);
    }

    #[test]
    fn builder() {
        let sec = std::time::Duration::from_secs(1);

        let mut authorities = dir::AuthorityContacts::builder();
        authorities.v3idents().push([22; 20].into());
        authorities.v3idents().push([44; 20].into());
        authorities.uploads().push(vec![
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 80)),
            SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::LOCALHOST, 80, 0, 0)),
        ]);

        let mut fallback = dir::FallbackDir::builder();
        fallback
            .rsa_identity([23; 20].into())
            .ed_identity([99; 32].into())
            .orports()
            .push("127.0.0.7:7".parse().unwrap());

        let mut bld = TorClientConfig::builder();
        *bld.tor_network().authorities() = authorities;
        bld.tor_network().set_fallback_caches(vec![fallback]);
        bld.storage()
            .cache_dir(CfgPath::new("/var/tmp/foo".to_owned()))
            .state_dir(CfgPath::new("/var/tmp/bar".to_owned()));
        bld.download_schedule().retry_certs().attempts(10);
        bld.download_schedule().retry_certs().initial_delay(sec);
        bld.download_schedule().retry_certs().parallelism(3);
        bld.download_schedule().retry_microdescs().attempts(30);
        bld.download_schedule()
            .retry_microdescs()
            .initial_delay(10 * sec);
        bld.download_schedule().retry_microdescs().parallelism(9);
        bld.override_net_params()
            .insert("wombats-per-quokka".to_owned(), 7);
        bld.path_rules()
            .ipv4_subnet_family_prefix(20)
            .ipv6_subnet_family_prefix(48);
        bld.circuit_timing()
            .max_dirtiness(90 * sec)
            .request_timeout(10 * sec)
            .request_max_retries(22)
            .request_loyalty(3600 * sec);
        bld.address_filter().allow_local_addrs(true);

        let val = bld.build().unwrap();

        assert_ne!(val, TorClientConfig::default());
    }

    #[test]
    fn removed_bridge_configuration_is_rejected() {
        // Reject old bridge settings before a client can bootstrap directly.
        for config in [
            serde_json::json!({"bridges": {"enabled": true}}),
            serde_json::json!({"bridges": {"transports": []}}),
            serde_json::json!({"bridgse": {"enabled": true}}),
        ] {
            let error = serde_json::from_value::<TorClientConfigBuilder>(config)
                .unwrap_err()
                .to_string();
            assert!(error.contains("unknown field"), "{error}");
        }
        let error = toml::from_str::<TorClientConfigBuilder>("[bridges]\nenabled = true\n")
            .unwrap_err()
            .to_string();
        assert!(error.contains("unknown field `bridges`"), "{error}");

        // Direct wallet configuration still uses the ordinary typed builder.
        TorClientConfigBuilder::from_directories("state", "cache")
            .build()
            .unwrap();
    }

    #[test]
    #[cfg(not(all(feature = "vanguards", feature = "onion-service-client")))]
    fn check_disabled_vanguards_static() {
        // Force us to evaluate the closure to ensure that it builds correctly.
        #[allow(clippy::borrowed_box)]
        let _: &Box<VanguardConfig> = LazyLock::force(&DISABLED_VANGUARDS);
    }
}
