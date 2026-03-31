// SPECIAL NOTE: ONLY EXPOSE THE I32 variants via the module as we do not care about other
// quantizable types! These are representing configurations in the genome which do not have
// quantizable sizes at all!

// Represents a 2D position within the Circuit Builder of brain visualizer
crate::define_signed_coordinate_2d_type_family!(GenomeCoordinate2D);
pub type GenomeCoordinate2DI32 = GenomeCoordinate2D<i32>;

// Represents a 3D position within the Cortical Area Viewer of Brain Visualizer
crate::define_signed_coordinate_3d_type_family!(GenomeCoordinate3D);

pub type GenomeCoordinate3DI32 = GenomeCoordinate3D<i32>;

