use crate::feagi_ecs::component::FECSComponentBase;
use crate::LinearIndexCountType;
use crate::quantizable::base_types::QuantizedIndexCountTrait;
use crate::quantizable::FeagiQuantizedGeneric;


//region Common

// Note: Yes, these are not very detailed. We only have basic metadata guarenteed

pub trait LinearCollection<LinearIndexCount, ElementType>
where
    LinearIndexCount: LinearIndexCountType,
{
    fn get_number_elements(&self) -> LinearIndexCount;
}

/// Defines a linear collection that can be easily resized at runtime (like a Vector) without
/// needing to be recreated
pub trait LinearCollectionResizable<LinearIndexCount, ElementType>:
LinearCollection<LinearIndexCount, ElementType>
where
    LinearIndexCount: LinearIndexCountType,
{
    fn get_number_elements(&self) -> LinearIndexCount;
}

//endregion

//region Quantizable

pub trait QuantizableLinearCollection<LinearIndexCountQuant, Value>:
LinearCollection<LinearIndexCountQuant, Value>
where
    LinearIndexCountQuant: QuantizedIndexCountTrait,
    Value: FeagiQuantizedGeneric
{

}

//endregion

//region ECS

/// Base trait for all Quantizable ECS supporting Collections
pub trait ECSLinearCollection<LinearIndexCountQuant, Value>:
LinearCollection<LinearIndexCountQuant, Value>
where
    LinearIndexCountQuant: LinearIndexCountType,
    Value: FECSComponentBase,
{

}

/// A tag trait designate any type of collection that is resizable at runtime (Vectors, Hashmaps, etc).
/// This is used to prevent usage of components in systems that expect easy runtime resizing. Note
/// that just because a struct does not implement this trait means that its impossible to resize,
/// however it may require a special System to create a new instance and destroy the old one
pub trait ECSLinearCollectionResizable<LinearIndexCount, Value>:
ECSLinearCollection<LinearIndexCount, Value>
where
    LinearIndexCount: LinearIndexCountType,
    Value: FECSComponentBase,
{

}

pub trait ECSLinearCollectionWithCPUCacheCopy<LinearIndexCount, Value, CPUCacheCopy>:
ECSLinearCollection<LinearIndexCount, Value>
+ LinearCollectionResizable<LinearIndexCount, Value>
where
    LinearIndexCount: LinearIndexCountType,
    Value: FECSComponentBase,
    CPUCacheCopy: ECSLinearCollection<LinearIndexCount, Value>
{

}

#[cfg(feature = "support_wgpu")]
/// A tag for any ECS Component data that lives in VRAM via WGPU
pub trait ECSLinearCollectionWGPU<LinearIndexCount, Value>:
ECSLinearCollection<LinearIndexCount, Value>
where
    LinearIndexCount: LinearIndexCountType,
    Value: FECSComponentWGPUBase,
{
    // TODO we likely need some common methods for pulling device reference and stuff
}

//endregion