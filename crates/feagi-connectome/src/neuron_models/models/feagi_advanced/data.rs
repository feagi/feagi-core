use crate::neuron_models::models::feagi_advanced::NMFeagiAdvancedTrait;
use feagi_basis::create_wrapped_quantized_decimal;
use feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};

create_wrapped_quantized_decimal!(
    /// For handling timing
    pub Period
);

//region Cortical Properties

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiAdvancedDataCorticalProperties<NM: NMFeagiAdvancedTrait> {

    #[doc = " How long till a neuron of the cortical area can fire again"]
    pub refractory_period: Period<NM::Period>,

    #[doc = " At what threshold do we fire"]
    pub firing_threshold: NM::MembranePotentialQuant,

    #[doc = " Amount of leak"]
    pub leak_coefficient: NM::MembranePotentialQuant,

    #[doc = " How variable a leak is"]
    pub leak_variability: NM::MembranePotentialQuant,

    #[doc = " Current post-synaptic"]
    pub postsynaptic_current: NM::MembranePotentialQuant,

    #[doc = " max postsynaptic current"]
    pub postsynaptic_current_max: NM::MembranePotentialQuant,

    #[doc = " neuron degeneration TODO shouldnt this be in memory?"]
    pub degeneration: NM::MembranePotentialQuant,

    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFeagiAdvancedTrait> NMFeagiAdvancedDataCorticalProperties<NM> {
    pub const fn new(
        refractory_period: Period<NM::Period>,
        firing_threshold: NM::MembranePotentialQuant,
        leak_coefficient: NM::MembranePotentialQuant,
        leak_variability: NM::MembranePotentialQuant,
        postsynaptic_current: NM::MembranePotentialQuant,
        postsynaptic_current_max: NM::MembranePotentialQuant,
        degeneration: NM::MembranePotentialQuant
    ) -> Self {
        Self {
            refractory_period,
            firing_threshold,
            leak_coefficient,
            leak_variability,
            postsynaptic_current,
            postsynaptic_current_max,
            degeneration,
            _p: ::core::marker::PhantomData
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFeagiAdvancedTrait>(&self)
        -> ::core::result::Result<
            NMFeagiAdvancedDataCorticalProperties<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        use ::feagi_basis::quantization::quantizable::QuantizedDecimalTrait;
        Ok(
            NMFeagiAdvancedDataCorticalProperties::new(
                QuantizedDecimalTrait::try_to_quantization(self.refractory_period)?,
                QuantizedDecimalTrait::try_to_quantization(self.firing_threshold)?,
                QuantizedDecimalTrait::try_to_quantization(self.leak_coefficient)?,
                QuantizedDecimalTrait::try_to_quantization(self.leak_variability)?,
                QuantizedDecimalTrait::try_to_quantization(self.postsynaptic_current)?,
                QuantizedDecimalTrait::try_to_quantization(self.postsynaptic_current_max)?,
                QuantizedDecimalTrait::try_to_quantization(self.degeneration)?
            )
        )
    }
}

impl<NM: NMFeagiAdvancedTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataPropertiesSpecification<NM>
for NMFeagiAdvancedDataCorticalProperties<NM> {}

//endregion

//region Cortical internal

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiAdvancedDataCorticalInternal<NM: NMFeagiAdvancedTrait> {

    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFeagiAdvancedTrait> NMFeagiAdvancedDataCorticalInternal<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFeagiAdvancedTrait>(&self)
        -> ::core::result::Result<
            NMFeagiAdvancedDataCorticalInternal<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        Ok(
            NMFeagiAdvancedDataCorticalInternal::new()
        )
    }
}

impl<NM: NMFeagiAdvancedTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataInternalSpecification<NM>
for NMFeagiAdvancedDataCorticalInternal<NM> {}

//endregion

//region Cortical Scratch

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiAdvancedDataCorticalScratch<NM: NMFeagiAdvancedTrait> {

    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFeagiAdvancedTrait> NMFeagiAdvancedDataCorticalScratch<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFeagiAdvancedTrait>(&self)
        -> ::core::result::Result<
            NMFeagiAdvancedDataCorticalScratch<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        Ok(
            NMFeagiAdvancedDataCorticalScratch::new()
        )
    }
}

impl<NM: NMFeagiAdvancedTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataScratchSpecification<NM>
for NMFeagiAdvancedDataCorticalScratch<NM> {}

//endregion

//region Cortical Variance

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiAdvancedDataCorticalVariance<NM: NMFeagiAdvancedTrait> {

    #[doc = " How much fire threshold increases in a cortical area on the X direction"]
    pub firing_threshold_increment_x: NM::MembranePotentialQuant,

    #[doc = " How much fire threshold increases in a cortical area on the Y direction"]
    pub firing_threshold_increment_y: NM::MembranePotentialQuant,

    #[doc = " How much fire threshold increases in a cortical area on the Z direction"]
    pub firing_threshold_increment_z: NM::MembranePotentialQuant,

    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFeagiAdvancedTrait> NMFeagiAdvancedDataCorticalVariance<NM> {
    pub const fn new(
        firing_threshold_increment_x: NM::MembranePotentialQuant,
        firing_threshold_increment_y: NM::MembranePotentialQuant,
        firing_threshold_increment_z: NM::MembranePotentialQuant
    ) -> Self {
        Self {
            firing_threshold_increment_x,
            firing_threshold_increment_y,
            firing_threshold_increment_z,
            _p: ::core::marker::PhantomData
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFeagiAdvancedTrait>(&self)
        -> ::core::result::Result<
            NMFeagiAdvancedDataCorticalVariance<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        use ::feagi_basis::quantization::quantizable::QuantizedDecimalTrait;
        Ok(
            NMFeagiAdvancedDataCorticalVariance::new(
                QuantizedDecimalTrait::try_to_quantization(self.firing_threshold_increment_x)?,
                QuantizedDecimalTrait::try_to_quantization(self.firing_threshold_increment_y)?,
                QuantizedDecimalTrait::try_to_quantization(self.firing_threshold_increment_z)?
            )
        )
    }
}

//endregion

//region Neuron Internal

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiAdvancedDataNeuronInternal<NM: NMFeagiAdvancedTrait> {

    #[doc = " How many more bursts the neuron is refractory for"]
    pub refractory_countdown: Period<NM::Period>,

    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFeagiAdvancedTrait> NMFeagiAdvancedDataNeuronInternal<NM> {
    pub const fn new(
        refractory_countdown: Period<NM::Period>
    ) -> Self {
        Self {
            refractory_countdown,
            _p: ::core::marker::PhantomData
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFeagiAdvancedTrait>(&self)
        -> ::core::result::Result<
            NMFeagiAdvancedDataNeuronInternal<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        use ::feagi_basis::quantization::quantizable::QuantizedDecimalTrait;
        Ok(
            NMFeagiAdvancedDataNeuronInternal::new(
                QuantizedDecimalTrait::try_to_quantization(self.refractory_countdown)?
            )
        )
    }
}

impl<NM: NMFeagiAdvancedTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronDataInternalSpecification<NM>
for NMFeagiAdvancedDataNeuronInternal<NM> {}

//endregion

//region Neuron Scratch

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiAdvancedDataNeuronScratch<NM: NMFeagiAdvancedTrait> {

    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFeagiAdvancedTrait> NMFeagiAdvancedDataNeuronScratch<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFeagiAdvancedTrait>(&self)
        -> ::core::result::Result<
            NMFeagiAdvancedDataNeuronScratch<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        Ok(
            NMFeagiAdvancedDataNeuronScratch::new()
        )
    }
}

impl<NM: NMFeagiAdvancedTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronDataScratchSpecification<NM>
for NMFeagiAdvancedDataNeuronScratch<NM> {}

//endregion

//region Neuron Variance

#[derive(Clone, PartialEq, Debug)]
pub struct NMFeagiAdvancedDataNeuronVariance<NM: NMFeagiAdvancedTrait> {

    #[doc = " Added to the firing threshold of the cortical area, for this neuron"]
    pub firing_threshold_offset: NM::MembranePotentialQuant,

    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFeagiAdvancedTrait> NMFeagiAdvancedDataNeuronVariance<NM> {
    pub const fn new(
        firing_threshold_offset: NM::MembranePotentialQuant
    ) -> Self {
        Self {
            firing_threshold_offset,
            _p: ::core::marker::PhantomData
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFeagiAdvancedTrait>(&self)
        -> ::core::result::Result<
            NMFeagiAdvancedDataNeuronVariance<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        use ::feagi_basis::quantization::quantizable::QuantizedDecimalTrait;
        Ok(
            NMFeagiAdvancedDataNeuronVariance::new(
                QuantizedDecimalTrait::try_to_quantization(self.firing_threshold_offset)?
            )
        )
    }
}

impl<NM: NMFeagiAdvancedTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronDataVarianceSpecification<NM>
for NMFeagiAdvancedDataNeuronVariance<NM> {}

//endregion



/*
use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use feagi_basis::prelude::QuantizedDecimalTrait;
use super::neurogenesis::FeagiAdvancedNeurogenesisDefaults;
use crate::neuron_models::defaults_generator::NeurogenesisDefaultsGenerator;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutData, NeuronLayoutOf};
use feagi_connectome_neural::neural_basis::neuron_model::model_data::{CorticalDataInternalSpecification, CorticalDataPropertiesSpecification, CorticalDataScratchSpecification, CorticalDataVarianceSpecification, NeuronDataInternalSpecification, NeuronDataScratchSpecification, NeuronDataVarianceSpecification};
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_proc::{define_cortical_dynamics_quantization_template, expand_cortical_dynamics_quantization};





define_cortical_dynamics_quantization_template! {
    /// Quantization For the Feagi Advanced type
    FeagiAdvanced

    NeuronLayout: NeuronLayoutTagVoxel

    // How the scratch and variance data of a neuron is generated
    NeurogenesisDefaults: FeagiAdvancedNeurogenesisDefaults<FIQ, Self>

    // How the neurons of a new cortical area are generated, every neuron starting from the same values
    NeurogenesisGenerator: NeurogenesisDefaultsGenerator<FIQ, Self>

    Quantizables: {
        /// For handling timing
        Period: QDECIMAL,
    }

    QuantizationLevels: {
        DefaultQuant: {
            MembranePotentialQuant: f32,
            Period: f32,
        }
    }

    DataSpecifications: {
        CorticalDataProperties: {
            /// How long till a neuron of the cortical area can fire again
            refractory_period: Period,
            /// At what threshold do we fire
            firing_threshold: MembranePotentialQuant,
            /// Amount of leak
            leak_coefficient: MembranePotentialQuant,
            /// How variable a leak is
            leak_variability: MembranePotentialQuant,
            /// Current post-synaptic
            postsynaptic_current: MembranePotentialQuant,
            /// max postsynaptic current
            postsynaptic_current_max: MembranePotentialQuant,
            /// neuron degeneration TODO shouldnt this be in memory?
            degeneration: MembranePotentialQuant,
        }

        CorticalDataVariance: {
            /// How much fire threshold increases in a cortical area on the X direction
            firing_threshold_increment_x: MembranePotentialQuant,
            /// How much fire threshold increases in a cortical area on the Y direction
            firing_threshold_increment_y: MembranePotentialQuant,
            /// How much fire threshold increases in a cortical area on the Z direction
            firing_threshold_increment_z: MembranePotentialQuant,
        }

        NeuronDataInternal: {
            /// How many more bursts the neuron is refractory for
            refractory_countdown: Period,
        }

        NeuronDataVariance: {
            /// Added to the firing threshold of the cortical area, for this neuron
            firing_threshold_offset: MembranePotentialQuant,
        }
    }
}

cortical_dynamics_quantization_feagi_advanced_template!(expand_cortical_dynamics_quantization);





 */