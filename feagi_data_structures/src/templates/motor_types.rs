// Note: possible coder types:
// F32Normalized0To1_Linear, F32NormalizedM1To1_SplitSignDivided, ImageFrame, None

/// Definition File for Motoers (Input Processing Units)
#[macro_export]
macro_rules! motor_definition {
    ($callback:ident) => {
        $callback! {
            MotorCorticalType {

                //region 0 - 1 Linear Float


                //endregion

                //region -1 -1 Split Sign Float
                #[doc = "Free spinning motor."]
                RotaryMotor => {
                    friendly_name: "Rotary Motor",
                    snake_case_identifier: "rotary_motor",
                    base_ascii: b"omot00",
                    channel_dimension_range: DimensionRange::new(1..2, 1..2, 1..u32::MAX),
                    default_coder_type: F32NormalizedM1To1_SplitSignDivided,
                    wrapped_data_type: WrappedIOType::F32Normalized0To1,
                },

                //endregion

                //region ImageFrame

            }
        }
    };
}