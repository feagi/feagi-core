use crate::FeagiDataError;
use crate::{define_index, define_nonzero_count, define_xyz_coordinates, define_xyz_dimensions};

//region Cortical Indexing

define_index!(
    CorticalUnitIndex,
    u16,
    "Index for grouping cortical units of the same type within a genome.

This index distinguishes between multiple instances of the same cortical type.
For example, multiple vision sensors would have different CorticalUnitIndex
values (0, 1, 2, etc.) while sharing the same base cortical type.

# Range
Values are 0-65535 (u16). IO cortical IDs store the index as a little-endian
u16 in bytes 6-7 of the 8-byte cortical ID.

# Usage in Cortical IDs
IO cortical IDs pack this index in bytes 6-7 (little-endian u16)."
);

impl From<u8> for CorticalUnitIndex {
    fn from(value: u8) -> Self {
        Self::from(value as u16)
    }
}

define_index!(
    CorticalSubUnitIndex,
    u8,
    "Index for cortical areas within a cortical unit. This allows easy identification of various
     cortical areas (which can be called CorticalSubUnits in this case) within a cortical unit"
);

define_index!(
    CorticalChannelIndex,
    u32,
    "Index for addressing specific channels within an I/O cortical area.

Cortical areas can contain multiple channels for processing different
aspects of data. This index addresses individual channels within a
specific cortical area for fine-grained data routing."
);

//endregion

//region Channels

define_nonzero_count!(
    CorticalChannelCount,
    u32,
    "The number of Cortical Channels cannot be zero."
);

define_nonzero_count!(NeuronDepth, u32, "The number of Neurons cannot be zero.");

define_xyz_dimensions!(
    CorticalChannelDimensions,
    u32,
    "CorticalChannelDimensions",
    0,
    "Dimensions of a channel within a cortical area.

Defines the 3D size of an individual channel, which represents
a subdivision of processing capability within a cortical area.
Channels allow for parallel processing of different data aspects.

# Usage
Used to define the spatial extent of individual channels for
data routing, processing allocation, and memory management."
);

//endregion

//region Spatial

define_xyz_coordinates!(
    NeuronVoxelCoordinate,
    u32,
    "NeuronVoxelCoordinate",
    "Coordinate local to a parent cortical area.

Represents a position within the bounds of a specific cortical area,
using unsigned integers since cortical coordinates are always positive
relative to the cortical area's origin.

# Usage
Used for addressing specific locations within individual cortical areas
for neuron placement, connectivity mapping, and spatial organization."
);

define_xyz_dimensions!(
    CorticalAreaDimensions,
    u32,
    "CorticalDimensions",
    0,
    "Dimensions of an entire cortical area.

Defines the complete 3D spatial extent of a cortical area,
including all channels and processing units within that area.
Represents the total neural space occupied by the cortical region.

# Usage
Used for cortical area placement within the genome, memory allocation,
and spatial relationship calculations between brain regions."
);

//endregion
