//! Entry points for use with smol runtimes.
//! This crate helps define a slim API around our async runtime so that we
//! can easily swap it out.

/// Re-export the Smol runtime constructor implemented in `impls/smol.rs`.
pub use crate::impls::smol::create_runtime as create_runtime_impl;

use crate::{RealCoarseTimeProvider, ToplevelBlockOn, compound::CompoundRuntime};
use std::io::Result as IoResult;

#[cfg(feature = "rustls")]
use crate::impls::rustls::RustlsProvider;

// Bring in our SmolRuntime type.
use crate::impls::smol::SmolRuntime;

/// The smol runtime using Rustls.
#[cfg(feature = "rustls")]
pub use SmolRustlsRuntime as PreferredRuntime;

/// A [`Runtime`](crate::Runtime) powered by smol and rustls.
#[derive(Clone)]
#[cfg(feature = "rustls")]
pub struct SmolRustlsRuntime {
    /// The actual runtime object.
    inner: RustlsInner,
}

/// Implementation type for SmolRuntime using Rustls.
#[cfg(feature = "rustls")]
type RustlsInner = CompoundRuntime<
    SmolRuntime,
    SmolRuntime,
    RealCoarseTimeProvider,
    SmolRuntime,
    SmolRuntime,
    RustlsProvider,
    SmolRuntime,
>;

#[cfg(feature = "rustls")]
crate::opaque::implement_opaque_runtime! {
    SmolRustlsRuntime { inner: RustlsInner }
}

#[cfg(feature = "rustls")]
impl SmolRustlsRuntime {
    /// Create a new `SmolRustlsRuntime` (owns its executor).
    pub fn create() -> IoResult<Self> {
        let rt = create_runtime_impl();
        let ct = RealCoarseTimeProvider::new();
        Ok(SmolRustlsRuntime {
            inner: CompoundRuntime::new(
                rt.clone(),
                rt.clone(),
                ct,
                rt.clone(),
                rt.clone(),
                RustlsProvider::default(),
                rt.clone(),
            ),
        })
    }

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

#[cfg(all(test, not(miri)))]
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
    fn debug() {
        #[cfg(feature = "rustls")]
        assert_eq!(
            format!("{:?}", SmolRustlsRuntime::create().unwrap()),
            "SmolRustlsRuntime { .. }"
        );
    }
}
