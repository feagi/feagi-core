use crate::caching::{MotorDeviceCache, SensorDeviceCache};
use crate::feedbacks::feedback_registration::FeedBackRegistration;
use crate::feedbacks::feedback_registration_targets::FeedbackRegistrationTargets;
use feagi_structures::FeagiDataError;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct FeedbackRegistrar {
    registered_feedbacks: Vec<(FeedbackRegistrationTargets, FeedBackRegistration)>,
}

impl FeedbackRegistrar {
    pub fn new() -> FeedbackRegistrar {
        FeedbackRegistrar {
            registered_feedbacks: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.registered_feedbacks.clear();
    }

    pub fn reload_all_from_self(
        &mut self,
        sensor_cache: Arc<Mutex<SensorDeviceCache>>,
        motor_cache: Arc<Mutex<MotorDeviceCache>>,
    ) -> Result<(), FeagiDataError> {
        for feedback in self.registered_feedbacks.iter() {
            let target = feedback.0.clone();
            let registration = feedback.1.clone();
            registration.try_registering_feedbacks(
                sensor_cache.clone(),
                motor_cache.clone(),
                target,
            )?;
        }
        Ok(())
    }

    pub(crate) fn push_verified_feedback(
        &mut self,
        target: FeedbackRegistrationTargets,
        feed_back_registration: FeedBackRegistration,
    ) -> Result<(), FeagiDataError> {
        self.verify_not_contain_registration(&target, &feed_back_registration)?;
        self.registered_feedbacks
            .push((target, feed_back_registration));
        Ok(())
    }

    fn verify_not_contain_registration(
        &self,
        targets: &FeedbackRegistrationTargets,
        registration: &FeedBackRegistration,
    ) -> Result<(), FeagiDataError> {
        let compare = &(targets.clone(), registration.clone());
        if self.registered_feedbacks.contains(compare) {
            return Err(FeagiDataError::BadParameters(format!(
                "Feedback {} already registered to motor unit {} channel {}, and sensor unit {} channel {}!",
                compare.1,
                compare.0.get_motor_unit_index(),
                compare.0.get_motor_channel_index(),
                compare.0.get_sensor_unit_index(),
                compare.0.get_sensor_channel_index(),
            )));
        }
        Ok(())
    }
}
