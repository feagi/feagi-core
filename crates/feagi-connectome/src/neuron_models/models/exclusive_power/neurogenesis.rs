//! How the scratch and variance data of `ExclusivePower` neurons is generated: the
//! `NeuronModel::NeurogenesisDefaults` of every quantization level of the model. Its
//! `NeuronModel::NeurogenesisByGenerator` is the shared `NeurogenesisGeneratorUniformDefault`
// TODO these are stubs, fill them in

use crate::feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::scratch_variance_defaults::NeurogenesisScratchVarianceDefaults;
use crate::neuron_models::models::exclusive_power::NMExclusivePowerTrait;
use crate::neuron_models::models::exclusive_power::{NMExclusivePowerDataCorticalVariance, NMExclusivePowerDataNeuronVariance};
use core::marker::PhantomData;
use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataVarianceSpecification;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::NeuronLayoutOf;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::neuron_models::models::exclusive_power::quantizations::{NMExclusivePowerNeurogenesisGeneratorQuanted, NMExclusivePowerNeurogenesisValidatedGeneratorQuanted, NMExclusivePowerQuantKind, NMExclusivePowerQuantFloat16, NMExclusivePowerQuantBFloat16, NMExclusivePowerQuantFloat32, NMExclusivePowerQuantFloat64};
#[cfg(feature = "alloc")]
use crate::neuron_models::models::exclusive_power::quantizations::{NMExclusivePowerNeurogenesisStorageQuanted, NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted, NMExclusivePowerNeurogenesisValidatedStorageQuanted, NMExclusivePowerNeurogenesisValidatedStorageOrGeneratorQuanted};
use crate::neuron_models::models::{NeuronModelEnumConversionError, NeurogenesisByGeneratorTypeOf};
#[cfg(feature = "alloc")]
use crate::neuron_models::models::{NeurogenesisFromStorageOf, NeurogenesisFromStorageOrByGeneratorOf};
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::validation::ValidatedNeurogenesisGenerator;
#[cfg(feature = "alloc")]
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::validation::{ValidatedNeurogenesisStorage, ValidatedNeurogenesisStorageOrGenerator};

//region Defaults

/// Generates the scratch and variance data of `ExclusivePower` neurons
#[derive(Clone, Debug)]
pub struct ExclusivePowerNeurogenesisDefaults<FIQ, NM>(PhantomData<fn() -> (FIQ, NM)>);

impl<FIQ, NM> NeurogenesisScratchVarianceDefaults<FIQ, NM> for ExclusivePowerNeurogenesisDefaults<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NMExclusivePowerTrait,
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

impl<NM: NMExclusivePowerTrait> CorticalDataVarianceSpecification<NM> for NMExclusivePowerDataCorticalVariance<NM> {
    fn neuron_variance_at<FIQ: FeagiIndexQuantization>(
        &self,
        _layout: &NeuronLayoutOf<FIQ, NM>,
        _index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> NMExclusivePowerDataNeuronVariance<NM> {
        NMExclusivePowerDataNeuronVariance::new()
    }
}

//endregion

//region Quant Level Conversions

impl<FIQ: FeagiIndexQuantization> From<NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantFloat16>> for NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ> {
    fn from(value: NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantFloat16>) -> Self {
        Self::Float16(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ>> for NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantFloat16> {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerNeurogenesisGeneratorQuanted::Float16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusivePowerQuantFloat16::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantBFloat16>> for NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ> {
    fn from(value: NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantBFloat16>) -> Self {
        Self::BFloat16(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ>> for NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantBFloat16> {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerNeurogenesisGeneratorQuanted::BFloat16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusivePowerQuantBFloat16::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantFloat32>> for NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ> {
    fn from(value: NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantFloat32>) -> Self {
        Self::Float32(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ>> for NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantFloat32> {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerNeurogenesisGeneratorQuanted::Float32(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusivePowerQuantFloat32::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantFloat64>> for NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ> {
    fn from(value: NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantFloat64>) -> Self {
        Self::Float64(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ>> for NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantFloat64> {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerNeurogenesisGeneratorQuanted::Float64(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusivePowerQuantFloat64::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantFloat16>> for NMExclusivePowerNeurogenesisStorageQuanted<FIQ> {
    fn from(value: NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantFloat16>) -> Self {
        Self::Float16(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusivePowerNeurogenesisStorageQuanted<FIQ>> for NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantFloat16> {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerNeurogenesisStorageQuanted::Float16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusivePowerQuantFloat16::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantBFloat16>> for NMExclusivePowerNeurogenesisStorageQuanted<FIQ> {
    fn from(value: NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantBFloat16>) -> Self {
        Self::BFloat16(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusivePowerNeurogenesisStorageQuanted<FIQ>> for NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantBFloat16> {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerNeurogenesisStorageQuanted::BFloat16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusivePowerQuantBFloat16::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantFloat32>> for NMExclusivePowerNeurogenesisStorageQuanted<FIQ> {
    fn from(value: NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantFloat32>) -> Self {
        Self::Float32(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusivePowerNeurogenesisStorageQuanted<FIQ>> for NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantFloat32> {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerNeurogenesisStorageQuanted::Float32(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusivePowerQuantFloat32::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantFloat64>> for NMExclusivePowerNeurogenesisStorageQuanted<FIQ> {
    fn from(value: NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantFloat64>) -> Self {
        Self::Float64(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusivePowerNeurogenesisStorageQuanted<FIQ>> for NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantFloat64> {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerNeurogenesisStorageQuanted::Float64(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusivePowerQuantFloat64::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantFloat16>> for NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    fn from(value: NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantFloat16>) -> Self {
        Self::Float16(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantFloat16> {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted::Float16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusivePowerQuantFloat16::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantBFloat16>> for NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    fn from(value: NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantBFloat16>) -> Self {
        Self::BFloat16(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantBFloat16> {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted::BFloat16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusivePowerQuantBFloat16::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantFloat32>> for NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    fn from(value: NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantFloat32>) -> Self {
        Self::Float32(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantFloat32> {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted::Float32(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusivePowerQuantFloat32::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantFloat64>> for NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    fn from(value: NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantFloat64>) -> Self {
        Self::Float64(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantFloat64> {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted::Float64(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusivePowerQuantFloat64::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisGenerator<FIQ, NMExclusivePowerQuantFloat16>> for NMExclusivePowerNeurogenesisValidatedGeneratorQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisGenerator<FIQ, NMExclusivePowerQuantFloat16>) -> Self {
        Self::Float16(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusivePowerNeurogenesisValidatedGeneratorQuanted<FIQ>> for ValidatedNeurogenesisGenerator<FIQ, NMExclusivePowerQuantFloat16> {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisValidatedGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerNeurogenesisValidatedGeneratorQuanted::Float16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusivePowerQuantFloat16::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisGenerator<FIQ, NMExclusivePowerQuantBFloat16>> for NMExclusivePowerNeurogenesisValidatedGeneratorQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisGenerator<FIQ, NMExclusivePowerQuantBFloat16>) -> Self {
        Self::BFloat16(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusivePowerNeurogenesisValidatedGeneratorQuanted<FIQ>> for ValidatedNeurogenesisGenerator<FIQ, NMExclusivePowerQuantBFloat16> {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisValidatedGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerNeurogenesisValidatedGeneratorQuanted::BFloat16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusivePowerQuantBFloat16::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisGenerator<FIQ, NMExclusivePowerQuantFloat32>> for NMExclusivePowerNeurogenesisValidatedGeneratorQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisGenerator<FIQ, NMExclusivePowerQuantFloat32>) -> Self {
        Self::Float32(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusivePowerNeurogenesisValidatedGeneratorQuanted<FIQ>> for ValidatedNeurogenesisGenerator<FIQ, NMExclusivePowerQuantFloat32> {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisValidatedGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerNeurogenesisValidatedGeneratorQuanted::Float32(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusivePowerQuantFloat32::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisGenerator<FIQ, NMExclusivePowerQuantFloat64>> for NMExclusivePowerNeurogenesisValidatedGeneratorQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisGenerator<FIQ, NMExclusivePowerQuantFloat64>) -> Self {
        Self::Float64(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusivePowerNeurogenesisValidatedGeneratorQuanted<FIQ>> for ValidatedNeurogenesisGenerator<FIQ, NMExclusivePowerQuantFloat64> {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisValidatedGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerNeurogenesisValidatedGeneratorQuanted::Float64(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusivePowerQuantFloat64::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisStorage<FIQ, NMExclusivePowerQuantFloat16>> for NMExclusivePowerNeurogenesisValidatedStorageQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisStorage<FIQ, NMExclusivePowerQuantFloat16>) -> Self {
        Self::Float16(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusivePowerNeurogenesisValidatedStorageQuanted<FIQ>> for ValidatedNeurogenesisStorage<FIQ, NMExclusivePowerQuantFloat16> {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisValidatedStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerNeurogenesisValidatedStorageQuanted::Float16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusivePowerQuantFloat16::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisStorage<FIQ, NMExclusivePowerQuantBFloat16>> for NMExclusivePowerNeurogenesisValidatedStorageQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisStorage<FIQ, NMExclusivePowerQuantBFloat16>) -> Self {
        Self::BFloat16(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusivePowerNeurogenesisValidatedStorageQuanted<FIQ>> for ValidatedNeurogenesisStorage<FIQ, NMExclusivePowerQuantBFloat16> {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisValidatedStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerNeurogenesisValidatedStorageQuanted::BFloat16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusivePowerQuantBFloat16::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisStorage<FIQ, NMExclusivePowerQuantFloat32>> for NMExclusivePowerNeurogenesisValidatedStorageQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisStorage<FIQ, NMExclusivePowerQuantFloat32>) -> Self {
        Self::Float32(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusivePowerNeurogenesisValidatedStorageQuanted<FIQ>> for ValidatedNeurogenesisStorage<FIQ, NMExclusivePowerQuantFloat32> {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisValidatedStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerNeurogenesisValidatedStorageQuanted::Float32(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusivePowerQuantFloat32::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisStorage<FIQ, NMExclusivePowerQuantFloat64>> for NMExclusivePowerNeurogenesisValidatedStorageQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisStorage<FIQ, NMExclusivePowerQuantFloat64>) -> Self {
        Self::Float64(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusivePowerNeurogenesisValidatedStorageQuanted<FIQ>> for ValidatedNeurogenesisStorage<FIQ, NMExclusivePowerQuantFloat64> {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisValidatedStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerNeurogenesisValidatedStorageQuanted::Float64(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusivePowerQuantFloat64::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusivePowerQuantFloat16>> for NMExclusivePowerNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusivePowerQuantFloat16>) -> Self {
        Self::Float16(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusivePowerNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>> for ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusivePowerQuantFloat16> {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerNeurogenesisValidatedStorageOrGeneratorQuanted::Float16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusivePowerQuantFloat16::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusivePowerQuantBFloat16>> for NMExclusivePowerNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusivePowerQuantBFloat16>) -> Self {
        Self::BFloat16(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusivePowerNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>> for ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusivePowerQuantBFloat16> {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerNeurogenesisValidatedStorageOrGeneratorQuanted::BFloat16(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusivePowerQuantBFloat16::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusivePowerQuantFloat32>> for NMExclusivePowerNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusivePowerQuantFloat32>) -> Self {
        Self::Float32(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusivePowerNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>> for ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusivePowerQuantFloat32> {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerNeurogenesisValidatedStorageOrGeneratorQuanted::Float32(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusivePowerQuantFloat32::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusivePowerQuantFloat64>> for NMExclusivePowerNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusivePowerQuantFloat64>) -> Self {
        Self::Float64(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMExclusivePowerNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>> for ValidatedNeurogenesisStorageOrGenerator<FIQ, NMExclusivePowerQuantFloat64> {
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMExclusivePowerNeurogenesisValidatedStorageOrGeneratorQuanted::Float64(v) => Ok(v),
            other => Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                expected: NMExclusivePowerQuantFloat64::QUANT_KIND,
                actual: other.quant_kind(),
            }),
        }
    }
}

//endregion
