

pub enum FeagiNPUDataError {
    NeuronIndexOutOfRange{given_neuron_index: u32, range: u32},
    NeuronCoordinateOutOfRange{given_neuron_coordinate: UnsignedCoordinate3DU32, range: Dimension3DU32},
    InvalidCorticalIndex{given_cortical_index: u32},
    InternalError(),
}
