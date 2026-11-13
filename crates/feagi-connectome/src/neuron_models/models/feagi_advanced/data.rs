use crate::neuron_models::models::feagi_advanced::NMFeagiAdvancedTrait;
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
pub struct NMFeagiAdvancedDataCorticalProperties<NM: NMFeagiAdvancedTrait> {
    #[doc = " How long till a neuron of the cortical area can fire again"]
    pub refractory_period: wrapped::Period<NM::Period>,

    #[doc = " At what threshold do we fire"]
    pub firing_threshold: CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,

    #[doc = " Amount of leak"]
    pub leak_coefficient: CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,

    #[doc = " How variable a leak is"]
    pub leak_variability: CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,

    #[doc = " Current post-synaptic"]
    pub postsynaptic_current: CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,

    #[doc = " max postsynaptic current"]
    pub postsynaptic_current_max: CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,

    #[doc = " neuron degeneration TODO shouldnt this be in memory?"]
    pub degeneration: CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,
}

impl<NM: NMFeagiAdvancedTrait> NMFeagiAdvancedDataCorticalProperties<NM> {
    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFeagiAdvancedTrait>(
        &self,
    ) -> ::core::result::Result<NMFeagiAdvancedDataCorticalProperties<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError> {
        use ::feagi_basis::quantization::quantizable::QuantizedDecimalTrait;
        Ok(NMFeagiAdvancedDataCorticalProperties {
            refractory_period: QuantizedDecimalTrait::try_to_quantization(self.refractory_period)?,
            firing_threshold: QuantizedDecimalTrait::try_to_quantization(self.firing_threshold)?,
            leak_coefficient: QuantizedDecimalTrait::try_to_quantization(self.leak_coefficient)?,
            leak_variability: QuantizedDecimalTrait::try_to_quantization(self.leak_variability)?,
            postsynaptic_current: QuantizedDecimalTrait::try_to_quantization(self.postsynaptic_current)?,
            postsynaptic_current_max: QuantizedDecimalTrait::try_to_quantization(self.postsynaptic_current_max)?,
            degeneration: QuantizedDecimalTrait::try_to_quantization(self.degeneration)?,
        })
    }
}

impl<NM: NMFeagiAdvancedTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataPropertiesSpecification<NM>
    for NMFeagiAdvancedDataCorticalProperties<NM>
{
}

//endregion

//region Cortical internal

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiAdvancedDataCorticalInternal<NM: NMFeagiAdvancedTrait> {
    #[serde(skip)]
    _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFeagiAdvancedTrait> NMFeagiAdvancedDataCorticalInternal<NM> {
    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFeagiAdvancedTrait>(
        &self,
    ) -> ::core::result::Result<NMFeagiAdvancedDataCorticalInternal<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError> {
        Ok(NMFeagiAdvancedDataCorticalInternal::default())
    }
}

impl<NM: NMFeagiAdvancedTrait> Default for NMFeagiAdvancedDataCorticalInternal<NM> {
    fn default() -> Self {
        Self { _p: Default::default() }
    }
}

impl<NM: NMFeagiAdvancedTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataInternalSpecification<NM>
    for NMFeagiAdvancedDataCorticalInternal<NM>
{
}

//endregion

//region Cortical Scratch

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiAdvancedDataCorticalScratch<NM: NMFeagiAdvancedTrait> {
    #[serde(skip)]
    _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFeagiAdvancedTrait> NMFeagiAdvancedDataCorticalScratch<NM> {
    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFeagiAdvancedTrait>(
        &self,
    ) -> ::core::result::Result<NMFeagiAdvancedDataCorticalScratch<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError> {
        Ok(NMFeagiAdvancedDataCorticalScratch::default())
    }
}

impl<NM: NMFeagiAdvancedTrait> Default for NMFeagiAdvancedDataCorticalScratch<NM> {
    fn default() -> Self {
        Self { _p: Default::default() }
    }
}

impl<NM: NMFeagiAdvancedTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataScratchSpecification<NM>
    for NMFeagiAdvancedDataCorticalScratch<NM>
{
}

//endregion

//region Cortical Variance

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiAdvancedDataCorticalVariance<NM: NMFeagiAdvancedTrait> {
    #[doc = " How much fire threshold increases in a cortical area on the X direction"]
    pub firing_threshold_increment_x: CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,

    #[doc = " How much fire threshold increases in a cortical area on the Y direction"]
    pub firing_threshold_increment_y: CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,

    #[doc = " How much fire threshold increases in a cortical area on the Z direction"]
    pub firing_threshold_increment_z: CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,
}

impl<NM: NMFeagiAdvancedTrait> NMFeagiAdvancedDataCorticalVariance<NM> {
    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFeagiAdvancedTrait>(
        &self,
    ) -> ::core::result::Result<NMFeagiAdvancedDataCorticalVariance<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError> {
        use ::feagi_basis::quantization::quantizable::QuantizedDecimalTrait;
        Ok(NMFeagiAdvancedDataCorticalVariance {
            firing_threshold_increment_x: QuantizedDecimalTrait::try_to_quantization(self.firing_threshold_increment_x)?,
            firing_threshold_increment_y: QuantizedDecimalTrait::try_to_quantization(self.firing_threshold_increment_y)?,
            firing_threshold_increment_z: QuantizedDecimalTrait::try_to_quantization(self.firing_threshold_increment_z)?,
        })
    }
}

//endregion

//region Neuron Internal

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiAdvancedDataNeuronInternal<NM: NMFeagiAdvancedTrait> {
    #[doc = " How many more bursts the neuron is refractory for"]
    pub refractory_countdown: wrapped::Period<NM::Period>,
}

impl<NM: NMFeagiAdvancedTrait> NMFeagiAdvancedDataNeuronInternal<NM> {
    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFeagiAdvancedTrait>(
        &self,
    ) -> ::core::result::Result<NMFeagiAdvancedDataNeuronInternal<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError> {
        use ::feagi_basis::quantization::quantizable::QuantizedDecimalTrait;
        Ok(NMFeagiAdvancedDataNeuronInternal {
            refractory_countdown: QuantizedDecimalTrait::try_to_quantization(self.refractory_countdown)?,
        })
    }
}

impl<NM: NMFeagiAdvancedTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronDataInternalSpecification<NM>
    for NMFeagiAdvancedDataNeuronInternal<NM>
{
}

//endregion

//region Neuron Scratch

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiAdvancedDataNeuronScratch<NM: NMFeagiAdvancedTrait> {
    #[serde(skip)]
    _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFeagiAdvancedTrait> NMFeagiAdvancedDataNeuronScratch<NM> {
    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFeagiAdvancedTrait>(
        &self,
    ) -> ::core::result::Result<NMFeagiAdvancedDataNeuronScratch<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError> {
        Ok(NMFeagiAdvancedDataNeuronScratch::default())
    }
}

impl<NM: NMFeagiAdvancedTrait> Default for NMFeagiAdvancedDataNeuronScratch<NM> {
    fn default() -> Self {
        Self { _p: Default::default() }
    }
}

impl<NM: NMFeagiAdvancedTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronDataScratchSpecification<NM>
    for NMFeagiAdvancedDataNeuronScratch<NM>
{
}

//endregion

//region Neuron Variance

#[derive(Clone, PartialEq, Debug)]
pub struct NMFeagiAdvancedDataNeuronVariance<NM: NMFeagiAdvancedTrait> {
    #[doc = " Added to the firing threshold of the cortical area, for this neuron"]
    pub firing_threshold_offset: CorticalAreaNeuronPotential<NM::MembranePotentialQuant>,
}

impl<NM: NMFeagiAdvancedTrait> NMFeagiAdvancedDataNeuronVariance<NM> {
    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFeagiAdvancedTrait>(
        &self,
    ) -> ::core::result::Result<NMFeagiAdvancedDataNeuronVariance<To>, ::feagi_basis::quantization::quantizable::DataValueQuantizationError> {
        use ::feagi_basis::quantization::quantizable::QuantizedDecimalTrait;
        Ok(NMFeagiAdvancedDataNeuronVariance {
            firing_threshold_offset: QuantizedDecimalTrait::try_to_quantization(self.firing_threshold_offset)?,
        })
    }
}

impl<NM: NMFeagiAdvancedTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronDataVarianceSpecification<NM>
    for NMFeagiAdvancedDataNeuronVariance<NM>
{
}

//endregion
