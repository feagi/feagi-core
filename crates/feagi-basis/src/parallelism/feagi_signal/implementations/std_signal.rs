use crate::parallelism::feagi_signal::signal_errors::FeagiSignalClosed;
use crate::parallelism::feagi_signal::signal_state::SignalState;
use crate::parallelism::feagi_signal::signal_traits::{FeagiSignalBackend, FeagiSignalListener, FeagiSignalNotifier};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// Shared between every notifier and the listener of a std signal
type SharedState<T> = Arc<Mutex<SignalState<T>>>;

/// Locks the state. A panic inside a `notify_with` update poisons the mutex but cannot leave the
/// state inconsistent, so the poison is ignored
fn lock<T>(state: &SharedState<T>) -> MutexGuard<'_, SignalState<T>> {
    state.lock().unwrap_or_else(PoisonError::into_inner)
}

//region Backend

/// Signals shared through an `Arc` and guarded by a std `Mutex`. Works across threads, and with
/// any executor (including `block_on_thread`)
#[derive(Debug, Clone, Copy, Default)]
pub struct FeagiSignalStdBackend;

impl FeagiSignalBackend for FeagiSignalStdBackend {
    type Notifier<T> = FeagiSignalNotifierStd<T>;
    type Listener<T> = FeagiSignalListenerStd<T>;
}

impl FeagiSignalStdBackend {
    /// Creates a new, independent notifier and listener. Can be called any number of times.
    /// Clone the notifier for multiple notifiers
    pub fn create_signal_pair<T>() -> (FeagiSignalNotifierStd<T>, FeagiSignalListenerStd<T>) {
        let state: SharedState<T> = Arc::new(Mutex::new(SignalState::new()));
        (FeagiSignalNotifierStd { state: state.clone() }, FeagiSignalListenerStd { state })
    }
}

//endregion

//region Notifier

/// Pushing half of a std signal
pub struct FeagiSignalNotifierStd<T> {
    state: SharedState<T>,
}

impl<T> FeagiSignalNotifier<T> for FeagiSignalNotifierStd<T> {
    fn notify_with(&self, update: impl FnOnce(&mut Option<T>)) {
        let waker = lock(&self.state).notify_with(update);
        // Woken after unlocking, so the listener doesn't wake up to a locked state
        if let Some(waker) = waker {
            waker.wake();
        }
    }
}

impl<T> Clone for FeagiSignalNotifierStd<T> {
    fn clone(&self) -> Self {
        lock(&self.state).add_notifier();
        FeagiSignalNotifierStd { state: self.state.clone() }
    }
}

impl<T> Drop for FeagiSignalNotifierStd<T> {
    fn drop(&mut self) {
        let waker = lock(&self.state).remove_notifier();
        if let Some(waker) = waker {
            waker.wake();
        }
    }
}

//endregion

//region Listener

/// Taking half of a std signal
// NOTE: Do NOT allow this to be clonable! There is exactly one listener per signal
pub struct FeagiSignalListenerStd<T> {
    state: SharedState<T>,
}

impl<T> FeagiSignalListener<T> for FeagiSignalListenerStd<T> {
    fn try_take(&mut self) -> Result<Option<T>, FeagiSignalClosed> {
        lock(&self.state).try_take()
    }

    async fn wait(&mut self) -> Result<T, FeagiSignalClosed> {
        core::future::poll_fn(|context| lock(&self.state).poll_take(context)).await
    }
}

impl<T> Drop for FeagiSignalListenerStd<T> {
    fn drop(&mut self) {
        lock(&self.state).remove_listener();
    }
}

//endregion
