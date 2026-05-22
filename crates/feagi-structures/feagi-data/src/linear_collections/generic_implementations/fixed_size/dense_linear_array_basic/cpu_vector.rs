use core::marker::PhantomData;
use crate::feagi_ecs::component::FECSComponentBase;
use crate::linear_collections::generic_implementations::fixed_size::dense_linear_array_basic::shared_trait::DenseLinearArrayBasic;
use crate::linear_collections::traits_and_generics::{ECSLinearCollection, LinearCollection, QuantizableLinearCollection};
use crate::quantizable::base_types::QuantizedIndexCountTrait;
use crate::quantizable::FeagiQuantizedGeneric;



pub struct CPUVectorDenseLinearBasic<LinearIndexCountQuant, Value>
where
    LinearIndexCountQuant: QuantizedIndexCountTrait,
    Value: FECSComponentBase + FeagiQuantizedGeneric,
{
    vector: Vec<Value>,
    _phantom: PhantomData<LinearIndexCountQuant>,
}

impl<LinearIndexCountQuant, Value> ECSLinearCollection<LinearIndexCountQuant, Value> for CPUVectorDenseLinearBasic<LinearIndexCountQuant, Value> where LinearIndexCountQuant: QuantizedIndexCountTrait, Value: FECSComponentBase + FeagiQuantizedGeneric, {}

impl<LinearIndexCountQuant, Value> LinearCollection<LinearIndexCountQuant, Value> for CPUVectorDenseLinearBasic<LinearIndexCountQuant, Value>
where
    LinearIndexCountQuant: QuantizedIndexCountTrait,
    Value: FECSComponentBase + FeagiQuantizedGeneric,
{
    fn get_number_elements(&self) -> LinearIndexCountQuant {
        LinearIndexCountQuant::from_usize(self.vector.len())
    }
}

impl<LinearIndexCountQuant, Value> QuantizableLinearCollection<LinearIndexCountQuant, Value> for CPUVectorDenseLinearBasic<LinearIndexCountQuant, Value> where LinearIndexCountQuant: QuantizedIndexCountTrait, Value: FECSComponentBase + FeagiQuantizedGeneric, {}

impl<LinearIndexCountQuant, Value> DenseLinearArrayBasic<LinearIndexCountQuant, Value> for CPUVectorDenseLinearBasic<LinearIndexCountQuant, Value>
where
    LinearIndexCountQuant: QuantizedIndexCountTrait,
    Value: FECSComponentBase + FeagiQuantizedGeneric,
{

}
