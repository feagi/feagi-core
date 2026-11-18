use burst_basis::connectome_editing::{ConnectomeEditEnum, ConnectomeEditError, ConnectomeEditResultantEnum};
use burst_basis::multi_engine::{EngineIndexingType};
use connectome_basis::quantization_levels::FeagiIndexQuantization;
use feagi_basis::target_frequency::TargetFrequency;



///  (Custom) Commands for a NPU
pub enum NeuralProcessingUnitRequest<FIQ: FeagiIndexQuantization, BEI: EngineIndexingType> {
    /// Set the rate the burst loop ticks at. Applies from the next tick
    BurstRate(TargetFrequency),
    /// Set the rate bursts send visualization at. Bursts never run faster than the burst rate, so
    /// a visualization rate above it means every burst sends visualization
    VisualizationRate(TargetFrequency),
    /// Stop bursting until resumed. While paused the worker only waits for and handles commands,
    /// so its caller can edit the connectome and update its own state before bursting continues
    Pause,
    /// Continue bursting after a pause
    Resume,
    /// Sends connectome edit a specific burst engine
    ConnectomeEdit(BEI, ConnectomeEditEnum<FIQ>),
    #[cfg(feature = "alloc")]
    /// Sends connectome edits to burst engines in a batch for faster processing
    ConnectomeEdits(Vec<(BEI, ConnectomeEditEnum<FIQ>)>)
}

/// (Custom) Responses for a NPU
pub enum NeuralProcessingUnitResultant<FIQ: FeagiIndexQuantization, BEI: EngineIndexingType> {
    /// The burst rate was set
    BurstRate,
    /// The visualization rate was set
    VisualizationRate,
    /// Bursting is paused. Also the reply if it already was paused
    Paused,
    /// Bursting has resumed. Also the reply if it was already resumed
    Resumed,
    /// What a single connectome edit returned
    ConnectomeEdit(BEI, Result<ConnectomeEditResultantEnum<FIQ>, ConnectomeEditError<FIQ>>),
    #[cfg(feature = "alloc")]
    /// What a batch of connectome edits returned. If any of them error, the batch will stop. 
    /// completed steps will and the errored step will return
    ConnectomeEdits(Result<Vec<ConnectomeEditResultantEnum<FIQ>>, ConnectomeEditError<FIQ>>), // TODO the offending burst engine index must return too!

}












