use feagi_data::{create_quantized_signed_integer_wrapper, create_quantized_spatial_signed_coordinate_3d_wrapper};

pub type GenomeCoordAxis = GenomeCoordAxisQuantized<i32>;
pub type GenomeCoordinate = GenomeCoordinateQuantized<i32>;

// Make this private so we only expose the s32 axis
create_quantized_signed_integer_wrapper!(private GenomeCoordAxisQuantized);


create_quantized_spatial_signed_coordinate_3d_wrapper!{
    private GenomeCoordinateQuantized, GenomeCoordAxisQuantized, GenomeCoordAxisQuantized, GenomeCoordAxisQuantized // Make this private so we only expose the s32 coordinate
}