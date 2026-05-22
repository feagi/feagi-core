//! An ECS Component is effectively just a pointer to data that can be easily combined for mass
//! parallel operations, but can also be used normally. Note in our implementation, these components
//! often have trait methods for quickly grabbing metadata, without needing to go to some
//! external device and waiting for it

use half::f16;
use crate::quantizable::base_types::decimal::custom_data_types::StorageF8;

// TODO Async vs Sync as tags, how would we handle the System though?

//region Components (individual Elements)

/// A tag denoting any ECS component at all. Does NOT indicate specific HW support
pub trait FECSComponentBase {}

/// A tag for any ECS Component data that is CPU compatible.
pub trait FECSComponentCPU: FECSComponentBase { }

/// A direct CPU compatible representation of data that is intended for another device type.
/// May not be as efficient space wise as the CPU native component, but can act as a high
/// performance transition state / cache. DOES NOT INSTANTLY WRITE DATA TO THE EXTERNAL DEVICE
/// WHEN YOU ACCESS IT VIA CPU
pub trait FECSComponentCPUBetween<ConnectingType: FECSComponentBase, CPUNativeType: FECSComponentCPU>: FECSComponentBase 
{
    fn copy_to_cpu_native(&self, cpu_native: &mut CPUNativeType);
    
    fn load_from_cpu_native(&mut self, cpu_native: &CPUNativeType);
}



//region Base Rust type Implementations

impl FECSComponentBase for u8 {}

impl FECSComponentBase for u16 {}

impl FECSComponentBase for u32 {}

impl FECSComponentBase for u64 {}

impl FECSComponentBase for i8 {}

impl FECSComponentBase for i16 {}

impl FECSComponentBase for i32 {}

impl FECSComponentBase for i64 {}

impl FECSComponentBase for StorageF8 {}

impl FECSComponentBase for f16 {}

impl FECSComponentBase for f32 {}

impl FECSComponentBase for f64 {}




impl FECSComponentCPU for u8 {}

impl FECSComponentCPU for u16 {}

impl FECSComponentCPU for u32 {}

impl FECSComponentCPU for u64 {}

impl FECSComponentCPU for i8 {}

impl FECSComponentCPU for i16 {}

impl FECSComponentCPU for i32 {}

impl FECSComponentCPU for i64 {}

impl FECSComponentCPU for StorageF8 {}

impl FECSComponentCPU for f16 {}

impl FECSComponentCPU for f32 {}

impl FECSComponentCPU for f64 {}

//endregion











