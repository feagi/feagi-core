use std::marker::PhantomData;
use crate::feagi_quantized_hardware_error::FeagiQuantizedHardwareError;
use crate::quantizable_base::index_count::{QuantizedIndexCountTrait, QuantizedIndexCountWrapperTrait};
use crate::quantizable_spatial::SpatialUnsignedBaseXDTrait;
use crate::quantizable_spatial::unsigned_spatial_4d::spatial_unsigned_4d::{verify_linear_index_within_unsigned_4d_bounds, SpatialDimension4DTrait, SpatialUnsignedBase4DTrait, SpatialUnsignedCoordinate4DTrait};

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct SpatialUnsigned4D<
    QuantIndex: QuantizedIndexCountTrait,
    QuantIndexWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
> {
    x: QuantIndexWrapper,
    y: QuantIndexWrapper,
    z: QuantIndexWrapper,
    t: QuantIndexWrapper,
    _p: PhantomData<QuantIndex>,
}

impl<QuantIndex, QuantIndexWrapper> SpatialUnsigned4D<QuantIndex, QuantIndexWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    QuantIndexWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    #[inline(always)]
    pub fn new(x: QuantIndex, y: QuantIndex, z: QuantIndex, t: QuantIndex) -> Result<Self, FeagiQuantizedHardwareError> {
        verify_linear_index_within_unsigned_4d_bounds(x, y, z, t)?;
        Ok(Self {
            x: QuantIndexWrapper::wrap_quant(x),
            y: QuantIndexWrapper::wrap_quant(y),
            z: QuantIndexWrapper::wrap_quant(z),
            t: QuantIndexWrapper::wrap_quant(t),
            _p: PhantomData,
        })
    }
}

impl<QuantIndex, QuantIndexWrapper> SpatialUnsignedBaseXDTrait<QuantIndex>
for SpatialUnsigned4D<QuantIndex, QuantIndexWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    QuantIndexWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    type LinearIndexWrapperType = QuantIndexWrapper;
}

impl<QuantIndex, QuantIndexWrapper> SpatialUnsignedBase4DTrait<QuantIndex>
for SpatialUnsigned4D<QuantIndex, QuantIndexWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    QuantIndexWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    type AxisXIndexWrapperType = QuantIndexWrapper;
    type AxisYIndexWrapperType = QuantIndexWrapper;
    type AxisZIndexWrapperType = QuantIndexWrapper;
    type AxisTIndexWrapperType = QuantIndexWrapper;

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
    fn get_t(&self) -> Self::AxisTIndexWrapperType {
        self.t
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
    fn get_t_mut(&mut self) -> &mut Self::AxisTIndexWrapperType {
        &mut self.t
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
    fn set_t(&mut self, new_value: Self::AxisTIndexWrapperType) {
        self.t = new_value;
    }

    #[inline(always)]
    fn new_unchecked(x: QuantIndex, y: QuantIndex, z: QuantIndex, t: QuantIndex) -> Self {
        Self {
            x: QuantIndexWrapper::wrap_quant(x),
            y: QuantIndexWrapper::wrap_quant(y),
            z: QuantIndexWrapper::wrap_quant(z),
            t: QuantIndexWrapper::wrap_quant(t),
            _p: PhantomData,
        }
    }
}

impl<QuantIndex, QuantIndexWrapper> SpatialUnsignedCoordinate4DTrait<QuantIndex>
for SpatialUnsigned4D<QuantIndex, QuantIndexWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    QuantIndexWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
}

impl<QuantIndex, QuantIndexWrapper> SpatialDimension4DTrait<QuantIndex>
for SpatialUnsigned4D<QuantIndex, QuantIndexWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    QuantIndexWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    type CoordinateType = Self;
}

//region Math
impl<QuantIndex, QuantIndexWrapper> core::ops::Add for SpatialUnsigned4D<QuantIndex, QuantIndexWrapper>
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
            t: self.t + rhs.t,
            _p: PhantomData,
        }
    }
}

impl<QuantIndex, QuantIndexWrapper> core::ops::Sub for SpatialUnsigned4D<QuantIndex, QuantIndexWrapper>
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
            t: self.t - rhs.t,
            _p: PhantomData,
        }
    }
}

impl<QuantIndex, QuantIndexWrapper> core::ops::AddAssign for SpatialUnsigned4D<QuantIndex, QuantIndexWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    QuantIndexWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self.t += rhs.t;
    }
}

impl<QuantIndex, QuantIndexWrapper> core::ops::SubAssign for SpatialUnsigned4D<QuantIndex, QuantIndexWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    QuantIndexWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
        self.t -= rhs.t;
    }
}

impl<QuantIndex, QuantIndexWrapper> core::cmp::PartialEq for SpatialUnsigned4D<QuantIndex, QuantIndexWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    QuantIndexWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z && self.t == other.t
    }
}

impl<QuantIndex, QuantIndexWrapper> core::cmp::PartialOrd for SpatialUnsigned4D<QuantIndex, QuantIndexWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    QuantIndexWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        (self.x.quant(), self.y.quant(), self.z.quant(), self.t.quant()).partial_cmp(&(
            other.x.quant(),
            other.y.quant(),
            other.z.quant(),
            other.t.quant(),
        ))
    }
}
//endregion

#[cfg(feature = "alloc")]
impl<QuantIndex, QuantIndexWrapper> core::fmt::Debug for SpatialUnsigned4D<QuantIndex, QuantIndexWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    QuantIndexWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("SpatialUnsigned4D")
            .field("x", self.x.quant_ref())
            .field("y", self.y.quant_ref())
            .field("z", self.z.quant_ref())
            .field("t", self.t.quant_ref())
            .finish()
    }
}

#[cfg(feature = "alloc")]
impl<QuantIndex, QuantIndexWrapper> core::fmt::Display for SpatialUnsigned4D<QuantIndex, QuantIndexWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    QuantIndexWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "({}, {}, {}, {})",
            self.x.quant_ref(),
            self.y.quant_ref(),
            self.z.quant_ref(),
            self.t.quant_ref()
        )
    }
}