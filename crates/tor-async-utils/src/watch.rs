//! Runtime-independent distribution of the most recent state.
//!
//! There is one sender and any number of receivers. Each receiver first yields
//! the current value, then observes updates; intermediate values may coalesce.
//! Notifications use the existing [`crate::oneshot_broadcast`] primitive.

use crate::oneshot_broadcast::{self, ReceiverFuture};
use futures::{Future, Sink, Stream};
use std::fmt;
use std::future::IntoFuture;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::task::{Context, Poll};

/// The current value and notification for the next update.
struct State<T> {
    value: T,
    changed: oneshot_broadcast::Receiver<()>,
}

/// The single writer of a watched value.
pub struct Sender<T> {
    shared: Arc<RwLock<State<T>>>,
    changed: Option<oneshot_broadcast::Sender<()>>,
}

/// A stream of snapshots of a watched value.
///
/// Cloning starts a fresh subscription, whose first item is the latest value.
pub struct Receiver<T> {
    shared: Arc<RwLock<State<T>>>,
    changed: ReceiverFuture<()>,
    initial: bool,
    closed: bool,
}

/// A send failed because no receivers remain.
pub struct SendError<T>(T);

/// An immediate receive did not produce a value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TryRecvError {
    /// No update is ready yet.
    Pending,
    /// The sender was dropped and all observed updates were consumed.
    Closed,
}

/// Create a watch initialized to [`Default::default`].
pub fn channel<T: Clone + Default>() -> (Sender<T>, Receiver<T>) {
    channel_with(T::default())
}

/// Create a watch initialized to `value`.
pub fn channel_with<T: Clone>(value: T) -> (Sender<T>, Receiver<T>) {
    let (changed_tx, changed_rx) = oneshot_broadcast::channel();
    let shared = Arc::new(RwLock::new(State {
        value,
        changed: changed_rx,
    }));
    let sender = Sender {
        shared,
        changed: Some(changed_tx),
    };
    let receiver = sender.receiver();
    (sender, receiver)
}

impl<T> Sender<T> {
    fn receiver(&self) -> Receiver<T> {
        let state = self.shared.read().unwrap_or_else(|e| e.into_inner());
        Receiver {
            shared: Arc::clone(&self.shared),
            changed: state.changed.clone().into_future(),
            initial: true,
            closed: false,
        }
    }

    /// Subscribe, including the current value as the first stream item.
    pub fn subscribe(&mut self) -> Receiver<T> {
        self.receiver()
    }

    /// Borrow the current value without notifying receivers.
    pub fn borrow(&mut self) -> impl Deref<Target = T> + '_ {
        ReadGuard(self.shared.read().unwrap_or_else(|e| e.into_inner()))
    }

    /// Borrow the current value mutably, notifying receivers on release.
    pub fn borrow_mut(&mut self) -> impl DerefMut<Target = T> + '_ {
        WriteGuard {
            state: Some(self.shared.write().unwrap_or_else(|e| e.into_inner())),
            changed: &mut self.changed,
        }
    }
}

impl<T> Receiver<T> {
    /// Borrow the latest value without consuming an update.
    pub fn borrow(&self) -> impl Deref<Target = T> + '_ {
        ReadGuard(self.shared.read().unwrap_or_else(|e| e.into_inner()))
    }
}

impl<T: Clone> Receiver<T> {
    /// Try to consume the current value or an update without waiting.
    pub fn try_recv(&mut self) -> Result<T, TryRecvError> {
        let mut cx = Context::from_waker(futures::task::noop_waker_ref());
        match Pin::new(self).poll_next(&mut cx) {
            Poll::Ready(Some(value)) => Ok(value),
            Poll::Ready(None) => Err(TryRecvError::Closed),
            Poll::Pending => Err(TryRecvError::Pending),
        }
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        let state = self.shared.read().unwrap_or_else(|e| e.into_inner());
        Self {
            shared: Arc::clone(&self.shared),
            changed: state.changed.clone().into_future(),
            initial: true,
            closed: false,
        }
    }
}

impl<T: Clone> Stream for Receiver<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
        if self.closed {
            return Poll::Ready(None);
        }
        if !self.initial {
            match Pin::new(&mut self.changed).poll(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Err(_)) => {
                    self.closed = true;
                    return Poll::Ready(None);
                }
                Poll::Ready(Ok(())) => {}
            }
        }
        let (value, changed) = {
            let state = self.shared.read().unwrap_or_else(|e| e.into_inner());
            (state.value.clone(), state.changed.clone().into_future())
        };
        self.changed = changed;
        self.initial = false;
        Poll::Ready(Some(value))
    }
}

impl<T> Sink<T> for Sender<T> {
    type Error = SendError<T>;

    fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(mut self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        if Arc::strong_count(&self.shared) == 1 {
            return Err(SendError(item));
        }
        *self.borrow_mut() = item;
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

/// Recovering a poisoned lock preserves the previous non-poisoning watch behavior.
struct ReadGuard<'a, T>(RwLockReadGuard<'a, State<T>>);

impl<T> Deref for ReadGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0.value
    }
}

struct WriteGuard<'a, T> {
    state: Option<RwLockWriteGuard<'a, State<T>>>,
    changed: &'a mut Option<oneshot_broadcast::Sender<()>>,
}

impl<T> Deref for WriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.state.as_ref().expect("guard released").value
    }
}

impl<T> DerefMut for WriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.state.as_mut().expect("guard released").value
    }
}

impl<T> Drop for WriteGuard<'_, T> {
    fn drop(&mut self) {
        let (changed_tx, changed_rx) = oneshot_broadcast::channel();
        self.state.as_mut().expect("guard released").changed = changed_rx;
        let previous = self.changed.replace(changed_tx).expect("sender dropped");
        // Publish the value and next notification atomically, then unlock before
        // running arbitrary receiver wakers.
        drop(self.state.take());
        previous.send(());
    }
}

impl<T> fmt::Debug for Sender<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Sender").finish()
    }
}

impl<T> fmt::Debug for Receiver<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Receiver").finish()
    }
}

impl<T> fmt::Debug for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SendError")
    }
}

impl<T> fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("watch channel has no receivers")
    }
}

impl<T> std::error::Error for SendError<T> {}

#[cfg(test)]
mod test {
    use super::*;
    use futures::{FutureExt as _, SinkExt as _, StreamExt as _};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::task::{Wake, Waker};
    use tor_rtcompat::SpawnExt as _;

    #[test]
    fn snapshots_and_subscriptions() {
        let (mut tx, mut rx) = channel_with(1);
        assert_eq!(*rx.borrow(), 1);
        assert_eq!(rx.try_recv(), Ok(1));
        assert_eq!(rx.try_recv(), Err(TryRecvError::Pending));
        *tx.borrow_mut() = 2;
        *tx.borrow_mut() = 3;
        assert_eq!(*rx.borrow(), 3);
        assert_eq!(rx.try_recv(), Ok(3));
        assert_eq!(rx.try_recv(), Err(TryRecvError::Pending));
        assert_eq!(rx.clone().try_recv(), Ok(3));
        assert_eq!(tx.subscribe().try_recv(), Ok(3));
    }

    #[test]
    fn closure_delivers_last_value() {
        let (mut tx, mut rx) = channel_with(1);
        assert_eq!(rx.try_recv(), Ok(1));
        *tx.borrow_mut() = 2;
        drop(tx);
        assert_eq!(rx.try_recv(), Ok(2));
        assert_eq!(rx.try_recv(), Err(TryRecvError::Closed));
        assert_eq!(rx.try_recv(), Err(TryRecvError::Closed));
        let mut after_close = rx.clone();
        assert_eq!(after_close.try_recv(), Ok(2));
        assert_eq!(after_close.try_recv(), Err(TryRecvError::Closed));
    }

    #[test]
    fn closed_send_and_resubscribe() {
        let (mut tx, rx) = channel_with(1);
        drop(rx);
        assert!(futures::executor::block_on(tx.send(2)).is_err());
        assert_eq!(*tx.borrow(), 1);
        let mut rx = tx.subscribe();
        futures::executor::block_on(tx.send(3)).unwrap();
        assert_eq!(rx.try_recv(), Ok(3));
    }

    #[derive(Default)]
    struct WakeCounter(AtomicUsize);

    impl Wake for WakeCounter {
        fn wake(self: Arc<Self>) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn cancellation_replaces_and_removes_wakers() {
        let (mut tx, mut rx) = channel_with(0);
        assert_eq!(rx.try_recv(), Ok(0));
        let first = Arc::new(WakeCounter::default());
        let second = Arc::new(WakeCounter::default());
        let first_waker = Waker::from(Arc::clone(&first));
        let second_waker = Waker::from(Arc::clone(&second));
        assert!(
            Pin::new(&mut rx)
                .poll_next(&mut Context::from_waker(&first_waker))
                .is_pending()
        );
        assert!(
            Pin::new(&mut rx)
                .poll_next(&mut Context::from_waker(&second_waker))
                .is_pending()
        );
        *tx.borrow_mut() = 1;
        assert_eq!(first.0.load(Ordering::SeqCst), 0);
        assert_eq!(second.0.load(Ordering::SeqCst), 1);
        assert_eq!(rx.try_recv(), Ok(1));
        assert!(
            Pin::new(&mut rx)
                .poll_next(&mut Context::from_waker(&second_waker))
                .is_pending()
        );
        drop(rx);
        *tx.borrow_mut() = 2;
        assert_eq!(second.0.load(Ordering::SeqCst), 1);
    }

    struct ReadOnWake(Receiver<usize>);

    impl Wake for ReadOnWake {
        fn wake(self: Arc<Self>) {
            // A notification must run after releasing the value's write lock.
            assert_eq!(*self.0.borrow(), 1);
        }
    }

    #[test]
    fn wake_can_read_the_published_value() {
        let (mut tx, mut rx) = channel_with(0);
        assert_eq!(rx.try_recv(), Ok(0));
        let waker = Waker::from(Arc::new(ReadOnWake(rx.clone())));
        assert!(
            Pin::new(&mut rx)
                .poll_next(&mut Context::from_waker(&waker))
                .is_pending()
        );
        *tx.borrow_mut() = 1;
        assert_eq!(rx.try_recv(), Ok(1));
    }

    #[test]
    fn writer_unwind_does_not_poison_the_watch() {
        let (mut tx, mut rx) = channel_with(0);
        assert_eq!(rx.try_recv(), Ok(0));
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            *tx.borrow_mut() = 1;
            // Hold a second mutable guard during unwinding.
            let _guard = tx.borrow_mut();
            panic!("writer failed");
        }));
        assert!(result.is_err());
        assert_eq!(rx.try_recv(), Ok(1));
        *tx.borrow_mut() = 2;
        assert_eq!(rx.try_recv(), Ok(2));
    }

    #[test]
    fn multiple_tasks_observe_updates_and_eof() {
        tor_rtmock::MockRuntime::test_with_various(|rt| async move {
            let (mut tx, rx) = channel_with(0);
            let mut joins = Vec::new();
            for _ in 0..20 {
                let mut rx = rx.clone();
                joins.push(
                    rt.spawn_with_handle(async move {
                        let mut last = 0;
                        while let Some(value) = rx.next().await {
                            assert!(value >= last);
                            last = value;
                            tor_rtcompat::task::yield_now().await;
                        }
                        assert_eq!(last, 100);
                    })
                    .unwrap(),
                );
            }
            rt.spawn(async move {
                for value in 1..=100 {
                    *tx.borrow_mut() = value;
                    tor_rtcompat::task::yield_now().await;
                }
            })
            .unwrap();
            for join in joins {
                join.await;
            }
        });
    }
}
