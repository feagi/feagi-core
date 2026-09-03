use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::data_interface_set::DataInterfaceChannelSet;
use crate::enclosed_burst_engine::EnclosedBurstEngine;

// TODO dont do pub access
/// Structure grouping a burst engine and the channels of it in a single struct
pub struct BurstEnginePackage<FIQ, DICS>
where
    FIQ: FeagiIndexQuantization,
    DICS: DataInterfaceChannelSet<FIQ>
{
    pub engine: EnclosedBurstEngine<FIQ>,
    pub channels: DICS
}

impl<FIQ, DICS> BurstEnginePackage<FIQ, DICS>
where
    FIQ: FeagiIndexQuantization,
    DICS: DataInterfaceChannelSet<FIQ>
{
    /// Create from an existing engine, and with blank channel connections starting out
    fn new_from_engine(
        engine: EnclosedBurstEngine<FIQ>
    ) -> Self {
        Self {
            engine,
            channels: DICS::default()
        }
    }
}
