// TODO this should be macro generated

use crate::cortical_area::implementations::feagi_advanced::quantization::FeagiAdvancedModelStandardQuant;
use crate::cortical_area::implementations::feagi_advanced::writer::FeagiAdvancedModelCorticalWriter;

/// This is the enum that is used to define what cortical area to create and how. All neuron models
/// have a "raw" method defined that allows creating from data directly, but can also have their own
/// various default implementations
pub enum CorticalWriterByModelQuant {
    FeagiAdvanced(FeagiAdvancedModelWriter),
}


pub enum FeagiAdvancedModelWriter {
    Standard(FeagiAdvancedModelCorticalWriter<FeagiAdvancedModelStandardQuant>),
}

impl From<FeagiAdvancedModelCorticalWriter<FeagiAdvancedModelStandardQuant>> for CorticalWriterByModelQuant {
    fn from(value: FeagiAdvancedModelCorticalWriter<FeagiAdvancedModelStandardQuant>) -> Self {
        CorticalWriterByModelQuant::FeagiAdvanced(FeagiAdvancedModelWriter::Standard(value))
    }
}
