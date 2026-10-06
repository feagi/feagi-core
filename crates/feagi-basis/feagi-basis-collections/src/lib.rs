
pub mod prelude {
    pub use super::FeagiBasisCollectionsError;
    pub use super::spatial_helpers::{SpatialCoordinate, SpatialDimensions};
    pub use super::bit_packed_bool::par_data::BitBatchParData;
    pub use super::generic_data::par_data::ParData;
    pub use super::create_wrapped_signed_integer_spatial;
    pub use super::create_wrapped_unsigned_integer_spatial_data;
    pub use super::create_wrapped_unsigned_integer_spatial_coordinate;
    pub use super::create_wrapped_unsigned_integer_spatial_dimensions;

}

pub mod generic_data;
pub mod bit_packed_bool;
pub mod spatial_helpers;
pub mod spatial_context;

mod feagi_basis_collections_error;


pub use feagi_basis_collections_error::FeagiBasisCollectionsError;

