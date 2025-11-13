use crate::genomic::cortical_area::{CorticalID, CorticalType, IOCorticalAreaDataType};
use crate::genomic::cortical_area::descriptors::{CorticalGroupIndex, CorticalUnitIndex};
use crate::genomic::cortical_area::io_cortical_area_data_type::{FrameChangeHandling, PercentageNeuronPositioning};

// TODO this should be macro generated from template!
// TODO Dighital needs a boolean
pub enum SensoryCorticalUnit {
    Infrared(FrameChangeHandling, PercentageNeuronPositioning),
    AnalogGPIO(FrameChangeHandling, PercentageNeuronPositioning),
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
            SensoryCorticalUnit::AnalogGPIO(handling, percentage_neuron_positioning) => {
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

    pub fn get_as_cortical_ids(&self, group: CorticalGroupIndex) -> Vec<CorticalID> {
        match self {
            SensoryCorticalUnit::Infrared(handling, percentage_neuron_positioning) => {
                vec![IOCorticalAreaDataType::Percentage(handling.clone(), percentage_neuron_positioning.clone()).as_io_cortical_id(true, *b"inf", CorticalUnitIndex::from(0), group)]
            }
            SensoryCorticalUnit::AnalogGPIO(handling, percentage_neuron_positioning) => {
                vec![IOCorticalAreaDataType::Percentage(handling.clone(), percentage_neuron_positioning.clone()).as_io_cortical_id(true, *b"inf", CorticalUnitIndex::from(0), group)]
            }
            SensoryCorticalUnit::Vision(handling) => {
                vec![IOCorticalAreaDataType::CartesianPlane(*handling).as_io_cortical_id(true, *b"vis", CorticalUnitIndex::from(0), group)]
            }
            SensoryCorticalUnit::SegmentedVision(handling) => {
                vec![
                    IOCorticalAreaDataType::CartesianPlane(*handling).as_io_cortical_id(true, *b"svi", CorticalUnitIndex::from(0), group),
                    IOCorticalAreaDataType::CartesianPlane(*handling).as_io_cortical_id(true, *b"svi", CorticalUnitIndex::from(1), group),
                    IOCorticalAreaDataType::CartesianPlane(*handling).as_io_cortical_id(true, *b"svi", CorticalUnitIndex::from(2), group),
                    IOCorticalAreaDataType::CartesianPlane(*handling).as_io_cortical_id(true, *b"svi", CorticalUnitIndex::from(3), group),
                    IOCorticalAreaDataType::CartesianPlane(*handling).as_io_cortical_id(true, *b"svi", CorticalUnitIndex::from(4), group),
                    IOCorticalAreaDataType::CartesianPlane(*handling).as_io_cortical_id(true, *b"svi", CorticalUnitIndex::from(5), group),
                    IOCorticalAreaDataType::CartesianPlane(*handling).as_io_cortical_id(true, *b"svi", CorticalUnitIndex::from(6), group),
                    IOCorticalAreaDataType::CartesianPlane(*handling).as_io_cortical_id(true, *b"svi", CorticalUnitIndex::from(7), group),
                    IOCorticalAreaDataType::CartesianPlane(*handling).as_io_cortical_id(true, *b"svi", CorticalUnitIndex::from(8), group),
                ]
            }
            SensoryCorticalUnit::IMU(handling, percentage_neuron_positioning) => {

                vec![
                    IOCorticalAreaDataType::SignedPercentage3D(*handling, *percentage_neuron_positioning).as_io_cortical_id(true, *b"imu", CorticalUnitIndex::from(0), group),
                    IOCorticalAreaDataType::SignedPercentage4D(*handling, *percentage_neuron_positioning).as_io_cortical_id(true, *b"imu", CorticalUnitIndex::from(1), group),
                ]
            }
        }
    }


}