use crate::cortical_area::CorticalIdentifier;
use feagi_connectome::quantization_levels::FeagiIndexQuantization;
use feagi_connectome::visualization::NeuronVisualizationWordIndex;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct GenomeVisualizationContext<FIQ: FeagiIndexQuantization> {
    contexts: Vec<Option<(CorticalIdentifier, NeuronVisualizationWordIndex<FIQ::NeuronIndexQuant>)>>
}

