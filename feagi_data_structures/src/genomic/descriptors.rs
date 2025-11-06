use crate::FeagiDataError;
use crate::{define_xyz_dimensions, define_xyz_dimension_range, define_xyz_coordinates, define_index, define_nonzero_count};


//region Cortical Groupings (Cortical ID index)

define_index!(CorticalGroupIndex, u8, 
    "Index for grouping cortical areas of the same type within a genome.

This index distinguishes between multiple instances of the same cortical type.
For example, multiple vision sensors would have different CorticalGroupingIndex
values (0, 1, 2, etc.) while sharing the same base cortical type.

# Range
Values are limited to 0-255 (u8) and are encoded in hexadecimal within cortical IDs.
This provides support for up to 256 instances of each cortical type.

# Usage in Cortical IDs
The index appears as the last two characters of a cortical ID:
- \"ivis00\" = Vision sensor, grouping index 0
- \"ivis01\" = Vision sensor, grouping index 1
- \"omot0A\" = Motor output, grouping index 10 (hexadecimal A)"
);

define_index!(CorticalUnitIndex, u8,
"Index for cortical areas within a cortical unit");


//endregion

//region Cortical Channels

define_nonzero_count!(CorticalChannelCount, u32, "The number of Cortical Channels cannot be zero."
);

define_nonzero_count!(NeuronDepth, u32, "The number of Neurons cannot be zero." );

define_index!(CorticalChannelIndex, u32,
    "Index for addressing specific channels within an I/O cortical area.

Cortical areas can contain multiple channels for processing different
aspects of data. This index addresses individual channels within a
specific cortical area for fine-grained data routing."
);

define_xyz_dimensions!(CorticalChannelDimensions, u32, "CorticalChannelDimensions", 0,
    "Dimensions of a channel within a cortical area.

Defines the 3D size of an individual channel, which represents
a subdivision of processing capability within a cortical area.
Channels allow for parallel processing of different data aspects.

# Usage
Used to define the spatial extent of individual channels for
data routing, processing allocation, and memory management."
);

//endregion

//region Cortical Areas

define_xyz_coordinates!(GenomeCoordinate, i32, "GenomeCoordinate",
    "Coordinate local to the FEAGI Genome space.

Represents a position within the global genome coordinate system,
using signed integers to allow for negative coordinates and relative
positioning across the entire genome space.

# Usage
Used for absolute positioning of cortical areas within the genome
and for calculating spatial relationships between different brain regions."
);

define_xyz_coordinates!(CorticalCoordinate, u32, "CorticalCoordinate",
    "Coordinate local to a parent cortical area.

Represents a position within the bounds of a specific cortical area,
using unsigned integers since cortical coordinates are always positive
relative to the cortical area's origin.

# Usage
Used for addressing specific locations within individual cortical areas
for neuron placement, connectivity mapping, and spatial organization."
);

define_xyz_dimensions!(CorticalDimensions, u32, "CorticalDimensions", 0,
    "Dimensions of an entire cortical area.

Defines the complete 3D spatial extent of a cortical area,
including all channels and processing units within that area.
Represents the total neural space occupied by the cortical region.

# Usage
Used for cortical area placement within the genome, memory allocation,
and spatial relationship calculations between brain regions."
);

define_xyz_dimension_range!(CorticalChannelDimensionRange, u32, CorticalDimensions, "CorticalChannelDimensionRange",
    "Range of possible dimensions for channels within a cortical area.

Defines the acceptable bounds for channel sizes, allowing for
flexible channel configuration while maintaining system constraints.
Each axis can have its own valid range.

# Usage
Used during cortical area configuration to validate channel
dimensions and ensure they fit within system limitations."
);
//endregion

//region Agent Device

define_index!(AgentDeviceIndex, u32,
"An index for a specific channel on a specific cortical group (or multiple). An alternate way to refer to channels"
);

//endregion

