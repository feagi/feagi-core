// NOTE: Only expose a specific quantization for most of these types!

use crate::{create_quantized_index_count_wrapper, create_quantized_spatial_index_coordinate_3d_wrapper, create_quantized_spatial_index_dimensions_3d_wrapper};

create_quantized_index_count_wrapper!(CorticalAreaCount);

// Used to denote the cortical area index within the NPU directly. May have various quantizations
create_quantized_index_count_wrapper!(CorticalAreaIndex);


//region Cortical Indexing

/// Index for grouping cortical units of the same type within a genome.
///
/// This index distinguishes between multiple instances of the same cortical type.
/// For example, multiple vision sensors would have different CorticalUnitIndex
/// values (0, 1, 2, etc.) while sharing the same base cortical type.
///
/// # Range
/// Values are limited to 0-255 (u8) and are encoded in hexadecimal within cortical IDs.
/// This provides support for up to 256 instances of each cortical unit type.
///
/// # Usage in Cortical IDs
/// The index appears as the last two characters of a cortical ID:
/// - \"ivis00\" = Vision sensor, grouping index 0
/// - \"ivis01\" = Vision sensor, grouping index 1
/// - \"omot0A\" = Motor output, grouping index 10 (hexadecimal A)
create_quantized_index_count_wrapper!(CorticalUnitIndex, u8);


/// Index for cortical areas within a cortical unit. This allows easy identification of various
/// cortical areas (which can be called CorticalSubUnits in this case) within a cortical unit
create_quantized_index_count_wrapper!(CorticalSubUnitIndex, u8);


/// Index for addressing specific channels within an I/O cortical area.
///
/// Cortical areas can contain multiple channels for processing different
/// aspects of data. This index addresses individual channels within a
/// specific cortical area for fine-grained data routing.
create_quantized_index_count_wrapper!(CorticalChannelIndex, u32);

//endregion



//region Channels

/// The number of cortical channels
create_quantized_index_count_wrapper!(CorticalChannelCount, u32);


/// The number of neuron_collections deep of a sensor / motor channel. Generally used to define resolution
create_quantized_index_count_wrapper!(CorticalChannelNeuronDepth, u32);



create_quantized_index_count_wrapper!(CorticalChannelCoordinateAxis, u32);
/// The coordinate of a neuron voxel in regards to its specific channel within a sensor / motor area
create_quantized_spatial_index_coordinate_3d_wrapper!(CorticalChannelCoordinate, u32, CorticalChannelCoordinateAxis, CorticalChannelCoordinateAxis, CorticalChannelCoordinateAxis);


create_quantized_index_count_wrapper!(CorticalChannelCoordinateLinearIndex, u32);
/// The dimensions of an individual cortical channel
create_quantized_spatial_index_dimensions_3d_wrapper!(CorticalChannelDimensions, u32, CorticalChannelCoordinate, CorticalChannelCoordinateLinearIndex, CorticalChannelCoordinateAxis, CorticalChannelCoordinateAxis, CorticalChannelCoordinateAxis);