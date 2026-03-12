use crate::base_quantizable::quantizable_ints::QuantizableInt;
use crate::FeagiBaseError;
use crate::base_quantizable::quantizable_uints::QuantizableUInt;


//region NonZeroIndex
#[repr(transparent)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct NonzeroCountType<T: QuantizableUInt>(T);

pub type NonzeroCountISize = NonzeroCountType<isize>;
pub type NonzeroCountU64 = NonzeroCountType<u64>;
pub type NonzeroCountU32 = NonzeroCountType<u32>;
pub type NonzeroCountU16 = NonzeroCountType<u16>;
pub type NonzeroCountU8 = NonzeroCountType<u8>;

impl<T: QuantizableUInt> NonzeroCountType<T> {

    pub const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES;

    pub(crate) fn new_unchecked(n: T) -> Self {
        Self(n)
    }

    pub fn new(n: T) -> Result<Self, FeagiBaseError> {
        if n.lt(T::one()) {
            return Err(FeagiBaseError::ValueCannotBeZero);
        }
        Ok(Self(n))
    }

    pub fn get(self) -> T {
        self.0
    }
}

impl<T: QuantizableUInt> core::ops::Deref for NonzeroCountType<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: QuantizableUInt> core::fmt::Display for NonzeroCountType<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

//endregion

//region 2D

//region Unsigned Coordinate 2D
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]

pub struct UnsignedCoordinate2DType<T: QuantizableUInt> {
    pub x: T,
    pub y: T,
}

pub type UnsignedCoordinate2DUSize = UnsignedCoordinate2DType<usize>;
pub type UnsignedCoordinate2DU64 = UnsignedCoordinate2DType<u64>;
pub type UnsignedCoordinate2DU32 = UnsignedCoordinate2DType<u32>;
pub type UnsignedCoordinate2DU16 = UnsignedCoordinate2DType<u16>;
pub type UnsignedCoordinate2DU8 = UnsignedCoordinate2DType<u8>;

impl<T: QuantizableUInt> UnsignedCoordinate2DType<T> {

    pub const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 2;

    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn new_with_fit_check(
        x: T,
        y: T,
        bounds: &Dimension2DType<T>,
    ) -> Result<Self, FeagiBaseError> {
        let coords = Self::new(x, y);
        bounds.verify_fit(&coords)?;
        Ok(coords)
    }
}

impl<T: QuantizableUInt> core::fmt::Display for UnsignedCoordinate2DType<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "UnsignedCoordinate2D<{}, {}>", self.x, self.y)
    }
}

impl<T: QuantizableUInt> Into<UnsignedCoordinate2DUSize> for UnsignedCoordinate2DType<T> {
    fn into(self) -> UnsignedCoordinate2DISize {
        UnsignedCoordinate2DISize::new(self.x as usize, self.y as usize)
    }
}

//endregion

//region Signed Coordinate 2D
pub struct SignedCoordinate2DType<T: QuantizableInt> {
    pub x: T,
    pub y: T,
}

pub type SignedCoordinate2DISize = SignedCoordinate2DType<isize>;
pub type SignedCoordinate2DI64 = SignedCoordinate2DType<i64>;
pub type SignedCoordinate2DI32 = SignedCoordinate2DType<i32>;
pub type SignedCoordinate2DI16 = SignedCoordinate2DType<i16>;
pub type SignedCoordinate2DI8 = SignedCoordinate2DType<i8>;

impl<T: QuantizableUInt> SignedCoordinate2DType<T> {

    pub const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 2;
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T: QuantizableUInt> core::fmt::Display for SignedCoordinate2DType<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SignedCoordinate2D<{}, {}>", self.x, self.y)
    }
}

impl<T: QuantizableUInt> Into<SignedCoordinate2DISize> for SignedCoordinate2DType<T> {
    fn into(self) -> SignedCoordinate2DISize {
        SignedCoordinate2DISize::new(self.x as isize, self.y as isize)
    }
}

//endregion

//region Dimension 2D
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Dimension2DType<T: QuantizableUInt> {
    pub x: NonzeroCountType<T>,
    pub y: NonzeroCountType<T>,
}

pub type Dimension2DUSize = Dimension2DType<usize>;
pub type Dimension2DU64 = Dimension2DType<u64>;
pub type Dimension2DU32 = Dimension2DType<u32>;
pub type Dimension2DU16 = Dimension2DType<u16>;
pub type Dimension2DU8 = Dimension2DType<u8>;

impl<T: QuantizableUInt> Dimension2DType<T> {

    pub const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 2;

    pub(crate) fn new_unchecked(x: T, y: T) -> Self {
        let x = NonzeroCountType::new_unchecked(x);
        let y = NonzeroCountType::new_unchecked(y);
        Self { x, y }
    }

    pub fn new(x: T, y: T) -> Result<Self, FeagiBaseError> {
        let x = NonzeroCountType::new(x)?;
        let y = NonzeroCountType::new(y)?;
        Ok(Self { x, y })
    }

    pub fn new_square(n: T) -> Result<Self, FeagiBaseError> {
        let x = NonzeroCountType::new(n)?;
        let y = x;
        Ok(Self { x, y })
    }

    pub fn does_fit(&self, coordinate: &UnsignedCoordinate2DType<T>) -> bool {
        coordinate.x.lt(self.x.get()) && coordinate.y.lt(self.y.get())
    }

    pub fn verify_fit(&self, coordinate: &UnsignedCoordinate2DType<T>) -> Result<(), FeagiBaseError> {
        if self.does_fit(coordinate) {
            return Ok(());
        }
        Err(FeagiBaseError::Coordinate2DOutOfBounds {
            coordinate,
            dimensions: self,
        })
    }
}

impl<T: QuantizableUInt + core::fmt::Display> core::fmt::Display for Dimension2DType<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Dimensions2D<{}, {}>", self.x, self.y)
    }
}

impl<T: QuantizableUInt> Into<Dimension2DUSize> for Dimension2DType<T> {
    fn into(self) -> Dimension2DUSize {
        Dimension2DUSize::new_unchecked(self.x as usize, self.y as usize)
    }
}

//endregion

//endregion

//region  3D

//region Unsigned Coordinate 3D
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct UnsignedCoordinate3DType<T: QuantizableUInt> {
    pub x: T,
    pub y: T,
    pub z: T,
}

pub type UnsignedCoordinate3DUSize = UnsignedCoordinate3DType<usize>;
pub type UnsignedCoordinate3DU64 = UnsignedCoordinate3DType<u64>;
pub type UnsignedCoordinate3DU32 = UnsignedCoordinate3DType<u32>;
pub type UnsignedCoordinate3DU16 = UnsignedCoordinate3DType<u16>;
pub type UnsignedCoordinate3DU8 = UnsignedCoordinate3DType<u8>;

impl<T: QuantizableUInt> UnsignedCoordinate3DType<T> {

    pub const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 3;

    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }

    pub fn new_with_fit_check(
        x: T,
        y: T,
        z: T,
        bounds: &Dimension3DType<T>,
    ) -> Result<Self, FeagiBaseError> {
        let coords = Self::new(x, y, z);
        bounds.verify_fit(&coords)?;
        Ok(coords)
    }
}

impl<T: QuantizableUInt + core::fmt::Display> core::fmt::Display for UnsignedCoordinate3DType<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "UnsignedCoordinate3D<{}, {}, {}>", self.x, self.y, self.z)
    }
}

impl<T: QuantizableUInt> Into<UnsignedCoordinate3DUSize> for UnsignedCoordinate3DType<T> {
    fn into(self) -> UnsignedCoordinate3DUSize {
        UnsignedCoordinate3DUSize::new(self.x as usize, self as usize, self as usize)
    }
}
//endregion

//region Signed Coordinate 3D
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct SignedCoordinate3DType<T: QuantizableInt> {
    pub x: T,
    pub y: T,
    pub z: T,
}

pub type SignedCoordinate3DISize = SignedCoordinate3DType<isize>;
pub type SignedCoordinate3DI64 = SignedCoordinate3DType<i64>;
pub type SignedCoordinate3DI32 = SignedCoordinate3DType<i32>;
pub type SignedCoordinate3DI16 = SignedCoordinate3DType<i16>;
pub type SignedCoordinate3DI8 = SignedCoordinate3DType<i8>;

impl<T: QuantizableUInt> SignedCoordinate3DType<T> {

    pub const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 3;

    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }
}

impl<T: QuantizableUInt + core::fmt::Display> core::fmt::Display for SignedCoordinate3DType<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SignedCoordinate3D<{}, {}, {}>", self.x, self.y, self.z)
    }
}

impl<T: QuantizableUInt> Into<SignedCoordinate3DISize> for SignedCoordinate3DType<T> {
    fn into(self) -> SignedCoordinate3DISize {
        SignedCoordinate3DISize::new(self.x as isize, self as isize, self as isize)
    }
}
//endregion

//region Dimension 3D
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Dimension3DType<T: QuantizableUInt> {
    pub x: NonzeroCountType<T>,
    pub y: NonzeroCountType<T>,
    pub z: NonzeroCountType<T>,
}

pub type Dimension3DUSize = Dimension3DType<usize>;
pub type Dimension3DU64 = Dimension3DType<u64>;
pub type Dimension3DU32 = Dimension3DType<u32>;
pub type Dimension3DU16 = Dimension3DType<u16>;
pub type Dimension3DU8 = Dimension3DType<u8>;

impl<T: QuantizableUInt> Dimension3DType<T> {

    pub const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 3;

    pub(crate) fn new_unchecked(x: T, y: T, z: T) -> Self {
        let x = NonzeroCountType::new_unchecked(x);
        let y = NonzeroCountType::new_unchecked(y);
        let z = NonzeroCountType::new_unchecked(z);
        Self { x, y, z }
    }

    pub fn new(x: T, y: T, z: T) -> Result<Self, FeagiBaseError> {
        let x = NonzeroCountType::new(x)?;
        let y = NonzeroCountType::new(y)?;
        let z = NonzeroCountType::new(z)?;
        Ok(Self { x, y, z })
    }

    pub fn new_cube(n: T) -> Result<Self, FeagiBaseError> {
        let x = NonzeroCountType::new(n)?;
        let y = x;
        let z = x;
        Ok(Self { x, y, z })
    }

    pub fn does_fit(&self, coordinate: &UnsignedCoordinate3DType<T>) -> bool {
        coordinate.x.lt(self.x.get())
            && coordinate.y.lt(self.y.get())
            && coordinate.z.lt(self.z.get())
    }

    pub fn verify_fit(&self, coordinate: &UnsignedCoordinate3DType<T>) -> Result<(), FeagiBaseError> {
        if self.does_fit(coordinate) {
            return Ok(());
        }
        Err(FeagiBaseError::Coordinate3DOutOfBounds {
            coordinate,
            dimensions: self,
        })
    }
}

impl<T: QuantizableUInt + core::fmt::Display> core::fmt::Display for Dimension3DType<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Dimensions3D<{}, {}, {}>", self.x, self.y, self.z)
    }
}

impl<T: QuantizableUInt> Into<Dimension3DUSize> for Dimension3DType<T> {
    fn into(self) -> Dimension3DUSize {
        Dimension3DUSize::new_unchecked(self.x as usize, self.y as usize, self.z as usize)
    }
}

//endregion

//endregion