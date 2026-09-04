use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::data_interface_set::DataInterfaceChannelSet;

// TODO dont do pub access of members
/// Structure grouping a burst engine and the channels of it in a single struct
pub struct BurstEnginePackage<FIQ, Engine, DataInterface>
where
    FIQ: FeagiIndexQuantization,
    Engine: EnclosedEngine<FIQ>,
    DataInterface: DataInterfaceChannelSet<FIQ>
{
    pub engine: Engine,
    pub channels: DataInterface,
    _p: core::marker::PhantomData<FIQ>
}

impl<FIQ, Engine, DataInterface> BurstEnginePackage<FIQ, Engine, DataInterface>
where
    FIQ: FeagiIndexQuantization,
    Engine: EnclosedEngine<FIQ>,
    DataInterface: DataInterfaceChannelSet<FIQ>
{
    /// Create from an existing engine, and with blank channel connections starting out
    pub(crate) fn new_from_engine(
        engine: Engine
    ) -> Self {
        Self {
            engine,
            channels: DataInterface::default(),
            _p: core::marker::PhantomData
        }
    }
}


pub trait EnclosedEngine<FIQ> {}
