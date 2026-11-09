use feagi_basis::collections::spatial_context::integer_signed::SignedIntegerSpatial;
use serde::{Deserialize, Serialize};

//region Positioning

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

impl Default for GenomeObjectPositioning {
    fn default() -> Self {
        Self {
            position_2d: GenomeObjectCoordinate2D::new_zero(),
            position_3d: GenomeObjectCoordinate3D::new_zero()
        }
    }
}

/// Defines the 3d coordinate of a genome object in relation to its parent brain region
pub type GenomeObjectCoordinate3D =
SignedIntegerSpatial<i64, 3>;

/// Defines the 2d coordinate of a genome object in relation to its parent brain region (circuit)
pub type GenomeObjectCoordinate2D =
SignedIntegerSpatial<i64, 2>;

//endregion

//region Metadata

// TODO sizing limits?
/// Stores data that all genome objects have. Bears no computational consequence.
pub struct GenomeObjectMetadata {
    pub name: String,
    pub description: String,
}

//endregion