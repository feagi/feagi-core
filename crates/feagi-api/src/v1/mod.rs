// API Version 1 - DTOs and mappings

pub mod dtos;
pub mod cortical_area_dtos;
pub mod brain_region_dtos;
pub mod mapping_dtos;
pub mod genome_dtos;
pub mod neuron_dtos;
pub mod runtime_dtos;
pub mod analytics_dtos;

// Re-export for convenience
pub use dtos::*;
pub use cortical_area_dtos::*;
pub use brain_region_dtos::*;
pub use mapping_dtos::*;
pub use genome_dtos::*;
pub use neuron_dtos::*;
pub use runtime_dtos::*;
pub use analytics_dtos::*;

// TODO: Add mapping module
// pub mod mapping;

