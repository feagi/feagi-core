use crate::neuron_models::models::feagi_simple::NMFeagiSimpleTrait;

use feagi_basis::neurons::CorticalAreaNeuronPotential;

pub mod wrapped {
    use feagi_basis::create_wrapped_quantized_decimal;

    create_wrapped_quantized_decimal!(
        /// For handling amounts of time
        pub Period
    );
}

//region Cortical Properties

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiSimpleDataCorticalProperties<NM: NMFeagiSimpleTrait> {
    #[doc = " How long till a neuron of the cortical area can fire again"]
    pub refractory_period: wrapped::Period<NM::Period>,

    #[doc = " At what threshold do we fire"]
    pub firing_threshold: CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,

    #[doc = " Current post-synaptic"]
    pub postsynaptic_current: CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,

    #[doc = " max postsynaptic current"]
    pub postsynaptic_current_max: CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,
}

impl<NM: NMFeagiSimpleTrait> NMFeagiSimpleDataCorticalProperties<NM> {
    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFeagiSimpleTrait>(
        &self,
    ) -> ::core::result::Result<NMFeagiSimpleDataCorticalProperties<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError> {
        use ::feagi_basis::quantization::quantizable::QuantizedDecimalTrait;

        Ok(NMFeagiSimpleDataCorticalProperties {
            refractory_period: QuantizedDecimalTrait::try_to_quantization(self.refractory_period)?,
            firing_threshold: QuantizedDecimalTrait::try_to_quantization(self.firing_threshold)?,
            postsynaptic_current: QuantizedDecimalTrait::try_to_quantization(self.postsynaptic_current)?,
            postsynaptic_current_max: QuantizedDecimalTrait::try_to_quantization(self.postsynaptic_current_max)?,
        })
    }
}

impl<NM: NMFeagiSimpleTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataPropertiesSpecification<NM>
    for NMFeagiSimpleDataCorticalProperties<NM>
{
}

//endregion

//region Cortical internal

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiSimpleDataCorticalInternal<NM: NMFeagiSimpleTrait> {
    #[serde(skip)]
    _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFeagiSimpleTrait> NMFeagiSimpleDataCorticalInternal<NM> {
    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFeagiSimpleTrait>(
        &self,
    ) -> ::core::result::Result<NMFeagiSimpleDataCorticalInternal<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError> {
        Ok(Default::default())
    }
}

impl<NM: NMFeagiSimpleTrait> Default for NMFeagiSimpleDataCorticalInternal<NM> {
    fn default() -> Self {
        Self {
            _p: core::marker::PhantomData,
        }
    }
}

impl<NM: NMFeagiSimpleTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataInternalSpecification<NM>
    for NMFeagiSimpleDataCorticalInternal<NM>
{
}

//endregion

//region Cortical Scratch

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiSimpleDataCorticalScratch<NM: NMFeagiSimpleTrait> {
    #[serde(skip)]
    _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFeagiSimpleTrait> NMFeagiSimpleDataCorticalScratch<NM> {
    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFeagiSimpleTrait>(
        &self,
    ) -> ::core::result::Result<NMFeagiSimpleDataCorticalScratch<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError> {
        Ok(Default::default())
    }
}

impl<NM: NMFeagiSimpleTrait> Default for NMFeagiSimpleDataCorticalScratch<NM> {
    fn default() -> Self {
        Self {
            _p: core::marker::PhantomData,
        }
    }
}

impl<NM: NMFeagiSimpleTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataScratchSpecification<NM>
    for NMFeagiSimpleDataCorticalScratch<NM>
{
}

//endregion

//region Cortical Variance

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiSimpleDataCorticalVariance<NM: NMFeagiSimpleTrait> {
    #[serde(skip)]
    _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFeagiSimpleTrait> Default for NMFeagiSimpleDataCorticalVariance<NM> {
    fn default() -> Self {
        Self {
            _p: core::marker::PhantomData,
        }
    }
}

impl<NM: NMFeagiSimpleTrait> NMFeagiSimpleDataCorticalVariance<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData,
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFeagiSimpleTrait>(
        &self,
    ) -> ::core::result::Result<NMFeagiSimpleDataCorticalVariance<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError> {
        Ok(NMFeagiSimpleDataCorticalVariance::new())
    }
}

//endregion

//region Neuron Internal

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiSimpleDataNeuronInternal<NM: NMFeagiSimpleTrait> {
    #[doc = " How many more bursts the neuron is refractory for"]
    pub refractory_countdown: wrapped::Period<NM::Period>,
}

impl<NM: NMFeagiSimpleTrait> NMFeagiSimpleDataNeuronInternal<NM> {
    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFeagiSimpleTrait>(
        &self,
    ) -> ::core::result::Result<NMFeagiSimpleDataNeuronInternal<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError> {
        use ::feagi_basis::quantization::quantizable::QuantizedDecimalTrait;
        Ok(NMFeagiSimpleDataNeuronInternal {
            refractory_countdown: QuantizedDecimalTrait::try_to_quantization(self.refractory_countdown)?,
        })
    }
}

impl<NM: NMFeagiSimpleTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronDataInternalSpecification<NM>
    for NMFeagiSimpleDataNeuronInternal<NM>
{
}

//endregion

//region Neuron Scratch

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiSimpleDataNeuronScratch<NM: NMFeagiSimpleTrait> {
    #[serde(skip)]
    _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFeagiSimpleTrait> NMFeagiSimpleDataNeuronScratch<NM> {
    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFeagiSimpleTrait>(
        &self,
    ) -> ::core::result::Result<NMFeagiSimpleDataNeuronScratch<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError> {
        Ok(Default::default())
    }
}

impl<NM: NMFeagiSimpleTrait> Default for NMFeagiSimpleDataNeuronScratch<NM> {
    fn default() -> Self {
        Self {
            _p: core::marker::PhantomData,
        }
    }
}

impl<NM: NMFeagiSimpleTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronDataScratchSpecification<NM>
    for NMFeagiSimpleDataNeuronScratch<NM>
{
}

//endregion

//region Neuron Variance

#[derive(Clone, PartialEq, Debug)]
pub struct NMFeagiSimpleDataNeuronVariance<NM: NMFeagiSimpleTrait> {
    #[doc = r" Public so that this struct may be written out with its fields named"]
    _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFeagiSimpleTrait> NMFeagiSimpleDataNeuronVariance<NM> {
    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFeagiSimpleTrait>(
        &self,
    ) -> ::core::result::Result<NMFeagiSimpleDataNeuronVariance<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError> {
        Ok(Default::default())
    }
}

impl<NM: NMFeagiSimpleTrait> Default for NMFeagiSimpleDataNeuronVariance<NM> {
    fn default() -> Self {
        Self {
            _p: core::marker::PhantomData,
        }
    }
}

impl<NM: NMFeagiSimpleTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronDataVarianceSpecification<NM>
    for NMFeagiSimpleDataNeuronVariance<NM>
{
}

//endregion
