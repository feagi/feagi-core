use crate::genomic::cortical_area::descriptors::CorticalSubUnitIndex;
use crate::genomic::cortical_area::descriptors::CorticalUnitIndex;
use crate::genomic::cortical_area::io_cortical_area_configuration_flag::{
    FrameChangeHandling, PercentageNeuronPositioning,
};
use crate::genomic::cortical_area::{
    CorticalAreaType, CorticalID, IOCorticalAreaConfigurationFlag,
};
use crate::sensor_cortical_units;
use paste;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)] // TODO move me!
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
                    accepted_wrapped_io_data_type: $accepted_wrapped_io_data_type:expr,
                    cortical_id_unit_reference: $cortical_id_unit_reference:expr,
                    number_cortical_areas: $number_cortical_areas:expr,
                    cortical_type_parameters: {
                        $($param_name:ident: $param_type:ty),* $(,)?
                    },
                    $(allowed_frame_change_handling: [$($allowed_frame:ident),* $(,)?],)? // TODO delete this!
                    cortical_area_properties: {
                        $($cortical_sub_unit_index:tt => ($io_cortical_area_configuration_flag_expr:expr, relative_position: [$rel_x:expr, $rel_y:expr, $rel_z:expr], channel_dimensions_default: [$dim_default_x:expr, $dim_default_y:expr, $dim_default_z:expr], channel_dimensions_min: [$dim_min_x:expr, $dim_min_y:expr, $dim_min_z:expr], channel_dimensions_max: [$dim_max_x:expr, $dim_max_y:expr, $dim_max_z:expr])),* $(,)?
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
                    #[doc = "Get cortical area types array for " $friendly_name " using individual parameters."]
                    pub const fn [<get_cortical_area_types_array_for_ $variant_name:snake _with_parameters >](
                        $($param_name: $param_type),*) -> [CorticalAreaType; $number_cortical_areas] {
                        [
                            $(CorticalAreaType::BrainInput($io_cortical_area_configuration_flag_expr)),*
                        ]
                    }

                    #[doc = "Get cortical IDs array for " $friendly_name " using individual parameters."]
                    pub const fn [<get_cortical_ids_array_for_ $variant_name:snake _with_parameters >](
                        $($param_name: $param_type,)* cortical_unit_index: CorticalUnitIndex) -> [CorticalID; $number_cortical_areas] {
                        let cortical_unit_identifier: [u8; 3] = $cortical_id_unit_reference;
                        [
                            $(
                                $io_cortical_area_configuration_flag_expr .as_io_cortical_id(true, cortical_unit_identifier, cortical_unit_index, CorticalSubUnitIndex::from($cortical_sub_unit_index))
                            ),*
                        ]
                    }
                }
            )*

            pub const fn get_snake_case_name(&self) -> &'static str {
                match self {
                    $(
                        SensoryCorticalUnit::$variant_name => paste::paste!{ stringify!([<$variant_name:snake>]) },
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
                        paste::paste!{ stringify!([<$variant_name:snake>]) } => Some(SensoryCorticalUnit::$variant_name),
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

            /// Returns the 3-byte cortical ID unit reference for this type. // TODO delete me!
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

            /// Returns the accepted wrapped IO data type name for this sensory unit type.
            pub const fn get_accepted_wrapped_io_data_type(&self) -> &'static str { // TODO delete me!
                match self {
                    $(
                        SensoryCorticalUnit::$variant_name => stringify!($accepted_wrapped_io_data_type),
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


            /// Returns the allowed frame change handling modes from the template, if restricted.
            /// If None is returned, all frame change handling modes are allowed.
            /// If Some is returned, only the specified modes are valid.
            pub fn get_allowed_frame_change_handling(&self) -> Option<&'static [FrameChangeHandling]> { // TODO delete me!
                match self {
                    $(
                        SensoryCorticalUnit::$variant_name => {
                            $crate::get_allowed_frame_change_handling_impl!($($($allowed_frame),*)?)
                        }
                    )*
                }
            }

            pub fn get_cortical_id_vector_from_index_and_serde_io_configuration_flags(&self, cortical_unit_index: CorticalUnitIndex, map: Map<String, Value>) -> Result<Vec<CorticalID>, crate::FeagiDataError> {
                match self {
                    $(
                        SensoryCorticalUnit::$variant_name => {
                            paste::paste! {
                                let array = SensoryCorticalUnit::[<get_cortical_ids_array_for_ $variant_name:snake _with_parameters >](
                                    $($param_type::try_from_serde_map(&map)?,)*
                                    cortical_unit_index);
                                return Ok(array.to_vec());
                            }
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
