use crate::quantizable_linear::base_types::QuantizedIndexCountTrait;


pub trait QuantizableLinearCollectionBase<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn max_linear_index(&self) -> LIQ;
}

pub trait QuantizableLinearCollectionCPUData<LIQ, Value>:
QuantizableLinearCollectionBase<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn try_get_value(&self, index: LIQ) -> Option<&Value>;

    fn try_get_value_mut(&mut self, index: LIQ) -> Option<&mut Value>;

    fn get_unchecked_value(&self, index: LIQ) -> &Value;

    fn get_unchecked_value_mut(&mut self, index: LIQ) -> &mut Value;
}


/// Data is dense and accessible
pub trait QuantizableLinearCollectionAsSlice<LIQ, Value>:
QuantizableLinearCollectionCPUData<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn get_values_slice(&self) -> &[Value];

    fn get_values_slice_mut(&mut self) -> &mut [Value];
}