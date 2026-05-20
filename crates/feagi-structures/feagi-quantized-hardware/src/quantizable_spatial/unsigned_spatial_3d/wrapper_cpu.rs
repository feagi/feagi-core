use core::marker::PhantomData;
use crate::feagi_quantized_hardware_error::FeagiQuantizedHardwareError;
use crate::quantizable_base::index_count::{QuantizedIndexCountTrait, QuantizedIndexCountWrapperTrait};
use crate::quantizable_spatial::shared_spatial_traits::SpatialUnsignedBaseXDTrait;
use crate::quantizable_spatial::unsigned_spatial_3d::spatial_unsigned_3d::{SpatialUnsignedBase3DTrait, SpatialUnsignedCoordinate3DTrait, SpatialDimension3DTrait, verify_linear_index_within_unsigned_3d_bounds};

//region Coordinate
#[repr(C)]
#[derive(Copy, Clone)]
pub struct SpatialUnsignedCoordinate3D<
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
> {
    x: XAxisWrapper,
    y: YAxisWrapper,
    z: ZAxisWrapper,
    _p: PhantomData<(LinearWrapper, QuantIndex)>,
}

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper>
    SpatialUnsignedCoordinate3D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    #[inline(always)]
    pub fn new_checked(x: XAxisWrapper, y: YAxisWrapper, z: ZAxisWrapper) -> Result<Self, FeagiQuantizedHardwareError> {
        verify_linear_index_within_unsigned_3d_bounds(*x.quant_ref(), *y.quant_ref(), *z.quant_ref())?;
        Ok(Self {
            x,
            y,
            z,
            _p: PhantomData,
        })
    }
}

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper> SpatialUnsignedBaseXDTrait<QuantIndex>
    for SpatialUnsignedCoordinate3D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    SpatialUnsignedCoordinate3D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper>:
{
    type LinearIndexWrapperType = LinearWrapper;
}

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper> SpatialUnsignedBase3DTrait<QuantIndex>
    for SpatialUnsignedCoordinate3D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    SpatialUnsignedCoordinate3D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper>:
{
    type AxisXIndexWrapperType = XAxisWrapper;
    type AxisYIndexWrapperType = YAxisWrapper;
    type AxisZIndexWrapperType = ZAxisWrapper;

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
            x: XAxisWrapper::wrap_quant(x),
            y: YAxisWrapper::wrap_quant(y),
            z: ZAxisWrapper::wrap_quant(z),
            _p: PhantomData,
        }
    }
}

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper> SpatialUnsignedCoordinate3DTrait<QuantIndex>
    for SpatialUnsignedCoordinate3D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    SpatialUnsignedCoordinate3D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper>:
{

}

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper> core::ops::Add
    for SpatialUnsignedCoordinate3D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
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

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper> core::ops::Sub
    for SpatialUnsignedCoordinate3D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
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

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper> core::ops::AddAssign
    for SpatialUnsignedCoordinate3D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper> core::ops::SubAssign
    for SpatialUnsignedCoordinate3D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

#[cfg(feature = "alloc")]
impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper> core::fmt::Debug
    for SpatialUnsignedCoordinate3D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("SpatialUnsignedCoordinate3D")
            .field("x", self.x.quant_ref())
            .field("y", self.y.quant_ref())
            .field("z", self.z.quant_ref())
            .finish()
    }
}

#[cfg(feature = "alloc")]
impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper> core::fmt::Display
    for SpatialUnsignedCoordinate3D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
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
//endregion Coordinate



//region Dimension
#[repr(C)]
#[derive(Copy, Clone)]
pub struct SpatialUnsignedDimension3D<
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    CoordinateType: SpatialUnsignedCoordinate3DTrait<QuantIndex>,
> {
    x: XAxisWrapper,
    y: YAxisWrapper,
    z: ZAxisWrapper,
    _p: PhantomData<(LinearWrapper, QuantIndex, CoordinateType)>,
}

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, CoordinateType>
    SpatialUnsignedDimension3D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, CoordinateType>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    CoordinateType: SpatialUnsignedCoordinate3DTrait<QuantIndex>,
{
    #[inline(always)]
    pub fn new_checked(x: XAxisWrapper, y: YAxisWrapper, z: ZAxisWrapper) -> Result<Self, FeagiQuantizedHardwareError> {
        verify_linear_index_within_unsigned_3d_bounds(*x.quant_ref(), *y.quant_ref(), *z.quant_ref())?;
        Ok(Self {
            x,
            y,
            z,
            _p: PhantomData,
        })
    }
}

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, CoordinateType>
    SpatialUnsignedBaseXDTrait<QuantIndex>
    for SpatialUnsignedDimension3D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, CoordinateType>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    CoordinateType: SpatialUnsignedCoordinate3DTrait<QuantIndex>,
    SpatialUnsignedDimension3D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, CoordinateType>:
{
    type LinearIndexWrapperType = LinearWrapper;
}

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, CoordinateType>
    SpatialUnsignedBase3DTrait<QuantIndex>
    for SpatialUnsignedDimension3D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, CoordinateType>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    CoordinateType: SpatialUnsignedCoordinate3DTrait<QuantIndex>,
    SpatialUnsignedDimension3D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, CoordinateType>:
{
    type AxisXIndexWrapperType = XAxisWrapper;
    type AxisYIndexWrapperType = YAxisWrapper;
    type AxisZIndexWrapperType = ZAxisWrapper;

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
            x: XAxisWrapper::wrap_quant(x),
            y: YAxisWrapper::wrap_quant(y),
            z: ZAxisWrapper::wrap_quant(z),
            _p: PhantomData,
        }
    }
}

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, CoordinateType>
    SpatialDimension3DTrait<QuantIndex>
    for SpatialUnsignedDimension3D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, CoordinateType>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    CoordinateType: SpatialUnsignedCoordinate3DTrait<QuantIndex>,
    SpatialUnsignedDimension3D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, CoordinateType>:
{
    type CoordinateType = CoordinateType;
}

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, CoordinateType> core::ops::Add
    for SpatialUnsignedDimension3D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, CoordinateType>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    CoordinateType: SpatialUnsignedCoordinate3DTrait<QuantIndex>,
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

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, CoordinateType> core::ops::Sub
    for SpatialUnsignedDimension3D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, CoordinateType>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    CoordinateType: SpatialUnsignedCoordinate3DTrait<QuantIndex>,
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

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, CoordinateType> core::ops::AddAssign
    for SpatialUnsignedDimension3D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, CoordinateType>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    CoordinateType: SpatialUnsignedCoordinate3DTrait<QuantIndex>,
{
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, CoordinateType> core::ops::SubAssign
    for SpatialUnsignedDimension3D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, CoordinateType>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    CoordinateType: SpatialUnsignedCoordinate3DTrait<QuantIndex>,
{
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

#[cfg(feature = "alloc")]
impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, CoordinateType> core::fmt::Debug
    for SpatialUnsignedDimension3D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, CoordinateType>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    CoordinateType: SpatialUnsignedCoordinate3DTrait<QuantIndex>,
{
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("SpatialUnsignedDimension3D")
            .field("x", self.x.quant_ref())
            .field("y", self.y.quant_ref())
            .field("z", self.z.quant_ref())
            .finish()
    }
}

#[cfg(feature = "alloc")]
impl<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, CoordinateType> core::fmt::Display
    for SpatialUnsignedDimension3D<QuantIndex, LinearWrapper, XAxisWrapper, YAxisWrapper, ZAxisWrapper, CoordinateType>
where
    QuantIndex: QuantizedIndexCountTrait,
    LinearWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    XAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    YAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    ZAxisWrapper: QuantizedIndexCountWrapperTrait<QuantIndex>,
    CoordinateType: SpatialUnsignedCoordinate3DTrait<QuantIndex>,
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
//endregion Dimension
