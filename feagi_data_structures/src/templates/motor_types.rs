

/// Definition File for Motors (Input Processing Units)
#[macro_export]
macro_rules! motor_definition {
    ($callback:ident) => {
        $callback! {
            MotorCorticalType {

                //region Percentage


                //endregion


                //region SignedPercentage

                #[doc = "Free spinning motor. Absolute (instant) change. Neurons encoded linearly"]
                RotaryMotorAbsoluteLinear => {
                    friendly_name: "Rotary Motor (Absolute Change, Linear Encoding)",
                    snake_case_identifier: "rotary_motor_absolute_linear",
                    base_ascii: b"omot00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: SignedPercentage_Absolute_Linear,
                    wrapped_data_type: WrappedIOType::SignedPercentage,
                    data_type: SignedPercentage,
                },

                #[doc = "Free spinning motor. Absolute (instant) change. Neurons encoded fractionally"]
                RotaryMotorAbsoluteFractional => {
                    friendly_name: "Rotary Motor (Absolute Change, Fractional Encoding)",
                    snake_case_identifier: "rotary_motor_absolute_fractional",
                    base_ascii: b"oMOT00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..33),
                    default_coder_type: SignedPercentage_Absolute_Fractional,
                    wrapped_data_type: WrappedIOType::SignedPercentage,
                    data_type: SignedPercentage,
                },

                #[doc = "Free spinning motor. Incremental change. Neurons encoded linearly"]
                RotaryMotorIncrementalLinear => {
                    friendly_name: "Rotary Motor (Incremental Change, Linear Encoding)",
                    snake_case_identifier: "rotary_motor_incremental_linear",
                    base_ascii: b"Omot00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: SignedPercentage_Incremental_Linear,
                    wrapped_data_type: WrappedIOType::SignedPercentage,
                    data_type: SignedPercentage,
                },

                #[doc = "Free spinning motor. Incremental change. Neurons encoded fractionally"]
                RotaryMotorIncrementalFractional => {
                    friendly_name: "Rotary Motor (Incremental Change, Fractional Encoding)",
                    snake_case_identifier: "rotary_motor_incremental_fractional",
                    base_ascii: b"OMOT00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..33),
                    default_coder_type: SignedPercentage_Incremental_Fractional,
                    wrapped_data_type: WrappedIOType::SignedPercentage,
                    data_type: SignedPercentage,
                },






                #[doc = "Servo with max / min rotation distances. Absolute (instant) change. Neurons encoded linearly"]
                PositionalServoAbsoluteLinear => {
                    friendly_name: "Positional Servo (Absolute Change, Linear Encoding)",
                    snake_case_identifier: "positional_servo_absolute_linear",
                    base_ascii: b"opse00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: SignedPercentage_Absolute_Linear,
                    wrapped_data_type: WrappedIOType::SignedPercentage,
                    data_type: SignedPercentage,
                },

                #[doc = "Servo with max / min rotation distances. Absolute (instant) change. Neurons encoded fractionally"]
                PositionalServoAbsoluteFractional => {
                    friendly_name: "Positional Servo (Absolute Change, Fractional Encoding)",
                    snake_case_identifier: "positional_servo_absolute_fractional",
                    base_ascii: b"oPSE00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..33),
                    default_coder_type: SignedPercentage_Absolute_Fractional,
                    wrapped_data_type: WrappedIOType::SignedPercentage,
                    data_type: SignedPercentage,
                },

                #[doc = "Servo with max / min rotation distances. Incremental change. Neurons encoded linearly"]
                PositionalServoIncrementalLinear => {
                    friendly_name: "Positional Servo (Incremental Change, Linear Encoding)",
                    snake_case_identifier: "positional_servo_incremental_linear",
                    base_ascii: b"Opse00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: SignedPercentage_Incremental_Linear,
                    wrapped_data_type: WrappedIOType::SignedPercentage,
                    data_type: SignedPercentage,
                },

                #[doc = "Servo with max / min rotation distances. Incremental change. Neurons encoded fractionally"]
                PositionalServoIncrementalFractional => {
                    friendly_name: "Positional Servo (Incremental Change, Fractional Encoding)",
                    snake_case_identifier: "positional_servo_incremental_fractional",
                    base_ascii: b"OPSE00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..33),
                    default_coder_type: SignedPercentage_Incremental_Fractional,
                    wrapped_data_type: WrappedIOType::SignedPercentage,
                    data_type: SignedPercentage,
                },

                //endregion

                //region Percentage4D

                #[doc = "Controls size and positioning of the central vision in a segmented frame. Absolute (instant) change. Neurons encoded linearly"]
                GazeAbsoluteLinear => {
                    friendly_name: "Gaze (Absolute Change, Linear Encoding)",
                    snake_case_identifier: "gaze_absolute_linear",
                    base_ascii: b"ogaz00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(4..5, 1..2, 1..u32::MAX),
                    default_coder_type: Percentage4D_Absolute_Linear,
                    wrapped_data_type: WrappedIOType::Percentage_4D,
                    data_type: SignedPercentage4D,
                },

                #[doc = "Controls size and positioning of the central vision in a segmented frame. Incremental change. Neurons encoded linearly"]
                GazeIncrementalLinear => {
                    friendly_name: "Gaze (Incremental Change, Linear Encoding)",
                    snake_case_identifier: "gaze_incremental_linear",
                    base_ascii: b"Ogaz00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(4..5, 1..2, 1..u32::MAX),
                    default_coder_type: Percentage4D_Incremental_Linear,
                    wrapped_data_type: WrappedIOType::Percentage_4D,
                    data_type: SignedPercentage4D,
                },

                //endregion

                //region MiscData

                /*
                #[doc = "Used for other stuff. Absolute (instant) change,"]
                MiscellaneousAbsolute => {
                    friendly_name: "Miscellaneous (Absolute)",
                    snake_case_identifier: "miscellaneous_absolute",
                    base_ascii: b"omis00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..u32::MAX),
                    default_coder_type: MiscData_Absolute,
                    wrapped_data_type: WrappedIOType::MiscData(None),
                    data_type: MiscData,
                },

                #[doc = "Used for other stuff. Incremental change,"]
                MiscellaneousIncremental => {
                    friendly_name: "Miscellaneous (Incremental Change)",
                    snake_case_identifier: "miscellaneous_incremental",
                    base_ascii: b"Omis00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..u32::MAX, 1..u32::MAX, 1..u32::MAX),
                    default_coder_type: MiscData_Incremental,
                    wrapped_data_type: WrappedIOType::MiscData(None),
                    data_type: MiscData,
                },

                 */

                //endregion

            }
        }
    };
}