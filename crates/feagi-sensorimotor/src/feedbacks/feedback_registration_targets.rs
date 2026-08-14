use crate::caching::{MotorDeviceCache, SensorDeviceCache};
use crate::data_types::descriptors::CorticalChannelIndex;
use feagi_data::feagi_data_error::FeagiDataError;
use feagi_genomic_context::cortical_unit::motor_cortical_unit::MotorCorticalUnit;
use feagi_genomic_context::cortical_unit::sensor_cortical_unit::SensoryCorticalUnit;
use feagi_genomic_context::cortical_unit::CorticalUnitIndex;
use serde::{Deserialize, Serialize};
use std::sync::MutexGuard;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FeedbackRegistrationTargets {
    #[serde(with = "crate::cortical_unit_index_serde")]
    sensor_unit_index: CorticalUnitIndex,
    sensor_channel_index: CorticalChannelIndex,
    #[serde(with = "crate::cortical_unit_index_serde")]
    motor_unit_index: CorticalUnitIndex,
    motor_channel_index: CorticalChannelIndex,
}

impl FeedbackRegistrationTargets {
    pub fn new(
        sensor_unit_index: CorticalUnitIndex,
        sensor_channel_index: CorticalChannelIndex,
        motor_unit_index: CorticalUnitIndex,
        motor_channel_index: CorticalChannelIndex,
    ) -> Self {
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

    #[allow(dead_code)]
    pub(crate) fn verify_existence(
        &self,
        sensor_type: SensoryCorticalUnit,
        motor_type: MotorCorticalUnit,
        sensor_cache: MutexGuard<'_, SensorDeviceCache>,
        motor_cache: MutexGuard<'_, MotorDeviceCache>,
    ) -> Result<(), FeagiDataError> {
        sensor_cache.verify_existence(
            sensor_type,
            self.sensor_unit_index,
            self.sensor_channel_index,
        )?;
        motor_cache.verify_existence(
            motor_type,
            self.motor_unit_index,
            self.motor_channel_index,
        )?;
        Ok(())
    }
}
