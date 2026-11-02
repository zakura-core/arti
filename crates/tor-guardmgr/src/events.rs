//! Code to remotely notify other crates about changes in the status of the
//! `GuardMgr`.

use std::{pin::Pin, task::Poll};

use crate::skew::SkewEstimate;

use futures::{Stream, StreamExt};
use tor_basic_utils::skip_fmt;

/// A stream of [`SkewEstimate`] events.
///
/// Note that this stream can be lossy: if multiple events trigger before you
/// read from it, you will only get the most recent estimate.
//
// SEMVER NOTE: this type is re-exported from tor-circmgr.
#[derive(Clone)]
pub struct ClockSkewEvents {
    /// The `tor_async_utils::watch::Receiver` that we're wrapping.
    ///
    /// We wrap this type so that we don't expose its entire API, and so that we
    /// can migrate to some other implementation in the future if we want.
    pub(crate) inner: tor_async_utils::watch::Receiver<Option<SkewEstimate>>,
}

impl core::fmt::Debug for ClockSkewEvents {
    #[inline]
    fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        let mut builder = formatter.debug_struct("ClockSkewEvents");
        let arg = {
            struct FormattedField<'a>(&'a tor_async_utils::watch::Receiver<Option<SkewEstimate>>);
            impl<'a> core::fmt::Debug for FormattedField<'a> {
                fn fmt(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                    skip_fmt(self.0, formatter)
                }
            }
            FormattedField(&self.inner)
        };
        builder.field("inner", &arg);
        builder.finish()
    }
}

impl Stream for ClockSkewEvents {
    type Item = Option<SkewEstimate>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        self.inner.poll_next_unpin(cx)
    }
}
impl ClockSkewEvents {
    /// Return our best estimate of our current clock skew, based on reports from the
    /// guards and fallbacks we have contacted.
    pub fn get(&self) -> Option<SkewEstimate> {
        self.inner.borrow().clone()
    }
}
