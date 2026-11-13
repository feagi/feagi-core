//! How the scratch and variance data of `ExclusiveDeath` neurons is generated: the
//! `NeuronModel::NeurogenesisDefaults` of every quantization level of the model. Its
//! `NeuronModel::NeurogenesisByGenerator` is the shared `NeurogenesisGeneratorUniformDefault`
// TODO these are stubs, fill them in

use crate::feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::scratch_variance_defaults::NeurogenesisScratchVarianceDefaults;
use crate::neuron_models::models::exclusive_death::NMExclusiveDeathTrait;
use crate::neuron_models::models::exclusive_death::{NMExclusiveDeathDataCorticalVariance, NMExclusiveDeathDataNeuronVariance};
use core::marker::PhantomData;
use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataVarianceSpecification;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::NeuronLayoutOf;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::neuron_models::models::exclusive_death::quantizations::{NMExclusiveDeathNeurogenesisGeneratorQuanted, NMExclusiveDeathNeurogenesisValidatedGeneratorQuanted, NMExclusiveDeathQuantKind, NMExclusiveDeathQuantFloat16, NMExclusiveDeathQuantBFloat16, NMExclusiveDeathQuantFloat32, NMExclusiveDeathQuantFloat64};
#[cfg(feature = "alloc")]
use crate::neuron_models::models::exclusive_death::quantizations::{NMExclusiveDeathNeurogenesisStorageQuanted, NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted, NMExclusiveDeathNeurogenesisValidatedStorageQuanted, NMExclusiveDeathNeurogenesisValidatedStorageOrGeneratorQuanted};
use crate::neuron_models::models::{NeuronModelEnumConversionError, NeurogenesisByGeneratorTypeOf};
#[cfg(feature = "alloc")]
use crate::neuron_models::models::{NeurogenesisFromStorageOf, NeurogenesisFromStorageOrByGeneratorOf};
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::validation::ValidatedNeurogenesisGenerator;
#[cfg(feature = "alloc")]
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::validation::{ValidatedNeurogenesisStorage, ValidatedNeurogenesisStorageOrGenerator};

//region Defaults

/// Generates the scratch and variance data of `ExclusiveDeath` neurons
#[derive(Clone, Debug)]
pub struct ExclusiveDeathNeurogenesisDefaults<FIQ, NM>(PhantomData<fn() -> (FIQ, NM)>);

impl<FIQ, NM> NeurogenesisScratchVarianceDefaults<FIQ, NM> for ExclusiveDeathNeurogenesisDefaults<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NMExclusiveDeathTrait,
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

impl<NM: NMExclusiveDeathTrait> CorticalDataVarianceSpecification<NM> for NMExclusiveDeathDataCorticalVariance<NM> {
    fn neuron_variance_at<FIQ: FeagiIndexQuantization>(
        &self,
        _layout: &NeuronLayoutOf<FIQ, NM>,
        _index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> NMExclusiveDeathDataNeuronVariance<NM> {
        NMExclusiveDeathDataNeuronVariance::new()
    }
}

//endregion

//region Quant Level Conversions

impl<FIQ: FeagiIndexQuantization> From<NeurogenesisByGeneratorTypeOf<FIQ, NMExclusiveDeathQuantFloat16>> for NMExclusiveDeathNeurogenesisGeneratorQuanted<FIQ> {
    fn from(value: NeurogenesisByGeneratorTypeOf<FIQ, NMExclusiveDeathQuantFloat16>) -> Self {
        Self::Float16(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusiveDeathNeurogenesisGeneratorQuanted<FIQ>> for NeurogenesisByGeneratorTypeOf<FIQ, NMExclusiveDeathQuantFloat16> {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathNeurogenesisGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathNeurogenesisGeneratorQuanted::Float16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusiveDeathQuantFloat16::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<NeurogenesisByGeneratorTypeOf<FIQ, NMExclusiveDeathQuantBFloat16>> for NMExclusiveDeathNeurogenesisGeneratorQuanted<FIQ> {
    fn from(value: NeurogenesisByGeneratorTypeOf<FIQ, NMExclusiveDeathQuantBFloat16>) -> Self {
        Self::BFloat16(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusiveDeathNeurogenesisGeneratorQuanted<FIQ>> for NeurogenesisByGeneratorTypeOf<FIQ, NMExclusiveDeathQuantBFloat16> {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathNeurogenesisGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathNeurogenesisGeneratorQuanted::BFloat16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusiveDeathQuantBFloat16::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<NeurogenesisByGeneratorTypeOf<FIQ, NMExclusiveDeathQuantFloat32>> for NMExclusiveDeathNeurogenesisGeneratorQuanted<FIQ> {
    fn from(value: NeurogenesisByGeneratorTypeOf<FIQ, NMExclusiveDeathQuantFloat32>) -> Self {
        Self::Float32(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusiveDeathNeurogenesisGeneratorQuanted<FIQ>> for NeurogenesisByGeneratorTypeOf<FIQ, NMExclusiveDeathQuantFloat32> {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathNeurogenesisGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathNeurogenesisGeneratorQuanted::Float32(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusiveDeathQuantFloat32::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<NeurogenesisByGeneratorTypeOf<FIQ, NMExclusiveDeathQuantFloat64>> for NMExclusiveDeathNeurogenesisGeneratorQuanted<FIQ> {
    fn from(value: NeurogenesisByGeneratorTypeOf<FIQ, NMExclusiveDeathQuantFloat64>) -> Self {
        Self::Float64(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusiveDeathNeurogenesisGeneratorQuanted<FIQ>> for NeurogenesisByGeneratorTypeOf<FIQ, NMExclusiveDeathQuantFloat64> {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathNeurogenesisGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathNeurogenesisGeneratorQuanted::Float64(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusiveDeathQuantFloat64::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NeurogenesisFromStorageOf<FIQ, NMExclusiveDeathQuantFloat16>> for NMExclusiveDeathNeurogenesisStorageQuanted<FIQ> {
    fn from(value: NeurogenesisFromStorageOf<FIQ, NMExclusiveDeathQuantFloat16>) -> Self {
        Self::Float16(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusiveDeathNeurogenesisStorageQuanted<FIQ>> for NeurogenesisFromStorageOf<FIQ, NMExclusiveDeathQuantFloat16> {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathNeurogenesisStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathNeurogenesisStorageQuanted::Float16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusiveDeathQuantFloat16::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NeurogenesisFromStorageOf<FIQ, NMExclusiveDeathQuantBFloat16>> for NMExclusiveDeathNeurogenesisStorageQuanted<FIQ> {
    fn from(value: NeurogenesisFromStorageOf<FIQ, NMExclusiveDeathQuantBFloat16>) -> Self {
        Self::BFloat16(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusiveDeathNeurogenesisStorageQuanted<FIQ>> for NeurogenesisFromStorageOf<FIQ, NMExclusiveDeathQuantBFloat16> {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathNeurogenesisStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathNeurogenesisStorageQuanted::BFloat16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusiveDeathQuantBFloat16::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NeurogenesisFromStorageOf<FIQ, NMExclusiveDeathQuantFloat32>> for NMExclusiveDeathNeurogenesisStorageQuanted<FIQ> {
    fn from(value: NeurogenesisFromStorageOf<FIQ, NMExclusiveDeathQuantFloat32>) -> Self {
        Self::Float32(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusiveDeathNeurogenesisStorageQuanted<FIQ>> for NeurogenesisFromStorageOf<FIQ, NMExclusiveDeathQuantFloat32> {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathNeurogenesisStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathNeurogenesisStorageQuanted::Float32(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusiveDeathQuantFloat32::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NeurogenesisFromStorageOf<FIQ, NMExclusiveDeathQuantFloat64>> for NMExclusiveDeathNeurogenesisStorageQuanted<FIQ> {
    fn from(value: NeurogenesisFromStorageOf<FIQ, NMExclusiveDeathQuantFloat64>) -> Self {
        Self::Float64(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusiveDeathNeurogenesisStorageQuanted<FIQ>> for NeurogenesisFromStorageOf<FIQ, NMExclusiveDeathQuantFloat64> {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathNeurogenesisStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathNeurogenesisStorageQuanted::Float64(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusiveDeathQuantFloat64::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusiveDeathQuantFloat16>> for NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    fn from(value: NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusiveDeathQuantFloat16>) -> Self {
        Self::Float16(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusiveDeathQuantFloat16> {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted::Float16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusiveDeathQuantFloat16::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusiveDeathQuantBFloat16>> for NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    fn from(value: NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusiveDeathQuantBFloat16>) -> Self {
        Self::BFloat16(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusiveDeathQuantBFloat16> {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted::BFloat16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusiveDeathQuantBFloat16::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusiveDeathQuantFloat32>> for NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    fn from(value: NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusiveDeathQuantFloat32>) -> Self {
        Self::Float32(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusiveDeathQuantFloat32> {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted::Float32(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusiveDeathQuantFloat32::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusiveDeathQuantFloat64>> for NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    fn from(value: NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusiveDeathQuantFloat64>) -> Self {
        Self::Float64(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusiveDeathQuantFloat64> {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathNeurogenesisStorageOrGeneratorQuanted::Float64(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusiveDeathQuantFloat64::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisGenerator<FIQ, NMExclusiveDeathQuantFloat16>> for NMExclusiveDeathNeurogenesisValidatedGeneratorQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisGenerator<FIQ, NMExclusiveDeathQuantFloat16>) -> Self {
        Self::Float16(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusiveDeathNeurogenesisValidatedGeneratorQuanted<FIQ>> for ValidatedNeurogenesisGenerator<FIQ, NMExclusiveDeathQuantFloat16> {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathNeurogenesisValidatedGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathNeurogenesisValidatedGeneratorQuanted::Float16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusiveDeathQuantFloat16::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisGenerator<FIQ, NMExclusiveDeathQuantBFloat16>> for NMExclusiveDeathNeurogenesisValidatedGeneratorQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisGenerator<FIQ, NMExclusiveDeathQuantBFloat16>) -> Self {
        Self::BFloat16(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusiveDeathNeurogenesisValidatedGeneratorQuanted<FIQ>> for ValidatedNeurogenesisGenerator<FIQ, NMExclusiveDeathQuantBFloat16> {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathNeurogenesisValidatedGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathNeurogenesisValidatedGeneratorQuanted::BFloat16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusiveDeathQuantBFloat16::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisGenerator<FIQ, NMExclusiveDeathQuantFloat32>> for NMExclusiveDeathNeurogenesisValidatedGeneratorQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisGenerator<FIQ, NMExclusiveDeathQuantFloat32>) -> Self {
        Self::Float32(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusiveDeathNeurogenesisValidatedGeneratorQuanted<FIQ>> for ValidatedNeurogenesisGenerator<FIQ, NMExclusiveDeathQuantFloat32> {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathNeurogenesisValidatedGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathNeurogenesisValidatedGeneratorQuanted::Float32(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusiveDeathQuantFloat32::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisGenerator<FIQ, NMExclusiveDeathQuantFloat64>> for NMExclusiveDeathNeurogenesisValidatedGeneratorQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisGenerator<FIQ, NMExclusiveDeathQuantFloat64>) -> Self {
        Self::Float64(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusiveDeathNeurogenesisValidatedGeneratorQuanted<FIQ>> for ValidatedNeurogenesisGenerator<FIQ, NMExclusiveDeathQuantFloat64> {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathNeurogenesisValidatedGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathNeurogenesisValidatedGeneratorQuanted::Float64(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusiveDeathQuantFloat64::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisStorage<FIQ, NMExclusiveDeathQuantFloat16>> for NMExclusiveDeathNeurogenesisValidatedStorageQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisStorage<FIQ, NMExclusiveDeathQuantFloat16>) -> Self {
        Self::Float16(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusiveDeathNeurogenesisValidatedStorageQuanted<FIQ>> for ValidatedNeurogenesisStorage<FIQ, NMExclusiveDeathQuantFloat16> {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathNeurogenesisValidatedStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathNeurogenesisValidatedStorageQuanted::Float16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusiveDeathQuantFloat16::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisStorage<FIQ, NMExclusiveDeathQuantBFloat16>> for NMExclusiveDeathNeurogenesisValidatedStorageQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisStorage<FIQ, NMExclusiveDeathQuantBFloat16>) -> Self {
        Self::BFloat16(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusiveDeathNeurogenesisValidatedStorageQuanted<FIQ>> for ValidatedNeurogenesisStorage<FIQ, NMExclusiveDeathQuantBFloat16> {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathNeurogenesisValidatedStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathNeurogenesisValidatedStorageQuanted::BFloat16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusiveDeathQuantBFloat16::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisStorage<FIQ, NMExclusiveDeathQuantFloat32>> for NMExclusiveDeathNeurogenesisValidatedStorageQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisStorage<FIQ, NMExclusiveDeathQuantFloat32>) -> Self {
        Self::Float32(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusiveDeathNeurogenesisValidatedStorageQuanted<FIQ>> for ValidatedNeurogenesisStorage<FIQ, NMExclusiveDeathQuantFloat32> {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathNeurogenesisValidatedStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathNeurogenesisValidatedStorageQuanted::Float32(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusiveDeathQuantFloat32::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisStorage<FIQ, NMExclusiveDeathQuantFloat64>> for NMExclusiveDeathNeurogenesisValidatedStorageQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisStorage<FIQ, NMExclusiveDeathQuantFloat64>) -> Self {
        Self::Float64(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusiveDeathNeurogenesisValidatedStorageQuanted<FIQ>> for ValidatedNeurogenesisStorage<FIQ, NMExclusiveDeathQuantFloat64> {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathNeurogenesisValidatedStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathNeurogenesisValidatedStorageQuanted::Float64(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusiveDeathQuantFloat64::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusiveDeathQuantFloat16>> for NMExclusiveDeathNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusiveDeathQuantFloat16>) -> Self {
        Self::Float16(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusiveDeathNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>> for ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusiveDeathQuantFloat16> {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathNeurogenesisValidatedStorageOrGeneratorQuanted::Float16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusiveDeathQuantFloat16::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusiveDeathQuantBFloat16>> for NMExclusiveDeathNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusiveDeathQuantBFloat16>) -> Self {
        Self::BFloat16(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusiveDeathNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>> for ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusiveDeathQuantBFloat16> {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathNeurogenesisValidatedStorageOrGeneratorQuanted::BFloat16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusiveDeathQuantBFloat16::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusiveDeathQuantFloat32>> for NMExclusiveDeathNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusiveDeathQuantFloat32>) -> Self {
        Self::Float32(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusiveDeathNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>> for ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusiveDeathQuantFloat32> {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathNeurogenesisValidatedStorageOrGeneratorQuanted::Float32(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusiveDeathQuantFloat32::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusiveDeathQuantFloat64>> for NMExclusiveDeathNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusiveDeathQuantFloat64>) -> Self {
        Self::Float64(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusiveDeathNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>> for ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusiveDeathQuantFloat64> {
    type Error = NeuronModelEnumConversionError<NMExclusiveDeathQuantKind>;

    fn try_from(value: NMExclusiveDeathNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusiveDeathNeurogenesisValidatedStorageOrGeneratorQuanted::Float64(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusiveDeathQuantFloat64::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

//endregion
