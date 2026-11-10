use feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataVarianceSpecification;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::NeuronLayoutOf;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
use crate::common_systems::neurogenesis_generator_uniform_defaults::NeurogenesisGeneratorUniformDefault;
use crate::neuron_models::family::NeuronModel;
use half::{bf16, f16};

/// Quantization For the Foundation Death type
pub trait NMFoundationDeathTrait: NeuronModel<
    NeuronLayoutTag=NeuronLayoutTagVoxel,
    CorticalDataProperties=NMFoundationDeathDataCorticalProperties<Self>,
    CorticalDataInternal=NMFoundationDeathDataCorticalInternal<Self>,
    CorticalDataScratch=NMFoundationDeathDataCorticalScratch<Self>,
    CorticalDataVariance=NMFoundationDeathDataCorticalVariance<Self>,
    NeuronDataInternal=NMFoundationDeathDataNeuronInternal<Self>,
    NeuronDataScratch=NMFoundationDeathDataNeuronScratch<Self>,
    NeuronDataVariance=NMFoundationDeathDataNeuronVariance<Self>
>
{
}

//region Quant levels

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
}

impl MembranePotentialQuantization for NMFoundationDeathQuantFloat64
{
    type MembranePotentialQuant = f64;
}

//endregion

//region Cortical Properties

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationDeathDataCorticalProperties<NM: NMFoundationDeathTrait> {

    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFoundationDeathTrait> NMFoundationDeathDataCorticalProperties<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFoundationDeathTrait>(&self)
        -> ::core::result::Result<
            NMFoundationDeathDataCorticalProperties<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        Ok(
            NMFoundationDeathDataCorticalProperties::new()
        )
    }
}

impl<NM: NMFoundationDeathTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataPropertiesSpecification<NM>
for NMFoundationDeathDataCorticalProperties<NM> {}

//endregion


//region Cortical internal

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationDeathDataCorticalInternal<NM: NMFoundationDeathTrait> {

    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFoundationDeathTrait> NMFoundationDeathDataCorticalInternal<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFoundationDeathTrait>(&self)
        -> ::core::result::Result<
            NMFoundationDeathDataCorticalInternal<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        Ok(
            NMFoundationDeathDataCorticalInternal::new()
        )
    }
}

impl<NM: NMFoundationDeathTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataInternalSpecification<NM>
for NMFoundationDeathDataCorticalInternal<NM> {}

//endregion

//region Cortical Scratch

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationDeathDataCorticalScratch<NM: NMFoundationDeathTrait> {

    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFoundationDeathTrait> NMFoundationDeathDataCorticalScratch<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFoundationDeathTrait>(&self)
        -> ::core::result::Result<
            NMFoundationDeathDataCorticalScratch<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        Ok(
            NMFoundationDeathDataCorticalScratch::new()
        )
    }
}

impl<NM: NMFoundationDeathTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataScratchSpecification<NM>
for NMFoundationDeathDataCorticalScratch<NM> {}

//endregion

//region Cortical Variance

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationDeathDataCorticalVariance<NM: NMFoundationDeathTrait> {

    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFoundationDeathTrait> NMFoundationDeathDataCorticalVariance<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFoundationDeathTrait>(&self)
        -> ::core::result::Result<
            NMFoundationDeathDataCorticalVariance<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        Ok(
            NMFoundationDeathDataCorticalVariance::new()
        )
    }
}

//endregion

//region Neuron Internal

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationDeathDataNeuronInternal<NM: NMFoundationDeathTrait> {

    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFoundationDeathTrait> NMFoundationDeathDataNeuronInternal<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFoundationDeathTrait>(&self)
        -> ::core::result::Result<
            NMFoundationDeathDataNeuronInternal<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        Ok(
            NMFoundationDeathDataNeuronInternal::new()
        )
    }
}

impl<NM: NMFoundationDeathTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronDataInternalSpecification<NM>
for NMFoundationDeathDataNeuronInternal<NM> {}

//endregion

//region Neuron Scratch

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationDeathDataNeuronScratch<NM: NMFoundationDeathTrait> {

    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFoundationDeathTrait> NMFoundationDeathDataNeuronScratch<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFoundationDeathTrait>(&self)
        -> ::core::result::Result<
            NMFoundationDeathDataNeuronScratch<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        Ok(
            NMFoundationDeathDataNeuronScratch::new()
        )
    }
}

impl<NM: NMFoundationDeathTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronDataScratchSpecification<NM>
for NMFoundationDeathDataNeuronScratch<NM> {}

//endregion

//region Neuron Variance

#[derive(Clone, PartialEq, Debug)]
pub struct NMFoundationDeathDataNeuronVariance<NM: NMFoundationDeathTrait> {

    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFoundationDeathTrait> NMFoundationDeathDataNeuronVariance<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFoundationDeathTrait>(&self)
        -> ::core::result::Result<
            NMFoundationDeathDataNeuronVariance<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        Ok(
            NMFoundationDeathDataNeuronVariance::new()
        )
    }
}

impl<NM: NMFoundationDeathTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronDataVarianceSpecification<NM>
for NMFoundationDeathDataNeuronVariance<NM> {}

//endregion


impl<NM: NMFoundationDeathTrait> CorticalDataVarianceSpecification<NM> for NMFoundationDeathDataCorticalVariance<NM> {
    fn neuron_variance_at<FIQ: FeagiIndexQuantization>(
        &self,
        _layout: &NeuronLayoutOf<NM, FIQ>,
        _index: &::feagi_basis::neurons::CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> NMFoundationDeathDataNeuronVariance<NM> {
        NMFoundationDeathDataNeuronVariance::new()
    }
}


/*
use super::neurogenesis::FoundationDeathNeurogenesisDefaults;
use crate::neuron_models::defaults_generator::NeurogenesisDefaultsGenerator;
use feagi_connectome_neural::neural_basis::neuron_model::model_data::{CorticalDataInternalSpecification, CorticalDataPropertiesSpecification, CorticalDataScratchSpecification, CorticalDataVarianceSpecification, NeuronDataInternalSpecification, NeuronDataScratchSpecification, NeuronDataVarianceSpecification};
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::{MembranePotentialQuantization, FeagiIndexQuantization};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_proc::{define_cortical_dynamics_quantization_template, expand_cortical_dynamics_quantization};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutOf};
use half::{bf16, f16};

define_cortical_dynamics_quantization_template! {
    /// Quantization For the Foundation Death type
    FoundationDeath

    NeuronLayout: NeuronLayoutTagVoxel

    // How the scratch and variance data of a neuron is generated
    NeurogenesisDefaults: FoundationDeathNeurogenesisDefaults<FIQ, Self>

    // How the neurons of a new cortical area are generated, every neuron starting from the same values
    NeurogenesisGenerator: NeurogenesisDefaultsGenerator<FIQ, Self>

    QuantizationLevels: {
        Float16: {
            MembranePotentialQuant: f16,
        }

        BFloat16: {
            MembranePotentialQuant: bf16,
        }

        Float32: {
            MembranePotentialQuant: f32,
        }

        Float64: {
            MembranePotentialQuant: f64,
        }
    }
}

cortical_dynamics_quantization_foundation_death_template!(expand_cortical_dynamics_quantization);

 */
