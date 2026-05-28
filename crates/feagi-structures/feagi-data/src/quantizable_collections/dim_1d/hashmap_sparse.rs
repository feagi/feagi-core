use ahash::AHashMap;
use feagi_ecs::collection::{FeagiECSCollectionDataLivesOnCPU, FeagiECSCollectionDataLivesOnDeviceBase};
use crate::quantizable_collections::shared_traits::{QuantizableLinearCollectionBase, QuantizableLinearCollectionCPUData};
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

impl<LIQ, Value> FeagiECSCollectionDataLivesOnDeviceBase for QuantizableLinearCollection1DHashmapSparse<LIQ, Value> where LIQ: QuantizedIndexCountTrait, Value: Clone, {}

impl<LIQ, Value> FeagiECSCollectionDataLivesOnCPU for QuantizableLinearCollection1DHashmapSparse<LIQ, Value> where LIQ: QuantizedIndexCountTrait, Value: Clone {}

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
