use crate::feagi_quantized_hardware_error::FeagiQuantizedHardwareError;
use crate::quantizable_base::{QuantizedIndexCountTrait, QuantizedIndexCountWrapperTrait};
use crate::quantizable_spatial::SpatialUnsignedBaseXDTrait;

//region 3D
pub trait SpatialUnsignedBase3DTrait<
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

    fn get_x(&self) -> Self::AxisXIndexWrapperType;
    fn get_y(&self) -> Self::AxisYIndexWrapperType;
    fn get_z(&self) -> Self::AxisZIndexWrapperType;
    fn get_x_mut(&mut self) -> &mut Self::AxisXIndexWrapperType;
    fn get_y_mut(&mut self) -> &mut Self::AxisYIndexWrapperType;
    fn get_z_mut(&mut self) -> &mut Self::AxisZIndexWrapperType;
    fn set_x(&mut self, new_value: Self::AxisXIndexWrapperType);
    fn set_y(&mut self, new_value: Self::AxisYIndexWrapperType);
    fn set_z(&mut self, new_value: Self::AxisZIndexWrapperType);
    fn new_unchecked(x: QuantIndex, y: QuantIndex, z: QuantIndex) -> Self; // NOTE: Cant wrap this due to odd interconnects between this and Dimensions
}

pub trait SpatialUnsignedCoordinate3DTrait<
    QuantIndex: QuantizedIndexCountTrait,
    LinearIndexWrapperType: QuantizedIndexCountWrapperTrait<QuantIndex>,
>:
SpatialUnsignedBase3DTrait<QuantIndex, LinearIndexWrapperType>
{

}

pub trait SpatialDimension3DTrait<
    QuantIndex: QuantizedIndexCountTrait,
    LinearIndexWrapperType: QuantizedIndexCountWrapperTrait<QuantIndex>,
    CoordinateType: SpatialUnsignedCoordinate3DTrait<QuantIndex, LinearIndexWrapperType>,
>:
SpatialUnsignedBase3DTrait<QuantIndex, LinearIndexWrapperType>
{
    /// Is given coordinate within these dimensions
    fn contains_coordinate(&self, coordinate: &CoordinateType) -> bool {
        coordinate.get_x().quant_ref() < self.get_x().quant_ref()
            && coordinate.get_y().quant_ref() < self.get_y().quant_ref()
            && coordinate.get_z().quant_ref() < self.get_z().quant_ref()
    }

    fn coordinate_to_linear_index(&self, coordinate: &CoordinateType) -> LinearIndexWrapperType {
        LinearIndexWrapperType::wrap_quant(
            *coordinate.get_x().quant_ref()
                + (*coordinate.get_y() .quant_ref()* *self.get_x().quant_ref())
                + (*coordinate.get_z().quant_ref() * *self.get_x().quant_ref() * *self.get_y().quant_ref())
        )
    }

    fn linear_index_to_coordinate(&self, linear_index: &LinearIndexWrapperType) -> CoordinateType {
        let plane = *self.get_x().quant_ref() * *self.get_y().quant_ref();
        let z = *linear_index.quant_ref() / plane;
        let rem = *linear_index.quant_ref() - z * plane;
        let y = rem / *self.get_x().quant_ref();
        let x = rem - y * *self.get_x().quant_ref();
        CoordinateType::new_unchecked(
            x,
            y,
            z,
        )
    }

    /// Get the max linear index (exclusive)
    fn get_max_linear_index(&self) -> LinearIndexWrapperType {
        LinearIndexWrapperType::wrap_quant(
            *self.get_x().quant_ref() * *self.get_y().quant_ref() * *self.get_z().quant_ref()
        )
    }
}


pub fn verify_linear_index_within_unsigned_3d_bounds<QuantIndex: QuantizedIndexCountTrait>(x: QuantIndex, y: QuantIndex, z: QuantIndex) -> Result<(), FeagiQuantizedHardwareError> {
    let xy = x.to_usize() * y.to_usize();
    FeagiQuantizedHardwareError::verify_quantization_index::<QuantIndex>(xy, "3D spatial coordinates would exceed linear quantization index!")?;
    FeagiQuantizedHardwareError::verify_quantization_index::<QuantIndex>(xy * z.to_usize(), "3D spatial coordinates would exceed linear quantization index!")
}
