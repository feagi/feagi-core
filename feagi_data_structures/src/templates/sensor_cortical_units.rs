//! Sensor cortical type definitions for FEAGI.
//!
//! Defines all supported sensor types including infrared, ultrasonic,
//! accelerometer, gyroscope, camera, and various other sensory inputs.

/// Macro defining all sensor (input processing unit) cortical types.
///
/// This macro generates enum variants and associated metadata for each
/// sensor type, including encoding methods, dimension ranges, and identifiers.
///
/// # Usage
/// ```ignore
/// sensor_definition!(define_io_cortical_types);
/// ```
#[macro_export]
macro_rules! sensor_definition {
    ($callback:ident) => {
        $callback! {
            SensorCorticalUnit {

                #[doc = "Infrared distance sensor for object detection."]
                Infrared => {
                    friendly_name: "Infrared Sensor",
                    snake_case_identifier: "infrared",
                    
                    
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