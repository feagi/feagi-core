//! What every signal backend guards behind its own lock. The logic lives here once, and each
//! backend only decides where the state is stored and how it is locked

use crate::parallelism::feagi_signal::signal_errors::FeagiSignalClosed;
use core::task::{Context, Poll, Waker};

/// The shared state of one signal. Methods that return a `Waker` leave waking it to the caller,
/// who should do so after unlocking
pub(crate) struct SignalState<T> {
    pending: Option<T>,
    /// The listener's waker, while it waits
    waker: Option<Waker>,
    notifier_count: usize,
    listener_dropped: bool,
}

impl<T> SignalState<T> {
    /// State of a new pair, with one notifier
    pub(crate) const fn new() -> Self {
        SignalState {
            pending: None,
            waker: None,
            notifier_count: 1,
            listener_dropped: false,
        }
    }

    /// Runs `update` on the pending value, and returns the listener's waker if it should be woken
    pub(crate) fn notify_with(&mut self, update: impl FnOnce(&mut Option<T>)) -> Option<Waker> {
        if self.listener_dropped {
            return None;
        }
        update(&mut self.pending);
        if self.pending.is_some() {
            self.waker.take()
        } else {
            None
        }
    }

    pub(crate) fn try_take(&mut self) -> Result<Option<T>, FeagiSignalClosed> {
        match self.pending.take() {
            Some(value) => Ok(Some(value)),
            None if self.is_closed() => Err(FeagiSignalClosed),
            None => Ok(None),
        }
    }

    /// Takes the pending value, or registers `context`'s waker to be woken by the next notify or
    /// by the signal closing
    pub(crate) fn poll_take(&mut self, context: &mut Context<'_>) -> Poll<Result<T, FeagiSignalClosed>> {
        match self.try_take() {
            Ok(Some(value)) => Poll::Ready(Ok(value)),
            Err(closed) => Poll::Ready(Err(closed)),
            Ok(None) => {
                match &mut self.waker {
                    Some(waker) if waker.will_wake(context.waker()) => {}
                    waker => *waker = Some(context.waker().clone()),
                }
                Poll::Pending
            }
        }
    }

    pub(crate) fn add_notifier(&mut self) {
        self.notifier_count += 1;
    }

    /// Returns the listener's waker if this was the last notifier, so it can see the signal closed
    pub(crate) fn remove_notifier(&mut self) -> Option<Waker> {
        self.notifier_count -= 1;
        if self.is_closed() {
            self.waker.take()
        } else {
            None
        }
    }

    /// Drops any pending value, and makes later notifies do nothing
    pub(crate) fn remove_listener(&mut self) {
        self.listener_dropped = true;
        self.pending = None;
        self.waker = None;
    }

    fn is_closed(&self) -> bool {
        self.notifier_count == 0
    }
}
