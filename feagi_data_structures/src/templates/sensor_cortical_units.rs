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
                    snake_case_identifier: "infrared",
                    accepted_data_type: Percentage,
                    subunit_identifiers: {
                        // Base Cortical ID (descriptor enu)
                        (b"i" b"inf" [0] [0] [0, 0] , Percentage_Absolute_Linear),
                    }

                }






                #[doc = "Infrared distance sensor for object detection."]
                Infrared => {
                    friendly_name: "Infrared Sensor",
                    snake_case_identifier: "infrared",
                    wrapped_data_type: WrappedIOType::Percentage,
                    data_type: Percentage,
                    subtype_mappings: [
                        (b"iinf00", (Absolute, Linear), Percentage_Absolute_Linear),
                        (b"iINF00", (Absolute, Fractional), Percentage_Absolute_Fractional),
                        (b"Iinf00", (Incremental, Linear), Percentage_Incremental_Linear),
                        (b"IINF00", (Incremental, Fractional), Percentage_Incremental_Fractional),
                    ]
                },


            }
        }
    };
}