/// Defines where data is being stored or used, as well as any method
#[repr(C)]
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum HardwareResidence
{
    /// Standard CPU / RAM system
    CPUStandard,
    // GPU with 16 byte stride via WGPU
    WGPUStandard,
}