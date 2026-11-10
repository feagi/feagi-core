use crate::common_systems::neurogenesis_generator_uniform_defaults::NeurogenesisGeneratorUniformDefault;
use crate::neuron_models::models::foundation_power::{NMFoundationPowerDataCorticalInternal, NMFoundationPowerDataCorticalProperties, NMFoundationPowerDataCorticalScratch, NMFoundationPowerDataCorticalVariance, NMFoundationPowerDataNeuronInternal, NMFoundationPowerDataNeuronScratch, NMFoundationPowerDataNeuronVariance, NMFoundationPowerTrait};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
use crate::neuron_models::models::{ProxyNMNeuroGenesisFromGenerator, ProxyNMNeuroGenesisFromStorage, ProxyNMNeurogenesisFromStorageOrByGenerator};
use half::{bf16, f16};
//region enums

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
/// Describes the quantization of the Foundation Power Neuron Model
pub enum NMFoundationPowerQuantKind {
    Float16,
    BFloat16,
    Float32,
    Float64,
}

/// Allows Neurogenesis of the Foundation Power Neuron Model by its generator for a given quantization
#[derive(Clone, Debug)]
pub enum NMFoundationPowerNeurogenesisGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Float16(ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationPowerQuantFloat16>),
    BFloat16(ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationPowerQuantBFloat16>),
    Float32(ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationPowerQuantFloat32>),
    Float64(ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationPowerQuantFloat64>)
}

impl<FIQ: FeagiIndexQuantization> NMFoundationPowerNeurogenesisGeneratorQuanted<FIQ> {
    /// The quantization of the generator held
    pub const fn quant_kind(&self) -> NMFoundationPowerQuantKind {
        match self {
            Self::Float16(_) => NMFoundationPowerQuantKind::Float16,
            Self::BFloat16(_) => NMFoundationPowerQuantKind::BFloat16,
            Self::Float32(_) => NMFoundationPowerQuantKind::Float32,
            Self::Float64(_) => NMFoundationPowerQuantKind::Float64,
        }
    }
}

/// Allows Neurogenesis of the Foundation Power Neuron Model from its stored data for a given quantization
#[derive(Clone, Debug)] // TODO add serde
pub enum NMFoundationPowerNeurogenesisStorageQuanted<FIQ: FeagiIndexQuantization> {
    Float16(ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationPowerQuantFloat16>),
    BFloat16(ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationPowerQuantBFloat16>),
    Float32(ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationPowerQuantFloat32>),
    Float64(ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationPowerQuantFloat64>)
}

impl<FIQ: FeagiIndexQuantization> NMFoundationPowerNeurogenesisStorageQuanted<FIQ> {
    /// The quantization of the storage held
    pub const fn quant_kind(&self) -> NMFoundationPowerQuantKind {
        match self {
            Self::Float16(_) => NMFoundationPowerQuantKind::Float16,
            Self::BFloat16(_) => NMFoundationPowerQuantKind::BFloat16,
            Self::Float32(_) => NMFoundationPowerQuantKind::Float32,
            Self::Float64(_) => NMFoundationPowerQuantKind::Float64,
        }
    }
}

/// Allows Neurogenesis of the Foundation Power Neuron Model from either its stored data or by its
/// generator, for a given quantization
#[derive(Clone, Debug)] // TODO add serde
pub enum NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted<FIQ: FeagiIndexQuantization> {
    Float16(ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationPowerQuantFloat16>),
    BFloat16(ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationPowerQuantBFloat16>),
    Float32(ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationPowerQuantFloat32>),
    Float64(ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationPowerQuantFloat64>)
}

impl<FIQ: FeagiIndexQuantization> NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    /// The quantization of the storage or generator held
    pub const fn quant_kind(&self) -> NMFoundationPowerQuantKind {
        match self {
            Self::Float16(_) => NMFoundationPowerQuantKind::Float16,
            Self::BFloat16(_) => NMFoundationPowerQuantKind::BFloat16,
            Self::Float32(_) => NMFoundationPowerQuantKind::Float32,
            Self::Float64(_) => NMFoundationPowerQuantKind::Float64,
        }
    }
}


//endregion

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationPowerQuantFloat16;

impl NMFoundationPowerTrait for NMFoundationPowerQuantFloat16 {
}

impl NeuronModel for NMFoundationPowerQuantFloat16 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::FoundationPowerNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisByGeneratorType<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMFoundationPowerDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFoundationPowerDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFoundationPowerDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFoundationPowerDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFoundationPowerDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFoundationPowerDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFoundationPowerDataNeuronVariance<Self>;

    type QuantKind = NMFoundationPowerQuantKind;
    const QUANT_KIND: Self::QuantKind = NMFoundationPowerQuantKind::Float16;
}

impl MembranePotentialQuantization for NMFoundationPowerQuantFloat16
{
    type MembranePotentialQuant = f16;
}

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationPowerQuantBFloat16;

impl NMFoundationPowerTrait for NMFoundationPowerQuantBFloat16 {
}

impl NeuronModel for NMFoundationPowerQuantBFloat16 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::FoundationPowerNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisByGeneratorType<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMFoundationPowerDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFoundationPowerDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFoundationPowerDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFoundationPowerDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFoundationPowerDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFoundationPowerDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFoundationPowerDataNeuronVariance<Self>;

    type QuantKind = NMFoundationPowerQuantKind;
    const QUANT_KIND: Self::QuantKind = NMFoundationPowerQuantKind::BFloat16;
}

impl MembranePotentialQuantization for NMFoundationPowerQuantBFloat16
{
    type MembranePotentialQuant = bf16;
}

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationPowerQuantFloat32;

impl NMFoundationPowerTrait for NMFoundationPowerQuantFloat32 {
}

impl NeuronModel for NMFoundationPowerQuantFloat32 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::FoundationPowerNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisByGeneratorType<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMFoundationPowerDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFoundationPowerDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFoundationPowerDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFoundationPowerDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFoundationPowerDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFoundationPowerDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFoundationPowerDataNeuronVariance<Self>;

    type QuantKind = NMFoundationPowerQuantKind;
    const QUANT_KIND: Self::QuantKind = NMFoundationPowerQuantKind::Float32;
}

impl MembranePotentialQuantization for NMFoundationPowerQuantFloat32
{
    type MembranePotentialQuant = f32;
}

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationPowerQuantFloat64;

impl NMFoundationPowerTrait for NMFoundationPowerQuantFloat64 {
}

impl NeuronModel for NMFoundationPowerQuantFloat64 {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> =
        super::neurogenesis::FoundationPowerNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisByGeneratorType<FIQ: FeagiIndexQuantization> =
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMFoundationPowerDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFoundationPowerDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFoundationPowerDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFoundationPowerDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFoundationPowerDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFoundationPowerDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFoundationPowerDataNeuronVariance<Self>;

    type QuantKind = NMFoundationPowerQuantKind;
    const QUANT_KIND: Self::QuantKind = NMFoundationPowerQuantKind::Float64;
}

impl MembranePotentialQuantization for NMFoundationPowerQuantFloat64
{
    type MembranePotentialQuant = f64;
}