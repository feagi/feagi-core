use crate::{define_xyz_coordinates, define_index};




define_xyz_coordinates!(GenomeCoordinate, i32, "GenomeCoordinate",
    "Coordinate local to the FEAGI Genome space.

Represents a position within the global genome coordinate system,
using signed integers to allow for negative coordinates and relative
positioning across the entire genome space.

# Usage
Used for absolute positioning of cortical areas within the genome
and for calculating spatial relationships between different brain regions."
);



define_index!(AgentDeviceIndex, u32,
"An index for a specific channel on a specific cortical group (or multiple). An alternate way to refer to channels"
);


