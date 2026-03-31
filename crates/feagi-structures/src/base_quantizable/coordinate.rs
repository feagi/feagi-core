/// Defines a full unsigned 2D coordinate type family. Pass `$dim_type` + `$display` to add
/// `new_with_fit_check` and a shorter `Display` label (e.g. `UnsignedCoordinate2D` for `UnsignedCoordinate2DType`).
#[macro_export]
macro_rules! define_unsigned_coordinate_2d_type_family {
    ($base_name:ident) => {
        $crate::define_unsigned_coordinate_2d_type_family!(@impl $base_name, $base_name);
    };
    ($base_name:ident, $dim_type:ident, $display:ident) => {
        $crate::define_unsigned_coordinate_2d_type_family!(@impl $base_name, $display, $dim_type);
    };
    (@impl $base_name:ident, $display:ident $(, $dim_type:ident)?) => {
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

            $(
            #[inline(always)]
            pub fn new_with_fit_check(
                x: T,
                y: T,
                bounds: &$dim_type<T>,
            ) -> Result<Self, $crate::FeagiStructuresError> {
                let coords = Self::new(x, y);
                bounds.verify_fit(&coords)?;
                Ok(coords)
            }
            )?
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt + core::fmt::Display> core::fmt::Display for $base_name<T> {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, concat!(stringify!($display), "<{}, {}>"), self.x, self.y)
            }
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> Into<$base_name<usize>> for $base_name<T> {
            #[inline(always)]
            fn into(self) -> $base_name<usize> {
                $base_name::new(self.x.to_usize(), self.y.to_usize())
            }
        }
    };
}

/// Defines a full signed 2D coordinate type family. Optional second ident is the `Display` prefix
/// (e.g. `SignedCoordinate2D` for struct `SignedCoordinate2DType`).
#[macro_export]
macro_rules! define_signed_coordinate_2d_type_family {
    ($base_name:ident) => {
        $crate::define_signed_coordinate_2d_type_family!(@impl $base_name, $base_name);
    };
    ($base_name:ident, $display:ident) => {
        $crate::define_signed_coordinate_2d_type_family!(@impl $base_name, $display);
    };
    (@impl $base_name:ident, $display:ident) => {
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
                write!(f, concat!(stringify!($display), "<{}, {}>"), self.x, self.y)
            }
        }

        impl<T: $crate::base_quantizable::signed_integer::QuantizableInt> Into<$base_name<isize>> for $base_name<T> {
            #[inline(always)]
            fn into(self) -> $base_name<isize> {
                $base_name::new(self.x.to_isize(), self.y.to_isize())
            }
        }
    };
}

/// Defines a full 2D dimension type family with fit checks. Optional third ident sets the `Display` prefix.
#[macro_export]
macro_rules! define_dimension_2d_type_family {
    ($base_name:ident, $coordinate_type:ident) => {
        $crate::define_dimension_2d_type_family!(@impl $base_name, $coordinate_type, $base_name);
    };
    ($base_name:ident, $coordinate_type:ident, $display:ident) => {
        $crate::define_dimension_2d_type_family!(@impl $base_name, $coordinate_type, $display);
    };
    (@impl $base_name:ident, $coordinate_type:ident, $display:ident) => {
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
                write!(f, concat!(stringify!($display), "<{}, {}>"), self.x, self.y)
            }
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> Into<$base_name<usize>> for $base_name<T> {
            #[inline(always)]
            fn into(self) -> $base_name<usize> {
                $base_name::new_unchecked(self.x.get().to_usize(), self.y.get().to_usize())
            }
        }
    };
}

/// Defines a full unsigned 3D coordinate type family. See [`define_unsigned_coordinate_2d_type_family`].
#[macro_export]
macro_rules! define_unsigned_coordinate_3d_type_family {
    ($base_name:ident) => {
        $crate::define_unsigned_coordinate_3d_type_family!(@impl $base_name, $base_name);
    };
    ($base_name:ident, $dim_type:ident, $display:ident) => {
        $crate::define_unsigned_coordinate_3d_type_family!(@impl $base_name, $display, $dim_type);
    };
    (@impl $base_name:ident, $display:ident $(, $dim_type:ident)?) => {
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

            $(
            #[inline(always)]
            pub fn new_with_fit_check(
                x: T,
                y: T,
                z: T,
                bounds: &$dim_type<T>,
            ) -> Result<Self, $crate::FeagiStructuresError> {
                let coords = Self::new(x, y, z);
                bounds.verify_fit(&coords)?;
                Ok(coords)
            }
            )?
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt + core::fmt::Display> core::fmt::Display for $base_name<T> {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(
                    f,
                    concat!(stringify!($display), "<{}, {}, {}>"),
                    self.x,
                    self.y,
                    self.z
                )
            }
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> Into<$base_name<usize>> for $base_name<T> {
            #[inline(always)]
            fn into(self) -> $base_name<usize> {
                $base_name::new(self.x.to_usize(), self.y.to_usize(), self.z.to_usize())
            }
        }
    };
}

/// Defines a full signed 3D coordinate type family. Optional second ident is the `Display` prefix.
#[macro_export]
macro_rules! define_signed_coordinate_3d_type_family {
    ($base_name:ident) => {
        $crate::define_signed_coordinate_3d_type_family!(@impl $base_name, $base_name);
    };
    ($base_name:ident, $display:ident) => {
        $crate::define_signed_coordinate_3d_type_family!(@impl $base_name, $display);
    };
    (@impl $base_name:ident, $display:ident) => {
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
                write!(
                    f,
                    concat!(stringify!($display), "<{}, {}, {}>"),
                    self.x,
                    self.y,
                    self.z
                )
            }
        }

        impl<T: $crate::base_quantizable::signed_integer::QuantizableInt> Into<$base_name<isize>> for $base_name<T> {
            #[inline(always)]
            fn into(self) -> $base_name<isize> {
                $base_name::new(self.x.to_isize(), self.y.to_isize(), self.z.to_isize())
            }
        }
    };
}

/// Defines a full 3D dimension type family with fit checks. Optional third ident sets the `Display` prefix.
#[macro_export]
macro_rules! define_dimension_3d_type_family {
    ($base_name:ident, $coordinate_type:ident) => {
        $crate::define_dimension_3d_type_family!(@impl $base_name, $coordinate_type, $base_name);
    };
    ($base_name:ident, $coordinate_type:ident, $display:ident) => {
        $crate::define_dimension_3d_type_family!(@impl $base_name, $coordinate_type, $display);
    };
    (@impl $base_name:ident, $coordinate_type:ident, $display:ident) => {
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
                write!(
                    f,
                    concat!(stringify!($display), "<{}, {}, {}>"),
                    self.x,
                    self.y,
                    self.z
                )
            }
        }

        impl<T: $crate::base_quantizable::unsigned_integer::QuantizableUInt> Into<$base_name<usize>> for $base_name<T> {
            #[inline(always)]
            fn into(self) -> $base_name<usize> {
                $base_name::new_unchecked(
                    self.x.get().to_usize(),
                    self.y.get().to_usize(),
                    self.z.get().to_usize(),
                )
            }
        }
    };
}

//region 2D — canonical layouts used by errors and fit checks (see macros above)

crate::define_unsigned_coordinate_2d_type_family!(UnsignedCoordinate2DType, Dimension2DType, UnsignedCoordinate2D);
crate::define_signed_coordinate_2d_type_family!(SignedCoordinate2DType, SignedCoordinate2D);
crate::define_dimension_2d_type_family!(Dimension2DType, UnsignedCoordinate2DType, Dimensions2D);

//endregion

//region 3D

crate::define_unsigned_coordinate_3d_type_family!(UnsignedCoordinate3DType, Dimension3DType, UnsignedCoordinate3D);
crate::define_signed_coordinate_3d_type_family!(SignedCoordinate3DType, SignedCoordinate3D);
crate::define_dimension_3d_type_family!(Dimension3DType, UnsignedCoordinate3DType, Dimensions3D);

//endregion
