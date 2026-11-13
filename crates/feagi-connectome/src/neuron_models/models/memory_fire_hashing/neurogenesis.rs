//! How the scratch and variance data of `MemoryFireHashing` neurons is generated: the
//! `NeuronModel::NeurogenesisDefaults` of every quantization level of the model. Its
//! `NeuronModel::NeurogenesisByGenerator` is the shared `NeurogenesisGeneratorUniformDefault`
// TODO these are stubs, fill them in

use crate::feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::scratch_variance_defaults::NeurogenesisScratchVarianceDefaults;
use crate::neuron_models::models::memory_fire_hashing::NMMemoryFireHashingTrait;
use crate::neuron_models::models::memory_fire_hashing::{NMMemoryFireHashingDataCorticalVariance, NMMemoryFireHashingDataNeuronVariance};
use core::marker::PhantomData;
use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataVarianceSpecification;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::NeuronLayoutOf;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::neuron_models::models::memory_fire_hashing::quantizations::{NMMemoryFireHashingNeurogenesisGeneratorQuanted, NMMemoryFireHashingNeurogenesisValidatedGeneratorQuanted, NMMemoryFireHashingQuantKind, NMMemoryFireHashingQuantStandard32Bit};
#[cfg(feature = "alloc")]
use crate::neuron_models::models::memory_fire_hashing::quantizations::{NMMemoryFireHashingNeurogenesisStorageQuanted, NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted, NMMemoryFireHashingNeurogenesisValidatedStorageQuanted, NMMemoryFireHashingNeurogenesisValidatedStorageOrGeneratorQuanted};
use crate::neuron_models::models::{NeuronModelEnumConversionError, NeurogenesisByGeneratorTypeOf};
#[cfg(feature = "alloc")]
use crate::neuron_models::models::{NeurogenesisFromStorageOf, NeurogenesisFromStorageOrByGeneratorOf};
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::validation::ValidatedNeurogenesisGenerator;
#[cfg(feature = "alloc")]
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::validation::{ValidatedNeurogenesisStorage, ValidatedNeurogenesisStorageOrGenerator};

//region Defaults

/// Generates the scratch and variance data of `MemoryFireHashing` neurons
#[derive(Clone, Debug)]
pub struct MemoryFireHashingNeurogenesisDefaults<FIQ, NM>(PhantomData<fn() -> (FIQ, NM)>);

impl<FIQ, NM> NeurogenesisScratchVarianceDefaults<FIQ, NM> for MemoryFireHashingNeurogenesisDefaults<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NMMemoryFireHashingTrait,
{
    fn static_get_cortical_scratch(
        _layout: &NeuronLayoutOf<FIQ, NM>,
        _cortical_properties: &NM::CorticalDataProperties,
        _cortical_internal: &NM::CorticalDataInternal,
    ) -> NM::CorticalDataScratch {
        NM::CorticalDataScratch::new() // no fields
    }

    fn static_get_neuron_scratch(
        _index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        _layout: &NeuronLayoutOf<FIQ, NM>,
        _internal_neuron_data: &NM::NeuronDataInternal,
    ) -> NM::NeuronDataScratch {
        NM::NeuronDataScratch::new() // no fields
    }

    /// The variance the model's cortical variance gives the neuron
    fn static_get_neuron_variance(
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        layout: &NeuronLayoutOf<FIQ, NM>,
        variance: &NM::CorticalDataVariance,
    ) -> NM::NeuronDataVariance {
        variance.neuron_variance_at::<FIQ>(layout, index)
    }
}

impl<NM: NMMemoryFireHashingTrait> CorticalDataVarianceSpecification<NM> for NMMemoryFireHashingDataCorticalVariance<NM> {
    fn neuron_variance_at<FIQ: FeagiIndexQuantization>(
        &self,
        _layout: &NeuronLayoutOf<FIQ, NM>,
        _index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> NMMemoryFireHashingDataNeuronVariance<NM> {
        NMMemoryFireHashingDataNeuronVariance::new()
    }
}

//endregion

//region Quant Level Conversions

impl<FIQ: FeagiIndexQuantization> From<NeurogenesisByGeneratorTypeOf<FIQ, NMMemoryFireHashingQuantStandard32Bit>> for NMMemoryFireHashingNeurogenesisGeneratorQuanted<FIQ> {
    fn from(value: NeurogenesisByGeneratorTypeOf<FIQ, NMMemoryFireHashingQuantStandard32Bit>) -> Self {
        Self::Standard32Bit(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMMemoryFireHashingNeurogenesisGeneratorQuanted<FIQ>> for NeurogenesisByGeneratorTypeOf<FIQ, NMMemoryFireHashingQuantStandard32Bit> {
    type Error = NeuronModelEnumConversionError<NMMemoryFireHashingQuantKind>;

    fn try_from(value: NMMemoryFireHashingNeurogenesisGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMMemoryFireHashingNeurogenesisGeneratorQuanted::Standard32Bit(v) => Ok(v),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NeurogenesisFromStorageOf<FIQ, NMMemoryFireHashingQuantStandard32Bit>> for NMMemoryFireHashingNeurogenesisStorageQuanted<FIQ> {
    fn from(value: NeurogenesisFromStorageOf<FIQ, NMMemoryFireHashingQuantStandard32Bit>) -> Self {
        Self::Standard32Bit(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMMemoryFireHashingNeurogenesisStorageQuanted<FIQ>> for NeurogenesisFromStorageOf<FIQ, NMMemoryFireHashingQuantStandard32Bit> {
    type Error = NeuronModelEnumConversionError<NMMemoryFireHashingQuantKind>;

    fn try_from(value: NMMemoryFireHashingNeurogenesisStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMMemoryFireHashingNeurogenesisStorageQuanted::Standard32Bit(v) => Ok(v),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMMemoryFireHashingQuantStandard32Bit>> for NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    fn from(value: NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMMemoryFireHashingQuantStandard32Bit>) -> Self {
        Self::Standard32Bit(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMMemoryFireHashingQuantStandard32Bit> {
    type Error = NeuronModelEnumConversionError<NMMemoryFireHashingQuantKind>;

    fn try_from(value: NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMMemoryFireHashingNeurogenesisStorageOrGeneratorQuanted::Standard32Bit(v) => Ok(v),
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisGenerator<FIQ, NMMemoryFireHashingQuantStandard32Bit>> for NMMemoryFireHashingNeurogenesisValidatedGeneratorQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisGenerator<FIQ, NMMemoryFireHashingQuantStandard32Bit>) -> Self {
        Self::Standard32Bit(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMMemoryFireHashingNeurogenesisValidatedGeneratorQuanted<FIQ>> for ValidatedNeurogenesisGenerator<FIQ, NMMemoryFireHashingQuantStandard32Bit> {
    type Error = NeuronModelEnumConversionError<NMMemoryFireHashingQuantKind>;

    fn try_from(value: NMMemoryFireHashingNeurogenesisValidatedGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMMemoryFireHashingNeurogenesisValidatedGeneratorQuanted::Standard32Bit(v) => Ok(v),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisStorage<FIQ, NMMemoryFireHashingQuantStandard32Bit>> for NMMemoryFireHashingNeurogenesisValidatedStorageQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisStorage<FIQ, NMMemoryFireHashingQuantStandard32Bit>) -> Self {
        Self::Standard32Bit(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMMemoryFireHashingNeurogenesisValidatedStorageQuanted<FIQ>> for ValidatedNeurogenesisStorage<FIQ, NMMemoryFireHashingQuantStandard32Bit> {
    type Error = NeuronModelEnumConversionError<NMMemoryFireHashingQuantKind>;

    fn try_from(value: NMMemoryFireHashingNeurogenesisValidatedStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMMemoryFireHashingNeurogenesisValidatedStorageQuanted::Standard32Bit(v) => Ok(v),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisStorageOrGenerator<FIQ, NMMemoryFireHashingQuantStandard32Bit>> for NMMemoryFireHashingNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisStorageOrGenerator<FIQ, NMMemoryFireHashingQuantStandard32Bit>) -> Self {
        Self::Standard32Bit(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMMemoryFireHashingNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>> for ValidatedNeurogenesisStorageOrGenerator<FIQ, NMMemoryFireHashingQuantStandard32Bit> {
    type Error = NeuronModelEnumConversionError<NMMemoryFireHashingQuantKind>;

    fn try_from(value: NMMemoryFireHashingNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMMemoryFireHashingNeurogenesisValidatedStorageOrGeneratorQuanted::Standard32Bit(v) => Ok(v),
        }
    }
}

//endregion
