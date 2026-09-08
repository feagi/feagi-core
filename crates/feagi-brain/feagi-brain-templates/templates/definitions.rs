
/// What type of data the area parses
pub enum ParsedDataType {
    Boolean,
    Percentage,
    Percentage_2D,
    Percentage_3D,
    Percentage_4D,
    SignedPercentage,
    SignedPercentage_2D,
    SignedPercentage_3D,
    SignedPercentage_4D,
    ImageFrame,
    SegmentedImageFrame,
    RawIMU,
    MiscData,
    GazeProperties,
    ImageFilteringSettings,
    PoseEstimationData
} //TODO get cortical_type_parameters

pub enum CorticalGeneratorDefinition {
    Dimensional {
        relative_position: [i32; 3],
        channel_dimensions_default: [u32; 3],
        channel_dimensions_min: [u32; 3],
        channel_dimensions_min: [u32; 3],
    }
    // TODO Vision based cortical area
    // TODO should vision even have multi channel?
}

pub struct CorticalUnitDefinition {
    pub doc_string: &'static str,
    pub snake_case: &'static str,
    pub pascal_case: &'static str,
    pub friendly_name: &'static str, // TODO migrate away for localization!
    pub parsed_data_type: ParsedDataType,
    pub cortical_area_generators: &'static [bool],
    pub temp_is_sensor: bool // TODO remove this! // TODO have searchable / lookup tags instead as some devices are a mix of sensor / motor
}

impl CorticalUnitDefinition {

    pub const fn number_cortical_areas(&self) -> usize {
        self.cortical_area_generators.len()
    }
}
