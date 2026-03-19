

#[cfg(not(feature = "support_64bit_indexing_quantization"))]
pub enum FeagiNPUDataError {
    NeuronIndexOutOfRange{given_neuron_index: u32, range: u32},
    NeuronCoordinateOutOfRange{given_neuron_coordinate: UnsignedCoordinate3DU32, range: Dimension3DU32},
    CorticalIndexOutOfRange{given_cortical_index: u32, range: u32},
    InternalError(),
}


#[cfg(feature = "support_64bit_indexing_quantization")]
pub enum FeagiNPUDataError {
    NeuronIndexOutOfRange{given_neuron_index: u64, range: u64},
    NeuronCoordinateOutOfRange{given_neuron_coordinate: UnsignedCoordinate3DU64, range: Dimension3DU64},
    CorticalIndexOutOfRange{given_cortical_index: u64, range: u64},
    InternalError(),
}