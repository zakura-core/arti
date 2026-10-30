//! Proof of Work schemes for onion services
//!
//! Tor supports optional proof-of-work client puzzles, for mitigating denial of
//! service attacks on onion services. This crate implements the specific puzzle
//! algorithms we use, and components for client and service integration.
//!
//! This fork retains compatibility error types but no puzzle solver or verifier.

mod err;

pub mod v1;

pub use err::{Error, RuntimeError, SolutionError};
