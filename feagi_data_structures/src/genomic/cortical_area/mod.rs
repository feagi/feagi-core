//mod cortical_area;
mod cortical_type;
mod cortical_id;
pub mod io_cortical_area_data_type;
pub mod descriptors;

pub use cortical_type::{CorticalType, MemoryCorticalType, CustomCorticalType, CoreCorticalType};
pub use io_cortical_area_data_type::{IOCorticalAreaDataType};
pub use cortical_id::CorticalID;
