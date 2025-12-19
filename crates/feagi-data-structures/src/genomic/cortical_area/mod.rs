mod cortical_area;
mod cortical_id;
mod cortical_type;
pub mod descriptors;
pub mod io_cortical_area_data_type; // TODO remove me

pub use cortical_area::CorticalArea;
pub use cortical_id::CorticalID;
pub use cortical_type::{
    CoreCorticalType, CorticalAreaType, CustomCorticalType, MemoryCorticalType,
};
pub use descriptors::CorticalAreaDimensions;
pub use io_cortical_area_data_type::IOCorticalAreaDataFlag;
