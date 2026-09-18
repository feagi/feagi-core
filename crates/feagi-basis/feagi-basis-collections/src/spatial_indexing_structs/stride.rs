use feagi_basis_quantization::prelude::{QuantizedUnsignedIntegerTrait, QuantizedUnsignedIntegerUnwrappedTrait};
use crate::spatial_indexing_structs::axis_order::AxisOrderArray;
use crate::spatial_indexing_structs::coordinate::SpatialCoordinate;
use crate::spatial_indexing_structs::dimensions::SpatialDimensions;

/// Generic owned stride value for an N-dimensional index space.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, ::serde::Serialize, ::serde::Deserialize)]
pub struct SpatialStride<QI: QuantizedUnsignedIntegerUnwrappedTrait, const NUM_DIMS: usize> {
    #[serde(with = "serde_arrays")]
    data: [QI; NUM_DIMS],
}

impl<QI: QuantizedUnsignedIntegerUnwrappedTrait, const NUM_DIMS: usize> SpatialStride<QI, NUM_DIMS> {
    /// Create a new stride from dimensions and axis order.
    pub fn new_stride<
        QDims: QuantizedUnsignedIntegerTrait<QuantType=QI>,
    >(
        dims: &SpatialDimensions<QDims, NUM_DIMS>,
        axis_order: &AxisOrderArray<NUM_DIMS>,
    ) -> Self {
        let mut stride_data = [QI::QUANT_ZERO; NUM_DIMS];
        let mut next_stride = 1usize;

        // Build per-axis strides from the axis traversal order.
        for &axis in axis_order.iter() {
            let axis_index = axis;
            stride_data[axis_index] = QI::quant_from_usize_unchecked(next_stride);
            let dim_axis = dims.as_slice()[axis_index].quant_to_usize();
            next_stride = next_stride.saturating_mul(dim_axis);
        }

        Self { data: stride_data }
    }

    /// Given changes to dimensions and axis order, update this stride.
    pub fn update_stride<
        QDims: QuantizedUnsignedIntegerTrait<QuantType=QI>,
    >(
        &mut self,
        dims: &SpatialDimensions<QDims, NUM_DIMS>,
        axis_order: &AxisOrderArray<NUM_DIMS>
    ) {
        *self = Self::new_stride(dims, axis_order);
    }

    /// Convert from coordinate to linear index.
    pub fn coordinate_to_linear<
        Linear: QuantizedUnsignedIntegerTrait<QuantType=QI::QuantType>,
        QCoords: QuantizedUnsignedIntegerTrait<QuantType=QI>,
    >(
        &self,
        coordinate: &SpatialCoordinate<QCoords, NUM_DIMS>,
    ) -> Linear {
        let mut linear_index = 0usize;
        for axis in 0..NUM_DIMS {
            let coord_axis = coordinate.as_slice()[axis].quant_to_usize();
            let stride_axis = self.data[axis].quant_to_usize();
            linear_index = linear_index + (coord_axis * stride_axis);
        }
        Linear::quant_from_usize_unchecked(linear_index)
    }

    /// Convert from linear index to coordinate.
    pub fn linear_to_coordinate<
        Linear: QuantizedUnsignedIntegerTrait<QuantType=QI::QuantType>,
        QCoords: QuantizedUnsignedIntegerTrait<QuantType=QI>,
        QDims: QuantizedUnsignedIntegerTrait<QuantType=QI>,
    >(
        &self,
        linear_index: Linear,
        dims: &SpatialDimensions<QDims, NUM_DIMS>,
    ) -> SpatialCoordinate<QCoords, NUM_DIMS> {
        let linear_index_usize = linear_index.quant_to_usize();
        let mut coordinate_data = [QCoords::QUANT_ZERO; NUM_DIMS];

        for axis in 0..NUM_DIMS {
            let stride_axis = self.data[axis].quant_to_usize();
            let dim_axis = dims.as_slice()[axis].quant_to_usize();
            let coord_axis = if dim_axis == 0 || stride_axis == 0 {
                0
            } else {
                (linear_index_usize / stride_axis) % dim_axis
            };
            coordinate_data[axis] = QCoords::quant_from_usize_unchecked(coord_axis);
        }

        SpatialCoordinate::new_coordinate(coordinate_data)
    }
}
