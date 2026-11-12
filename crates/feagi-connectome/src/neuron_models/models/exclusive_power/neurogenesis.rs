//! How the scratch and variance data of `ExclusivePower` neurons is generated: the
//! `NeuronModel::NeurogenesisDefaults` of every quantization level of the model. Its
//! `NeuronModel::NeurogenesisGenerator` is the shared `NeurogenesisDefaultsGenerator`
// TODO these are stubs, fill them in

use crate::neuron_models::models::exclusive_power::quantizations::{NMExclusivePowerNeurogenesisGeneratorQuanted, NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted, NMExclusivePowerNeurogenesisStorageQuanted, NMExclusivePowerQuantBFloat16, NMExclusivePowerQuantFloat16, NMExclusivePowerQuantFloat32, NMExclusivePowerQuantFloat64, NMExclusivePowerQuantKind};
use crate::neuron_models::models::exclusive_power::NMExclusivePowerTrait;
use crate::neuron_models::models::exclusive_power::{NMExclusivePowerDataCorticalVariance, NMExclusivePowerDataNeuronVariance};
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

impl<FIQ> From<NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantFloat16>> for NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantFloat16>) -> Self {
        NMExclusivePowerNeurogenesisGeneratorQuanted::Float16(value)
    }
}

impl<FIQ> TryFrom<NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ>> for NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantFloat16>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMExclusivePowerNeurogenesisGeneratorQuanted::Float16(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMExclusivePowerQuantFloat16::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantBFloat16>> for NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantBFloat16>) -> Self {
        NMExclusivePowerNeurogenesisGeneratorQuanted::BFloat16(value)
    }
}

impl<FIQ> TryFrom<NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ>> for NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantBFloat16>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMExclusivePowerNeurogenesisGeneratorQuanted::BFloat16(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMExclusivePowerQuantBFloat16::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantFloat32>> for NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantFloat32>) -> Self {
        NMExclusivePowerNeurogenesisGeneratorQuanted::Float32(value)
    }
}

impl<FIQ> TryFrom<NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ>> for NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantFloat32>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMExclusivePowerNeurogenesisGeneratorQuanted::Float32(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMExclusivePowerQuantFloat32::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantFloat64>> for NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantFloat64>) -> Self {
        NMExclusivePowerNeurogenesisGeneratorQuanted::Float64(value)
    }
}

impl<FIQ> TryFrom<NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ>> for NeurogenesisByGeneratorTypeOf<FIQ, NMExclusivePowerQuantFloat64>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMExclusivePowerNeurogenesisGeneratorQuanted::Float64(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMExclusivePowerQuantFloat64::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

//endregion

//endregion

//region Defaults

/// Generates the scratch and variance data of `ExclusivePower` neurons
#[derive(Clone, Debug)]
pub struct ExclusivePowerNeurogenesisDefaults<FIQ, NM>(PhantomData<fn() -> (FIQ, NM)>);

impl<FIQ, NM> NeurogenesisScratchVarianceGenerator<FIQ, NM> for ExclusivePowerNeurogenesisDefaults<FIQ, NM>
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

//region Storage and StorageOrGenerator Conversions

//region Storage

impl<FIQ> From<NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantFloat16>> for NMExclusivePowerNeurogenesisStorageQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantFloat16>) -> Self {
        NMExclusivePowerNeurogenesisStorageQuanted::Float16(value)
    }
}

impl<FIQ> TryFrom<NMExclusivePowerNeurogenesisStorageQuanted<FIQ>> for NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantFloat16>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMExclusivePowerNeurogenesisStorageQuanted::Float16(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMExclusivePowerQuantFloat16::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantBFloat16>> for NMExclusivePowerNeurogenesisStorageQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantBFloat16>) -> Self {
        NMExclusivePowerNeurogenesisStorageQuanted::BFloat16(value)
    }
}

impl<FIQ> TryFrom<NMExclusivePowerNeurogenesisStorageQuanted<FIQ>> for NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantBFloat16>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMExclusivePowerNeurogenesisStorageQuanted::BFloat16(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMExclusivePowerQuantBFloat16::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantFloat32>> for NMExclusivePowerNeurogenesisStorageQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantFloat32>) -> Self {
        NMExclusivePowerNeurogenesisStorageQuanted::Float32(value)
    }
}

impl<FIQ> TryFrom<NMExclusivePowerNeurogenesisStorageQuanted<FIQ>> for NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantFloat32>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMExclusivePowerNeurogenesisStorageQuanted::Float32(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMExclusivePowerQuantFloat32::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantFloat64>> for NMExclusivePowerNeurogenesisStorageQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantFloat64>) -> Self {
        NMExclusivePowerNeurogenesisStorageQuanted::Float64(value)
    }
}

impl<FIQ> TryFrom<NMExclusivePowerNeurogenesisStorageQuanted<FIQ>> for NeurogenesisFromStorageOf<FIQ, NMExclusivePowerQuantFloat64>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMExclusivePowerNeurogenesisStorageQuanted::Float64(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMExclusivePowerQuantFloat64::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

//endregion

//region Storage Or Generator

impl<FIQ> From<NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantFloat16>> for NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantFloat16>) -> Self {
        NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted::Float16(value)
    }
}

impl<FIQ> TryFrom<NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantFloat16>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted::Float16(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMExclusivePowerQuantFloat16::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantBFloat16>> for NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantBFloat16>) -> Self {
        NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted::BFloat16(value)
    }
}

impl<FIQ> TryFrom<NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantBFloat16>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted::BFloat16(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMExclusivePowerQuantBFloat16::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantFloat32>> for NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantFloat32>) -> Self {
        NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted::Float32(value)
    }
}

impl<FIQ> TryFrom<NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantFloat32>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted::Float32(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMExclusivePowerQuantFloat32::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantFloat64>> for NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantFloat64>) -> Self {
        NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted::Float64(value)
    }
}

impl<FIQ> TryFrom<NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMExclusivePowerQuantFloat64>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMExclusivePowerQuantKind>;

    fn try_from(value: NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMExclusivePowerNeurogenesisStorageOrGeneratorQuanted::Float64(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMExclusivePowerQuantFloat64::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

//endregion

//endregion
