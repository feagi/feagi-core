use crate::parallelism::feagi_signal::signal_errors::FeagiSignalClosed;

// NOTE: Creating a pair is left to each backend's own constructor rather than this trait. Static
// (embassy) backends need caller provided storage to build a pair, heap allocated ones do not

/// Which kind of signal (std, embassy) a notifier / listener pair is built from. Implemented by a
/// zero sized marker per kind
pub trait FeagiSignalBackend {
    /// Pushes values to the listener
    type Notifier<T>: FeagiSignalNotifier<T>;
    /// Takes the newest value pushed by any notifier
    type Listener<T>: FeagiSignalListener<T>;
}

/// Pushing half of a signal. Clone it for more notifiers on the same signal. The signal closes
/// once every notifier is dropped
pub trait FeagiSignalNotifier<T>: Clone {
    /// Replaces any value the listener has not taken yet. Never blocks or fails. Does nothing if
    /// the listener was dropped
    fn notify(&self, value: T) {
        self.notify_with(|pending| *pending = Some(value));
    }

    /// Updates the pending value in place, for merging instead of replacing (such as OR-ing
    /// flags). `None` means the listener has taken everything so far. Runs while the signal is
    /// locked (a critical section on embassy), so keep it short. Does nothing if the listener was
    /// dropped
    fn notify_with(&self, update: impl FnOnce(&mut Option<T>));
}

/// Taking half of a signal. Do NOT allow this to be clonable! There is exactly one listener per
/// signal
pub trait FeagiSignalListener<T> {
    /// Takes the pending value if there is one, otherwise returns `None` immediately. Errors with
    /// `FeagiSignalClosed` once every notifier is dropped and nothing is pending
    fn try_take(&mut self) -> Result<Option<T>, FeagiSignalClosed>;

    /// Waits for a value, then takes it. Errors with `FeagiSignalClosed` once every notifier is
    /// dropped and nothing is pending
    async fn wait(&mut self) -> Result<T, FeagiSignalClosed>;
}
