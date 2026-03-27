use crate::base_quantizable::signed_integer::QuantizableInt;
use crate::base_quantizable::nonzero_count::NonzeroCountType;
use crate::FeagiStructuresError;
use crate::base_quantizable::unsigned_integer::QuantizableUInt;

/// Defines a full unsigned 2D coordinate type family with fit-check helpers.
#[macro_export]
macro_rules! define_unsigned_coordinate_2d_type_family {
    ($base_name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize
        )]
        pub struct $base_name<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> {
            pub x: T,
            pub y: T,
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> $base_name<T> {
            pub const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 2;

            #[inline(always)]
            pub fn new(x: T, y: T) -> Self {
                Self { x, y }
            }
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt + core::fmt::Display> core::fmt::Display for $base_name<T> {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, concat!(stringify!($base_name), "<{}, {}>"), self.x, self.y)
            }
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> Into<$base_name> for $base_name<T> {
            #[inline(always)]
            fn into(self) -> $base_name {
                $base_name::new(self.x.to_usize(), self.y.to_usize())
            }
        }
    };
}

/// Defines a full signed 2D coordinate type family.
#[macro_export]
macro_rules! define_signed_coordinate_2d_type_family {
    ($base_name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize
        )]
        pub struct $base_name<T: $crate::base_quantizable::signed_integer::QuantizableInt> {
            pub x: T,
            pub y: T,
        }

        impl<T: $crate::base_quantizable::signed_integer::QuantizableInt> $base_name<T> {
            pub const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 2;

            #[inline(always)]
            pub fn new(x: T, y: T) -> Self {
                Self { x, y }
            }
        }

        impl<T: $crate::base_quantizable::signed_integer::QuantizableInt + core::fmt::Display> core::fmt::Display for $base_name<T> {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, concat!(stringify!($base_name), "<{}, {}>"), self.x, self.y)
            }
        }

        impl<T: $crate::base_quantizable::signed_integer::QuantizableInt> Into<$base_name> for $base_name<T> {
            #[inline(always)]
            fn into(self) -> $base_name {
                $base_name::new(self.x.to_isize(), self.y.to_isize())
            }
        }
    };
}

/// Defines a full 2D dimension type family with fit checks.
#[macro_export]
macro_rules! define_dimension_2d_type_family {
    ($base_name:ident, $coordinate_type:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize
        )]
        pub struct $base_name<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> {
            pub x: $crate::base_quantizable::nonzero_count::NonzeroCountType<T>,
            pub y: $crate::base_quantizable::nonzero_count::NonzeroCountType<T>,
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> $base_name<T> {
            pub const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 2;

            #[inline(always)]
            pub(crate) fn new_unchecked(x: T, y: T) -> Self {
                let x = $crate::base_quantizable::nonzero_count::NonzeroCountType::new_unchecked(x);
                let y = $crate::base_quantizable::nonzero_count::NonzeroCountType::new_unchecked(y);
                Self { x, y }
            }

            #[inline(always)]
            pub fn new(x: T, y: T) -> Result<Self, $crate::FeagiStructuresError> {
                let x = $crate::base_quantizable::nonzero_count::NonzeroCountType::new(x)?;
                let y = $crate::base_quantizable::nonzero_count::NonzeroCountType::new(y)?;
                Ok(Self { x, y })
            }

            #[inline(always)]
            pub fn new_square(n: T) -> Result<Self, $crate::FeagiStructuresError> {
                let x = $crate::base_quantizable::nonzero_count::NonzeroCountType::new(n)?;
                let y = x;
                Ok(Self { x, y })
            }

            #[inline(always)]
            pub fn does_fit(&self, coordinate: &$coordinate_type<T>) -> bool {
                coordinate.x < self.x.get() && coordinate.y < self.y.get()
            }

            #[inline(always)]
            pub fn verify_fit(&self, coordinate: &$coordinate_type<T>) -> Result<(), $crate::FeagiStructuresError> {
                if self.does_fit(coordinate) {
                    Ok(())
                } else {
                    Err($crate::FeagiStructuresError::Coordinate2DOutOfBounds {
                        context: "coordinate does not fit in 2D bounds",
                        coordinate: (*coordinate).into(),
                        dimensions: (*self).into(),
                    })
                }
            }
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt + core::fmt::Display> core::fmt::Display for $base_name<T> {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, concat!(stringify!($base_name), "<{}, {}>"), self.x, self.y)
            }
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> Into<$base_name> for $base_name<T> {
            #[inline(always)]
            fn into(self) -> $base_name {
                $base_name::new_unchecked(self.x.get().to_usize(), self.y.get().to_usize())
            }
        }
    };
}

/// Defines a full unsigned 3D coordinate type family with fit-check helpers.
#[macro_export]
macro_rules! define_unsigned_coordinate_3d_type_family {
    ($base_name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize
        )]
        pub struct $base_name<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> {
            pub x: T,
            pub y: T,
            pub z: T,
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> $base_name<T> {
            pub const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 3;

            #[inline(always)]
            pub fn new(x: T, y: T, z: T) -> Self {
                Self { x, y, z }
            }
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt + core::fmt::Display> core::fmt::Display for $base_name<T> {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, concat!(stringify!($base_name), "<{}, {}, {}>"), self.x, self.y, self.z)
            }
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> Into<$base_name> for $base_name<T> {
            #[inline(always)]
            fn into(self) -> $base_name {
                $base_name::new(self.x.to_usize(), self.y.to_usize(), self.z.to_usize())
            }
        }
    };
}

/// Defines a full signed 3D coordinate type family.
#[macro_export]
macro_rules! define_signed_coordinate_3d_type_family {
    ($base_name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize
        )]
        pub struct $base_name<T: $crate::base_quantizable::signed_integer::QuantizableInt> {
            pub x: T,
            pub y: T,
            pub z: T,
        }

        impl<T: $crate::base_quantizable::signed_integer::QuantizableInt> $base_name<T> {
            pub const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 3;

            #[inline(always)]
            pub fn new(x: T, y: T, z: T) -> Self {
                Self { x, y, z }
            }
        }

        impl<T: $crate::base_quantizable::signed_integer::QuantizableInt + core::fmt::Display> core::fmt::Display for $base_name<T> {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, concat!(stringify!($base_name), "<{}, {}, {}>"), self.x, self.y, self.z)
            }
        }

        impl<T: $crate::base_quantizable::signed_integer::QuantizableInt> Into<$base_name> for $base_name<T> {
            #[inline(always)]
            fn into(self) -> $base_name {
                $base_name::new(self.x.to_isize(), self.y.to_isize(), self.z.to_isize())
            }
        }
    };
}

/// Defines a full 3D dimension type family with fit checks.
#[macro_export]
macro_rules! define_dimension_3d_type_family {
    ($base_name:ident, $coordinate_type:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize
        )]
        pub struct $base_name<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> {
            pub x: $crate::base_quantizable::nonzero_count::NonzeroCountType<T>,
            pub y: $crate::base_quantizable::nonzero_count::NonzeroCountType<T>,
            pub z: $crate::base_quantizable::nonzero_count::NonzeroCountType<T>,
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> $base_name<T> {
            pub const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 3;

            #[inline(always)]
            pub(crate) fn new_unchecked(x: T, y: T, z: T) -> Self {
                let x = $crate::base_quantizable::nonzero_count::NonzeroCountType::new_unchecked(x);
                let y = $crate::base_quantizable::nonzero_count::NonzeroCountType::new_unchecked(y);
                let z = $crate::base_quantizable::nonzero_count::NonzeroCountType::new_unchecked(z);
                Self { x, y, z }
            }

            #[inline(always)]
            pub fn new(x: T, y: T, z: T) -> Result<Self, $crate::FeagiStructuresError> {
                let x = $crate::base_quantizable::nonzero_count::NonzeroCountType::new(x)?;
                let y = $crate::base_quantizable::nonzero_count::NonzeroCountType::new(y)?;
                let z = $crate::base_quantizable::nonzero_count::NonzeroCountType::new(z)?;
                Ok(Self { x, y, z })
            }

            #[inline(always)]
            pub fn new_cube(n: T) -> Result<Self, $crate::FeagiStructuresError> {
                let x = $crate::base_quantizable::nonzero_count::NonzeroCountType::new(n)?;
                let y = x;
                let z = x;
                Ok(Self { x, y, z })
            }

            #[inline(always)]
            pub fn does_fit(&self, coordinate: &$coordinate_type<T>) -> bool {
                coordinate.x < self.x.get()
                    && coordinate.y < self.y.get()
                    && coordinate.z < self.z.get()
            }

            #[inline(always)]
            pub fn verify_fit(&self, coordinate: &$coordinate_type<T>) -> Result<(), $crate::FeagiStructuresError> {
                if self.does_fit(coordinate) {
                    Ok(())
                } else {
                    Err($crate::FeagiStructuresError::Coordinate3DOutOfBounds {
                        context: "coordinate does not fit in 3D bounds",
                        coordinate: (*coordinate).into(),
                        dimensions: (*self).into(),
                    })
                }
            }
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt + core::fmt::Display> core::fmt::Display for $base_name<T> {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, concat!(stringify!($base_name), "<{}, {}, {}>"), self.x, self.y, self.z)
            }
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> Into<$base_name> for $base_name<T> {
            #[inline(always)]
            fn into(self) -> $base_name {
                $base_name::new_unchecked(
                    self.x.get().to_usize(),
                    self.y.get().to_usize(),
                    self.z.get().to_usize(),
                )
            }
        }
    };
}


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

impl<T: QuantizableUInt> UnsignedCoordinate2DType<T> {

    pub const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 2;

    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    pub fn new_with_fit_check(
        x: T,
        y: T,
        bounds: &Dimension2DType<T>,
    ) -> Result<Self, FeagiStructuresError> {
        let coords = Self::new(x, y);
        bounds.verify_fit(&coords)?;
        Ok(coords)
    }
}

impl<T: QuantizableUInt + core::fmt::Display> core::fmt::Display for UnsignedCoordinate2DType<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "UnsignedCoordinate2D<{}, {}>", self.x, self.y)
    }
}

impl<T: QuantizableUInt> Into<UnsignedCoordinate2DType<usize>> for UnsignedCoordinate2DType<T> {
    fn into(self) -> UnsignedCoordinate2DType<usize> {
        UnsignedCoordinate2DType::new(self.x.to_usize(), self.y.to_usize())
    }
}

//endregion

//region Signed Coordinate 2D
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
pub struct SignedCoordinate2DType<T: QuantizableInt> {
    pub x: T,
    pub y: T,
}

impl<T: QuantizableInt> SignedCoordinate2DType<T> {

    pub const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 2;
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T: QuantizableInt + core::fmt::Display> core::fmt::Display for SignedCoordinate2DType<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SignedCoordinate2D<{}, {}>", self.x, self.y)
    }
}

impl<T: QuantizableInt> Into<SignedCoordinate2DType<isize>> for SignedCoordinate2DType<T> {
    fn into(self) -> SignedCoordinate2DType<isize> {
        SignedCoordinate2DType::new(self.x.to_isize(), self.y.to_isize())
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

impl<T: QuantizableUInt> Dimension2DType<T> {

    pub const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 2;

    pub(crate) fn new_unchecked(x: T, y: T) -> Self {
        let x = NonzeroCountType::new_unchecked(x);
        let y = NonzeroCountType::new_unchecked(y);
        Self { x, y }
    }

    pub fn new(x: T, y: T) -> Result<Self, FeagiStructuresError> {
        let x = NonzeroCountType::new(x)?;
        let y = NonzeroCountType::new(y)?;
        Ok(Self { x, y })
    }

    pub fn new_square(n: T) -> Result<Self, FeagiStructuresError> {
        let x = NonzeroCountType::new(n)?;
        let y = x;
        Ok(Self { x, y })
    }

    pub fn does_fit(&self, coordinate: &UnsignedCoordinate2DType<T>) -> bool {
        coordinate.x < self.x.get() && coordinate.y < self.y.get()
    }

    pub fn verify_fit(&self, coordinate: &UnsignedCoordinate2DType<T>) -> Result<(), FeagiStructuresError> {
        if self.does_fit(coordinate) {
            return Ok(());
        }
        Err(FeagiStructuresError::Coordinate2DOutOfBounds {
            context: "coordinate does not fit in 2D bounds",
            coordinate: (*coordinate).into(),
            dimensions: (*self).into(),
        })
    }
}

impl<T: QuantizableUInt + core::fmt::Display> core::fmt::Display for Dimension2DType<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Dimensions2D<{}, {}>", self.x, self.y)
    }
}

impl<T: QuantizableUInt> Into<Dimension2DType<usize>> for Dimension2DType<T> {
    fn into(self) -> Dimension2DType<usize> {
        Dimension2DType::new_unchecked(self.x.get().to_usize(), self.y.get().to_usize())
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
    ) -> Result<Self, FeagiStructuresError> {
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

impl<T: QuantizableUInt> Into<UnsignedCoordinate3DType<usize>> for UnsignedCoordinate3DType<T> {
    fn into(self) -> UnsignedCoordinate3DType<usize> {
        UnsignedCoordinate3DType::new(self.x.to_usize(), self.y.to_usize(), self.z.to_usize())
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

impl<T: QuantizableInt> SignedCoordinate3DType<T> {

    pub const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 3;

    pub fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }
}

impl<T: QuantizableInt + core::fmt::Display> core::fmt::Display for SignedCoordinate3DType<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SignedCoordinate3D<{}, {}, {}>", self.x, self.y, self.z)
    }
}

impl<T: QuantizableInt> Into<SignedCoordinate3DType<isize>> for SignedCoordinate3DType<T> {
    fn into(self) -> SignedCoordinate3DType<isize> {
        SignedCoordinate3DType::new(self.x.to_isize(), self.y.to_isize(), self.z.to_isize())
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

impl<T: QuantizableUInt> Dimension3DType<T> {

    pub const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 3;

    pub(crate) fn new_unchecked(x: T, y: T, z: T) -> Self {
        let x = NonzeroCountType::new_unchecked(x);
        let y = NonzeroCountType::new_unchecked(y);
        let z = NonzeroCountType::new_unchecked(z);
        Self { x, y, z }
    }

    pub fn new(x: T, y: T, z: T) -> Result<Self, FeagiStructuresError> {
        let x = NonzeroCountType::new(x)?;
        let y = NonzeroCountType::new(y)?;
        let z = NonzeroCountType::new(z)?;
        Ok(Self { x, y, z })
    }

    pub fn new_cube(n: T) -> Result<Self, FeagiStructuresError> {
        let x = NonzeroCountType::new(n)?;
        let y = x;
        let z = x;
        Ok(Self { x, y, z })
    }

    pub fn does_fit(&self, coordinate: &UnsignedCoordinate3DType<T>) -> bool {
        coordinate.x < self.x.get()
            && coordinate.y < self.y.get()
            && coordinate.z < self.z.get()
    }

    pub fn verify_fit(&self, coordinate: &UnsignedCoordinate3DType<T>) -> Result<(), FeagiStructuresError> {
        if self.does_fit(coordinate) {
            return Ok(());
        }
        Err(FeagiStructuresError::Coordinate3DOutOfBounds {
            context: "coordinate does not fit in 3D bounds",
            coordinate: (*coordinate).into(),
            dimensions: (*self).into(),
        })
    }
}

impl<T: QuantizableUInt + core::fmt::Display> core::fmt::Display for Dimension3DType<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Dimensions3D<{}, {}, {}>", self.x, self.y, self.z)
    }
}

impl<T: QuantizableUInt> Into<Dimension3DType<usize>> for Dimension3DType<T> {
    fn into(self) -> Dimension3DType<usize> {
        Dimension3DType::new_unchecked(
            self.x.get().to_usize(),
            self.y.get().to_usize(),
            self.z.get().to_usize(),
        )
    }
}

//endregion

//endregion