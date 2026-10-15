pub use coordinate::SpatialCoordinate;
pub use dimensions::SpatialDimensions;
pub use spatial_indexing_error::SpatialHelperError;
pub use stride::SpatialStride;

mod spatial_indexing_error;

pub mod axis_order;

mod coordinate;
mod dimensions;
pub mod spatial_index_context;
mod stride;
