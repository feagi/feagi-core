use feagi_basis::collections::spatial_context::integer_signed::SignedIntegerSpatial;

/// Defines the 3d coordinate of a genome object in relation to its parent brain region
pub type GenomeObjectCoordinate3D =
SignedIntegerSpatial<i64, 3>;

/// Defines the 2d coordinate of a genome object in relation to its parent brain region (circuit)
pub type GenomeObjectCoordinate2D =
SignedIntegerSpatial<i64, 2>;
