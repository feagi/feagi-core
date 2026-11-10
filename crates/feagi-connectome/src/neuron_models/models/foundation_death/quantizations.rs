use crate::common_systems::neurogenesis_generator_uniform_defaults::NeurogenesisGeneratorUniformDefault;
use crate::connectome_basis::neuron_model::foundation_death::{NMFoundationDeathDataCorticalInternal, NMFoundationDeathDataCorticalProperties, NMFoundationDeathDataCorticalScratch, NMFoundationDeathDataCorticalVariance, NMFoundationDeathDataNeuronInternal, NMFoundationDeathDataNeuronScratch, NMFoundationDeathDataNeuronVariance, NMFoundationDeathTrait};
use crate::connectome_basis::neuron_model::{NeuronLayoutTagVoxel, NeuronModel};
use crate::connectome_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
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