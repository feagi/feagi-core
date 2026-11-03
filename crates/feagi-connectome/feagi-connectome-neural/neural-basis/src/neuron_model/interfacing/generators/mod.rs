use crate::neuron_model::interfacing::generators::cortical::CorticalAreaGeneratorCortical;
use crate::neuron_model::interfacing::generators::neuron::CorticalAreaGeneratorNeuron;
use crate::neuron_model::neuron_model::NeuronModel;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;

pub mod cortical;
pub mod neuron;

pub use cortical_area_generator_from_stored::CorticalAreaGeneratorUniform;
pub use cortical_area_generator_from_defaults::{CorticalAreaGeneratorFromDefaults, CorticalModelDefaultsSpawnData};
mod cortical_area_generator_from_stored;
mod cortical_area_generator_from_defaults;

/// Something that wants a generator with its concrete types
pub trait CorticalAreaGeneratorVisitor<FIQ: FeagiIndexQuantization> {
    type Output;
    fn visit<NM: NeuronModel, CAG: CorticalAreaGenerator<FIQ, NM>>(self, generator: CAG) -> Self::Output;
}

/// Something that holds a generator and can give it, with its real types, to a visitor
pub trait CorticalAreaGeneratorDispatch<FIQ: FeagiIndexQuantization>: Send {
    fn dispatch<V: CorticalAreaGeneratorVisitor<FIQ>>(self, visitor: V) -> V::Output;
}

/// Handles generating a cortical area
pub trait CorticalAreaGenerator<FIQ, NM>: Send
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel
{
    type CorticalGenerator: CorticalAreaGeneratorCortical<FIQ, NM>;
    type NeuronGenerator: CorticalAreaGeneratorNeuron<FIQ, NM>;

    fn get_cortical_generator(&self) -> &Self::CorticalGenerator;
    fn get_neuron_generator(&self) -> &Self::NeuronGenerator;
}