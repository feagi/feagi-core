use connectome_basis::quantization_levels::FeagiIndexQuantization;
use feagi_basis::target_frequency::TargetFrequency;

/// Custom requests to the single engine neuron processor
pub enum NeuronProcessorSingleEngineRequest<FIQ: FeagiIndexQuantization> {
    /// Set the rate the burst loop ticks at. Applies from the next tick
    BurstRate(TargetFrequency),
    /// Set the rate bursts send visualization at. Bursts never run faster than the burst rate, so
    /// a visualization rate above it means every burst sends visualization
    VisualizationRate(TargetFrequency),
    /// Apply these connectome edit steps in order. Stops at the first step that fails. Pauses the
    /// worker, which stays paused afterward, even if a step failed, until sent `Resume`
    ConnectomeEdits(Vec<ConnectomeEditStepEnum<FIQ>>),
    /// Stop bursting until resumed. While paused the worker only waits for and handles commands,
    /// so its caller can edit the connectome and update its own state before bursting continues
    Pause,
    /// Continue bursting after a pause
    Resume,
}