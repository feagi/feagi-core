use crate::FeagiDataError;
use crate::basic_components::{Dimensions, DimensionRange, CoordinateU32, CoordinateI32};

//region Macros
/// Defines the index of something as an integer of a certain type
macro_rules! define_index {
    ($name:ident, $inner:ty, $doc:expr) => {
        #[doc = $doc]
        #[repr(transparent)]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord
        )]
        pub struct $name($inner);

        impl std::ops::Deref for $name {
            type Target = $inner;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl From<$inner> for $name {
            fn from(value: $inner) -> Self {
                $name(value)
            }
        }

        impl From<$name> for $inner {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

/// Define the count of something that cannot be 0
macro_rules! define_nonzero_count {
    ($name:ident, $base:ty, $doc:expr) => {
        #[doc = $doc]
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name($base);
        
        impl $name {
            /// Creates a new instance, returns Err if validation fails
            pub fn new(value: $base) -> Result<Self, FeagiDataError> {
                if value == 0 {
                    return Err(FeagiDataError::BadParameters("Count cannot be zero!".into()));
                }
                Ok($name(value))
            }
        }
        impl From<$base> for $name {
            fn from(value: $base) -> Self {
                Self(value)
            }
        }

        impl From<$name> for $base {
            fn from(value: $name) -> $base {
                value.0
            }
        }

        impl std::ops::Deref for $name {
            type Target = $base;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
        
    }
}

/// Define a coordinate wrapper type with specific semantic meaning
macro_rules! define_coordinate_u32 {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[repr(transparent)]
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(CoordinateU32);
        
        impl $name {
            /// Creates a new coordinate instance.
            pub fn new(x: u32, y: u32, z: u32) -> Self {
                $name(CoordinateU32::new(x, y, z))
            }
        }
        
        impl std::ops::Deref for $name {
            type Target = CoordinateU32;
            
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        
        impl From<CoordinateU32> for $name {
            fn from(coord: CoordinateU32) -> Self {
                $name(coord)
            }
        }
        
        impl From<$name> for CoordinateU32 {
            fn from(coord: $name) -> Self {
                coord.0
            }
        }
        
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

/// Define a coordinate wrapper type with specific semantic meaning (i32 version)
macro_rules! define_coordinate_i32 {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[repr(transparent)]
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(CoordinateI32);
        
        impl $name {
            /// Creates a new coordinate instance.
            pub fn new(x: i32, y: i32, z: i32) -> Self {
                $name(CoordinateI32::new(x, y, z))
            }
        }
        
        impl std::ops::Deref for $name {
            type Target = CoordinateI32;
            
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        
        impl From<CoordinateI32> for $name {
            fn from(coord: CoordinateI32) -> Self {
                $name(coord)
            }
        }
        
        impl From<$name> for CoordinateI32 {
            fn from(coord: $name) -> Self {
                coord.0
            }
        }
        
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

/// Define a dimensions wrapper type with specific semantic meaning
macro_rules! define_dimensions {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[repr(transparent)]
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(Dimensions);
        
        impl $name {
            /// Creates new dimensions, ensuring all values are non-zero.
            pub fn new(x: u32, y: u32, z: u32) -> Result<Self, FeagiDataError> {
                Ok($name(Dimensions::new(x, y, z)?))
            }
            
            /// Verifies that a coordinate falls within these dimensional bounds.
            pub fn verify_coordinate_in_bounds(&self, coordinate: &CoordinateU32) -> Result<(), FeagiDataError> {
                self.0.verify_coordinate_in_bounds(coordinate)
            }
            
            /// Gets the x dimension.
            pub fn x(&self) -> u32 {
                self.0.x
            }
            
            /// Gets the y dimension.
            pub fn y(&self) -> u32 {
                self.0.y
            }
            
            /// Gets the z dimension.
            pub fn z(&self) -> u32 {
                self.0.z
            }
        }
        
        impl std::ops::Deref for $name {
            type Target = Dimensions;
            
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        
        impl From<Dimensions> for $name {
            fn from(dims: Dimensions) -> Self {
                $name(dims)
            }
        }
        
        impl From<$name> for Dimensions {
            fn from(dims: $name) -> Self {
                dims.0
            }
        }
        
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}

/// Define a dimension range wrapper type with specific semantic meaning
macro_rules! define_dimension_range {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[repr(transparent)]
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(DimensionRange);
        
        impl $name {
            /// Creates a new dimension range, ensuring no ranges are empty.
            pub fn new(x: std::ops::Range<u32>, y: std::ops::Range<u32>, z: std::ops::Range<u32>) -> Result<Self, FeagiDataError> {
                Ok($name(DimensionRange::new(x, y, z)?))
            }
            
            /// Returns true if any axis spans more than one value.
            pub fn is_ambiguous(&self) -> bool {
                self.0.is_ambiguous()
            }
            
            /// Verifies that a coordinate falls within all axis ranges.
            pub fn verify_coordinate_u32_within_range(&self, coordinate: &CoordinateU32) -> Result<(), FeagiDataError> {
                self.0.verify_coordinate_u32_within_range(coordinate)
            }
        }
        
        impl std::ops::Deref for $name {
            type Target = DimensionRange;
            
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        
        impl From<DimensionRange> for $name {
            fn from(range: DimensionRange) -> Self {
                $name(range)
            }
        }
        
        impl From<$name> for DimensionRange {
            fn from(range: $name) -> Self {
                range.0
            }
        }
        
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
    };
}
//endregion

//region Cortical Group Index

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

// NOTE: We do not need a Cortical Group Count!

//endregion

//region Cortical Channel Index

define_index!(CorticalChannelIndex, u32,
    "Index for addressing specific channels within an I/O cortical area.

Cortical areas can contain multiple channels for processing different
aspects of data. This index addresses individual channels within a
specific cortical area for fine-grained data routing."
);

define_index!(AgentDeviceIndex, u32,
"An index for a specific channel on a specific cortical group (or multiple). An alternate way to refer to channels"
);

define_nonzero_count!(CorticalChannelCount, u32, "The number of Cortical Channels. Cannot be zero."
);

//endregion

//region Cortical Coordinates

define_coordinate_u32!(CorticalCoordinate,
    "Coordinate local to a parent cortical area.

Represents a position within the bounds of a specific cortical area,
using unsigned integers since cortical coordinates are always positive
relative to the cortical area's origin.

# Usage
Used for addressing specific locations within individual cortical areas
for neuron placement, connectivity mapping, and spatial organization."
);

//endregion

//region Genome Coordinates

define_coordinate_i32!(GenomeCoordinate,
    "Coordinate local to the FEAGI Genome space.

Represents a position within the global genome coordinate system,
using signed integers to allow for negative coordinates and relative
positioning across the entire genome space.

# Usage
Used for absolute positioning of cortical areas within the genome
and for calculating spatial relationships between different brain regions."
);

//endregion

//region Cortical Channel Dimensions

define_dimensions!(CorticalChannelDimensions,
    "Dimensions of a channel within a cortical area.

Defines the 3D size of an individual channel, which represents
a subdivision of processing capability within a cortical area.
Channels allow for parallel processing of different data aspects.

# Usage
Used to define the spatial extent of individual channels for
data routing, processing allocation, and memory management."
);

//endregion

//region Cortical Channel Dimension Range

define_dimension_range!(CorticalChannelDimensionRange,
    "Range of possible dimensions for channels within a cortical area.

Defines the acceptable bounds for channel sizes, allowing for
flexible channel configuration while maintaining system constraints.
Each axis can have its own valid range.

# Usage
Used during cortical area configuration to validate channel
dimensions and ensure they fit within system limitations."
);

//endregion

//region Cortical Dimensions

define_dimensions!(CorticalDimensions,
    "Dimensions of an entire cortical area.

Defines the complete 3D spatial extent of a cortical area,
including all channels and processing units within that area.
Represents the total neural space occupied by the cortical region.

# Usage
Used for cortical area placement within the genome, memory allocation,
and spatial relationship calculations between brain regions."
);

//endregion

