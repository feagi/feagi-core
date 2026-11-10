use crate::neuron_models::models::foundation_power::NMFoundationPowerTrait;
use feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};

//region Cortical Properties

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationPowerDataCorticalProperties<NM: NMFoundationPowerTrait> {

    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFoundationPowerTrait> NMFoundationPowerDataCorticalProperties<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFoundationPowerTrait>(&self)
        -> ::core::result::Result<
            NMFoundationPowerDataCorticalProperties<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        Ok(
            NMFoundationPowerDataCorticalProperties::new()
        )
    }
}

impl<NM: NMFoundationPowerTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataPropertiesSpecification<NM>
for NMFoundationPowerDataCorticalProperties<NM> {}

//endregion


//region Cortical internal

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationPowerDataCorticalInternal<NM: NMFoundationPowerTrait> {

    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFoundationPowerTrait> NMFoundationPowerDataCorticalInternal<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFoundationPowerTrait>(&self)
        -> ::core::result::Result<
            NMFoundationPowerDataCorticalInternal<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        Ok(
            NMFoundationPowerDataCorticalInternal::new()
        )
    }
}

impl<NM: NMFoundationPowerTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataInternalSpecification<NM>
for NMFoundationPowerDataCorticalInternal<NM> {}

//endregion

//region Cortical Scratch

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationPowerDataCorticalScratch<NM: NMFoundationPowerTrait> {

    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFoundationPowerTrait> NMFoundationPowerDataCorticalScratch<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFoundationPowerTrait>(&self)
        -> ::core::result::Result<
            NMFoundationPowerDataCorticalScratch<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        Ok(
            NMFoundationPowerDataCorticalScratch::new()
        )
    }
}

impl<NM: NMFoundationPowerTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataScratchSpecification<NM>
for NMFoundationPowerDataCorticalScratch<NM> {}

//endregion

//region Cortical Variance

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationPowerDataCorticalVariance<NM: NMFoundationPowerTrait> {

    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFoundationPowerTrait> NMFoundationPowerDataCorticalVariance<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFoundationPowerTrait>(&self)
        -> ::core::result::Result<
            NMFoundationPowerDataCorticalVariance<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        Ok(
            NMFoundationPowerDataCorticalVariance::new()
        )
    }
}

//endregion

//region Neuron Internal

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationPowerDataNeuronInternal<NM: NMFoundationPowerTrait> {

    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFoundationPowerTrait> NMFoundationPowerDataNeuronInternal<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFoundationPowerTrait>(&self)
        -> ::core::result::Result<
            NMFoundationPowerDataNeuronInternal<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        Ok(
            NMFoundationPowerDataNeuronInternal::new()
        )
    }
}

impl<NM: NMFoundationPowerTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronDataInternalSpecification<NM>
for NMFoundationPowerDataNeuronInternal<NM> {}

//endregion

//region Neuron Scratch

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFoundationPowerDataNeuronScratch<NM: NMFoundationPowerTrait> {

    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFoundationPowerTrait> NMFoundationPowerDataNeuronScratch<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFoundationPowerTrait>(&self)
        -> ::core::result::Result<
            NMFoundationPowerDataNeuronScratch<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        Ok(
            NMFoundationPowerDataNeuronScratch::new()
        )
    }
}

impl<NM: NMFoundationPowerTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronDataScratchSpecification<NM>
for NMFoundationPowerDataNeuronScratch<NM> {}

//endregion

//region Neuron Variance

#[derive(Clone, PartialEq, Debug)]
pub struct NMFoundationPowerDataNeuronVariance<NM: NMFoundationPowerTrait> {

    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFoundationPowerTrait> NMFoundationPowerDataNeuronVariance<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFoundationPowerTrait>(&self)
        -> ::core::result::Result<
            NMFoundationPowerDataNeuronVariance<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        Ok(
            NMFoundationPowerDataNeuronVariance::new()
        )
    }
}

impl<NM: NMFoundationPowerTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronDataVarianceSpecification<NM>
for NMFoundationPowerDataNeuronVariance<NM> {}

//endregion


/*
use super::neurogenesis::FoundationPowerNeurogenesisDefaults;
use crate::neuron_models::defaults_generator::NeurogenesisDefaultsGenerator;
use feagi_connectome_neural::neural_basis::neuron_model::model_data::{CorticalDataInternalSpecification, CorticalDataPropertiesSpecification, CorticalDataScratchSpecification, CorticalDataVarianceSpecification, NeuronDataInternalSpecification, NeuronDataScratchSpecification, NeuronDataVarianceSpecification};
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutOf};
use feagi_connectome_neural::neural_basis::quantization_levels::{MembranePotentialQuantization, FeagiIndexQuantization};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_proc::{define_cortical_dynamics_quantization_template, expand_cortical_dynamics_quantization};
use half::{bf16, f16};

define_cortical_dynamics_quantization_template! {
    /// Quantization For the Foundation Power type
    FoundationPower

    NeuronLayout: NeuronLayoutTagVoxel

    // How the scratch and variance data of a neuron is generated
    NeurogenesisDefaults: FoundationPowerNeurogenesisDefaults<FIQ, Self>

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

cortical_dynamics_quantization_foundation_power_template!(expand_cortical_dynamics_quantization);

 */
