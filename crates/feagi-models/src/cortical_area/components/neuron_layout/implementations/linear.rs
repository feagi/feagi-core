use feagi_data::neurons::neuron::indexing::NeuronCount;
use crate::cortical_area::components::neuron_layout::neuron_layout::{NeuronLayoutConfigTrait, NeuronLayoutModelTrait};
use crate::quantization_levels::burst_engine_index_quantization::BurstEngineIndexQuantization;

/// Defines any Cortical Area with neurons whos position can only be described by a linear index
pub struct LinearNeuronLayoutModel<BEIQ>
where
    BEIQ: BurstEngineIndexQuantization,
{
    _p: core::marker::PhantomData<BEIQ>,
}

impl<BEIQ> NeuronLayoutModelTrait<BEIQ> for LinearNeuronLayoutModel<BEIQ>
where
    BEIQ: BurstEngineIndexQuantization,
{

}

impl<BEIQ> LinearNeuronLayoutModel<BEIQ>
where
    BEIQ: BurstEngineIndexQuantization,
{
    pub fn new(_p: core::marker::PhantomData<BEIQ>) -> Self {
        Self { _p }
    }
}

/// Defines any Cortical Area with neurons whos position can only be described by a linear index
pub struct LinearNeuronLayoutConfig<BEIQ>
where
    BEIQ: BurstEngineIndexQuantization,
{
    pub neuron_count: NeuronCount<BEIQ::NeuronIndexQuant>
}

impl<BEIQ> NeuronLayoutConfigTrait<BEIQ> for LinearNeuronLayoutConfig<BEIQ>
where
    BEIQ: BurstEngineIndexQuantization,
{
    fn get_number_of_area_neurons(&self) -> NeuronCount<BEIQ::NeuronIndexQuant> {
        self.neuron_count
    }
}

impl<BEIQ> LinearNeuronLayoutConfig<BEIQ>
where
    BEIQ: BurstEngineIndexQuantization,
{
    pub fn new(neuron_count: NeuronCount<BEIQ::NeuronIndexQuant>) -> Self {
        Self { neuron_count }
    }
}