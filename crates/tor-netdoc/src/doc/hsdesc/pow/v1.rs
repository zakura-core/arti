//! Unsupported `v1` proof of work scheme. This fork never solves or verifies PoW.

use crate::Result;
use crate::doc::hsdesc::inner::HsInnerKwd;
use crate::parse::tokenize::Item;

/// Marker for a `pow-params v1` line which was not parsed
///
/// This fork remembers the presence of the advertised scheme without parsing
/// its parameters or computing a solution. Rebuilding cannot enable PoW.
///
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct PowParamsV1;

impl PowParamsV1 {
    /// Accept any Item as a 'PowParamsV1' without actually parsing it
    ///
    #[allow(clippy::unnecessary_wraps)]
    pub(super) fn from_item(_item: &Item<'_, HsInnerKwd>) -> Result<Self> {
        Ok(Self)
    }
}
