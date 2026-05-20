pub mod spatial_unsigned_4d;
mod wrapper_cpu;

pub use spatial_unsigned_4d::{
    SpatialDimension4DTrait,
    SpatialUnsignedBase4DTrait,
    SpatialUnsignedCoordinate4DTrait,
};

pub use wrapper_cpu::{SpatialUnsignedCoordinate4D, SpatialUnsignedDimension4D};