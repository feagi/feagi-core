use std::marker::PhantomData;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;
use crate::feagi_collection_error::{FeagiDataCollectionError, FeagiFailInvalidDimensions};
use crate::generic_data::par_data::{ParDataStore, ParDataStoreMut, ParDataStoreResizable};
use crate::prelude::SpatialDimensions;
use crate::spatial_indexing_structs::axis_order::{AxisOrderArray, AxisOrderEnum};
use crate::spatial_indexing_structs::spatial_index_context::SpatialContext;
use crate::spatial_indexing_structs::SpatialCoordinate;

pub trait SpatialParDataTrait<QLinear, QCoord, QDims, S, const NUM_DIMS: usize>: Sized
where
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
    QDims: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
    S: ParDataStore
{
    fn data_as_slice(&self) -> &[S::Elem];

    fn get_spatial_context(&self) -> &SpatialContext<QDims, NUM_DIMS>;

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
    where S: ParDataStoreMut;

    //region Impls

    fn get_linear_mut(&mut self, linear_index: &QLinear) -> Option<&mut S::Elem>
    where S: ParDataStoreMut
    {
        self.data_as_slice_mut().get_mut(linear_index.quant_to_usize())
    }

    fn get_mut(&mut self, coordinate: &SpatialCoordinate<QCoord, NUM_DIMS>) -> Option<&mut S::Elem>
    where S: ParDataStoreMut
    {
        let linear: QLinear = self.get_spatial_context().coordinate_to_linear(&coordinate);
        self.get_linear_mut(&linear)
    }


    //endregion


    //endregion
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "QDims: ::serde::de::DeserializeOwned"))]
pub struct SpatialParDataOwning<QLinear, QCoord, QDims, S, D, const NUM_DIMS: usize>
where
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
    QDims: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
    S: ParDataStore<Elem=D> + DeserializeOwned + 'static,
{
    context: SpatialContext<QDims, NUM_DIMS>,
    data: S,
    _p: core::marker::PhantomData<(QLinear, QCoord)>
}

impl <QLinear, QCoord, QDims, S, D, const NUM_DIMS: usize>
SpatialParDataOwning<QLinear, QCoord, QDims, S, D, NUM_DIMS>
where
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
    QDims: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
    S: ParDataStore<Elem=D> + DeserializeOwned + 'static,
{
    pub fn new(dimensions: SpatialDimensions<QDims, NUM_DIMS>, axis_order: AxisOrderEnum, data: S) -> Result<Self, FeagiDataCollectionError> {
        if dimensions.spatial_element_count() != data.len() {
            return Err(FeagiFailInvalidDimensions::new("Dimensions are invalid for the given data size").into())
        }
        let context = SpatialContext::new(dimensions, axis_order);
        Ok(Self {
            context,
            data,
            _p: PhantomData
        })
    }

    pub fn new_with_element_default(dimensions: SpatialDimensions<QDims, NUM_DIMS>, axis_order: AxisOrderEnum) -> Self
    where
        S: ParDataStoreResizable,
        S::Elem: Default
    {
        let data = S::new_from_element_default(dimensions.spatial_element_count());
        let context = SpatialContext::new(dimensions, axis_order);
        Self {
            context,
            data,
            _p: PhantomData
        }
    }

    pub fn new_with_element_clone(dimensions: SpatialDimensions<QDims, NUM_DIMS>, axis_order: AxisOrderEnum, source_element: S::Elem) -> Self
    where
        S: ParDataStoreResizable,
        S::Elem: Clone
    {
        let data = S::new_from_element_clonable(dimensions.spatial_element_count(), source_element);
        let context = SpatialContext::new(dimensions, axis_order);
        Self {
            context,
            data,
            _p: PhantomData
        }
    }
}

impl <QLinear, QCoord, QDims, S, D, const NUM_DIMS: usize>
SpatialParDataTrait<QLinear, QCoord, QDims, S, NUM_DIMS> for
SpatialParDataOwning<QLinear, QCoord, QDims, S, D, NUM_DIMS>
where
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
    QDims: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
    S: ParDataStore<Elem=D> + DeserializeOwned + 'static,
{
    fn data_as_slice(&self) -> &[S::Elem] {
        self.data.store_as_slice()
    }

    fn get_spatial_context(&self) -> &SpatialContext<QDims, NUM_DIMS> {
        &self.context
    }

    fn data_as_slice_mut(&mut self) -> &mut [S::Elem]
    where
        S: ParDataStoreMut
    {
        self.data.store_as_mut_slice()
    }
}