//! Relates to things not specific to the genome structs themselves. Always imported with this
//! crate. This module is not disabled via features


#[cfg(feature = "std")]
pub mod brain_region_identifiers;
//pub mod cortical_area;
pub mod cortical_mapping;
pub mod connectivity_rule;
pub mod mapping_entry;
pub mod cortical_area;

