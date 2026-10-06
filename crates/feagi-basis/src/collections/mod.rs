
pub mod prelude {
    pub use super::CollectionsError;
    pub use super::spatial_helpers::{SpatialCoordinate, SpatialDimensions};
    pub use super::bit_packed_bool::par_data::BitBatchParData;
    pub use super::par_data::par_data::ParData;
    pub use super::create_wrapped_signed_integer_spatial;
    pub use super::create_wrapped_unsigned_integer_spatial_data;
    pub use super::create_wrapped_unsigned_integer_spatial_coordinate;
    pub use super::create_wrapped_unsigned_integer_spatial_dimensions;

}

pub use crate::{
    create_wrapped_signed_integer_spatial, create_wrapped_unsigned_integer_spatial_coordinate,
    create_wrapped_unsigned_integer_spatial_data, create_wrapped_unsigned_integer_spatial_dimensions,
};

pub mod par_data;
pub mod bit_packed_bool;
pub mod spatial_helpers;
pub mod spatial_context;

mod collections_error;
pub mod generic_collections;

pub use collections_error::CollectionsError;

