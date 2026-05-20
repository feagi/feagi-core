use core::marker::PhantomData;
use crate::feagi_quantized_hardware_error::FeagiQuantizedHardwareError;
use crate::quantizable_base::index_count::{QuantizedIndexCountTrait, QuantizedIndexCountWrapperTrait};
use crate::quantizable_spatial::shared_spatial_traits::SpatialUnsignedBaseXDTrait;
use crate::quantizable_spatial::unsigned_spatial_4d::spatial_unsigned_4d::{SpatialUnsignedBase4DTrait, SpatialUnsignedCoordinate4DTrait, SpatialDimension4DTrait, verify_linear_index_within_unsigned_4d_bounds};

//region Coordinate
#[repr(C)]
#[derive(Copy, Clone)]
pub struct SpatialUnsignedCoordinate4D<
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    TAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
> {
    x: XAxisWrapper,
    y: YAxisWrapper,
    z: ZAxisWrapper,
    t: TAxisWrapper,
    _p: PhantomData<(LinearWrapper, QuantIndex)>,
}

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper>
    SpatialUnsignedCoordinate4D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    TAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    #[inline(always)]
    pub fn new_checked(x: XAxisWrapper, y: YAxisWrapper, z: ZAxisWrapper, t: TAxisWrapper) -> Result<Self, FeagiQuantizedHardwareError> {
        verify_linear_index_within_unsigned_4d_bounds(*x.quant_ref(), *y.quant_ref(), *z.quant_ref(), *t.quant_ref())?;
        Ok(Self {
            x,
            y,
            z,
            t,
            _p: PhantomData,
        })
    }
}

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper> SpatialUnsignedBaseXDTrait<QuantIndex>
    for SpatialUnsignedCoordinate4D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    TAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    SpatialUnsignedCoordinate4D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper>:
{
    type LinearIndexWrapperType = LinearWrapper;
}

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper> SpatialUnsignedBase4DTrait<QuantIndex>
    for SpatialUnsignedCoordinate4D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    TAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    SpatialUnsignedCoordinate4D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper>:
{
    type AxisXIndexWrapperType = XAxisWrapper;
    type AxisYIndexWrapperType = YAxisWrapper;
    type AxisZIndexWrapperType = ZAxisWrapper;
    type AxisTIndexWrapperType = TAxisWrapper;

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
            x: XAxisWrapper::wrap_quant(x),
            y: YAxisWrapper::wrap_quant(y),
            z: ZAxisWrapper::wrap_quant(z),
            t: TAxisWrapper::wrap_quant(t),
            _p: PhantomData,
        }
    }
}

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper> SpatialUnsignedCoordinate4DTrait<QuantIndex>
    for SpatialUnsignedCoordinate4D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    TAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    SpatialUnsignedCoordinate4D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper>:
{

}

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper> core::ops::Add
    for SpatialUnsignedCoordinate4D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    TAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
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

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper> core::ops::Sub
    for SpatialUnsignedCoordinate4D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    TAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
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

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper> core::ops::AddAssign
    for SpatialUnsignedCoordinate4D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    TAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self.t += rhs.t;
    }
}

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper> core::ops::SubAssign
    for SpatialUnsignedCoordinate4D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    TAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
        self.t -= rhs.t;
    }
}

#[cfg(feature = "alloc")]
impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper> core::fmt::Debug
    for SpatialUnsignedCoordinate4D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    TAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("SpatialUnsignedCoordinate4D")
            .field("x", self.x.quant_ref())
            .field("y", self.y.quant_ref())
            .field("z", self.z.quant_ref())
            .field("t", self.t.quant_ref())
            .finish()
    }
}

#[cfg(feature = "alloc")]
impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper> core::fmt::Display
    for SpatialUnsignedCoordinate4D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    TAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
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
//endregion Coordinate



//region Dimension
#[repr(C)]
#[derive(Copy, Clone)]
pub struct SpatialUnsignedDimension4D<
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    TAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    CoordinateType: SpatialUnsignedCoordinate4DTrait<QuantIndex>,
> {
    x: XAxisWrapper,
    y: YAxisWrapper,
    z: ZAxisWrapper,
    t: TAxisWrapper,
    _p: PhantomData<(LinearWrapper, QuantIndex, CoordinateType)>,
}

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper, CoordinateType>
    SpatialUnsignedDimension4D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper, CoordinateType>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    TAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    CoordinateType: SpatialUnsignedCoordinate4DTrait<QuantIndex>,
{
    #[inline(always)]
    pub fn new_checked(x: XAxisWrapper, y: YAxisWrapper, z: ZAxisWrapper, t: TAxisWrapper) -> Result<Self, FeagiQuantizedHardwareError> {
        verify_linear_index_within_unsigned_4d_bounds(*x.quant_ref(), *y.quant_ref(), *z.quant_ref(), *t.quant_ref())?;
        Ok(Self {
            x,
            y,
            z,
            t,
            _p: PhantomData,
        })
    }
}

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper, CoordinateType>
    SpatialUnsignedBaseXDTrait<QuantIndex>
    for SpatialUnsignedDimension4D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper, CoordinateType>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    TAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    CoordinateType: SpatialUnsignedCoordinate4DTrait<QuantIndex>,
    SpatialUnsignedDimension4D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper, CoordinateType>:
{
    type LinearIndexWrapperType = LinearWrapper;
}

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper, CoordinateType>
    SpatialUnsignedBase4DTrait<QuantIndex>
    for SpatialUnsignedDimension4D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper, CoordinateType>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    TAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    CoordinateType: SpatialUnsignedCoordinate4DTrait<QuantIndex>,
    SpatialUnsignedDimension4D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper, CoordinateType>:
{
    type AxisXIndexWrapperType = XAxisWrapper;
    type AxisYIndexWrapperType = YAxisWrapper;
    type AxisZIndexWrapperType = ZAxisWrapper;
    type AxisTIndexWrapperType = TAxisWrapper;

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
            x: XAxisWrapper::wrap_quant(x),
            y: YAxisWrapper::wrap_quant(y),
            z: ZAxisWrapper::wrap_quant(z),
            t: TAxisWrapper::wrap_quant(t),
            _p: PhantomData,
        }
    }
}

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper, CoordinateType>
    SpatialDimension4DTrait<QuantIndex>
    for SpatialUnsignedDimension4D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper, CoordinateType>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    TAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    CoordinateType: SpatialUnsignedCoordinate4DTrait<QuantIndex>,
    SpatialUnsignedDimension4D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper, CoordinateType>:
{
    type CoordinateType = CoordinateType;
}

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper, CoordinateType> core::ops::Add
    for SpatialUnsignedDimension4D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper, CoordinateType>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    TAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    CoordinateType: SpatialUnsignedCoordinate4DTrait<QuantIndex>,
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

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper, CoordinateType> core::ops::Sub
    for SpatialUnsignedDimension4D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper, CoordinateType>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    TAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    CoordinateType: SpatialUnsignedCoordinate4DTrait<QuantIndex>,
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

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper, CoordinateType> core::ops::AddAssign
    for SpatialUnsignedDimension4D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper, CoordinateType>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    TAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    CoordinateType: SpatialUnsignedCoordinate4DTrait<QuantIndex>,
{
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
        self.t += rhs.t;
    }
}

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper, CoordinateType> core::ops::SubAssign
    for SpatialUnsignedDimension4D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper, CoordinateType>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    TAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    CoordinateType: SpatialUnsignedCoordinate4DTrait<QuantIndex>,
{
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
        self.t -= rhs.t;
    }
}

#[cfg(feature = "alloc")]
impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper, CoordinateType> core::fmt::Debug
    for SpatialUnsignedDimension4D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper, CoordinateType>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    TAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    CoordinateType: SpatialUnsignedCoordinate4DTrait<QuantIndex>,
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("SpatialUnsignedDimension4D")
            .field("x", self.x.quant_ref())
            .field("y", self.y.quant_ref())
            .field("z", self.z.quant_ref())
            .field("t", self.t.quant_ref())
            .finish()
    }
}

#[cfg(feature = "alloc")]
impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper, CoordinateType> core::fmt::Display
    for SpatialUnsignedDimension4D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, TAxisWrapper, CoordinateType>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    TAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    CoordinateType: SpatialUnsignedCoordinate4DTrait<QuantIndex>,
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
//endregion Dimension
