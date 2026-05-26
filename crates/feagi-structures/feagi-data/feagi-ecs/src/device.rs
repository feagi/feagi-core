//! Defines the device type the data resides in. CPU is always supported, but other crates may add
//! their own devices

#[doc(hidden)]
/// Shared trait among all Feagi ECS Device traits
pub trait FeagiECSDeviceBase {}

/// Defines that data lives on the CPU / RAM
pub trait FeagiECSDeviceCPU: FeagiECSDeviceBase {}