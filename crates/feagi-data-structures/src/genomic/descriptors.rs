use crate::{define_index, define_xy_coordinates, define_xyz_coordinates};

define_xy_coordinates!(
    GenomeCoordinate2D,
    i32,
    "GenomeCoordinate2D",
    "2D coordinate local to the FEAGI Genome space.

Represents a 2D position within the global genome coordinate system,
using signed integers to allow for negative coordinates and relative
positioning across the entire genome space.

# Usage
Used for 2D visualization positioning of brain regions and cortical areas
within the genome's 2D projection plane."
);

define_xyz_coordinates!(
    GenomeCoordinate3D,
    i32,
    "GenomeCoordinate3D",
    "3D coordinate local to the FEAGI Genome space.

Represents a position within the global genome coordinate system,
using signed integers to allow for negative coordinates and relative
positioning across the entire genome space.

# Usage
Used for absolute positioning of cortical areas within the genome
and for calculating spatial relationships between different brain regions."
);

// Alias for backward compatibility
pub type GenomeCoordinate = GenomeCoordinate3D;

define_index!(AgentDeviceIndex, u32,
"An index for a specific channel on a specific cortical group (or multiple). An alternate way to refer to channels"
);
