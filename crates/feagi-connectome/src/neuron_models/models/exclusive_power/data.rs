use crate::neuron_models::models::exclusive_power::NMExclusivePowerTrait;

//region Cortical Properties

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMExclusivePowerDataCorticalProperties<NM: NMExclusivePowerTrait> {
    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMExclusivePowerTrait> NMExclusivePowerDataCorticalProperties<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData,
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMExclusivePowerTrait>(
        &self,
    ) -> ::core::result::Result<NMExclusivePowerDataCorticalProperties<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError>
    {
        Ok(NMExclusivePowerDataCorticalProperties::new())
    }
}

impl<NM: NMExclusivePowerTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataPropertiesSpecification<NM>
    for NMExclusivePowerDataCorticalProperties<NM>
{
}

//endregion

//region Cortical internal

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMExclusivePowerDataCorticalInternal<NM: NMExclusivePowerTrait> {
    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMExclusivePowerTrait> NMExclusivePowerDataCorticalInternal<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData,
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMExclusivePowerTrait>(
        &self,
    ) -> ::core::result::Result<NMExclusivePowerDataCorticalInternal<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError> {
        Ok(NMExclusivePowerDataCorticalInternal::new())
    }
}

impl<NM: NMExclusivePowerTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataInternalSpecification<NM>
    for NMExclusivePowerDataCorticalInternal<NM>
{
}

//endregion

//region Cortical Scratch

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMExclusivePowerDataCorticalScratch<NM: NMExclusivePowerTrait> {
    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMExclusivePowerTrait> NMExclusivePowerDataCorticalScratch<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData,
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMExclusivePowerTrait>(
        &self,
    ) -> ::core::result::Result<NMExclusivePowerDataCorticalScratch<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError> {
        Ok(NMExclusivePowerDataCorticalScratch::new())
    }
}

impl<NM: NMExclusivePowerTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataScratchSpecification<NM>
    for NMExclusivePowerDataCorticalScratch<NM>
{
}

//endregion

//region Cortical Variance

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMExclusivePowerDataCorticalVariance<NM: NMExclusivePowerTrait> {
    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMExclusivePowerTrait> NMExclusivePowerDataCorticalVariance<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData,
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMExclusivePowerTrait>(
        &self,
    ) -> ::core::result::Result<NMExclusivePowerDataCorticalVariance<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError> {
        Ok(NMExclusivePowerDataCorticalVariance::new())
    }
}

//endregion

//region Neuron Internal

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMExclusivePowerDataNeuronInternal<NM: NMExclusivePowerTrait> {
    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMExclusivePowerTrait> NMExclusivePowerDataNeuronInternal<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData,
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMExclusivePowerTrait>(
        &self,
    ) -> ::core::result::Result<NMExclusivePowerDataNeuronInternal<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError> {
        Ok(NMExclusivePowerDataNeuronInternal::new())
    }
}

impl<NM: NMExclusivePowerTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronDataInternalSpecification<NM>
    for NMExclusivePowerDataNeuronInternal<NM>
{
}

//endregion

//region Neuron Scratch

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMExclusivePowerDataNeuronScratch<NM: NMExclusivePowerTrait> {
    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMExclusivePowerTrait> NMExclusivePowerDataNeuronScratch<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData,
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMExclusivePowerTrait>(
        &self,
    ) -> ::core::result::Result<NMExclusivePowerDataNeuronScratch<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError> {
        Ok(NMExclusivePowerDataNeuronScratch::new())
    }
}

impl<NM: NMExclusivePowerTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronDataScratchSpecification<NM>
    for NMExclusivePowerDataNeuronScratch<NM>
{
}

//endregion

//region Neuron Variance

#[derive(Clone, PartialEq, Debug)]
pub struct NMExclusivePowerDataNeuronVariance<NM: NMExclusivePowerTrait> {
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMExclusivePowerTrait> NMExclusivePowerDataNeuronVariance<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData,
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMExclusivePowerTrait>(
        &self,
    ) -> ::core::result::Result<NMExclusivePowerDataNeuronVariance<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError> {
        Ok(NMExclusivePowerDataNeuronVariance::new())
    }
}

impl<NM: NMExclusivePowerTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronDataVarianceSpecification<NM>
    for NMExclusivePowerDataNeuronVariance<NM>
{
}

//endregion
