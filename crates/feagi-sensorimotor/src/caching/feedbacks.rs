

/*


    // TODO we need to discuss how to handle absolute,  linear, and we need to figure out better error handling ehre
    // TODO we can change the call back signature // TODO feedback
    pub fn reflex_absolute_gaze_to_absolute_segmented_vision(&mut self, gaze_group: CorticalGroupIndex, gaze_channel: CorticalChannelIndex, segmentation_group: CorticalGroupIndex, segmentation_channel: CorticalChannelIndex) -> Result<FeagiSignalIndex, FeagiDataError> {

        // Simple way to check if valid. // TODO we should probably have a proper method
        let mut m = self.motors.lock().unwrap();
        _ = m.try_read_postprocessed_cached_value(MotorCorticalType::GazeAbsoluteLinear, gaze_group, gaze_channel)?;

        let s = self.sensors.lock().unwrap();
        _ = s.try_read_postprocessed_cached_value(SensorCorticalType::ImageCameraCenterAbsolute, segmentation_group, segmentation_channel)?;
        mem::drop(s);

        let motor_ref = Arc::clone(&self.motors);
        let sensor_ref = Arc::clone(&self.sensors);

        let closure = move |_: &()| {
            let motors = motor_ref.lock().unwrap();
            let wrapped_motor = motors.try_read_postprocessed_cached_value(MotorCorticalType::GazeAbsoluteLinear, gaze_group, gaze_channel).unwrap();
            let per_4d: Percentage4D = wrapped_motor.try_into().unwrap();
            let gaze: GazeProperties = GazeProperties::new_from_4d(per_4d);

            let mut sensors = sensor_ref.lock().unwrap();
            let stage_properties = sensors.try_get_single_stage_properties(SensorCorticalType::ImageCameraCenterAbsolute, segmentation_group, segmentation_channel, 0.into()).unwrap();
            let mut segmentation_stage_properties: ImageSegmentorStageProperties = stage_properties.as_any().downcast_ref::<ImageSegmentorStageProperties>().unwrap().clone();
            segmentation_stage_properties.update_from_gaze(gaze);
            _ = sensors.try_update_single_stage_properties(SensorCorticalType::ImageCameraCenterAbsolute, segmentation_group, segmentation_channel, 0.into(), Box::new(segmentation_stage_properties));
        };

        let index = m.try_register_motor_callback(MotorCorticalType::GazeAbsoluteLinear, gaze_group, gaze_channel, closure)?;

        Ok(index)
    }

 */