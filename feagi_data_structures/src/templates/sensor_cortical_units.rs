#[macro_export]
macro_rules! sensor_cortical_units {
    ($callback:ident) => {
        $callback! {
            SensoryCorticalUnit {

                #[doc = "Infrared distance sensor for object detection."]
                Infrared => {
                    friendly_name: "Infrared Sensor",
                    snake_case_name: "infrared",
                    accepted_wrapped_io_data_type: Percentage,
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





            }
        }
    };
}