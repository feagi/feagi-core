use crate::neuron_models::models::foundation_death::NMFoundationDeathTrait;
use feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};

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
