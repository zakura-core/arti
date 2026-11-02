//! Configuration elements for the guard manager

use tor_basic_utils::define_accessor_trait;
use tor_dircommon::fallback::FallbackList;

define_accessor_trait! {
    /// Configuration for a guard manager
    ///
    /// If the guard manager gains new configurabilities, this trait will gain additional
    /// supertraits, as an API break.
    ///
    /// Prefer to use `TorClientConfig`, which will always implement this trait.
    pub trait GuardMgrConfig {
        fallbacks: FallbackList,
    }
}

/// Helpers for testing configuration
#[cfg(any(test, feature = "testing"))]
pub(crate) mod testing {
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

    /// A dummy test configuration, with transparent fields for testing
    #[derive(Default, Debug)]
    #[allow(clippy::exhaustive_structs)]
    #[allow(missing_docs)]
    pub struct TestConfig {
        pub fallbacks: FallbackList,
    }

    impl core::convert::AsRef<FallbackList> for TestConfig {
        #[inline]
        fn as_ref(&self) -> &FallbackList {
            &self.fallbacks
        }
    }
    impl GuardMgrConfig for TestConfig {}
}
