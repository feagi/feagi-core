use crate::FeagiBaseError;

#[repr(transparent)]
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct NonzeroCount(usize);

impl NonzeroCount {
    pub fn new(n: usize) -> Result<Self, FeagiBaseError> {
        if n == 0 {
            return Err(FeagiBaseError::ValueCannotBeZero);
        }
    }
}
impl std::ops::Deref for NonzeroCount {
    type Target = NonzeroCount;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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
pub struct Coordinate2D {
    pub x: usize,
    pub y: usize,
}

impl Coordinate2D {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub fn new_with_fit_check(x: usize, y: usize, bounds: &Dimension2D) -> Result<Self, FeagiBaseError> {
        let coords = Self::new(x, y);
        bounds.verify_fit(coords)?;
        Ok(coords)
    }


}

impl std::fmt::Display for Coordinate2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    pub fn new(x: usize, y: usize) -> Result<Self, FeagiBaseError> {
        let x = NonzeroCount::new(x)?;
        let y = NonzeroCount::new(y)?;
        Ok(Self { x, y })

    }

    pub fn new_square(n: usize) -> Result<Self, FeagiBaseError> {
        let x = NonzeroCount::new(n)?;
        let y = x.clone();
        Ok(Self { x, y })
    }

    pub fn does_fit(&self, coordinate: &Coordinate2D) -> bool {
        coordinate.x < self.x && coordinate.y < self.y
    }

    pub fn verify_fit(&self, coordinate: &Coordinate2D) -> Result<(), FeagiBaseError> {
        if self.does_fit(coordinate) {
            Ok(())
        }
        Err(FeagiBaseError::Coordinate2DOutOfBounds{coordinate, dimensions: &self})
    }
}

impl std::fmt::Display for Dimension2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dimensions2D<{}, {}>", self.x, self.y)
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
pub struct Coordinate3D {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

impl Coordinate3D {
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        Self { x, y }
    }

    pub fn new_with_fit_check(x: usize, y: usize, z: usize, bounds: &Dimension3D) -> Result<Self, FeagiBaseError> {
        let coords = Self::new(x, y, z);
        bounds.verify_fit(coords)?;
        Ok(coords)
    }


}

impl std::fmt::Display for Coordinate3D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    pub fn new(x: usize, y: usize, z: usize) -> Result<Self, FeagiBaseError> {
        let x = NonzeroCount::new(x)?;
        let y = NonzeroCount::new(y)?;
        let z = NonzeroCount::new(z)?;
        Ok(Self { x, y, z })

    }

    pub fn new_cube(n: usize) -> Result<Self, FeagiBaseError> {
        let x = NonzeroCount::new(n)?;
        let y = x.clone();
        let z = x.clone();
        Ok(Self { x, y, z })
    }

    pub fn does_fit(&self, coordinate: &Coordinate3D) -> bool {
        coordinate.x < self.x && coordinate.y < self.y && coordinate.z < self.z
    }

    pub fn verify_fit(&self, coordinate: &Coordinate3D) -> Result<(), FeagiBaseError> {
        if self.does_fit(coordinate) {
            Ok(())
        }
        Err(FeagiBaseError::Coordinate3DOutOfBounds{coordinate, dimensions: &self})
    }
}

impl std::fmt::Display for Dimension3D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Dimensions3D<{}, {}, {}>", self.x, self.y, self.z)
    }
}