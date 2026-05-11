use crate::genomic::cortical_area::CorticalID;
use crate::neuron_collections::common_neuron_structs::{NeuronVoxelCoordinate, NeuronVoxelDimensions};

#[derive(Debug)]
pub enum FeagiStructuresNeuronVoxelError {
    NeuronIndexOutOfRange{context: &'static str, given_neuron_index: usize, range: usize},
    NeuronCoordinateOutOfRange{context: &'static str,given_neuron_coordinate: NeuronVoxelCoordinate<u32>, range: NeuronVoxelDimensions<u32>},
    IncompatibleNeuronDataFormat{context: &'static str},
    NoCorticalIDInNeuronCollection{context: &'static str, cortical_id: CorticalID},
    BadParameters{context: &'static str,},
    InternalError{context: &'static str,},
}

#[cfg(feature = "alloc")]
impl core::fmt::Display for FeagiStructuresNeuronVoxelError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NeuronIndexOutOfRange {
                context,
                given_neuron_index,
                range,
            } => {
                write!(
                    f,
                    "neuron index out of range ({context}): index {given_neuron_index}, exclusive upper bound {range}"
                )
            }
            Self::NeuronCoordinateOutOfRange {
                context,
                given_neuron_coordinate,
                range,
            } => {
                write!(
                    f,
                    "neuron coordinate out of range ({context}): coordinate {given_neuron_coordinate}, bounds {range}"
                )
            }
            Self::IncompatibleNeuronDataFormat { context } => {
                write!(f, "incompatible neuron data format: {context}")
            }
            Self::NoCorticalIDInNeuronCollection {
                context,
                cortical_id,
            } => {
                write!(
                    f,
                    "no cortical ID in neuron collection ({context}): cortical_id {cortical_id}"
                )
            }
            Self::BadParameters { context } => write!(f, "bad parameters: {context}"),
            Self::InternalError { context } => write!(f, "internal error: {context}"),
        }
    }
}

// TODO
//#[cfg(all(feature = "alloc", feature = "std"))]
//impl std::error::Error for FeagiNeuronVoxelError {}