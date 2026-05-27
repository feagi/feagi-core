use crate::cortical_units::definitions::cortical_area_common::IOCorticalAreaDefinition;


/// Defines a cortical unit in a way that can be stored in a const array, such that we can use
/// it for runtime validation or for compile time macro generation
pub struct CorticalUnitDefinition {
    pub name: &'static str,
    pub friendly_english_name: &'static str,
    pub cortical_unit_prefix_bytes: IOCorticalID4BytePrefix,
    pub cortical_unit_data_type_flag: CorticalUnitDataType,
    pub number_cortical_areas: u8,
    pub cortical_area_default_properties: IOCorticalAreaDefinitions
}

/// All input / output cortical areas start with 4 bytes, with the first being b'i' or b'0' for
/// input or output, and the next 3 being something unique denoting the cortical unit it is
/// within
pub(crate) type IOCorticalID4BytePrefix = [u8; 4];


// Yes this is kinda dumb but I dont care anymore
/// Allows storing the multiple cortical area definitions for a cortical unit in a const friendly
/// way.
pub(crate) type IOCorticalAreaDefinitions = [Option<IOCorticalAreaDefinition>; 9]; // 9 is the most cortical areas in a unit right now, otherwise this number is arbitrary

/// The type of data the cortical unit as a whole store. Cortical Areas can encode only a few
/// specific points of information, but by combining them together, we can encode multiple
/// more complex structures as listed here
pub enum CorticalUnitDataType {
    Percentage,
    Percentage2D,
    Percentage3D,
    Percentage4D,
    SignedPercentage,
    SignedPercentage2D,
    SignedPercentage3D,
    SignedPercentage4D,
    Boolean,
    MiscData,
    ImageFrame,
    SegmentedImageFrame,
    RawIMU,
    GazeProperties,
    ImageFilteringSettings,
}



