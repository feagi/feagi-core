use feagi_data::data_channels::data_cycler::DataCycleEndpoint;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::BurstEngineEnum;

/// Represents the burst engine and the interfaces needed to interact with it
pub trait BurstEnginePackage<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    type CycleEndpoint: DataCycleEndpoint<()>; // TODO data // TODO force fire / probe functions, should take some sort of iterator?

    fn new_from_engine(
        engine: BurstEngineEnum<FIQ>,
        existing_agent_interfaces: Vec<Self::CycleEndpoint>,
    ) -> Self;

    fn get_existing_interfaces(&self) 
        -> &[Self::CycleEndpoint];

    fn get_existing_interfaces_mut(&mut self) 
        -> &mut Vec<Self::CycleEndpoint>;

    fn get_engine(&self) -> &BurstEngineEnum<FIQ>;
    
    fn get_engine_mut(&mut self) -> &mut BurstEngineEnum<FIQ>;
    
    fn add_agent_interface(&mut self, agent_requirements: (), buffer_size: usize) -> (usize, Self::CycleEndpoint) {
        let index = self.get_existing_interfaces().len();
        let (inner, outer) = Self::CycleEndpoint::new_data_cycle_endpoint_pair(buffer_size);
        self.get_existing_interfaces_mut().push(inner);
        // TODO inject first struct into A
        (index, outer)
    }
    
    fn remove_agent_interface(&mut self, agent_index: usize) -> Option<Self::CycleEndpoint> {
        Some(self.get_existing_interfaces_mut().swap_remove(agent_index))
    }
}

