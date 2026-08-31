use feagi_data::data_channels::implementations::flume::FlumeDataCycleEndpoint;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::{BurstEngineEnum, BurstEnginePackage};

pub struct FlumeBurstEnginePackage<
    FIQ: FeagiIndexQuantization,
>
{
    pub burst_engine: BurstEngineEnum<FIQ>,
    pub agent_interfaces: Vec<<FlumeBurstEnginePackage<FIQ> as BurstEnginePackage<FIQ>>::CycleEndpoint>,
}

impl<FIQ: FeagiIndexQuantization> BurstEnginePackage<FIQ> for FlumeBurstEnginePackage<FIQ> {
    type CycleEndpoint = FlumeDataCycleEndpoint<()>;

    fn new_from_engine(engine: BurstEngineEnum<FIQ>, existing_interfaces: Vec<<FlumeBurstEnginePackage<FIQ> as BurstEnginePackage<FIQ>>::CycleEndpoint>) -> Self {
        Self {
            burst_engine: engine,
            agent_interfaces: existing_interfaces
        }
    }

    fn get_existing_interfaces(&self) -> &[Self::CycleEndpoint] {
        self.agent_interfaces.as_slice()
    }

    fn get_existing_interfaces_mut(&mut self) -> &mut Vec<Self::CycleEndpoint> {
        &mut self.agent_interfaces
    }

    fn get_engine(&self) -> &BurstEngineEnum<FIQ> {
        &self.burst_engine
    }

    fn get_engine_mut(&mut self) -> &mut BurstEngineEnum<FIQ> {
        &mut self.burst_engine
    }
}


