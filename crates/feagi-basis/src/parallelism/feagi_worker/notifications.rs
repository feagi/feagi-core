//! Notifications a worker handle sends to its worker, and the worker's replies.

use crate::parallelism::feagi_channel::recycling::Recyclable;

/// A notification from a worker handle to its worker. Establishes universal commands but also
/// allows custom ones
pub enum FeagiWorkerNotification<CustomNotification> {
    /// Do nothing, no effect. Also the value of an unused channel slot
    Nothing,
    /// Stop the worker. The worker replies before it exits
    Stop,
    /// Some specific command for the worker
    Custom(CustomNotification),
}

impl<CustomNotification> Recyclable for FeagiWorkerNotification<CustomNotification> {
    fn new_for_recycler() -> Self {
        FeagiWorkerNotification::Nothing
    }

    fn recycle_self(&mut self) {
        *self = FeagiWorkerNotification::Nothing;
    }
}

/// A worker's reply to a notification
pub enum FeagiWorkerReply<CustomReply> {
    /// No reply content. Also the value of an unused channel slot
    Nothing,
    /// The notification was received and acted upon
    Acknowledged,
    /// Reply to a custom notification
    Custom(CustomReply),
}

impl<CustomReply> Recyclable for FeagiWorkerReply<CustomReply> {
    fn new_for_recycler() -> Self {
        FeagiWorkerReply::Nothing
    }

    fn recycle_self(&mut self) {
        *self = FeagiWorkerReply::Nothing;
    }
}
