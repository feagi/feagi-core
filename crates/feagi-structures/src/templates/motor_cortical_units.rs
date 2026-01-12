#[macro_export]
macro_rules! motor_cortical_units {
    ($callback:ident) => {
        $callback! {
            MotorCorticalUnit {

                #[doc = "Free spinning motor."]
                RotaryMotor => {
                    friendly_name: "Rotary Motor",
                    accepted_wrapped_io_data_type: SignedPercentage, // This property determines what type of registration funciton will be generated
                    cortical_id_unit_reference: *b"mot",
                    number_cortical_areas: 1,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                        percentage_neuron_positioning: PercentageNeuronPositioning
                    },
                    cortical_area_properties: {
                        0 => (IOCorticalAreaConfigurationFlag::SignedPercentage(frame_change_handling, percentage_neuron_positioning), relative_position: [-20, 0, -10], channel_dimensions_default: [1, 1, 10], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [1, 1, 1024])
                    }
                },

                #[doc = "Servo Position, defined by min / max distances"]
                PositionalServo => {
                    friendly_name: "Positional Servo",
                    accepted_wrapped_io_data_type: SignedPercentage,
                    cortical_id_unit_reference: *b"pse",
                    number_cortical_areas: 1,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                        percentage_neuron_positioning: PercentageNeuronPositioning
                    },
                    cortical_area_properties: {
                        0 => (IOCorticalAreaConfigurationFlag::SignedPercentage(frame_change_handling, percentage_neuron_positioning), relative_position: [-10, 0, -10], channel_dimensions_default: [1, 1, 10], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [1, 1, 1024])
                    }
                },

                #[doc = "Gaze control, where the first 2 numbers are the XY center, and the last number is the relative size"]
                Gaze => {
                    friendly_name: "Gaze Control",
                    accepted_wrapped_io_data_type: GazeProperties,
                    cortical_id_unit_reference: *b"gaz",
                    number_cortical_areas: 2,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                        percentage_neuron_positioning: PercentageNeuronPositioning
                    },
                    cortical_area_properties: {
                        0 => (IOCorticalAreaConfigurationFlag::Percentage2D(frame_change_handling, percentage_neuron_positioning), relative_position: [10, 0, -10], channel_dimensions_default: [8, 8, 1], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [1024, 1024, 1]), // Eccentricity
                        1 => (IOCorticalAreaConfigurationFlag::Percentage(frame_change_handling, percentage_neuron_positioning), relative_position: [0, 0, -10], channel_dimensions_default: [1, 1, 10], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [1, 1, 1024]) // Modularity
                    }
                },


                #[doc = "Miscellaneous motor that does not fit existing templates."]
                MiscData => {
                    friendly_name: "Miscellaneous Motor",
                    accepted_wrapped_io_data_type: MiscData,
                    cortical_id_unit_reference: *b"mis",
                    number_cortical_areas: 1,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                    },
                    cortical_area_properties: {
                        0 => (IOCorticalAreaConfigurationFlag::Misc(frame_change_handling), relative_position: [-30, 0, -10], channel_dimensions_default: [8, 8, 1], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [1024, 1024, 1024])
                    }
                },

                #[doc = "Text output (English) - token stream encoded as absolute fractional bitplanes along Z (z=0 is MSB)."]
                TextEnglishOutput => {
                    friendly_name: "Text Output (English)",
                    accepted_wrapped_io_data_type: MiscData,
                    cortical_id_unit_reference: *b"ten",
                    number_cortical_areas: 1,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                    },
                    allowed_frame_change_handling: [Absolute],
                    cortical_area_properties: {
                        // 1x1x16 default: one token per FEAGI tick, encoded into 16 bitplanes (supports GPT-2 vocab via token_id+1 offset).
                        0 => (IOCorticalAreaConfigurationFlag::Misc(frame_change_handling), relative_position: [50, 0, -10], channel_dimensions_default: [1, 1, 16], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [1, 1, 32])
                    }
                },

                #[doc = "Object semantic segmentation output (bitplane class encoding)"]
                ObjectSegmentation => {
                    friendly_name: "Object Segmentation",
                    accepted_wrapped_io_data_type: MiscData,
                    cortical_id_unit_reference: *b"seg",
                    number_cortical_areas: 1,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                    },
                    allowed_frame_change_handling: [Absolute],
                    cortical_area_properties: {
                        0 => (IOCorticalAreaConfigurationFlag::Misc(frame_change_handling), relative_position: [-50, 30, 0], channel_dimensions_default: [32, 32, 8], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [4096, 4096, 1024])
                    }
                },

                #[doc = "Visual thoughts output - RGB image generation from brain activity"]
                SimpleVisionOutput => {
                    friendly_name: "Simple Vision Output",
                    accepted_wrapped_io_data_type: ImageFrame,
                    cortical_id_unit_reference: *b"img",
                    number_cortical_areas: 1,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                    },
                    cortical_area_properties: {
                        0 => (IOCorticalAreaConfigurationFlag::CartesianPlane(frame_change_handling), relative_position: [-50, 100, 0], channel_dimensions_default: [128, 128, 3], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [4096, 4096, 3])
                    }
                },

                #[doc = "Image Processing configuration - dynamically control brightness, contrast, and per pixel diff thresholding"]
                DynamicImageProcessing => {
                    friendly_name: "Dynamic Image Processing Settings",
                    accepted_wrapped_io_data_type: ImageFilteringSettings,
                    cortical_id_unit_reference: *b"ifs",
                    number_cortical_areas: 4,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                        percentage_neuron_positioning: PercentageNeuronPositioning
                    },
                    cortical_area_properties: {
                        0 => (IOCorticalAreaConfigurationFlag::Percentage(frame_change_handling, percentage_neuron_positioning), relative_position: [0, 0, 0], channel_dimensions_default: [1, 1, 10], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [1, 1, 1024]), // brightness
                        1 => (IOCorticalAreaConfigurationFlag::Percentage(frame_change_handling, percentage_neuron_positioning), relative_position: [0, 0, -10], channel_dimensions_default: [1, 1, 10], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [1, 1, 1024]), // contrast
                        2 => (IOCorticalAreaConfigurationFlag::Percentage2D(frame_change_handling, percentage_neuron_positioning), relative_position: [0, 0, -30], channel_dimensions_default: [2, 1, 10], channel_dimensions_min: [2, 1, 1], channel_dimensions_max: [2, 1, 1024]), // per pixel diff
                        3 => (IOCorticalAreaConfigurationFlag::Percentage2D(frame_change_handling, percentage_neuron_positioning), relative_position: [0, 0, -30], channel_dimensions_default: [2, 1, 10], channel_dimensions_min: [2, 1, 1], channel_dimensions_max: [2, 1, 1024]) // image diff
                    }
                },

            }
        }
    };
}
