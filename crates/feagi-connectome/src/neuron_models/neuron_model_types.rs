use crate::neuron_models::models::{NMModeled, NMNeurogenesisValidatedStorageOrGenerator};
use feagi_connectome_neural::neural_basis::connectome_editing::ConnectomeNeuronModelTypes;
use feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;

pub struct FeagiNeuronModelTypes;

impl<FIQ: FeagiIndexQuantization> ConnectomeNeuronModelTypes<FIQ> for FeagiNeuronModelTypes {
    type CorticalAreaNeurogenesis = NMModeled<NMNeurogenesisValidatedStorageOrGenerator<FIQ>>;
}

