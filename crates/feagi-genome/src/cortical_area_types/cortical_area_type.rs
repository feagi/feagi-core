//! The `CorticalAreaType` family, laid out as its cortical type family template expects: the base
//! trait and every generated item of the family, with nested sub modules per super type and
//! cortical type, each cortical type module holding its struct

pub use crate::basis::cortical_area::cortical_area_type::CorticalAreaType;
pub use super::family::*;

/// The `Foundation` super type
pub mod foundation {
    /// The `Power` cortical type
    pub mod power {
        pub use crate::cortical_area_types::super_type_foundation::power::*;
    }
}

/// The `Interconnect` cortical type
pub mod interconnect {
    pub use crate::cortical_area_types::interconnect::*;
}
