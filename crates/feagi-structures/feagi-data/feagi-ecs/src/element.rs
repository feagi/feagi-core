//! Defines the device type the element data resides in. CPU is always supported, but other crates may add
//! their own devices

// NOTE: We cannot realistically handle moving individual elements between devices. That should
// only be done at the collection level.

#[doc(hidden)]
/// Shared trait among all Feagi ECS Device traits
pub trait FeagiECSElementOnDevice {}

/// Defines that the element data lives on the CPU / RAM
pub trait FeagiECSElementOnCPU: FeagiECSElementOnDevice {}

// Other devices should be defined here but feature gated

