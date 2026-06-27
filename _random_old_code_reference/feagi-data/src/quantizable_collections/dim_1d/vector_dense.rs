use feagi_pdi::tag_device::{PDITagCPU, PDITagGenericDevice};
use crate::quantizable_collections::shared_traits::{QuantizableLinearCollectionAsSlice, QuantizableLinearCollectionBase, QuantizableLinearCollectionCPUData, QuantizableLinearCollectionCPUIterWithIndex};
use crate::quantizable_linear::base_types::QuantizedIndexCountTrait;

pub struct QuantizableLinearCollection1DVectorDense<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    values: Vec<Value>,
    max_linear_index: LIQ
}

impl<LIQ, Value> QuantizableLinearCollection1DVectorDense<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    pub fn new_uniform(max_linear_index: LIQ, filling_value: Value) -> Self {
        let values = vec![filling_value; max_linear_index.to_usize()];

        Self {
            values,
            max_linear_index,
        }
    }

    pub fn new_with_iter<I>(max_linear_index: LIQ, iterator: I) -> Self
    where
        I: IntoIterator<Item = Value>,
    {
        // TODO debug checks
        /*
        let expected_len = max_linear_index.to_usize();
        let values: Vec<Value> = iterator.into_iter().collect();

        assert_eq!(
            values.len(),
            expected_len,
            "iterator must produce exactly one value per linear element",
        );

         */

        let values: Vec<Value> = iterator.into_iter().collect();

        Self {
            values,
            max_linear_index,
        }
    }

    /// Mainly used to make more unique wrapper systems
    pub fn internal_get_values_mut(&mut self) -> &mut Vec<Value> {
        &mut self.values
    }

    /// Mainly used to make more unique wrapper systems
    pub fn internal_get_max_linear_index_mut(&mut self) -> &mut LIQ {
        &mut self.max_linear_index
    }
}

impl<LIQ, Value> QuantizableLinearCollectionBase<LIQ, Value> for QuantizableLinearCollection1DVectorDense<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn max_linear_index(&self) -> LIQ {
        self.max_linear_index
    }
}


//region PDI CPU Access


impl<LIQ, Value> QuantizableLinearCollectionCPUData<LIQ, Value> for QuantizableLinearCollection1DVectorDense<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn try_get_value(&self, index: LIQ) -> Option<&Value> {
        self.values.get(index.to_usize())
    }

    fn try_get_value_mut(&mut self, index: LIQ) -> Option<&mut Value> {
        self.values.get_mut(index.to_usize())
    }

    fn get_unchecked_value(&self, index: LIQ) -> &Value {
        &self.values[index.to_usize()]
    }

    fn get_unchecked_value_mut(&mut self, index: LIQ) -> &mut Value {
        &mut self.values[index.to_usize()]
    }
}

impl<LIQ, Value> QuantizableLinearCollectionAsSlice<LIQ, Value> for QuantizableLinearCollection1DVectorDense<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn get_values_slice(&self) -> &[Value] {
        self.values.as_slice()
    }

    fn get_values_slice_mut(&mut self) -> &mut [Value] {
        self.values.as_mut_slice()
    }
}

impl<LIQ, Value> QuantizableLinearCollectionCPUIterWithIndex<LIQ, Value> for QuantizableLinearCollection1DVectorDense<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn iter_with_index<'a>(&'a self) -> impl Iterator<Item=(LIQ, &'a Value)>
    where
        Value: 'a
    {
        self.values
            .iter()
            .enumerate()
            .map(|(index, value)| (LIQ::from_usize_unchecked(index), value))
    }

    fn iter_mut_with_index<'a>(&'a mut self) -> impl Iterator<Item=(LIQ, &'a mut Value)>
    where
        Value: 'a
    {
        self.values
            .iter_mut()
            .enumerate()
            .map(|(index, value)| (LIQ::from_usize_unchecked(index), value))
    }
}

//endregion


//region PDI TAgging

impl<LIQ, Value> PDITagGenericDevice for QuantizableLinearCollection1DVectorDense<LIQ, Value> where LIQ: QuantizedIndexCountTrait, Value: Clone, {}

impl<LIQ, Value> PDITagCPU for QuantizableLinearCollection1DVectorDense<LIQ, Value> where LIQ: QuantizedIndexCountTrait, Value: Clone, {}

//endregion
