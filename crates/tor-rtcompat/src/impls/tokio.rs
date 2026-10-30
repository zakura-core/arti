//! Re-exports of the tokio runtime for use with arti.
//!
//! This crate helps define a slim API around our async runtime so that we
//! can easily swap it out.

/// Types used for networking (tokio implementation)
pub(crate) mod net {
    use crate::{impls, traits};
    use async_trait::async_trait;
    #[cfg(unix)]
    use tor_general_addr::unix;

    pub(crate) use tokio_crate::net::{
        TcpListener as TokioTcpListener, TcpStream as TokioTcpStream, UdpSocket as TokioUdpSocket,
    };
    #[cfg(unix)]
    pub(crate) use tokio_crate::net::{
        UnixListener as TokioUnixListener, UnixStream as TokioUnixStream,
    };

    use futures::io::{AsyncRead, AsyncWrite};
    use paste::paste;
    use tokio_util::compat::{Compat, TokioAsyncReadCompatExt as _};

    use std::io::Result as IoResult;
    use std::net::SocketAddr;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    /// Provide a set of network stream wrappers for a single stream type.
    macro_rules! stream_impl {
        {
            $kind:ident,
            $addr:ty,
            $cvt_addr:ident
        } => {paste!{
            /// Wrapper for Tokio's
            #[doc = stringify!($kind)]
            /// streams,
            /// that implements the standard
            /// AsyncRead and AsyncWrite.
            pub struct [<$kind Stream>] {
                /// Underlying tokio_util::compat::Compat wrapper.
                s: Compat<[<Tokio $kind Stream>]>,
            }
            impl From<[<Tokio $kind Stream>]> for [<$kind Stream>] {
                fn from(s: [<Tokio $kind Stream>]) ->  [<$kind Stream>] {
                    let s = s.compat();
                    [<$kind Stream>] { s }
                }
            }
            impl AsyncRead for  [<$kind Stream>] {
                fn poll_read(
                    mut self: Pin<&mut Self>,
                    cx: &mut Context<'_>,
                    buf: &mut [u8],
                ) -> Poll<IoResult<usize>> {
                    Pin::new(&mut self.s).poll_read(cx, buf)
                }
            }
            impl AsyncWrite for  [<$kind Stream>] {
                fn poll_write(
                    mut self: Pin<&mut Self>,
                    cx: &mut Context<'_>,
                    buf: &[u8],
                ) -> Poll<IoResult<usize>> {
                    Pin::new(&mut self.s).poll_write(cx, buf)
                }
                fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
                    Pin::new(&mut self.s).poll_flush(cx)
                }
                fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<IoResult<()>> {
                    Pin::new(&mut self.s).poll_close(cx)
                }
            }

            /// Wrap a Tokio
            #[doc = stringify!($kind)]
            /// Listener to behave as a futures::io::TcpListener.
            pub struct [<$kind Listener>] {
                /// The underlying listener.
                pub(super) lis: [<Tokio $kind Listener>],
            }

            /// Asynchronous stream that yields incoming connections from a
            #[doc = stringify!($kind)]
            /// Listener.
            ///
            /// This is analogous to async_std::net::Incoming.
            pub struct [<Incoming $kind Streams>] {
                /// Reference to the underlying listener.
                pub(super) lis: [<Tokio $kind Listener>],
            }

            impl futures::stream::Stream for [<Incoming $kind Streams>] {
                type Item = IoResult<([<$kind Stream>], $addr)>;

                fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                    match self.lis.poll_accept(cx) {
                        Poll::Ready(Ok((s, a))) => Poll::Ready(Some(Ok((s.into(), $cvt_addr(a)? )))),
                        Poll::Ready(Err(e)) => Poll::Ready(Some(Err(e))),
                        Poll::Pending => Poll::Pending,
                    }
                }
            }
            impl traits::NetStreamListener<$addr> for [<$kind Listener>] {
                type Stream = [<$kind Stream>];
                type Incoming = [<Incoming $kind Streams>];
                fn incoming(self) -> Self::Incoming {
                    [<Incoming $kind Streams>] { lis: self.lis }
                }
                fn local_addr(&self) -> IoResult<$addr> {
                    $cvt_addr(self.lis.local_addr()?)
                }
            }
        }}
    }

    /// Try to convert a tokio `unix::SocketAddr` into a crate::SocketAddr.
    ///
    /// Frustratingly, this information is _right there_: Tokio's SocketAddr has a
    /// std::unix::net::SocketAddr internally, but there appears to be no way to get it out.
    #[cfg(unix)]
    #[allow(clippy::needless_pass_by_value)]
    fn try_cvt_tokio_unix_addr(
        addr: tokio_crate::net::unix::SocketAddr,
    ) -> IoResult<unix::SocketAddr> {
        if addr.is_unnamed() {
            crate::unix::new_unnamed_socketaddr()
        } else if let Some(p) = addr.as_pathname() {
            unix::SocketAddr::from_pathname(p)
        } else {
            Err(crate::unix::UnsupportedAfUnixAddressType.into())
        }
    }

    /// Wrapper for (not) converting std::net::SocketAddr to itself.
    #[allow(clippy::unnecessary_wraps)]
    fn identity_fn_socketaddr(addr: std::net::SocketAddr) -> IoResult<std::net::SocketAddr> {
        Ok(addr)
    }

    stream_impl! { Tcp, std::net::SocketAddr, identity_fn_socketaddr }
    #[cfg(unix)]
    stream_impl! { Unix, unix::SocketAddr, try_cvt_tokio_unix_addr }

    /// Wrap a Tokio UdpSocket
    pub struct UdpSocket {
        /// The underelying UdpSocket
        socket: TokioUdpSocket,
    }

    impl UdpSocket {
        /// Bind a UdpSocket
        pub async fn bind(addr: SocketAddr) -> IoResult<Self> {
            TokioUdpSocket::bind(addr)
                .await
                .map(|socket| UdpSocket { socket })
        }
    }

    #[async_trait]
    impl traits::UdpSocket for UdpSocket {
        async fn recv(&self, buf: &mut [u8]) -> IoResult<(usize, SocketAddr)> {
            self.socket.recv_from(buf).await
        }

        async fn send(&self, buf: &[u8], target: &SocketAddr) -> IoResult<usize> {
            self.socket.send_to(buf, target).await
        }

        fn local_addr(&self) -> IoResult<SocketAddr> {
            self.socket.local_addr()
        }
    }

    impl traits::StreamOps for TcpStream {
        fn set_tcp_notsent_lowat(&self, notsent_lowat: u32) -> IoResult<()> {
            impls::streamops::set_tcp_notsent_lowat(&self.s, notsent_lowat)
        }

        #[cfg(target_os = "linux")]
        fn new_handle(&self) -> Box<dyn traits::StreamOps + Send + Unpin> {
            Box::new(impls::streamops::TcpSockFd::from_fd(&self.s))
        }
    }

    #[cfg(unix)]
    impl traits::StreamOps for UnixStream {
        fn set_tcp_notsent_lowat(&self, _notsent_lowat: u32) -> IoResult<()> {
            Err(traits::UnsupportedStreamOp::new(
                "set_tcp_notsent_lowat",
                "unsupported on Unix streams",
            )
            .into())
        }
    }
}

// ==============================

use crate::network::{TcpConnectOptions, TcpListenOptions};
#[cfg(unix)]
use crate::network::{UnixConnectOptions, UnixListenOptions};
use crate::traits::*;
use async_trait::async_trait;
use futures::Future;
use std::any::Any;
use std::io::Result as IoResult;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
#[cfg(unix)]
use tor_general_addr::unix;
use tracing::instrument;

impl SleepProvider for TokioRuntimeHandle {
    type SleepFuture = tokio_crate::time::Sleep;
    fn sleep(&self, duration: Duration) -> Self::SleepFuture {
        tokio_crate::time::sleep(duration)
    }
}

#[async_trait]
impl crate::traits::NetStreamProvider for TokioRuntimeHandle {
    type Stream = net::TcpStream;
    type Listener = net::TcpListener;
    type ConnectOptions = TcpConnectOptions;
    type ListenOptions = TcpListenOptions;

    #[instrument(skip_all, level = "trace")]
    async fn connect(
        &self,
        addr: &std::net::SocketAddr,
        options: &Self::ConnectOptions,
    ) -> IoResult<Self::Stream> {
        // The socket before connect() has been called.
        let socket = super::tcp_pre_connect(addr, options)?;

        // It might seem a little weird to convert the `socket2::Socket` to a std `TcpStream` before
        // it's connected, but this is the approach recommended by tokio.
        //
        // https://docs.rs/tokio/latest/tokio/net/struct.TcpSocket.html#method.from_std_stream
        //
        // > Converts a `std::net::TcpStream` into a `TcpSocket`. The provided socket must not have
        // > been connected prior to calling this function. This function is typically used together
        // > with crates such as socket2 to configure socket options that are not available on
        // > `TcpSocket`.
        //
        // The socket will already be non-blocking.
        let socket = std::net::TcpStream::from(socket);
        let socket = tokio_crate::net::TcpSocket::from_std_stream(socket);

        // Let tokio handle the connection.
        let socket = socket.connect(*addr).await?;

        Ok(socket.into())
    }
    async fn listen(
        &self,
        addr: &std::net::SocketAddr,
        options: &Self::ListenOptions,
    ) -> IoResult<Self::Listener> {
        // Use an implementation that's the same across all runtimes.
        let lis = net::TokioTcpListener::from_std(super::tcp_listen(addr, options)?)?;

        Ok(net::TcpListener { lis })
    }
}

#[cfg(unix)]
#[async_trait]
impl crate::traits::NetStreamProvider<unix::SocketAddr> for TokioRuntimeHandle {
    type Stream = net::UnixStream;
    type Listener = net::UnixListener;
    type ConnectOptions = UnixConnectOptions;
    type ListenOptions = UnixListenOptions;

    #[instrument(skip_all, level = "trace")]
    async fn connect(
        &self,
        addr: &unix::SocketAddr,
        options: &Self::ConnectOptions,
    ) -> IoResult<Self::Stream> {
        // Will fail to compile if we add options without handling them here.
        let UnixConnectOptions {} = options;

        let path = addr
            .as_pathname()
            .ok_or(crate::unix::UnsupportedAfUnixAddressType)?;
        let s = net::TokioUnixStream::connect(path).await?;
        Ok(s.into())
    }
    async fn listen(
        &self,
        addr: &unix::SocketAddr,
        options: &Self::ListenOptions,
    ) -> IoResult<Self::Listener> {
        // Will fail to compile if we add options without handling them here.
        let UnixListenOptions {} = options;

        let path = addr
            .as_pathname()
            .ok_or(crate::unix::UnsupportedAfUnixAddressType)?;
        let lis = net::TokioUnixListener::bind(path)?;
        Ok(net::UnixListener { lis })
    }
}

#[cfg(not(unix))]
crate::impls::impl_unix_non_provider! { TokioRuntimeHandle }

#[async_trait]
impl crate::traits::UdpProvider for TokioRuntimeHandle {
    type UdpSocket = net::UdpSocket;

    async fn bind(&self, addr: &std::net::SocketAddr) -> IoResult<Self::UdpSocket> {
        net::UdpSocket::bind(*addr).await
    }
}

/// Create and return a new Tokio multithreaded runtime.
pub(crate) fn create_runtime() -> IoResult<TokioRuntimeHandle> {
    let runtime = tokio_crate::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .map_err(std::io::Error::other)?;
    let handle = runtime.block_on(async { tokio_crate::runtime::Handle::current() });
    Ok(TokioRuntimeHandle {
        owned: Some(Arc::new(runtime)),
        handle,
    })
}

/// Wrapper around a Handle to a tokio runtime.
///
/// Ideally, this type would go away, and we would just use
/// `tokio::runtime::Handle` directly.  Unfortunately, we can't implement
/// `futures::Spawn` on it ourselves because of Rust's orphan rules, so we need
/// to define a new type here.
///
/// # Limitations
///
/// Note that Arti requires that the runtime should have working implementations
/// for Tokio's time, net, and io facilities, but we have no good way to check
/// that when creating this object.
#[derive(Clone, Debug)]
pub struct TokioRuntimeHandle {
    /// If present, the tokio executor that we've created (and which we own).
    ///
    /// We never access this directly; only through `handle`.  We keep it here
    /// so that our Runtime types can be agnostic about whether they own the
    /// executor.
    owned: Option<Arc<tokio_crate::runtime::Runtime>>,
    /// The underlying Handle.
    handle: tokio_crate::runtime::Handle,
}

impl TokioRuntimeHandle {
    /// Wrap a tokio runtime handle into a format that Arti can use.
    ///
    /// # Limitations
    ///
    /// Note that Arti requires that the runtime should have working
    /// implementations for Tokio's time, net, and io facilities, but we have
    /// no good way to check that when creating this object.
    pub(crate) fn new(handle: tokio_crate::runtime::Handle) -> Self {
        handle.into()
    }

    /// Return true if this handle owns the executor that it points to.
    pub fn is_owned(&self) -> bool {
        self.owned.is_some()
    }
}

impl From<tokio_crate::runtime::Handle> for TokioRuntimeHandle {
    fn from(handle: tokio_crate::runtime::Handle) -> Self {
        Self {
            owned: None,
            handle,
        }
    }
}

impl ToplevelBlockOn for TokioRuntimeHandle {
    #[track_caller]
    fn block_on<F: Future>(&self, f: F) -> F::Output {
        self.handle.block_on(f)
    }
}

/// The opaque blocking-task future used by the Tokio runtimes.
///
/// Dropping this future detaches its task. A task panic or cancellation causes
/// polling the future to panic, as required by [`Blocking`].
pub struct TokioBlockingHandle<T>(tokio_crate::task::JoinHandle<T>);

impl<T: std::fmt::Debug> std::fmt::Debug for TokioBlockingHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BlockingHandle(InnerBh)")
    }
}

impl<T: 'static> Future for TokioBlockingHandle<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<T> {
        match futures::ready!(Pin::new(&mut self.get_mut().0).poll(cx)) {
            Ok(value) => Poll::Ready(value),
            Err(error) => panic!(
                "Task has been canceled or has panicked. \
                 Are you dropping the executor to early? Error: {error}"
            ),
        }
    }
}

impl Blocking for TokioRuntimeHandle {
    type ThreadHandle<T: Send + 'static> = TokioBlockingHandle<T>;

    #[track_caller]
    fn spawn_blocking<F, T>(&self, f: F) -> Self::ThreadHandle<T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        TokioBlockingHandle(self.handle.spawn_blocking(f))
    }

    #[track_caller]
    fn reenter_block_on<F: Future>(&self, future: F) -> F::Output {
        self.handle.block_on(future)
    }

    #[track_caller]
    fn blocking_io<F, T>(&self, f: F) -> impl Future<Output = T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        let r = tokio_crate::task::block_in_place(f);
        std::future::ready(r)
    }
}

impl futures::task::Spawn for TokioRuntimeHandle {
    #[track_caller]
    fn spawn_obj(
        &self,
        future: futures::task::FutureObj<'static, ()>,
    ) -> Result<(), futures::task::SpawnError> {
        let join_handle = self.handle.spawn(future);
        drop(join_handle); // this makes the task detached.
        Ok(())
    }
}

#[async_trait]
impl<T: Send + 'static> JoinSet<T> for tokio_crate::task::JoinSet<T> {
    fn spawn<F>(&mut self, task: F) -> Result<(), futures::task::SpawnError>
    where
        F: Future<Output = T> + Send + 'static,
    {
        let _ = self.spawn(task);
        Ok(())
    }

    async fn join_next(&mut self) -> Option<Result<T, Box<dyn Any + Send>>> {
        Some(self.join_next().await?.map_err(|e| {
            // Note: this will panic if the error was not caused
            // by the task panicking (i.e. if the task was cancelled).
            // In practice, however, our API doesn't expose any way
            // of cancelling the tasks added to a JoinSet,
            // so this can never panic.
            //
            // If we ever *do* implement cancellation facilities,
            // we need to avoid calling into_panic() without checking
            // if e.is_panic() first.
            e.into_panic()
        }))
    }
}

impl JoinSetProvider for TokioRuntimeHandle {
    type JoinSet<T: Send + Sync + 'static> = tokio_crate::task::JoinSet<T>;

    fn new_join_set<T>(&self) -> Self::JoinSet<T>
    where
        T: Send + Sync + 'static,
    {
        tokio_crate::task::JoinSet::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use futures::FutureExt as _;
    use std::panic::AssertUnwindSafe;
    use std::sync::mpsc;

    /// Bound synchronization failures without leaving a blocking worker stuck.
    const TEST_TIMEOUT: Duration = Duration::from_secs(5);

    /// Notify a test when runtime shutdown drops an unfinished async task.
    struct TaskDropped(mpsc::Sender<()>);

    impl Drop for TaskDropped {
        fn drop(&mut self) {
            let _ = self.0.send(());
        }
    }

    /// Launch a task that remains pending until its runtime is shut down.
    fn pending_task(runtime: &TokioRuntimeHandle) -> mpsc::Receiver<()> {
        let (dropped_tx, dropped_rx) = mpsc::channel();
        let (started_tx, started_rx) = futures::channel::oneshot::channel();
        futures::task::SpawnExt::spawn(runtime, async move {
            let guard = TaskDropped(dropped_tx);
            let _ = started_tx.send(());
            std::future::pending::<()>().await;
            drop(guard);
        })
        .expect("spawn pending task");
        runtime
            .block_on(async {
                tokio_crate::time::timeout(TEST_TIMEOUT, started_rx)
                    .await
                    .expect("task starts before deadline")
            })
            .expect("task started");
        dropped_rx
    }

    #[test]
    fn owned_runtime_lives_until_last_clone() {
        let runtime = create_runtime().expect("create runtime");
        assert!(runtime.is_owned());
        let clone = runtime.clone();
        let dropped = pending_task(&runtime);
        drop(runtime);
        assert!(matches!(dropped.try_recv(), Err(mpsc::TryRecvError::Empty)));
        clone.block_on(async {
            tokio_crate::time::sleep(Duration::from_millis(1)).await;
        });
        drop(clone);
        dropped
            .recv_timeout(TEST_TIMEOUT)
            .expect("last owner shuts down pending tasks");
    }

    #[test]
    fn borrowed_runtime_does_not_own_executor() {
        let owner = tokio_crate::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("create caller-owned runtime");
        let runtime = TokioRuntimeHandle::new(owner.handle().clone());
        assert!(!runtime.is_owned());
        let dropped = pending_task(&runtime);
        drop(runtime);
        assert!(matches!(dropped.try_recv(), Err(mpsc::TryRecvError::Empty)));
        owner.block_on(async {
            tokio_crate::time::sleep(Duration::from_millis(1)).await;
        });
        drop(owner);
        dropped
            .recv_timeout(TEST_TIMEOUT)
            .expect("caller owner shuts down pending tasks");
    }

    #[test]
    fn blocking_result_and_reentry() {
        /// A result that must not acquire an unnecessary `Debug` bound.
        struct NoDebug(u8);

        /// Preserve handle auto-traits even when the result is not `Sync`.
        fn assert_handle_traits<F: Future + Send + Sync + Unpin>() {}
        assert_handle_traits::<<TokioRuntimeHandle as Blocking>::ThreadHandle<std::cell::Cell<u8>>>(
        );

        let runtime = create_runtime().expect("create runtime");
        let reentry = runtime.clone();
        runtime.block_on(async {
            let result = runtime
                .spawn_blocking(move || {
                    reentry.reenter_block_on(async {
                        tokio_crate::time::sleep(Duration::from_millis(1)).await;
                        NoDebug(17)
                    })
                })
                .await;
            assert_eq!(result.0, 17);
            let handle = runtime.spawn_blocking(|| 23_u8);
            assert_eq!(format!("{handle:?}"), "BlockingHandle(InnerBh)");
            assert_eq!(handle.await, 23);
        });
    }

    #[test]
    fn dropped_blocking_handle_detaches() {
        let runtime = create_runtime().expect("create runtime");
        runtime.block_on(async {
            let (started_tx, started_rx) = futures::channel::oneshot::channel();
            let (finished_tx, finished_rx) = futures::channel::oneshot::channel();
            let (release_tx, release_rx) = mpsc::channel();
            let handle = runtime.spawn_blocking(move || {
                let _ = started_tx.send(());
                release_rx
                    .recv_timeout(TEST_TIMEOUT)
                    .expect("release worker");
                let _ = finished_tx.send(29);
            });
            started_rx.await.expect("blocking callback started");
            drop(handle);
            release_tx.send(()).expect("release detached callback");
            assert_eq!(finished_rx.await.expect("detached callback finished"), 29);
        });
    }

    #[test]
    fn blocking_panic_is_reported_when_polled() {
        let runtime = create_runtime().expect("create runtime");
        let handle = runtime.spawn_blocking(|| -> u8 { panic!("blocking panic vector") });
        let panic = runtime
            .block_on(AssertUnwindSafe(handle).catch_unwind())
            .expect_err("awaiting a panicked callback panics");
        let message = panic.downcast_ref::<String>().expect("panic message");
        assert!(message.contains("blocking panic vector"));
    }

    #[test]
    fn queued_blocking_cancellation_is_reported_when_polled() {
        let owner = tokio_crate::runtime::Builder::new_multi_thread()
            .max_blocking_threads(1)
            .build()
            .expect("create bounded blocking pool");
        let runtime = TokioRuntimeHandle::new(owner.handle().clone());
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let blocker = owner.spawn_blocking(move || {
            started_tx.send(()).expect("notify blocked worker");
            let _ = release_rx.recv_timeout(TEST_TIMEOUT);
        });
        started_rx
            .recv_timeout(TEST_TIMEOUT)
            .expect("worker started");
        let called = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let callback_called = Arc::clone(&called);
        let queued = runtime.spawn_blocking(move || {
            callback_called.store(true, std::sync::atomic::Ordering::SeqCst);
        });
        queued.0.abort();
        release_tx.send(()).expect("release blocking pool");
        let result = runtime.block_on(AssertUnwindSafe(queued).catch_unwind());
        owner.block_on(blocker).expect("worker finished");
        assert!(result.is_err());
        assert!(!called.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn last_owner_drop_in_async_context() {
        /// Use a child process so an unexpected double panic cannot abort tests.
        const CHILD_ENV: &str = "ARTI_TEST_LAST_RUNTIME_DROP";
        if std::env::var_os(CHILD_ENV).is_some() {
            let runtime = create_runtime().expect("create owned runtime");
            let caller = tokio_crate::runtime::Builder::new_current_thread()
                .build()
                .expect("create caller runtime");
            caller.block_on(async move { drop(runtime) });
            return;
        }
        let (_, module) = module_path!().split_once("::").expect("test module path");
        let output = std::process::Command::new(std::env::current_exe().expect("test binary"))
            .args([
                "--exact",
                &format!("{module}::last_owner_drop_in_async_context"),
                "--nocapture",
            ])
            .env(CHILD_ENV, "1")
            .output()
            .expect("run runtime-drop child test");
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("Cannot drop a runtime"),
            "{}",
            String::from_utf8_lossy(&output.stderr),
        );
    }
}
