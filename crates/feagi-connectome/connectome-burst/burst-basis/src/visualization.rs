use connectome_basis::quantization_levels::FeagiIndexQuantization;
use connectome_basis::visualization::BurstVisualizationData;
use feagi_basis::parallelism::feagi_channel::base_channels::base_channels::{FeagiChannelBuilder, FeagiChannelTransmitter};
use feagi_basis::parallelism::feagi_channel::base_channels::implementations::alloc_channels::FeagiChannelAllocBuilder;

/// Used by a burst engine to indicate visualization support
pub trait BurstEngineVisualizationSupport: Send {
    /// True if visualization is supported
    const VISUALIZATION_SUPPORTED: bool;

    /// Define the transmitter to use (or none if no visualization)
    type VisualizationTransmitter;
}

/// No visualization support possible on this burst engine
pub struct BurstEngineVisualizationNone;

impl BurstEngineVisualizationSupport for BurstEngineVisualizationNone {
    const VISUALIZATION_SUPPORTED: bool = false;
    type VisualizationTransmitter = ();
}


/// Full visualization features are supported by this burst engine
#[derive(Default)]
pub struct BurstEngineVisualizationFull<FIQ, Tx>
where
    FIQ: FeagiIndexQuantization,
    Tx: FeagiChannelTransmitter<BurstVisualizationData<FIQ>>
{
    _p: core::marker::PhantomData<(FIQ, Tx)>
}

unsafe impl <FIQ, Tx> Send for BurstEngineVisualizationFull<FIQ, Tx>
where
    FIQ: FeagiIndexQuantization,
    Tx: FeagiChannelTransmitter<BurstVisualizationData<FIQ>> {}


impl<FIQ, Tx> BurstEngineVisualizationSupport for BurstEngineVisualizationFull<FIQ, Tx>
where
    FIQ: FeagiIndexQuantization,
    Tx: FeagiChannelTransmitter<BurstVisualizationData<FIQ>>
{
    const VISUALIZATION_SUPPORTED: bool = true;
    type VisualizationTransmitter = Tx;
}

#[cfg(feature = "alloc")]
/// Sending half of the visualization channel, owned by the burst engine
pub type AllocVisualizationTransmitter<FIQ> =
<FeagiChannelAllocBuilder<BurstVisualizationData<FIQ>> as FeagiChannelBuilder<BurstVisualizationData<FIQ>>>::Transmitter;

#[cfg(feature = "alloc")]
/// Receiving half of the visualization channel, for whatever listens to the burst engine's
/// visualization
pub type AllocVisualizationReceiver<FIQ> =
<FeagiChannelAllocBuilder<BurstVisualizationData<FIQ>> as FeagiChannelBuilder<BurstVisualizationData<FIQ>>>::Receiver;


// NOTE: Actual Visualization Data lives on `connectome-basis` intentionally! This is because
// clients need to be able to parse it and thus need access to it, and we don't want to require
// clients to import all of the connectome burst system in order to access it!
