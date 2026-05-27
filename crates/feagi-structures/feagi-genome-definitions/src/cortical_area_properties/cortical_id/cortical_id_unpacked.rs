use crate::cortical_area_properties::cortical_area_types::CoreCorticalType;
use crate::cortical_area_properties::cortical_id::CorticalIDPacked;
use crate::cortical_area_properties::io_cortical_area_properties::{CorticalSubUnitIndex, CorticalUnitIndex};
use crate::cortical_units::{MotorCorticalUnit, SensorCorticalUnit};
use crate::cortical_units::definitions::cortical_area_common::CorticalAreaDataTypeFlag;

/// Number of bytes expected for a cortical identifier

/// Represents a cortical area identifier as a set of nested enums, mainly for easier interpretation
pub enum CorticalIDUnpacked {
    Core(CoreCorticalType),
    Memory([u8; CorticalIDPacked::BYTE_COUNT]), // We need the full on bytes as they are randomly generated, we need them to go back
    Custom([u8; CorticalIDPacked::BYTE_COUNT]),
    Sensor(SensorCorticalUnit, CorticalUnitIndex, CorticalSubUnitIndex, CorticalAreaDataTypeFlag),
    Motor(MotorCorticalUnit, CorticalUnitIndex, CorticalSubUnitIndex, CorticalAreaDataTypeFlag),
}

impl CorticalIDUnpacked {
    pub fn to_packed(&self) -> CorticalIDPacked {
        match self {
            CorticalIDUnpacked::Core(core_type) => {
                core_type.to_cortical_identifier_packed()
            }
            CorticalIDUnpacked::Memory(bytes) => {
                CorticalIDPacked::new_const_unchecked(*bytes)
            }
            CorticalIDUnpacked::Custom(bytes) => {
                CorticalIDPacked::new_const_unchecked(*bytes)
            }
            CorticalIDUnpacked::Sensor(_, _, _, _) => {
                todo!()
            }
            CorticalIDUnpacked::Motor(_, _, _, _) => {
                todo!()
            }
        }
    }
}





