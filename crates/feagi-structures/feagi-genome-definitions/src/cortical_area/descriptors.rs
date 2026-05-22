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


    //region Cortical Indexing

    define_quantizable_uint_type_family!(CorticalUnitIndexType);
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
    pub type CorticalUnitIndex = CorticalUnitIndexType<u8>;
    
    impl CorticalUnitIndex {
        pub const fn const_from(u_8: u8) -> Self {
            Self { 0: u_8}
        }
    }


    define_quantizable_uint_type_family!(CorticalSubUnitIndexType);
    /// Index for cortical areas within a cortical unit. This allows easy identification of various
    /// cortical areas (which can be called CorticalSubUnits in this case) within a cortical unit
    pub type CorticalSubUnitIndex = CorticalSubUnitIndexType<u8>;

    impl CorticalSubUnitIndex {
        pub const fn const_from(u_8: u8) -> Self {
            Self { 0: u_8}
        }
    }


    define_quantizable_uint_type_family!(CorticalChannelIndexType);
    /// Index for addressing specific channels within an I/O cortical area.
    ///
    /// Cortical areas can contain multiple channels for processing different
    /// aspects of data. This index addresses individual channels within a
    /// specific cortical area for fine-grained data routing.
    pub type CorticalChannelIndex = CorticalChannelIndexType<u32>;

    //endregion

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
