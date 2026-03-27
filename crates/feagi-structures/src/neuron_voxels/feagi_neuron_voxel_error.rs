use crate::base_quantizable::coordinate::{Dimension3DType, UnsignedCoordinate3DType};
use crate::genomic::cortical_area::CorticalID;

pub enum FeagiNeuronVoxelError {
    NeuronIndexOutOfRange{context: &'static str, given_neuron_index: usize, range: usize},
    NeuronCoordinateOutOfRange{context: &'static str,given_neuron_coordinate: UnsignedCoordinate3DType<u32>, range: Dimension3DType<u32>},
    IncompatibleNeuronDataFormat{context: &'static str},
    NoCorticalIDInNeuronCollection{context: &'static str, cortical_id: CorticalID},
    BadParameters{context: &'static str,},
    InternalError{context: &'static str,},
}