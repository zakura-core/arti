//! Implement a sink-blocking policy that allows a limited number of items to be sent.

use tor_error::{Bug, internal};

/// A sink-blocking [`Policy`](super::Policy) that can allow a limited number of items to be sent.
///
/// This policy may be in three states:
///  - Completely blocked
///  - Completely unblocked: Able to send an unlimited number of items.
///  - _Will become blocked_ after a certain number of items are sent.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CountingPolicy {
    /// The number of items that may currently be sent.
    ///
    /// [`UNLIMITED`] represents an unlimited number.
    remaining: u32,
}

/// The sentinel for an unlimited number of items.
const UNLIMITED: u32 = u32::MAX;

/// The largest possible limited number of cells in a CountingPolicy.
const MAX_LIMIT: u32 = UNLIMITED - 1;

impl CountingPolicy {
    /// Return a new unlimited CountingPolicy.
    pub(crate) fn new_unlimited() -> Self {
        Self {
            remaining: UNLIMITED,
        }
    }

    /// Return a new completely blocked CountingPolicy.
    pub(crate) fn new_blocked() -> Self {
        Self { remaining: 0 }
    }

    /// Return a new CountingPolicy that allows `n` items, and then becomes blocked.
    ///
    /// # Limitations:
    ///
    /// If `n` is greater than `MAX_LIMIT`, only `MAX_LIMIT` items will be allowed.
    pub(crate) fn new_limited(n: u32) -> Self {
        Self {
            remaining: n.min(MAX_LIMIT),
        }
    }

    /// Return a new CountingPolicy that allows up to `n` more items to be sent
    /// than this one.
    ///
    /// # Limitations:
    ///
    /// If the total number of allowed items would be greater than `MAX_LIMIT`,
    /// only `MAX_LIMIT` items will be allowed.
    //
    // Correctness: Note that this method returns a new CountingPolicy,
    // and does not change self.
    // Therefore it obeys the invariants of the `Policy` trait.
    fn saturating_add(&self, n: u32) -> Self {
        match self.remaining {
            UNLIMITED => Self::new_unlimited(),
            remaining => Self::new_limited(remaining.saturating_add(n)),
        }
    }
}

impl super::Policy for CountingPolicy {
    fn is_blocking(&self) -> bool {
        self.remaining == 0
    }

    // Correctness:
    //
    // This is the only method that takes a `&mut CountingPolicy`.
    // It can decrement the counter, but never increment it.
    // Therefore, it can cause `self` to become blocked,
    // but it cannot cause a blocked `self` to become unblocked.
    // Thus the invariants of the `Policy` trait are preserved.
    fn take_one(&mut self) -> Result<(), Bug> {
        match self.remaining {
            // Unlimited: nothing to do.
            UNLIMITED => Ok(()),
            0 => Err(internal!(
                "Tried to take_one() from a blocked CountingPolicy."
            )),
            remaining => {
                self.remaining = remaining - 1;
                Ok(())
            }
        }
    }
}

impl<S> super::SinkBlocker<S, CountingPolicy> {
    /// Put this `SinkBlocker` into a blocked state.
    pub(crate) fn set_blocked(&mut self) {
        self.update_policy(CountingPolicy::new_blocked());
    }

    /// Put this `SinkBlocker` into an unlimited state.
    pub(crate) fn set_unlimited(&mut self) {
        // Correctness: Note that this _replaces_ the Policy object,
        // and does not modify an existing Policy object.
        // This is the permitted way to make a SinkBlocker unblocked.
        self.update_policy(CountingPolicy::new_unlimited());
    }

    /// Allow `n` additional items to bypass the current blocking of this `SinkBlocker`.
    ///
    /// (This function has no effect if the `SinkBlocker` is currently unlimited.)
    pub(crate) fn allow_n_additional_items(&mut self, n: u32) {
        // Correctness: Note that this _replaces_ the Policy object,
        // and does not modify an existing Policy object.
        // This is the permitted way to make a SinkBlocker unblocked.
        self.update_policy(self.policy.saturating_add(n));
    }

    /// Return true if there is no limit on this policy.
    pub(crate) fn is_unlimited(&self) -> bool {
        self.policy.remaining == UNLIMITED
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
    use crate::util::sink_blocker::Policy as _;

    #[test]
    fn counting_unlimited() {
        let mut unlimited = CountingPolicy::new_unlimited();
        assert_eq!(unlimited.is_blocking(), false);
        assert!(unlimited.take_one().is_ok());
        assert!(unlimited.take_one().is_ok());
        assert_eq!(unlimited.is_blocking(), false);
        let u2 = unlimited.saturating_add(99);
        assert_eq!(u2.remaining, UNLIMITED); // still unlimited.
    }

    #[test]
    fn counting_blocked() {
        let mut blocked = CountingPolicy::new_blocked();
        assert_eq!(blocked.is_blocking(), true);
        assert!(blocked.take_one().is_err());
        assert_eq!(blocked.remaining, 0); // A failed take must leave it blocked.
        let mut u2 = blocked.saturating_add(99);
        assert_eq!(u2.remaining, 99); // New policy is limited to 99.
        assert_eq!(u2.is_blocking(), false);
        assert!(u2.take_one().is_ok());
        assert_eq!(u2.remaining, 98); // You take one down, you pass it around...
    }

    #[test]
    fn counting_limited() {
        let mut limited = CountingPolicy::new_limited(2);
        assert_eq!(limited.is_blocking(), false);
        assert!(limited.take_one().is_ok());
        assert_eq!(limited.is_blocking(), false);
        assert!(limited.take_one().is_ok());
        assert_eq!(limited.is_blocking(), true);
        assert!(limited.take_one().is_err());

        let limited = CountingPolicy::new_limited(99);
        let lim2 = limited.saturating_add(25);
        assert_eq!(lim2.remaining, 25 + 99);
        let mut lim3 = limited.saturating_add(u32::MAX);
        assert_eq!(lim3.remaining, MAX_LIMIT);
        assert!(lim3.take_one().is_ok());
        assert_eq!(lim3.remaining, MAX_LIMIT - 1); // Saturation stays finite.

        let mut limited = CountingPolicy::new_limited(u32::MAX);
        assert_eq!(limited.remaining, MAX_LIMIT);
        assert!(limited.take_one().is_ok());
        assert_eq!(limited.remaining, MAX_LIMIT - 1); // Clamping stays finite.
    }
}
