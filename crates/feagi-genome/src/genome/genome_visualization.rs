use crate::basis::cortical_area::cortical_identifiers::CorticalIdentifier;
use feagi_connectome::connectome_basis::quantization_levels::FeagiIndexQuantization;
use feagi_connectome::connectome_basis::visualization::NeuronVisualizationVoxelByteIndex;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct GenomeVisualizationContext<FIQ: FeagiIndexQuantization> {
    contexts: Vec<Option<(CorticalIdentifier, NeuronVisualizationVoxelByteIndex<FIQ::NeuronIndexQuant>)>>
}

