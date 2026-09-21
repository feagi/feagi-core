
pub use dimensions::SpatialDimensions;
pub use stride::SpatialStride;
pub use coordinate::SpatialCoordinate;
pub use owning_spatial_indexing::{OwningSpatialIndexing, SpatialCoordinateIter};

pub mod axis_order;

mod dimensions;
mod stride;
mod coordinate;
mod owning_spatial_indexing;


