use crate::genomic::cortical_area::{CorticalID, CorticalType, IOCorticalAreaDataType};
use crate::genomic::cortical_area::io_cortical_area_data_type::{FrameChangeHandling, PercentageNeuronPositioning};

// TODO this should be macro generated from template!
pub enum SensoryCorticalUnit {
    Infrared(FrameChangeHandling, PercentageNeuronPositioning),
    Vision(FrameChangeHandling),
    SegmentedVision(FrameChangeHandling), // 9 cartesian cortical areas
    IMU(FrameChangeHandling, PercentageNeuronPositioning), // 1 3d percentage area, 1 4d percentage area // TODO placeholder!
}

// TODO talk about experience of user of switching from one type to the other

// TODO I hate allocating on the heap. There are some other crates that can do "vectors" on the stack that may be worth looking into

impl SensoryCorticalUnit {
    pub fn get_as_cortical_types(&self) -> Vec<CorticalType> {
        match self {
            SensoryCorticalUnit::Infrared(handling, percentage_neuron_positioning) => {
                vec![CorticalType::BrainInput(IOCorticalAreaDataType::Percentage(handling.clone(), percentage_neuron_positioning.clone()))]
            }
            SensoryCorticalUnit::Vision(handling) => {
                vec![CorticalType::BrainInput(IOCorticalAreaDataType::CartesianPlane(*handling))]
            }
            SensoryCorticalUnit::SegmentedVision(handling) => {
                vec![
                    CorticalType::BrainInput(IOCorticalAreaDataType::CartesianPlane(*handling)),
                    CorticalType::BrainInput(IOCorticalAreaDataType::CartesianPlane(*handling)),
                    CorticalType::BrainInput(IOCorticalAreaDataType::CartesianPlane(*handling)),
                    CorticalType::BrainInput(IOCorticalAreaDataType::CartesianPlane(*handling)),
                    CorticalType::BrainInput(IOCorticalAreaDataType::CartesianPlane(*handling)),
                    CorticalType::BrainInput(IOCorticalAreaDataType::CartesianPlane(*handling)),
                    CorticalType::BrainInput(IOCorticalAreaDataType::CartesianPlane(*handling)),
                    CorticalType::BrainInput(IOCorticalAreaDataType::CartesianPlane(*handling)),
                    CorticalType::BrainInput(IOCorticalAreaDataType::CartesianPlane(*handling)),
                ]
            }
            SensoryCorticalUnit::IMU(handling, percentage_neuron_positioning) => {
                vec![
                    CorticalType::BrainInput(IOCorticalAreaDataType::SignedPercentage3D(*handling, *percentage_neuron_positioning)),
                    CorticalType::BrainInput(IOCorticalAreaDataType::SignedPercentage4D(*handling, *percentage_neuron_positioning)),
                ]
            }
        }
    }

    pub fn get_as_cortical_ids(&self) -> Vec<CorticalID> {
        match self {
            SensoryCorticalUnit::Infrared(handling, percentage_neuron_positioning) => {
                vec![CorticalType::BrainInput(IOCorticalAreaDataType::Percentage(handling.clone(), percentage_neuron_positioning.clone()))]
            }
            SensoryCorticalUnit::Vision(handling) => {
                vec![CorticalType::BrainInput(IOCorticalAreaDataType::CartesianPlane(*handling))]
            }
            SensoryCorticalUnit::SegmentedVision(handling) => {
                vec![
                    CorticalType::BrainInput(IOCorticalAreaDataType::CartesianPlane(*handling)),
                    CorticalType::BrainInput(IOCorticalAreaDataType::CartesianPlane(*handling)),
                    CorticalType::BrainInput(IOCorticalAreaDataType::CartesianPlane(*handling)),
                    CorticalType::BrainInput(IOCorticalAreaDataType::CartesianPlane(*handling)),
                    CorticalType::BrainInput(IOCorticalAreaDataType::CartesianPlane(*handling)),
                    CorticalType::BrainInput(IOCorticalAreaDataType::CartesianPlane(*handling)),
                    CorticalType::BrainInput(IOCorticalAreaDataType::CartesianPlane(*handling)),
                    CorticalType::BrainInput(IOCorticalAreaDataType::CartesianPlane(*handling)),
                    CorticalType::BrainInput(IOCorticalAreaDataType::CartesianPlane(*handling)),
                ]
            }
            SensoryCorticalUnit::IMU(handling, percentage_neuron_positioning) => {
                vec![
                    CorticalType::BrainInput(IOCorticalAreaDataType::SignedPercentage3D(*handling, *percentage_neuron_positioning)),
                    CorticalType::BrainInput(IOCorticalAreaDataType::SignedPercentage4D(*handling, *percentage_neuron_positioning)),
                ]
            }
        }
    }


}