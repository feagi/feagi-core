pub mod generic_data;
pub mod bit_packed_bool;
pub mod spatial_indexing_structs;

mod feagi_basis_collections_error;

pub use feagi_basis_collections_error::FeagiBasisCollectionsError;

pub mod prelude {
    pub use super::FeagiBasisCollectionsError;
    pub use super::spatial_indexing_structs::{SpatialCoordinate, SpatialDimensions};
}
