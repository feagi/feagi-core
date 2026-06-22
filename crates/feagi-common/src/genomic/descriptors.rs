use crate::{create_quantized_signed_integer_wrapper, create_quantized_spatial_signed_coordinate_2d_wrapper, create_quantized_spatial_signed_coordinate_3d_wrapper};

create_quantized_signed_integer_wrapper!(GenomeAxis, i32);

// Represents a 2D position within the Circuit Builder of brain visualizer
create_quantized_spatial_signed_coordinate_2d_wrapper!(GenomeCoordinate2D, i32, GenomeAxis, GenomeAxis);

// Represents a 3D position within the Cortical Area Viewer of Brain Visualizer
create_quantized_spatial_signed_coordinate_3d_wrapper!(GenomeCoordinate3D, i32, GenomeAxis, GenomeAxis, GenomeAxis);
