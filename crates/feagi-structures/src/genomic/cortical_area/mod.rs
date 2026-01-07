#[allow(clippy::module_inception)]
mod cortical_area;
mod cortical_id;
mod cortical_area_type;
pub mod descriptors;
pub mod io_cortical_area_configuration_flag; // TODO remove me

pub use cortical_area::CorticalArea;
pub use cortical_id::CorticalID;
pub use cortical_area_type::{
    CoreCorticalType, CorticalAreaType, CustomCorticalType, MemoryCorticalType,
};
pub use descriptors::CorticalAreaDimensions;
pub use io_cortical_area_configuration_flag::IOCorticalAreaConfigurationFlag;
