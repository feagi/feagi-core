use crate::descriptors::{Dimension3DUSize, UnsignedCoordinate3DUSize};

pub enum FeagiNeuronError {
    NeuronCoordinateOutOfRange{given_neuron_coordinate: UnsignedCoordinate3DUSize, range: Dimension3DUSize},
    IncompatibleNeuronDataFormat(),
    BadParameters(),
    InternalError(),
}