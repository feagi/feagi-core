use feagi_basis::create_wrapped_quantized_decimal;
use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataVarianceSpecification;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutData, NeuronLayoutOf};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
use crate::common_systems::neurogenesis_generator_uniform_defaults::NeurogenesisGeneratorUniformDefault;
use crate::neuron_models::family::NeuronModel;

create_wrapped_quantized_decimal!(
    /// For handling timing
    pub Period
);

/// Quantization For the Feagi Simple type
pub trait NMFeagiSimpleTrait: NeuronModel<
    NeuronLayoutTag=NeuronLayoutTagVoxel,
    CorticalDataProperties=NMFeagiSimpleDataCorticalProperties<Self>,
    CorticalDataInternal=NMFeagiSimpleDataCorticalInternal<Self>,
    CorticalDataScratch=NMFeagiSimpleDataCorticalScratch<Self>,
    CorticalDataVariance=NMFeagiSimpleDataCorticalVariance<Self>,
    NeuronDataInternal=NMFeagiSimpleDataNeuronInternal<Self>,
    NeuronDataScratch=NMFeagiSimpleDataNeuronScratch<Self>,
    NeuronDataVariance=NMFeagiSimpleDataNeuronVariance<Self>
>
{
    #[doc = " For handling timing"]
    type Period: ::feagi_basis::quantization::quantizable::QuantizedDecimalUnwrappedTrait;
}

//region Quant levels

#[derive(Clone, Copy, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiSimpleQuantDefaultQuant;

impl NMFeagiSimpleTrait for NMFeagiSimpleQuantDefaultQuant {
    type Period = f32;
}

impl NeuronModel for NMFeagiSimpleQuantDefaultQuant {
    type NeuronLayoutTag = NeuronLayoutTagVoxel;

    type NeurogenesisScratchVarianceGeneratorType<FIQ: FeagiIndexQuantization> = 
        super::neurogenesis::FeagiSimpleNeurogenesisDefaults<FIQ, Self>;

    type NeurogenesisByGeneratorType<FIQ: FeagiIndexQuantization> = 
        NeurogenesisGeneratorUniformDefault<FIQ, Self>;

    type CorticalDataProperties = NMFeagiSimpleDataCorticalProperties<Self>;
    type CorticalDataInternal = NMFeagiSimpleDataCorticalInternal<Self>;
    type CorticalDataScratch = NMFeagiSimpleDataCorticalScratch<Self>;
    type CorticalDataVariance = NMFeagiSimpleDataCorticalVariance<Self>;
    type NeuronDataInternal = NMFeagiSimpleDataNeuronInternal<Self>;
    type NeuronDataScratch = NMFeagiSimpleDataNeuronScratch<Self>;
    type NeuronDataVariance = NMFeagiSimpleDataNeuronVariance<Self>;
}

impl MembranePotentialQuantization for NMFeagiSimpleQuantDefaultQuant
{
    type MembranePotentialQuant = f32;
}

//endregion

//region Cortical Properties

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiSimpleDataCorticalProperties<NM: NMFeagiSimpleTrait> {

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

impl<NM: NMFeagiSimpleTrait> NMFeagiSimpleDataCorticalProperties<NM> {
    pub const fn new(
        refractory_period: Period<NM::Period>,
        firing_threshold: NM::MembranePotentialQuant,
        postsynaptic_current: NM::MembranePotentialQuant,
        postsynaptic_current_max: NM::MembranePotentialQuant
    ) -> Self {
        Self {
            refractory_period,
            firing_threshold,
            postsynaptic_current,
            postsynaptic_current_max,
            _p: ::core::marker::PhantomData
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFeagiSimpleTrait>(&self)
        -> ::core::result::Result<
            NMFeagiSimpleDataCorticalProperties<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        use ::feagi_basis::quantization::quantizable::QuantizedDecimalTrait;
        Ok(
            NMFeagiSimpleDataCorticalProperties::new(
                QuantizedDecimalTrait::try_to_quantization(self.refractory_period)?,
                QuantizedDecimalTrait::try_to_quantization(self.firing_threshold)?,
                QuantizedDecimalTrait::try_to_quantization(self.postsynaptic_current)?,
                QuantizedDecimalTrait::try_to_quantization(self.postsynaptic_current_max)?
            )
        )
    }
}

impl<NM: NMFeagiSimpleTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataPropertiesSpecification<NM>
for NMFeagiSimpleDataCorticalProperties<NM> {}

//endregion


//region Cortical internal

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiSimpleDataCorticalInternal<NM: NMFeagiSimpleTrait> {

    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFeagiSimpleTrait> NMFeagiSimpleDataCorticalInternal<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFeagiSimpleTrait>(&self)
        -> ::core::result::Result<
            NMFeagiSimpleDataCorticalInternal<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        Ok(
            NMFeagiSimpleDataCorticalInternal::new()
        )
    }
}

impl<NM: NMFeagiSimpleTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataInternalSpecification<NM>
for NMFeagiSimpleDataCorticalInternal<NM> {}

//endregion

//region Cortical Scratch

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiSimpleDataCorticalScratch<NM: NMFeagiSimpleTrait> {

    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFeagiSimpleTrait> NMFeagiSimpleDataCorticalScratch<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFeagiSimpleTrait>(&self)
        -> ::core::result::Result<
            NMFeagiSimpleDataCorticalScratch<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        Ok(
            NMFeagiSimpleDataCorticalScratch::new()
        )
    }
}

impl<NM: NMFeagiSimpleTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataScratchSpecification<NM>
for NMFeagiSimpleDataCorticalScratch<NM> {}

//endregion

//region Cortical Variance

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiSimpleDataCorticalVariance<NM: NMFeagiSimpleTrait> {

    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFeagiSimpleTrait> NMFeagiSimpleDataCorticalVariance<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFeagiSimpleTrait>(&self)
        -> ::core::result::Result<
            NMFeagiSimpleDataCorticalVariance<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        Ok(
            NMFeagiSimpleDataCorticalVariance::new()
        )
    }
}

//endregion

//region Neuron Internal

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiSimpleDataNeuronInternal<NM: NMFeagiSimpleTrait> {

    #[doc = " How many more bursts the neuron is refractory for"]
    pub refractory_countdown: Period<NM::Period>,

    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFeagiSimpleTrait> NMFeagiSimpleDataNeuronInternal<NM> {
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
    pub fn requantize<To: NMFeagiSimpleTrait>(&self)
        -> ::core::result::Result<
            NMFeagiSimpleDataNeuronInternal<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        use ::feagi_basis::quantization::quantizable::QuantizedDecimalTrait;
        Ok(
            NMFeagiSimpleDataNeuronInternal::new(
                QuantizedDecimalTrait::try_to_quantization(self.refractory_countdown)?
            )
        )
    }
}

impl<NM: NMFeagiSimpleTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronDataInternalSpecification<NM>
for NMFeagiSimpleDataNeuronInternal<NM> {}

//endregion

//region Neuron Scratch

#[derive(Clone, PartialEq, Debug, ::serde::Serialize, ::serde::Deserialize)]
pub struct NMFeagiSimpleDataNeuronScratch<NM: NMFeagiSimpleTrait> {

    #[serde(skip)]
    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFeagiSimpleTrait> NMFeagiSimpleDataNeuronScratch<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFeagiSimpleTrait>(&self)
        -> ::core::result::Result<
            NMFeagiSimpleDataNeuronScratch<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        Ok(
            NMFeagiSimpleDataNeuronScratch::new()
        )
    }
}

impl<NM: NMFeagiSimpleTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronDataScratchSpecification<NM>
for NMFeagiSimpleDataNeuronScratch<NM> {}

//endregion

//region Neuron Variance

#[derive(Clone, PartialEq, Debug)]
pub struct NMFeagiSimpleDataNeuronVariance<NM: NMFeagiSimpleTrait> {

    #[doc = r" Public so that this struct may be written out with its fields named"]
    pub _p: ::core::marker::PhantomData<NM>,
}

impl<NM: NMFeagiSimpleTrait> NMFeagiSimpleDataNeuronVariance<NM> {
    pub const fn new() -> Self {
        Self {
            _p: ::core::marker::PhantomData
        }
    }

    #[doc = r" Converts to the quantization of `To`. Returns an error if a decimal becomes"]
    #[doc = r" infinite or NaN"]
    pub fn requantize<To: NMFeagiSimpleTrait>(&self)
        -> ::core::result::Result<
            NMFeagiSimpleDataNeuronVariance<To>,
            ::feagi_basis::quantization::quantizable::DataValueQuantizationError
        >
    {
        Ok(
            NMFeagiSimpleDataNeuronVariance::new()
        )
    }
}

impl<NM: NMFeagiSimpleTrait> feagi_connectome_neural::neural_basis::neuron_model::model_data::NeuronDataVarianceSpecification<NM>
for NMFeagiSimpleDataNeuronVariance<NM> {}

//endregion


impl<NM: NMFeagiSimpleTrait> CorticalDataVarianceSpecification<NM> for NMFeagiSimpleDataCorticalVariance<NM> {
    fn neuron_variance_at<FIQ: FeagiIndexQuantization>(
        &self,
        layout: &NeuronLayoutOf<NM, FIQ>,
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> NMFeagiSimpleDataNeuronVariance<NM> {
        let coordinates = layout.get_neuron_layout_context(index);
        let [x, y, z, _d] = coordinates.as_slice();
        NMFeagiSimpleDataNeuronVariance::new()
    }
}


/*
use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use super::neurogenesis::FeagiSimpleNeurogenesisDefaults;
use crate::neuron_models::defaults_generator::NeurogenesisDefaultsGenerator;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayoutData, NeuronLayoutOf};
use feagi_connectome_neural::neural_basis::neuron_model::model_data::{CorticalDataInternalSpecification, CorticalDataPropertiesSpecification, CorticalDataScratchSpecification, CorticalDataVarianceSpecification, NeuronDataInternalSpecification, NeuronDataScratchSpecification, NeuronDataVarianceSpecification};
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::{FeagiIndexQuantization, MembranePotentialQuantization};
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::voxel::NeuronLayoutTagVoxel;
use feagi_connectome_proc::{define_cortical_dynamics_quantization_template, expand_cortical_dynamics_quantization};

define_cortical_dynamics_quantization_template! {
    /// Quantization For the Feagi Simple type
    FeagiSimple

    NeuronLayout: NeuronLayoutTagVoxel

    // How the scratch and variance data of a neuron is generated
    NeurogenesisDefaults: FeagiSimpleNeurogenesisDefaults<FIQ, Self>

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
            /// Current post-synaptic
            postsynaptic_current: MembranePotentialQuant,
            /// max postsynaptic current
            postsynaptic_current_max: MembranePotentialQuant,
        }

        CorticalDataVariance: {

        }

        NeuronDataInternal: {
            /// How many more bursts the neuron is refractory for
            refractory_countdown: Period,
        }

        NeuronDataVariance: {
        }
    }
}

cortical_dynamics_quantization_feagi_simple_template!(expand_cortical_dynamics_quantization);

 */
