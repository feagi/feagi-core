

pub enum FeagiNPUDataError {
    NeuronIndexOutOfRange{given_neuron_index: usize, range: usize},
    NeuronCoordinateOutOfRange{given_neuron_coordinate: UnsignedCoordinate3DUSize, range: Dimension3DUSize},
    CorticalIndexOutOfRange{given_cortical_index: usize, range: usize},
    InternalError(),
}