//! Unsupported `v1` proof of work scheme. This fork never solves or verifies PoW.

use void::Void;

/// Stub for the runtime error type when `v1` is disabled
#[derive(Debug, Clone)]
pub struct RuntimeErrorV1(Void);

impl core::fmt::Display for RuntimeErrorV1 {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let _0 = &self.0;
        core::fmt::Display::fmt(_0, formatter)
    }
}

impl core::error::Error for RuntimeErrorV1 {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        Some(&self.0)
    }
}

/// Stub for the solution error type when `v1` is disabled
#[derive(Debug, Clone)]
pub struct SolutionErrorV1(Void);

impl core::fmt::Display for SolutionErrorV1 {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let _0 = &self.0;
        core::fmt::Display::fmt(_0, formatter)
    }
}

impl core::error::Error for SolutionErrorV1 {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        Some(&self.0)
    }
}
