

/// Definition File for Motoers (Input Processing Units)
#[macro_export]
macro_rules! motor_definition {
    ($callback:ident) => {
        $callback! {
            MotorCorticalType {

                //region Percentage


                //endregion


                //region SignedPercentage

                #[doc = "Free spinning motor. Instantaneous change. Neurons encoded linearly"]
                RotaryMotorInstantLinear => {
                    friendly_name: "Rotary Motor (Instant Change, Linear Encoding)",
                    snake_case_identifier: "rotary_motor_instant_linear",
                    base_ascii: b"omot00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: SignedPercentage_Instant_Linear,
                    wrapped_data_type: WrappedIOType::SignedPercentage,
                },

                #[doc = "Free spinning motor. Instantaneous change. Neurons encoded fractionally exponentially"]
                RotaryMotorInstantFractional => {
                    friendly_name: "Rotary Motor (Instant Change, Fractional Encoding)",
                    snake_case_identifier: "rotary_motor_instant_fractional",
                    base_ascii: b"oMOT00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..33),
                    default_coder_type: SignedPercentage_Instant_Fractional,
                    wrapped_data_type: WrappedIOType::SignedPercentage,
                },

                #[doc = "Free spinning motor. Incremental change. Neurons encoded linearly"]
                RotaryMotorIncrementalLinear => {
                    friendly_name: "Rotary Motor (Incremental Change, Linear Encoding)",
                    snake_case_identifier: "rotary_motor_incremental_linear",
                    base_ascii: b"Omot00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: SignedPercentage_Incremental_Linear,
                    wrapped_data_type: WrappedIOType::SignedPercentage,
                },

                #[doc = "Free spinning motor. Incremental change. Neurons encoded fractionally exponentially"]
                RotaryMotorIncrementalFractional => {
                    friendly_name: "Rotary Motor (Incremental Change, Fractional Encoding)",
                    snake_case_identifier: "rotary_motor_incremental_fractional",
                    base_ascii: b"OMOT00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..33),
                    default_coder_type: SignedPercentage_Incremental_Fractional,
                    wrapped_data_type: WrappedIOType::SignedPercentage,
                },

                #[doc = "Servo with max / min rotation distances. Instantaneous change. Neurons encoded linearly"]
                PositionalServoInstantLinear => {
                    friendly_name: "Positional Servo (Instant Change, Linear Encoding)",
                    snake_case_identifier: "positional_servo_instant_linear",
                    base_ascii: b"opse00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: SignedPercentage_Instant_Linear,
                    wrapped_data_type: WrappedIOType::SignedPercentage,
                },

                #[doc = "Servo with max / min rotation distances. Instantaneous change. Neurons encoded fractionally exponentially"]
                PositionalServoInstantFractional => {
                    friendly_name: "Positional Servo (Instant Change, Fractional Encoding)",
                    snake_case_identifier: "positional_servo_instant_fractional",
                    base_ascii: b"oPSE00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..33),
                    default_coder_type: SignedPercentage_Instant_Fractional,
                    wrapped_data_type: WrappedIOType::SignedPercentage,
                },

                #[doc = "Servo with max / min rotation distances. Incremental change. Neurons encoded linearly"]
                PositionalServoIncrementalLinear => {
                    friendly_name: "Positional Servo (Incremental Change, Linear Encoding)",
                    snake_case_identifier: "positional_servo_incremental_linear",
                    base_ascii: b"Opse00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: SignedPercentage_Incremental_Linear,
                    wrapped_data_type: WrappedIOType::SignedPercentage,
                },

                #[doc = "Servo with max / min rotation distances. Incremental change. Neurons encoded fractionally exponentially"]
                PositionalServoIncrementalFractional => {
                    friendly_name: "Positional Servo (Incremental Change, Fractional Encoding)",
                    snake_case_identifier: "positional_servo_incremental_fractional",
                    base_ascii: b"OPSE00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..33),
                    default_coder_type: SignedPercentage_Incremental_Fractional,
                    wrapped_data_type: WrappedIOType::SignedPercentage,
                },

                //endregion

                //region Percentage4D

                #[doc = "Controls size and positioning of the central vision in a segmented frame. Instantaneous change. Neurons encoded linearly"]
                GazeInstantLinear => {
                    friendly_name: "Gaze (Instant Change, Linear Encoding)",
                    snake_case_identifier: "gaze_instant_linear",
                    base_ascii: b"ogaz00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(4..5, 1..2, 1..u32::MAX),
                    default_coder_type: Percentage4D_Instant_Linear,
                    wrapped_data_type: WrappedIOType::Percentage_4D,
                },

                #[doc = "Controls size and positioning of the central vision in a segmented frame. Instantaneous change. Neurons encoded fractionally exponentially"]
                GazeInstantFractional => {
                    friendly_name: "Gaze (Instant Change, Fractional Encoding)",
                    snake_case_identifier: "gaze_instant_fractional",
                    base_ascii: b"oGAZ00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(4..5, 1..2, 1..33),
                    default_coder_type: Percentage4D_Instant_Fractional,
                    wrapped_data_type: WrappedIOType::Percentage_4D,
                },

                #[doc = "Controls size and positioning of the central vision in a segmented frame. Incremental change. Neurons encoded linearly"]
                GazeIncrementalLinear => {
                    friendly_name: "Gaze (Incremental Change, Linear Encoding)",
                    snake_case_identifier: "gaze_incremental_linear",
                    base_ascii: b"Ogaz00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(4..5, 1..2, 1..u32::MAX),
                    default_coder_type: Percentage4D_Incremental_Linear,
                    wrapped_data_type: WrappedIOType::Percentage_4D,
                },

                #[doc = "Controls size and positioning of the central vision in a segmented frame. Incremental change. Neurons encoded fractionally exponentially"]
                GazeIncrementalFractional => {
                    friendly_name: "Gaze (Incremental Change, Fractional Encoding)",
                    snake_case_identifier: "gaze_incremental_fractional",
                    base_ascii: b"OGAZ00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(4..5, 1..2, 1..33),
                    default_coder_type: Percentage4D_Incremental_Fractional,
                    wrapped_data_type: WrappedIOType::Percentage_4D,
                },

                //endregion

                //region MiscData

                #[doc = "Used for other stuff"]
                Miscellaneous => {
                    friendly_name: "Miscellaneous",
                    snake_case_identifier: "miscellaneous",
                    base_ascii: b"omis00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..u32::MAX),
                    default_coder_type: TODO,
                    wrapped_data_type: WrappedIOType::MiscData(None),
                },

                //endregion

            }
        }
    };
}