use core::marker::PhantomData;
use feagi_data::neurons::NeuronCorticalLocalIndex;
use feagi_data::quantization_levels::cortical_potential_quantization::CorticalPotentialCPUQuantization;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_models::neuron_models::neuron_model_traits::neuron_model_data::NeuronModelNeuronData;




/*

//region Cortical Local Neuron



/// Iterates with immutable references to neuron model data
pub trait CorticalNeuronIteratorRef<'a, FIQ, CPQ, Model>: Iterator<Item = IteratingCorticalNeuronRef<'a, FIQ, CPQ, Model>>
where
    FIQ: FeagiIndexQuantization,
    CPQ: CorticalPotentialCPUQuantization,
    Model: NeuronModelNeuronData<CPQ> + 'a,
{
}

/// Iterates with mutable references to neuron model data
pub trait CorticalNeuronIteratorRefMut<'a, FIQ, CPQ, Model>: Iterator<Item = IteratingCorticalNeuronRefMut<'a, FIQ, CPQ, Model>>
where
    FIQ: FeagiIndexQuantization,
    CPQ: CorticalPotentialCPUQuantization,
    Model: NeuronModelNeuronData<CPQ> + 'a,
{
}

/// Iterates with owned neuron model data
pub trait CorticalNeuronIterator<FIQ, CPQ, Model>: Iterator<Item = IteratingCorticalNeuron<FIQ, CPQ, Model>>
where
    FIQ: FeagiIndexQuantization,
    CPQ: CorticalPotentialCPUQuantization,
    Model: NeuronModelNeuronData<CPQ>,
{
}

pub struct IteratingCorticalNeuronRef<'a, FIQ, CPQ, Model>
where
    FIQ: FeagiIndexQuantization,
    CPQ: CorticalPotentialCPUQuantization,
    Model: NeuronModelNeuronData<CPQ>,
{
    pub local_index: &'a NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
    pub data: &'a Model,
    _p: PhantomData<CPQ>,
}

pub struct IteratingCorticalNeuronRefMut<'a, FIQ, CPQ, Model>
where
    FIQ: FeagiIndexQuantization,
    CPQ: CorticalPotentialCPUQuantization,
    Model: NeuronModelNeuronData<CPQ>,
{
    pub local_index: &'a NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
    pub data: &'a mut Model,
    _p: PhantomData<CPQ>,
}

pub struct IteratingCorticalNeuron<FIQ, CPQ, Model>
where
    FIQ: FeagiIndexQuantization,
    CPQ: CorticalPotentialCPUQuantization,
    Model: NeuronModelNeuronData<CPQ>,
{
    pub local_index: NeuronCorticalLocalIndex<FIQ::NeuronIndexCountQuant>,
    pub data: Model,
    _p: PhantomData<CPQ>,
}

//endregion

 */