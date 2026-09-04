use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;


/// A vector of "notifications" of notable results from processing a phase of neuron dynamics.
/// An empty vector means nothing of note has occurred.
/// Note that the internal vector is heapless. Ergo only the number of notification variants possible
/// may be pushed to this struct
pub struct BurstPhaseOutput<Notification, const NOTIFICATION_VARIANTS_COUNT: usize>
where
    Notification: Sized + Send + Sync
{
    output: heapless::Vec<Notification, NOTIFICATION_VARIANTS_COUNT>,
}

impl<Notification, const NOTIFICATION_VARIANTS_COUNT: usize> BurstPhaseOutput<Notification, NOTIFICATION_VARIANTS_COUNT>
where
    Notification: Sized + Send + Sync
{
    
    /// Creates a new struct with no notifications.
    pub fn new_empty() -> Self {
        Self {
            output: heapless::Vec::new(),
        }
    }

    /// Returns true if there are no notifications
    pub fn is_empty(&self) -> bool {
        self.output.is_empty()
    }

    /// Adds a notification.
    pub fn add_notification(&mut self, notification: Notification) -> Result<(), ()> { // TODO Error!
        self.output.push(notification).map_err(
            |_| ()
        )
    }

    /// Returns an iterator that goes over all contained notifications. Calling this will empty the
    /// internal vector regardless of if the iterator is consumed or not
    pub fn drain_all(&mut self) -> heapless::vec::Drain<'_, Notification, usize> {
        self.output.drain(0..)
    }
}
