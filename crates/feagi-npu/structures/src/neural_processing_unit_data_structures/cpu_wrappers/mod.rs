//! Wrappers are zero cost abstractions over quantizable types. While they are generally CPU
//! only, they still serve great help at context

mod cortical_spatial;
mod indexes_global;
mod indexes_typed_by_mp_quant_and_model;
mod values;

pub use cortical_spatial::*;
pub use indexes_global::*;
pub use indexes_typed_by_mp_quant_and_model::*;
pub use values::*;
