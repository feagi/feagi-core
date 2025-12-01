#[macro_export]
macro_rules! motor_cortical_units {
    ($callback:ident) => {
        $callback! {
            MotorCorticalUnit {

                #[doc = "Free spinning motor."]
                RotaryMotor => {
                    friendly_name: "Rotary Motor",
                    snake_case_name: "rotary_motor",
                    accepted_wrapped_io_data_type: SignedPercentage, // This property determines what type of registration funciton will be generated
                    cortical_id_unit_reference: *b"mot",
                    number_cortical_areas: 1,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                        percentage_neuron_positioning: PercentageNeuronPositioning
                    },
                    cortical_area_types: {
                        (IOCorticalAreaDataType::SignedPercentage(frame_change_handling, percentage_neuron_positioning), 0)
                    },
                    unit_default_topology: {
                        0 => { relative_position: [0, 0, 0], dimensions: [1, 1, 10] }
                    }
                },

                #[doc = "Servo Position, defined by min / max distances"]
                PositionalServo => {
                    friendly_name: "Positional Servo",
                    snake_case_name: "positional_servo",
                    accepted_wrapped_io_data_type: SignedPercentage,
                    cortical_id_unit_reference: *b"pse",
                    number_cortical_areas: 1,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                        percentage_neuron_positioning: PercentageNeuronPositioning
                    },
                    cortical_area_types: {
                        (IOCorticalAreaDataType::SignedPercentage(frame_change_handling, percentage_neuron_positioning), 0)
                    },
                    unit_default_topology: {
                        0 => { relative_position: [0, 0, 0], dimensions: [1, 1, 10] }
                    }
                },

                #[doc = "Gaze control, where the first 2 numbers are the XY center, and the last number is the relative size"]
                Shock => {
                    friendly_name: "Gaze Control",
                    snake_case_name: "gaze_control",
                    accepted_wrapped_io_data_type: GazeProperties,
                    cortical_id_unit_reference: *b"gaz",
                    number_cortical_areas: 2,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                        percentage_neuron_positioning: PercentageNeuronPositioning
                    },
                    cortical_area_types: {
                        (IOCorticalAreaDataType::Percentage2D(frame_change_handling, percentage_neuron_positioning), 0),
                        (IOCorticalAreaDataType::Percentage(frame_change_handling, percentage_neuron_positioning), 1)
                    },
                    unit_default_topology: {
                        0 => { relative_position: [0, 0, 0], dimensions: [8, 8, 10] },
                        1 => { relative_position: [0, 0, -10], dimensions: [1, 1, 10] }
                    }
                },


                #[doc = "Miscellaneous motor that does not fit existing templates."]
                MiscData => {
                    friendly_name: "Miscellaneous Motor",
                    snake_case_name: "miscellaneous",
                    accepted_wrapped_io_data_type: MiscData,
                    cortical_id_unit_reference: *b"mis",
                    number_cortical_areas: 1,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                    },
                    cortical_area_types: {
                        (IOCorticalAreaDataType::Misc(frame_change_handling), 0)
                    },
                    unit_default_topology: {
                        0 => { relative_position: [0, 0, 0], dimensions: [8, 8, 1] }
                    }
                },

            }
        }
    };
}