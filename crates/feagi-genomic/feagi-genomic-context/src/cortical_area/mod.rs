
//mod cortical_area;
mod cortical_area_type;
mod cortical_id;
mod dimensional_cortical_area_type;
pub mod io_cortical_area_configuration_flag;

pub mod descriptors;
mod cortical_area_model_type;
mod cortical_area_count;
mod cortical_channel;

//pub use cortical_area::CorticalArea;
pub use cortical_area_type::{
    CoreCorticalType, CorticalAreaType, CustomCorticalType, MemoryCorticalType,
};
pub use cortical_id::CorticalID;
pub use dimensional_cortical_area_type::DimensionCorticalAreaType;
pub use cortical_area_model_type::CorticalAreaModelType;
