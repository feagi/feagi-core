use serde::{Deserialize, Serialize};
use feagi_basis::collections::spatial_context::integer_signed::SignedIntegerSpatial;

/// Specifies that a given genome object has positioning context in 2D and 3D space
pub trait GenomeObjectHasPositioning {
    fn get_genome_positioning(&self) -> &GenomeObjectPositioning;

    fn get_genome_positioning_mut(&mut self) -> &mut GenomeObjectPositioning;

    // TODO default impl fns for getting positions, setting positions, checking things, etc
}

// TODO some sort of sizing thing should be a separate trait

/// Defines the position of a Genome Object in 3D and 2D (CB) space relative to its parent brain region
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenomeObjectPositioning {
    pub position_2d: GenomeObjectCoordinate2D,
    pub position_3d: GenomeObjectCoordinate3D,
}

/// Defines the 3d coordinate of a genome object in relation to its parent brain region
pub type GenomeObjectCoordinate3D =
SignedIntegerSpatial<i64, 3>;

/// Defines the 2d coordinate of a genome object in relation to its parent brain region (circuit)
pub type GenomeObjectCoordinate2D =
SignedIntegerSpatial<i64, 2>;
