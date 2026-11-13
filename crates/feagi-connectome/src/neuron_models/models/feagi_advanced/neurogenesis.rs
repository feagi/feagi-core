//! How the scratch and variance data of `FeagiAdvanced` neurons is generated: the
//! `NeuronModel::NeurogenesisDefaults` of every quantization level of the model. Its
//! `NeuronModel::NeurogenesisByGenerator` is the shared `NeurogenesisGeneratorUniformDefault`
// TODO these are stubs, fill them in

use crate::feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::scratch_variance_defaults::NeurogenesisScratchVarianceDefaults;
use crate::neuron_models::models::feagi_advanced::NMFeagiAdvancedTrait;
use crate::neuron_models::models::feagi_advanced::{NMFeagiAdvancedDataCorticalVariance, NMFeagiAdvancedDataNeuronVariance};
use core::marker::PhantomData;
use feagi_basis::neurons::{CorticalAreaNeuronLocalIndex, CorticalAreaNeuronPotential};
use feagi_basis::prelude::QuantizedDecimalTrait;
use feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataVarianceSpecification;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayout, NeuronLayoutOf};
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use crate::neuron_models::models::feagi_advanced::quantizations::{NMFeagiAdvancedNeurogenesisGeneratorQuanted, NMFeagiAdvancedNeurogenesisValidatedGeneratorQuanted, NMFeagiAdvancedQuantKind, NMFeagiAdvancedQuantStandard32Bit};
#[cfg(feature = "alloc")]
use crate::neuron_models::models::feagi_advanced::quantizations::{NMFeagiAdvancedNeurogenesisStorageQuanted, NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted, NMFeagiAdvancedNeurogenesisValidatedStorageQuanted, NMFeagiAdvancedNeurogenesisValidatedStorageOrGeneratorQuanted};
use crate::neuron_models::models::{NeuronModelEnumConversionError, NeurogenesisByGeneratorTypeOf};
#[cfg(feature = "alloc")]
use crate::neuron_models::models::{NeurogenesisFromStorageOf, NeurogenesisFromStorageOrByGeneratorOf};
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::validation::ValidatedNeurogenesisGenerator;
#[cfg(feature = "alloc")]
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::validation::{ValidatedNeurogenesisStorage, ValidatedNeurogenesisStorageOrGenerator};

//region Scratch Variance

/// Generates the scratch and variance data of `FeagiAdvanced` neurons
#[derive(Clone, Debug)]
pub struct FeagiAdvancedScratchVarianceGenerator<FIQ, NM>(PhantomData<fn() -> (FIQ, NM)>);

impl<FIQ, NM> NeurogenesisScratchVarianceDefaults<FIQ, NM> for FeagiAdvancedScratchVarianceGenerator<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NMFeagiAdvancedTrait,
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

impl<NM: NMFeagiAdvancedTrait> CorticalDataVarianceSpecification<NM> for NMFeagiAdvancedDataCorticalVariance<NM> {
    fn neuron_variance_at<FIQ: FeagiIndexQuantization>(
        &self,
        layout: &NeuronLayoutOf<FIQ, NM>,
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> NMFeagiAdvancedDataNeuronVariance<NM> {
        let coordinates = layout.get_neuron_layout_context(index)
            .expect("Neuron index during neurogenesis must come from this layout"); // TODO Address This Properly
        let [x, y, z, _d] = coordinates.as_slice();

        let firing_threshold_offset = self.firing_threshold_increment_x * CorticalAreaNeuronPotential::from_quantized_unsigned_integer(*x)
            + self.firing_threshold_increment_y * CorticalAreaNeuronPotential::from_quantized_unsigned_integer(*y)
            + self.firing_threshold_increment_z * CorticalAreaNeuronPotential::from_quantized_unsigned_integer(*z);
        
        NMFeagiAdvancedDataNeuronVariance {
            firing_threshold_offset,
        }
    }
}

//endregion

//region Quant Level Conversions

impl<FIQ: FeagiIndexQuantization> From<NeurogenesisByGeneratorTypeOf<FIQ, NMFeagiAdvancedQuantStandard32Bit>> for NMFeagiAdvancedNeurogenesisGeneratorQuanted<FIQ> {
    fn from(value: NeurogenesisByGeneratorTypeOf<FIQ, NMFeagiAdvancedQuantStandard32Bit>) -> Self {
        Self::Standard32Bit(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMFeagiAdvancedNeurogenesisGeneratorQuanted<FIQ>> for NeurogenesisByGeneratorTypeOf<FIQ, NMFeagiAdvancedQuantStandard32Bit> {
    type Error = NeuronModelEnumConversionError<NMFeagiAdvancedQuantKind>;

    fn try_from(value: NMFeagiAdvancedNeurogenesisGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMFeagiAdvancedNeurogenesisGeneratorQuanted::Standard32Bit(v) => Ok(v),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NeurogenesisFromStorageOf<FIQ, NMFeagiAdvancedQuantStandard32Bit>> for NMFeagiAdvancedNeurogenesisStorageQuanted<FIQ> {
    fn from(value: NeurogenesisFromStorageOf<FIQ, NMFeagiAdvancedQuantStandard32Bit>) -> Self {
        Self::Standard32Bit(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMFeagiAdvancedNeurogenesisStorageQuanted<FIQ>> for NeurogenesisFromStorageOf<FIQ, NMFeagiAdvancedQuantStandard32Bit> {
    type Error = NeuronModelEnumConversionError<NMFeagiAdvancedQuantKind>;

    fn try_from(value: NMFeagiAdvancedNeurogenesisStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMFeagiAdvancedNeurogenesisStorageQuanted::Standard32Bit(v) => Ok(v),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMFeagiAdvancedQuantStandard32Bit>> for NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted<FIQ> {
    fn from(value: NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMFeagiAdvancedQuantStandard32Bit>) -> Self {
        Self::Standard32Bit(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMFeagiAdvancedQuantStandard32Bit> {
    type Error = NeuronModelEnumConversionError<NMFeagiAdvancedQuantKind>;

    fn try_from(value: NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted::Standard32Bit(v) => Ok(v),
        }
    }
}

impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisGenerator<FIQ, NMFeagiAdvancedQuantStandard32Bit>> for NMFeagiAdvancedNeurogenesisValidatedGeneratorQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisGenerator<FIQ, NMFeagiAdvancedQuantStandard32Bit>) -> Self {
        Self::Standard32Bit(value)
    }
}

impl<FIQ: FeagiIndexQuantization> TryFrom<NMFeagiAdvancedNeurogenesisValidatedGeneratorQuanted<FIQ>> for ValidatedNeurogenesisGenerator<FIQ, NMFeagiAdvancedQuantStandard32Bit> {
    type Error = NeuronModelEnumConversionError<NMFeagiAdvancedQuantKind>;

    fn try_from(value: NMFeagiAdvancedNeurogenesisValidatedGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMFeagiAdvancedNeurogenesisValidatedGeneratorQuanted::Standard32Bit(v) => Ok(v),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisStorage<FIQ, NMFeagiAdvancedQuantStandard32Bit>> for NMFeagiAdvancedNeurogenesisValidatedStorageQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisStorage<FIQ, NMFeagiAdvancedQuantStandard32Bit>) -> Self {
        Self::Standard32Bit(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMFeagiAdvancedNeurogenesisValidatedStorageQuanted<FIQ>> for ValidatedNeurogenesisStorage<FIQ, NMFeagiAdvancedQuantStandard32Bit> {
    type Error = NeuronModelEnumConversionError<NMFeagiAdvancedQuantKind>;

    fn try_from(value: NMFeagiAdvancedNeurogenesisValidatedStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMFeagiAdvancedNeurogenesisValidatedStorageQuanted::Standard32Bit(v) => Ok(v),
        }
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> From<ValidatedNeurogenesisStorageOrGenerator<FIQ, NMFeagiAdvancedQuantStandard32Bit>> for NMFeagiAdvancedNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ> {
    fn from(value: ValidatedNeurogenesisStorageOrGenerator<FIQ, NMFeagiAdvancedQuantStandard32Bit>) -> Self {
        Self::Standard32Bit(value)
    }
}

#[cfg(feature = "alloc")]
impl<FIQ: FeagiIndexQuantization> TryFrom<NMFeagiAdvancedNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>> for ValidatedNeurogenesisStorageOrGenerator<FIQ, NMFeagiAdvancedQuantStandard32Bit> {
    type Error = NeuronModelEnumConversionError<NMFeagiAdvancedQuantKind>;

    fn try_from(value: NMFeagiAdvancedNeurogenesisValidatedStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMFeagiAdvancedNeurogenesisValidatedStorageOrGeneratorQuanted::Standard32Bit(v) => Ok(v),
        }
    }
}

//endregion
