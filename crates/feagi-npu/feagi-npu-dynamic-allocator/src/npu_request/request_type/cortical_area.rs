
use feagi_genomic::feagi_genomic_context::cortical_area::{CorticalAreaType, CorticalID};
use feagi_models::neuron::genome_interface::cortical_area_spawner::DimensionalCorticalAreaSpawner;
use crate::npu_request::npu_request::{ConnectomeRequest, NPURequestConnectomeConsequences,};
use crate::npu_state_manager::burst_engine_context::burst_engine_context::BurstEngineIndex;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum NPUCorticalAreaRequestType {
    AddArea,
    DuplicateArea,
    EditAreaCorticalData,
    EditAreaNeuronData,
    ChangeNeuronModel,
    ResizeArea,
    DeleteCorticalArea,
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct NPURequestAddCorticalArea
{
    // TODO we should generate the cortical ID, not be given it!
    cortical_id: CorticalID,
    cortical_area_class: CorticalAreaType, // TODO this enum sucks. replace it
    cortical_area_generator: (), // TODO
    specific_burst_engine_index: Option<BurstEngineIndex>,
}

impl NPURequestAddCorticalArea
{
    // TODO should this even be an option?
    pub fn new_core_area() -> Self {
        todo!()
    }

    pub fn new_interneuron_area(
        cortical_id: CorticalID,
        dimensional_area_spawner: Box<dyn DimensionalCorticalAreaSpawner>
    
    ) -> Self {
        todo!()
    }

    pub fn new_sensor_area() -> Self {
        todo!()
    }

    pub fn new_motor_area() -> Self {
        todo!()
    }

    pub fn new_memory_area() -> Self {
        todo!()
    }
}

impl ConnectomeRequest for NPURequestAddCorticalArea {

    fn get_connectome_consequences() -> NPURequestConnectomeConsequences {
        todo!()
    }
}