//! How the scratch and variance data of `MemoryFireHashing` neurons is generated: the
//! `NeuronModel::NeurogenesisDefaults` of every quantization level of the model. Its
//! `NeuronModel::NeurogenesisGenerator` is the shared `NeurogenesisDefaultsGenerator`
// TODO these are stubs, fill them in

use crate::neuron_models::models::memory_fire_hashing::quantizations::{NMMemoryFireHashingNeurogenesisGeneratorQuanted, NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted, NMMemoryFireHashingNeurogenesisStorageQuanted, NMMemoryFireHashingQuantKind, NMMemoryFireHashingQuantStandard32Bit};
use crate::neuron_models::models::memory_fire_hashing::NMMemoryFireHashingTrait;
use crate::neuron_models::models::memory_fire_hashing::{NMMemoryFireHashingDataCorticalVariance, NMMemoryFireHashingDataNeuronVariance};
use crate::neuron_models::models::{NeuronModelEnumConversionError, NeurogenesisByGeneratorTypeOf, NeurogenesisFromStorageOf, NeurogenesisFromStorageOrByGeneratorOf};
use core::marker::PhantomData;
use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataVarianceSpecification;
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::NeurogenesisScratchVarianceGenerator;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::NeuronLayoutOf;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
//region Generator

// NOTE: we are using NeurogenesisGeneratorUniformDefault, not a specific impl for this model

//region Impls

impl<FIQ> From<NeurogenesisByGeneratorTypeOf<FIQ, NMMemoryFireHashingQuantStandard32Bit>> for NMMemoryFireHashingNeurogenesisGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NeurogenesisByGeneratorTypeOf<FIQ, NMMemoryFireHashingQuantStandard32Bit>) -> Self {
        NMMemoryFireHashingNeurogenesisGeneratorQuanted::Standard32Bit(value)
    }
}

impl<FIQ> TryFrom<NMMemoryFireHashingNeurogenesisGeneratorQuanted<FIQ>> for NeurogenesisByGeneratorTypeOf<FIQ, NMMemoryFireHashingQuantStandard32Bit>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMMemoryFireHashingQuantKind>;

    fn try_from(value: NMMemoryFireHashingNeurogenesisGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMMemoryFireHashingNeurogenesisGeneratorQuanted::Standard32Bit(v) => {
                Ok(v)
            },
        }
    }
}

//endregion

//endregion

//region Defaults

/// Generates the scratch and variance data of `MemoryFireHashing` neurons
#[derive(Clone, Debug)]
pub struct MemoryFireHashingNeurogenesisDefaults<FIQ, NM>(PhantomData<fn() -> (FIQ, NM)>);

impl<FIQ, NM> NeurogenesisScratchVarianceGenerator<FIQ, NM> for MemoryFireHashingNeurogenesisDefaults<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NMMemoryFireHashingTrait,
{
    fn static_get_cortical_scratch(
        _layout: &NeuronLayoutOf<NM, FIQ>,
        _cortical_properties: &NM::CorticalDataProperties,
        _cortical_internal: &NM::CorticalDataInternal,
    ) -> NM::CorticalDataScratch {
        NM::CorticalDataScratch::new() // no fields
    }

    fn static_get_neuron_scratch(
        _index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        _layout: &NeuronLayoutOf<NM, FIQ>,
        _internal_neuron_data: &NM::NeuronDataInternal,
    ) -> NM::NeuronDataScratch {
        NM::NeuronDataScratch::new() // no fields
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

impl<NM: NMMemoryFireHashingTrait> CorticalDataVarianceSpecification<NM> for NMMemoryFireHashingDataCorticalVariance<NM> {
    fn neuron_variance_at<FIQ: FeagiIndexQuantization>(
        &self,
        _layout: &NeuronLayoutOf<NM, FIQ>,
        _index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> NMMemoryFireHashingDataNeuronVariance<NM> {
        NMMemoryFireHashingDataNeuronVariance::new()
    }
}

//endregion

//region Storage and StorageOrGenerator Conversions

//region Storage

impl<FIQ> From<NeurogenesisFromStorageOf<FIQ, NMMemoryFireHashingQuantStandard32Bit>> for NMMemoryFireHashingNeurogenesisStorageQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NeurogenesisFromStorageOf<FIQ, NMMemoryFireHashingQuantStandard32Bit>) -> Self {
        NMMemoryFireHashingNeurogenesisStorageQuanted::Standard32Bit(value)
    }
}

impl<FIQ> TryFrom<NMMemoryFireHashingNeurogenesisStorageQuanted<FIQ>> for NeurogenesisFromStorageOf<FIQ, NMMemoryFireHashingQuantStandard32Bit>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMMemoryFireHashingQuantKind>;

    fn try_from(value: NMMemoryFireHashingNeurogenesisStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMMemoryFireHashingNeurogenesisStorageQuanted::Standard32Bit(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMMemoryFireHashingQuantStandard32Bit::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

//endregion

//region Storage Or Generator

impl<FIQ> From<NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMMemoryFireHashingQuantStandard32Bit>> for NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMMemoryFireHashingQuantStandard32Bit>) -> Self {
        NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted::Standard32Bit(value)
    }
}

impl<FIQ> TryFrom<NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMMemoryFireHashingQuantStandard32Bit>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMMemoryFireHashingQuantKind>;

    fn try_from(value: NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted::Standard32Bit(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMMemoryFireHashingQuantStandard32Bit::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

//endregion

//endregion
