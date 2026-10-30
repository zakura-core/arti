//! Unsupported `v1` proof of work scheme. This fork never solves or verifies PoW.

use void::Void;

/// Stub for the runtime error type when `v1` is disabled
#[derive(Debug, Clone, derive_more::Display, derive_more::Error)]
pub struct RuntimeErrorV1(Void);

/// Stub for the solution error type when `v1` is disabled
#[derive(Debug, Clone, derive_more::Display, derive_more::Error)]
pub struct SolutionErrorV1(Void);
