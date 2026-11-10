use crate::common_systems::neurogenesis_generator_uniform_defaults::NeurogenesisGeneratorUniformDefault;
use crate::connectome_basis::neuron_model::memory_fire_hashing::{NMMemoryFireHashingDataCorticalInternal, NMMemoryFireHashingDataCorticalProperties, NMMemoryFireHashingDataCorticalScratch, NMMemoryFireHashingDataCorticalVariance, NMMemoryFireHashingDataNeuronInternal, NMMemoryFireHashingDataNeuronScratch, NMMemoryFireHashingDataNeuronVariance, NMMemoryFireHashingTrait};
use crate::connectome_basis::neuron_model::{NeuronLayoutTagSingleSteppedDynamic, NeuronModel};
use crate::connectome_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMMemoryFireHashingQuantDefaultQuant;

impl NMMemoryFireHashingTrait for NMMemoryFireHashingQuantDefaultQuant {
    type Period = f32;
}

impl NeuronModel for NMMemoryFireHashingQuantDefaultQuant {
    type NeuronLayoutTag = NeuronLayoutTagSingleSteppedDynamic;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::MemoryFireHashingNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisByGeneratorType<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMMemoryFireHashingDataCorticalProperties<Self>;
    type CorticalDataInternal = NMMemoryFireHashingDataCorticalInternal<Self>;
    type CorticalDataScratch = NMMemoryFireHashingDataCorticalScratch<Self>;
    type CorticalDataVariance = NMMemoryFireHashingDataCorticalVariance<Self>;
    type NeuronDataInternal = NMMemoryFireHashingDataNeuronInternal<Self>;
    type NeuronDataScratch = NMMemoryFireHashingDataNeuronScratch<Self>;
    type NeuronDataVariance = NMMemoryFireHashingDataNeuronVariance<Self>;
}

impl MembranePotentialQuantization for NMMemoryFireHashingQuantDefaultQuant
{
    type MembranePotentialQuant = f32;
}