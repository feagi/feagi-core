#[macro_export]
macro_rules! define_unsigned_coordinate_2d_type_family {
    ($base_name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize
        )]
        pub struct $base_name<T: $crate::base_quantizable::QuantizableUIntType> {
            pub x: T,
            pub y: T,
        }

        impl<T: $crate::base_quantizable::QuantizableUIntType> $base_name<T> {
            pub const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 2;

            #[inline(always)]
            pub fn new(x: T, y: T) -> Self {
                Self { x, y }
            }

            #[inline(always)]
            pub fn to_usize(self) -> $base_name<usize> {
                $base_name::new(self.x.to_usize(), self.y.to_usize())
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

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_quantizable::QuantizableUIntType + core::fmt::Display> core::fmt::Display for $base_name<T> {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, concat!(stringify!($base_name), "<{}, {}>"), self.x, self.y)
            }
        }
    };
}

#[macro_export]
macro_rules! define_signed_coordinate_2d_type_family {
    ($base_name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize
        )]
        pub struct $base_name<T: $crate::base_quantizable::QuantizableIntType> {
            pub x: T,
            pub y: T,
        }

        impl<T: $crate::base_quantizable::QuantizableIntType> $base_name<T> {
            pub const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 2;

            #[inline(always)]
            pub fn new(x: T, y: T) -> Self {
                Self { x, y }
            }

            /// Maps each component with [`QuantizableIntType::to_isize`]. Prefer this over [`Into`]
            /// to avoid overlapping the standard `From<T> for T` / `Into` blanket when `T` is `isize`.
            #[inline(always)]
            pub fn to_isize(self) -> $base_name<isize> {
                $base_name::new(self.x.to_isize(), self.y.to_isize())
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_quantizable::QuantizableIntType + core::fmt::Display> core::fmt::Display for $base_name<T> {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, concat!(stringify!($base_name), "<{}, {}>"), self.x, self.y)
            }
        }
    };
}

#[macro_export]
macro_rules! define_dimension_2d_type_family {
    ($base_name:ident, $coordinate_type:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize
        )]
        pub struct $base_name<T: $crate::base_quantizable::QuantizableUIntType> {
            pub x: $crate::base_quantizable::NonzeroCount<T>,
            pub y: $crate::base_quantizable::NonzeroCount<T>,
        }

        impl<T: $crate::base_quantizable::QuantizableUIntType> $base_name<T> {
            pub const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 2;

            #[inline(always)]
            pub(crate) fn new_unchecked(x: T, y: T) -> Self {
                let x = $crate::base_quantizable::NonzeroCount::new_unchecked(x);
                let y = $crate::base_quantizable::NonzeroCount::new_unchecked(y);
                Self { x, y }
            }

            #[inline(always)]
            pub fn new(x: T, y: T) -> Result<Self, $crate::FeagiStructuresError> {
                let x = $crate::base_quantizable::NonzeroCount::new(x)?;
                let y = $crate::base_quantizable::NonzeroCount::new(y)?;
                Ok(Self { x, y })
            }

            #[inline(always)]
            pub fn new_square(n: T) -> Result<Self, $crate::FeagiStructuresError> {
                let x = $crate::base_quantizable::NonzeroCount::new(n)?;
                let y = x;
                Ok(Self { x, y })
            }

            #[inline(always)]
            pub fn to_usize(self) -> $base_name<usize> {
                $base_name::new_unchecked(self.x.get().to_usize(), self.y.get().to_usize())
            }

            #[inline(always)]
            pub fn does_fit(&self, coordinate: &$coordinate_type<T>) -> bool {
                coordinate.x < self.x.get() && coordinate.y < self.y.get()
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_quantizable::QuantizableUIntType + core::fmt::Display> core::fmt::Display for $base_name<T> {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, concat!(stringify!($base_name), "<{}, {}>"), self.x, self.y)
            }
        }
    };
}

#[macro_export]
macro_rules! define_unsigned_coordinate_3d_type_family {
    ($base_name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize
        )]
        pub struct $base_name<T: $crate::base_quantizable::QuantizableUIntType> {
            pub x: T,
            pub y: T,
            pub z: T,
        }

        impl<T: $crate::base_quantizable::QuantizableUIntType> $base_name<T> {
            pub const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 3;
            #[inline(always)]
            pub fn new(x: T, y: T, z: T) -> Self {
                Self { x, y, z }
            }

            #[inline(always)]
            pub fn to_usize(self) -> $base_name<usize> {
                $base_name::new(self.x.to_usize(), self.y.to_usize(), self.z.to_usize())
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_quantizable::QuantizableUIntType + core::fmt::Display> core::fmt::Display for $base_name<T> {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(
                    f,
                    concat!(stringify!($base_name), "<{}, {}, {}>"),
                    self.x,
                    self.y,
                    self.z
                )
            }
        }
    };
}

#[macro_export]
macro_rules! define_signed_coordinate_3d_type_family {
    ($base_name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize
        )]
        pub struct $base_name<T: $crate::base_quantizable::QuantizableIntType> {
            pub x: T,
            pub y: T,
            pub z: T,
        }

        impl<T: $crate::base_quantizable::QuantizableIntType> $base_name<T> {
            pub const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 3;

            #[inline(always)]
            pub fn new(x: T, y: T, z: T) -> Self {
                Self { x, y, z }
            }

            #[inline(always)]
            pub fn to_isize(self) -> $base_name<isize> {
                $base_name::new(self.x.to_isize(), self.y.to_isize(), self.z.to_isize())
            }
        }

        // Additive tuple <-> struct conversions. Call sites in
        // higher-level crates (e.g. `feagi-services`) pass or receive
        // `(T, T, T)` for genome-editor payloads and rely on `.into()`
        // in both directions; expose the symmetric `From` impls here so
        // those callers don't have to reach into the struct fields
        // manually.
        impl<T: $crate::base_quantizable::QuantizableIntType> From<(T, T, T)> for $base_name<T> {
            #[inline(always)]
            fn from(value: (T, T, T)) -> Self {
                Self::new(value.0, value.1, value.2)
            }
        }

        impl<T: $crate::base_quantizable::QuantizableIntType> From<$base_name<T>> for (T, T, T) {
            #[inline(always)]
            fn from(value: $base_name<T>) -> Self {
                (value.x, value.y, value.z)
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_quantizable::QuantizableIntType + core::fmt::Display> core::fmt::Display for $base_name<T> {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(
                    f,
                    concat!(stringify!($base_name), "<{}, {}, {}>"),
                    self.x,
                    self.y,
                    self.z
                )
            }
        }
    };
}

#[macro_export]
macro_rules! define_dimension_3d_type_family {
    ($base_name:ident, $coordinate_type:ident) =>  {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize
        )]
        pub struct $base_name<T: $crate::base_quantizable::QuantizableUIntType> {
            pub x: $crate::base_quantizable::NonzeroCount<T>,
            pub y: $crate::base_quantizable::NonzeroCount<T>,
            pub z: $crate::base_quantizable::NonzeroCount<T>,
        }

        impl<T: $crate::base_quantizable::QuantizableUIntType> $base_name<T> {
            pub const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 3;

            #[inline(always)]
            pub(crate) fn new_unchecked(x: T, y: T, z: T) -> Self {
                let x = $crate::base_quantizable::NonzeroCount::new_unchecked(x);
                let y = $crate::base_quantizable::NonzeroCount::new_unchecked(y);
                let z = $crate::base_quantizable::NonzeroCount::new_unchecked(z);
                Self { x, y, z }
            }

            #[inline(always)]
            pub fn new(x: T, y: T, z: T) -> Result<Self, $crate::FeagiStructuresError> {
                let x = $crate::base_quantizable::NonzeroCount::new(x)?;
                let y = $crate::base_quantizable::NonzeroCount::new(y)?;
                let z = $crate::base_quantizable::NonzeroCount::new(z)?;
                Ok(Self { x, y, z })
            }

            #[inline(always)]
            pub fn new_cube(n: T) -> Result<Self, $crate::FeagiStructuresError> {
                let x = $crate::base_quantizable::NonzeroCount::new(n)?;
                let y = x;
                let z = x;
                Ok(Self { x, y, z })
            }

            #[inline(always)]
            pub fn to_usize(self) -> $base_name<usize> {
                $base_name::new_unchecked(
                    self.x.get().to_usize(),
                    self.y.get().to_usize(),
                    self.z.get().to_usize(),
                )
            }

            #[inline(always)]
            pub fn does_fit(&self, coordinate: &$coordinate_type<T>) -> bool {
                coordinate.x < self.x.get()
                    && coordinate.y < self.y.get()
                    && coordinate.z < self.z.get()
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_quantizable::QuantizableUIntType + core::fmt::Display> core::fmt::Display for $base_name<T> {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(
                    f,
                    concat!(stringify!($base_name), "<{}, {}, {}>"),
                    self.x,
                    self.y,
                    self.z
                )
            }
        }
    };
}
