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

    //region infrared
    pub const fn get_infrared_cortical_area_types_array(frame_change_handling: FrameChangeHandling, percentage_neuron_positioning: PercentageNeuronPositioning) -> [CorticalAreaType; 1] {
        [
            CorticalAreaType::BrainInput(IOCorticalAreaDataType::Percentage(frame_change_handling, percentage_neuron_positioning)),
        ]
    }

    pub const fn get_infrared_cortical_ids_array(frame_change_handling: FrameChangeHandling, percentage_neuron_positioning: PercentageNeuronPositioning, cortical_group_index: CorticalGroupIndex) -> [CorticalID; 1] {
        let cortical_unit_identifier: [u8; 3] = *b"inf";

        [
            IOCorticalAreaDataType::Percentage(frame_change_handling, percentage_neuron_positioning).as_io_cortical_id(true, cortical_unit_identifier, CorticalUnitIndex::from(0), cortical_group_index),
        ]

    }
    //endregion

    //region segmented_vision
    pub const fn get_segmented_vision_cortical_area_types_array(frame_change_handling: FrameChangeHandling) -> [CorticalAreaType; 9] {
        [
            CorticalAreaType::BrainInput(IOCorticalAreaDataType::CartesianPlane(frame_change_handling)),
            CorticalAreaType::BrainInput(IOCorticalAreaDataType::CartesianPlane(frame_change_handling)),
            CorticalAreaType::BrainInput(IOCorticalAreaDataType::CartesianPlane(frame_change_handling)),
            CorticalAreaType::BrainInput(IOCorticalAreaDataType::CartesianPlane(frame_change_handling)),
            CorticalAreaType::BrainInput(IOCorticalAreaDataType::CartesianPlane(frame_change_handling)),
            CorticalAreaType::BrainInput(IOCorticalAreaDataType::CartesianPlane(frame_change_handling)),
            CorticalAreaType::BrainInput(IOCorticalAreaDataType::CartesianPlane(frame_change_handling)),
            CorticalAreaType::BrainInput(IOCorticalAreaDataType::CartesianPlane(frame_change_handling)),
            CorticalAreaType::BrainInput(IOCorticalAreaDataType::CartesianPlane(frame_change_handling)),
        ]
    }

    pub const fn get_segmented_vision_cortical_ids_array(frame_change_handling: FrameChangeHandling, cortical_group_index: CorticalGroupIndex) -> [CorticalID; 9] {
        let cortical_unit_identifier: [u8; 3] = *b"svi";

        [
            IOCorticalAreaDataType::CartesianPlane(frame_change_handling).as_io_cortical_id(true, cortical_unit_identifier, CorticalUnitIndex::from(0), cortical_group_index),
            IOCorticalAreaDataType::CartesianPlane(frame_change_handling).as_io_cortical_id(true, cortical_unit_identifier, CorticalUnitIndex::from(1), cortical_group_index),
            IOCorticalAreaDataType::CartesianPlane(frame_change_handling).as_io_cortical_id(true, cortical_unit_identifier, CorticalUnitIndex::from(2), cortical_group_index),
            IOCorticalAreaDataType::CartesianPlane(frame_change_handling).as_io_cortical_id(true, cortical_unit_identifier, CorticalUnitIndex::from(3), cortical_group_index),
            IOCorticalAreaDataType::CartesianPlane(frame_change_handling).as_io_cortical_id(true, cortical_unit_identifier, CorticalUnitIndex::from(4), cortical_group_index),
            IOCorticalAreaDataType::CartesianPlane(frame_change_handling).as_io_cortical_id(true, cortical_unit_identifier, CorticalUnitIndex::from(5), cortical_group_index),
            IOCorticalAreaDataType::CartesianPlane(frame_change_handling).as_io_cortical_id(true, cortical_unit_identifier, CorticalUnitIndex::from(6), cortical_group_index),
            IOCorticalAreaDataType::CartesianPlane(frame_change_handling).as_io_cortical_id(true, cortical_unit_identifier, CorticalUnitIndex::from(7), cortical_group_index),
            IOCorticalAreaDataType::CartesianPlane(frame_change_handling).as_io_cortical_id(true, cortical_unit_identifier, CorticalUnitIndex::from(8), cortical_group_index),
        ]

    }
    //endregion





}

impl Display for SensoryCorticalUnit {

    // TODO macro
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