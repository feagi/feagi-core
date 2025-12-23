use crate::genomic::cortical_area::descriptors::CorticalUnitIndex;
use crate::genomic::cortical_area::descriptors::{CorticalSubUnitIndex};
use crate::genomic::cortical_area::io_cortical_area_data_type::{
    FrameChangeHandling, PercentageNeuronPositioning,
};
use crate::genomic::cortical_area::{CorticalAreaType, CorticalID, IOCorticalAreaDataFlag};
use crate::genomic::sensory_cortical_unit::UnitTopology;
use crate::motor_cortical_units;
use paste;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

macro_rules! define_motor_cortical_units_enum {
    (
        MotorCorticalUnit {
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
        #[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, serde::Deserialize, serde::Serialize)]
        pub enum MotorCorticalUnit {
            $(
                $(#[doc = $doc])?
                $variant_name,
            )*
        }

        impl MotorCorticalUnit {
            $(
                paste::paste! {
                    #[doc = "Get cortical area types array for " $friendly_name "."]
                    pub const fn [<get_cortical_area_types_array_for_ $snake_case_name >](
                        $($param_name: $param_type),*) -> [CorticalAreaType; $number_cortical_areas] {
                        [
                            $(CorticalAreaType::BrainOutput($cortical_area_type_expr)),*
                        ]
                    }

                    #[doc = "Get cortical IDs array for " $friendly_name "."]
                    pub const fn [<get_cortical_ids_array_for_ $snake_case_name >](
                        $($param_name: $param_type,)* cortical_unit_index: CorticalUnitIndex) -> [CorticalID; $number_cortical_areas] {
                        let cortical_unit_identifier: [u8; 3] = $cortical_id_unit_reference;
                        [
                            $(
                                $cortical_area_type_expr .as_io_cortical_id(false, cortical_unit_identifier, cortical_unit_index, CorticalSubUnitIndex::from($cortical_sub_unit_index))
                            ),*
                        ]
                    }
                }
            )*

            pub const fn get_snake_case_name(&self) -> &'static str {
                match self {
                    $(
                        MotorCorticalUnit::$variant_name => $snake_case_name,
                    )*
                }
            }

            /// Parse a motor cortical unit from its snake_case name
            ///
            /// # Arguments
            /// * `name` - The snake_case name (e.g., "positional_servo", "led_matrix")
            ///
            /// # Returns
            /// * `Some(MotorCorticalUnit)` - If name matches a known type
            /// * `None` - If name is not recognized
            pub fn from_snake_case_name(name: &str) -> Option<MotorCorticalUnit> {
                match name {
                    $(
                        $snake_case_name => Some(MotorCorticalUnit::$variant_name),
                    )*
                    _ => None,
                }
            }

            // TODO from_snake_case_name_const

            /// Returns all available motor cortical unit types.
            /// This is useful for enumerating all possible motor types in the system.
            pub const fn list_all() -> &'static [MotorCorticalUnit] {
                &[
                    $(
                        MotorCorticalUnit::$variant_name,
                    )*
                ]
            }

            /// Returns the friendly (human-readable) name for this motor cortical unit type.
            pub const fn get_friendly_name(&self) -> &'static str {
                match self {
                    $(
                        MotorCorticalUnit::$variant_name => $friendly_name,
                    )*
                }
            }

            /// Returns the 3-byte cortical ID unit reference for this type.
            pub const fn get_cortical_id_unit_reference(&self) -> [u8; 3] {
                match self {
                    $(
                        MotorCorticalUnit::$variant_name => $cortical_id_unit_reference,
                    )*
                }
            }

            /// Returns the number of cortical areas this type creates.
            pub const fn get_number_cortical_areas(&self) -> usize {
                match self {
                    $(
                        MotorCorticalUnit::$variant_name => $number_cortical_areas,
                    )*
                }
            }

            /// Returns the default topology for all units of this cortical type.
            pub fn get_unit_default_topology(&self) -> HashMap<CorticalSubUnitIndex, UnitTopology> {
                match self {
                    $(
                        MotorCorticalUnit::$variant_name => {
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

        impl Display for MotorCorticalUnit {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(
                        MotorCorticalUnit::$variant_name => write!(f, $friendly_name),
                    )*
                }
            }
        }
    };

}
// Generate the MotorCorticalUnit enum and all helper methods from the template
motor_cortical_units!(define_motor_cortical_units_enum);
