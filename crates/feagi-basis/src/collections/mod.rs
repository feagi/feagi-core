pub mod prelude {
    pub use super::bit_packed_bool::par_data::BitBatchParData;
    pub use super::create_wrapped_signed_integer_spatial;
    pub use super::create_wrapped_unsigned_integer_spatial_coordinate;
    pub use super::create_wrapped_unsigned_integer_spatial_data;
    pub use super::create_wrapped_unsigned_integer_spatial_dimensions;
    pub use super::quant_indexed::quant_indexed::QuantIndexed;
    pub use super::spatial_helpers::{SpatialCoordinate, SpatialDimensions};
    pub use super::CollectionsError;
}

pub use crate::{
    create_wrapped_signed_integer_spatial, create_wrapped_unsigned_integer_spatial_coordinate, create_wrapped_unsigned_integer_spatial_data,
    create_wrapped_unsigned_integer_spatial_dimensions,
};

pub mod bit_packed_bool;
pub mod quant_indexed;
pub mod spatial_context;
pub mod spatial_helpers;

mod collections_error;
pub mod generic_collections;

pub use collections_error::CollectionsError;
