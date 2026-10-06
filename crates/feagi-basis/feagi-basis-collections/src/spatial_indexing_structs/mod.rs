
pub use dimensions::SpatialDimensions;
pub use stride::SpatialStride;
pub use coordinate::SpatialCoordinate;
pub use spatial_indexing_error::SpatialIndexingError;

mod spatial_indexing_error;

pub mod axis_order;

mod dimensions;
mod stride;
mod coordinate;
pub mod spatial_index_context;

