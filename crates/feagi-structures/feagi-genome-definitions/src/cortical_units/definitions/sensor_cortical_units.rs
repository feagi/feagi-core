use crate::cortical_units::definitions::cortical_area_common::{CorticalAreaDataTypeFlag, IOCorticalAreaDefinition, FrameChangeHandling, PercentageNeuronPositioning};
use crate::cortical_units::definitions::cortical_unit_common::{CorticalUnitDataType, CorticalUnitDefinition, IOCorticalID4BytePrefix};

pub const NUMBER_SENSOR_CORTICAL_UNITS: usize = 14;

const fn create_sensor_bytes(bytes: [u8; 3]) -> IOCorticalID4BytePrefix {
    [b'i', bytes[0], bytes[1], bytes[2]]
}

pub const SensorCorticalUnits: [CorticalUnitDefinition; NUMBER_SENSOR_CORTICAL_UNITS] =
[
    CorticalUnitDefinition {
        name: "Infrared",
        friendly_english_name: "Infrared Sensor",
        cortical_unit_prefix_bytes: create_sensor_bytes(*b"inf"),
        cortical_unit_data_type_flag: CorticalUnitDataType::Percentage,
        number_cortical_areas: 1,
        cortical_area_default_properties: [
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Infrared Distance",
                cortical_sub_unit_index: 0,
                cortical_area_data_type: CorticalAreaDataTypeFlag::Percentage(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Fractional
                ),
                relative_position: (10, 0, -20),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (1, 1, 10),
                channel_dimensions_max: (1, 1, 1024),
            }),
            None, None, None, None, None, None, None, None,
        ],
    },


    CorticalUnitDefinition {
        name: "Proximity",
        friendly_english_name: "Proximity Sensor",
        cortical_unit_prefix_bytes: create_sensor_bytes(*b"pro"),
        cortical_unit_data_type_flag: CorticalUnitDataType::Percentage,
        number_cortical_areas: 1,
        cortical_area_default_properties: [
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Proximity Distance",
                cortical_sub_unit_index: 0,
                cortical_area_data_type: CorticalAreaDataTypeFlag::Percentage(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Fractional
                ),
                relative_position: (20, 0, -20),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (1, 1, 10),
                channel_dimensions_max: (1, 1, 1024),
            }),
            None, None, None, None, None, None, None, None,
        ],
    },


    CorticalUnitDefinition {
        name: "Shock",
        friendly_english_name: "Shock sensor",
        cortical_unit_prefix_bytes: create_sensor_bytes(*b"shk"),
        cortical_unit_data_type_flag: CorticalUnitDataType::Percentage,
        number_cortical_areas: 1,
        cortical_area_default_properties: [
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Shock",
                cortical_sub_unit_index: 0,
                cortical_area_data_type: CorticalAreaDataTypeFlag::Percentage(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Fractional
                ),
                relative_position: (30, 0, -20),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (1, 1, 10),
                channel_dimensions_max: (1, 1, 1024),
            }),
            None, None, None, None, None, None, None, None,
        ],
    },


    CorticalUnitDefinition {
        name: "Battery",
        friendly_english_name: "Battery Sensor",
        cortical_unit_prefix_bytes: create_sensor_bytes(*b"bat"),
        cortical_unit_data_type_flag: CorticalUnitDataType::Percentage,
        number_cortical_areas: 1,
        cortical_area_default_properties: [
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Battery",
                cortical_sub_unit_index: 0,
                cortical_area_data_type: CorticalAreaDataTypeFlag::Percentage(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Fractional
                ),
                relative_position: (40, 0, -20),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (1, 1, 10),
                channel_dimensions_max: (1, 1, 1024),
            }),
            None, None, None, None, None, None, None, None,
        ],
    },


    CorticalUnitDefinition {
        name: "Servo",
        friendly_english_name: "Servo Encoder",
        cortical_unit_prefix_bytes: create_sensor_bytes(*b"svm"),
        cortical_unit_data_type_flag: CorticalUnitDataType::Percentage,
        number_cortical_areas: 1,
        cortical_area_default_properties: [
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Servo Encoder",
                cortical_sub_unit_index: 0,
                cortical_area_data_type: CorticalAreaDataTypeFlag::Percentage(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Fractional
                ),
                relative_position: (25, 0, -10),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (1, 1, 10),
                channel_dimensions_max: (1, 1, 1024),
            }),
            None, None, None, None, None, None, None, None,
        ],
    },


    CorticalUnitDefinition {
        name: "AnalogGPIO",
        friendly_english_name: "Analog GPIO Sensor",
        cortical_unit_prefix_bytes: create_sensor_bytes(*b"agp"),
        cortical_unit_data_type_flag: CorticalUnitDataType::Percentage,
        number_cortical_areas: 1,
        cortical_area_default_properties: [
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Analog GPIO Sensor",
                cortical_sub_unit_index: 0,
                cortical_area_data_type: CorticalAreaDataTypeFlag::Percentage(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Fractional
                ),
                relative_position: (60, 0, -10),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (8, 8, 1),
                channel_dimensions_max: (1024, 1024, 1),
            }),
            None, None, None, None, None, None, None, None,
        ],
    },


    CorticalUnitDefinition {
        name: "DigitalGPIO",
        friendly_english_name: "Digital GPIO Sensor",
        cortical_unit_prefix_bytes: create_sensor_bytes(*b"dgp"),
        cortical_unit_data_type_flag: CorticalUnitDataType::Boolean,
        number_cortical_areas: 1,
        cortical_area_default_properties: [
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Digital GPIO Sensor",
                cortical_sub_unit_index: 0,
                cortical_area_data_type: CorticalAreaDataTypeFlag::Boolean(),
                relative_position: (70, 0, -10),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (1, 1, 1),
                channel_dimensions_max: (1, 1, 1),
            }),
            None, None, None, None, None, None, None, None,
        ],
    },


    CorticalUnitDefinition {
        name: "MiscData",
        friendly_english_name: "Miscellaneous Sensor",
        cortical_unit_prefix_bytes: create_sensor_bytes(*b"mis"),
        cortical_unit_data_type_flag: CorticalUnitDataType::MiscData,
        number_cortical_areas: 1,
        cortical_area_default_properties: [
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Miscellaneous Sensor",
                cortical_sub_unit_index: 0,
                cortical_area_data_type: CorticalAreaDataTypeFlag::MiscData(
                    FrameChangeHandling::Absolute
                ),
                relative_position: (220, 0, -30),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (8, 8, 1),
                channel_dimensions_max: (1024, 1024, 1),
            }),
            None, None, None, None, None, None, None, None,
        ],
    },


    CorticalUnitDefinition {
        name: "TextEnglishInput",
        friendly_english_name: "Text Input (English)",
        cortical_unit_prefix_bytes: create_sensor_bytes(*b"ten"),
        cortical_unit_data_type_flag: CorticalUnitDataType::MiscData,
        number_cortical_areas: 1,
        cortical_area_default_properties: [
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Text Input (English)",
                cortical_sub_unit_index: 0,
                cortical_area_data_type: CorticalAreaDataTypeFlag::MiscData(
                    FrameChangeHandling::Absolute
                ),
                relative_position: (70, 0, -30),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (1, 1, 16),
                channel_dimensions_max: (1, 1, 32),
            }),
            None, None, None, None, None, None, None, None,
        ],
    },


    CorticalUnitDefinition {
        name: "CountInput",
        friendly_english_name: "Count Input",
        cortical_unit_prefix_bytes: create_sensor_bytes(*b"cnt"),
        cortical_unit_data_type_flag: CorticalUnitDataType::Percentage,
        number_cortical_areas: 1,
        cortical_area_default_properties: [
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Count Input",
                cortical_sub_unit_index: 0,
                cortical_area_data_type: CorticalAreaDataTypeFlag::Percentage(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Fractional
                ),
                relative_position: (110, 0, -30),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (1, 1, 10),
                channel_dimensions_max: (1, 1, 1024),
            }),
            None, None, None, None, None, None, None, None,
        ],
    },


    CorticalUnitDefinition {
        name: "Vision",
        friendly_english_name: "Simple Vision",
        cortical_unit_prefix_bytes: create_sensor_bytes(*b"img"),
        cortical_unit_data_type_flag: CorticalUnitDataType::ImageFrame,
        number_cortical_areas: 1,
        cortical_area_default_properties: [
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Simple Vision",
                cortical_sub_unit_index: 0,
                cortical_area_data_type: CorticalAreaDataTypeFlag::CartesianPlane(
                    FrameChangeHandling::Absolute
                ),
                relative_position: (-100, 30, 0),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (64, 64, 3),
                channel_dimensions_max: (4096, 4096, 3),
            }),
            None, None, None, None, None, None, None, None,
        ],
    },


    CorticalUnitDefinition {
        name: "SegmentedVision",
        friendly_english_name: "Segmented Vision",
        cortical_unit_prefix_bytes: create_sensor_bytes(*b"svi"),
        cortical_unit_data_type_flag: CorticalUnitDataType::SegmentedImageFrame,
        number_cortical_areas: 9,
        cortical_area_default_properties: [
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Segmented Vision Lower Left",
                cortical_sub_unit_index: 0,
                cortical_area_data_type: CorticalAreaDataTypeFlag::CartesianPlane(
                    FrameChangeHandling::Absolute
                ),
                relative_position: (-70, -70, 0),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (32, 32, 1),
                channel_dimensions_max: (4096, 4096, 3),
            }),
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Segmented Vision Lower Middle",
                cortical_sub_unit_index: 1,
                cortical_area_data_type: CorticalAreaDataTypeFlag::CartesianPlane(
                    FrameChangeHandling::Absolute
                ),
                relative_position: (60, -70, 0),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (32, 32, 1),
                channel_dimensions_max: (4096, 4096, 3),
            }),
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Segmented Vision Lower Right",
                cortical_sub_unit_index: 2,
                cortical_area_data_type: CorticalAreaDataTypeFlag::CartesianPlane(
                    FrameChangeHandling::Absolute
                ),
                relative_position: (150, -70, 0),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (32, 32, 1),
                channel_dimensions_max: (4096, 4096, 3),
            }),
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Segmented Vision Middle Left",
                cortical_sub_unit_index: 3,
                cortical_area_data_type: CorticalAreaDataTypeFlag::CartesianPlane(
                    FrameChangeHandling::Absolute
                ),
                relative_position: (-70, 60, 0),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (32, 32, 1),
                channel_dimensions_max: (4096, 4096, 3),
            }),
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Segmented Vision Middle Middle",
                cortical_sub_unit_index: 4,
                cortical_area_data_type: CorticalAreaDataTypeFlag::CartesianPlane(
                    FrameChangeHandling::Absolute
                ),
                relative_position: (0, 0, 0),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (128, 128, 3),
                channel_dimensions_max: (4096, 4096, 3),
            }),
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Segmented Vision Middle Right",
                cortical_sub_unit_index: 5,
                cortical_area_data_type: CorticalAreaDataTypeFlag::CartesianPlane(
                    FrameChangeHandling::Absolute
                ),
                relative_position: (150, 60, 0),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (32, 32, 1),
                channel_dimensions_max: (4096, 4096, 3),
            }),
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Segmented Vision Upper Left",
                cortical_sub_unit_index: 6,
                cortical_area_data_type: CorticalAreaDataTypeFlag::CartesianPlane(
                    FrameChangeHandling::Absolute
                ),
                relative_position: (-70, 150, 0),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (32, 32, 1),
                channel_dimensions_max: (4096, 4096, 3),
            }),
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Segmented Vision Upper Middle",
                cortical_sub_unit_index: 7,
                cortical_area_data_type: CorticalAreaDataTypeFlag::CartesianPlane(
                    FrameChangeHandling::Absolute
                ),
                relative_position: (60, 150, 0),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (32, 32, 1),
                channel_dimensions_max: (4096, 4096, 3),
            }),
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Segmented Vision Upper Right",
                cortical_sub_unit_index: 8,
                cortical_area_data_type: CorticalAreaDataTypeFlag::CartesianPlane(
                    FrameChangeHandling::Absolute
                ),
                relative_position: (150, 150, 0),
                channel_dimensions_min: (1, 1, 1),
                channel_dimensions_default: (32, 32, 1),
                channel_dimensions_max: (4096, 4096, 3),
            }),
        ],
    },


    CorticalUnitDefinition {
        name: "RawIMU",
        friendly_english_name: "Raw IMU",
        cortical_unit_prefix_bytes: create_sensor_bytes(*b"rim"),
        cortical_unit_data_type_flag: CorticalUnitDataType::RawIMU,
        number_cortical_areas: 3,
        cortical_area_default_properties: [
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Raw IMU Accelerometer",
                cortical_sub_unit_index: 0,
                cortical_area_data_type: CorticalAreaDataTypeFlag::SignedPercentage3D(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Fractional
                ),
                relative_position: (70, 0, -10),
                channel_dimensions_min: (3, 1, 1),
                channel_dimensions_default: (3, 1, 10),
                channel_dimensions_max: (3, 1, 1024),
            }),
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Raw IMU Gyroscope",
                cortical_sub_unit_index: 1,
                cortical_area_data_type: CorticalAreaDataTypeFlag::SignedPercentage3D(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Fractional
                ),
                relative_position: (80, 0, -10),
                channel_dimensions_min: (3, 1, 1),
                channel_dimensions_default: (3, 1, 10),
                channel_dimensions_max: (3, 1, 1024),
            }),
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Raw IMU Magnetometer",
                cortical_sub_unit_index: 2,
                cortical_area_data_type: CorticalAreaDataTypeFlag::SignedPercentage3D(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Fractional
                ),
                relative_position: (90, 0, -10),
                channel_dimensions_min: (3, 1, 1),
                channel_dimensions_default: (3, 1, 10),
                channel_dimensions_max: (3, 1, 1024),
            }),
            None, None, None, None, None, None,
        ],
    },


    CorticalUnitDefinition {
        name: "SmartIMU",
        friendly_english_name: "Smart IMU",
        cortical_unit_prefix_bytes: create_sensor_bytes(*b"sim"),
        cortical_unit_data_type_flag: CorticalUnitDataType::SignedPercentage4D,
        number_cortical_areas: 1,
        cortical_area_default_properties: [
            Some(IOCorticalAreaDefinition {
                friendly_english_name: "Smart IMU",
                cortical_sub_unit_index: 0,
                cortical_area_data_type: CorticalAreaDataTypeFlag::SignedPercentage4D(
                    FrameChangeHandling::Absolute,
                    PercentageNeuronPositioning::Fractional
                ),
                relative_position: (100, 0, -10),
                channel_dimensions_min: (4, 1, 1),
                channel_dimensions_default: (4, 1, 10),
                channel_dimensions_max: (4, 1, 1024),
            }),
            None, None, None, None, None, None, None, None,
        ],
    },
];