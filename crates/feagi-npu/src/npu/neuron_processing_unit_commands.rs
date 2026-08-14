
/// Number of bursts per second
pub type BurstFrequency = f64;

/// Instructs the NPU how to handle its burst engine(s)
pub enum NeuronProcessingUnitCommands {
    /// Starts the burst engine to run at a given frequency, or updates the currently running freq
    RunAtFrequency(BurstFrequency),
    /// Stop the burst engines from running
    StopNPU,
    /// Send in a request to edit the connectome // TODO this should be feature gated
    EditConnectome(),
}


