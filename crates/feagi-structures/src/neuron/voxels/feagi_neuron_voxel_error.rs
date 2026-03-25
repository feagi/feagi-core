use crate::base_quantizable::coordinate::{Dimension3DUSize, UnsignedCoordinate3DUSize};

pub enum FeagiNeuronVoxelError {
    NeuronIndexOutOfRange{given_neuron_index: usize, range: usize},
    NeuronCoordinateOutOfRange{given_neuron_coordinate: UnsignedCoordinate3DUSize, range: Dimension3DUSize},
    IncompatibleNeuronDataFormat(),
    BadParameters(),
    InternalError(),
}