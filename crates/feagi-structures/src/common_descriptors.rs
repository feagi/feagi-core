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
pub struct Coordinate2DType<T: QuantizableUInt> {
    pub x: T,
    pub y: T,
}

pub type Coordinate2DU64 = Coordinate2DType<u64>;
pub type Coordinate2DU32 = Coordinate2DType<u32>;
pub type Coordinate2DU16 = Coordinate2DType<u16>;
pub type Coordinate2DU8 = Coordinate2DType<u8>;

impl<T: QuantizableUInt> Coordinate2DType<T> {

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

impl<T: QuantizableUInt> core::fmt::Display for Coordinate2DType<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Coordinate2D<{}, {}>", self.x, self.y)
    }
}

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

    pub fn does_fit(&self, coordinate: &Coordinate2DType<T>) -> bool {
        coordinate.x.lt(self.x.get()) && coordinate.y.lt(self.y.get())
    }

    pub fn verify_fit(&self, coordinate: &Coordinate2DType<T>) -> Result<(), FeagiBaseError> {
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

//endregion

//region  3D

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
pub struct Coordinate3DType<T: QuantizableUInt> {
    pub x: T,
    pub y: T,
    pub z: T,
}

pub type Coordinate3DU64 = Coordinate3DType<u64>;
pub type Coordinate3DU32 = Coordinate3DType<u32>;
pub type Coordinate3DU16 = Coordinate3DType<u16>;
pub type Coordinate3DU8 = Coordinate3DType<u8>;

impl<T: QuantizableUInt> Coordinate3DType<T> {

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

impl<T: QuantizableUInt + core::fmt::Display> core::fmt::Display for Coordinate3DType<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Coordinate3D<{}, {}, {}>", self.x, self.y, self.z)
    }
}

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

    pub fn does_fit(&self, coordinate: &Coordinate3DType<T>) -> bool {
        coordinate.x.lt(self.x.get())
            && coordinate.y.lt(self.y.get())
            && coordinate.z.lt(self.z.get())
    }

    pub fn verify_fit(&self, coordinate: &Coordinate3DType<T>) -> Result<(), FeagiBaseError> {
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

//endregion