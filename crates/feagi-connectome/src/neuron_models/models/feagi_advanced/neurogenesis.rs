//! How the scratch and variance data of `FeagiAdvanced` neurons is generated: the
//! `NeuronModel::NeurogenesisDefaults` of every quantization level of the model. Its
//! `NeuronModel::NeurogenesisGenerator` is the shared `NeurogenesisDefaultsGenerator`
// TODO these are stubs, fill them in

use crate::neuron_models::models::feagi_advanced::quantizations::{NMFeagiAdvancedNeurogenesisGeneratorQuanted, NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted, NMFeagiAdvancedNeurogenesisStorageQuanted, NMFeagiAdvancedQuantKind, NMFeagiAdvancedQuantStandard32Bit};
use crate::neuron_models::models::feagi_advanced::NMFeagiAdvancedTrait;
use crate::neuron_models::models::feagi_advanced::{NMFeagiAdvancedDataCorticalVariance, NMFeagiAdvancedDataNeuronVariance};
use crate::neuron_models::models::{NeuronModelEnumConversionError, NeurogenesisByGeneratorTypeOf, NeurogenesisFromStorageOf, NeurogenesisFromStorageOrByGeneratorOf};
use core::marker::PhantomData;
use feagi_basis::neurons::{CorticalAreaNeuronLocalIndex, CorticalAreaNeuronPotential};
use feagi_basis::prelude::QuantizedDecimalTrait;
use feagi_connectome_neural::neural_basis::neuron_model::model_data::CorticalDataVarianceSpecification;
use feagi_connectome_neural::neural_basis::neuron_model::neurogenesis::NeurogenesisScratchVarianceGenerator;
use feagi_connectome_neural::neural_basis::neuron_model::neuron_layout::{NeuronLayout, NeuronLayoutOf};
use feagi_connectome_neural::neural_basis::neuron_model::NeuronModel;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
//region Generator

// NOTE: we are using NeurogenesisGeneratorUniformDefault, not a specific impl for this model

//region Impls

impl<FIQ> From<NeurogenesisByGeneratorTypeOf<FIQ, NMFeagiAdvancedQuantStandard32Bit>> for NMFeagiAdvancedNeurogenesisGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NeurogenesisByGeneratorTypeOf<FIQ, NMFeagiAdvancedQuantStandard32Bit>) -> Self {
        NMFeagiAdvancedNeurogenesisGeneratorQuanted::Standard32Bit(value)
    }
}

impl<FIQ> TryFrom<NMFeagiAdvancedNeurogenesisGeneratorQuanted<FIQ>> for NeurogenesisByGeneratorTypeOf<FIQ, NMFeagiAdvancedQuantStandard32Bit>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFeagiAdvancedQuantKind>;

    fn try_from(value: NMFeagiAdvancedNeurogenesisGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        match value {
            NMFeagiAdvancedNeurogenesisGeneratorQuanted::Standard32Bit(v) => {
                Ok(v)
            },
        }
    }
}


//endregion


//endregion


//region Scratch Variance

/// Generates the scratch and variance data of `FeagiAdvanced` neurons
#[derive(Clone, Debug)]
pub struct FeagiAdvancedScratchVarianceGenerator<FIQ, NM>(PhantomData<fn() -> (FIQ, NM)>);

impl<FIQ, NM> NeurogenesisScratchVarianceGenerator<FIQ, NM> for FeagiAdvancedScratchVarianceGenerator<FIQ, NM>
where
    FIQ: FeagiIndexQuantization,
    NM: NMFeagiAdvancedTrait,
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

impl<NM: NMFeagiAdvancedTrait> CorticalDataVarianceSpecification<NM> for NMFeagiAdvancedDataCorticalVariance<NM> {
    fn neuron_variance_at<FIQ: FeagiIndexQuantization>(
        &self,
        layout: &NeuronLayoutOf<FIQ, NM>,
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
    ) -> NMFeagiAdvancedDataNeuronVariance<NM> {
        let coordinates = layout.get_neuron_layout_context(index);
        let [x, y, z, _d] = coordinates.as_slice();
        NMFeagiAdvancedDataNeuronVariance::new(
            self.firing_threshold_increment_x * CorticalAreaNeuronPotential::from_quantized_unsigned_integer(*x)
                + self.firing_threshold_increment_y * CorticalAreaNeuronPotential::from_quantized_unsigned_integer(*y)
                + self.firing_threshold_increment_z * CorticalAreaNeuronPotential::from_quantized_unsigned_integer(*z),
        )
    }
}

//endregion

//region Storage and StorageOrGenerator Conversions 

//region Storage

impl<FIQ> From<NeurogenesisFromStorageOf<FIQ, NMFeagiAdvancedQuantStandard32Bit>> for NMFeagiAdvancedNeurogenesisStorageQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NeurogenesisFromStorageOf<FIQ, NMFeagiAdvancedQuantStandard32Bit>) -> Self {
        NMFeagiAdvancedNeurogenesisStorageQuanted::Standard32Bit(value)
    }
}

impl<FIQ> TryFrom<NMFeagiAdvancedNeurogenesisStorageQuanted<FIQ>> for NeurogenesisFromStorageOf<FIQ, NMFeagiAdvancedQuantStandard32Bit>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFeagiAdvancedQuantKind>;

    fn try_from(value: NMFeagiAdvancedNeurogenesisStorageQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFeagiAdvancedNeurogenesisStorageQuanted::Standard32Bit(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFeagiAdvancedQuantStandard32Bit::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

//endregion

//region Storage Or Generator

impl<FIQ> From<NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMFeagiAdvancedQuantStandard32Bit>> for NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted<FIQ>
where
    FIQ: FeagiIndexQuantization,
{
    fn from(value: NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMFeagiAdvancedQuantStandard32Bit>) -> Self {
        NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted::Standard32Bit(value)
    }
}

impl<FIQ> TryFrom<NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted<FIQ>> for NeurogenesisFromStorageOrByGeneratorOf<FIQ, NMFeagiAdvancedQuantStandard32Bit>
where
    FIQ: FeagiIndexQuantization,
{
    type Error = NeuronModelEnumConversionError<NMFeagiAdvancedQuantKind>;

    fn try_from(value: NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted<FIQ>) -> Result<Self, Self::Error> {
        let actual = value.quant_kind();
        match value {
            NMFeagiAdvancedNeurogenesisStorageOrGeneratorQuanted::Standard32Bit(v) => {
                Ok(v)
            },
            _ => {
                Err(NeuronModelEnumConversionError::IncompatibleUnwrap {
                    expected: NMFeagiAdvancedQuantStandard32Bit::QUANT_KIND,
                    actual,
                })
            }
        }
    }
}

//endregion

//endregion