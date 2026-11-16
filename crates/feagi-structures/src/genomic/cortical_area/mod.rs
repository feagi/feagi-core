#[allow(clippy::module_inception)]
mod cortical_area;
mod cortical_area_type;
mod cortical_id;
pub mod descriptors;
pub mod io_cortical_area_configuration_flag;

pub use cortical_area::CorticalArea;
pub use cortical_area_type::{
    default_psp_uniform_distribution, CoreCorticalType, CorticalAreaType, CustomCorticalType,
    MemoryCorticalType, PSP_UNIFORM_DISTRIBUTION_KEY,
};
pub use cortical_id::CorticalID;
pub use descriptors::CorticalAreaDimensions;
pub use io_cortical_area_configuration_flag::IOCorticalAreaConfigurationFlag;
