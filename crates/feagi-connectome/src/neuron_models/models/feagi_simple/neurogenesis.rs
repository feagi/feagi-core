//! How the scratch and variance data of `FeagiSimple` neurons is generated: the
//! `NeuronModel::NeurogenesisDefaults` of every quantization level of the model. Its
//! `NeuronModel::NeurogenesisGenerator` is the shared `NeurogenesisDefaultsGenerator`
// TODO these are stubs, fill them in

use core::marker::PhantomData;
use crate::neuron_models::models::feagi_simple::quantizations::{NMFeagiSimpleNeurogenesisGeneratorQuanted, NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted, NMFeagiSimpleNeurogenesisStorageQuanted, NMFeagiSimpleQuantKind, NMFeagiSimpleQuantStandard32Bit};
use crate::neuron_models::models::feagi_simple::NMFeagiSimpleTrait;
use crate::neuron_models::models::feagi_simple::{NMFeagiSimpleDataCorticalVariance, NMFeagiSimpleDataNeuronVariance};
use crate::neuron_models::models::{NeuronModelEnumConversionError, NeurogenesisByGeneratorTypeOf, NeurogenesisFromStorageOf};
use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataVarianceSpecification;
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::NeurogenesisScratchVarianceGenerator;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::NeuronLayoutOf;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::from_storage_or_by_generator::NeurogenesisFromStorageOrByGeneratorOf;
use crate::neuron_models::neurogenesis_generator_uniform_defaults::NeurogenesisGeneratorUniformDefault;

//region Generator

// NOTE: we are using NeurogenesisGeneratorUniformDefault, not a specific impl for this model

pub type NMFeagiSimpleNeuroGenesisGenerator<FIQ: FeagiIndexQuantization, NM: NMFeagiSimpleTrait> = NeurogenesisGeneratorUniformDefault<FIQ, NM>;


//region Impls

impl<FIQ> From<NeurogenesisByGeneratorTypeOf<FIQ, NMFeagiSimpleQuantStandard32Bit>> for NMFeagiSimpleNeurogenesisGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NeurogenesisByGeneratorTypeOf<FIQ, NMFeagiSimpleQuantStandard32Bit>) -> Self {
        NMFeagiSimpleNeurogenesisGeneratorQuanted::Standard32Bit(value)
    }
}

impl<FIQ> TryFrom<NMFeagiSimpleNeurogenesisGeneratorQuanted<FIQ>> for NeurogenesisByGeneratorTypeOf<FIQ, NMFeagiSimpleQuantStandard32Bit>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFeagiSimpleQuantKind>;

    fn try_from(value: NMFeagiSimpleNeurogenesisGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {

        match value {
            NMFeagiSimpleNeurogenesisGeneratorQuanted::Standard32Bit(v) => {
                Ok(v)
            },
        }
    }
}

//endregion

//endregion

//region Defaults

/// Generates the scratch and variance data of `FeagiSimple` neurons
#[derive(Clone, Debug)]
pub struct NMFeagiSimpleScratchVarianceGenerator<FIQ, NM>(PhantomData<fn() -> (FIQ, NM)>);

impl<FIQ, NM> NeurogenesisScratchVarianceGenerator<FIQ, NM> for NMFeagiSimpleScratchVarianceGenerator<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NMFeagiSimpleTrait,
{
    fn static_get_cortical_scratch(
        _layout: &NeuronLayoutOf<NM, FIQ>,
        _cortical_properties: &NM::CorticalDataProperties,
        _cortical_internal: &NM::CorticalDataInternal,
    ) -> NM::CorticalDataScratch {
        NM::CorticalDataScratch::default() // no fields
    }

    fn static_get_neuron_scratch(
        _index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        _layout: &NeuronLayoutOf<NM, FIQ>,
        _internal_neuron_data: &NM::NeuronDataInternal,
    ) -> NM::NeuronDataScratch {
        NM::NeuronDataScratch::default() // no fields
    }

    /// The variance the model's cortical variance gives the neuron
    fn static_get_neuron_variance(
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        layout: &NeuronLayoutOf<NM, FIQ>,
        variance: &NM::CorticalDataVariance,
    ) -> NM::NeuronDataVariance {
        variance.neuron_variance_at::<FIQ>(layout, index)
    }
}

impl<NM: NMFeagiSimpleTrait> CorticalDataVarianceSpecification<NM> for NMFeagiSimpleDataCorticalVariance<NM> {
    fn neuron_variance_at<FIQ: FeagiIndexQuantization>(
        &self,
        _layout: &NeuronLayoutOf<NM, FIQ>,
        _index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> NMFeagiSimpleDataNeuronVariance<NM> {
        NMFeagiSimpleDataNeuronVariance::default()
    }
}

//endregion

//region Storage and StorageOrGenerator Conversions

//region Storage

impl<FIQ> From<NeurogenesisFromStorageOf<FIQ, NMFeagiSimpleQuantStandard32Bit>> for NMFeagiSimpleNeurogenesisStorageQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NeurogenesisFromStorageOf<FIQ, NMFeagiSimpleQuantStandard32Bit>) -> Self {
        NMFeagiSimpleNeurogenesisStorageQuanted::Standard32Bit(value)
    }
}

impl<FIQ> TryFrom<NMFeagiSimpleNeurogenesisStorageQuanted<FIQ>> for NeurogenesisFromStorageOf<FIQ, NMFeagiSimpleQuantStandard32Bit>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFeagiSimpleQuantKind>;

    fn try_from(value: NMFeagiSimpleNeurogenesisStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFeagiSimpleNeurogenesisStorageQuanted::Standard32Bit(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFeagiSimpleQuantStandard32Bit::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

//endregion

//region Storage Or Generator

impl<FIQ> From<NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMFeagiSimpleQuantStandard32Bit>> for NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMFeagiSimpleQuantStandard32Bit>) -> Self {
        NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted::Standard32Bit(value)
    }
}

impl<FIQ> TryFrom<NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMFeagiSimpleQuantStandard32Bit>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFeagiSimpleQuantKind>;

    fn try_from(value: NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted::Standard32Bit(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFeagiSimpleQuantStandard32Bit::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

//endregion

//endregion
