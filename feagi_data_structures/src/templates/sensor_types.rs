// Note: possible coder types:
// F32Normalized0To1_Linear, F32NormalizedM1To1_SplitSignDivided, ImageFrame, None



/// Definition File for Sensors (Input Processing Units)
#[macro_export]
macro_rules! sensor_definition {
    ($callback:ident) => {
        $callback! {
            SensorCorticalType {

                //region 0 - 1 Linear Float

                #[doc = "Infrared distance sensor for object detection"]
                Infrared => {
                    friendly_name: "Infrared Sensor",
                    snake_case_identifier: "infrared",
                    base_ascii: b"iinf00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..2),
                    default_coder_type: F32Normalized0To1_Linear,
                    wrapped_data_type:WrappedIOType:: F32Normalized0To1,
                },
                #[doc = "Inverted infrared sensor that provides reverse object detection readings."]
                ReverseInfrared => {
                    friendly_name: "Infrared (Inverted) Sensor",
                    snake_case_identifier: "infrared_inverted",
                    base_ascii: b"iiif00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..2),
                    default_coder_type: F32Normalized0To1_Linear,
                    wrapped_data_type: WrappedIOType::F32Normalized0To1,
                },
                #[doc = "Digital GPIO input pin for reading binary signals (high/low states)."]
                DigitalGPIOInput => {
                    friendly_name: "GPIO Digital Input",
                    snake_case_identifier: "gpio_digital",
                    base_ascii: b"idgp00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..2),
                    default_coder_type: F32Normalized0To1_Linear,
                    wrapped_data_type: WrappedIOType::F32Normalized0To1,
                },
                #[doc = "Proximity sensor for detecting nearby objects and measuring distances."]
                Proximity => {
                    friendly_name: "Proximity",
                    snake_case_identifier: "proximity",
                    base_ascii: b"ipro00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: F32Normalized0To1_Linear,
                    wrapped_data_type: WrappedIOType::F32Normalized0To1,
                },
                #[doc = "Shock sensor for sensing 'pain'"]
                Shock => {
                    friendly_name: "Shock",
                    snake_case_identifier: "shock",
                    base_ascii: b"ishk00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..2),
                    default_coder_type: F32Normalized0To1_Linear,
                    wrapped_data_type: WrappedIOType::F32Normalized0To1,
                },
                #[doc = "Battery level sensor for monitoring power remaining."]
                Battery => {
                    friendly_name: "Battery Gauge",
                    snake_case_identifier: "battery_gauge",
                    base_ascii: b"ibat00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: F32Normalized0To1_Linear,
                    wrapped_data_type: WrappedIOType::F32Normalized0To1,
                },

                //endregion

                //region -1 -1 Split Sign Float

                #[doc = "Servo position feedback sensor for monitoring actuator positions."]
                ServoPosition => {
                    friendly_name: "Servo Position",
                    snake_case_identifier: "servo_position",
                    base_ascii: b"isvp00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: F32NormalizedM1To1_SplitSignDivided,
                    wrapped_data_type: WrappedIOType::F32NormalizedM1To1,
                },

                //endregion

                //region ImageFrame

                #[doc = "Image camera input. Either alone or in the center of segmented/peripheral image camera setups"]
                ImageCameraCenter => {
                    friendly_name: "Center Image Camera Input",
                    snake_case_identifier: "image_camera_center",
                    base_ascii: b"iic400",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                },
                #[doc = "Top Left peripheral image camera input."]
                ImageCameraTopLeft => {
                    friendly_name: "Top Left Image Camera Input",
                    snake_case_identifier: "image_camera_top_left",
                    base_ascii: b"iic600",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: None,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                },
                #[doc = "Top Middle peripheral image camera input."]
                ImageCameraTopMiddle => {
                    friendly_name: "Top Middle Image Camera Input",
                    snake_case_identifier: "image_camera_top_middle",
                    base_ascii: b"iic700",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: None,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                },
                #[doc = "Top Right peripheral image camera input."]
                ImageCameraTopRight => {
                    friendly_name: "Top Right Image Camera Input",
                    snake_case_identifier: "image_camera_top_right",
                    base_ascii: b"iic800",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: None,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                },
                #[doc = "Middle Left peripheral image camera input."]
                ImageCameraMiddleLeft => {
                    friendly_name: "Middle Left Image Camera Input",
                    snake_case_identifier: "image_camera_middle_left",
                    base_ascii: b"iic300",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: None,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                },
                #[doc = "Middle Right peripheral image camera input."]
                ImageCameraMiddleRight => {
                    friendly_name: "Middle Right Image Camera Input",
                    snake_case_identifier: "image_camera_middle_right",
                    base_ascii: b"iic500",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: None,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                },
                #[doc = "Bottom Left peripheral image camera input."]
                ImageCameraBottomLeft => {
                    friendly_name: "Bottom Left Image Camera Input",
                    snake_case_identifier: "image_camera_bottom_left",
                    base_ascii: b"iic000",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: None,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                },
                #[doc = "Bottom Middle peripheral image camera input."]
                ImageCameraBottomMiddle => {
                    friendly_name: "Bottom Middle Image Camera Input",
                    snake_case_identifier: "image_camera_bottom_middle",
                    base_ascii: b"iic100",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: None,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                },
                #[doc = "Bottom Right peripheral image camera input."]
                ImageCameraBottomRight => {
                    friendly_name: "Bottom Right Image Camera Input",
                    snake_case_identifier: "image_camera_bottom_right",
                    base_ascii: b"iic200",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: None,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                }
                //endregion
            }
        }
    };
}