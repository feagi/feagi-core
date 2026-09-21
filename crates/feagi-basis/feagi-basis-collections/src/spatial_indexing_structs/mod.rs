
pub use dimensions::SpatialDimensions;
pub use stride::SpatialStride;
pub use coordinate::SpatialCoordinate;

pub mod axis_order;

mod dimensions;
mod stride;
mod coordinate;
pub mod spatial_index_mapper;
pub mod spatial_index_context;

