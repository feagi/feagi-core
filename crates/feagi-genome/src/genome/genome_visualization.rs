use crate::basis::cortical_area::cortical_id_legacy::CorticalIDLegacy;
use feagi_connectome::quantization_levels::feagi_index_quantization::FeagiIndexQuantization;
use feagi_connectome::visualization::NeuronVisualizationWordIndex;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct GenomeVisualizationContext<FIQ: FeagiIndexQuantization> {
    contexts: Vec<Option<(CorticalIDLegacy, NeuronVisualizationWordIndex<FIQ::NeuronIndexQuant>)>>
}

