use crate::common_systems::neurogenesis_generator_uniform_defaults::NeurogenesisGeneratorUniformDefault;
use crate::connectome_basis::neuron_model::feagi_advanced::{NMFeagiAdvancedDataCorticalInternal, NMFeagiAdvancedDataCorticalProperties, NMFeagiAdvancedDataCorticalScratch, NMFeagiAdvancedDataCorticalVariance, NMFeagiAdvancedDataNeuronInternal, NMFeagiAdvancedDataNeuronScratch, NMFeagiAdvancedDataNeuronVariance, NMFeagiAdvancedTrait};
use crate::connectome_basis::neuron_model::{NeuronLayoutTagVoxel, NeuronModel};
use crate::connectome_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
use crate::neuron_models::{ProxyNMNeuroGenesisFromGenerator, ProxyNMNeuroGenesisFromStorage, ProxyNMNeurogenesisFromStorageOrByGenerator};

//region enums

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
/// Describes the quantization of the Feagi Advanced Neuron Model
pub enum NMFeagiAdvancedQuantKind {
    Standard32Bit,
}

/// Allows Neurogenesis of the Feagi Advanced Neuron Model by its generator for a given quantization
#[derive(Clone, Debug)]
pub enum NMFeagiAdvancedNeurogenesisGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(ProxyNMNeuroGenesisFromGenerator<FIQ, NMFeagiAdvancedQuantStandard32Bit>)
}

impl<FIQ: FeagiIndexQuantization> NMFeagiAdvancedNeurogenesisGeneratorQuanted<FIQ> {
    /// The quantization of the generator held
    pub const fn quant_kind(&self) -> NMFeagiAdvancedQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiAdvancedQuantKind::Standard32Bit,
        }
    }
}

/// Allows Neurogenesis of the Feagi Advanced Neuron Model from its stored data for a given quantization
#[derive(Clone, Debug)] // TODO add serde
pub enum NMFeagiAdvancedNeurogenesisStorageQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(ProxyNMNeuroGenesisFromStorage<FIQ, NMFeagiAdvancedQuantStandard32Bit>)
}

impl<FIQ: FeagiIndexQuantization> NMFeagiAdvancedNeurogenesisStorageQuanted<FIQ> {
    /// The quantization of the storage held
    pub const fn quant_kind(&self) -> NMFeagiAdvancedQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiAdvancedQuantKind::Standard32Bit,
        }
    }
}

/// Allows Neurogenesis of the Feagi Advanced Neuron Model from either its stored data or by its
/// generator, for a given quantization
pub enum NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Standard32Bit(ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFeagiAdvancedQuantStandard32Bit>)
}

impl<FIQ: FeagiIndexQuantization> NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    /// The quantization of the storage or generator held
    pub const fn quant_kind(&self) -> NMFeagiAdvancedQuantKind {
        match self {
            Self::Standard32Bit(_) => NMFeagiAdvancedQuantKind::Standard32Bit,
        }
    }
}


//endregion

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiAdvancedQuantStandard32Bit;

impl NMFeagiAdvancedTrait for NMFeagiAdvancedQuantStandard32Bit {
    type Period = f32;
}

impl NeuronModel for NMFeagiAdvancedQuantStandard32Bit {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::FeagiAdvancedScratchVarianceGenerator<FIQ, Self>;

    type NeurogenesisByGeneratorType<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMFeagiAdvancedDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFeagiAdvancedDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFeagiAdvancedDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFeagiAdvancedDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFeagiAdvancedDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFeagiAdvancedDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFeagiAdvancedDataNeuronVariance<Self>;
    
    type QuantKind = NMFeagiAdvancedQuantKind;
    const QUANT_KIND: Self::QuantKind = NMFeagiAdvancedQuantKind::Standard32Bit;
}

impl MembranePotentialQuantization for NMFeagiAdvancedQuantStandard32Bit
{
    type MembranePotentialQuant = f32;
}