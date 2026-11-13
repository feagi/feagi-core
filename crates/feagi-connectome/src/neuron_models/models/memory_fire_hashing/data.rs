use crate::neuron_models::models::memory_fire_hashing::NMMemoryFireHashingTrait;
use feagi_basis::create_wrapped_quantized_decimal;

create_wrapped_quantized_decimal!(
    /// For handling timing
    pub Period
);

//region Cortical Properties

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMMemoryFireHashingDataCorticalProperties<NM: NMMemoryFireHashingTrait> {
    #[doc = " How long till a neuron of the cortical area can fire again"]
    pub refractory_period: Period<NM::Period>,

    #[doc = " At what threshold do we fire"]
    pub firing_threshold: NM::MembranePotentialQuant,

    #[doc = " Current post-synaptic"]
    pub postsynaptic_current: NM::MembranePotentialQuant,

    #[doc = " max postsynaptic current"]
    pub postsynaptic_current_max: NM::MembranePotentialQuant,

    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMMemoryFireHashingTrait> NMMemoryFireHashingDataCorticalProperties<NM> {
    pub const fn new(
        refractory_period: Period<NM::Period>,
        firing_threshold: NM::MembranePotentialQuant,
        postsynaptic_current: NM::MembranePotentialQuant,
        postsynaptic_current_max: NM::MembranePotentialQuant,
    ) -> Self {
        Self {
            refractory_period,
            firing_threshold,
            postsynaptic_current,
            postsynaptic_current_max,
            _p: ::core::marker::PhantomData,
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMMemoryFireHashingTrait>(
        &self,
    ) -> ::core::result::Result<NMMemoryFireHashingDataCorticalProperties<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError>
    {
        use ::feagi_basis::quantization::quantizable::QuantizedDecimalTrait;
        Ok(NMMemoryFireHashingDataCorticalProperties::new(
            QuantizedDecimalTrait::try_to_quantization(self.refractory_period)?,
            QuantizedDecimalTrait::try_to_quantization(self.firing_threshold)?,
            QuantizedDecimalTrait::try_to_quantization(self.postsynaptic_current)?,
            QuantizedDecimalTrait::try_to_quantization(self.postsynaptic_current_max)?,
        ))
    }
}

impl<NM: NMMemoryFireHashingTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataPropertiesSpecification<NM>
    for NMMemoryFireHashingDataCorticalProperties<NM>
{
}

//endregion

//region Cortical internal

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMMemoryFireHashingDataCorticalInternal<NM: NMMemoryFireHashingTrait> {
    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMMemoryFireHashingTrait> NMMemoryFireHashingDataCorticalInternal<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData,
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMMemoryFireHashingTrait>(
        &self,
    ) -> ::core::result::Result<NMMemoryFireHashingDataCorticalInternal<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError>
    {
        Ok(NMMemoryFireHashingDataCorticalInternal::new())
    }
}

impl<NM: NMMemoryFireHashingTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataInternalSpecification<NM>
    for NMMemoryFireHashingDataCorticalInternal<NM>
{
}

//endregion

//region Cortical Scratch

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMMemoryFireHashingDataCorticalScratch<NM: NMMemoryFireHashingTrait> {
    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMMemoryFireHashingTrait> NMMemoryFireHashingDataCorticalScratch<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData,
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMMemoryFireHashingTrait>(
        &self,
    ) -> ::core::result::Result<NMMemoryFireHashingDataCorticalScratch<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError>
    {
        Ok(NMMemoryFireHashingDataCorticalScratch::new())
    }
}

impl<NM: NMMemoryFireHashingTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataScratchSpecification<NM>
    for NMMemoryFireHashingDataCorticalScratch<NM>
{
}

//endregion

//region Cortical Variance

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMMemoryFireHashingDataCorticalVariance<NM: NMMemoryFireHashingTrait> {
    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMMemoryFireHashingTrait> NMMemoryFireHashingDataCorticalVariance<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData,
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMMemoryFireHashingTrait>(
        &self,
    ) -> ::core::result::Result<NMMemoryFireHashingDataCorticalVariance<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError>
    {
        Ok(NMMemoryFireHashingDataCorticalVariance::new())
    }
}

//endregion

//region Neuron Internal

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMMemoryFireHashingDataNeuronInternal<NM: NMMemoryFireHashingTrait> {
    #[doc = " How many more bursts the neuron is refractory for"]
    pub refractory_countdown: Period<NM::Period>,

    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMMemoryFireHashingTrait> NMMemoryFireHashingDataNeuronInternal<NM> {
    pub const fn new(refractory_countdown: Period<NM::Period>) -> Self {
        Self {
            refractory_countdown,
            _p: ::core::marker::PhantomData,
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMMemoryFireHashingTrait>(
        &self,
    ) -> ::core::result::Result<NMMemoryFireHashingDataNeuronInternal<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError> {
        use ::feagi_basis::quantization::quantizable::QuantizedDecimalTrait;
        Ok(NMMemoryFireHashingDataNeuronInternal::new(QuantizedDecimalTrait::try_to_quantization(
            self.refractory_countdown,
        )?))
    }
}

impl<NM: NMMemoryFireHashingTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronDataInternalSpecification<NM>
    for NMMemoryFireHashingDataNeuronInternal<NM>
{
}

//endregion

//region Neuron Scratch

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMMemoryFireHashingDataNeuronScratch<NM: NMMemoryFireHashingTrait> {
    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMMemoryFireHashingTrait> NMMemoryFireHashingDataNeuronScratch<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData,
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMMemoryFireHashingTrait>(
        &self,
    ) -> ::core::result::Result<NMMemoryFireHashingDataNeuronScratch<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError> {
        Ok(NMMemoryFireHashingDataNeuronScratch::new())
    }
}

impl<NM: NMMemoryFireHashingTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronDataScratchSpecification<NM>
    for NMMemoryFireHashingDataNeuronScratch<NM>
{
}

//endregion

//region Neuron Variance

#[derive(Clone, PartialEq, Debug)]
pub struct NMMemoryFireHashingDataNeuronVariance<NM: NMMemoryFireHashingTrait> {
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMMemoryFireHashingTrait> NMMemoryFireHashingDataNeuronVariance<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData,
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMMemoryFireHashingTrait>(
        &self,
    ) -> ::core::result::Result<NMMemoryFireHashingDataNeuronVariance<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError> {
        Ok(NMMemoryFireHashingDataNeuronVariance::new())
    }
}

impl<NM: NMMemoryFireHashingTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronDataVarianceSpecification<NM>
    for NMMemoryFireHashingDataNeuronVariance<NM>
{
}

//endregion
