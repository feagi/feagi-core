use feagi_data::{create_quantized_signed_integer_wrapper, create_quantized_spatial_signed_coordinate_3d_wrapper};

create_quantized_signed_integer_wrapper!(GenomeAxis);
create_quantized_spatial_signed_coordinate_3d_wrapper!{
    GenomeCoordinate, GenomeAxis, GenomeAxis, GenomeAxis
}