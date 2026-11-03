use serde::{Deserialize, Serialize};
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_connectome::visualization::NeuronVisualizationWordIndex;
use crate::basis::specifications::cortical_area::cortical_id::CorticalID;

#[derive(Clone, Serialize, Deserialize)]
pub struct GenomeVisualizationContext<FIQ: FeagiIndexQuantization> {
    contexts: Vec<Option<(CorticalID, NeuronVisualizationWordIndex<FIQ::NeuronIndexQuant>)>>
}

