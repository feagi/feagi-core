use crate::feagi_ecs::component::FECSComponentBase;
use crate::collection_traits::traits_and_generics::{ECSLinearCollection, QuantizableLinearCollection};
use crate::quantizable::base_types::QuantizedIndexCountTrait;
use crate::quantizable::FeagiQuantizedGeneric;

pub trait DenseLinearArrayBasic<LinearIndexCountQuant, Value>:
ECSLinearCollection<LinearIndexCountQuant, Value>
+ QuantizableLinearCollection<LinearIndexCountQuant, Value>
where
    LinearIndexCountQuant: QuantizedIndexCountTrait,
    Value: FECSComponentBase + FeagiQuantizedGeneric,
{

}

pub trait DenseLinearArrayBasicCPUAccess<LinearIndexCountQuant, Value>:
DenseLinearArrayBasic<LinearIndexCountQuant, Value>
where
    LinearIndexCountQuant: QuantizedIndexCountTrait,
    Value: FECSComponentBase + FeagiQuantizedGeneric,
{
    fn get_slice(&self) -> &[Value];
    fn get_slice_mut(&mut self) -> &mut [Value];
}