#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
// @@ begin lint list maintained by maint/add_warning @@
#![allow(renamed_and_removed_lints)] // @@REMOVE_WHEN(ci_arti_stable)
#![allow(unknown_lints)] // @@REMOVE_WHEN(ci_arti_nightly)
#![warn(missing_docs)]
#![warn(noop_method_call)]
#![warn(unreachable_pub)]
#![warn(clippy::all)]
#![deny(clippy::await_holding_lock)]
#![deny(clippy::cargo_common_metadata)]
#![deny(clippy::cast_lossless)]
#![deny(clippy::checked_conversions)]
#![allow(clippy::cognitive_complexity)] // See arti#2556
#![deny(clippy::debug_assert_with_mut_call)]
#![deny(clippy::exhaustive_enums)]
#![deny(clippy::exhaustive_structs)]
#![deny(clippy::expl_impl_clone_on_copy)]
#![deny(clippy::fallible_impl_from)]
#![deny(clippy::implicit_clone)]
#![deny(clippy::large_stack_arrays)]
#![warn(clippy::manual_ok_or)]
#![deny(clippy::missing_docs_in_private_items)]
#![warn(clippy::needless_borrow)]
#![warn(clippy::needless_pass_by_value)]
#![warn(clippy::option_option)]
#![warn(clippy::or_fun_call)]
#![deny(clippy::print_stderr)]
#![deny(clippy::print_stdout)]
#![warn(clippy::rc_buffer)]
#![deny(clippy::ref_option_ref)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![warn(clippy::trait_duplication_in_bounds)]
#![deny(clippy::unchecked_time_subtraction)]
#![deny(clippy::unnecessary_wraps)]
#![warn(clippy::unseparated_literal_suffix)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::mod_module_files)]
#![allow(clippy::let_unit_value)] // This can reasonably be done for explicitness
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::significant_drop_in_scrutinee)] // arti/-/merge_requests/588/#note_2812945
#![allow(clippy::result_large_err)] // temporary workaround for arti#587
#![allow(clippy::needless_raw_string_hashes)] // complained-about code is fine, often best
#![allow(clippy::needless_lifetimes)] // See arti#1765
#![allow(mismatched_lifetime_syntaxes)] // temporary workaround for arti#2060
#![allow(clippy::collapsible_if)] // See arti#2342
#![deny(clippy::unused_async)]
#![deny(clippy::string_slice)] // See arti#2571
#![allow(recursion_depth_exceeding_limit)] // arti#2715, rust/issues/159228
//! <!-- @@ end lint list maintained by maint/add_warning @@ -->

// TODO #1645 (either remove this, or decide to have it everywhere)
#![cfg_attr(not(all(feature = "full")), allow(unused))]

#[cfg(all(feature = "rustls", feature = "tokio"))]
pub(crate) mod impls;
pub mod task;

mod coarse_time;
mod compound;
mod dyn_time;
pub mod general;
pub mod join_set;
mod network;
mod opaque;
pub mod scheduler;
mod timer;
mod traits;
pub mod unimpl;
pub mod unix;

#[cfg(all(feature = "rustls", feature = "tokio"))]
use std::io;
pub use traits::{
    Blocking, CertifiedConn, CoarseTimeProvider, JoinSet, JoinSetProvider, NetStreamListener,
    NetStreamProvider, NoOpStreamOpsHandle, Runtime, SleepProvider, SpawnExt, StreamOps,
    TlsProvider, ToplevelBlockOn, ToplevelRuntime, UdpProvider, UdpSocket, UnsupportedStreamOp,
};

pub use coarse_time::{CoarseDuration, CoarseInstant, RealCoarseTimeProvider};
pub use dyn_time::DynTimeProvider;
pub use network::{
    CommonConnectOptions, CommonListenOptions, TcpConnectOptions, TcpListenOptions,
    UnixConnectOptions, UnixListenOptions,
};
pub use timer::{SleepProviderExt, Timeout, TimeoutError};

/// Traits used to describe TLS connections and objects that can
/// create them.
pub mod tls {
    #[cfg(all(feature = "rustls", feature = "tokio"))]
    pub use crate::impls::unimpl_tls::UnimplementedTls;
    pub use crate::traits::{
        CertifiedConn, TlsAcceptorSettings, TlsConnector, TlsServerUnsupported,
    };

    #[cfg(all(feature = "rustls", feature = "tokio"))]
    pub use crate::impls::rustls::RustlsProvider;
    #[cfg(all(feature = "rustls", feature = "tls-server", feature = "tokio"))]
    pub use crate::impls::rustls::rustls_server::{RustlsAcceptor, RustlsServerStream};
}

#[cfg(all(feature = "rustls", feature = "tokio"))]
pub mod tokio;

pub use compound::{CompoundRuntime, RuntimeSubstExt};

#[cfg(all(feature = "rustls", feature = "tokio"))]
use tokio as preferred_backend_mod;

/// The default runtime wrapper for embedded Tokio/Rustls clients.
///
/// The process [**may not fork**](crate#do-not-fork)
/// (except, very carefully, before exec)
/// after creating this or any other `Runtime`.
#[cfg(all(feature = "rustls", feature = "tokio"))]
#[derive(Clone)]
pub struct PreferredRuntime {
    /// The underlying runtime object.
    inner: preferred_backend_mod::PreferredRuntime,
}

#[cfg(all(feature = "rustls", feature = "tokio"))]
crate::opaque::implement_opaque_runtime! {
    PreferredRuntime { inner : preferred_backend_mod::PreferredRuntime }
}

#[cfg(all(feature = "rustls", feature = "tokio"))]
impl PreferredRuntime {
    /// Obtain a [`PreferredRuntime`] from the currently running asynchronous runtime.
    /// Generally, this is what you want.
    ///
    /// This tries to get a handle to a currently running asynchronous runtime, and
    /// wraps it; the returned [`PreferredRuntime`] isn't the same thing as the
    /// asynchronous runtime object itself (e.g. `tokio::runtime::Runtime`).
    ///
    /// # Errors
    ///
    /// Returns an error outside a Tokio runtime context. See
    /// [`tokio_crate::runtime::Handle::try_current`].
    ///
    /// # Usage notes
    ///
    /// Once you have a runtime returned by this function, you should
    /// just create more handles to it via [`Clone`].
    ///
    /// The process [**may not fork**](crate#do-not-fork)
    /// (except, very carefully, before exec)
    /// after creating this or any other `Runtime`.
    //
    // ## Note to Arti developers
    //
    // We should never call this from inside other Arti crates, or from
    // library crates that want to support multiple runtimes!  This
    // function is for Arti _users_ who want to wrap some existing Tokio
    // runtime as a [`Runtime`]. It is not for library
    // crates that want to work with multiple runtimes.
    pub fn current() -> io::Result<Self> {
        let rt = preferred_backend_mod::PreferredRuntime::current()?;

        Ok(Self { inner: rt })
    }

    /// Create and return a new instance of the default [`Runtime`].
    ///
    /// Generally you should call this function at most once, and then use
    /// [`Clone::clone()`] to create additional references to that runtime.
    ///
    /// Tokio users may want to avoid this function and instead obtain a runtime using
    /// [`PreferredRuntime::current`]: this function always _builds_ a runtime,
    /// and if you already have a runtime, that isn't what you want with Tokio.
    ///
    /// If you need more fine-grained control over a runtime, you can create it
    /// using an appropriate builder type or function.
    ///
    /// The process [**may not fork**](crate#do-not-fork)
    /// (except, very carefully, before exec)
    /// after creating this or any other `Runtime`.
    //
    // ## Note to Arti developers
    //
    // We should never call this from inside other Arti crates, or from
    // library crates that want to support multiple runtimes!  This
    // function is for Arti _users_ who want to wrap some existing Tokio
    // runtime as a [`Runtime`]. It is not for library
    // crates that want to work with multiple runtimes.
    pub fn create() -> io::Result<Self> {
        let rt = preferred_backend_mod::PreferredRuntime::create()?;

        Ok(Self { inner: rt })
    }

    /// Helper to run a single test function in a freshly created runtime.
    ///
    /// # Panics
    ///
    /// Panics if we can't create this runtime.
    ///
    /// # Warning
    ///
    /// This API is **NOT** for consumption outside Arti. Semver guarantees are not provided.
    #[doc(hidden)]
    pub fn run_test<P, F, O>(func: P) -> O
    where
        P: FnOnce(Self) -> F,
        F: futures::Future<Output = O>,
    {
        let runtime = Self::create().expect("Failed to create runtime");
        runtime.clone().block_on(func(runtime))
    }
}

/// Helpers for test_with_all_runtimes
///
/// # Warning
///
/// This API is **NOT** for consumption outside Arti. Semver guarantees are not provided.
#[doc(hidden)]
pub mod testing__ {
    /// A trait for an object that might represent a test failure, or which
    /// might just be `()`.
    pub trait TestOutcome {
        /// Abort if the test has failed.
        fn check_ok(&self);
    }
    impl TestOutcome for () {
        fn check_ok(&self) {}
    }
    impl<E: std::fmt::Debug> TestOutcome for Result<(), E> {
        fn check_ok(&self) {
            self.as_ref().expect("Test failure");
        }
    }
}

/// Run a test closure using the built-in Tokio/Rustls runtime.
///
/// Usually, prefer `tor_rtmock::MockRuntime::test_with_various` to this.
/// Use this macro only when you need to interact with things
/// that `MockRuntime` can't handle,
///
/// If everything in your test case is supported by `MockRuntime`,
/// you should use that instead:
/// that will give superior test coverage *and* a (more) deterministic test.
///
//
// NOTE(eta): changing this #[cfg] can affect tests inside this crate that use
//            this macro, like in scheduler.rs
#[macro_export]
#[cfg(all(feature = "rustls", feature = "tokio",))]
macro_rules! test_with_all_runtimes {
    ( $fn:expr ) => {{
        use $crate::testing__::TestOutcome;
        $crate::tokio::TokioRustlsRuntime::run_test($fn).check_ok();
    }};
}

/// Run a test closure, passing as argument one supported runtime.
///
/// Usually, prefer `tor_rtmock::MockRuntime::test_with_various` to this.
/// Use this macro only when you need to interact with things
/// that `MockRuntime` can't handle.
///
/// If everything in your test case is supported by `MockRuntime`,
/// you should use that instead:
/// that will give superior test coverage *and* a (more) deterministic test.
///
/// Uses the preferred Tokio/Rustls wrapper.
#[macro_export]
#[cfg(all(feature = "rustls", feature = "tokio",))]
macro_rules! test_with_one_runtime {
    ( $fn:expr ) => {{ $crate::PreferredRuntime::run_test($fn) }};
}

#[cfg(all(
    test,
    feature = "rustls",
    feature = "tokio",
    not(miri), // Many of these tests use real sockets or SystemTime.
))]
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
    #![allow(clippy::unnecessary_wraps)]
    use crate::SleepProviderExt;
    use crate::ToplevelRuntime;

    use crate::traits::*;

    use futures::io::{AsyncReadExt, AsyncWriteExt};
    use futures::stream::StreamExt;
    use std::io::Result as IoResult;
    use std::net::SocketAddr;
    use std::net::{Ipv4Addr, SocketAddrV4};
    use web_time_compat::{Duration, Instant, InstantExt, SystemTimeExt};

    // Test "sleep" with a tiny delay, and make sure that at least that
    // much delay happens.
    fn small_delay<R: ToplevelRuntime>(runtime: &R) -> IoResult<()> {
        let rt = runtime.clone();
        runtime.block_on(async {
            let i1 = Instant::get();
            let one_msec = Duration::from_millis(1);
            rt.sleep(one_msec).await;
            let i2 = Instant::get();
            assert!(i2 >= i1 + one_msec);
        });
        Ok(())
    }

    // Try a timeout operation that will succeed.
    fn small_timeout_ok<R: ToplevelRuntime>(runtime: &R) -> IoResult<()> {
        let rt = runtime.clone();
        runtime.block_on(async {
            let one_day = Duration::from_secs(86400);
            let outcome = rt.timeout(one_day, async { 413_u32 }).await;
            assert_eq!(outcome, Ok(413));
        });
        Ok(())
    }

    // Try a timeout operation that will time out.
    fn small_timeout_expire<R: ToplevelRuntime>(runtime: &R) -> IoResult<()> {
        use futures::future::pending;

        let rt = runtime.clone();
        runtime.block_on(async {
            let one_micros = Duration::from_micros(1);
            let outcome = rt.timeout(one_micros, pending::<()>()).await;
            assert_eq!(outcome, Err(crate::TimeoutError));
            assert_eq!(
                outcome.err().unwrap().to_string(),
                "Timeout expired".to_string()
            );
        });
        Ok(())
    }
    // Try a little wallclock delay.
    //
    // NOTE: This test will fail if the clock jumps a lot while it's
    // running.  We should use simulated time instead.
    fn tiny_wallclock<R: ToplevelRuntime>(runtime: &R) -> IoResult<()> {
        let rt = runtime.clone();
        runtime.block_on(async {
            let i1 = Instant::get();
            let now = runtime.wallclock();
            let one_millis = Duration::from_millis(1);
            let one_millis_later = now + one_millis;

            rt.sleep_until_wallclock(one_millis_later).await;

            let i2 = Instant::get();
            let newtime = runtime.wallclock();
            assert!(newtime >= one_millis_later);
            assert!(i2 - i1 >= one_millis);
        });
        Ok(())
    }

    // Try connecting to ourself and sending a little data.
    //
    // NOTE: requires Ipv4 localhost.
    fn self_connect_tcp<R: ToplevelRuntime>(runtime: &R) -> IoResult<()> {
        let localhost = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
        let rt1 = runtime.clone();

        let listen_options = Default::default();
        let listener =
            runtime.block_on(rt1.listen(&(SocketAddr::from(localhost)), &listen_options))?;
        let addr = listener.local_addr()?;

        runtime.block_on(async {
            let task1 = async {
                let mut buf = vec![0_u8; 11];
                let (mut con, _addr) = listener.incoming().next().await.expect("closed?")?;
                con.read_exact(&mut buf[..]).await?;
                IoResult::Ok(buf)
            };
            let task2 = async {
                let connect_options = Default::default();
                let mut con = rt1.connect(&addr, &connect_options).await?;
                con.write_all(b"Hello world").await?;
                con.flush().await?;
                IoResult::Ok(())
            };

            let (data, send_r) = futures::join!(task1, task2);
            send_r?;

            assert_eq!(&data?[..], b"Hello world");

            Ok(())
        })
    }

    // Try connecting to ourself and sending a little data.
    //
    // NOTE: requires Ipv4 localhost.
    fn self_connect_udp<R: ToplevelRuntime>(runtime: &R) -> IoResult<()> {
        let localhost = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
        let rt1 = runtime.clone();

        let socket1 = runtime.block_on(rt1.bind(&(localhost.into())))?;
        let addr1 = socket1.local_addr()?;

        let socket2 = runtime.block_on(rt1.bind(&(localhost.into())))?;
        let addr2 = socket2.local_addr()?;

        runtime.block_on(async {
            let task1 = async {
                let mut buf = [0_u8; 16];
                let (len, addr) = socket1.recv(&mut buf[..]).await?;
                IoResult::Ok((buf[..len].to_vec(), addr))
            };
            let task2 = async {
                socket2.send(b"Hello world", &addr1).await?;
                IoResult::Ok(())
            };

            let (recv_r, send_r) = futures::join!(task1, task2);
            send_r?;
            let (buff, addr) = recv_r?;
            assert_eq!(addr2, addr);
            assert_eq!(&buff, b"Hello world");

            Ok(())
        })
    }

    // Try out our incoming connection stream code.
    //
    // We launch a few connections and make sure that we can read data on
    // them.
    fn listener_stream<R: ToplevelRuntime>(runtime: &R) -> IoResult<()> {
        let localhost = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
        let rt1 = runtime.clone();

        let listen_options = Default::default();
        let listener = runtime
            .block_on(rt1.listen(&SocketAddr::from(localhost), &listen_options))
            .unwrap();
        let addr = listener.local_addr().unwrap();
        let mut stream = listener.incoming();

        runtime.block_on(async {
            let task1 = async {
                let mut n = 0_u32;
                loop {
                    let (mut con, _addr) = stream.next().await.unwrap()?;
                    let mut buf = [0_u8; 11];
                    con.read_exact(&mut buf[..]).await?;
                    n += 1;
                    if &buf[..] == b"world done!" {
                        break IoResult::Ok(n);
                    }
                }
            };
            let task2 = async {
                let connect_options = Default::default();
                for _ in 0_u8..5 {
                    let mut con = rt1.connect(&addr, &connect_options).await?;
                    con.write_all(b"Hello world").await?;
                    con.flush().await?;
                }
                let mut con = rt1.connect(&addr, &connect_options).await?;
                con.write_all(b"world done!").await?;
                con.flush().await?;
                con.close().await?;
                IoResult::Ok(())
            };

            let (n, send_r) = futures::join!(task1, task2);
            send_r?;

            assert_eq!(n?, 6);

            Ok(())
        })
    }

    // Exercise the unvalidated Tor TLS connector with an expired certificate.
    fn simple_tls<R: ToplevelRuntime>(runtime: &R) -> IoResult<()> {
        use futures_rustls::rustls::{
            self,
            pki_types::{CertificateDer, PrivatePkcs8KeyDer},
        };
        use std::sync::Arc;

        // The certificate and RSA key are the existing test.pfx fixture,
        // converted to DER without regenerating either value. The certificate
        // has Tor-style v3 encoding without extensions and is expired.
        let certificate = CertificateDer::from(include_bytes!("test-cert.der").to_vec());
        let private_key = PrivatePkcs8KeyDer::from(include_bytes!("test-key.der").to_vec());
        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![certificate.clone()], private_key.into())
            .unwrap();

        let localhost = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
        let listener = std::net::TcpListener::bind(localhost)?;
        let addr = listener.local_addr()?;

        let th = std::thread::spawn(move || {
            use std::io::{Read, Write};
            let (socket, _addr) = listener.accept()?;
            let connection = rustls::ServerConnection::new(Arc::new(config)).unwrap();
            let mut connection = rustls::StreamOwned::new(connection, socket);
            let mut buf = [0_u8; 16];
            loop {
                let n = connection.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                connection.write_all(&buf[..n])?;
                connection.flush()?;
            }
            IoResult::Ok(())
        });

        let connector = runtime.tls_connector();
        runtime.block_on(async {
            let text = b"I Suddenly Dont Understand Anything";
            let mut buf = vec![0_u8; text.len()];
            let conn = runtime.connect(&addr, &Default::default()).await?;
            let mut conn = connector.negotiate_unvalidated(conn, "Kan.Aya").await?;
            assert_eq!(
                conn.peer_certificate()?.unwrap().as_ref(),
                certificate.as_ref()
            );
            conn.write_all(text).await?;
            conn.flush().await?;
            conn.read_exact(&mut buf[..]).await?;
            assert_eq!(&buf[..], text);
            conn.close().await?;
            IoResult::Ok(())
        })?;
        th.join().unwrap()?;
        IoResult::Ok(())
    }

    fn simple_tls_server<R: ToplevelRuntime>(runtime: &R) -> IoResult<()> {
        let mut rng = tor_basic_utils::test_rng::testing_rng();
        let tls_cert = tor_cert_x509::TlsKeyAndCert::create(
            &mut rng,
            std::time::SystemTime::get(),
            "prospit.example.org",
            "derse.example.org",
        )
        .unwrap();
        let cert = tls_cert.certificates_der()[0].to_vec();
        let settings = TlsAcceptorSettings::new(tls_cert).unwrap();

        let Ok(tls_acceptor) = runtime.tls_acceptor(settings) else {
            println!("Skipping tls-server test for runtime {:?}", runtime);
            return IoResult::Ok(());
        };
        println!("Running tls-server test for runtime {:?}", runtime);

        let tls_connector = runtime.tls_connector();

        let localhost: SocketAddr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0).into();
        let rt1 = runtime.clone();

        let msg = b"Derse Reviles Him And Outlaws Frogs Wherever They Can";
        runtime.block_on(async move {
            let listen_options = Default::default();
            let listener = runtime.listen(&localhost, &listen_options).await.unwrap();
            let address = listener.local_addr().unwrap();

            let h1 = runtime
                .spawn_with_handle(async move {
                    let conn = listener.incoming().next().await.unwrap().unwrap().0;
                    let mut conn = tls_acceptor.negotiate_unvalidated(conn, "").await.unwrap();

                    let mut buf = vec![];
                    conn.read_to_end(&mut buf).await.unwrap();
                    (buf, conn.own_certificate().unwrap().unwrap().into_owned())
                })
                .unwrap();

            let h2 = runtime
                .spawn_with_handle(async move {
                    let connect_options = Default::default();
                    let conn = rt1.connect(&address, &connect_options).await.unwrap();
                    let mut conn = tls_connector
                        .negotiate_unvalidated(conn, "prospit.example.org")
                        .await
                        .unwrap();
                    conn.write_all(msg).await.unwrap();
                    conn.close().await.unwrap();
                    conn.peer_certificate().unwrap().unwrap().into_owned()
                })
                .unwrap();

            let (received, server_own_cert) = h1.await;
            let client_peer_cert = h2.await;
            assert_eq!(received, msg);
            assert_eq!(&server_own_cert, &cert);
            assert_eq!(&client_peer_cert, &cert);
        });
        IoResult::Ok(())
    }

    macro_rules! tests_with_runtime {
        { $runtime:expr  => $($id:ident),* $(,)? } => {
            $(
                #[test]
                fn $id() -> std::io::Result<()> {
                    super::$id($runtime)
                }
            )*
        }
    }

    macro_rules! runtime_tests {
        { $($id:ident),* $(,)? } =>
        {
           #[cfg(feature="tokio")]
            mod tokio_runtime_tests {
                tests_with_runtime! { &crate::tokio::PreferredRuntime::create()? => $($id),* }
            }
            #[cfg(feature = "tokio")]
            mod default_runtime_tests {
                tests_with_runtime! { &crate::PreferredRuntime::create()? => $($id),* }
            }
        }
    }

    macro_rules! tls_runtime_tests {
        { $($id:ident),* $(,)? } =>
        {
            #[cfg(all(feature="tokio", feature="rustls"))]
            mod tokio_rustls_tests {
                tests_with_runtime! {  &crate::tokio::TokioRustlsRuntime::create()? => $($id),* }
            }
            #[cfg(feature = "tokio")]
            mod default_runtime_tls_tests {
                tests_with_runtime! { &crate::PreferredRuntime::create()? => $($id),* }
            }
        }
    }

    runtime_tests! {
        small_delay,
        small_timeout_ok,
        small_timeout_expire,
        tiny_wallclock,
        self_connect_tcp,
        self_connect_udp,
        listener_stream,
    }

    tls_runtime_tests! {
        simple_tls,
        simple_tls_server,
    }
}
