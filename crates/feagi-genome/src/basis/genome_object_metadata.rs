use crate::genome_object_positioning::{GenomeObjectCoordinate2D, GenomeObjectCoordinate3D};

/// Stores data that all genome objects have. Bears no computational consequence.
pub struct GenomeObjectMetadata {
    pub name: String,
    pub description: String,
    pub position_3d: GenomeObjectCoordinate3D,
    pub position_2d: GenomeObjectCoordinate2D,
}

// TODO sizing limits?