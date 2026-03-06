use crate::common_descriptors::{Coordinate3D, Dimension3D};

pub enum FeagiNeuronError {
    NeuronCoordinateOutOfRange{given_neuron_coordinate: Coordinate3D, range: Dimension3D},
    IncompatibleNeuronDataFormat(),
    BadParameters(),
    InternalError(),
}