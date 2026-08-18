// TODO this should be macro generated

use crate::cortical_area::neuron_model_implementations::feagi_advanced::composers::FeagiAdvancedModelCorticalWriter;
use crate::cortical_area::neuron_model_implementations::feagi_advanced::quantization::FeagiAdvancedModelStandardQuant;
use serde::{Deserialize, Serialize};

/// This is the enum that is used to define what cortical area to create and how. All neuron models
/// have a "raw" method defined that allows creating from data directly, but can also have their own
/// various default implementations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CorticalWriterByModelQuant { // TODO we may want to have a break down by neuron layout
    FeagiAdvanced(FeagiAdvancedModelWriter),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeagiAdvancedModelWriter {
    Standard(FeagiAdvancedModelCorticalWriter<FeagiAdvancedModelStandardQuant>),
}

impl From<FeagiAdvancedModelCorticalWriter<FeagiAdvancedModelStandardQuant>> for CorticalWriterByModelQuant {
    fn from(value: FeagiAdvancedModelCorticalWriter<FeagiAdvancedModelStandardQuant>) -> Self {
        CorticalWriterByModelQuant::FeagiAdvanced(FeagiAdvancedModelWriter::Standard(value))
    }
}
