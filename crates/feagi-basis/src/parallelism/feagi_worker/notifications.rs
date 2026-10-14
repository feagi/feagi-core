use thiserror::Error;

//region Notification Channels

/// Sending half of a worker notification channel.
pub trait WorkerNotificationSender<CN> {
    /// Waits for capacity, then sends. Returns the value if the channel is closed.
    fn send(&self, value: CN) -> impl core::future::Future<Output = Result<(), CN>>;
    /// Sends without waiting. Returns the value if the channel is full or closed.
    fn try_send(&self, value: CN) -> Result<(), CN>;
}

/// Receiving half of a worker notification channel.
pub trait WorkerNotificationReceiver<CN> {
    /// Waits for the next value. `None` means every sender is gone.
    fn recv(&self) -> impl core::future::Future<Output = Option<CN>>;
    /// Returns immediately with a value, or reports empty or closed.
    fn try_recv(&self) -> Result<CN, NotificationReceiveError<CN>>;
}


//endregion

//region Worker Notification Errors

/// An error for a parent sending notifications to a child worker
#[derive(Debug, Error)]
pub enum NotificationSendError<CN> {
    #[error("Notification channel is closed, child worker likely terminated unexpectedly!")]
    ChannelClosed(CN),
    #[error("Notification channel is full, child worker is likely unresponsive!")]
    ChannelFull(CN),
}

/// An error for a child worker receiving notifications from their parent. Either the parent
/// failed or the channel is empty (this is not an error)
#[derive(Debug, Error)]
pub enum NotificationReceiveError<CN> {
    #[error("Notification channel is closed, parent likely terminated unexpectedly!")]
    ChannelClosed(CN),
    #[error("Notification channel is empty. This is not an error")]
    ChannelEmpty(CN),
}

//endregion

//region Notifications

/// Backend of all Feagi Worker Notifications. Establishes some universal commands but also
/// allows some custom ones
#[derive(Clone, Default)]
pub enum FeagiWorkerNotifications<CustomCommand: Send + Clone> {
    #[default]
    /// Do nothing, no effect. Mainly here to simplify pattern matching
    Nothing,
    /// Stop the worker
    Stop,
    /// Channel Closed, worker likely needs to be stopped
    ChannelClosed,
    /// Some specific command for the worker
    Custom(CustomCommand)
}

impl<CustomCommand: Send + Clone> From<Result<FeagiWorkerNotifications<CustomCommand>, thingbuf::mpsc::errors::TryRecvError>>
for FeagiWorkerNotifications<CustomCommand> {
    fn from(value: Result<FeagiWorkerNotifications<CustomCommand>, thingbuf::mpsc::errors::TryRecvError>) -> Self {
        match value {
            Ok(command) => {command}
            Err(err) => {
                match err {
                    thingbuf::mpsc::errors::TryRecvError::Empty => { FeagiWorkerNotifications::Nothing}
                    thingbuf::mpsc::errors::TryRecvError::Closed => { FeagiWorkerNotifications::ChannelClosed}
                    _ => { FeagiWorkerNotifications::ChannelClosed } // Not sure how this can happen?
                }
            }
        }
    }
}

//unsafe impl<CustomCommand: Send + Clone> Send for GenericThreadWorkerNotification<CustomCommand> {}

//endregion