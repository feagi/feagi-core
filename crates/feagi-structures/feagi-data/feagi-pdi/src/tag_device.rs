//! Just a set of traits that make it easy to tag something PDI related as to belong to a certain
//! device without having to create multiple traits

// NOTE: Do not feature gate the various hardware keys here
/// Allows for identification of supported device at runtime
#[derive(Debug, PartialOrd, PartialEq, Clone, Copy, Hash)]
pub enum PDIDeviceClass {
    Unknown, // You really shouldn't be getting this
    CPU,
    WGPU
}

/// Root trait for Tag Devices
pub trait PDITagGenericDevice {
    const DEVICE_CLASS: PDIDeviceClass = PDIDeviceClass::Unknown;
}

/// Denotes that the PDI struct works on the CPU, and thus should have Rust accessible members,
/// functions, and implementations for data reading, writing, and manipulation
pub trait PDITagCPU: PDITagGenericDevice {
    const DEVICE_CLASS: PDIDeviceClass = PDIDeviceClass::CPU;
}

// Other devices should be defined here but feature gated
