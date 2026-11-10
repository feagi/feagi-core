//! How the scratch and variance data of `FoundationDeath` neurons is generated: the
//! `NeuronModel::NeurogenesisDefaults` of every quantization level of the model. Its
//! `NeuronModel::NeurogenesisGenerator` is the shared `NeurogenesisDefaultsGenerator`
// TODO these are stubs, fill them in

use crate::neuron_models::models::foundation_death::quantizations::{NMFoundationDeathNeurogenesisGeneratorQuanted, NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted, NMFoundationDeathNeurogenesisStorageQuanted, NMFoundationDeathQuantBFloat16, NMFoundationDeathQuantFloat16, NMFoundationDeathQuantFloat32, NMFoundationDeathQuantFloat64, NMFoundationDeathQuantKind};
use crate::neuron_models::models::foundation_death::NMFoundationDeathTrait;
use crate::neuron_models::models::foundation_death::{NMFoundationDeathDataCorticalVariance, NMFoundationDeathDataNeuronVariance};
use crate::neuron_models::models::{NeuronModelEnumConversionError, ProxyNMNeuroGenesisFromGenerator, ProxyNMNeuroGenesisFromStorage, ProxyNMNeurogenesisFromStorageOrByGenerator};
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

impl<FIQ> From<ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationDeathQuantFloat16>> for NMFoundationDeathNeurogenesisGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationDeathQuantFloat16>) -> Self {
        NMFoundationDeathNeurogenesisGeneratorQuanted::Float16(value)
    }
}

impl<FIQ> TryFrom<NMFoundationDeathNeurogenesisGeneratorQuanted<FIQ>> for ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationDeathQuantFloat16>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFoundationDeathQuantKind>;

    fn try_from(value: NMFoundationDeathNeurogenesisGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationDeathNeurogenesisGeneratorQuanted::Float16(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationDeathQuantFloat16::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationDeathQuantBFloat16>> for NMFoundationDeathNeurogenesisGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationDeathQuantBFloat16>) -> Self {
        NMFoundationDeathNeurogenesisGeneratorQuanted::BFloat16(value)
    }
}

impl<FIQ> TryFrom<NMFoundationDeathNeurogenesisGeneratorQuanted<FIQ>> for ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationDeathQuantBFloat16>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFoundationDeathQuantKind>;

    fn try_from(value: NMFoundationDeathNeurogenesisGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationDeathNeurogenesisGeneratorQuanted::BFloat16(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationDeathQuantBFloat16::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationDeathQuantFloat32>> for NMFoundationDeathNeurogenesisGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationDeathQuantFloat32>) -> Self {
        NMFoundationDeathNeurogenesisGeneratorQuanted::Float32(value)
    }
}

impl<FIQ> TryFrom<NMFoundationDeathNeurogenesisGeneratorQuanted<FIQ>> for ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationDeathQuantFloat32>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFoundationDeathQuantKind>;

    fn try_from(value: NMFoundationDeathNeurogenesisGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationDeathNeurogenesisGeneratorQuanted::Float32(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationDeathQuantFloat32::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationDeathQuantFloat64>> for NMFoundationDeathNeurogenesisGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationDeathQuantFloat64>) -> Self {
        NMFoundationDeathNeurogenesisGeneratorQuanted::Float64(value)
    }
}

impl<FIQ> TryFrom<NMFoundationDeathNeurogenesisGeneratorQuanted<FIQ>> for ProxyNMNeuroGenesisFromGenerator<FIQ, NMFoundationDeathQuantFloat64>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFoundationDeathQuantKind>;

    fn try_from(value: NMFoundationDeathNeurogenesisGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationDeathNeurogenesisGeneratorQuanted::Float64(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationDeathQuantFloat64::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

//endregion

//endregion

//region Defaults

/// Generates the scratch and variance data of `FoundationDeath` neurons
#[derive(Clone, Debug)]
pub struct FoundationDeathNeurogenesisDefaults<FIQ, NM>(PhantomData<fn() -> (FIQ, NM)>);

impl<FIQ, NM> NeurogenesisScratchVarianceGenerator<FIQ, NM> for FoundationDeathNeurogenesisDefaults<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NMFoundationDeathTrait,
{
    fn static_get_cortical_scratch(
        _layout: &NeuronLayoutOf<NM, FIQ>,
        _cortical_properties: &NM::CorticalDataProperties,
        _cortical_internal: &NM::CorticalDataInternal,
    ) -> NM::CorticalDataScratch {
        todo!("FoundationDeath cortical scratch")
    }

    fn static_get_neuron_scratch(
        _index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        _layout: &NeuronLayoutOf<NM, FIQ>,
        _internal_neuron_data: &NM::NeuronDataInternal,
    ) -> NM::NeuronDataScratch {
        todo!("FoundationDeath neuron scratch")
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

impl<NM: NMFoundationDeathTrait> CorticalDataVarianceSpecification<NM> for NMFoundationDeathDataCorticalVariance<NM> {
    fn neuron_variance_at<FIQ: FeagiIndexQuantization>(
        &self,
        _layout: &NeuronLayoutOf<NM, FIQ>,
        _index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> NMFoundationDeathDataNeuronVariance<NM> {
        NMFoundationDeathDataNeuronVariance::new()
    }
}

//endregion

//region Storage and StorageOrGenerator Conversions

//region Storage

impl<FIQ> From<ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationDeathQuantFloat16>> for NMFoundationDeathNeurogenesisStorageQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationDeathQuantFloat16>) -> Self {
        NMFoundationDeathNeurogenesisStorageQuanted::Float16(value)
    }
}

impl<FIQ> TryFrom<NMFoundationDeathNeurogenesisStorageQuanted<FIQ>> for ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationDeathQuantFloat16>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFoundationDeathQuantKind>;

    fn try_from(value: NMFoundationDeathNeurogenesisStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationDeathNeurogenesisStorageQuanted::Float16(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationDeathQuantFloat16::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationDeathQuantBFloat16>> for NMFoundationDeathNeurogenesisStorageQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationDeathQuantBFloat16>) -> Self {
        NMFoundationDeathNeurogenesisStorageQuanted::BFloat16(value)
    }
}

impl<FIQ> TryFrom<NMFoundationDeathNeurogenesisStorageQuanted<FIQ>> for ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationDeathQuantBFloat16>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFoundationDeathQuantKind>;

    fn try_from(value: NMFoundationDeathNeurogenesisStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationDeathNeurogenesisStorageQuanted::BFloat16(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationDeathQuantBFloat16::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationDeathQuantFloat32>> for NMFoundationDeathNeurogenesisStorageQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationDeathQuantFloat32>) -> Self {
        NMFoundationDeathNeurogenesisStorageQuanted::Float32(value)
    }
}

impl<FIQ> TryFrom<NMFoundationDeathNeurogenesisStorageQuanted<FIQ>> for ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationDeathQuantFloat32>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFoundationDeathQuantKind>;

    fn try_from(value: NMFoundationDeathNeurogenesisStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationDeathNeurogenesisStorageQuanted::Float32(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationDeathQuantFloat32::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationDeathQuantFloat64>> for NMFoundationDeathNeurogenesisStorageQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationDeathQuantFloat64>) -> Self {
        NMFoundationDeathNeurogenesisStorageQuanted::Float64(value)
    }
}

impl<FIQ> TryFrom<NMFoundationDeathNeurogenesisStorageQuanted<FIQ>> for ProxyNMNeuroGenesisFromStorage<FIQ, NMFoundationDeathQuantFloat64>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFoundationDeathQuantKind>;

    fn try_from(value: NMFoundationDeathNeurogenesisStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationDeathNeurogenesisStorageQuanted::Float64(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationDeathQuantFloat64::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

//endregion

//region Storage Or Generator

impl<FIQ> From<ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationDeathQuantFloat16>> for NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationDeathQuantFloat16>) -> Self {
        NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted::Float16(value)
    }
}

impl<FIQ> TryFrom<NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>> for ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationDeathQuantFloat16>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFoundationDeathQuantKind>;

    fn try_from(value: NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted::Float16(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationDeathQuantFloat16::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationDeathQuantBFloat16>> for NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationDeathQuantBFloat16>) -> Self {
        NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted::BFloat16(value)
    }
}

impl<FIQ> TryFrom<NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>> for ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationDeathQuantBFloat16>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFoundationDeathQuantKind>;

    fn try_from(value: NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted::BFloat16(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationDeathQuantBFloat16::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationDeathQuantFloat32>> for NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationDeathQuantFloat32>) -> Self {
        NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted::Float32(value)
    }
}

impl<FIQ> TryFrom<NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>> for ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationDeathQuantFloat32>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFoundationDeathQuantKind>;

    fn try_from(value: NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted::Float32(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationDeathQuantFloat32::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

impl<FIQ> From<ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationDeathQuantFloat64>> for NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationDeathQuantFloat64>) -> Self {
        NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted::Float64(value)
    }
}

impl<FIQ> TryFrom<NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>> for ProxyNMNeurogenesisFromStorageOrByGenerator<FIQ, NMFoundationDeathQuantFloat64>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFoundationDeathQuantKind>;

    fn try_from(value: NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFoundationDeathNeurogenesisStorageOrGeneratorQuanted::Float64(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFoundationDeathQuantFloat64::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

//endregion

//endregion
