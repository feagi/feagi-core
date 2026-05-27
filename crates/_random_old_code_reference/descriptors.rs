// NOTE: Only expose a specific quantization for most of these types!

pub use generated::CorticalUnitIndex;
pub use generated::CorticalSubUnitIndex;
pub use generated::CorticalChannelIndex;
pub use generated::CorticalChannelCount;
pub use generated::CorticalChannelNeuronDepth;
pub use generated::CorticalChannelCoordinate;
pub use generated::CorticalChannelDimensions;











// NOTE: Since these macros generate generic public types, generate them in this module, and expose  only the quantization we want above
mod generated {

    

    //region Channels

    define_quantizable_uint_type_family!(CorticalChannelCountType);
    add_non_zero_constructors_to_quant_uint!(CorticalChannelCountType);
    /// The number of cortical channels
    pub type CorticalChannelCount = CorticalChannelCountType<u32>;


    define_quantizable_uint_type_family!(CorticalChannelNeuronDepthType);
    add_non_zero_constructors_to_quant_uint!(CorticalChannelNeuronDepthType);
    /// The number of neuron_collections deep of a sensor / motor channel. Generally used to define resolution
    pub type CorticalChannelNeuronDepth = CorticalChannelNeuronDepthType<u32>;

    define_unsigned_coordinate_3d_type_family!(CorticalChannelCoordinateType);
    /// The coordinate of a neuron voxel in regards to its specific channel within a sensor / motor area
    pub type CorticalChannelCoordinate = CorticalChannelCoordinateType<u32>;

    define_dimension_3d_type_family!(CorticalChannelDimensionsType, CorticalChannelCoordinateType);
    /// The dimensions of an individual cortical channel
    pub type CorticalChannelDimensions = CorticalChannelDimensionsType<u32>;

    //endregion
}
