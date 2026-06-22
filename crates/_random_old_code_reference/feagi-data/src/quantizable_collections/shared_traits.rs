//! Base traits for linear quantizable collections with some PDI components mixed in.
//! As this is base level, we do not even have FeagiError support, so we use Options for now
// TODO maybe this should be moved up? 

use feagi_pdi::tag_device::{PDITagCPU, PDITagGenericDevice};
use crate::quantizable_linear::base_types::QuantizedIndexCountTrait;


pub trait QuantizableLinearCollectionBase<LIQ, Value>:
PDITagGenericDevice
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn max_linear_index(&self) -> LIQ;
}

/// Defines that the data is accessible via Sync CPU functions
pub trait QuantizableLinearCollectionCPUData<LIQ, Value>:
QuantizableLinearCollectionBase<LIQ, Value>
+ PDITagCPU
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn try_get_value(&self, index: LIQ) -> Option<&Value>;

    fn try_get_value_mut(&mut self, index: LIQ) -> Option<&mut Value>;

    fn get_unchecked_value(&self, index: LIQ) -> &Value;

    fn get_unchecked_value_mut(&mut self, index: LIQ) -> &mut Value;
}

/// Defines that the data is accessible via Sync CPU functions
pub trait QuantizableLinearCollectionCPUIterWithIndex<LIQ, Value>:
QuantizableLinearCollectionCPUData<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    fn iter_with_index<'a>(&'a self) -> impl Iterator<Item = (LIQ, &'a Value)> where Value: 'a;

    fn iter_mut_with_index<'a>(&'a mut self) -> impl Iterator<Item = (LIQ, &'a mut Value)> where Value: 'a;
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

/// Structure is sparse, and can add / remove items in a Sync manner
pub trait QuantizableLinearCollectionCPUSparse<LIQ, Value>:
QuantizableLinearCollectionCPUData<LIQ, Value>
where
    LIQ: QuantizedIndexCountTrait,
    Value: Clone
{
    /// Tries inserting a value at an index. Returns any value being bumped out if it is.
    fn insert_value_at_index(&mut self, index: LIQ, value: Value) -> Option<Value>;
    
    /// Tries to remove a value at a given index. Returns any value that was removed if there was one there
    fn remove_value_at_index(&mut self, index: LIQ) -> Option<Value>;
}
