use crate::cortical_units::definitions::cortical_area_common::{CorticalAreaDataTypeFlag, IOCorticalAreaDefinition, FrameChangeHandling, PercentageNeuronPositioning};
use crate::cortical_units::definitions::cortical_unit_common::{CorticalUnitDataType, CorticalUnitDefinition, IOCorticalID4BytePrefix};

pub(crate) const fn create_motor_bytes(bytes: [u8; 3]) -> IOCorticalID4BytePrefix {
    [b'0', bytes[0], bytes[1], bytes[2]]
}

pub const NUMBER_MOTOR_CORTICAL_UNITS: usize = 9;

pub const MotorCorticalUnits: [CorticalUnitDefinition; NUMBER_MOTOR_CORTICAL_UNITS] =
[
    CorticalUnitDefinition {
        name: "RotaryMotor",
        friendly_english_name: "Rotary Motor",
        cortical_unit_prefix_bytes: create_motor_bytes(*b"mot"),
        cortical_unit_data_type_flag: CorticalUnitDataType::SignedPercentage,
        number_cortical_areas: 1,
        cortical_area_default_properties: [
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Rotary Motor",
                cortical_sub_unit_index: 0,
                cortical_area_data_type: CorticalAreaDataTypeFlag::SignedPercentage(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Fractional
                ),
                relative_position: (-20, 0, -10),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (1, 1, 10),
                channel_dimensions_max: (1, 1, 1024),
            }),
            None, None, None, None, None, None, None, None,
        ],
    },


    CorticalUnitDefinition {
        name: "PositionalServo",
        friendly_english_name: "Positional Servo",
        cortical_unit_prefix_bytes: create_motor_bytes(*b"pse"),
        cortical_unit_data_type_flag: CorticalUnitDataType::Percentage,
        number_cortical_areas: 2,
        cortical_area_default_properties: [
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Positional Servo Absolute",
                cortical_sub_unit_index: 0,
                cortical_area_data_type: CorticalAreaDataTypeFlag::Percentage(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Fractional
                ),
                relative_position: (-20, 0, -10),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (1, 1, 10),
                channel_dimensions_max: (1, 1, 1024),
            }),
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Positional Servo Incremental",
                cortical_sub_unit_index: 1,
                cortical_area_data_type: CorticalAreaDataTypeFlag::Percentage(
                    FrameChangeHandling::Incremental,
                    PercentageNeuronPositioning::Fractional
                ),
                relative_position: (-40, 0, -10),
                channel_dimensions_min: (2, 1, 1),
                channel_dimensions_default: (2, 1, 10),
                channel_dimensions_max: (2, 1, 1024),
            }),
            None, None, None, None, None, None, None,
        ],
    },


    CorticalUnitDefinition {
        name: "Gaze",
        friendly_english_name: "Gaze Control",
        cortical_unit_prefix_bytes: create_motor_bytes(*b"gaz"),
        cortical_unit_data_type_flag: CorticalUnitDataType::GazeProperties,
        number_cortical_areas: 2,
        cortical_area_default_properties: [
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Gaze Eccentricity",
                cortical_sub_unit_index: 0,
                cortical_area_data_type: CorticalAreaDataTypeFlag::Percentage2D(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Fractional
                ),
                relative_position: (10, 0, -10),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (8, 8, 1),
                channel_dimensions_max: (1024, 1024, 1),
            }),
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Gaze Modularity",
                cortical_sub_unit_index: 1,
                cortical_area_data_type: CorticalAreaDataTypeFlag::Percentage(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Fractional
                ),
                relative_position: (0, 0, -10),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (1, 1, 10),
                channel_dimensions_max: (1, 1, 1024),
            }),
            None, None, None, None, None, None, None,
        ],
    },


    CorticalUnitDefinition {
        name: "MiscData",
        friendly_english_name: "Miscellaneous Motor",
        cortical_unit_prefix_bytes: create_motor_bytes(*b"mis"),
        cortical_unit_data_type_flag: CorticalUnitDataType::MiscData,
        number_cortical_areas: 1,
        cortical_area_default_properties: [
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Miscellaneous Motor",
                cortical_sub_unit_index: 0,
                cortical_area_data_type: CorticalAreaDataTypeFlag::MiscData(
                    FrameChangeHandling::Absolute
                ),
                relative_position: (300, 0, -30),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (8, 8, 1),
                channel_dimensions_max: (1024, 1024, 1024),
            }),
            None, None, None, None, None, None, None, None,
        ],
    },


    CorticalUnitDefinition {
        name: "TextEnglishOutput",
        friendly_english_name: "Text Output (English)",
        cortical_unit_prefix_bytes: create_motor_bytes(*b"ten"),
        cortical_unit_data_type_flag: CorticalUnitDataType::MiscData,
        number_cortical_areas: 1,
        cortical_area_default_properties: [
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Text Output (English)",
                cortical_sub_unit_index: 0,
                cortical_area_data_type: CorticalAreaDataTypeFlag::MiscData(
                    FrameChangeHandling::Absolute
                ),
                relative_position: (85, 0, -30),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (1, 1, 16),
                channel_dimensions_max: (1, 1, 32),
            }),
            None, None, None, None, None, None, None, None,
        ],
    },


    CorticalUnitDefinition {
        name: "CountOutput",
        friendly_english_name: "Count Output",
        cortical_unit_prefix_bytes: create_motor_bytes(*b"cnt"),
        cortical_unit_data_type_flag: CorticalUnitDataType::Percentage,
        number_cortical_areas: 1,
        cortical_area_default_properties: [
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Count Output",
                cortical_sub_unit_index: 0,
                cortical_area_data_type: CorticalAreaDataTypeFlag::Percentage(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Fractional
                ),
                relative_position: (175, 0, -30),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (1, 1, 10),
                channel_dimensions_max: (1, 1, 1024),
            }),
            None, None, None, None, None, None, None, None,
        ],
    },


    CorticalUnitDefinition {
        name: "ObjectSegmentation",
        friendly_english_name: "Object Segmentation",
        cortical_unit_prefix_bytes: create_motor_bytes(*b"seg"),
        cortical_unit_data_type_flag: CorticalUnitDataType::MiscData,
        number_cortical_areas: 1,
        cortical_area_default_properties: [
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Object Segmentation",
                cortical_sub_unit_index: 0,
                cortical_area_data_type: CorticalAreaDataTypeFlag::MiscData(
                    FrameChangeHandling::Absolute
                ),
                relative_position: (-200, 0, 0),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (32, 32, 8),
                channel_dimensions_max: (4096, 4096, 1024),
            }),
            None, None, None, None, None, None, None, None,
        ],
    },


    CorticalUnitDefinition {
        name: "SimpleVisionOutput",
        friendly_english_name: "Simple Vision",
        cortical_unit_prefix_bytes: create_motor_bytes(*b"img"),
        cortical_unit_data_type_flag: CorticalUnitDataType::ImageFrame,
        number_cortical_areas: 1,
        cortical_area_default_properties: [
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Simple Vision",
                cortical_sub_unit_index: 0,
                cortical_area_data_type: CorticalAreaDataTypeFlag::CartesianPlane(
                    FrameChangeHandling::Absolute
                ),
                relative_position: (-240, 60, -20),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (128, 128, 3),
                channel_dimensions_max: (4096, 4096, 3),
            }),
            None, None, None, None, None, None, None, None,
        ],
    },


    CorticalUnitDefinition {
        name: "DynamicImageProcessing",
        friendly_english_name: "Image Enhancements",
        cortical_unit_prefix_bytes: create_motor_bytes(*b"ifs"),
        cortical_unit_data_type_flag: CorticalUnitDataType::ImageFilteringSettings,
        number_cortical_areas: 4,
        cortical_area_default_properties: [
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Image Enhancements Brightness",
                cortical_sub_unit_index: 0,
                cortical_area_data_type: CorticalAreaDataTypeFlag::Percentage(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Fractional
                ),
                relative_position: (0, 0, 0),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (1, 1, 10),
                channel_dimensions_max: (1, 1, 1024),
            }),
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Image Enhancements Contrast",
                cortical_sub_unit_index: 1,
                cortical_area_data_type: CorticalAreaDataTypeFlag::Percentage(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Fractional
                ),
                relative_position: (0, 0, -10),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (1, 1, 10),
                channel_dimensions_max: (1, 1, 1024),
            }),
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Image Enhancements Per Pixel Diff",
                cortical_sub_unit_index: 2,
                cortical_area_data_type: CorticalAreaDataTypeFlag::Percentage2D(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Fractional
                ),
                relative_position: (0, 0, -30),
                channel_dimensions_min: (2, 1, 1),
                channel_dimensions_default: (2, 1, 10),
                channel_dimensions_max: (2, 1, 1024),
            }),
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Image Enhancements Image Diff",
                cortical_sub_unit_index: 3,
                cortical_area_data_type: CorticalAreaDataTypeFlag::Percentage2D(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Fractional
                ),
                relative_position: (0, 0, -30),
                channel_dimensions_min: (2, 1, 1),
                channel_dimensions_default: (2, 1, 10),
                channel_dimensions_max: (2, 1, 1024),
            }),
            None, None, None, None, None,
        ],
    },
];