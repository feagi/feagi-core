use std::fmt::Display;
use crate::{sensor_cortical_units, FeagiDataError};
use crate::genomic::cortical_area::{CorticalID, CorticalAreaType, IOCorticalAreaDataType};
use crate::genomic::cortical_area::descriptors::{CorticalGroupIndex, CorticalUnitIndex};
use crate::genomic::cortical_area::io_cortical_area_data_type::{DataTypeConfigurationFlag, FrameChangeHandling, PercentageNeuronPositioning};
use paste;

macro_rules! define_sensory_cortical_units_enum {
    (
        SensorCorticalUnit {
            $(
                $(#[doc = $doc:expr])?
                $variant_name:ident => {
                    friendly_name: $friendly_name:expr,
                    snake_case_name: $snake_case_name:expr,
                    accepted_wrapped_io_data_type: $accepted_wrapped_io_data_type:expr,
                    cortical_id_unit_reference: $cortical_id_unit_reference:expr,
                    number_cortical_areas: $number_cortical_areas:expr,
                    cortical_type_parameters: {
                        $($param_name:ident: $param_type:ty),* $(,)?
                    },
                    cortical_area_types: {
                        $(($cortical_area_type_expr:expr, $area_index:expr)),* $(,)?
                    }
                }
            ),* $(,)?
        }
    ) => {
        #[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
        pub enum SensoryCorticalUnit {
            $(
                $(#[doc = $doc])?
                $variant_name,
            )*
        }

        impl SensoryCorticalUnit {
            $(
                paste::paste! {
                    #[doc = "Get cortical area types array for " $friendly_name "."]
                    pub const fn [<get_ $snake_case_name _cortical_area_types_array>](
                        $($param_name: $param_type),*) -> [CorticalAreaType; $number_cortical_areas] {
                        [
                            $(CorticalAreaType::BrainInput($cortical_area_type_expr)),*
                        ]
                    }

                    #[doc = "Get cortical IDs array for " $friendly_name "."]
                    pub const fn [<get_ $snake_case_name _cortical_ids_array>](
                        $($param_name: $param_type,)* cortical_group_index: CorticalGroupIndex) -> [CorticalID; $number_cortical_areas] {
                        let cortical_unit_identifier: [u8; 3] = $cortical_id_unit_reference;
                        [
                            $($cortical_area_type_expr .as_io_cortical_id(true, cortical_unit_identifier, CorticalUnitIndex::from($area_index), cortical_group_index)),*
                        ]

                    }
                }
            )*

        }
    };


}
// Generate the SensoryCorticalUnit enum and all helper methods from the template
sensor_cortical_units!(define_sensory_cortical_units_enum);