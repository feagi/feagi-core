//! Sensor cortical type definitions for FEAGI.
//!
//! Defines all supported sensor types including infrared, ultrasonic,
//! accelerometer, gyroscope, camera, and various other sensory inputs.

/// Macro defining all sensor (input processing unit) cortical types.
///
/// This macro generates enum variants and associated metadata for each
/// sensor type, including encoding methods, dimension ranges, and identifiers.
///
/// # Usage
/// ```ignore
/// sensor_definition!(define_io_cortical_types);
/// ```
#[macro_export]
macro_rules! sensor_definition {
    ($callback:ident) => {
        $callback! {
            SensorCorticalType {

                //region Percentage

                #[doc = "Infrared distance sensor for object detection."]
                Infrared => {
                    friendly_name: "Infrared Sensor",
                    snake_case_identifier: "infrared",
                    wrapped_data_type: WrappedIOType::Percentage,
                    data_type: Percentage,
                    subtype_mappings: [
                        (b"iinf00", (Absolute, Linear), Percentage_Absolute_Linear),
                        (b"iINF00", (Absolute, Fractional), Percentage_Absolute_Fractional),
                        (b"Iinf00", (Incremental, Linear), Percentage_Incremental_Linear),
                        (b"IINF00", (Incremental, Fractional), Percentage_Incremental_Fractional),
                    ]
                },


                #[doc = "Inverted infrared sensor that provides reverse object detection readings."]
                ReverseInfraredAbsoluteLinear => {
                    friendly_name: "Infrared (Inverted) Sensor (Absolute Change, Linear Encoding)",
                    snake_case_identifier: "infrared_inverted_absolute_linear",
                    base_ascii: b"iiif00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..2),
                    default_coder_type: Percentage_Absolute_Linear,
                    wrapped_data_type: WrappedIOType::Percentage,
                    data_type: Percentage,
                    subtype_mappings: [
                        (b"iiif00", (Absolute, Linear), Percentage_Absolute_Linear),
                        (b"iIIF00", (Absolute, Fractional), Percentage_Absolute_Fractional),
                        (b"Iiif00", (Incremental, Linear), Percentage_Incremental_Linear),
                        (b"IIIF00", (Incremental, Fractional), Percentage_Incremental_Fractional),
                    ]
                },


                #[doc = "Digital GPIO input pin for reading binary signals (high/low states). Absolute (instant) change. Neurons encoded linearly"]
                DigitalGPIOInputAbsoluteLinear => {
                    friendly_name: "GPIO Digital Input (Absolute Change, Linear Encoding)",
                    snake_case_identifier: "gpio_digital_absolute_linear",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..2),
                    default_coder_type: Percentage_Absolute_Linear,
                    wrapped_data_type: WrappedIOType::Percentage,
                    data_type: Percentage,
                    subtype_mappings: [
                        (b"idgp00", (Absolute, Linear), Percentage_Absolute_Linear),
                        (b"iDGP00", (Absolute, Fractional), Percentage_Absolute_Fractional),
                        (b"Idgp00", (Incremental, Linear), Percentage_Incremental_Linear),
                        (b"IDGP00", (Incremental, Fractional), Percentage_Incremental_Fractional),
                    ]
                },


                #[doc = "Proximity sensor for detecting nearby objects and measuring distances. Absolute (instant) change. Neurons encoded linearly"]
                ProximityAbsoluteLinear => {
                    friendly_name: "Proximity (Absolute Change, Linear Encoding)",
                    snake_case_identifier: "proximity_absolute_linear",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: Percentage_Absolute_Linear,
                    wrapped_data_type: WrappedIOType::Percentage,
                    data_type: Percentage,
                    subtype_mappings: [
                        (b"ipro00", (Absolute, Linear), Percentage_Absolute_Linear),
                        (b"iPRO00", (Absolute, Fractional), Percentage_Absolute_Fractional),
                        (b"Ipro00", (Incremental, Linear), Percentage_Incremental_Linear),
                        (b"IPRO00", (Incremental, Fractional), Percentage_Incremental_Fractional),
                    ]
                },


                #[doc = "Shock sensor for sensing 'pain'. Absolute (instant) change. Neurons encoded linearly"]
                ShockAbsoluteLinear => {
                    friendly_name: "Shock (Absolute Change, Linear Encoding)",
                    snake_case_identifier: "shock_absolute_linear",
                    base_ascii: b"ishk00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..2),
                    default_coder_type: Percentage_Absolute_Linear,
                    wrapped_data_type: WrappedIOType::Percentage,
                    data_type: Percentage,
                    subtype_mappings: [
                        (b"ishk00", (Absolute, Linear), Percentage_Absolute_Linear),
                        (b"iSHK00", (Absolute, Fractional), Percentage_Absolute_Fractional),
                        (b"Ishk00", (Incremental, Linear), Percentage_Incremental_Linear),
                        (b"ISHK00", (Incremental, Fractional), Percentage_Incremental_Fractional),
                    ]
                },


                #[doc = "Battery level sensor for monitoring power remaining. Absolute (instant) change. Neurons encoded linearly"]
                BatteryAbsoluteLinear => {
                    friendly_name: "Battery Gauge (Absolute Change, Linear Encoding)",
                    snake_case_identifier: "battery_gauge_absolute_linear",
                    base_ascii: b"ibat00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: Percentage_Absolute_Linear,
                    wrapped_data_type: WrappedIOType::Percentage,
                    data_type: Percentage,
                    subtype_mappings: [
                        (b"ibat00", (Absolute, Linear), Percentage_Absolute_Linear),
                        (b"iBAT00", (Absolute, Fractional), Percentage_Absolute_Fractional),
                        (b"Ibat00", (Incremental, Linear), Percentage_Incremental_Linear),
                        (b"IBAT00", (Incremental, Fractional), Percentage_Incremental_Fractional),
                    ]
                },


                #[doc = "Analog GPIO input pin for reading binary signals (high/low states). Absolute (instant) change. Neurons encoded linearly"]
                AnalogGPIOInputAbsoluteLinear => {
                    friendly_name: "GPIO Analog Input (Absolute Change, Linear Encoding)",
                    snake_case_identifier: "gpio_analog_absolute_linear",
                    base_ascii: b"iagp00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: Percentage_Absolute_Linear,
                    wrapped_data_type: WrappedIOType::Percentage,
                    data_type: Percentage,
                    subtype_mappings: [
                        (b"iagp00", (Absolute, Linear), Percentage_Absolute_Linear),
                        (b"iAGP00", (Absolute, Fractional), Percentage_Absolute_Fractional),
                        (b"Iagp00", (Incremental, Linear), Percentage_Incremental_Linear),
                        (b"IAGP00", (Incremental, Fractional), Percentage_Incremental_Fractional),
                    ]
                },

                //endregion

                //region SignedFloat

                #[doc = "Servo position feedback sensor for monitoring actuator positions. Absolute (instant) change. Neurons encoded linearly"]
                ServoPositionAbsoluteLinear => {
                    friendly_name: "Servo Position (Absolute Change, Linear Encoding)",
                    snake_case_identifier: "servo_position_absolute_linear",
                    base_ascii: b"isvp00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: SignedPercentage_Absolute_Linear,
                    wrapped_data_type: WrappedIOType::SignedPercentage,
                    data_type: SignedPercentage,
                    subtype_mappings: [
                        (b"isvp00", (Absolute, Linear), SignedPercentage_Absolute_Linear),
                        (b"iSVP00", (Absolute, Fractional), SignedPercentage_Absolute_Fractional),
                        (b"Isvp00", (Incremental, Linear), SignedPercentage_Incremental_Linear),
                        (b"ISVP00", (Incremental, Fractional), SignedPercentage_Incremental_Fractional),
                    ]
                },


                #[doc = "Servo motion feedback sensor for monitoring actuator positions. Absolute (instant) change. Neurons encoded linearly"]
                ServoMotionAbsoluteLinear => {
                    friendly_name: "Servo Motion (Absolute Change, Linear Encoding)",
                    snake_case_identifier: "servo_motion_absolute_linear",
                    base_ascii: b"isvm00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: SignedPercentage_Absolute_Linear,
                    wrapped_data_type: WrappedIOType::SignedPercentage,
                    data_type: SignedPercentage,
                    subtype_mappings: [
                        (b"isvm00", (Absolute, Linear), SignedPercentage_Absolute_Linear),
                        (b"iSVM00", (Absolute, Fractional), SignedPercentage_Absolute_Fractional),
                        (b"Isvm00", (Incremental, Linear), SignedPercentage_Incremental_Linear),
                        (b"ISVM00", (Incremental, Fractional), SignedPercentage_Incremental_Fractional),
                    ]
                },

                //endregion

                //region Misc

                #[doc = "Miscellaneous area for all types of data FEAGI has no specific implementation for. Absolute (instant) change."]
                MiscellaneousAbsolute=> {
                    friendly_name: "Miscellaneous (Absolute Change)",
                    snake_case_identifier: "miscellaneous_absolute",
                    base_ascii: b"imis00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..u32::MAX),
                    default_coder_type: MiscData_Absolute,
                    wrapped_data_type: WrappedIOType::MiscData(None),
                    data_type: MiscData,
                    subtype_mappings: [
                        (b"imis00", (Absolute), MiscData_Absolute),
                        (b"Imis00", (Incremental), MiscData_Incremental),
                    ]
                },

                //endregion

                //region ImageFrame

                #[doc = "Image camera input. Either alone or in the center of segmented/peripheral image camera setups. Absolute (instant) change."]
                ImageCameraCenterAbsolute => {
                    friendly_name: "Center Image Camera Input (Absolute Change)",
                    snake_case_identifier: "image_camera_center_absolute",
                    base_ascii: b"iic400",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Absolute,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                    data_type: ImageFrame,
                },

                #[doc = "Image camera input. Either alone or in the center of segmented/peripheral image camera setups. Incremental change."]
                ImageCameraCenterIncremental => {
                    friendly_name: "Center Image Camera Input (Incremental Change)",
                    snake_case_identifier: "image_camera_center_incremental",
                    base_ascii: b"Iic400",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Incremental,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                    data_type: ImageFrame,
                },
                #[doc = "Top Left peripheral image camera input. Absolute (instant) change."]
                ImageCameraTopLeftAbsolute => {
                    friendly_name: "Top Left Image Camera Input (Absolute Change)",
                    snake_case_identifier: "image_camera_top_left_absolute",
                    base_ascii: b"iic600",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Absolute,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                    data_type: ImageFrame,
                },

                #[doc = "Top Left peripheral image camera input. Incremental change."]
                ImageCameraTopLeftIncremental => {
                    friendly_name: "Top Left Image Camera Input (Incremental Change)",
                    snake_case_identifier: "image_camera_top_left_incremental",
                    base_ascii: b"Iic600",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Incremental,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                    data_type: ImageFrame,
                },

                #[doc = "Top Middle peripheral image camera input. Absolute (instant) change."]
                ImageCameraTopMiddleAbsolute => {
                    friendly_name: "Top Middle Image Camera Input (Absolute Change)",
                    snake_case_identifier: "image_camera_top_middle_absolute",
                    base_ascii: b"iic700",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Absolute,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                    data_type: ImageFrame,
                },

                #[doc = "Top Middle peripheral image camera input. Incremental change."]
                ImageCameraTopMiddleIncremental => {
                    friendly_name: "Top Middle Image Camera Input (Incremental Change)",
                    snake_case_identifier: "image_camera_top_middle_incremental",
                    base_ascii: b"Iic700",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Incremental,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                    data_type: ImageFrame,
                },
                #[doc = "Top Right peripheral image camera input. Absolute (instant) change."]
                ImageCameraTopRightAbsolute => {
                    friendly_name: "Top Right Image Camera Input (Absolute Change)",
                    snake_case_identifier: "image_camera_top_right_absolute",
                    base_ascii: b"iic800",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Absolute,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                    data_type: ImageFrame,
                },

                #[doc = "Top Right peripheral image camera input. Incremental change."]
                ImageCameraTopRightIncremental => {
                    friendly_name: "Top Right Image Camera Input (Incremental Change)",
                    snake_case_identifier: "image_camera_top_right_incremental",
                    base_ascii: b"Iic800",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Incremental,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                    data_type: ImageFrame,
                },

                #[doc = "Middle Left peripheral image camera input. Absolute (instant) change."]
                ImageCameraMiddleLeftAbsolute => {
                    friendly_name: "Middle Left Image Camera Input (Absolute Change)",
                    snake_case_identifier: "image_camera_middle_left_absolute",
                    base_ascii: b"iic300",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Absolute,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                    data_type: ImageFrame,
                },

                #[doc = "Middle Left peripheral image camera input. Incremental change."]
                ImageCameraMiddleLeftIncremental => {
                    friendly_name: "Middle Left Image Camera Input (Incremental Change)",
                    snake_case_identifier: "image_camera_middle_left_incremental",
                    base_ascii: b"Iic300",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Incremental,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                    data_type: ImageFrame,
                },

                #[doc = "Middle Right peripheral image camera input. Absolute (instant) change."]
                ImageCameraMiddleRightAbsolute => {
                    friendly_name: "Middle Right Image Camera Input (Absolute Change)",
                    snake_case_identifier: "image_camera_middle_right_absolute",
                    base_ascii: b"iic500",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Absolute,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                    data_type: ImageFrame,
                },

                #[doc = "Middle Right peripheral image camera input. Incremental change."]
                ImageCameraMiddleRightIncremental => {
                    friendly_name: "Middle Right Image Camera Input (Incremental Change)",
                    snake_case_identifier: "image_camera_middle_right_incremental",
                    base_ascii: b"Iic500",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Incremental,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                    data_type: ImageFrame,
                },

                #[doc = "Bottom Left peripheral image camera input. Absolute (instant) change."]
                ImageCameraBottomLeftAbsolute => {
                    friendly_name: "Bottom Left Image Camera Input (Absolute Change)",
                    snake_case_identifier: "image_camera_bottom_left_absolute",
                    base_ascii: b"iic000",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Absolute,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                    data_type: ImageFrame,
                },

                #[doc = "Bottom Left peripheral image camera input. Incremental change."]
                ImageCameraBottomLeftIncremental => {
                    friendly_name: "Bottom Left Image Camera Input (Incremental Change)",
                    snake_case_identifier: "image_camera_bottom_left_incremental",
                    base_ascii: b"Iic000",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Incremental,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                    data_type: ImageFrame,
                },

                #[doc = "Bottom Middle peripheral image camera input. Absolute (instant) change."]
                ImageCameraBottomMiddleAbsolute => {
                    friendly_name: "Bottom Middle Image Camera Input (Absolute Change)",
                    snake_case_identifier: "image_camera_bottom_middle_absolute",
                    base_ascii: b"iic100",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Absolute,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                    data_type: ImageFrame,
                },

                #[doc = "Bottom Middle peripheral image camera input. Incremental change."]
                ImageCameraBottomMiddleIncremental => {
                    friendly_name: "Bottom Middle Image Camera Input (Incremental Change)",
                    snake_case_identifier: "image_camera_bottom_middle_incremental",
                    base_ascii: b"Iic100",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Incremental,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                    data_type: ImageFrame,
                },

                #[doc = "Bottom Right peripheral image camera input. Absolute (instant) change."]
                ImageCameraBottomRightAbsolute => {
                    friendly_name: "Bottom Right Image Camera Input (Absolute Change)",
                    snake_case_identifier: "image_camera_bottom_right_absolute",
                    base_ascii: b"iic200",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Absolute,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                    data_type: ImageFrame,
                },

                #[doc = "Bottom Right peripheral image camera input. Incremental change."]
                ImageCameraBottomRightIncremental => {
                    friendly_name: "Bottom Right Image Camera Input (Incremental Change)",
                    snake_case_identifier: "image_camera_bottom_right_incremental",
                    base_ascii: b"Iic200",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Incremental,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                    data_type: ImageFrame,
                }
                //endregion
            }
        }
    };
}