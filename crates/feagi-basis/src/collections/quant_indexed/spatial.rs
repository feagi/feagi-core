use crate::collections::quant_indexed::quant_indexed::{QuantContiguousStore, QuantContiguousStoreMut, QuantContiguousStoreResizable};
use crate::collections::prelude::SpatialDimensions;
use crate::collections::spatial_context::spatial_context_error::SpatialContextError;
use crate::collections::spatial_helpers::axis_order::{AxisOrder, AxisOrderArray};
use crate::collections::spatial_helpers::spatial_index_context::SpatialHelper;
use crate::collections::spatial_helpers::SpatialCoordinate;
use crate::collections::spatial_helpers::SpatialHelperError;
use crate::quantization::prelude::QuantizedUnsignedIntegerTrait;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

pub trait SpatialQuantIndexedTrait<QLinear, QCoord, QDims, AxisOrderType, S, const NUM_DIMS: usize>: Sized
where
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
    QDims: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
    AxisOrderType: AxisOrder<NUM_DIMS> + 'static,
    S: QuantContiguousStore,
{
    fn data_as_slice(&self) -> &[S::Elem];

    fn get_spatial_context(&self) -> &SpatialHelper<QDims, AxisOrderType, NUM_DIMS>;

    //region Impls
    fn get_dimensions(&self) -> &SpatialDimensions<QDims, NUM_DIMS> {
        self.get_spatial_context().get_dimensions()
    }

    fn get_axis_order_array(&self) -> &AxisOrderArray<NUM_DIMS> {
        self.get_spatial_context().get_axis_order_array()
    }

    fn get_linear(&self, linear_index: &QLinear) -> Option<&S::Elem> {
        self.data_as_slice().get(linear_index.quant_to_usize())
    }

    fn get(&self, coordinate: &SpatialCoordinate<QCoord, NUM_DIMS>) -> Option<&S::Elem> {
        let linear: QLinear = self.get_spatial_context().coordinate_to_linear(&coordinate);
        self.get_linear(&linear)
    }

    //endregion

    //region fixed size Mutable

    fn data_as_slice_mut(&mut self) -> &mut [S::Elem]
    where
        S: QuantContiguousStoreMut;

    //region Impls

    fn get_linear_mut(&mut self, linear_index: &QLinear) -> Option<&mut S::Elem>
    where
        S: QuantContiguousStoreMut,
    {
        self.data_as_slice_mut().get_mut(linear_index.quant_to_usize())
    }

    fn get_mut(&mut self, coordinate: &SpatialCoordinate<QCoord, NUM_DIMS>) -> Option<&mut S::Elem>
    where
        S: QuantContiguousStoreMut,
    {
        let linear: QLinear = self.get_spatial_context().coordinate_to_linear(&coordinate);
        self.get_linear_mut(&linear)
    }

    //endregion

    //endregion
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "QDims: ::serde::de::DeserializeOwned"))]
pub struct SpatialQuantIndexedOwning<QLinear, QCoord, QDims, AxisOrderType, S, D, const NUM_DIMS: usize>
where
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
    QDims: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
    AxisOrderType: AxisOrder<NUM_DIMS>,
    S: QuantContiguousStore<Elem = D> + DeserializeOwned + 'static,
{
    context: SpatialHelper<QDims, AxisOrderType, NUM_DIMS>,
    data: S,
    _p: core::marker::PhantomData<(QLinear, QCoord)>,
}

impl<QLinear, QCoord, QDims, AxisOrderType, S, D, const NUM_DIMS: usize> SpatialQuantIndexedOwning<QLinear, QCoord, QDims, AxisOrderType, S, D, NUM_DIMS>
where
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
    QDims: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
    AxisOrderType: AxisOrder<NUM_DIMS>,
    S: QuantContiguousStore<Elem = D> + DeserializeOwned + 'static,
{
    pub fn new(dimensions: SpatialDimensions<QDims, NUM_DIMS>, data: S) -> Result<Self, SpatialHelperError> {
        if dimensions.spatial_element_count() != data.len() {
            return Err(SpatialHelperError::InvalidContext(SpatialContextError::DimensionsCannotBeZero));
        }
        let context = SpatialHelper::new(dimensions);
        Ok(Self {
            context,
            data,
            _p: PhantomData,
        })
    }

    pub fn new_with_element_default(dimensions: SpatialDimensions<QDims, NUM_DIMS>) -> Self
    where
        S: QuantContiguousStoreResizable,
        S::Elem: Default,
    {
        let data = S::new_from_element_default(dimensions.spatial_element_count());
        let context = SpatialHelper::new(dimensions);
        Self {
            context,
            data,
            _p: PhantomData,
        }
    }

    pub fn new_with_element_clone(dimensions: SpatialDimensions<QDims, NUM_DIMS>, source_element: S::Elem) -> Self
    where
        S: QuantContiguousStoreResizable,
        S::Elem: Clone,
    {
        let data = S::new_from_element_clonable(dimensions.spatial_element_count(), source_element);
        let context = SpatialHelper::new(dimensions);
        Self {
            context,
            data,
            _p: PhantomData,
        }
    }
}

impl<QLinear, QCoord, QDims, AxisOrderType, S, D, const NUM_DIMS: usize> SpatialQuantIndexedTrait<QLinear, QCoord, QDims, AxisOrderType, S, NUM_DIMS>
    for SpatialQuantIndexedOwning<QLinear, QCoord, QDims, AxisOrderType, S, D, NUM_DIMS>
where
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
    QDims: QuantizedUnsignedIntegerTrait<QuantType = QLinear::QuantType>,
    AxisOrderType: AxisOrder<NUM_DIMS> + 'static,
    S: QuantContiguousStore<Elem = D> + DeserializeOwned + 'static,
{
    fn data_as_slice(&self) -> &[S::Elem] {
        self.data.store_as_slice()
    }

    fn get_spatial_context(&self) -> &SpatialHelper<QDims, AxisOrderType, NUM_DIMS> {
        &self.context
    }

    fn data_as_slice_mut(&mut self) -> &mut [S::Elem]
    where
        S: QuantContiguousStoreMut,
    {
        self.data.store_as_mut_slice()
    }
}
