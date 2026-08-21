use feagi_data::neurons::neuron::indexing::NeuronCount;
use crate::cortical_area::components::neuron_layout::neuron_layout::{NeuronLayoutConfigTrait, NeuronLayoutModelTrait};
use crate::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;

/// Defines any Cortical area that can be represented as voxels (3d with a 4th dimension of density).
pub struct VoxelNeuronLayoutModel<BEIQ>
where
    BEIQ: BurstEngineIndexQuantization,
{
    _p: core::marker::PhantomData<BEIQ>,
}

impl<BEIQ> NeuronLayoutModelTrait<BEIQ> for VoxelNeuronLayoutModel<BEIQ>
where
    BEIQ: BurstEngineIndexQuantization,
{
    
}

impl<BEIQ> VoxelNeuronLayoutModel<BEIQ>
where
    BEIQ: BurstEngineIndexQuantization,
{
    pub fn new(_p: core::marker::PhantomData<BEIQ>) -> Self {
        Self { _p }
    }
}

/// Defines any Cortical area that can be represented as voxels (3d with a 4th dimension of density).
pub struct VoxelNeuronLayoutConfig<BEIQ>
where
    BEIQ: BurstEngineIndexQuantization,
{
    _p: core::marker::PhantomData<BEIQ>,
    pub dimensions:  // TODO
}

impl<BEIQ> NeuronLayoutConfigTrait<BEIQ> for VoxelNeuronLayoutConfig<BEIQ>
where
    BEIQ: BurstEngineIndexQuantization,
{
    fn get_number_of_area_neurons(&self) -> NeuronCount<BEIQ::NeuronIndexQuant> {
        todo!()
    }
}

impl<BEIQ> VoxelNeuronLayoutConfig<BEIQ>
where
    BEIQ: BurstEngineIndexQuantization,
{
    pub fn new(_p: core::marker::PhantomData<BEIQ>) -> Self {
        Self { _p, dimensions: () }
    }
    
    // TODO funcs to convert dim to linear and vice versa
}