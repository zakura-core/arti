//! Entry points for use with async_std runtimes.
pub use crate::impls::async_std::create_runtime as create_runtime_impl;
use crate::{RealCoarseTimeProvider, ToplevelBlockOn, compound::CompoundRuntime};
use std::io::Result as IoResult;

#[cfg(feature = "rustls")]
use crate::impls::rustls::RustlsProvider;

use async_executors::AsyncStd;

/// The async_std runtime using Rustls.
#[cfg(feature = "rustls")]
pub use AsyncStdRustlsRuntime as PreferredRuntime;

#[cfg(feature = "rustls")]
/// A [`Runtime`](crate::Runtime) powered by `async_std` and `rustls`.
#[derive(Clone)]
pub struct AsyncStdRustlsRuntime {
    /// The actual runtime object.
    inner: RustlsInner,
}

/// Implementation type for AsyncStdRustlsRuntime.
#[cfg(feature = "rustls")]
type RustlsInner = CompoundRuntime<
    AsyncStd,
    AsyncStd,
    RealCoarseTimeProvider,
    AsyncStd,
    AsyncStd,
    RustlsProvider,
    AsyncStd,
>;

#[cfg(feature = "rustls")]
crate::opaque::implement_opaque_runtime! {
    AsyncStdRustlsRuntime { inner: RustlsInner }
}

#[cfg(feature = "rustls")]
impl AsyncStdRustlsRuntime {
    /// Return a new [`AsyncStdRustlsRuntime`]
    ///
    /// Generally you should call this function only once, and then use
    /// [`Clone::clone()`] to create additional references to that
    /// runtime.
    pub fn create() -> IoResult<Self> {
        let rt = create_runtime_impl();
        let ct = RealCoarseTimeProvider::new();
        Ok(AsyncStdRustlsRuntime {
            inner: CompoundRuntime::new(rt, rt, ct, rt, rt, RustlsProvider::default(), rt),
        })
    }

    /// Return an [`AsyncStdRustlsRuntime`] for the currently running
    /// `async_std` executor.
    ///
    /// Note that since async_std executors are global, there is no distinction
    /// between this method and [`AsyncStdRustlsRuntime::create()`]: it is
    /// provided only for API consistency with the Tokio runtimes.
    pub fn current() -> IoResult<Self> {
        Self::create()
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

#[cfg(not(miri))] // async_std startup seems to fail under miri.
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
    fn current() {
        // We should actually have to run this inside a runtime with async_std,
        // but let's do it anyway to make sure that "current" works.
        let runtime = PreferredRuntime::create().unwrap();
        runtime.block_on(async {
            #[cfg(feature = "rustls")]
            assert!(AsyncStdRustlsRuntime::current().is_ok());
        });
    }

    #[test]
    fn debug() {
        #[cfg(feature = "rustls")]
        assert_eq!(
            format!("{:?}", AsyncStdRustlsRuntime::create().unwrap()),
            "AsyncStdRustlsRuntime { .. }"
        );
    }
}
