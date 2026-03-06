use crate::FeagiBaseError;


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
pub struct NonzeroCount(u32);

impl NonzeroCount {

    pub const NUMBER_OF_BYTES: usize = size_of::<u32>();

    pub fn new(n: u32) -> Result<Self, FeagiBaseError> {
        if n == 0 {
            return Err(FeagiBaseError::ValueCannotBeZero);
        }
        Ok(Self(n))
    }
}

impl core::ops::Deref for NonzeroCount {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::fmt::Display for NonzeroCount {
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
pub struct Coordinate2D {
    pub x: u32,
    pub y: u32,
}

impl Coordinate2D {

    pub const NUMBER_OF_BYTES: usize = size_of::<u32>() * 2;

    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    pub fn new_with_fit_check(
        x: u32,
        y: u32,
        bounds: &Dimension2D,
    ) -> Result<Self, FeagiBaseError> {
        let coords = Self::new(x, y);
        bounds.verify_fit(&coords)?;
        Ok(coords)
    }
}

impl core::fmt::Display for Coordinate2D {
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
pub struct Dimension2D {
    pub x: NonzeroCount,
    pub y: NonzeroCount,
}

impl Dimension2D {

    pub const NUMBER_OF_BYTES: usize = size_of::<u32>() * 2;

    pub fn new(x: u32, y: u32) -> Result<Self, FeagiBaseError> {
        let x = NonzeroCount::new(x)?;
        let y = NonzeroCount::new(y)?;
        Ok(Self { x, y })
    }

    pub fn new_square(n: u32) -> Result<Self, FeagiBaseError> {
        let x = NonzeroCount::new(n)?;
        let y = x;
        Ok(Self { x, y })
    }

    pub fn does_fit(&self, coordinate: &Coordinate2D) -> bool {
        coordinate.x < self.x.get() && coordinate.y < self.y.get()
    }

    pub fn verify_fit(&self, coordinate: &Coordinate2D) -> Result<(), FeagiBaseError> {
        if self.does_fit(coordinate) {
            return Ok(());
        }
        Err(FeagiBaseError::Coordinate2DOutOfBounds {
            coordinate,
            dimensions: self,
        })
    }

    pub fn number_elements(&self) -> u32 {
        // TODO what if there is an overflow?
        *self.x * *self.y
    }
}

impl core::fmt::Display for Dimension2D {
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
pub struct Coordinate3D {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl Coordinate3D {

    pub const NUMBER_OF_BYTES: usize = size_of::<u32>() * 3;

    pub fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }

    pub fn new_with_fit_check(
        x: u32,
        y: u32,
        z: u32,
        bounds: &Dimension3D,
    ) -> Result<Self, FeagiBaseError> {
        let coords = Self::new(x, y, z);
        bounds.verify_fit(&coords)?;
        Ok(coords)
    }
}

impl core::fmt::Display for Coordinate3D {
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
pub struct Dimension3D {
    pub x: NonzeroCount,
    pub y: NonzeroCount,
    pub z: NonzeroCount,
}

impl Dimension3D {

    pub const NUMBER_OF_BYTES: usize = size_of::<u32>() * 3;

    pub fn new(x: u32, y: u32, z: u32) -> Result<Self, FeagiBaseError> {
        let x = NonzeroCount::new(x)?;
        let y = NonzeroCount::new(y)?;
        let z = NonzeroCount::new(z)?;
        Ok(Self { x, y, z })
    }

    pub fn new_cube(n: u32) -> Result<Self, FeagiBaseError> {
        let x = NonzeroCount::new(n)?;
        let y = x;
        let z = x;
        Ok(Self { x, y, z })
    }

    pub fn does_fit(&self, coordinate: &Coordinate3D) -> bool {
        coordinate.x < self.x.get()
            && coordinate.y < self.y.get()
            && coordinate.z < self.z.get()
    }

    pub fn verify_fit(&self, coordinate: &Coordinate3D) -> Result<(), FeagiBaseError> {
        if self.does_fit(coordinate) {
            return Ok(());
        }
        Err(FeagiBaseError::Coordinate3DOutOfBounds {
            coordinate,
            dimensions: self,
        })
    }

    pub fn number_elements(&self) -> u32 {
        // TODO what if there is an overflow?
        *self.x * *self.y * *self.z
    }
}

impl core::fmt::Display for Dimension3D {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Dimensions3D<{}, {}, {}>", self.x, self.y, self.z)
    }
}

//endregion