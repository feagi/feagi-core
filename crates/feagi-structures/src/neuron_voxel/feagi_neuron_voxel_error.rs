#[derive(Debug)]
pub enum FeagiNeuronVoxelError {
    NeuronVoxelIndexOutOfRange{context: &'static str, given_voxel_index: u32, range: u32},
    NeuronVoxelCoordinateOutOfRange{context: &'static str, given_voxel_coordinate: (u32, u32, u32), range: u32},
    InvalidVoxelDensity{context: &'static str},
    BadParameters{context: &'static str,},
    InternalError{context: &'static str,},
}