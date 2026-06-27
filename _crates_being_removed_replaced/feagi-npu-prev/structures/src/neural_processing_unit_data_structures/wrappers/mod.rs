//! Wrappers are zero cost abstractions over quantizable types. While they are generally CPU
//! only, they still serve great help at context

mod indexes_spatial_cortical_area;
mod indexes_burst_engine;
mod indexes_typed_by_mp_quant_and_model;
mod values;

pub use indexes_spatial_cortical_area::*;
pub use indexes_burst_engine::*;
pub use indexes_typed_by_mp_quant_and_model::*;
pub use values::*;
