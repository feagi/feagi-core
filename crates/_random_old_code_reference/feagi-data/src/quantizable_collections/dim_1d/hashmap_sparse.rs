use ahash::AHashMap;
use feagi_pdi::tag_device::{PDITagCPU, PDITagGenericDevice};
use crate::quantizable_collections::shared_traits::{QuantizableLinearCollectionBase, QuantizableLinearCollectionCPUData, QuantizableLinearCollectionCPUIterWithIndex, QuantizableLinearCollectionCPUSparse};
use crate::quantizable_linear::base_types::QuantizedIndexCountTrait;


// TODO some way to free memory, resize?

pub struct QuantizableLinearCollection1DHashmapSparse<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    values: AHashMap<LIQ, Value>,
    max_linear_index: LIQ
}


impl<LIQ, Value> QuantizableLinearCollection1DHashmapSparse<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    pub fn new(max_linear_index: LIQ) -> Self {
        QuantizableLinearCollection1DHashmapSparse{
            values: AHashMap::new(),
            max_linear_index
        }
    }

    /// Mainly used to make more unique wrapper systems
    pub fn internal_get_values_mut(&mut self) -> &mut AHashMap<LIQ, Value>{
        &mut self.values
    }

    /// Mainly used to make more unique wrapper systems
    pub fn internal_get_max_linear_index_mut(&mut self) -> &mut LIQ {
        &mut self.max_linear_index
    }
}


impl<LIQ, Value> QuantizableLinearCollectionBase<LIQ, Value> for QuantizableLinearCollection1DHashmapSparse<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn max_linear_index(&self) -> LIQ {
        self.max_linear_index
    }
}


//region PDI CPU Access

impl<LIQ, Value> QuantizableLinearCollectionCPUData<LIQ, Value> for QuantizableLinearCollection1DHashmapSparse<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn try_get_value(&self, index: LIQ) -> Option<&Value> {
        self.values.get(&index)
    }

    fn try_get_value_mut(&mut self, index: LIQ) -> Option<&mut Value> {
        self.values.get_mut(&index)
    }

    // The unchecked are the same

    fn get_unchecked_value(&self, index: LIQ) -> &Value {
        self.values.get(&index).unwrap()
    }

    fn get_unchecked_value_mut(&mut self, index: LIQ) -> &mut Value {
        self.values.get_mut(&index).unwrap()
    }
}

impl<LIQ, Value> QuantizableLinearCollectionCPUIterWithIndex<LIQ, Value> for QuantizableLinearCollection1DHashmapSparse<LIQ, Value>
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
            .map(|(index, value)| (*index, value))
    }

    fn iter_mut_with_index<'a>(&'a mut self) -> impl Iterator<Item=(LIQ, &'a mut Value)>
    where
        Value: 'a
    {
        self.values
            .iter_mut()
            .map(|(index, value)| (*index, value))
    }
}

impl<LIQ, Value> QuantizableLinearCollectionCPUSparse<LIQ, Value> for QuantizableLinearCollection1DHashmapSparse<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn insert_value_at_index(&mut self, index: LIQ, value: Value) -> Option<Value> {
        self.values.insert(index, value)
    }

    fn remove_value_at_index(&mut self, index: LIQ) -> Option<Value> {
        self.values.remove(&index)
    }
}

//endregion


//region PDI TAgging

impl<LIQ, Value> PDITagGenericDevice for QuantizableLinearCollection1DHashmapSparse<LIQ, Value> where LIQ: QuantizedIndexCountTrait, Value: Clone, {}

impl<LIQ, Value> PDITagCPU for QuantizableLinearCollection1DHashmapSparse<LIQ, Value> where LIQ: QuantizedIndexCountTrait, Value: Clone, {}

//endregion
