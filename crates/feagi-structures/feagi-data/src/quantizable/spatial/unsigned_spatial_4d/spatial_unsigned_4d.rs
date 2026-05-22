use crate::quantizable::feagi_data_quantized_error::FeagiDataQuantizedError;
use crate::quantizable::base_types::{QuantizedIndexCountTrait, QuantizedIndexCountWrapperTrait};
use crate::quantizable::spatial::shared_spatial_traits::SpatialUnsignedBaseXDTrait;

//region 4D
pub trait SpatialUnsignedBase4DTrait<
    QuantIndex: QuantizedIndexCountTrait,
    LinearIndexWrapperType: QuantizedIndexCountWrapperTrait<QuantIndex>,
>:
SpatialUnsignedBaseXDTrait<QuantIndex, LinearIndexWrapperType>
{
    // NOTE: We assume that once the struct is created, that the size of it is valid for
    // the current quantization. Be cautious when modifying it!
    type AxisXIndexWrapperType: QuantizedIndexCountWrapperTrait<QuantIndex>;
    type AxisYIndexWrapperType: QuantizedIndexCountWrapperTrait<QuantIndex>;
    type AxisZIndexWrapperType: QuantizedIndexCountWrapperTrait<QuantIndex>;
    type AxisTIndexWrapperType: QuantizedIndexCountWrapperTrait<QuantIndex>;

    fn get_x(&self) -> Self::AxisXIndexWrapperType;
    fn get_y(&self) -> Self::AxisYIndexWrapperType;
    fn get_z(&self) -> Self::AxisZIndexWrapperType;
    fn get_t(&self) -> Self::AxisTIndexWrapperType;
    fn get_x_mut(&mut self) -> &mut Self::AxisXIndexWrapperType;
    fn get_y_mut(&mut self) -> &mut Self::AxisYIndexWrapperType;
    fn get_z_mut(&mut self) -> &mut Self::AxisZIndexWrapperType;
    fn get_t_mut(&mut self) -> &mut Self::AxisTIndexWrapperType;
    fn set_x(&mut self, new_value: Self::AxisXIndexWrapperType);
    fn set_y(&mut self, new_value: Self::AxisYIndexWrapperType);
    fn set_z(&mut self, new_value: Self::AxisZIndexWrapperType);
    fn set_t(&mut self, new_value: Self::AxisTIndexWrapperType);
    fn new_unchecked(x: QuantIndex, y: QuantIndex, z: QuantIndex, t: QuantIndex) -> Self; // NOTE: Cant wrap this due to odd interconnects between this and Dimensions
}

pub trait SpatialUnsignedCoordinate4DTrait<
    QuantIndex: QuantizedIndexCountTrait,
    LinearIndexWrapperType: QuantizedIndexCountWrapperTrait<QuantIndex>,
>:
SpatialUnsignedBase4DTrait<QuantIndex, LinearIndexWrapperType>
{

}

pub trait SpatialDimension4DTrait<
    QuantIndex: QuantizedIndexCountTrait,
    LinearIndexWrapperType: QuantizedIndexCountWrapperTrait<QuantIndex>,
    CoordinateType: SpatialUnsignedCoordinate4DTrait<QuantIndex, LinearIndexWrapperType>,
>:
SpatialUnsignedBase4DTrait<QuantIndex, LinearIndexWrapperType>
{
    /// Is given coordinate within these dimensions
    fn contains_coordinate(&self, coordinate: &CoordinateType) -> bool {
        coordinate.get_x().quant_ref() < self.get_x().quant_ref()
            && coordinate.get_y().quant_ref() < self.get_y().quant_ref()
            && coordinate.get_z().quant_ref() < self.get_z().quant_ref()
            && coordinate.get_t().quant_ref() < self.get_t().quant_ref()
    }

    fn coordinate_to_linear_index(&self, coordinate: &CoordinateType) -> LinearIndexWrapperType {
        let xy_plane = *self.get_x().quant_ref() * *self.get_y().quant_ref();
        let xyz_volume = xy_plane * *self.get_z().quant_ref();

        LinearIndexWrapperType::wrap_quant(
            *coordinate.get_x().quant_ref()
                + (*coordinate.get_y() .quant_ref()* *self.get_x().quant_ref())
                + (*coordinate.get_z().quant_ref() * xy_plane)
                + (*coordinate.get_t().quant_ref() * xyz_volume)
        )
    }

    fn linear_index_to_coordinate(&self, linear_index: &LinearIndexWrapperType) -> CoordinateType {
        let xy_plane = *self.get_x().quant_ref() * *self.get_y().quant_ref();
        let xyz_volume = xy_plane * *self.get_z().quant_ref();
        let t = *linear_index.quant_ref() / xyz_volume;
        let rem_after_t = *linear_index.quant_ref() - t * xyz_volume;
        let z = rem_after_t / xy_plane;
        let rem_after_z = rem_after_t - z * xy_plane;
        let y = rem_after_z / *self.get_x().quant_ref();
        let x = rem_after_z - y * *self.get_x().quant_ref();

        CoordinateType::new_unchecked(
            x,
            y,
            z,
            t
        )
    }

    /// Get the max linear index (exclusive)
    fn get_max_linear_index(&self) -> LinearIndexWrapperType {
        LinearIndexWrapperType::wrap_quant(
            *self.get_x().quant_ref() * *self.get_y().quant_ref() * *self.get_z().quant_ref() * *self.get_t().quant_ref()
        )
    }
}

pub fn verify_linear_index_within_unsigned_4d_bounds<QuantIndex: QuantizedIndexCountTrait>(x: QuantIndex, y: QuantIndex, z: QuantIndex, t: QuantIndex) -> Result<(), FeagiDataQuantizedError> {
    let xy = x.to_usize() * y.to_usize();
    FeagiDataQuantizedError::verify_quantization_index::<QuantIndex>(xy, "4D spatial coordinates would exceed linear quantization index!")?;
    let xyz = xy * z.to_usize();
    FeagiDataQuantizedError::verify_quantization_index::<QuantIndex>(xyz, "4D spatial coordinates would exceed linear quantization index!")?;
    FeagiDataQuantizedError::verify_quantization_index::<QuantIndex>(xyz * t.to_usize(), "4D spatial coordinates would exceed linear quantization index!")
}