use crate::common_systems::neurogenesis_generator_uniform_defaults::NeurogenesisGeneratorUniformDefault;
use crate::connectome_basis::neuron_model::feagi_simple::{NMFeagiSimpleDataCorticalInternal, NMFeagiSimpleDataCorticalProperties, NMFeagiSimpleDataCorticalScratch, NMFeagiSimpleDataCorticalVariance, NMFeagiSimpleDataNeuronInternal, NMFeagiSimpleDataNeuronScratch, NMFeagiSimpleDataNeuronVariance, NMFeagiSimpleTrait};
use crate::connectome_basis::neuron_model::{NeuronLayoutTagVoxel, NeuronModel};
use crate::connectome_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
use crate::neuron_models::{ProxyNMNeuroGenesisFromGenerator, ProxyNMNeuroGenesisFromStorage, ProxyNMNeurogenesisFromStorageOrByGenerator};

//region enums

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
/// Describes the quantization of the Feagi Simple Neuron Model
pub enum NMFeagiSimpleQuantKind {
    Standard32Bit,
}

/// Allows Neurogenesis of the Feagi Simple Neuron Model by its generator for a given quantization
#[derive(Clone, Debug)]
pub enum NMFeagiSimpleNeurogenesisGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(ProxyNMNeuroGenesisFromGenerator<FIQ, NMFeagiSimpleQuantStandard32Bit>)
}

impl<FIQ: FeagiIndexQuantization> NMFeagiSimpleNeurogenesisGeneratorQuanted<FIQ> {
    /// The quantization of the generator held
    pub const fn quant_kind(&self) -> NMFeagiSimpleQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiSimpleQuantKind::Standard32Bit,
        }
    }
}

/// Allows Neurogenesis of the Feagi Simple Neuron Model from its stored data for a given quantization
#[derive(Clone, Debug)] // TODO add serde
pub enum NMFeagiSimpleNeurogenesisStorageQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(ProxyNMNeuroGenesisFromStorage<FIQ, NMFeagiSimpleQuantStandard32Bit>)
}

impl<FIQ: FeagiIndexQuantization> NMFeagiSimpleNeurogenesisStorageQuanted<FIQ> {
    /// The quantization of the storage held
    pub const fn quant_kind(&self) -> NMFeagiSimpleQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiSimpleQuantKind::Standard32Bit,
        }
    }
}

/// Allows Neurogenesis of the Feagi Simple Neuron Model from either its stored data or by its
/// generator, for a given quantization
pub enum NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFeagiSimpleQuantStandard32Bit>)
}

impl<FIQ: FeagiIndexQuantization> NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    /// The quantization of the storage or generator held
    pub const fn quant_kind(&self) -> NMFeagiSimpleQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiSimpleQuantKind::Standard32Bit,
        }
    }
}


//endregion

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiSimpleQuantStandard32Bit;

impl NMFeagiSimpleTrait for NMFeagiSimpleQuantStandard32Bit {
    type Period = f32;
}

impl NeuronModel for NMFeagiSimpleQuantStandard32Bit {
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

    type QuantKind = NMFeagiSimpleQuantKind;
    const QUANT_KIND: Self::QuantKind = NMFeagiSimpleQuantKind::Standard32Bit;
}

impl MembranePotentialQuantization for NMFeagiSimpleQuantStandard32Bit
{
    type MembranePotentialQuant = f32;
}