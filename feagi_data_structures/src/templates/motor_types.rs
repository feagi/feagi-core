

/// Definition File for Motoers (Input Processing Units)
#[macro_export]
macro_rules! motor_definition {
    ($callback:ident) => {
        $callback! {
            MotorCorticalType {

                //region Percentage


                //endregion

                //region Percentage4D

                #[doc = "Controls size and positioning of the central vision in a segmented frame."]
                Gaze => {
                    friendly_name: "Gaze",
                    snake_case_identifier: "gaze",
                    base_ascii: b"ogaz00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(4..5, 1..2, 1..33),
                    default_coder_type: TODO,
                    wrapped_data_type: WrappedIOType::Percentage4D,
                },

                //endregion

                //region SignedPercentage

                #[doc = "Free spinning motor."]
                RotaryMotor => {
                    friendly_name: "Rotary Motor",
                    snake_case_identifier: "rotary_motor",
                    base_ascii: b"omot00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: TODO,
                    wrapped_data_type: WrappedIOType::SignedPercentage,
                },

                #[doc = "Servo with max / min rotation distances"]
                PositionalServo => {
                    friendly_name: "Positional Servo",
                    snake_case_identifier: "positional_servo",
                    base_ascii: b"opse00",
                    channel_dimension_range: CorticalChannelDimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: TODO,
                    wrapped_data_type: WrappedIOType::SignedPercentage,
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