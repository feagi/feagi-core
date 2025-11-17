#[macro_export]
macro_rules! sensor_cortical_units {
    ($callback:ident) => {
        $callback! {
            SensorCorticalUnit {

                // Cortical ID formatting
                // (1 char: base cortical type, 3 char: cortical unit type, 1 char data type configuration enum, 1 char subunit index, 2 chars subunit group index)

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
                        CorticalAreaType::BrainInput(IOCorticalAreaDataType::Percentage(frame_change_handling, percentage_neuron_positioning))
                    }
                }

                #[doc = "Segmented vision processing, with a higher resolution center and lower resolution peripherals"]
                SegmentedVision => {
                    friendly_name: "Segmented Vision",
                    snake_case_name: "segmented_vision",
                    accepted_wrapped_io_data_type: WrappedIOType::SegmentedImageFrame(None),
                    cortical_id_unit_reference: *b"svi"
                    number_areas: 9,
                    cortical_type_parameters: {
                        frame_change_handling: FrameChangeHandling,
                    },
                    cortical_area_types: {
                        CorticalAreaType::BrainInput(IOCorticalAreaDataType::Percentage(CartesianPlane))
                    }

                }







            }
        }
    };
}