

#[cfg(feature = "support_wgpu")]
/// A tag for any ECS Component data that lives on the GPU via WGPU. This is pretty
/// much just any structs that are made of u32, f32, and i32 that have a size multiple of 16 bytes.
/// Note that technically other quantization levels can be sent over but it isnt recommended
pub trait FECSComponentWGPUBase { }
