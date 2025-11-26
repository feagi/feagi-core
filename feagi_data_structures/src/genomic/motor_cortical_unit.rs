use std::fmt::{Display, Formatter};
use std::collections::HashMap;
use crate::{motor_cortical_units, FeagiDataError};
use crate::genomic::cortical_area::{CorticalID, CorticalAreaType, IOCorticalAreaDataType};
use crate::genomic::cortical_area::descriptors::{CorticalGroupIndex, CorticalUnitIndex};
use crate::genomic::cortical_area::io_cortical_area_data_type::{FrameChangeHandling, PercentageNeuronPositioning};
use crate::genomic::sensory_cortical_unit::UnitTopology;
use paste;

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
                    cortical_area_types: {
                        $(($cortical_area_type_expr:expr, $area_index:expr)),* $(,)?
                    },
                    unit_default_topology: {
                        $($unit_idx:tt => { relative_position: [$rel_x:expr, $rel_y:expr, $rel_z:expr], dimensions: [$dim_x:expr, $dim_y:expr, $dim_z:expr] }),* $(,)?
                    }
                }
            ),* $(,)?
        }
    ) => {
        #[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
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
                            $(
                                $cortical_area_type_expr .as_io_cortical_id(true, cortical_unit_identifier, CorticalUnitIndex::from($area_index), cortical_group_index)
                            ),*
                        ]
                    }
                }
            )*

            pub const fn get_type_from_cortical_id_bytes(bytes: &[u8; CorticalID::NUMBER_OF_BYTES]) -> Result<MotorCorticalUnit, FeagiDataError> {
                if bytes[0] != b'i' {
                    return Err(FeagiDataError::ConstError("Given Cortical ID cannot be decoded into a sensory cortical unit as it does not start with 'i'"));
                }
                todo!();
            }

            pub const fn get_snake_case_name(&self) -> &'static str {
                match self {
                    $(
                        MotorCorticalUnit::$variant_name => $snake_case_name,
                    )*
                }
            }

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
            pub fn get_unit_default_topology(&self) -> HashMap<usize, UnitTopology> {
                match self {
                    $(
                        MotorCorticalUnit::$variant_name => {
                            let mut topology = HashMap::new();
                            $(
                                topology.insert(
                                    $unit_idx,
                                    UnitTopology {
                                        relative_position: [$rel_x, $rel_y, $rel_z],
                                        dimensions: [$dim_x, $dim_y, $dim_z],
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

