//! Traits and code to define different mechanisms for building Channels to
//! different kinds of targets.

use std::sync::{Arc, Mutex};

use crate::event::ChanMgrEventSender;
use async_trait::async_trait;
use tor_linkspec::OwnedChanTarget;
use tor_proto::channel::Channel;
use tor_proto::memquota::ChannelAccount;
use tracing::{debug, instrument};

#[cfg(feature = "relay")]
use safelog::Sensitive;

/// An opaque type that lets a `ChannelFactory` update the `ChanMgr` about bootstrap progress.
///
/// A future release of this crate might make this type less opaque.
// FIXME(eta): Do that.
#[derive(Clone)]
pub struct BootstrapReporter(pub(crate) Arc<Mutex<ChanMgrEventSender>>);

impl BootstrapReporter {
    #[cfg(test)]
    /// Create a useless version of this type to satisfy some test.
    pub(crate) fn fake() -> Self {
        let (snd, _rcv) = crate::event::channel();
        Self(Arc::new(Mutex::new(snd)))
    }
}

/// An object that knows how to build `Channels` to `ChanTarget`s.
///
/// This trait must be object-safe.
///
/// Every [`ChanMgr`](crate::ChanMgr) has a `ChannelFactory` that it uses to
/// construct all of its channels.
///
/// A `ChannelFactory` can be implemented in terms of a
/// [`TransportImplHelper`](crate::transport::TransportImplHelper), by wrapping it in a
/// `ChanBuilder`.
///
// FIXME(eta): Rectify the below situation.
/// (In fact, as of the time of writing, this is the *only* way to implement this trait
/// outside of this crate while keeping bootstrap status reporting, since `BootstrapReporter`
/// is an opaque type.)
#[async_trait]
pub trait ChannelFactory: Send + Sync {
    /// Open an authenticated channel to `target`.
    ///
    /// This method does does not necessarily handle retries or timeouts,
    /// although some of its implementations may.
    ///
    async fn connect_via_transport(
        &self,
        target: &OwnedChanTarget,
        reporter: BootstrapReporter,
        memquota: ChannelAccount,
    ) -> crate::Result<Arc<Channel>>;
}

/// Similar to [`ChannelFactory`], but for building channels from incoming streams.
#[async_trait]
pub trait IncomingChannelFactory: Send + Sync {
    /// The type of byte stream that's required to build channels for incoming connections.
    type Stream: Send + Sync + 'static;

    /// Open a channel from `peer` with the given `stream`. The channel may or may not be
    /// authenticated.
    #[cfg(feature = "relay")]
    async fn accept_from_transport(
        &self,
        peer: Sensitive<std::net::SocketAddr>,
        stream: Self::Stream,
        memquota: ChannelAccount,
    ) -> crate::Result<Arc<Channel>>;
}

#[async_trait]
impl<CF> crate::mgr::AbstractChannelFactory for CF
where
    CF: ChannelFactory + IncomingChannelFactory + Sync,
{
    type Channel = tor_proto::channel::Channel;
    type BuildSpec = OwnedChanTarget;
    type Stream = CF::Stream;

    #[instrument(skip_all, level = "trace")]
    async fn build_channel(
        &self,
        target: &Self::BuildSpec,
        reporter: BootstrapReporter,
        memquota: ChannelAccount,
    ) -> crate::Result<Arc<Self::Channel>> {
        debug!("Attempting to open a new channel to {target}");
        self.connect_via_transport(target, reporter, memquota).await
    }

    #[cfg(feature = "relay")]
    #[instrument(skip_all, level = "trace")]
    async fn build_channel_using_incoming(
        &self,
        peer: Sensitive<std::net::SocketAddr>,
        stream: Self::Stream,
        memquota: ChannelAccount,
    ) -> crate::Result<Arc<tor_proto::channel::Channel>> {
        debug!("Attempting to open a new channel from {peer}");
        self.accept_from_transport(peer, stream, memquota).await
    }
}

/// A shared channel factory that can be cloned without cloning its builder.
pub(crate) struct CompoundFactory<CF> {
    /// The shared channel factory
    default_factory: Arc<CF>,
}

impl<CF> Clone for CompoundFactory<CF> {
    fn clone(&self) -> Self {
        Self {
            default_factory: Arc::clone(&self.default_factory),
        }
    }
}

#[async_trait]
impl<CF: ChannelFactory> ChannelFactory for CompoundFactory<CF> {
    #[instrument(skip_all, level = "trace")]
    async fn connect_via_transport(
        &self,
        target: &OwnedChanTarget,
        reporter: BootstrapReporter,
        memquota: ChannelAccount,
    ) -> crate::Result<Arc<Channel>> {
        self.default_factory
            .connect_via_transport(target, reporter, memquota)
            .await
    }
}

#[async_trait]
impl<CF: IncomingChannelFactory> IncomingChannelFactory for CompoundFactory<CF> {
    type Stream = CF::Stream;

    #[cfg(feature = "relay")]
    async fn accept_from_transport(
        &self,
        peer: Sensitive<std::net::SocketAddr>,
        stream: Self::Stream,
        memquota: ChannelAccount,
    ) -> crate::Result<Arc<Channel>> {
        self.default_factory
            .accept_from_transport(peer, stream, memquota)
            .await
    }
}

impl<CF: ChannelFactory + 'static> CompoundFactory<CF> {
    /// Share `default_factory` between channel requests.
    pub(crate) fn new(default_factory: Arc<CF>) -> Self {
        Self { default_factory }
    }

    /// Return a reference to the default channel factory.
    #[cfg(feature = "relay")]
    pub(crate) fn default_factory(&self) -> &CF {
        &self.default_factory
    }

    /// Replace the default channel factory.
    #[cfg(feature = "relay")]
    pub(crate) fn replace_default_factory(&mut self, factory: Arc<CF>) {
        self.default_factory = factory;
    }
}
