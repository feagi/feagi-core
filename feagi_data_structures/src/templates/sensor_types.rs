/// Definition File for Sensors (Input Processing Units)
#[macro_export]
macro_rules! sensor_definition {
    ($callback:ident) => {
        $callback! {
            SensorCorticalType {

                //region Percentage

                #[doc = "Infrared distance sensor for object detection. Absolute (instant) change. Neurons encoded linearly"]
                InfraredAbsoluteLinear => {
                    friendly_name: "Infrared Sensor (Absolute Change, Linear Encoding)",
                    snake_case_identifier: "infrared_absolute_linear",
                    base_ascii: b"iinf00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..2),
                    default_coder_type: Percentage_Absolute_Linear,
                    wrapped_data_type:WrappedIOType:: Percentage,
                },

                #[doc = "Infrared distance sensor for object detection. Absolute (instant) change. Neurons encoded fractionally exponentially"]
                InfraredAbsoluteFractional => {
                    friendly_name: "Infrared Sensor (Absolute Change, Linear Encoding)",
                    snake_case_identifier: "infrared_absolute_fractional",
                    base_ascii: b"iINF00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..2),
                    default_coder_type: Percentage_Absolute_Fractional,
                    wrapped_data_type:WrappedIOType:: Percentage,
                },

                #[doc = "Infrared distance sensor for object detection. Incremental change. Neurons encoded linearly"]
                InfraredIncrementalLinear => {
                    friendly_name: "Infrared Sensor (Absolute Change, Linear Encoding)",
                    snake_case_identifier: "infrared_incremental_linear",
                    base_ascii: b"Iinf00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..2),
                    default_coder_type: Percentage_Incremental_Linear,
                    wrapped_data_type:WrappedIOType:: Percentage,
                },

                #[doc = "Infrared distance sensor for object detection. Incremental change. Neurons encoded fractionally exponentially"]
                InfraredIncrementalFractional => {
                    friendly_name: "Infrared Sensor (Absolute Change, Linear Encoding)",
                    snake_case_identifier: "infrared_incremental_fractional",
                    base_ascii: b"IINF00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..2),
                    default_coder_type: Percentage_Incremental_Fractional,
                    wrapped_data_type:WrappedIOType:: Percentage,
                },



                #[doc = "Inverted infrared sensor that provides reverse object detection readings. Absolute (instant) change. Neurons encoded linearly"]
                ReverseInfraredAbsoluteLinear => {
                    friendly_name: "Infrared (Inverted) Sensor (Absolute Change, Linear Encoding)",
                    snake_case_identifier: "infrared_inverted_absolute_linear",
                    base_ascii: b"iiif00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..2),
                    default_coder_type: Percentage_Absolute_Linear,
                    wrapped_data_type: WrappedIOType::Percentage,
                },

                #[doc = "Inverted infrared sensor that provides reverse object detection readings. Absolute (instant) change. Neurons encoded fractionally exponentially"]
                ReverseInfraredAbsoluteFractional => {
                    friendly_name: "Infrared (Inverted) Sensor (Absolute Change, Fractional Encoding)",
                    snake_case_identifier: "infrared_inverted_absolute_fractional",
                    base_ascii: b"iIIF00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..2),
                    default_coder_type: Percentage_Absolute_Fractional,
                    wrapped_data_type: WrappedIOType::Percentage,
                },

                #[doc = "Inverted infrared sensor that provides reverse object detection readings. Incremental change. Neurons encoded linearly"]
                ReverseInfraredIncrementalLinear => {
                    friendly_name: "Infrared (Inverted) Sensor (Incremental Change, Linear Encoding)",
                    snake_case_identifier: "infrared_inverted_incremental_linear",
                    base_ascii: b"Iiif00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..2),
                    default_coder_type: Percentage_Incremental_Linear,
                    wrapped_data_type: WrappedIOType::Percentage,
                },

                #[doc = "Inverted infrared sensor that provides reverse object detection readings. Incremental change. Neurons encoded fractionally exponentially"]
                ReverseInfraredIncrementalFractional => {
                    friendly_name: "Infrared (Inverted) Sensor (Incremental Change, Fractional Encoding)",
                    snake_case_identifier: "infrared_inverted_incremental_fractional",
                    base_ascii: b"IIIF00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..2),
                    default_coder_type: Percentage_Incremental_Fractional,
                    wrapped_data_type: WrappedIOType::Percentage,
                },



                #[doc = "Digital GPIO input pin for reading binary signals (high/low states). Absolute (instant) change. Neurons encoded linearly"]
                DigitalGPIOInputAbsoluteLinear => {
                    friendly_name: "GPIO Digital Input (Absolute Change, Linear Encoding)",
                    snake_case_identifier: "gpio_digital_absolute_linear",
                    base_ascii: b"idgp00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..2),
                    default_coder_type: Percentage_Absolute_Linear,
                    wrapped_data_type: WrappedIOType::Percentage,
                },

                #[doc = "Digital GPIO input pin for reading binary signals (high/low states). Absolute (instant) change. Neurons encoded fractionally exponentially"]
                DigitalGPIOInputAbsoluteFractional => {
                    friendly_name: "GPIO Digital Input (Absolute Change, Fractional Encoding)",
                    snake_case_identifier: "gpio_digital_absolute_fractional",
                    base_ascii: b"iDGP00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..2),
                    default_coder_type: Percentage_Absolute_Fractional,
                    wrapped_data_type: WrappedIOType::Percentage,
                },

                #[doc = "Digital GPIO input pin for reading binary signals (high/low states). Incremental change. Neurons encoded linearly"]
                DigitalGPIOInputIncrementalLinear => {
                    friendly_name: "GPIO Digital Input (Incremental Change, Linear Encoding)",
                    snake_case_identifier: "gpio_digital_incremental_linear",
                    base_ascii: b"Idgp00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..2),
                    default_coder_type: Percentage_Incremental_Linear,
                    wrapped_data_type: WrappedIOType::Percentage,
                },

                #[doc = "Digital GPIO input pin for reading binary signals (high/low states). Incremental change. Neurons encoded fractionally exponentially"]
                DigitalGPIOInputIncrementalFractional => {
                    friendly_name: "GPIO Digital Input (Incremental Change, Fractional Encoding)",
                    snake_case_identifier: "gpio_digital_incremental_fractional",
                    base_ascii: b"IDGP00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..2),
                    default_coder_type: Percentage_Incremental_Fractional,
                    wrapped_data_type: WrappedIOType::Percentage,
                },



                #[doc = "Proximity sensor for detecting nearby objects and measuring distances. Absolute (instant) change. Neurons encoded linearly"]
                ProximityAbsoluteLinear => {
                    friendly_name: "Proximity (Absolute Change, Linear Encoding)",
                    snake_case_identifier: "proximity_absolute_linear",
                    base_ascii: b"ipro00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: Percentage_Absolute_Linear,
                    wrapped_data_type: WrappedIOType::Percentage,
                },

                #[doc = "Proximity sensor for detecting nearby objects and measuring distances. Absolute (instant) change. Neurons encoded fractionally exponentially"]
                ProximityAbsoluteFractional => {
                    friendly_name: "Proximity (Absolute Change, Fractional Encoding)",
                    snake_case_identifier: "proximity_absolute_fractional",
                    base_ascii: b"iPRO00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..33),
                    default_coder_type: Percentage_Absolute_Fractional,
                    wrapped_data_type: WrappedIOType::Percentage,
                },

                #[doc = "Proximity sensor for detecting nearby objects and measuring distances. Incremental change. Neurons encoded linearly"]
                ProximityIncrementalLinear => {
                    friendly_name: "Proximity (Incremental Change, Linear Encoding)",
                    snake_case_identifier: "proximity_incremental_linear",
                    base_ascii: b"Ipro00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: Percentage_Incremental_Linear,
                    wrapped_data_type: WrappedIOType::Percentage,
                },

                #[doc = "Proximity sensor for detecting nearby objects and measuring distances. Incremental change. Neurons encoded fractionally exponentially"]
                ProximityIncrementalFractional => {
                    friendly_name: "Proximity (Incremental Change, Fractional Encoding)",
                    snake_case_identifier: "proximity_incremental_fractional",
                    base_ascii: b"IPRO00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..33),
                    default_coder_type: Percentage_Incremental_Fractional,
                    wrapped_data_type: WrappedIOType::Percentage,
                },



                #[doc = "Shock sensor for sensing 'pain'. Absolute (instant) change. Neurons encoded linearly"]
                ShockAbsoluteLinear => {
                    friendly_name: "Shock (Absolute Change, Linear Encoding)",
                    snake_case_identifier: "shock_absolute_linear",
                    base_ascii: b"ishk00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..2),
                    default_coder_type: Percentage_Absolute_Linear,
                    wrapped_data_type: WrappedIOType::Percentage,
                },

                #[doc = "Shock sensor for sensing 'pain'. Absolute (instant) change. Neurons encoded fractionally exponentially"]
                ShockAbsoluteFractional => {
                    friendly_name: "Shock (Absolute Change, Fractional Encoding)",
                    snake_case_identifier: "shock_absolute_fractional",
                    base_ascii: b"iSHK00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..2),
                    default_coder_type: Percentage_Absolute_Fractional,
                    wrapped_data_type: WrappedIOType::Percentage,
                },

                #[doc = "Shock sensor for sensing 'pain'. Incremental change. Neurons encoded linearly"]
                ShockIncrementalLinear => {
                    friendly_name: "Shock (Incremental Change, Linear Encoding)",
                    snake_case_identifier: "shock_incremental_linear",
                    base_ascii: b"Ishk00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..2),
                    default_coder_type: Percentage_Incremental_Linear,
                    wrapped_data_type: WrappedIOType::Percentage,
                },

                #[doc = "Shock sensor for sensing 'pain'. Incremental change. Neurons encoded fractionally exponentially"]
                ShockIncrementalFractional => {
                    friendly_name: "Shock (Incremental Change, Fractional Encoding)",
                    snake_case_identifier: "shock_incremental_fractional",
                    base_ascii: b"ISHK00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..2),
                    default_coder_type: Percentage_Incremental_Fractional,
                    wrapped_data_type: WrappedIOType::Percentage,
                },



                #[doc = "Battery level sensor for monitoring power remaining. Absolute (instant) change. Neurons encoded linearly"]
                BatteryAbsoluteLinear => {
                    friendly_name: "Battery Gauge (Absolute Change, Linear Encoding)",
                    snake_case_identifier: "battery_gauge_absolute_linear",
                    base_ascii: b"ibat00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: Percentage_Absolute_Linear,
                    wrapped_data_type: WrappedIOType::Percentage,
                },

                #[doc = "Battery level sensor for monitoring power remaining. Absolute (instant) change. Neurons encoded fractionally exponentially"]
                BatteryAbsoluteFractional => {
                    friendly_name: "Battery Gauge (Absolute Change, Fractional Encoding)",
                    snake_case_identifier: "battery_gauge_absolute_fractional",
                    base_ascii: b"iBAT00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..33),
                    default_coder_type: Percentage_Absolute_Fractional,
                    wrapped_data_type: WrappedIOType::Percentage,
                },

                #[doc = "Battery level sensor for monitoring power remaining. Incremental change. Neurons encoded linearly"]
                BatteryIncrementalLinear => {
                    friendly_name: "Battery Gauge (Incremental Change, Linear Encoding)",
                    snake_case_identifier: "battery_gauge_incremental_linear",
                    base_ascii: b"Ibat00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: Percentage_Incremental_Linear,
                    wrapped_data_type: WrappedIOType::Percentage,
                },

                #[doc = "Battery level sensor for monitoring power remaining. Incremental change. Neurons encoded fractionally exponentially"]
                BatteryIncrementalFractional => {
                    friendly_name: "Battery Gauge (Incremental Change, Fractional Encoding)",
                    snake_case_identifier: "battery_gauge_incremental_fractional",
                    base_ascii: b"IBAT00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..33),
                    default_coder_type: Percentage_Incremental_Fractional,
                    wrapped_data_type: WrappedIOType::Percentage,
                },



                #[doc = "Analog GPIO input pin for reading binary signals (high/low states). Absolute (instant) change. Neurons encoded linearly"]
                AnalogGPIOInputAbsoluteLinear => {
                    friendly_name: "GPIO Analog Input (Absolute Change, Linear Encoding)",
                    snake_case_identifier: "gpio_analog_absolute_linear",
                    base_ascii: b"iagp00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: Percentage_Absolute_Linear,
                    wrapped_data_type: WrappedIOType::Percentage,
                },

                #[doc = "Analog GPIO input pin for reading binary signals (high/low states). Absolute (instant) change. Neurons encoded fractionally exponentially"]
                AnalogGPIOInputAbsoluteFractional => {
                    friendly_name: "GPIO Analog Input (Absolute Change, Fractional Encoding)",
                    snake_case_identifier: "gpio_analog_absolute_fractional",
                    base_ascii: b"iAGP00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..33),
                    default_coder_type: Percentage_Absolute_Fractional,
                    wrapped_data_type: WrappedIOType::Percentage,
                },

                #[doc = "Analog GPIO input pin for reading binary signals (high/low states). Incremental change. Neurons encoded linearly"]
                AnalogGPIOInputIncrementalLinear => {
                    friendly_name: "GPIO Analog Input (Incremental Change, Linear Encoding)",
                    snake_case_identifier: "gpio_analog_incremental_linear",
                    base_ascii: b"Iagp00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: Percentage_Incremental_Linear,
                    wrapped_data_type: WrappedIOType::Percentage,
                },

                #[doc = "Analog GPIO input pin for reading binary signals (high/low states). Incremental change. Neurons encoded fractionally exponentially"]
                AnalogGPIOInputIncrementalFractional => {
                    friendly_name: "GPIO Analog Input (Incremental Change, Fractional Encoding)",
                    snake_case_identifier: "gpio_analog_incremental_fractional",
                    base_ascii: b"IAGP00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..33),
                    default_coder_type: Percentage_Incremental_Fractional,
                    wrapped_data_type: WrappedIOType::Percentage,
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
                },

                #[doc = "Servo position feedback sensor for monitoring actuator positions. Absolute (instant) change. Neurons encoded fractionally exponentially"]
                ServoPositionAbsoluteFractional => {
                    friendly_name: "Servo Position (Absolute Change, Fractional Encoding)",
                    snake_case_identifier: "servo_position_absolute_fractional",
                    base_ascii: b"iSVP00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..33),
                    default_coder_type: SignedPercentage_Absolute_Fractional,
                    wrapped_data_type: WrappedIOType::SignedPercentage,
                },

                #[doc = "Servo position feedback sensor for monitoring actuator positions. Incremental change. Neurons encoded linearly"]
                ServoPositionIncrementalLinear => {
                    friendly_name: "Servo Position (Incremental Change, Linear Encoding)",
                    snake_case_identifier: "servo_position_incremental_linear",
                    base_ascii: b"Isvp00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: SignedPercentage_Incremental_Linear,
                    wrapped_data_type: WrappedIOType::SignedPercentage,
                },

                #[doc = "Servo position feedback sensor for monitoring actuator positions. Incremental change. Neurons encoded fractionally exponentially"]
                ServoPositionIncrementalFractional => {
                    friendly_name: "Servo Position (Incremental Change, Fractional Encoding)",
                    snake_case_identifier: "servo_position_incremental_fractional",
                    base_ascii: b"ISVP00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..33),
                    default_coder_type: SignedPercentage_Incremental_Fractional,
                    wrapped_data_type: WrappedIOType::SignedPercentage,
                },




                #[doc = "Servo motion feedback sensor for monitoring actuator positions. Absolute (instant) change. Neurons encoded linearly"]
                ServoMotionAbsoluteLinear => {
                    friendly_name: "Servo Motion (Absolute Change, Linear Encoding)",
                    snake_case_identifier: "servo_motion_absolute_linear",
                    base_ascii: b"isvm00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: SignedPercentage_Absolute_Linear,
                    wrapped_data_type: WrappedIOType::SignedPercentage,
                },

                #[doc = "Servo motion feedback sensor for monitoring actuator positions. Absolute (instant) change. Neurons encoded fractionally exponentially"]
                ServoMotionAbsoluteFractional => {
                    friendly_name: "Servo Motion (Absolute Change, Fractional Encoding)",
                    snake_case_identifier: "servo_motion_absolute_fractional",
                    base_ascii: b"iSVM00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..33),
                    default_coder_type: SignedPercentage_Absolute_Fractional,
                    wrapped_data_type: WrappedIOType::SignedPercentage,
                },

                #[doc = "Servo motion feedback sensor for monitoring actuator positions. Incremental change. Neurons encoded linearly"]
                ServoMotionIncrementalLinear => {
                    friendly_name: "Servo Motion (Incremental Change, Linear Encoding)",
                    snake_case_identifier: "servo_motion_incremental_linear",
                    base_ascii: b"Isvm00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: SignedPercentage_Incremental_Linear,
                    wrapped_data_type: WrappedIOType::SignedPercentage,
                },

                #[doc = "Servo motion feedback sensor for monitoring actuator positions. Incremental change. Neurons encoded fractionally exponentially"]
                ServoMotionIncrementalFractional => {
                    friendly_name: "Servo Motion (Incremental Change, Fractional Encoding)",
                    snake_case_identifier: "servo_motion_incremental_fractional",
                    base_ascii: b"ISVM00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..33),
                    default_coder_type: SignedPercentage_Incremental_Fractional,
                    wrapped_data_type: WrappedIOType::SignedPercentage,
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
                },

                #[doc = "Miscellaneous area for all types of data FEAGI has no specific implementation for. Incremental change."]
                MiscellaneousIncremental => {
                    friendly_name: "Miscellaneous (incremental_change)",
                    snake_case_identifier: "miscellaneous_incremental",
                    base_ascii: b"imis00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..u32::MAX),
                    default_coder_type: MiscData_Incremental,
                    wrapped_data_type: WrappedIOType::MiscData(None),
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
                },

                #[doc = "Image camera input. Either alone or in the center of segmented/peripheral image camera setups. Incremental change."]
                ImageCameraCenterIncremental => {
                    friendly_name: "Center Image Camera Input (Incremental Change)",
                    snake_case_identifier: "image_camera_center_incremental",
                    base_ascii: b"Iic400",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Incremental,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                },
                #[doc = "Top Left peripheral image camera input. Absolute (instant) change."]
                ImageCameraTopLeftAbsolute => {
                    friendly_name: "Top Left Image Camera Input (Absolute Change)",
                    snake_case_identifier: "image_camera_top_left_absolute",
                    base_ascii: b"iic600",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Absolute,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                },

                #[doc = "Top Left peripheral image camera input. Incremental change."]
                ImageCameraTopLeftIncremental => {
                    friendly_name: "Top Left Image Camera Input (Incremental Change)",
                    snake_case_identifier: "image_camera_top_left_incremental",
                    base_ascii: b"Iic600",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Incremental,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                },

                #[doc = "Top Middle peripheral image camera input. Absolute (instant) change."]
                ImageCameraTopMiddleAbsolute => {
                    friendly_name: "Top Middle Image Camera Input (Absolute Change)",
                    snake_case_identifier: "image_camera_top_middle_absolute",
                    base_ascii: b"iic700",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Absolute,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                },

                #[doc = "Top Middle peripheral image camera input. Incremental change."]
                ImageCameraTopMiddleIncremental => {
                    friendly_name: "Top Middle Image Camera Input (Incremental Change)",
                    snake_case_identifier: "image_camera_top_middle_incremental",
                    base_ascii: b"Iic700",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Incremental,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                },
                #[doc = "Top Right peripheral image camera input. Absolute (instant) change."]
                ImageCameraTopRightAbsolute => {
                    friendly_name: "Top Right Image Camera Input (Absolute Change)",
                    snake_case_identifier: "image_camera_top_right_absolute",
                    base_ascii: b"iic800",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Absolute,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                },

                #[doc = "Top Right peripheral image camera input. Incremental change."]
                ImageCameraTopRightIncremental => {
                    friendly_name: "Top Right Image Camera Input (Incremental Change)",
                    snake_case_identifier: "image_camera_top_right_incremental",
                    base_ascii: b"Iic800",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Incremental,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                },

                #[doc = "Middle Left peripheral image camera input. Absolute (instant) change."]
                ImageCameraMiddleLeftAbsolute => {
                    friendly_name: "Middle Left Image Camera Input (Absolute Change)",
                    snake_case_identifier: "image_camera_middle_left_absolute",
                    base_ascii: b"iic300",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Absolute,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                },

                #[doc = "Middle Left peripheral image camera input. Incremental change."]
                ImageCameraMiddleLeftIncremental => {
                    friendly_name: "Middle Left Image Camera Input (Incremental Change)",
                    snake_case_identifier: "image_camera_middle_left_incremental",
                    base_ascii: b"Iic300",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Incremental,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                },

                #[doc = "Middle Right peripheral image camera input. Absolute (instant) change."]
                ImageCameraMiddleRightAbsolute => {
                    friendly_name: "Middle Right Image Camera Input (Absolute Change)",
                    snake_case_identifier: "image_camera_middle_right_absolute",
                    base_ascii: b"iic500",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Absolute,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                },

                #[doc = "Middle Right peripheral image camera input. Incremental change."]
                ImageCameraMiddleRightIncremental => {
                    friendly_name: "Middle Right Image Camera Input (Incremental Change)",
                    snake_case_identifier: "image_camera_middle_right_incremental",
                    base_ascii: b"Iic500",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Incremental,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                },

                #[doc = "Bottom Left peripheral image camera input. Absolute (instant) change."]
                ImageCameraBottomLeftAbsolute => {
                    friendly_name: "Bottom Left Image Camera Input (Absolute Change)",
                    snake_case_identifier: "image_camera_bottom_left_absolute",
                    base_ascii: b"iic000",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Absolute,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                },

                #[doc = "Bottom Left peripheral image camera input. Incremental change."]
                ImageCameraBottomLeftIncremental => {
                    friendly_name: "Bottom Left Image Camera Input (Incremental Change)",
                    snake_case_identifier: "image_camera_bottom_left_incremental",
                    base_ascii: b"Iic000",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Incremental,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                },

                #[doc = "Bottom Middle peripheral image camera input. Absolute (instant) change."]
                ImageCameraBottomMiddleAbsolute => {
                    friendly_name: "Bottom Middle Image Camera Input (Absolute Change)",
                    snake_case_identifier: "image_camera_bottom_middle_absolute",
                    base_ascii: b"iic100",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Absolute,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                },

                #[doc = "Bottom Middle peripheral image camera input. Incremental change."]
                ImageCameraBottomMiddleIncremental => {
                    friendly_name: "Bottom Middle Image Camera Input (Incremental Change)",
                    snake_case_identifier: "image_camera_bottom_middle_incremental",
                    base_ascii: b"Iic100",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Incremental,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                },

                #[doc = "Bottom Right peripheral image camera input. Absolute (instant) change."]
                ImageCameraBottomRightAbsolute => {
                    friendly_name: "Bottom Right Image Camera Input (Absolute Change)",
                    snake_case_identifier: "image_camera_bottom_right_absolute",
                    base_ascii: b"iic200",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Absolute,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                },

                #[doc = "Bottom Right peripheral image camera input. Incremental change."]
                ImageCameraBottomRightIncremental => {
                    friendly_name: "Bottom Right Image Camera Input (Incremental Change)",
                    snake_case_identifier: "image_camera_bottom_right_incremental",
                    base_ascii: b"Iic200",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..5),
                    default_coder_type: ImageFrame_Incremental,
                    wrapped_data_type: WrappedIOType::ImageFrame(None),
                }
                //endregion
            }
        }
    };
}