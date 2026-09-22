use std::marker::PhantomData;
use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;
use crate::generic_data::par_data::{ParDataStore, ParDataStoreMut};
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

pub struct SpatialParDataOwning<QLinear, QCoord, QDims, S, const NUM_DIMS: usize>
where
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
    QDims: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
    S: ParDataStore + 'static
{
    context: SpatialContext<QDims, NUM_DIMS>,
    data: S,
    _p: core::marker::PhantomData<(QLinear, QCoord)>
}

impl <QLinear, QCoord, QDims, S, const NUM_DIMS: usize>
SpatialParDataOwning<QLinear, QCoord, QDims, S, NUM_DIMS>
where
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
    QDims: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
    S: ParDataStore + 'static
{
    pub fn new(dimensions: SpatialDimensions<QDims, NUM_DIMS>, axis_order: AxisOrderEnum, data: S) -> Result<Self, ()> {
        if dimensions.spatial_element_count() != data.len() {
            return Err(())
        }
        let context = SpatialContext::new(dimensions, axis_order);
        Ok(Self {
            context,
            data,
            _p: PhantomData
        })
    }

    // TODO new default? new with cloning?
}

impl <QLinear, QCoord, QDims, S, const NUM_DIMS: usize>
SpatialParDataTrait<QLinear, QCoord, QDims, S, NUM_DIMS> for
SpatialParDataOwning<QLinear, QCoord, QDims, S, NUM_DIMS>
where
    QLinear: QuantizedUnsignedIntegerTrait,
    QCoord: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
    QDims: QuantizedUnsignedIntegerTrait<QuantType=QLinear::QuantType>,
    S: ParDataStore + 'static
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