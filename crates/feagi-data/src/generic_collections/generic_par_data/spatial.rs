use crate::values::quantizable::QuantizedUnsignedIntegerTrait;
use crate::values::spatial::unsigned_integer::UnsignedIntegerSpatialWrappedDimensionsTrait;

pub struct SpatialLinearCollectionTranslator<'a, QI, D, Dims, const DIM_COUNT: usize>
where
    QI: QuantizedUnsignedIntegerTrait,
    D: Clone,
    Dims: UnsignedIntegerSpatialWrappedDimensionsTrait<QI::QuantType, DIM_COUNT>,
{

}