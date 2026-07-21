use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use crate::neuron::common_structs::model_and_quantization::NestedNeuronModelTypeAndQuantization;
use crate::neuron::models_shared_traits::data::{NeuronModelCorticalData, NeuronModelNeuronData};


pub trait CorticalAreaInitializer {

    const QUANT_AND_MODEL: NestedNeuronModelTypeAndQuantization;

    type ModelQuantization: CorticalPotentialQuantization;
    type CorticalData: NeuronModelCorticalData<Self::ModelQuantization>;
    type NeuronData: NeuronModelNeuronData<Self::ModelQuantization>;

    /// How many neurons the cortical area needs
    fn number_neurons_needed(&self) -> usize;

    fn initialize_cortical_area(&self, cortical_data: &mut Self::CorticalData, neuron_data: &mut [Self::NeuronData]);
}

/// Initialize a cortical area
pub struct UniformCorticalAreaInitializer<CPQ: CorticalPotentialQuantization, NMCD: NeuronModelCorticalData<CPQ>, NMND: NeuronModelNeuronData<CPQ>> {
    cortical_data: NMCD,
    uniform_neuron_data: NMND,
    _p: core::marker::PhantomData<CPQ>
}

impl<CPQ: CorticalPotentialQuantization, NMCD: NeuronModelCorticalData<CPQ>, NMND: NeuronModelNeuronData<CPQ>> UniformCorticalAreaInitializer<CPQ, NMCD, NMND> {
    pub fn new(cortical_data: NMCD, uniform_neuron_data: NMND) -> Self {
        Self {
            cortical_data,
            uniform_neuron_data,
            _p: core::marker::PhantomData
        }
    }
}

impl<CPQ: CorticalPotentialQuantization, NMCD: NeuronModelCorticalData<CPQ>, NMND: NeuronModelNeuronData<CPQ>> CorticalAreaInitializer for UniformCorticalAreaInitializer<CPQ, NMCD, NMND> {
    const QUANT_AND_MODEL: NestedNeuronModelTypeAndQuantization = ();
    type ModelQuantization = CPQ;
    type CorticalData = NMCD;
    type NeuronData = NMND;

    fn number_neurons_needed(&self) -> usize {
        todo!()
    }

    fn initialize_cortical_area(&self, cortical_data: &mut Self::CorticalData, neuron_data: &mut [Self::NeuronData]) {
        todo!()
    }
}

