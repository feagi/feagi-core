use crate::neuron::shared::data::NeuronModelNeuronData;
use feagi_data::neurons::{DimensionalCorticalArea4DDimensions, NeuronCorticalLocalIndex};
use feagi_data::feagi_quantization_levels::cortical_potential_quantization::CorticalPotentialQuantization;
use feagi_data::feagi_quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use std::marker::PhantomData;

/// Writes neuron data within a dimensional cortical area
pub trait DimensionalCorticalNeuronWriter<FIQ: FeagiIndexQuantization> {
    type CorticalModelAndQuant: CorticalPotentialQuantization;
    type NeuronData: NeuronModelNeuronData<Self::CorticalModelAndQuant>;

    /// This function will be called using some `DimensionalCorticalNeuronIterator` to modify
    /// a given set of neurons from a cortical area
    fn write_dimensional_neuron_data(
        &self,
        existing_neuron: &mut Self::NeuronData,
        index: NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
        cortical_dimensions: &DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>,
    );
}

/// Writes neuron data within a memory cortical area
pub trait MemoryCorticalNeuronWriter<FIQ: FeagiIndexQuantization> {
    type CorticalModelAndQuant: CorticalPotentialQuantization;
    type NeuronData: NeuronModelNeuronData<Self::CorticalModelAndQuant>;

    /// This function will be called using some `MemoryCorticalNeuronIterator` to modify
    /// a given set of neurons from a cortical area
    fn write_memory_neuron_data(
        &self,
        existing_neuron: &mut Self::NeuronData,
        index: NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
        total_neuron_count: &NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
    );
}

/// Overwrites every neuron with a given copy of data
pub struct UniformReplacingNeuronWriter<FIQ: FeagiIndexQuantization, CPQ: CorticalPotentialQuantization, NMND: NeuronModelNeuronData<CPQ>> {
    uniform_neuron: NMND,
    _p: PhantomData<(FIQ, CPQ, NMND)>,
}

impl<FIQ: FeagiIndexQuantization, CPQ: CorticalPotentialQuantization, NMND: NeuronModelNeuronData<CPQ>> DimensionalCorticalNeuronWriter<FIQ>
for UniformReplacingNeuronWriter<FIQ, CPQ, NMND>
{
    type CorticalModelAndQuant = CPQ;
    type NeuronData = NMND;

    fn write_dimensional_neuron_data(
        &self,
        existing_neuron: &mut Self::NeuronData,
        _index: NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
        _cortical_dimensions: &DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>,
    ) {
        *existing_neuron = self.uniform_neuron.clone();
    }
}

impl<FIQ: FeagiIndexQuantization, CPQ: CorticalPotentialQuantization, NMND: NeuronModelNeuronData<CPQ>> MemoryCorticalNeuronWriter<FIQ>
for UniformReplacingNeuronWriter<FIQ, CPQ, NMND>
{
    type CorticalModelAndQuant = CPQ;
    type NeuronData = NMND;

    fn write_memory_neuron_data(
        &self,
        existing_neuron: &mut Self::NeuronData,
        _index: NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
        _total_neuron_count: &NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
    ) {
        *existing_neuron = self.uniform_neuron.clone();
    }
}
