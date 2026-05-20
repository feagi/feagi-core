use std::marker::PhantomData;
use crate::feagi_quantized_hardware_error::FeagiQuantizedHardwareError;
use crate::quantizable_base::index_count::{QuantizedIndexCountTrait, QuantizedIndexCountWrapperTrait};
use crate::quantizable_spatial::SpatialUnsignedBaseXDTrait;
use crate::quantizable_spatial::unsigned_spatial_3d::spatial_unsigned_3d::{verify_linear_index_within_unsigned_3d_bounds, SpatialDimension3DTrait, SpatialUnsignedBase3DTrait, SpatialUnsignedCoordinate3DTrait};

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct SpatialUnsigned3D<
    QuantIndex: QuantizedIndexCountTrait,
    QuantIndexWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
> {
    x: QuantIndexWrapper,
    y: QuantIndexWrapper,
    z: QuantIndexWrapper,
    _p: PhantomData<QuantIndex>,
}

impl<QuantIndex, QuantIndexWrapper> SpatialUnsigned3D<QuantIndex, QuantIndexWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    QuantIndexWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    #[inline(always)]
    pub fn new(x: QuantIndex, y: QuantIndex, z: QuantIndex) -> Result<Self, FeagiQuantizedHardwareError> {
        verify_linear_index_within_unsigned_3d_bounds(x, y, z)?;
        Ok(Self {
            x: QuantIndexWrapper::wrap_quant(x),
            y: QuantIndexWrapper::wrap_quant(y),
            z: QuantIndexWrapper::wrap_quant(z),
            _p: PhantomData,
        })
    }
}

impl<QuantIndex, QuantIndexWrapper> SpatialUnsignedBaseXDTrait<QuantIndex>
for SpatialUnsigned3D<QuantIndex, QuantIndexWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    QuantIndexWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    type LinearIndexWrapperType = QuantIndexWrapper;
}

impl<QuantIndex, QuantIndexWrapper> SpatialUnsignedBase3DTrait<QuantIndex>
for SpatialUnsigned3D<QuantIndex, QuantIndexWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    QuantIndexWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    type AxisXIndexWrapperType = QuantIndexWrapper;
    type AxisYIndexWrapperType = QuantIndexWrapper;
    type AxisZIndexWrapperType = QuantIndexWrapper;

    #[inline(always)]
    fn get_x(&self) -> Self::AxisXIndexWrapperType {
        self.x
    }

    #[inline(always)]
    fn get_y(&self) -> Self::AxisYIndexWrapperType {
        self.y
    }

    #[inline(always)]
    fn get_z(&self) -> Self::AxisZIndexWrapperType {
        self.z
    }

    #[inline(always)]
    fn get_x_mut(&mut self) -> &mut Self::AxisXIndexWrapperType {
        &mut self.x
    }

    #[inline(always)]
    fn get_y_mut(&mut self) -> &mut Self::AxisYIndexWrapperType {
        &mut self.y
    }

    #[inline(always)]
    fn get_z_mut(&mut self) -> &mut Self::AxisZIndexWrapperType {
        &mut self.z
    }

    #[inline(always)]
    fn set_x(&mut self, new_value: Self::AxisXIndexWrapperType) {
        self.x = new_value;
    }

    #[inline(always)]
    fn set_y(&mut self, new_value: Self::AxisYIndexWrapperType) {
        self.y = new_value;
    }

    #[inline(always)]
    fn set_z(&mut self, new_value: Self::AxisZIndexWrapperType) {
        self.z = new_value;
    }

    #[inline(always)]
    fn new_unchecked(x: QuantIndex, y: QuantIndex, z: QuantIndex) -> Self {
        Self {
            x: QuantIndexWrapper::wrap_quant(x),
            y: QuantIndexWrapper::wrap_quant(y),
            z: QuantIndexWrapper::wrap_quant(z),
            _p: PhantomData,
        }
    }
}

impl<QuantIndex, QuantIndexWrapper> SpatialUnsignedCoordinate3DTrait<QuantIndex>
for SpatialUnsigned3D<QuantIndex, QuantIndexWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    QuantIndexWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
}

impl<QuantIndex, QuantIndexWrapper> SpatialDimension3DTrait<QuantIndex>
for SpatialUnsigned3D<QuantIndex, QuantIndexWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    QuantIndexWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    type CoordinateType = Self;
}

//region Math
impl<QuantIndex, QuantIndexWrapper> core::ops::Add for SpatialUnsigned3D<QuantIndex, QuantIndexWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    QuantIndexWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            _p: PhantomData,
        }
    }
}

impl<QuantIndex, QuantIndexWrapper> core::ops::Sub for SpatialUnsigned3D<QuantIndex, QuantIndexWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    QuantIndexWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            _p: PhantomData,
        }
    }
}

impl<QuantIndex, QuantIndexWrapper> core::ops::AddAssign for SpatialUnsigned3D<QuantIndex, QuantIndexWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    QuantIndexWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl<QuantIndex, QuantIndexWrapper> core::ops::SubAssign for SpatialUnsigned3D<QuantIndex, QuantIndexWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    QuantIndexWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl<QuantIndex, QuantIndexWrapper> core::cmp::PartialEq for SpatialUnsigned3D<QuantIndex, QuantIndexWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    QuantIndexWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }
}

impl<QuantIndex, QuantIndexWrapper> core::cmp::PartialOrd for SpatialUnsigned3D<QuantIndex, QuantIndexWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    QuantIndexWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        (self.x.quant(), self.y.quant(), self.z.quant())
            .partial_cmp(&(other.x.quant(), other.y.quant(), other.z.quant()))
    }
}
//endregion

#[cfg(feature = "alloc")]
impl<QuantIndex, QuantIndexWrapper> core::fmt::Debug for SpatialUnsigned3D<QuantIndex, QuantIndexWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    QuantIndexWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("SpatialUnsigned3D")
            .field("x", self.x.quant_ref())
            .field("y", self.y.quant_ref())
            .field("z", self.z.quant_ref())
            .finish()
    }
}

#[cfg(feature = "alloc")]
impl<QuantIndex, QuantIndexWrapper> core::fmt::Display for SpatialUnsigned3D<QuantIndex, QuantIndexWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    QuantIndexWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "({}, {}, {})",
            self.x.quant_ref(),
            self.y.quant_ref(),
            self.z.quant_ref()
        )
    }
}