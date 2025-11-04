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

//region Cortical Variant Enums

pub type DataTypeConfigurationFlag = u8;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PercentageNeuronPositioning {
    Linear,
    Fractional,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FrameChangeHandling {
    Absolute,
    Incremental,
}

#[derive(Debug, PartialEq, Eq)]
pub enum IOCorticalAreaDataType {
    Percentage(FrameChangeHandling, PercentageNeuronPositioning),
    Percentage2D(FrameChangeHandling, PercentageNeuronPositioning),
    Percentage3D(FrameChangeHandling, PercentageNeuronPositioning),
    Percentage4D(FrameChangeHandling, PercentageNeuronPositioning),
    SignedPercentage(FrameChangeHandling, PercentageNeuronPositioning),
    SignedPercentage2D(FrameChangeHandling, PercentageNeuronPositioning),
    SignedPercentage3D(FrameChangeHandling, PercentageNeuronPositioning),
    SignedPercentage4D(FrameChangeHandling, PercentageNeuronPositioning),
    CartesianPlane(FrameChangeHandling),
    Misc(FrameChangeHandling)
}

impl IOCorticalAreaDataType {
    
    /// Converts the enum to a u8 representation.
    /// 
    /// Bit layout:
    /// - Bits 0-3: Variant type (0-9)
    /// - Bit 4: FrameChangeHandling (0=Absolute, 1=Incremental)
    /// - Bit 5: PercentageNeuronPositioning (0=Linear, 1=Fractional, only valid for variants 0-7)
    /// - Bits 6-7: Reserved
    /// 
    /// This function is `const`, allowing it to be used in compile-time constant expressions.
    pub const fn to_data_type_configuration_flag(&self) -> DataTypeConfigurationFlag {
        let (variant, frame_handling, positioning) = match self {
            IOCorticalAreaDataType::Percentage(f, p) => (0u8, *f, Some(*p)),
            IOCorticalAreaDataType::Percentage2D(f, p) => (1u8, *f, Some(*p)),
            IOCorticalAreaDataType::Percentage3D(f, p) => (2u8, *f, Some(*p)),
            IOCorticalAreaDataType::Percentage4D(f, p) => (3u8, *f, Some(*p)),
            IOCorticalAreaDataType::SignedPercentage(f, p) => (4u8, *f, Some(*p)),
            IOCorticalAreaDataType::SignedPercentage2D(f, p) => (5u8, *f, Some(*p)),
            IOCorticalAreaDataType::SignedPercentage3D(f, p) => (6u8, *f, Some(*p)),
            IOCorticalAreaDataType::SignedPercentage4D(f, p) => (7u8, *f, Some(*p)),
            IOCorticalAreaDataType::CartesianPlane(f) => (8u8, *f, None),
            IOCorticalAreaDataType::Misc(f) => (9u8, *f, None),
        };

        let frame_bits = match frame_handling {
            FrameChangeHandling::Absolute => 0u8,
            FrameChangeHandling::Incremental => 1u8,
        };

        let positioning_bits = match positioning {
            Some(PercentageNeuronPositioning::Linear) => 0u8,
            Some(PercentageNeuronPositioning::Fractional) => 1u8,
            None => 0u8, // Not applicable for CartesianPlane/Misc
        };

        // Pack: variant (4 bits) | frame_handling (1 bit) << 4 | positioning (1 bit) << 5
        variant | (frame_bits << 4) | (positioning_bits << 5)
    }

    /// Attempts to convert a u8 to the enum.
    /// 
    /// Returns an error if the variant type is invalid (>9) or if the positioning
    /// bit is set for variants that don't support it (CartesianPlane, Misc).
    pub fn try_from_data_type_configuration_flag(value: DataTypeConfigurationFlag) -> Result<Self, FeagiDataError> {
        let variant = value & 0x0F; // Bits 0-3
        let frame_handling = (value >> 4) & 0x01; // Bit 4
        let positioning = (value >> 5) & 0x01; // Bit 5

        let frame_handling_enum = match frame_handling {
            0 => FrameChangeHandling::Absolute,
            1 => FrameChangeHandling::Incremental,
            _ => return Err(FeagiDataError::BadParameters("Invalid frame handling value".to_string())),
        };

        let positioning_enum = match positioning {
            0 => PercentageNeuronPositioning::Linear,
            1 => PercentageNeuronPositioning::Fractional,
            _ => return Err(FeagiDataError::BadParameters("Invalid positioning value".to_string())),
        };

        match variant {
            0 => Ok(IOCorticalAreaDataType::Percentage(frame_handling_enum, positioning_enum)),
            1 => Ok(IOCorticalAreaDataType::Percentage2D(frame_handling_enum, positioning_enum)),
            2 => Ok(IOCorticalAreaDataType::Percentage3D(frame_handling_enum, positioning_enum)),
            3 => Ok(IOCorticalAreaDataType::Percentage4D(frame_handling_enum, positioning_enum)),
            4 => Ok(IOCorticalAreaDataType::SignedPercentage(frame_handling_enum, positioning_enum)),
            5 => Ok(IOCorticalAreaDataType::SignedPercentage2D(frame_handling_enum, positioning_enum)),
            6 => Ok(IOCorticalAreaDataType::SignedPercentage3D(frame_handling_enum, positioning_enum)),
            7 => Ok(IOCorticalAreaDataType::SignedPercentage4D(frame_handling_enum, positioning_enum)),
            8 => {
                // CartesianPlane doesn't use positioning, but we'll accept it if set to 0
                if positioning != 0 {
                    return Err(FeagiDataError::BadParameters("CartesianPlane variant does not support positioning parameter".to_string()));
                }
                Ok(IOCorticalAreaDataType::CartesianPlane(frame_handling_enum))
            }
            9 => {
                // Misc doesn't use positioning, but we'll accept it if set to 0
                if positioning != 0 {
                    return Err(FeagiDataError::BadParameters("Misc variant does not support positioning parameter".to_string()));
                }
                Ok(IOCorticalAreaDataType::Misc(frame_handling_enum))
            }
            _ => Err(FeagiDataError::BadParameters(format!("Invalid variant type: {}", variant))),
        }
    }
}

impl From<&IOCorticalAreaDataType> for DataTypeConfigurationFlag {
    fn from(data_type: &IOCorticalAreaDataType) -> Self {
        data_type.to_data_type_configuration_flag()
    }
}

impl From<IOCorticalAreaDataType> for DataTypeConfigurationFlag {
    fn from(data_type: IOCorticalAreaDataType) -> Self {
        (&data_type).into()
    }
}

impl TryFrom<DataTypeConfigurationFlag> for IOCorticalAreaDataType {
    type Error = FeagiDataError;

    fn try_from(value: DataTypeConfigurationFlag) -> Result<Self, Self::Error> {
        IOCorticalAreaDataType::try_from_data_type_configuration_flag(value)
    }
}

/// Macro to get an `IOCorticalAreaDataType` as its u8 representation at compile time.
///
/// This macro takes an `IOCorticalAreaDataType` expression and returns
/// its `DataTypeConfigurationFlag` (u8) representation. This can be used
/// in const contexts and for static byte definitions.
///
/// # Example
/// ```ignore
/// use feagi_data_structures::data_type_flag_const;
/// use feagi_data_structures::genomic::descriptors::*;
///
/// const FLAG: u8 = data_type_flag_const!(
///     IOCorticalAreaDataType::Percentage(
///         FrameChangeHandling::Incremental,
///         PercentageNeuronPositioning::Fractional
///     )
/// );
/// // FLAG is now 35, computed at compile time
/// ```
#[macro_export]
macro_rules! data_type_flag_const {
    ($data_type:expr) => {
        {
            $data_type.to_data_type_configuration_flag()
        }
    };
}



//endregion
