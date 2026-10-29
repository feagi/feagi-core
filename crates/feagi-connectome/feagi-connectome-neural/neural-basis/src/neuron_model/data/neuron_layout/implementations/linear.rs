use crate::neuron_model::data::neuron_layout::NeuronLayout;
use crate::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_basis::neurons::{CorticalAreaNeuronCount, CorticalAreaNeuronLocalIndex};
use serde::{Deserialize, Serialize};

/// Defines that the neurons are laid out in linear fashion
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NeuronLayoutLinear;

/// The data of [`NeuronLayoutLinear`]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NeuronLayoutLinearData<FIQ: FeagiIndexQuantization> {
    pub neuron_count: CorticalAreaNeuronCount<FIQ::NeuronIndexQuant>,
}

impl NeuronLayout for NeuronLayoutLinear {
    type Data<FIQ: FeagiIndexQuantization> = NeuronLayoutLinearData<FIQ>;
    type CorticalContext<FIQ: FeagiIndexQuantization> = CorticalAreaNeuronCount<FIQ::NeuronIndexQuant>;
    type PerNeuronContext<FIQ: FeagiIndexQuantization> = CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>;

    fn build_new<FIQ: FeagiIndexQuantization>(cortical_context: Self::CorticalContext<FIQ>) -> Self::Data<FIQ> {
        NeuronLayoutLinearData {
            neuron_count: cortical_context
        }
    }

    fn get_neuron_count<FIQ: FeagiIndexQuantization>(data: &Self::Data<FIQ>) -> CorticalAreaNeuronCount<FIQ::NeuronIndexQuant> {
        data.neuron_count
    }

    fn get_cortical_layout_context<FIQ: FeagiIndexQuantization>(data: &Self::Data<FIQ>) -> &Self::CorticalContext<FIQ> {
        &data.neuron_count
    }

    fn get_neuron_layout_context<FIQ: FeagiIndexQuantization>(_data: &Self::Data<FIQ>, neuron_index: &CorticalAreaNeuronLocalIndex<FIQ::NeuronIndexQuant>) -> Self::PerNeuronContext<FIQ> {
        neuron_index.clone() // no further details lol
    }

    fn update_cortical_context<FIQ: FeagiIndexQuantization>(data: &mut Self::Data<FIQ>, cortical_context: Self::CorticalContext<FIQ>) {
        data.neuron_count = cortical_context
    }
}
