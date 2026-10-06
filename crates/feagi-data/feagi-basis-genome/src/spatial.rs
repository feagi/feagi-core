

/// Along the X, Y, or Z axis, a position of a Genome Object within the Genome per axis is an i64 value
pub type GenomeAxisPosition = quantized_wrappers::GenomeAxisPosition<i64>;

/// The 3D position of a Genome Object within the Genome (within its parent brain region)
pub type GenomeCoordinate = quantized_wrappers::GenomeCoordinate<i64>;


// We make use of the quantized wrapper system, but we only want to expose the 64 bit values
mod quantized_wrappers {
    use feagi_basis_quantization::{create_wrapped_quantized_signed_integer, create_wrapped_signed_integer_spatial};
    
    create_wrapped_quantized_signed_integer!(
    pub GenomeAxisPosition
    );

    create_wrapped_signed_integer_spatial!(
        pub GenomeCoordinate,
        3,
        (0, x, GenomeAxisPosition), (1, y, GenomeAxisPosition), (2, z, GenomeAxisPosition)
    );
    
}





