#[macro_export]
macro_rules! sensor_cortical_units {
    ($callback:ident) => {
        $callback! {
            SensoryCorticalUnit {

                #[doc = "Infrared distance sensor for object detection."]
                Infrared => {
                    friendly_name: "Infrared Sensor",
                    snake_case_name: "infrared",
                    accepted_wrapped_io_data_type: Percentage, // This property determines what type of registration funciton will be generated
                    cortical_id_unit_reference: *b"inf",
                    number_cortical_areas: 1,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                        percentage_neuron_positioning: PercentageNeuronPositioning
                    },
                    cortical_area_properties: {
                        0 => (IOCorticalAreaDataFlag::Percentage(frame_change_handling, percentage_neuron_positioning), relative_position: [0, 0, 0], channel_dimensions_default: [1, 1, 10], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [1, 1, 1024])
                    }
                },

                #[doc = "Proximity (distance) sensor for object detection."]
                Proximity => {
                    friendly_name: "Proximity Sensor",
                    snake_case_name: "proximity",
                    accepted_wrapped_io_data_type: Percentage,
                    cortical_id_unit_reference: *b"pro",
                    number_cortical_areas: 1,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                        percentage_neuron_positioning: PercentageNeuronPositioning
                    },
                    cortical_area_properties: {
                        0 => (IOCorticalAreaDataFlag::Percentage(frame_change_handling, percentage_neuron_positioning), relative_position: [0, 0, 0], channel_dimensions_default: [1, 1, 10], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [1, 1, 1024])
                    }
                },

                #[doc = "Shocking sensor for sensing 'pain'. Useful for training."]
                Shock => {
                    friendly_name: "Shock sensor",
                    snake_case_name: "shock",
                    accepted_wrapped_io_data_type: Percentage,
                    cortical_id_unit_reference: *b"shk",
                    number_cortical_areas: 1,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                        percentage_neuron_positioning: PercentageNeuronPositioning
                    },
                    cortical_area_properties: {
                        0 => (IOCorticalAreaDataFlag::Percentage(frame_change_handling, percentage_neuron_positioning), relative_position: [0, 0, 0], channel_dimensions_default: [1, 1, 10], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [1, 1, 1024])
                    }
                },

                #[doc = "Battery level sensor."]
                Battery => {
                    friendly_name: "Battery Sensor",
                    snake_case_name: "battery",
                    accepted_wrapped_io_data_type: Percentage,
                    cortical_id_unit_reference: *b"bat",
                    number_cortical_areas: 1,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                        percentage_neuron_positioning: PercentageNeuronPositioning
                    },
                    cortical_area_properties: {
                        0 => (IOCorticalAreaDataFlag::Percentage(frame_change_handling, percentage_neuron_positioning), relative_position: [0, 0, 0], channel_dimensions_default: [1, 1, 10], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [1, 1, 1024])
                    }
                },

                #[doc = "Servo position sensor for monitoring actuator position."]
                Servo => {
                    friendly_name: "Servo Sensor",
                    snake_case_name: "servo",
                    accepted_wrapped_io_data_type: Percentage,
                    cortical_id_unit_reference: *b"svm",
                    number_cortical_areas: 1,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                        percentage_neuron_positioning: PercentageNeuronPositioning
                    },
                    cortical_area_properties: {
                        0 => (IOCorticalAreaDataFlag::Percentage(frame_change_handling, percentage_neuron_positioning), relative_position: [0, 0, 0], channel_dimensions_default: [8, 8, 1], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [1024, 1024, 1])
                    }
                },

                #[doc = "Analog GPIO input, such as an input from the GPIO pins on a Raspberry pi"]
                AnalogGPIO => {
                    friendly_name: "Analog GPIO Sensor",
                    snake_case_name: "analog_gpio",
                    accepted_wrapped_io_data_type: Percentage,
                    cortical_id_unit_reference: *b"agp",
                    number_cortical_areas: 1,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                        percentage_neuron_positioning: PercentageNeuronPositioning
                    },
                    cortical_area_properties: {
                        0 => (IOCorticalAreaDataFlag::Percentage(frame_change_handling, percentage_neuron_positioning), relative_position: [0, 0, 0], channel_dimensions_default: [8, 8, 1], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [1024, 1024, 1])
                    }
                },

                #[doc = "Digital GPIO input, such as an input from the GPIO pins on a Raspberry pi"]
                DigitalGPIO => {
                    friendly_name: "Digital GPIO Sensor",
                    snake_case_name: "digital_gpio",
                    accepted_wrapped_io_data_type: Boolean,
                    cortical_id_unit_reference: *b"dgp",
                    number_cortical_areas: 1,
                    cortical_type_parameters: {},
                    cortical_area_properties: {
                        0 => (IOCorticalAreaDataFlag::Boolean, relative_position: [0, 0, 0], channel_dimensions_default: [1, 1, 1], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [1, 1, 1])
                    }
                },

                #[doc = "Miscellaneous sensor that does not fit existing templates."]
                MiscData => {
                    friendly_name: "Miscellaneous Sensor",
                    snake_case_name: "miscellaneous",
                    accepted_wrapped_io_data_type: MiscData,
                    cortical_id_unit_reference: *b"mis",
                    number_cortical_areas: 1,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                    },
                    cortical_area_properties: {
                        0 => (IOCorticalAreaDataFlag::Misc(frame_change_handling), relative_position: [0, 0, 0], channel_dimensions_default: [8, 8, 1], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [1024, 1024, 1])
                    }
                },

                #[doc = "Text input (English) - token stream encoded as absolute fractional bitplanes along Z (z=0 is MSB)."]
                TextEnglishInput => {
                    friendly_name: "Text Input (English)",
                    snake_case_name: "text_english_input",
                    accepted_wrapped_io_data_type: MiscData,
                    cortical_id_unit_reference: *b"ten",
                    number_cortical_areas: 1,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                    },
                    cortical_area_properties: {
                        // 1x1x16 default: one token per FEAGI tick, encoded into 16 bitplanes (supports GPT-2 vocab via token_id+1 offset).
                        0 => (IOCorticalAreaDataFlag::Misc(frame_change_handling), relative_position: [0, 0, 0], channel_dimensions_default: [1, 1, 16], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [1, 1, 32])
                    }
                },

                #[doc = "Camera vision input"]
                Vision => {
                    friendly_name: "Simple Vision",
                    snake_case_name: "simple_vision",
                    accepted_wrapped_io_data_type: ImageFrame,
                    cortical_id_unit_reference: *b"img",
                    number_cortical_areas: 1,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                    },
                    cortical_area_properties: {
                        0 => (IOCorticalAreaDataFlag::CartesianPlane(frame_change_handling), relative_position: [0, 0, 0], channel_dimensions_default: [64, 64, 3], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [4096, 4096, 3])
                    }
                },



                #[doc = "Segmented vision processing, with a higher resolution center and lower resolution peripherals"]
                SegmentedVision => {
                    friendly_name: "Segmented Vision",
                    snake_case_name: "segmented_vision",
                    accepted_wrapped_io_data_type: SegmentedImageFrame,
                    cortical_id_unit_reference: *b"svi",
                    number_cortical_areas: 9,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                    },
                    cortical_area_properties: {
                        0 => (IOCorticalAreaDataFlag::CartesianPlane(frame_change_handling), relative_position: [-70, -70, 0], channel_dimensions_default: [32, 32, 1], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [4096, 4096, 3]), // Lower Left
                        1 => (IOCorticalAreaDataFlag::CartesianPlane(frame_change_handling), relative_position: [60, -70, 0], channel_dimensions_default: [32, 32, 1], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [4096, 4096, 3]), // Lower Middle
                        2 => (IOCorticalAreaDataFlag::CartesianPlane(frame_change_handling), relative_position: [150, -70, 0], channel_dimensions_default: [32, 32, 1], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [4096, 4096, 3]), // Lower Right
                        3 => (IOCorticalAreaDataFlag::CartesianPlane(frame_change_handling), relative_position: [-70, 60, 0], channel_dimensions_default: [32, 32, 1], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [4096, 4096, 3]), // Middle Left
                        4 => (IOCorticalAreaDataFlag::CartesianPlane(frame_change_handling), relative_position: [0, 0, 0], channel_dimensions_default: [128, 128, 3], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [4096, 4096, 3]), // Middle Middle
                        5 => (IOCorticalAreaDataFlag::CartesianPlane(frame_change_handling), relative_position: [150, 60, 0], channel_dimensions_default: [32, 32, 1], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [4096, 4096, 3]), // Middle Right
                        6 => (IOCorticalAreaDataFlag::CartesianPlane(frame_change_handling), relative_position: [-70, 150, 0], channel_dimensions_default: [32, 32, 1], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [4096, 4096, 3]), // Upper Left
                        7 => (IOCorticalAreaDataFlag::CartesianPlane(frame_change_handling), relative_position: [60, 150, 0], channel_dimensions_default: [32, 32, 1], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [4096, 4096, 3]), // Upper Middle
                        8 => (IOCorticalAreaDataFlag::CartesianPlane(frame_change_handling), relative_position: [150, 150, 0], channel_dimensions_default: [32, 32, 1], channel_dimensions_min: [1, 1, 1], channel_dimensions_max: [4096, 4096, 3]) // Upper Right
                    }
                },


                #[doc = "Accelerometer, allows for relative tracking of position and motion"]
                Accelerometer => {
                    friendly_name: "Accelerometer",
                    snake_case_name: "accelerometer",
                    accepted_wrapped_io_data_type: Percentage_3D,
                    cortical_id_unit_reference: *b"acc",
                    number_cortical_areas: 1,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                        percentage_neuron_positioning: PercentageNeuronPositioning
                    },
                    cortical_area_properties: {
                        0 => (IOCorticalAreaDataFlag::SignedPercentage3D(frame_change_handling, percentage_neuron_positioning), relative_position: [0, 0, 0], channel_dimensions_default: [3, 1, 10], channel_dimensions_min: [3, 1, 1], channel_dimensions_max: [3, 1, 1024])
                    }
                },



                #[doc = "Gyroscope (Quaternion), Allows for tracking rotation without gimbal lock"]
                Gyroscope => {
                    friendly_name: "Gyroscope",
                    snake_case_name: "gyroscope",
                    accepted_wrapped_io_data_type: SignedPercentage_4D,
                    cortical_id_unit_reference: *b"gyq",
                    number_cortical_areas: 1,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                        percentage_neuron_positioning: PercentageNeuronPositioning
                    },
                    cortical_area_properties: {
                        0 => (IOCorticalAreaDataFlag::SignedPercentage4D(frame_change_handling, percentage_neuron_positioning), relative_position: [0, 0, 0], channel_dimensions_default: [4, 1, 10], channel_dimensions_min: [4, 1, 1], channel_dimensions_max: [4, 1, 1024])
                    }
                },




            }
        }
    };
}
