//! Defines the device type the data resides in. CPU is always supported, but other crates may add
//! their own devices

#[doc(hidden)]
/// Shared trait among all Feagi ECS Device traits
pub trait FeagiECSCollectionOnDevice {}

/// Defines that the collection data lives on the CPU / RAM
pub trait FeagiECSCollectionOnCPU: FeagiECSCollectionOnDevice {}

// Other devices should be defined here but feature gated
