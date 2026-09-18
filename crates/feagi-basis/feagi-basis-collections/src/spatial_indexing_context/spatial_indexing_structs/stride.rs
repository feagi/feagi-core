use feagi_basis_quantization::prelude::QuantizedUnsignedIntegerTrait;
use crate::spatial_indexing_context::spatial_indexing_structs::axis_order::SpatialAxisOrder;
use crate::spatial_indexing_context::spatial_indexing_structs::coordinate::SpatialCoordinate;
use crate::spatial_indexing_context::spatial_indexing_structs::dimensions::SpatialDimensions;

/// Generic owned stride value for an N-dimensional index space.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, ::serde::Serialize, ::serde::Deserialize)]
pub struct SpatialStride<QI: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize> {
    #[serde(with = "serde_arrays")]
    data: [QI; NUM_DIMS],
}

impl<QI: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize> SpatialStride<QI, NUM_DIMS> {
    /// Create a new stride from dimensions and axis order.
    pub fn new_stride(
        dims: &SpatialDimensions<QI, NUM_DIMS>,
        axis_order: &SpatialAxisOrder<QI, NUM_DIMS>,
    ) -> Self {
        let mut stride_data = [QI::QUANT_ZERO; NUM_DIMS];
        let mut next_stride = 1usize;

        // Build per-axis strides from the axis traversal order.
        for &axis in axis_order.as_slice().iter() {
            let axis_index = axis.quant_to_usize();
            stride_data[axis_index] = QI::quant_from_usize_unchecked(next_stride);
            let dim_axis = dims.as_slice()[axis_index].quant_to_usize();
            next_stride = next_stride.saturating_mul(dim_axis);
        }

        Self { data: stride_data }
    }

    /// Given changes to dimensions and axis order, update this stride.
    pub fn update_stride(
        &mut self,
        dims: &SpatialDimensions<QI, NUM_DIMS>,
        axis_order: &SpatialAxisOrder<QI, NUM_DIMS>,
    ) {
        *self = Self::new_stride(dims, axis_order);
    }

    /// Convert from coordinate to linear index.
    pub fn coordinate_to_linear<Linear: QuantizedUnsignedIntegerTrait<QuantType = QI::QuantType>>(
        &self,
        coordinate: &SpatialCoordinate<QI, NUM_DIMS>,
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
    pub fn linear_to_coordinate<Linear: QuantizedUnsignedIntegerTrait<QuantType = QI::QuantType>>(
        &self,
        linear_index: Linear,
        dims: &SpatialDimensions<QI, NUM_DIMS>,
    ) -> SpatialCoordinate<QI, NUM_DIMS> {
        let linear_index_usize = linear_index.quant_to_usize();
        let mut coordinate_data = [QI::QUANT_ZERO; NUM_DIMS];

        for axis in 0..NUM_DIMS {
            let stride_axis = self.data[axis].quant_to_usize();
            let dim_axis = dims.as_slice()[axis].quant_to_usize();
            let coord_axis = if dim_axis == 0 || stride_axis == 0 {
                0usize
            } else {
                (linear_index_usize / stride_axis) % dim_axis
            };
            coordinate_data[axis] = QI::quant_from_usize_unchecked(coord_axis);
        }

        SpatialCoordinate::new_coordinate(coordinate_data)
    }
}
