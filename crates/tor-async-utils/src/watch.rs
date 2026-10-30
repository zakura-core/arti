//! Runtime-independent distribution of the most recent state.
//!
//! There is one sender and any number of receivers. Each receiver first yields
//! the current value, then observes updates; intermediate values may coalesce.
//! Notifications use [`event_listener`], as in the directory event publisher.

use event_listener::{Event, EventListener};
use futures::{Future, Sink, Stream};
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::task::{Context, Poll};

/// The current value and its publication state.
struct State<T> {
    /// The most recently published value.
    value: T,
    /// Changes whenever a writer releases a mutable value guard.
    ///
    /// Wrapping is intentional: receivers compare for inequality, not order.
    version: usize,
    /// Whether the unique sender has been dropped.
    closed: bool,
}

/// Value and notifications shared by the sender and receivers.
struct Shared<T> {
    /// Value, version and closure are observed atomically under one lock.
    state: RwLock<State<T>>,
    /// Wakes receivers to recheck the publication state.
    event: Event,
}

/// The single writer of a watched value.
///
/// Dropping this sender closes the stream after any pending final value.
/// Closing its [`Sink`] flushes updates without closing subscriptions.
pub struct Sender<T> {
    /// The value and its publication state.
    shared: Arc<Shared<T>>,
}

/// A stream of snapshots of a watched value.
///
/// Cloning starts a fresh subscription, whose first item is the latest value.
pub struct Receiver<T> {
    /// Shared value, also retaining the value after the sender drops.
    shared: Arc<Shared<T>>,
    /// Registered before checking publication state to prevent lost wakeups.
    listener: EventListener,
    /// Last observed version, or None before the initial snapshot.
    version: Option<usize>,
}

/// A send failed because no receivers remain.
pub struct SendError<T>(T);

impl<T> SendError<T> {
    /// Recover the value that could not be sent.
    pub fn into_inner(self) -> T {
        self.0
    }
}

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
    let shared = Arc::new(Shared {
        state: RwLock::new(State {
            value,
            version: 0,
            closed: false,
        }),
        event: Event::new(),
    });
    let sender = Sender { shared };
    let receiver = sender.receiver();
    (sender, receiver)
}

impl<T> Sender<T> {
    /// Start a fresh subscription at the latest value and its next
    /// notification.
    fn receiver(&self) -> Receiver<T> {
        Receiver {
            shared: Arc::clone(&self.shared),
            listener: self.shared.event.listen(),
            version: None,
        }
    }

    /// Subscribe, including the current value as the first stream item.
    pub fn subscribe(&mut self) -> Receiver<T> {
        self.receiver()
    }

    /// Borrow the current value without notifying receivers.
    pub fn borrow(&mut self) -> impl Deref<Target = T> + '_ {
        ReadGuard(self.shared.state.read().unwrap_or_else(|e| e.into_inner()))
    }

    /// Borrow the current value mutably, notifying receivers on release.
    pub fn borrow_mut(&mut self) -> impl DerefMut<Target = T> + '_ {
        WriteGuard {
            state: Some(self.shared.state.write().unwrap_or_else(|e| e.into_inner())),
            event: &self.shared.event,
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.shared
            .state
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .closed = true;
        self.shared.event.notify(usize::MAX);
    }
}

impl<T> Receiver<T> {
    /// Borrow the latest value without consuming an update.
    pub fn borrow(&self) -> impl Deref<Target = T> + '_ {
        ReadGuard(self.shared.state.read().unwrap_or_else(|e| e.into_inner()))
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
        Self {
            shared: Arc::clone(&self.shared),
            listener: self.shared.event.listen(),
            version: None,
        }
    }
}

impl<T: Clone> Stream for Receiver<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
        loop {
            // Register the listener before reading the state: a concurrent
            // writer must either make the value ready or notify this listener.
            let (value, version, closed) = {
                let state = self.shared.state.read().unwrap_or_else(|e| e.into_inner());
                let value = (self.version != Some(state.version)).then(|| state.value.clone());
                (value, state.version, state.closed)
            };
            if let Some(value) = value {
                self.version = Some(version);
                return Poll::Ready(Some(value));
            }
            // Deliver a final unseen snapshot before returning EOF.
            if closed {
                return Poll::Ready(None);
            }
            match Pin::new(&mut self.listener).poll(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(()) => self.listener = self.shared.event.listen(),
            }
        }
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

/// An immutable value guard, retaining the previous non-poisoning behavior.
struct ReadGuard<'a, T>(RwLockReadGuard<'a, State<T>>);

impl<T> Deref for ReadGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0.value
    }
}

/// A mutable value guard that notifies only after releasing the write lock.
struct WriteGuard<'a, T> {
    /// The lock to release before waking receivers.
    state: Option<RwLockWriteGuard<'a, State<T>>>,
    /// The event to notify after publishing the value and releasing the lock.
    event: &'a Event,
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
        let state = self.state.as_mut().expect("guard released");
        state.version = state.version.wrapping_add(1);
        // Publish the value and version atomically, then unlock before running
        // arbitrary receiver wakers.
        drop(self.state.take());
        self.event.notify(usize::MAX);
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
    use futures::{SinkExt as _, StreamExt as _};
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
    fn version_wraparound_preserves_updates_and_eof() {
        let (mut tx, mut rx) = channel_with(1);
        tx.shared.state.write().expect("state lock").version = usize::MAX;
        assert_eq!(rx.try_recv(), Ok(1));
        *tx.borrow_mut() = 2;
        assert_eq!(tx.shared.state.read().expect("state lock").version, 0);
        assert_eq!(rx.try_recv(), Ok(2));
        *tx.borrow_mut() = 3;
        drop(tx);
        assert_eq!(rx.try_recv(), Ok(3));
        assert_eq!(rx.try_recv(), Err(TryRecvError::Closed));
    }

    #[test]
    fn closed_send_and_resubscribe() {
        let (mut tx, rx) = channel_with(1);
        drop(rx);
        let error = futures::executor::block_on(tx.send(2)).expect_err("no receivers");
        assert_eq!(error.into_inner(), 2);
        assert_eq!(*tx.borrow(), 1);
        let mut rx = tx.subscribe();
        futures::executor::block_on(tx.send(3)).expect("receiver remains");
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
    fn dropping_an_idle_sender_wakes_the_receiver() {
        let (tx, mut rx) = channel_with(0);
        assert_eq!(rx.try_recv(), Ok(0));
        let counter = Arc::new(WakeCounter::default());
        let waker = Waker::from(Arc::clone(&counter));
        assert!(
            Pin::new(&mut rx)
                .poll_next(&mut Context::from_waker(&waker))
                .is_pending()
        );
        drop(tx);
        assert_eq!(counter.0.load(Ordering::SeqCst), 1);
        assert_eq!(rx.try_recv(), Err(TryRecvError::Closed));
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
                    .expect("spawn reader"),
                );
            }
            rt.spawn(async move {
                for value in 1..=100 {
                    *tx.borrow_mut() = value;
                    tor_rtcompat::task::yield_now().await;
                }
            })
            .expect("spawn writer");
            for join in joins {
                join.await;
            }
        });
    }

    #[test]
    fn parallel_readers_clones_and_sender_drop() {
        const READERS: usize = 8;
        const LAST: usize = 10_000;
        let (mut tx, rx) = channel_with(0);
        let start = Arc::new(std::sync::Barrier::new(READERS + 1));
        std::thread::scope(|scope| {
            let mut readers = Vec::new();
            for _ in 0..READERS {
                let mut rx = rx.clone();
                let start = Arc::clone(&start);
                readers.push(scope.spawn(move || {
                    start.wait();
                    futures::executor::block_on(async move {
                        let mut last = 0;
                        while let Some(value) = rx.next().await {
                            assert!(value >= last);
                            last = value;
                            assert!(*rx.borrow() >= value);
                            // Exercise a fresh subscription while another OS
                            // thread updates the shared value or drops its
                            // sender.
                            let mut cloned = rx.clone();
                            assert!(cloned.next().await.expect("initial snapshot") >= value);
                            std::thread::yield_now();
                        }
                        assert_eq!(last, LAST);
                    });
                }));
            }
            drop(rx);
            start.wait();
            for value in 1..=LAST {
                *tx.borrow_mut() = value;
                if value % 16 == 0 {
                    std::thread::yield_now();
                }
            }
            drop(tx);
            for reader in readers {
                reader.join().expect("reader succeeded");
            }
        });
    }
}
