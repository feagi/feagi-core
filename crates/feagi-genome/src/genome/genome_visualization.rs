use crate::basis::cortical_area::cortical_identifiers::CorticalIdentifier;
use feagi_connectome::feagi_connectome_neural::neural_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome::feagi_connectome_neural::neural_basis::visualization::NeuronVisualizationWordIndex;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct GenomeVisualizationContext<FIQ: FeagiIndexQuantization> {
    contexts: Vec<Option<(CorticalIdentifier, NeuronVisualizationWordIndex<FIQ::NeuronIndexQuant>)>>
}

