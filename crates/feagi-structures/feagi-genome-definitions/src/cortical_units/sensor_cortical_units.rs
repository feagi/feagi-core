use crate::cortical_area_properties::io_cortical_area_properties::{CorticalSubUnitIndex, CorticalUnitIndex};
use crate::cortical_units::definitions::cortical_area_common::CorticalAreaDataTypeFlag;

pub enum SensorCorticalUnit {
    // TODO macro gen
}

impl SensorCorticalUnit {
    pub fn cortical_id_from_properties(unit_index: CorticalUnitIndex,subunit_index: CorticalSubUnitIndex, data_type_flag: CorticalAreaDataTypeFlag) -> SensorCorticalUnit {
        todo!()
    }
}