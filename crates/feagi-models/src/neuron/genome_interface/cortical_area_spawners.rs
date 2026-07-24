//! Traits and a basic implementation for creating cortical areas per layout

use crate::neuron::neuron_model_quantization::NeuronModelQuantization;
use feagi_data::neurons::DimensionalCorticalArea4DDimensions;
use feagi_data::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use crate::neuron::model_extensions::neuron_layout_implementations::DimensionalNeuronModel;
use crate::neuron::neuron_model::NeuronModel;

//region Dimensional

/// Writes the data for a dimensional cortical area, both dimensional and neuron, to init a cortical
/// area of a given neuron model
pub trait DimensionalCorticalAreaSpawner<FIQ, NMQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NMQ: NeuronModelQuantization,
    NM: DimensionalNeuronModel<FIQ, NMQ>,
{

    /// Given a mutable slice of all neurons of a dimensional cortical area and the cortical area,
    /// write the data for this new cortical area. Note that if the area does not contain per
    /// neuron data, existing_neurons will be empty
    fn write_all_neuron_data(
        &self,
        existing_cortical: &mut NM::CorticalData,
        existing_neurons: &mut [NM::NeuronData],
        dimensions: &DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>,
    );
}

/// For any type of cortical area, writes neuron data in a uniform matter, which is a common default.
pub struct UniformDimensionalCorticalAreaSpawner<FIQ, NMQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NMQ: NeuronModelQuantization,
    NM: NeuronModel<FIQ, NMQ>,
{
    cortical_data: NM::CorticalData,
    uniform_neuron_data: NM::NeuronData,
    _p: core::marker::PhantomData<NMQ>,
}

impl<FIQ, NMQ, NM> UniformDimensionalCorticalAreaSpawner<FIQ, NMQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NMQ: NeuronModelQuantization,
    NM: NeuronModel<FIQ, NMQ>,
{
    pub fn new(cortical_data: NM::CorticalData, uniform_neuron_data: NM::NeuronData) -> Self {
        Self {
            cortical_data,
            uniform_neuron_data,
            _p: core::marker::PhantomData,
        }
    }
}

impl<FIQ, NMQ, DNM> DimensionalCorticalAreaSpawner<FIQ, NMQ, DNM> for UniformDimensionalCorticalAreaSpawner<FIQ, NMQ, DNM>
where
    FIQ: FeagiIndexQuantization,
    NMQ: NeuronModelQuantization,
    DNM: DimensionalNeuronModel<FIQ, NMQ>,
{

    fn write_all_neuron_data(
        &self,
        existing_cortical: &mut DNM::CorticalData,
        existing_neurons: &mut [DNM::NeuronData],
        _dimensions: &DimensionalCorticalArea4DDimensions<FIQ::NeuronIndexCountQuant>,
    ) {
        *existing_cortical = self.cortical_data.clone();
        existing_neurons.iter_mut().for_each(|existing| {
            *existing = self.uniform_neuron_data.clone();
        })
    }
}

//endregion

// TODO Formless


// TODO also implement for formless
