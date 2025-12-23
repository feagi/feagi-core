use crate::genomic::cortical_area::descriptors::CorticalUnitIndex;
use crate::genomic::cortical_area::descriptors::{CorticalSubUnitIndex};
use crate::genomic::cortical_area::io_cortical_area_data_type::{
    FrameChangeHandling, PercentageNeuronPositioning,
};
use crate::genomic::cortical_area::{CorticalAreaType, CorticalID, IOCorticalAreaDataFlag};
use crate::sensor_cortical_units;
use paste;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UnitTopology {
    pub relative_position: [i32; 3],
    pub channel_dimensions_default: [u32; 3],
    pub channel_dimensions_min: [u32; 3],
    pub channel_dimensions_max: [u32; 3],
}

macro_rules! define_sensory_cortical_units_enum {
    (
        SensoryCorticalUnit {
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
                    cortical_area_properties: {
                        $($cortical_sub_unit_index:tt => ($cortical_area_type_expr:expr, relative_position: [$rel_x:expr, $rel_y:expr, $rel_z:expr], channel_dimensions_default: [$dim_default_x:expr, $dim_default_y:expr, $dim_default_z:expr], channel_dimensions_min: [$dim_min_x:expr, $dim_min_y:expr, $dim_min_z:expr], channel_dimensions_max: [$dim_max_x:expr, $dim_max_y:expr, $dim_max_z:expr])),* $(,)?
                    }
                }
            ),* $(,)?
        }
    ) => {
        #[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, serde::Serialize, serde::Deserialize)]
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
                    pub const fn [<get_cortical_area_types_array_for_ $snake_case_name >](
                        $($param_name: $param_type),*) -> [CorticalAreaType; $number_cortical_areas] {
                        [
                            $(CorticalAreaType::BrainInput($cortical_area_type_expr)),*
                        ]
                    }

                    #[doc = "Get cortical IDs array for " $friendly_name "."]
                    pub const fn [<get_cortical_ids_array_for_ $snake_case_name >](
                        $($param_name: $param_type,)* cortical_unit_index: CorticalUnitIndex) -> [CorticalID; $number_cortical_areas] {
                        let cortical_unit_identifier: [u8; 3] = $cortical_id_unit_reference;
                        [
                            $(
                                $cortical_area_type_expr .as_io_cortical_id(true, cortical_unit_identifier, cortical_unit_index, CorticalSubUnitIndex::from($cortical_sub_unit_index))
                            ),*
                        ]
                    }
                }
            )*

            pub const fn get_snake_case_name(&self) -> &'static str {
                match self {
                    $(
                        SensoryCorticalUnit::$variant_name => $snake_case_name,
                    )*
                }
            }

            /// Parse a sensory cortical unit from its snake_case name
            ///
            /// # Arguments
            /// * `name` - The snake_case name (e.g., "simple_vision", "accelerometer")
            ///
            /// # Returns
            /// * `Some(SensoryCorticalUnit)` - If name matches a known type
            /// * `None` - If name is not recognized
            pub fn from_snake_case_name(name: &str) -> Option<SensoryCorticalUnit> {
                match name {
                    $(
                        $snake_case_name => Some(SensoryCorticalUnit::$variant_name),
                    )*
                    _ => None,
                }
            }

            /// Returns all available sensory cortical unit types.
            /// This is useful for enumerating all possible sensor types in the system.
            pub const fn list_all() -> &'static [SensoryCorticalUnit] {
                &[
                    $(
                        SensoryCorticalUnit::$variant_name,
                    )*
                ]
            }

            /// Returns the friendly (human-readable) name for this sensory cortical unit type.
            pub const fn get_friendly_name(&self) -> &'static str {
                match self {
                    $(
                        SensoryCorticalUnit::$variant_name => $friendly_name,
                    )*
                }
            }

            /// Returns the 3-byte cortical ID unit reference for this type.
            pub const fn get_cortical_id_unit_reference(&self) -> [u8; 3] {
                match self {
                    $(
                        SensoryCorticalUnit::$variant_name => $cortical_id_unit_reference,
                    )*
                }
            }

            /// Returns the number of cortical areas this type creates.
            pub const fn get_number_cortical_areas(&self) -> usize {
                match self {
                    $(
                        SensoryCorticalUnit::$variant_name => $number_cortical_areas,
                    )*
                }
            }



            /// Returns the default topology for all units of this cortical type.
            pub fn get_unit_default_topology(&self) -> HashMap<CorticalSubUnitIndex, UnitTopology> {
                match self {
                    $(
                        SensoryCorticalUnit::$variant_name => {
                            let mut topology = HashMap::new();
                            $(
                                topology.insert(
                                    CorticalSubUnitIndex::from($cortical_sub_unit_index),
                                    UnitTopology {
                                        relative_position: [$rel_x, $rel_y, $rel_z],
                                        channel_dimensions_default: [$dim_default_x, $dim_default_y, $dim_default_z],
                                        channel_dimensions_min: [$dim_min_x, $dim_min_y, $dim_min_z],
                                        channel_dimensions_max: [$dim_max_x, $dim_max_y, $dim_max_z],
                                    }
                                );
                            )*
                            topology
                        }
                    )*
                }
            }

        }

        impl Display for SensoryCorticalUnit {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        SensoryCorticalUnit::$variant_name => write!(f, $friendly_name),
                    )*
                }
    }
        }
    };

}
// Generate the SensoryCorticalUnit enum and all helper methods from the template
sensor_cortical_units!(define_sensory_cortical_units_enum);
