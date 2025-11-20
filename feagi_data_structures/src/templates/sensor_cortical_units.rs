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
                    cortical_area_types: {
                        (IOCorticalAreaDataType::Percentage(frame_change_handling, percentage_neuron_positioning), 0)
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
                    cortical_area_types: {
                        (IOCorticalAreaDataType::Percentage(frame_change_handling, percentage_neuron_positioning), 0)
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
                    cortical_area_types: {
                        (IOCorticalAreaDataType::Percentage(frame_change_handling, percentage_neuron_positioning), 0)
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
                    cortical_area_types: {
                        (IOCorticalAreaDataType::Percentage(frame_change_handling, percentage_neuron_positioning), 0)
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
                    cortical_area_types: {
                        (IOCorticalAreaDataType::Percentage(frame_change_handling, percentage_neuron_positioning), 0)
                    }
                },

                #[doc = "Analog GPIO input, such as an input from the GPIO pins on a Raspberry pi"]
                Servo => {
                    friendly_name: "Analog GPIO Sensor",
                    snake_case_name: "analog_gpio",
                    accepted_wrapped_io_data_type: Percentage,
                    cortical_id_unit_reference: *b"agp",
                    number_cortical_areas: 1,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                        percentage_neuron_positioning: PercentageNeuronPositioning
                    },
                    cortical_area_types: {
                        (IOCorticalAreaDataType::Percentage(frame_change_handling, percentage_neuron_positioning), 0)
                    }
                },

                #[doc = "Miscellaneous sensor that does not fit existing templates."]
                Misc => {
                    friendly_name: "Miscellaneous Sensor",
                    snake_case_name: "miscellaneous",
                    accepted_wrapped_io_data_type: Miscellaneous,
                    cortical_id_unit_reference: *b"mis",
                    number_cortical_areas: 1,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                    },
                    cortical_area_types: {
                        (IOCorticalAreaDataType::Misc(frame_change_handling), 0)
                    }
                },

                #[doc = "Camera vision input"]
                Servo => {
                    friendly_name: "Vision Sensor",
                    snake_case_name: "vision",
                    accepted_wrapped_io_data_type: ImageFrame,
                    cortical_id_unit_reference: *b"img",
                    number_cortical_areas: 1,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                    },
                    cortical_area_types: {
                        (IOCorticalAreaDataType::ImageFrame(frame_change_handling), 0)
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
                    cortical_area_types: {
                        (IOCorticalAreaDataType::CartesianPlane(frame_change_handling), 0),
                        (IOCorticalAreaDataType::CartesianPlane(frame_change_handling), 1),
                        (IOCorticalAreaDataType::CartesianPlane(frame_change_handling), 2),
                        (IOCorticalAreaDataType::CartesianPlane(frame_change_handling), 3),
                        (IOCorticalAreaDataType::CartesianPlane(frame_change_handling), 4),
                        (IOCorticalAreaDataType::CartesianPlane(frame_change_handling), 5),
                        (IOCorticalAreaDataType::CartesianPlane(frame_change_handling), 6),
                        (IOCorticalAreaDataType::CartesianPlane(frame_change_handling), 7),
                        (IOCorticalAreaDataType::CartesianPlane(frame_change_handling), 8),
                    }
                },


                /*
                #[doc = "IMU, allows for relative tracking of position and rotation"]
                InertialMeasurementUnit => {
                    friendly_name: "Inertial measurement unit",
                    snake_case_name: "inertial_measurement_unit",
                    accepted_wrapped_io_data_type: SegmentedImageFrame,
                    cortical_id_unit_reference: *b"imu",
                    number_cortical_areas: 2,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                        percentage_neuron_positioning: PercentageNeuronPositioning
                    },
                    cortical_area_types: {
                        (IOCorticalAreaDataType::SignedPercentage3D(frame_change_handling, percentage_neuron_positioning), 0),
                        (IOCorticalAreaDataType::SignedPercentage4D(frame_change_handling, percentage_neuron_positioning), 1),
                    }
                },
                 */




            }
        }
    };
}