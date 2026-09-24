pub mod generic_data;
pub mod feagi_collection_error;
pub mod bit_packed_bool;
pub mod spatial_indexing_structs;

pub mod prelude {
    pub use super::feagi_collection_error::FeagiDataCollectionError;
    pub use super::spatial_indexing_structs::{SpatialCoordinate, SpatialDimensions};
}
