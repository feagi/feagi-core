use crate::common_systems::neurogenesis_generator_uniform_defaults::NeurogenesisGeneratorUniformDefault;
use crate::neuron_models::models::foundation_death::{NMFoundationDeathDataCorticalInternal, NMFoundationDeathDataCorticalProperties, NMFoundationDeathDataCorticalScratch, NMFoundationDeathDataCorticalVariance, NMFoundationDeathDataNeuronInternal, NMFoundationDeathDataNeuronScratch, NMFoundationDeathDataNeuronVariance, NMFoundationDeathTrait};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
use crate::neuron_models::models::{ProxyNMNeuroGenesisFromGenerator, ProxyNMNeuroGenesisFromStorage, ProxyNMNeurogenesisFromStorageOrByGenerator};
use half::{bf16, f16};
//region enums

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
/// Describes the quantization of the Foundation Death Neuron Model
pub enum NMFoundationDeathQuantKind {
    Float16,
    BFloat16,
    Float32,
    Float64,
}

/// Allows Neurogenesis of the Foundation Death Neuron Model by its generator for a given quantization
#[derive(Clone, Debug)]
pub enum NMFoundationDeathNeurogenesisGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Float16(ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationDeathQuantFloat16>),
    BFloat16(ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationDeathQuantBFloat16>),
    Float32(ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationDeathQuantFloat32>),
    Float64(ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationDeathQuantFloat64>)
}

impl<FIQ: FeagiIndexQuantization> NMFoundationDeathNeurogenesisGeneratorQuanted<FIQ> {
    /// The quantization of the generator held
    pub const fn quant_kind(&self) -> NMFoundationDeathQuantKind {
        match self {
            Self::Float16(_) => NMFoundationDeathQuantKind::Float16,
            Self::BFloat16(_) => NMFoundationDeathQuantKind::BFloat16,
            Self::Float32(_) => NMFoundationDeathQuantKind::Float32,
            Self::Float64(_) => NMFoundationDeathQuantKind::Float64,
        }
    }
}

/// Allows Neurogenesis of the Foundation Death Neuron Model from its stored data for a given quantization
#[derive(Clone, Debug)] // TODO add serde
pub enum NMFoundationDeathNeurogenesisStorageQuanted<FIQ: FeagiIndexQuantization> {
    Float16(ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationDeathQuantFloat16>),
    BFloat16(ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationDeathQuantBFloat16>),
    Float32(ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationDeathQuantFloat32>),
    Float64(ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationDeathQuantFloat64>)
}

impl<FIQ: FeagiIndexQuantization> NMFoundationDeathNeurogenesisStorageQuanted<FIQ> {
    /// The quantization of the storage held
    pub const fn quant_kind(&self) -> NMFoundationDeathQuantKind {
        match self {
            Self::Float16(_) => NMFoundationDeathQuantKind::Float16,
            Self::BFloat16(_) => NMFoundationDeathQuantKind::BFloat16,
            Self::Float32(_) => NMFoundationDeathQuantKind::Float32,
            Self::Float64(_) => NMFoundationDeathQuantKind::Float64,
        }
    }
}

/// Allows Neurogenesis of the Foundation Death Neuron Model from either its stored data or by its
/// generator, for a given quantization
#[derive(Clone, Debug)] // TODO add serde
pub enum NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Float16(ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationDeathQuantFloat16>),
    BFloat16(ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationDeathQuantBFloat16>),
    Float32(ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationDeathQuantFloat32>),
    Float64(ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationDeathQuantFloat64>)
}

impl<FIQ: FeagiIndexQuantization> NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    /// The quantization of the storage or generator held
    pub const fn quant_kind(&self) -> NMFoundationDeathQuantKind {
        match self {
            Self::Float16(_) => NMFoundationDeathQuantKind::Float16,
            Self::BFloat16(_) => NMFoundationDeathQuantKind::BFloat16,
            Self::Float32(_) => NMFoundationDeathQuantKind::Float32,
            Self::Float64(_) => NMFoundationDeathQuantKind::Float64,
        }
    }
}


//endregion

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationDeathQuantFloat16;

impl NMFoundationDeathTrait for NMFoundationDeathQuantFloat16 {
}

impl NeuronModel for NMFoundationDeathQuantFloat16 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::FoundationDeathNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisByGeneratorType<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMFoundationDeathDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFoundationDeathDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFoundationDeathDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFoundationDeathDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFoundationDeathDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFoundationDeathDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFoundationDeathDataNeuronVariance<Self>;

    type QuantKind = NMFoundationDeathQuantKind;
    const QUANT_KIND: Self::QuantKind = NMFoundationDeathQuantKind::Float16;
}

impl MembranePotentialQuantization for NMFoundationDeathQuantFloat16
{
    type MembranePotentialQuant = f16;
}

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationDeathQuantBFloat16;

impl NMFoundationDeathTrait for NMFoundationDeathQuantBFloat16 {
}

impl NeuronModel for NMFoundationDeathQuantBFloat16 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::FoundationDeathNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisByGeneratorType<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMFoundationDeathDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFoundationDeathDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFoundationDeathDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFoundationDeathDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFoundationDeathDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFoundationDeathDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFoundationDeathDataNeuronVariance<Self>;

    type QuantKind = NMFoundationDeathQuantKind;
    const QUANT_KIND: Self::QuantKind = NMFoundationDeathQuantKind::BFloat16;
}

impl MembranePotentialQuantization for NMFoundationDeathQuantBFloat16
{
    type MembranePotentialQuant = bf16;
}

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationDeathQuantFloat32;

impl NMFoundationDeathTrait for NMFoundationDeathQuantFloat32 {
}

impl NeuronModel for NMFoundationDeathQuantFloat32 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::FoundationDeathNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisByGeneratorType<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMFoundationDeathDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFoundationDeathDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFoundationDeathDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFoundationDeathDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFoundationDeathDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFoundationDeathDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFoundationDeathDataNeuronVariance<Self>;

    type QuantKind = NMFoundationDeathQuantKind;
    const QUANT_KIND: Self::QuantKind = NMFoundationDeathQuantKind::Float32;
}

impl MembranePotentialQuantization for NMFoundationDeathQuantFloat32
{
    type MembranePotentialQuant = f32;
}

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationDeathQuantFloat64;

impl NMFoundationDeathTrait for NMFoundationDeathQuantFloat64 {
}

impl NeuronModel for NMFoundationDeathQuantFloat64 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::FoundationDeathNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisByGeneratorType<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMFoundationDeathDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFoundationDeathDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFoundationDeathDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFoundationDeathDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFoundationDeathDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFoundationDeathDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFoundationDeathDataNeuronVariance<Self>;

    type QuantKind = NMFoundationDeathQuantKind;
    const QUANT_KIND: Self::QuantKind = NMFoundationDeathQuantKind::Float64;
}

impl MembranePotentialQuantization for NMFoundationDeathQuantFloat64
{
    type MembranePotentialQuant = f64;
}