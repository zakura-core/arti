//! Proof of Work schemes for onion services
//!
//! This fork retains the existing feature-off error types for compatibility.
//! It does not implement onion-service puzzles, solvers or verifiers.
//!
//! Protocol background: <https://spec.torproject.org/hspow-spec/index.html>.

mod err;

pub mod v1;

pub use err::{Error, RuntimeError, SolutionError};
