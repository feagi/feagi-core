//! How the scratch and variance data of `FoundationPower` neurons is generated: the
//! `NeuronModel::NeurogenesisDefaults` of every quantization level of the model. Its
//! `NeuronModel::NeurogenesisGenerator` is the shared `NeurogenesisDefaultsGenerator`
// TODO these are stubs, fill them in

use crate::neuron_models::models::foundation_power::NMFoundationPowerTrait;
use crate::neuron_models::models::foundation_power::{NMFoundationPowerDataCorticalVariance, NMFoundationPowerDataNeuronVariance};
use core::marker::PhantomData;
use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;
use feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataVarianceSpecification;
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::NeurogenesisScratchVarianceGenerator;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::NeuronLayoutOf;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use crate::neuron_models::models::foundation_power::quantizations::{NMFoundationPowerNeurogenesisGeneratorQuanted, NMFoundationPowerNeurogenesisStorageQuanted, NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted, NMFoundationPowerQuantKind, NMFoundationPowerQuantFloat16, NMFoundationPowerQuantBFloat16, NMFoundationPowerQuantFloat32, NMFoundationPowerQuantFloat64};
use crate::neuron_models::{NeuronModelEnumConversionError, ProxyNMNeuroGenesisFromGenerator, ProxyNMNeuroGenesisFromStorage, ProxyNMNeurogenesisFromStorageOrByGenerator};

//region Generator

// NOTE: we are using NeurogenesisGeneratorUniformDefault, not a specific impl for this model

//region Impls

impl<FIQ> From<ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationPowerQuantFloat16>> for NMFoundationPowerNeurogenesisGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationPowerQuantFloat16>) -> Self {
        NMFoundationPowerNeurogenesisGeneratorQuanted::Float16(value)
    }
}

impl<FIQ> TryFrom<NMFoundationPowerNeurogenesisGeneratorQuanted<FIQ>> for ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationPowerQuantFloat16>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFoundationPowerQuantKind>;

    fn try_from(value: NMFoundationPowerNeurogenesisGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationPowerNeurogenesisGeneratorQuanted::Float16(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationPowerQuantFloat16::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationPowerQuantBFloat16>> for NMFoundationPowerNeurogenesisGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationPowerQuantBFloat16>) -> Self {
        NMFoundationPowerNeurogenesisGeneratorQuanted::BFloat16(value)
    }
}

impl<FIQ> TryFrom<NMFoundationPowerNeurogenesisGeneratorQuanted<FIQ>> for ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationPowerQuantBFloat16>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFoundationPowerQuantKind>;

    fn try_from(value: NMFoundationPowerNeurogenesisGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationPowerNeurogenesisGeneratorQuanted::BFloat16(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationPowerQuantBFloat16::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationPowerQuantFloat32>> for NMFoundationPowerNeurogenesisGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationPowerQuantFloat32>) -> Self {
        NMFoundationPowerNeurogenesisGeneratorQuanted::Float32(value)
    }
}

impl<FIQ> TryFrom<NMFoundationPowerNeurogenesisGeneratorQuanted<FIQ>> for ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationPowerQuantFloat32>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFoundationPowerQuantKind>;

    fn try_from(value: NMFoundationPowerNeurogenesisGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationPowerNeurogenesisGeneratorQuanted::Float32(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationPowerQuantFloat32::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationPowerQuantFloat64>> for NMFoundationPowerNeurogenesisGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationPowerQuantFloat64>) -> Self {
        NMFoundationPowerNeurogenesisGeneratorQuanted::Float64(value)
    }
}

impl<FIQ> TryFrom<NMFoundationPowerNeurogenesisGeneratorQuanted<FIQ>> for ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationPowerQuantFloat64>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFoundationPowerQuantKind>;

    fn try_from(value: NMFoundationPowerNeurogenesisGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationPowerNeurogenesisGeneratorQuanted::Float64(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationPowerQuantFloat64::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

//endregion

//endregion

//region Defaults

/// Generates the scratch and variance data of `FoundationPower` neurons
#[derive(Clone, Debug)]
pub struct FoundationPowerNeurogenesisDefaults<FIQ, NM>(PhantomData<fn() -> (FIQ, NM)>);

impl<FIQ, NM> NeurogenesisScratchVarianceGenerator<FIQ, NM> for FoundationPowerNeurogenesisDefaults<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NMFoundationPowerTrait,
{
    fn static_get_cortical_scratch(
        _layout: &NeuronLayoutOf<NM, FIQ>,
        _cortical_properties: &NM::CorticalDataProperties,
        _cortical_internal: &NM::CorticalDataInternal,
    ) -> NM::CorticalDataScratch {
        todo!("FoundationPower cortical scratch")
    }

    fn static_get_neuron_scratch(
        _index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        _layout: &NeuronLayoutOf<NM, FIQ>,
        _internal_neuron_data: &NM::NeuronDataInternal,
    ) -> NM::NeuronDataScratch {
        todo!("FoundationPower neuron scratch")
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

impl<NM: NMFoundationPowerTrait> CorticalDataVarianceSpecification<NM> for NMFoundationPowerDataCorticalVariance<NM> {
    fn neuron_variance_at<FIQ: FeagiIndexQuantization>(
        &self,
        _layout: &NeuronLayoutOf<NM, FIQ>,
        _index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> NMFoundationPowerDataNeuronVariance<NM> {
        NMFoundationPowerDataNeuronVariance::new()
    }
}

//endregion

//region Storage and StorageOrGenerator Conversions

//region Storage

impl<FIQ> From<ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationPowerQuantFloat16>> for NMFoundationPowerNeurogenesisStorageQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationPowerQuantFloat16>) -> Self {
        NMFoundationPowerNeurogenesisStorageQuanted::Float16(value)
    }
}

impl<FIQ> TryFrom<NMFoundationPowerNeurogenesisStorageQuanted<FIQ>> for ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationPowerQuantFloat16>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFoundationPowerQuantKind>;

    fn try_from(value: NMFoundationPowerNeurogenesisStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationPowerNeurogenesisStorageQuanted::Float16(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationPowerQuantFloat16::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationPowerQuantBFloat16>> for NMFoundationPowerNeurogenesisStorageQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationPowerQuantBFloat16>) -> Self {
        NMFoundationPowerNeurogenesisStorageQuanted::BFloat16(value)
    }
}

impl<FIQ> TryFrom<NMFoundationPowerNeurogenesisStorageQuanted<FIQ>> for ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationPowerQuantBFloat16>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFoundationPowerQuantKind>;

    fn try_from(value: NMFoundationPowerNeurogenesisStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationPowerNeurogenesisStorageQuanted::BFloat16(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationPowerQuantBFloat16::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationPowerQuantFloat32>> for NMFoundationPowerNeurogenesisStorageQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationPowerQuantFloat32>) -> Self {
        NMFoundationPowerNeurogenesisStorageQuanted::Float32(value)
    }
}

impl<FIQ> TryFrom<NMFoundationPowerNeurogenesisStorageQuanted<FIQ>> for ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationPowerQuantFloat32>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFoundationPowerQuantKind>;

    fn try_from(value: NMFoundationPowerNeurogenesisStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationPowerNeurogenesisStorageQuanted::Float32(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationPowerQuantFloat32::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationPowerQuantFloat64>> for NMFoundationPowerNeurogenesisStorageQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationPowerQuantFloat64>) -> Self {
        NMFoundationPowerNeurogenesisStorageQuanted::Float64(value)
    }
}

impl<FIQ> TryFrom<NMFoundationPowerNeurogenesisStorageQuanted<FIQ>> for ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationPowerQuantFloat64>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFoundationPowerQuantKind>;

    fn try_from(value: NMFoundationPowerNeurogenesisStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationPowerNeurogenesisStorageQuanted::Float64(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationPowerQuantFloat64::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

//endregion

//region Storage Or Generator

impl<FIQ> From<ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationPowerQuantFloat16>> for NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationPowerQuantFloat16>) -> Self {
        NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted::Float16(value)
    }
}

impl<FIQ> TryFrom<NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted<FIQ>> for ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationPowerQuantFloat16>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFoundationPowerQuantKind>;

    fn try_from(value: NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted::Float16(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationPowerQuantFloat16::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationPowerQuantBFloat16>> for NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationPowerQuantBFloat16>) -> Self {
        NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted::BFloat16(value)
    }
}

impl<FIQ> TryFrom<NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted<FIQ>> for ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationPowerQuantBFloat16>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFoundationPowerQuantKind>;

    fn try_from(value: NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted::BFloat16(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationPowerQuantBFloat16::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationPowerQuantFloat32>> for NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationPowerQuantFloat32>) -> Self {
        NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted::Float32(value)
    }
}

impl<FIQ> TryFrom<NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted<FIQ>> for ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationPowerQuantFloat32>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFoundationPowerQuantKind>;

    fn try_from(value: NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted::Float32(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationPowerQuantFloat32::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationPowerQuantFloat64>> for NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationPowerQuantFloat64>) -> Self {
        NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted::Float64(value)
    }
}

impl<FIQ> TryFrom<NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted<FIQ>> for ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationPowerQuantFloat64>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFoundationPowerQuantKind>;

    fn try_from(value: NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationPowerNeurogenesisStorageOrGeneratorQuanted::Float64(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationPowerQuantFloat64::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

//endregion

//endregion
