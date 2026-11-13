use crate::neuron_model::neuron_layout::NeuronLayoutTag;
use crate::neuron_model::NeuronModel;
use crate::quantization_levels::FeagiIndexQuantization;
use feagi_basis::neurons::CorticalAreaNeuronLocalIndex;

/// Shared static generator for initing scratch and variance data defaults
pub trait NeurogenesisScratchVarianceDefaults<FIQ, NM>: Clone + core::fmt::Debug + Send + Sync
where
    FIQ: FeagiIndexQuantization,
    NM: NeuronModel
{
    fn static_get_cortical_scratch(
        layout: &<NM::NeuronLayoutTag as NeuronLayoutTag>::Layout<FIQ>,
        cortical_properties: &NM::CorticalDataProperties,
        cortical_internal: &NM::CorticalDataInternal,
    ) -> NM::CorticalDataScratch;

    fn static_get_neuron_scratch(
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        layout: &<NM::NeuronLayoutTag as NeuronLayoutTag>::Layout<FIQ>,
        internal_neuron_data: &NM::NeuronDataInternal
    ) -> NM::NeuronDataScratch;

    fn static_get_neuron_variance(
        index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>,
        layout: &<NM::NeuronLayoutTag as NeuronLayoutTag>::Layout<FIQ>,
        variance: &NM::CorticalDataVariance
    ) -> NM::NeuronDataVariance;
}