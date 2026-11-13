//! How the scratch and variance data of `FeagiAdvanced` neurons is generated: the
//! `NeuronModel::NeurogenesisScratchVarianceDefaults` of every quantization level of the model. Its
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

//region Scratch Variance

/// Generates the scratch and variance data of `FeagiAdvanced` neurons
#[derive(Clone, Debug)]
pub struct FeagiAdvancedScratchVarianceDefaults<FIQ, NM>(PhantomData<fn() -> (FIQ, NM)>);

impl<FIQ, NM> NeurogenesisScratchVarianceDefaults<FIQ, NM> for FeagiAdvancedScratchVarianceDefaults<FIQ, NM>
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
        let coordinates = layout
            .get_neuron_layout_context(index)
            .expect("Neuron index during neurogenesis must come from this layout"); // TODO Address This Properly
        let [x, y, z, _d] = coordinates.as_slice();

        let firing_threshold_offset = self.firing_threshold_increment_x * CorticalAreaNeuronPotential::from_quantized_unsigned_integer(*x)
            + self.firing_threshold_increment_y * CorticalAreaNeuronPotential::from_quantized_unsigned_integer(*y)
            + self.firing_threshold_increment_z * CorticalAreaNeuronPotential::from_quantized_unsigned_integer(*z);

        NMFeagiAdvancedDataNeuronVariance { firing_threshold_offset }
    }
}

//endregion
