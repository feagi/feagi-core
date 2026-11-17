use connectome_basis::visualization::BurstVisualizationData;
use feagi_basis::parallelism::feagi_channel::base_channels::base_channels::FeagiChannelBuilder;
use feagi_basis::parallelism::feagi_channel::base_channels::implementations::alloc_channels::FeagiChannelAllocBuilder;

pub mod parallel_burst_engines;
pub mod placebo_engine;
pub mod shared_structs;

/// Sending half of the visualization channel, owned by the burst engine
pub type NeuronProcessorSingleEngineVisualizationTransmitter<FIQ> =
<FeagiChannelAllocBuilder<BurstVisualizationData<FIQ>> as FeagiChannelBuilder<BurstVisualizationData<FIQ>>>::Transmitter;

/// Receiving half of the visualization channel, for whatever listens to the burst engine's
/// visualization
pub type NeuronProcessorSingleEngineVisualizationReceiver<FIQ> =
<FeagiChannelAllocBuilder<BurstVisualizationData<FIQ>> as FeagiChannelBuilder<BurstVisualizationData<FIQ>>>::Receiver;