use crate::spatial_indexing_structs::axis_order::SpatialAxisOrder;
use crate::spatial_indexing_structs::coordinate::SpatialCoordinate;
use crate::spatial_indexing_structs::dimensions::SpatialDimensions;
use crate::spatial_indexing_structs::stride::SpatialStride;
use feagi_basis_quantization::prelude::{QuantizedUnsignedIntegerTrait, QuantizedUnsignedIntegerUnwrappedTrait};

pub trait SpatialIndexingHelper<QI: QuantizedUnsignedIntegerUnwrappedTrait<QuantType=QI>, const NUM_DIMS: usize> {
    const AXIS_ORDER: SpatialAxisOrder<QI, NUM_DIMS>;

    type LinearIndex: QuantizedUnsignedIntegerTrait<QuantType=QI>;

    type CoordinateQuant: QuantizedUnsignedIntegerTrait<QuantType=QI>;

    type DimensionsQuant: QuantizedUnsignedIntegerTrait<QuantType=QI>;

    /// Get the current in use dimensions
    fn get_dimensions(&self) -> &SpatialDimensions<Self::DimensionsQuant, NUM_DIMS>;

    /// Update dimensions, including updating the stride as well
    fn update_dimensions(&mut self, new_dims: SpatialDimensions<Self::DimensionsQuant, NUM_DIMS>) {
        self._get_stride_mut().update_stride(&new_dims, &Self::AXIS_ORDER);
        *self._get_dimensions_mut() = new_dims;
    }

    // NOTE: Yes, this is a bit ugly here, but the best way above to handle this

    #[doc(hidden)]
    fn _get_stride(&self) -> &SpatialStride<QI, NUM_DIMS>;

    #[doc(hidden)]
    fn _get_dimensions_mut(&mut self) -> &mut SpatialDimensions<Self::DimensionsQuant, NUM_DIMS>;

    #[doc(hidden)]
    fn _get_stride_mut(&mut self) -> &mut SpatialStride<QI, NUM_DIMS>;
}
