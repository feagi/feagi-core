use crate::common_systems::neurogenesis_generator_uniform_defaults::NeurogenesisGeneratorUniformDefault;
use crate::connectome_basis::neuron_model::feagi_simple::{NMFeagiSimpleDataCorticalInternal, NMFeagiSimpleDataCorticalProperties, NMFeagiSimpleDataCorticalScratch, NMFeagiSimpleDataCorticalVariance, NMFeagiSimpleDataNeuronInternal, NMFeagiSimpleDataNeuronScratch, NMFeagiSimpleDataNeuronVariance, NMFeagiSimpleTrait};
use crate::connectome_basis::neuron_model::{NeuronLayoutTagVoxel, NeuronModel};
use crate::connectome_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiSimpleQuantDefaultQuant;

impl NMFeagiSimpleTrait for NMFeagiSimpleQuantDefaultQuant {
    type Period = f32;
}

impl NeuronModel for NMFeagiSimpleQuantDefaultQuant {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::FeagiSimpleNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisByGeneratorType<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMFeagiSimpleDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFeagiSimpleDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFeagiSimpleDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFeagiSimpleDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFeagiSimpleDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFeagiSimpleDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFeagiSimpleDataNeuronVariance<Self>;
}

impl MembranePotentialQuantization for NMFeagiSimpleQuantDefaultQuant
{
    type MembranePotentialQuant = f32;
}