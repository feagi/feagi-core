use crate::basis::cortical_area::CorticalIdentifier;
use serde::{Deserialize, Serialize};
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome::connectome_basis::visualization::NeuronVisualizationWordIndex;

#[derive(Clone, Serialize, Deserialize)]
pub struct GenomeVisualizationContext<FIQ: FeagiIndexQuantization> {
    contexts: Vec<Option<(CorticalIdentifier, NeuronVisualizationWordIndex<FIQ::NeuronIndexQuant>)>>
}

