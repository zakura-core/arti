//! Unsupported `v1` proof of work scheme. This fork never solves or verifies PoW.

use tor_bytes::{EncodeResult, Reader, Result, Writer};
use void::Void;

/// Uninhabited compatibility type for the unsupported `v1` scheme
///
/// This type can be named but can never be constructed in this fork.
/// The reader will generate an [`super::UnrecognizedProofOfWork`] instead.
///
#[derive(Debug, Clone, PartialEq)]
pub struct ProofOfWorkV1(Void);

impl ProofOfWorkV1 {
    /// Stub reader implementation; never matches
    #[allow(clippy::unnecessary_wraps)]
    pub(super) fn try_take_body_from(_scheme: u8, _b: &mut Reader<'_>) -> Result<Option<Self>> {
        Ok(None)
    }

    /// Stub writer implementation; uncallable due to void type
    pub(super) fn write_onto<B: Writer + ?Sized>(&self, _b: &mut B) -> EncodeResult<()> {
        void::unreachable(self.0)
    }
}
