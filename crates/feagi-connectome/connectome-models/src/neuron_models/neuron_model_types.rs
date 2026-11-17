use crate::neuron_models::models::{NMModeled, NMNeurogenesisValidatedStorageOrGenerator};
use connectome_basis::connectome_editing::ConnectomeNeuronModelTypes;
use connectome_basis::quantization_levels::FeagiIndexQuantization;

pub struct FeagiNeuronModelTypes;

impl<FIQ: FeagiIndexQuantization> ConnectomeNeuronModelTypes<FIQ> for FeagiNeuronModelTypes {
    type CorticalAreaNeurogenesis = NMModeled<NMNeurogenesisValidatedStorageOrGenerator<FIQ>>;
}

