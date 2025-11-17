#[macro_export]
macro_rules! sensor_cortical_units {
    ($callback:ident) => {
        $callback! {
            SensoryCorticalUnit {

                #[doc = "Infrared distance sensor for object detection."]
                Infrared => {
                    friendly_name: "Infrared Sensor",
                    snake_case_name: "infrared",
                    accepted_wrapped_io_data_type: WrappedIOType::Percentage,
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

                #[doc = "Segmented vision processing, with a higher resolution center and lower resolution peripherals"]
                SegmentedVision => {
                    friendly_name: "Segmented Vision",
                    snake_case_name: "segmented_vision",
                    accepted_wrapped_io_data_type: WrappedIOType::SegmentedImageFrame(None),
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

                #[doc = "IMU, allows for relative tracking of position and rotation"]
                InertialMeasurementUnit => {
                    friendly_name: "Inertial measurement unit",
                    snake_case_name: "inertial_measurement_unit",
                    accepted_wrapped_io_data_type: WrappedIOType::SegmentedImageFrame(None),
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







            }
        }
    };
}