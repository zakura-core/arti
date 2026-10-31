//! Fused oneshot channels that preserve cancellation in
//! [`select!`](https://docs.rs/futures/0.3.34/futures/macro.select.html).
//!
//! A bare [`futures::channel::oneshot::Receiver`] doesn't work properly with
//! `futures::select!`, because it has a broken
//! [`FusedFuture`](futures::future::FusedFuture)
//! implementation.
//! See [futures-rs #2455](https://github.com/rust-lang/futures-rs/issues/2455).
//!
//! Wrapping it up in a [`future::Fuse`](futures::future::Fuse) works around this,
//! with a minor performance penalty.
//!
//! ### Limitations
//!
//! The API of this [`Receiver`] is rather more limited.
//! For example, it lacks `.try_recv()`.
//
// The veneer is rather thin and the types from `futures-rs` show through.
// If we change this in the future, it will be a breaking change.
use futures::FutureExt as _;
use futures::channel::oneshot as fut_oneshot;

pub use fut_oneshot::Canceled;

/// `oneshot::Sender` type alias
//
// This has to be `pub type` rather than `pub use` so that
// (i) call sites don't trip the "disallowed types" lint
// (ii) we can apply a fine-grained allow, here.
#[allow(clippy::disallowed_types)]
pub type Sender<T> = fut_oneshot::Sender<T>;

/// A fused oneshot receiver that preserves cancellation in `select!`.
#[allow(clippy::disallowed_types)]
pub type Receiver<T> = futures::future::Fuse<fut_oneshot::Receiver<T>>;

/// Return a fresh oneshot channel
pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    #[allow(clippy::disallowed_methods)]
    let (tx, rx) = fut_oneshot::channel();
    (tx, rx.fuse())
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{future::FusedFuture, task::noop_waker};
    use std::task::{Context, Poll};

    const VALUE: u32 = 41;

    fn poll_once<T>(receiver: &mut Receiver<T>) -> Poll<Result<T, Canceled>> {
        receiver.poll_unpin(&mut Context::from_waker(&noop_waker()))
    }

    /// The pinned upstream receiver hides cancellation from `select!`.
    /// Revisit the wrapper if an upstream update changes this control case.
    #[test]
    #[allow(clippy::disallowed_methods, clippy::disallowed_types)]
    fn native_receiver_skips_unobserved_cancellation() {
        let (sender, mut receiver) = fut_oneshot::channel::<u32>();
        drop(sender);
        assert!(receiver.is_terminated());
        let selected = async {
            futures::select! {
                result = receiver => Some(result),
                complete => None,
            }
        }
        .now_or_never();
        assert_eq!(selected, Some(None));
    }

    #[test]
    fn select_observes_cancellation_before_first_poll() {
        let (sender, mut receiver) = channel::<u32>();
        drop(sender);
        assert!(!receiver.is_terminated());
        let selected = async {
            futures::select! {
                result = receiver => Some(result),
                complete => None,
            }
        }
        .now_or_never();
        assert_eq!(selected, Some(Some(Err(Canceled))));
        assert!(receiver.is_terminated());
        assert!(poll_once(&mut receiver).is_pending());
    }

    #[test]
    fn cancellation_after_pending_is_observed_once() {
        let (sender, mut receiver) = channel::<u32>();
        assert!(poll_once(&mut receiver).is_pending());
        drop(sender);
        assert!(!receiver.is_terminated());
        assert_eq!(poll_once(&mut receiver), Poll::Ready(Err(Canceled)));
        assert!(receiver.is_terminated());
        assert!(poll_once(&mut receiver).is_pending());
    }

    #[test]
    fn sent_value_is_observed_once() {
        let (sender, mut receiver) = channel();
        assert!(poll_once(&mut receiver).is_pending());
        assert!(sender.send(VALUE).is_ok());
        assert!(!receiver.is_terminated());
        assert_eq!(poll_once(&mut receiver), Poll::Ready(Ok(VALUE)));
        assert!(receiver.is_terminated());
        assert!(poll_once(&mut receiver).is_pending());
    }

    #[test]
    fn receiver_drop_notifies_sender_and_preserves_unsent_value() {
        let (mut sender, receiver) = channel();
        drop(receiver);
        assert!(sender.is_canceled());
        assert_eq!(sender.cancellation().now_or_never(), Some(()));
        assert_eq!(sender.send(VALUE), Err(VALUE));
    }
}
