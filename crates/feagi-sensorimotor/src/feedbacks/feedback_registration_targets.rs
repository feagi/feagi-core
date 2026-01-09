use std::sync::{MutexGuard};
use serde::{Deserialize, Serialize};
use feagi_structures::FeagiDataError;
use feagi_structures::genomic::cortical_area::descriptors::{CorticalChannelIndex, CorticalUnitIndex};
use feagi_structures::genomic::{MotorCorticalUnit, SensoryCorticalUnit};
use crate::caching::{MotorDeviceCache, SensorDeviceCache};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(PartialEq)]
pub struct 
FeedbackRegistrationTargets {
    sensor_unit_index: CorticalUnitIndex,
    sensor_channel_index: CorticalChannelIndex,
    motor_unit_index: CorticalUnitIndex,
    motor_channel_index: CorticalChannelIndex,
}

impl FeedbackRegistrationTargets {
    pub fn new(sensor_unit_index: CorticalUnitIndex,
               sensor_channel_index: CorticalChannelIndex,
               motor_unit_index: CorticalUnitIndex,
               motor_channel_index: CorticalChannelIndex) -> Self {
        FeedbackRegistrationTargets {
            sensor_unit_index,
            sensor_channel_index,
            motor_unit_index,
            motor_channel_index,
        }
    }

    pub fn get_sensor_unit_index(&self) -> CorticalUnitIndex {
        self.sensor_unit_index
    }

    pub fn get_sensor_channel_index(&self) -> CorticalChannelIndex {
        self.sensor_channel_index
    }

    pub fn get_motor_unit_index(&self) -> CorticalUnitIndex {
        self.motor_unit_index
    }

    pub fn get_motor_channel_index(&self) -> CorticalChannelIndex {
        self.motor_channel_index
    }

    pub(crate) fn verify_existence(&self,
                                   sensor_type: SensoryCorticalUnit,
                                   motor_type: MotorCorticalUnit,
                                   sensor_cache: MutexGuard<'_, SensorDeviceCache>,
                                   motor_cache: MutexGuard<'_, MotorDeviceCache>
    ) -> Result<(), FeagiDataError> {
        sensor_cache.verify_existence(sensor_type, self.sensor_unit_index, self.sensor_channel_index)?;
        motor_cache.verify_existence(motor_type, self.motor_unit_index, self.motor_channel_index)?;
        Ok(())
    }
}