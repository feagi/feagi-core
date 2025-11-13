use std::fmt::Display;
use crate::FeagiDataError;
use crate::genomic::cortical_area::{CorticalID, CorticalAreaType, IOCorticalAreaDataType};
use crate::genomic::cortical_area::descriptors::{CorticalGroupIndex, CorticalUnitIndex};
use crate::genomic::cortical_area::io_cortical_area_data_type::{DataTypeConfigurationFlag, FrameChangeHandling, PercentageNeuronPositioning};

// TODO this should be macro generated from template!
// TODO Digital needs a boolean
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum SensoryCorticalUnit {
    Infrared,
    AnalogGPIO,
    Vision,
    SegmentedVision,
    IMU,
}

impl SensoryCorticalUnit {

    // TODO macro
    pub const fn get_segmented_vision_cortical_area_types(frame_change_handling: FrameChangeHandling) -> [CorticalAreaType; 9] {
        let io_cortical_area_data_type: IOCorticalAreaDataType = IOCorticalAreaDataType::CartesianPlane(frame_change_handling);
        [
            CorticalAreaType::BrainInput(io_cortical_area_data_type),
            CorticalAreaType::BrainInput(io_cortical_area_data_type),
            CorticalAreaType::BrainInput(io_cortical_area_data_type),
            CorticalAreaType::BrainInput(io_cortical_area_data_type),
            CorticalAreaType::BrainInput(io_cortical_area_data_type),
            CorticalAreaType::BrainInput(io_cortical_area_data_type),
            CorticalAreaType::BrainInput(io_cortical_area_data_type),
            CorticalAreaType::BrainInput(io_cortical_area_data_type),
            CorticalAreaType::BrainInput(io_cortical_area_data_type),
        ]
    }

    // TODO macro
    pub const fn get_segmented_vision_cortical_ids(frame_change_handling: FrameChangeHandling, cortical_group_index: CorticalGroupIndex) -> [CorticalID; 9] {
        let io_cortical_area_data_type: IOCorticalAreaDataType = IOCorticalAreaDataType::CartesianPlane(frame_change_handling);
        let cortical_unit_identifier: [u8; 3] = *b"svi";

        [
            io_cortical_area_data_type.as_io_cortical_id(true, cortical_unit_identifier, CorticalUnitIndex::from(0), cortical_group_index),
            io_cortical_area_data_type.as_io_cortical_id(true, cortical_unit_identifier, CorticalUnitIndex::from(1), cortical_group_index),
            io_cortical_area_data_type.as_io_cortical_id(true, cortical_unit_identifier, CorticalUnitIndex::from(2), cortical_group_index),
            io_cortical_area_data_type.as_io_cortical_id(true, cortical_unit_identifier, CorticalUnitIndex::from(3), cortical_group_index),
            io_cortical_area_data_type.as_io_cortical_id(true, cortical_unit_identifier, CorticalUnitIndex::from(4), cortical_group_index),
            io_cortical_area_data_type.as_io_cortical_id(true, cortical_unit_identifier, CorticalUnitIndex::from(5), cortical_group_index),
            io_cortical_area_data_type.as_io_cortical_id(true, cortical_unit_identifier, CorticalUnitIndex::from(6), cortical_group_index),
            io_cortical_area_data_type.as_io_cortical_id(true, cortical_unit_identifier, CorticalUnitIndex::from(7), cortical_group_index),
            io_cortical_area_data_type.as_io_cortical_id(true, cortical_unit_identifier, CorticalUnitIndex::from(8), cortical_group_index),
        ]

    }





}

impl Display for SensoryCorticalUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SensoryCorticalUnit::Infrared => write!(f, "Infrared"),
            SensoryCorticalUnit::AnalogGPIO => write!(f, "Analog GPIO"),
            SensoryCorticalUnit::Vision => write!(f, "Vision"),
            SensoryCorticalUnit::SegmentedVision => write!(f, "Segmented Vision"),
            SensoryCorticalUnit::IMU => write!(f, "IMU"),
        }
    }
}