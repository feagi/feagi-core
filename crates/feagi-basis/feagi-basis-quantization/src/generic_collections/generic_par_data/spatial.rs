use crate::generic_collections::generic_par_data::linear::ParDataVector;
use crate::values::quantizable::QuantizedUnsignedIntegerTrait;
use crate::values::spatial::unsigned_integer::UnsignedIntegerSpatialWrappedDimensionsTrait;

pub struct SpatialCollectionVector<QI, D, Dims, const DIM_COUNT: usize>
where
    QI: QuantizedUnsignedIntegerTrait,
    D: Clone,
    Dims: UnsignedIntegerSpatialWrappedDimensionsTrait<QI::QuantType, DIM_COUNT>,
{
    dimensions: Dims,
    data: ParDataVector<QI, D>,
}

impl<QI, D, Dims, const DIM_COUNT: usize> SpatialCollectionVector<QI, D, Dims, DIM_COUNT>
where
    QI: QuantizedUnsignedIntegerTrait,
    D: Clone,
    Dims: UnsignedIntegerSpatialWrappedDimensionsTrait<QI::QuantType, DIM_COUNT>,
{
    pub fn new(dimensions: Dims, data: ParDataVector<QI, D>) -> Self {
        Self { dimensions, data }
    }

    pub fn get_count(&self) -> Dims::LinearCount {
        self.dimensions.number_contained_elements()
    }

    pub fn dimensions(&self) -> &Dims {
        &self.dimensions
    }

    pub fn get_by_linear(&self, linear_index: &Dims::LinearIndex) -> Option<&D> {
        todo!()
    }

    pub fn get_by_linear_mut(&self, linear_index: &Dims::LinearIndex) -> Option<&mut D> {
        todo!()
    }

    pub fn get_by_coordinate(&self, coordinate: &Dims::Coordinate) -> Option<&D> {
        todo!()
    }

    pub fn get_by_coordinate_mut(&self, coordinate: &Dims::Coordinate) -> Option<&mut D> {
        todo!()
    }
}
