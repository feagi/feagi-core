//! How the scratch and variance data of `FeagiSimple` neurons is generated: the
//! `NeuronModel::NeurogenesisDefaults` of every quantization level of the model. Its
//! `NeuronModel::NeurogenesisByGenerator` is the shared `NeurogenesisGeneratorUniformDefault`
// TODO these are stubs, fill them in

use crate::feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::scratch_variance_defaults::NeurogenesisScratchVarianceDefaults;
use crate::neuron_models::models::feagi_simple::NMFeagiSimpleTrait;
use crate::neuron_models::models::feagi_simple::{NMFeagiSimpleDataCorticalVariance, NMFeagiSimpleDataNeuronVariance};
use core::marker::PhantomData;
use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataVarianceSpecification;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::NeuronLayoutOf;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::neuron_models::models::feagi_simple::quantizations::{NMFeagiSimpleNeurogenesisGeneratorQuanted, NMFeagiSimpleNeurogenesisValidatedGeneratorQuanted, NMFeagiSimpleQuantKind, NMFeagiSimpleQuantStandard32Bit};
#[cfg(feature = "alloc")]
use crate::neuron_models::models::feagi_simple::quantizations::{NMFeagiSimpleNeurogenesisStorageQuanted, NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted, NMFeagiSimpleNeurogenesisValidatedStorageQuanted, NMFeagiSimpleNeurogenesisValidatedStorageOrGeneratorQuanted};
use crate::neuron_models::models::{NeuronModelEnumConversionError, NeurogenesisByGeneratorTypeOf};
#[cfg(feature = "alloc")]
use crate::neuron_models::models::{NeurogenesisFromStorageOf, NeurogenesisFromStorageOrByGeneratorOf};
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::validation::ValidatedNeurogenesisGenerator;
#[cfg(feature = "alloc")]
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::validation::{ValidatedNeurogenesisStorage, ValidatedNeurogenesisStorageOrGenerator};

//region Defaults

/// Generates the scratch and variance data of `FeagiSimple` neurons
#[derive(Clone, Debug)]
pub struct NMFeagiSimpleScratchVarianceGenerator<FIQ, NM>(PhantomData<fn() -> (FIQ, NM)>);

impl<FIQ, NM> NeurogenesisScratchVarianceDefaults<FIQ, NM> for NMFeagiSimpleScratchVarianceGenerator<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NMFeagiSimpleTrait,
{
    fn static_get_cortical_scratch(
        _layout: &NeuronLayoutOf<FIQ, NM>,
        _cortical_properties: &NM::CorticalDataProperties,
        _cortical_internal: &NM::CorticalDataInternal,
    ) -> NM::CorticalDataScratch {
        NM::CorticalDataScratch::default() // no fields
    }

    fn static_get_neuron_scratch(
        _index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        _layout: &NeuronLayoutOf<FIQ, NM>,
        _internal_neuron_data: &NM::NeuronDataInternal,
    ) -> NM::NeuronDataScratch {
        NM::NeuronDataScratch::default() // no fields
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

impl<NM: NMFeagiSimpleTrait> CorticalDataVarianceSpecification<NM> for NMFeagiSimpleDataCorticalVariance<NM> {
    fn neuron_variance_at<FIQ: FeagiIndexQuantization>(
        &self,
        _layout: &NeuronLayoutOf<FIQ, NM>,
        _index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> NMFeagiSimpleDataNeuronVariance<NM> {
        NMFeagiSimpleDataNeuronVariance::default()
    }
}

//endregion

//region Quant Level Conversions

impl<FIQ: FeagiIndexQuantization> From<NeurogenesisByGeneratorTypeOf<FIQ, NMFeagiSimpleQuantStandard32Bit>> for NMFeagiSimpleNeurogenesisGeneratorQuanted<FIQ> {
    fn from(value: NeurogenesisByGeneratorTypeOf<FIQ, NMFeagiSimpleQuantStandard32Bit>) -> Self {
        Self::Standard32Bit(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMFeagiSimpleNeurogenesisGeneratorQuanted<FIQ>> for NeurogenesisByGeneratorTypeOf<FIQ, NMFeagiSimpleQuantStandard32Bit> {
    type Error = NeuronModelEnumConversionError<NMFeagiSimpleQuantKind>;

    fn try_from(value: NMFeagiSimpleNeurogenesisGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMFeagiSimpleNeurogenesisGeneratorQuanted::Standard32Bit(v) => Ok(v),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NeurogenesisFromStorageOf<FIQ, NMFeagiSimpleQuantStandard32Bit>> for NMFeagiSimpleNeurogenesisStorageQuanted<FIQ> {
    fn from(value: NeurogenesisFromStorageOf<FIQ, NMFeagiSimpleQuantStandard32Bit>) -> Self {
        Self::Standard32Bit(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMFeagiSimpleNeurogenesisStorageQuanted<FIQ>> for NeurogenesisFromStorageOf<FIQ, NMFeagiSimpleQuantStandard32Bit> {
    type Error = NeuronModelEnumConversionError<NMFeagiSimpleQuantKind>;

    fn try_from(value: NMFeagiSimpleNeurogenesisStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMFeagiSimpleNeurogenesisStorageQuanted::Standard32Bit(v) => Ok(v),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMFeagiSimpleQuantStandard32Bit>> for NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    fn from(value: NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMFeagiSimpleQuantStandard32Bit>) -> Self {
        Self::Standard32Bit(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMFeagiSimpleQuantStandard32Bit> {
    type Error = NeuronModelEnumConversionError<NMFeagiSimpleQuantKind>;

    fn try_from(value: NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMFeagiSimpleNeurogenesisStorageOrGeneratorQuanted::Standard32Bit(v) => Ok(v),
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisGenerator<FIQ, NMFeagiSimpleQuantStandard32Bit>> for NMFeagiSimpleNeurogenesisValidatedGeneratorQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisGenerator<FIQ, NMFeagiSimpleQuantStandard32Bit>) -> Self {
        Self::Standard32Bit(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMFeagiSimpleNeurogenesisValidatedGeneratorQuanted<FIQ>> for ValidatedNeurogenesisGenerator<FIQ, NMFeagiSimpleQuantStandard32Bit> {
    type Error = NeuronModelEnumConversionError<NMFeagiSimpleQuantKind>;

    fn try_from(value: NMFeagiSimpleNeurogenesisValidatedGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMFeagiSimpleNeurogenesisValidatedGeneratorQuanted::Standard32Bit(v) => Ok(v),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisStorage<FIQ, NMFeagiSimpleQuantStandard32Bit>> for NMFeagiSimpleNeurogenesisValidatedStorageQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisStorage<FIQ, NMFeagiSimpleQuantStandard32Bit>) -> Self {
        Self::Standard32Bit(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMFeagiSimpleNeurogenesisValidatedStorageQuanted<FIQ>> for ValidatedNeurogenesisStorage<FIQ, NMFeagiSimpleQuantStandard32Bit> {
    type Error = NeuronModelEnumConversionError<NMFeagiSimpleQuantKind>;

    fn try_from(value: NMFeagiSimpleNeurogenesisValidatedStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMFeagiSimpleNeurogenesisValidatedStorageQuanted::Standard32Bit(v) => Ok(v),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisStorageOrGenerator<FIQ, NMFeagiSimpleQuantStandard32Bit>> for NMFeagiSimpleNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisStorageOrGenerator<FIQ, NMFeagiSimpleQuantStandard32Bit>) -> Self {
        Self::Standard32Bit(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMFeagiSimpleNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>> for ValidatedNeurogenesisStorageOrGenerator<FIQ, NMFeagiSimpleQuantStandard32Bit> {
    type Error = NeuronModelEnumConversionError<NMFeagiSimpleQuantKind>;

    fn try_from(value: NMFeagiSimpleNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMFeagiSimpleNeurogenesisValidatedStorageOrGeneratorQuanted::Standard32Bit(v) => Ok(v),
        }
    }
}

//endregion
