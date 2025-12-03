//mod cortical_area;
mod cortical_type;
mod cortical_id;
pub mod io_cortical_area_data_type; // TODO remove me
pub mod descriptors;

pub use cortical_type::{CorticalAreaType, MemoryCorticalType, CustomCorticalType, CoreCorticalType};
pub use io_cortical_area_data_type::{IOCorticalAreaDataFlag};
pub use cortical_id::CorticalID;
