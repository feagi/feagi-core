use crate::neuron_models::shared_traits_and_structs::base_traits_all_devices::NeuronModelData;
use crate::neuron_models::shared_traits_and_structs::base_traits_cpu::CorticalModelDataCPU;
use crate::neuron_models::shared_traits_and_structs::cortical_configuration::CorticalConfigurationDimensionalCPU;
use feagi_structures::feagi_data::feagi_bitpacking::bitpacking_backends::{BitPacked, BitPackedU32};
use feagi_structures::feagi_data::feagi_bitpacking::collections::contiguous_vector::BitPackedContiguousBoolVector;
use feagi_structures::feagi_data::quantizable_spatial::index::SpatialIndexDimensions4D;
use feagi_structures::feagi_data::shared_quantization_sets::{FeagiGlobalQuantization, NeuronModelQuantization};




/// Contains Vectorized Neuron Data with cortical contexts, written for CPU Rayon implementations
pub(crate) struct NPUDimensionalCorticalAreaCPURayon<FGQ, NMQ, CMD, NMD>
where FGQ: FeagiGlobalQuantization,
      NMQ: NeuronModelQuantization,
      CMD: CorticalModelDataCPU<FGQ, NMQ>,
      NMD: NeuronModelData<FGQ, NMQ>
{
    pub cortical_configuration: CorticalConfigurationDimensionalCPU<FGQ>,

    pub cortical_area_global_index_offset: FGQ::CorticalAreaIndexCountQuant, // how much greater the global offset is than the local
    pub neuron_global_index_offset: FGQ::CorticalAreaIndexCountQuant, // how much greater the global offset is than the local
    pub neuron_model_index_offset: FGQ::NeuronIndexCountQuant, // how much greater the model offset is than the local neuron model offset


    pub neuron_input: Vec<NMQ::NeuronPotentialQuant>,
    pub neuron_potentials: Vec<NMQ::NeuronPotentialQuant>,

    pub cortical_model_data: CMD,
    pub neuron_model_data: Vec<NMD>,

    pub neuron_is_firing: BitPackedContiguousBoolVector<BitPackedU32>, // TODO make a selector
    pub number_firing_neurons_in_current_burst: FGQ::NeuronIndexCountQuant,
}

impl<FGQ, NMQ, CMD, NMD> NPUDimensionalCorticalAreaCPURayon<FGQ, NMQ, CMD, NMD>
where FGQ: FeagiGlobalQuantization,
      NMQ: NeuronModelQuantization,
      CMD: CorticalModelDataCPU<FGQ, NMQ>,
      NMD: NeuronModelData<FGQ, NMQ>
{
    pub fn new(
        cortical_configuration: CorticalConfigurationDimensionalCPU<FGQ::NeuronIndexCountQuant>,
        cortical_model_data: CMD,
        cortical_area_global_index_offset: FGQ::CorticalAreaIndexCountQuant,
        neuron_global_index_offset: FGQ::NeuronIndexCountQuant,
        neuron_model_index_offset: FGQ::NeuronIndexCountQuant,
    ) -> Self {
        let number_neurons = cortical_configuration.dimensions.max_linear_index();

        Self {
            cortical_configuration,
            cortical_area_global_index_offset,
            neuron_global_index_offset,
            neuron_model_index_offset,
            neuron_input: vec![number_neurons],
            neuron_potentials: vec![number_neurons],
            cortical_model_data,
            neuron_model_data: vec![], // TODO
            neuron_is_firing: BitPackedContiguousBoolVector::new(number_neurons),
            number_firing_neurons_in_current_burst: 0,
        }

    }

    pub fn get_number_neurons(&self) -> FGQ::NeuronIndexCountQuant
    {
        self.cortical_configuration.dimensions.max_linear_index()
    }

    pub fn get_neuron_firing_flag_word_size(&self) -> usize {
        BitPackedU32::NUMBER_BITS // TODO make dynamic
    }

    pub fn get_number_neuron_words(&self) -> usize {
        if self.neuron_model_data.get_number_neurons() as usize % self.get_neuron_firing_flag_word_size() == 0 {
            return (self.neuron_model_data.get_number_neurons() as usize) / self.get_neuron_firing_flag_word_size()
        }
        ((self.neuron_model_data.get_number_neurons() as usize) / self.get_neuron_firing_flag_word_size()) + 1
    }



    pub fn process_neuron_dynamics_for_word(
        &mut self,
        neuron_word_index: usize
    ) {
        let mut neuron_index: usize = neuron_word_index * self.get_neuron_firing_flag_word_size();
        let mut bit_index: usize = 0;
        let neuron_model_slice = self.neuron_is_firing.make_range_from_word_index(neuron_word_index, self.get_number_neurons());
        let neuron_model

    }


}