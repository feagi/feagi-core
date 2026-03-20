

// TODO a lot of these should have additional bounds checking when in debug mode



/// Converts an index over a set of dimensions into a coordinate
pub fn relative_index_to_coordinate<Quant>(relative_index: &Quant, dimensions: &NeuronVoxelDimensions<Quant>, neurons_per_voxel: Quant) -> NeuronVoxelCoordinates<Quant> where
    Quant: QuantizableUInt,
{
    // TODO debug bounds checking
    let div_index = relative_index / neurons_per_voxel;
    let x = div_index % dimensions.x;
    let y = (div_index / dimensions.x) % dimensions.y;
    let z = div_index / (dimensions.x * dimensions.y);
    NeuronVoxelCoordinates::new(x, y, z)
}

/// Converts an index over a set of dimensions into a coordinate in place
pub fn relative_index_to_coordinate_in_place<Quant>(relative_index: &Quant, dimensions: &NeuronVoxelDimensions<Quant>, neurons_per_voxel: Quant, out: &mut NeuronVoxelCoordinates<Quant>) where
    Quant: QuantizableUInt,
{
    // TODO debug bounds checking
    let div_index = relative_index / neurons_per_voxel;
    out.x = div_index % dimensions.x;
    out.y = (div_index / dimensions.x) % dimensions.y;
    out.z = div_index / (dimensions.x * dimensions.y);
}