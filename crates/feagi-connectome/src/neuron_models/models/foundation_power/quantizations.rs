use crate::common_systems::neurogenesis_generator_uniform_defaults::NeurogenesisGeneratorUniformDefault;
use crate::connectome_basis::neuron_model::foundation_power::{NMFoundationPowerDataCorticalInternal, NMFoundationPowerDataCorticalProperties, NMFoundationPowerDataCorticalScratch, NMFoundationPowerDataCorticalVariance, NMFoundationPowerDataNeuronInternal, NMFoundationPowerDataNeuronScratch, NMFoundationPowerDataNeuronVariance, NMFoundationPowerTrait};
use crate::connectome_basis::neuron_model::{NeuronLayoutTagVoxel, NeuronModel};
use crate::connectome_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
use half::{bf16, f16};

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
}

impl MembranePotentialQuantization for NMFoundationPowerQuantFloat64
{
    type MembranePotentialQuant = f64;
}