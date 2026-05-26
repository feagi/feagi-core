//! Defines the type of memory allocation used to hold the data


#[doc(hidden)]
/// Shared trait among all Feagi ECS Memory traits
pub trait FeagiECSMemoryBase {}

/// Memory is assigned at compile time
pub trait FeagiECSMemoryStatic: FeagiECSMemoryBase {}

#[cfg(feature = "alloc")]
/// Structure cannot be resized during runtime but can be "resized" by recreating it
pub trait FeagiECSMemorySized: FeagiECSMemoryBase {}

#[cfg(feature = "alloc")]
/// Structure can be resized during runtime (like pushing to a vector)
pub trait FeagiECSMemoryResizable: FeagiECSMemoryBase {}