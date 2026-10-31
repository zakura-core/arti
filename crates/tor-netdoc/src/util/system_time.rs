//! System-time arithmetic for signed-document validity intervals.

#![forbid(unsafe_code)]

use std::time::{Duration, SystemTime};

/// Add a validity tolerance, saturating at the platform's maximum time.
pub(crate) fn saturating_add(time: SystemTime, duration: Duration) -> SystemTime {
    #[cfg(saturating_time_nightly)]
    {
        SystemTime::saturating_add(&time, duration)
    }
    #[cfg(not(saturating_time_nightly))]
    {
        time.checked_add(duration).unwrap_or_else(max_value)
    }
}

/// Subtract a validity tolerance, saturating at the platform's minimum time.
pub(crate) fn saturating_sub(time: SystemTime, duration: Duration) -> SystemTime {
    #[cfg(saturating_time_nightly)]
    {
        SystemTime::saturating_sub(&time, duration)
    }
    #[cfg(not(saturating_time_nightly))]
    {
        time.checked_sub(duration).unwrap_or_else(min_value)
    }
}

/// Return the lazily calculated maximum for this platform.
#[cfg(any(test, not(saturating_time_nightly)))]
fn max_value() -> SystemTime {
    static MAX: std::sync::LazyLock<SystemTime> =
        std::sync::LazyLock::new(|| find_limit(SystemTime::checked_add));
    *MAX
}

/// Return the lazily calculated minimum for this platform.
#[cfg(any(test, not(saturating_time_nightly)))]
fn min_value() -> SystemTime {
    static MIN: std::sync::LazyLock<SystemTime> =
        std::sync::LazyLock::new(|| find_limit(SystemTime::checked_sub));
    *MIN
}

/// Find a platform limit using the original saturating-time search.
///
/// An unchanged checked result counts as an overflow, preserving the workaround
/// for <https://github.com/rust-lang/rust/issues/149995>.
#[cfg(any(test, not(saturating_time_nightly)))]
fn find_limit(f: impl Fn(&SystemTime, Duration) -> Option<SystemTime>) -> SystemTime {
    const INITIAL_STEP: Duration = Duration::new(1_000_000_000_000_000_000, 0);
    const ONE_NS: Duration = Duration::new(0, 1);

    let mut step = INITIAL_STEP;
    let mut res = SystemTime::UNIX_EPOCH;
    loop {
        let next = f(&res, step);
        match next {
            Some(st) if st != res => res = st,
            None | Some(_) => {
                if step == ONE_NS {
                    return res;
                } else {
                    step = std::cmp::max(ONE_NS, step / 2);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONE_NS: Duration = Duration::new(0, 1);

    #[test]
    fn platform_limits() {
        let max = max_value();
        let min = min_value();
        assert_eq!(max.checked_add(Duration::ZERO), Some(max));
        assert_eq!(max.checked_add(ONE_NS), None);
        assert_eq!(max.checked_sub(Duration::ZERO), Some(max));
        assert_eq!(min.checked_sub(Duration::ZERO), Some(min));
        assert_eq!(min.checked_sub(ONE_NS), None);
        assert_eq!(min.checked_add(Duration::ZERO), Some(min));

        let before_max = max.checked_sub(ONE_NS).expect("below maximum");
        let after_min = min.checked_add(ONE_NS).expect("above minimum");
        assert_eq!(saturating_add(before_max, ONE_NS), max);
        assert_eq!(saturating_sub(after_min, ONE_NS), min);
    }

    #[test]
    fn saturating_boundary_vectors() {
        let max = max_value();
        let min = min_value();
        assert_eq!(saturating_add(max, Duration::ZERO), max);
        assert_eq!(saturating_add(max, ONE_NS), max);
        assert_eq!(saturating_sub(max, Duration::ZERO), max);
        assert_eq!(saturating_sub(min, Duration::ZERO), min);
        assert_eq!(saturating_sub(min, ONE_NS), min);
        assert_eq!(saturating_add(min, Duration::ZERO), min);

        let epoch = SystemTime::UNIX_EPOCH;
        for value in [
            epoch.checked_sub(ONE_NS).expect("before epoch"),
            epoch,
            epoch.checked_add(ONE_NS).expect("after epoch"),
        ] {
            assert_eq!(saturating_add(value, Duration::MAX), max);
            assert_eq!(saturating_sub(value, Duration::MAX), min);
        }
    }

    #[test]
    fn within_range_vectors() {
        let epoch = SystemTime::UNIX_EPOCH;
        let duration = Duration::new(86_400, 123);
        let before = epoch.checked_sub(duration).expect("before epoch");
        let after = epoch.checked_add(duration).expect("after epoch");
        assert_eq!(saturating_add(epoch, duration), after);
        assert_eq!(saturating_sub(epoch, duration), before);
        assert_eq!(saturating_add(before, duration), epoch);
        assert_eq!(saturating_sub(after, duration), epoch);
    }

    #[test]
    fn unchanged_checked_result_does_not_loop() {
        assert_eq!(find_limit(|time, _| Some(*time)), SystemTime::UNIX_EPOCH);
        assert_eq!(find_limit(|_, _| None), SystemTime::UNIX_EPOCH);
    }

    #[cfg(target_family = "unix")]
    #[test]
    fn unix_limits() {
        let epoch = SystemTime::UNIX_EPOCH;
        assert_eq!(
            max_value(),
            epoch
                .checked_add(Duration::new(i64::MAX as u64, 999_999_999))
                .expect("Unix maximum"),
        );
        assert_eq!(
            min_value(),
            epoch
                .checked_sub(Duration::new(i64::MAX as u64 + 1, 0))
                .expect("Unix minimum"),
        );
    }

    #[cfg(saturating_time_nightly)]
    #[test]
    fn nightly_limits_match_platform_search() {
        assert_eq!(max_value(), SystemTime::MAX);
        assert_eq!(min_value(), SystemTime::MIN);
    }
}
