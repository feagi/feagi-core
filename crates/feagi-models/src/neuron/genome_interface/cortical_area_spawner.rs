//! Traits and a basic implementation for creating cortical areas per layout

use crate::neuron::neuron_model_data::{NeuronModelCorticalData, NeuronModelNeuronData};
use crate::neuron::neuron_model_quantization::NeuronModelQuantization;
use feagi_data::neurons::DimensionalCorticalArea4DDimensions;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

pub trait CorticalAreaSpawner {
    
}

/// Writes the data for a dimensional cortical area, both dimensional and neuron, to init a cortical
/// area of a given neuron model
pub trait DimensionalCorticalAreaSpawner {
    type NeuronModelQuantization: NeuronModelQuantization;
    type CorticalData: NeuronModelCorticalData<Self::NeuronModelQuantization>;
    type NeuronData: NeuronModelNeuronData<Self::NeuronModelQuantization>;

    /// Given a mutable slice of all neurons of a dimensional cortical area and the cortical area,
    /// write the data for this new cortical area. Note that if the area does not contain per
    /// neuron data, existing_neurons will be empty
    fn write_all_neuron_data<FIQ: FeagiIndexQuantization>(
        &self,
        existing_cortical: &mut Self::CorticalData,
        existing_neurons: &mut [Self::NeuronData],
        dimensions: &DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>,
    );
}

// TODO Formless

/// For any type of cortical area, writes neuron data in a uniform matter, which is a common default.
pub struct UniformCorticalAreaSpawner<NMQ, NMCD, NMND>
where
    NMQ: NeuronModelQuantization,
    NMCD: NeuronModelCorticalData<NMQ>,
    NMND: NeuronModelNeuronData<NMQ>,
{
    cortical_data: NMCD,
    uniform_neuron_data: NMND,
    _p: core::marker::PhantomData<NMQ>,
}

impl<NMQ, NMCD, NMND> UniformCorticalAreaSpawner<NMQ, NMCD, NMND>
where
    NMQ: NeuronModelQuantization,
    NMCD: NeuronModelCorticalData<NMQ>,
    NMND: NeuronModelNeuronData<NMQ>,
{
    pub fn new(cortical_data: NMCD, uniform_neuron_data: NMND) -> Self {
        Self {
            cortical_data,
            uniform_neuron_data,
            _p: core::marker::PhantomData,
        }
    }
}

impl<NMQ, NMCD, NMND> DimensionalCorticalAreaSpawner for UniformCorticalAreaSpawner<NMQ, NMCD, NMND>
where
    NMQ: NeuronModelQuantization,
    NMCD: NeuronModelCorticalData<NMQ>,
    NMND: NeuronModelNeuronData<NMQ>,
{
    type NeuronModelQuantization = NMQ;
    type CorticalData = NMCD;
    type NeuronData = NMND;

    fn write_all_neuron_data<FIQ: FeagiIndexQuantization>(
        &self,
        existing_cortical: &mut Self::CorticalData,
        existing_neurons: &mut [Self::NeuronData],
        _dimensions: &DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>,
    ) {
        *existing_cortical = self.cortical_data.clone();
        existing_neurons.iter_mut().for_each(|existing| {
            *existing = self.uniform_neuron_data.clone();
        })
    }
}

// TODO also implement for formless
