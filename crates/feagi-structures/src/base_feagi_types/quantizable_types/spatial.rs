use crate::base_feagi_types::quantizable_types::shared::FeagiBaseQuantizationType;
use crate::base_feagi_types::quantizable_types::shared::FeagiBaseMultiElementQuantizationType;
use crate::define_nonzero_count_family;

define_nonzero_count_family!(SpatialDimensionAxis);


//region 2D

//region Unsigned Coordinate

#[macro_export]
macro_rules! define_unsigned_coordinate_2d_type_family {
    ($base_name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[cfg_attr(feature = "alloc", derive(serde::Serialize, serde::Deserialize))]
        pub struct $base_name<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> {
            pub x: T,
            pub y: T,
        }

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> $base_name<T> {
            #[inline(always)]
            pub const fn new(x: T, y: T) -> Self {
                Self { x, y }
            }

            #[inline(always)]
            pub fn to_usize(self) -> $base_name<usize> {
                $base_name::new(self.x.to_usize(), self.y.to_usize())
            }
        }

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> From<(T, T)> for $base_name<T> {
            #[inline(always)]
            fn from(value: (T, T)) -> Self {
                Self::new(value.0, value.1)
            }
        }

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> From<$base_name<T>> for (T, T) {
            #[inline(always)]
            fn from(value: $base_name<T>) -> Self {
                (value.x, value.y)
            }
        }

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::Add for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn add(self, rhs: Self) -> Self::Output {
                Self::new(self.x + rhs.x, self.y + rhs.y)
            }
        }

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::Sub for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn sub(self, rhs: Self) -> Self::Output {
                Self::new(self.x - rhs.x, self.y - rhs.y)
            }
        }

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::Mul for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn mul(self, rhs: Self) -> Self::Output {
                Self::new(self.x * rhs.x, self.y * rhs.y)
            }
        }

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::Div for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn div(self, rhs: Self) -> Self::Output {
                Self::new(self.x / rhs.x, self.y / rhs.y)
            }
        }

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::AddAssign for $base_name<T> {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                self.x += rhs.x;
                self.y += rhs.y;
            }
        }

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::SubAssign for $base_name<T> {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                self.x -= rhs.x;
                self.y -= rhs.y;
            }
        }

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::MulAssign for $base_name<T> {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: Self) {
                self.x *= rhs.x;
                self.y *= rhs.y;
            }
        }

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::DivAssign for $base_name<T> {
            #[inline(always)]
            fn div_assign(&mut self, rhs: Self) {
                self.x /= rhs.x;
                self.y /= rhs.y;
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> Default for $base_name<T> {
            #[inline(always)]
            fn default() -> Self {
                Self::new(T::ZERO, T::ZERO)
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType + core::fmt::Display> core::fmt::Display for $base_name<T> {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, concat!(stringify!($base_name), "<{}, {}>"), self.x, self.y)
            }
        }

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> $crate::base_feagi_types::quantizable_types::FeagiBaseQuantizationType for $base_name<T> {
            const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 2;

            #[inline(always)]
            fn saturating_add(self, other: Self) -> Self {
                Self::new(
                    self.x.saturating_add(other.x),
                    self.y.saturating_add(other.y),
                )
            }

            #[inline(always)]
            fn checked_add(self, other: Self) -> Option<Self> {
                Some(Self::new(
                    self.x.checked_add(other.x)?,
                    self.y.checked_add(other.y)?,
                ))
            }

            #[inline(always)]
            fn saturating_sub(self, other: Self) -> Self {
                Self::new(
                    self.x.saturating_sub(other.x),
                    self.y.saturating_sub(other.y),
                )
            }

            #[inline(always)]
            fn checked_sub(self, other: Self) -> Option<Self> {
                Some(Self::new(
                    self.x.checked_sub(other.x)?,
                    self.y.checked_sub(other.y)?,
                ))
            }

            #[inline(always)]
            fn saturating_mul(self, other: Self) -> Self {
                Self::new(
                    self.x.saturating_mul(other.x),
                    self.y.saturating_mul(other.y),
                )
            }

            #[inline(always)]
            fn checked_mul(self, other: Self) -> Option<Self> {
                Some(Self::new(
                    self.x.checked_mul(other.x)?,
                    self.y.checked_mul(other.y)?,
                ))
            }

            #[inline(always)]
            fn checked_div(self, other: Self) -> Option<Self> {
                Some(Self::new(
                    self.x.checked_div(other.x)?,
                    self.y.checked_div(other.y)?,
                ))
            }
        }

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> $crate::base_feagi_types::quantizable_types::FeagiBaseMultiElementQuantizationType for $base_name<T> {
            const NUMBER_ELEMENTS: usize = 2;
            const ALL_ZEROS: Self = Self::new(T::ZERO, T::ZERO);
            const ALL_ONES: Self = Self::new(T::ONE, T::ONE);

            type ElementType = T;
        }

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> $crate::base_feagi_types::quantizable_types::spatial::QuantizableUInt2DCoordinateType
            for $base_name<T>
        {
            #[inline(always)]
            fn from_tuple(tuple: (Self::ElementType, Self::ElementType)) -> Self {
                Self::new(tuple.0, tuple.1)
            }

            #[inline(always)]
            fn from_usize_tuple(usize_tuple: (usize, usize)) -> Self {
                Self::new(T::from_usize(usize_tuple.0), T::from_usize(usize_tuple.1))
            }

            #[inline(always)]
            fn to_tuple(self) -> (Self::ElementType, Self::ElementType) {
                (self.x, self.y)
            }

            #[inline(always)]
            fn to_usize_tuple(self) -> (usize, usize) {
                (self.x.to_usize(), self.y.to_usize())
            }
        }

    };
}

pub trait QuantizableUInt2DCoordinateType:
FeagiBaseMultiElementQuantizationType
{
    fn from_tuple(tuple: (Self::ElementType, Self::ElementType)) -> Self;
    fn from_usize_tuple(usize_tuple: (usize, usize)) -> Self;
    fn to_tuple(self) -> (Self::ElementType, Self::ElementType);
    fn to_usize_tuple(self) -> (usize, usize);
}

//endregion

//region Signed Coordinate

#[macro_export]
macro_rules! define_signed_coordinate_2d_type_family {
    ($base_name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[cfg_attr(feature = "alloc", derive(serde::Serialize, serde::Deserialize))]
        pub struct $base_name<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> {
            pub x: T,
            pub y: T,
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> $base_name<T> {
            #[inline(always)]
            pub const fn new(x: T, y: T) -> Self {
                Self { x, y }
            }

            #[inline(always)]
            pub fn to_isize(self) -> $base_name<isize> {
                $base_name::new(self.x.to_isize(), self.y.to_isize())
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> From<(T, T)> for $base_name<T> {
            #[inline(always)]
            fn from(value: (T, T)) -> Self {
                Self::new(value.0, value.1)
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> From<$base_name<T>> for (T, T) {
            #[inline(always)]
            fn from(value: $base_name<T>) -> Self {
                (value.x, value.y)
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> core::ops::Add for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn add(self, rhs: Self) -> Self::Output {
                Self::new(self.x + rhs.x, self.y + rhs.y)
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> core::ops::Sub for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn sub(self, rhs: Self) -> Self::Output {
                Self::new(self.x - rhs.x, self.y - rhs.y)
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> core::ops::Mul for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn mul(self, rhs: Self) -> Self::Output {
                Self::new(self.x * rhs.x, self.y * rhs.y)
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> core::ops::Div for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn div(self, rhs: Self) -> Self::Output {
                Self::new(self.x / rhs.x, self.y / rhs.y)
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> core::ops::AddAssign for $base_name<T> {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                self.x += rhs.x;
                self.y += rhs.y;
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> core::ops::SubAssign for $base_name<T> {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                self.x -= rhs.x;
                self.y -= rhs.y;
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> core::ops::MulAssign for $base_name<T> {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: Self) {
                self.x *= rhs.x;
                self.y *= rhs.y;
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> core::ops::DivAssign for $base_name<T> {
            #[inline(always)]
            fn div_assign(&mut self, rhs: Self) {
                self.x /= rhs.x;
                self.y /= rhs.y;
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> Default for $base_name<T> {
            #[inline(always)]
            fn default() -> Self {
                Self::new(T::ZERO, T::ZERO)
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType + core::fmt::Display> core::fmt::Display for $base_name<T> {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, concat!(stringify!($base_name), "<{}, {}>"), self.x, self.y)
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> $crate::base_feagi_types::quantizable_types::FeagiBaseQuantizationType for $base_name<T> {
            const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 2;

            #[inline(always)]
            fn saturating_add(self, other: Self) -> Self {
                Self::new(
                    self.x.saturating_add(other.x),
                    self.y.saturating_add(other.y),
                )
            }

            #[inline(always)]
            fn checked_add(self, other: Self) -> Option<Self> {
                Some(Self::new(
                    self.x.checked_add(other.x)?,
                    self.y.checked_add(other.y)?,
                ))
            }

            #[inline(always)]
            fn saturating_sub(self, other: Self) -> Self {
                Self::new(
                    self.x.saturating_sub(other.x),
                    self.y.saturating_sub(other.y),
                )
            }

            #[inline(always)]
            fn checked_sub(self, other: Self) -> Option<Self> {
                Some(Self::new(
                    self.x.checked_sub(other.x)?,
                    self.y.checked_sub(other.y)?,
                ))
            }

            #[inline(always)]
            fn saturating_mul(self, other: Self) -> Self {
                Self::new(
                    self.x.saturating_mul(other.x),
                    self.y.saturating_mul(other.y),
                )
            }

            #[inline(always)]
            fn checked_mul(self, other: Self) -> Option<Self> {
                Some(Self::new(
                    self.x.checked_mul(other.x)?,
                    self.y.checked_mul(other.y)?,
                ))
            }

            #[inline(always)]
            fn checked_div(self, other: Self) -> Option<Self> {
                Some(Self::new(
                    self.x.checked_div(other.x)?,
                    self.y.checked_div(other.y)?,
                ))
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> $crate::base_feagi_types::quantizable_types::FeagiBaseMultiElementQuantizationType for $base_name<T> {
            const NUMBER_ELEMENTS: usize = 2;
            const ALL_ZEROS: Self = Self::new(T::ZERO, T::ZERO);
            const ALL_ONES: Self = Self::new(T::ONE, T::ONE);

            type ElementType = T;
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> $crate::base_feagi_types::quantizable_types::spatial::QuantizableInt2DCoordinateType
            for $base_name<T>
        {
            #[inline(always)]
            fn from_tuple(tuple: (Self::ElementType, Self::ElementType)) -> Self {
                Self::new(tuple.0, tuple.1)
            }

            #[inline(always)]
            fn from_isize_tuple(isize_tuple: (isize, isize)) -> Self {
                Self::new(T::from_isize(isize_tuple.0), T::from_isize(isize_tuple.1))
            }

            #[inline(always)]
            fn to_tuple(self) -> (Self::ElementType, Self::ElementType) {
                (self.x, self.y)
            }

            #[inline(always)]
            fn to_isize_tuple(self) -> (isize, isize) {
                (self.x.to_isize(), self.y.to_isize())
            }
        }
    };
}

pub trait QuantizableInt2DCoordinateType:
FeagiBaseMultiElementQuantizationType
{
    fn from_tuple(tuple: (Self::ElementType, Self::ElementType)) -> Self;
    fn from_isize_tuple(isize_tuple: (isize, isize)) -> Self;
    fn to_tuple(self) -> (Self::ElementType, Self::ElementType);
    fn to_isize_tuple(self) -> (isize, isize);
}

//endregion

//region Dimensions

#[macro_export]
macro_rules! define_dimension_2d_type_family {
    ($base_name:ident, $coordinate_type:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[cfg_attr(feature = "alloc", derive(serde::Serialize, serde::Deserialize))]
        pub struct $base_name<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> {
            pub x: $crate::base_feagi_types::quantizable_types::spatial::SpatialDimensionAxis<T>,
            pub y: $crate::base_feagi_types::quantizable_types::spatial::SpatialDimensionAxis<T>,
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> $base_name<T> {
            #[inline(always)]
            pub(crate) const fn new_unchecked(x: T, y: T) -> Self {
                let x = $crate::base_feagi_types::quantizable_types::spatial::SpatialDimensionAxis::new_unchecked(x);
                let y = $crate::base_feagi_types::quantizable_types::spatial::SpatialDimensionAxis::new_unchecked(y);
                Self { x, y }
            }

            #[inline(always)]
            pub fn new(x: T, y: T) -> Result<Self, $crate::FeagiStructuresError> {
                let x = $crate::base_feagi_types::quantizable_types::spatial::SpatialDimensionAxis::new(x)?;
                let y = $crate::base_feagi_types::quantizable_types::spatial::SpatialDimensionAxis::new(y)?;
                Ok(Self { x, y })
            }

            #[inline(always)]
            pub fn new_square(n: T) -> Result<Self, $crate::FeagiStructuresError> {
                let x = $crate::base_feagi_types::quantizable_types::spatial::SpatialDimensionAxis::new(n)?;
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

            #[inline(always)]
            pub fn number_elements(&self) -> usize {
                (self.x * self.y).to_usize()
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::Add for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn add(self, rhs: Self) -> Self::Output {
                Self {
                    x: self.x + rhs.x,
                    y: self.y + rhs.y,
                }
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::Sub for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn sub(self, rhs: Self) -> Self::Output {
                Self {
                    x: self.x - rhs.x,
                    y: self.y - rhs.y,
                }
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::Mul for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn mul(self, rhs: Self) -> Self::Output {
                Self {
                    x: self.x * rhs.x,
                    y: self.y * rhs.y,
                }
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::Div for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn div(self, rhs: Self) -> Self::Output {
                Self {
                    x: self.x / rhs.x,
                    y: self.y / rhs.y,
                }
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::AddAssign for $base_name<T> {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                self.x += rhs.x;
                self.y += rhs.y;
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::SubAssign for $base_name<T> {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                self.x -= rhs.x;
                self.y -= rhs.y;
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::MulAssign for $base_name<T> {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: Self) {
                self.x *= rhs.x;
                self.y *= rhs.y;
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::DivAssign for $base_name<T> {
            #[inline(always)]
            fn div_assign(&mut self, rhs: Self) {
                self.x /= rhs.x;
                self.y /= rhs.y;
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> Default for $base_name<T> {
            #[inline(always)]
            fn default() -> Self {
                Self::new_unchecked(T::ONE, T::ONE)
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType + core::fmt::Display> core::fmt::Display for $base_name<T> {
            #[inline(always)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, concat!(stringify!($base_name), "<{}, {}>"), self.x, self.y)
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> $crate::base_feagi_types::quantizable_types::FeagiBaseQuantizationType for $base_name<T> {
            const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 2;

            #[inline(always)]
            fn saturating_add(self, other: Self) -> Self {
                Self {
                    x: self.x.saturating_add(other.x),
                    y: self.y.saturating_add(other.y),
                }
            }

            #[inline(always)]
            fn checked_add(self, other: Self) -> Option<Self> {
                Some(Self {
                    x: self.x.checked_add(other.x)?,
                    y: self.y.checked_add(other.y)?,
                })
            }

            #[inline(always)]
            fn saturating_sub(self, other: Self) -> Self {
                Self {
                    x: self.x.saturating_sub(other.x),
                    y: self.y.saturating_sub(other.y),
                }
            }

            #[inline(always)]
            fn checked_sub(self, other: Self) -> Option<Self> {
                Some(Self {
                    x: self.x.checked_sub(other.x)?,
                    y: self.y.checked_sub(other.y)?,
                })
            }

            #[inline(always)]
            fn saturating_mul(self, other: Self) -> Self {
                Self {
                    x: self.x.saturating_mul(other.x),
                    y: self.y.saturating_mul(other.y),
                }
            }

            #[inline(always)]
            fn checked_mul(self, other: Self) -> Option<Self> {
                Some(Self {
                    x: self.x.checked_mul(other.x)?,
                    y: self.y.checked_mul(other.y)?,
                })
            }

            #[inline(always)]
            fn checked_div(self, other: Self) -> Option<Self> {
                Some(Self {
                    x: self.x.checked_div(other.x)?,
                    y: self.y.checked_div(other.y)?,
                })
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> $crate::base_feagi_types::quantizable_types::FeagiBaseMultiElementQuantizationType for $base_name<T> {
            const NUMBER_ELEMENTS: usize = 2;
            const ALL_ZEROS: Self = Self::new_unchecked(T::ONE, T::ONE);
            const ALL_ONES: Self = Self::new_unchecked(T::ONE, T::ONE);

            type ElementType = T;
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> $crate::base_feagi_types::quantizable_types::spatial::QuantizableUInt2DDimensionType
            for $base_name<T>
        {
            type CoordinateType = $coordinate_type<T>;

            #[inline(always)]
            fn from_tuple(tuple: (Self::ElementType, Self::ElementType)) -> Self {
                Self::new_unchecked(tuple.0, tuple.1)
            }

            #[inline(always)]
            fn to_tuple(self) -> (Self::ElementType, Self::ElementType) {
                (self.x.get(), self.y.get())
            }

            #[inline(always)]
            fn fits_coordinate(&self, coordinate: &Self::CoordinateType) -> bool {
                self.does_fit(coordinate)
            }
        }
    };
}

pub trait QuantizableUInt2DDimensionType:
FeagiBaseMultiElementQuantizationType
{
    type CoordinateType: QuantizableUInt2DCoordinateType;

    fn from_tuple(tuple: (Self::ElementType, Self::ElementType)) -> Self;
    fn to_tuple(self) -> (Self::ElementType, Self::ElementType);
    fn fits_coordinate(&self, coordinate: &Self::CoordinateType) -> bool;
}

//endregion

//endregion

//region 3D

//region Unsigned Coordinate

#[macro_export]
macro_rules! define_unsigned_coordinate_3d_type_family {
    ($base_name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[cfg_attr(feature = "alloc", derive(serde::Serialize, serde::Deserialize))]
        pub struct $base_name<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> {
            pub x: T,
            pub y: T,
            pub z: T,
        }

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> $base_name<T> {
            #[inline(always)]
            pub const fn new(x: T, y: T, z: T) -> Self {
                Self { x, y, z }
            }

            #[inline(always)]
            pub fn to_usize(self) -> $base_name<usize> {
                $base_name::new(self.x.to_usize(), self.y.to_usize(), self.z.to_usize())
            }
        }

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> From<(T, T, T)> for $base_name<T> {
            #[inline(always)]
            fn from(value: (T, T, T)) -> Self {
                Self::new(value.0, value.1, value.2)
            }
        }

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> From<$base_name<T>> for (T, T, T) {
            #[inline(always)]
            fn from(value: $base_name<T>) -> Self {
                (value.x, value.y, value.z)
            }
        }

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::Add for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn add(self, rhs: Self) -> Self::Output {
                Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
            }
        }

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::Sub for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn sub(self, rhs: Self) -> Self::Output {
                Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
            }
        }

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::Mul for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn mul(self, rhs: Self) -> Self::Output {
                Self::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
            }
        }

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::Div for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn div(self, rhs: Self) -> Self::Output {
                Self::new(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z)
            }
        }

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::AddAssign for $base_name<T> {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                self.x += rhs.x;
                self.y += rhs.y;
                self.z += rhs.z;
            }
        }

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::SubAssign for $base_name<T> {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                self.x -= rhs.x;
                self.y -= rhs.y;
                self.z -= rhs.z;
            }
        }

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::MulAssign for $base_name<T> {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: Self) {
                self.x *= rhs.x;
                self.y *= rhs.y;
                self.z *= rhs.z;
            }
        }

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::DivAssign for $base_name<T> {
            #[inline(always)]
            fn div_assign(&mut self, rhs: Self) {
                self.x /= rhs.x;
                self.y /= rhs.y;
                self.z /= rhs.z;
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> Default for $base_name<T> {
            #[inline(always)]
            fn default() -> Self {
                Self::new(T::ZERO, T::ZERO, T::ZERO)
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType + core::fmt::Display> core::fmt::Display for $base_name<T> {
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

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> $crate::base_feagi_types::quantizable_types::FeagiBaseQuantizationType for $base_name<T> {
            const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 3;

            #[inline(always)]
            fn saturating_add(self, other: Self) -> Self {
                Self::new(
                    self.x.saturating_add(other.x),
                    self.y.saturating_add(other.y),
                    self.z.saturating_add(other.z),
                )
            }

            #[inline(always)]
            fn checked_add(self, other: Self) -> Option<Self> {
                Some(Self::new(
                    self.x.checked_add(other.x)?,
                    self.y.checked_add(other.y)?,
                    self.z.checked_add(other.z)?,
                ))
            }

            #[inline(always)]
            fn saturating_sub(self, other: Self) -> Self {
                Self::new(
                    self.x.saturating_sub(other.x),
                    self.y.saturating_sub(other.y),
                    self.z.saturating_sub(other.z),
                )
            }

            #[inline(always)]
            fn checked_sub(self, other: Self) -> Option<Self> {
                Some(Self::new(
                    self.x.checked_sub(other.x)?,
                    self.y.checked_sub(other.y)?,
                    self.z.checked_sub(other.z)?,
                ))
            }

            #[inline(always)]
            fn saturating_mul(self, other: Self) -> Self {
                Self::new(
                    self.x.saturating_mul(other.x),
                    self.y.saturating_mul(other.y),
                    self.z.saturating_mul(other.z),
                )
            }

            #[inline(always)]
            fn checked_mul(self, other: Self) -> Option<Self> {
                Some(Self::new(
                    self.x.checked_mul(other.x)?,
                    self.y.checked_mul(other.y)?,
                    self.z.checked_mul(other.z)?,
                ))
            }

            #[inline(always)]
            fn checked_div(self, other: Self) -> Option<Self> {
                Some(Self::new(
                    self.x.checked_div(other.x)?,
                    self.y.checked_div(other.y)?,
                    self.z.checked_div(other.z)?,
                ))
            }
        }

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> $crate::base_feagi_types::quantizable_types::FeagiBaseMultiElementQuantizationType for $base_name<T> {
            const NUMBER_ELEMENTS: usize = 3;
            const ALL_ZEROS: Self = Self::new(T::ZERO, T::ZERO, T::ZERO);
            const ALL_ONES: Self = Self::new(T::ONE, T::ONE, T::ONE);

            type ElementType = T;
        }

        impl<T: crate::base_feagi_types::quantizable_types::QuantizableUIntType> $crate::base_feagi_types::quantizable_types::spatial::QuantizableUInt3DCoordinateType
            for $base_name<T>
        {
            #[inline(always)]
            fn from_tuple(tuple: (Self::ElementType, Self::ElementType, Self::ElementType)) -> Self {
                Self::new(tuple.0, tuple.1, tuple.2)
            }

            #[inline(always)]
            fn from_usize_tuple(usize_tuple: (usize, usize, usize)) -> Self {
                Self::new(
                    T::from_usize(usize_tuple.0),
                    T::from_usize(usize_tuple.1),
                    T::from_usize(usize_tuple.2),
                )
            }

            #[inline(always)]
            fn to_tuple(self) -> (Self::ElementType, Self::ElementType, Self::ElementType) {
                (self.x, self.y, self.z)
            }

            #[inline(always)]
            fn to_usize_tuple(self) -> (usize, usize, usize) {
                (self.x.to_usize(), self.y.to_usize(), self.z.to_usize())
            }
        }
    };
}

pub trait QuantizableUInt3DCoordinateType:
FeagiBaseMultiElementQuantizationType
{
    fn from_tuple(tuple: (Self::ElementType, Self::ElementType, Self::ElementType)) -> Self;
    fn from_usize_tuple(usize_tuple: (usize, usize, usize)) -> Self;
    fn to_tuple(self) -> (Self::ElementType, Self::ElementType, Self::ElementType);
    fn to_usize_tuple(self) -> (usize, usize, usize);
}

//endregion

//region Signed Coordinate

#[macro_export]
macro_rules! define_signed_coordinate_3d_type_family {
    ($base_name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[cfg_attr(feature = "alloc", derive(serde::Serialize, serde::Deserialize))]
        pub struct $base_name<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> {
            pub x: T,
            pub y: T,
            pub z: T,
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> $base_name<T> {
            #[inline(always)]
            pub const fn new(x: T, y: T, z: T) -> Self {
                Self { x, y, z }
            }

            #[inline(always)]
            pub fn to_isize(self) -> $base_name<isize> {
                $base_name::new(self.x.to_isize(), self.y.to_isize(), self.z.to_isize())
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> From<(T, T, T)> for $base_name<T> {
            #[inline(always)]
            fn from(value: (T, T, T)) -> Self {
                Self::new(value.0, value.1, value.2)
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> From<$base_name<T>> for (T, T, T) {
            #[inline(always)]
            fn from(value: $base_name<T>) -> Self {
                (value.x, value.y, value.z)
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> core::ops::Add for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn add(self, rhs: Self) -> Self::Output {
                Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> core::ops::Sub for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn sub(self, rhs: Self) -> Self::Output {
                Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> core::ops::Mul for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn mul(self, rhs: Self) -> Self::Output {
                Self::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> core::ops::Div for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn div(self, rhs: Self) -> Self::Output {
                Self::new(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z)
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> core::ops::AddAssign for $base_name<T> {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                self.x += rhs.x;
                self.y += rhs.y;
                self.z += rhs.z;
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> core::ops::SubAssign for $base_name<T> {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                self.x -= rhs.x;
                self.y -= rhs.y;
                self.z -= rhs.z;
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> core::ops::MulAssign for $base_name<T> {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: Self) {
                self.x *= rhs.x;
                self.y *= rhs.y;
                self.z *= rhs.z;
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> core::ops::DivAssign for $base_name<T> {
            #[inline(always)]
            fn div_assign(&mut self, rhs: Self) {
                self.x /= rhs.x;
                self.y /= rhs.y;
                self.z /= rhs.z;
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> Default for $base_name<T> {
            #[inline(always)]
            fn default() -> Self {
                Self::new(T::ZERO, T::ZERO, T::ZERO)
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType + core::fmt::Display> core::fmt::Display for $base_name<T> {
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

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> $crate::base_feagi_types::quantizable_types::FeagiBaseQuantizationType for $base_name<T> {
            const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 3;

            #[inline(always)]
            fn saturating_add(self, other: Self) -> Self {
                Self::new(
                    self.x.saturating_add(other.x),
                    self.y.saturating_add(other.y),
                    self.z.saturating_add(other.z),
                )
            }

            #[inline(always)]
            fn checked_add(self, other: Self) -> Option<Self> {
                Some(Self::new(
                    self.x.checked_add(other.x)?,
                    self.y.checked_add(other.y)?,
                    self.z.checked_add(other.z)?,
                ))
            }

            #[inline(always)]
            fn saturating_sub(self, other: Self) -> Self {
                Self::new(
                    self.x.saturating_sub(other.x),
                    self.y.saturating_sub(other.y),
                    self.z.saturating_sub(other.z),
                )
            }

            #[inline(always)]
            fn checked_sub(self, other: Self) -> Option<Self> {
                Some(Self::new(
                    self.x.checked_sub(other.x)?,
                    self.y.checked_sub(other.y)?,
                    self.z.checked_sub(other.z)?,
                ))
            }

            #[inline(always)]
            fn saturating_mul(self, other: Self) -> Self {
                Self::new(
                    self.x.saturating_mul(other.x),
                    self.y.saturating_mul(other.y),
                    self.z.saturating_mul(other.z),
                )
            }

            #[inline(always)]
            fn checked_mul(self, other: Self) -> Option<Self> {
                Some(Self::new(
                    self.x.checked_mul(other.x)?,
                    self.y.checked_mul(other.y)?,
                    self.z.checked_mul(other.z)?,
                ))
            }

            #[inline(always)]
            fn checked_div(self, other: Self) -> Option<Self> {
                Some(Self::new(
                    self.x.checked_div(other.x)?,
                    self.y.checked_div(other.y)?,
                    self.z.checked_div(other.z)?,
                ))
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> $crate::base_feagi_types::quantizable_types::FeagiBaseMultiElementQuantizationType for $base_name<T> {
            const NUMBER_ELEMENTS: usize = 3;
            const ALL_ZEROS: Self = Self::new(T::ZERO, T::ZERO, T::ZERO);
            const ALL_ONES: Self = Self::new(T::ONE, T::ONE, T::ONE);

            type ElementType = T;
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableIntType> $crate::base_feagi_types::quantizable_types::spatial::QuantizableInt3DCoordinateType
            for $base_name<T>
        {
            #[inline(always)]
            fn from_tuple(tuple: (Self::ElementType, Self::ElementType, Self::ElementType)) -> Self {
                Self::new(tuple.0, tuple.1, tuple.2)
            }

            #[inline(always)]
            fn from_isize_tuple(isize_tuple: (isize, isize, isize)) -> Self {
                Self::new(
                    T::from_isize(isize_tuple.0),
                    T::from_isize(isize_tuple.1),
                    T::from_isize(isize_tuple.2),
                )
            }

            #[inline(always)]
            fn to_tuple(self) -> (Self::ElementType, Self::ElementType, Self::ElementType) {
                (self.x, self.y, self.z)
            }

            #[inline(always)]
            fn to_isize_tuple(self) -> (isize, isize, isize) {
                (self.x.to_isize(), self.y.to_isize(), self.z.to_isize())
            }
        }
    };
}

pub trait QuantizableInt3DCoordinateType:
FeagiBaseMultiElementQuantizationType
{
    fn from_tuple(tuple: (Self::ElementType, Self::ElementType, Self::ElementType)) -> Self;
    fn from_isize_tuple(isize_tuple: (isize, isize, isize)) -> Self;
    fn to_tuple(self) -> (Self::ElementType, Self::ElementType, Self::ElementType);
    fn to_isize_tuple(self) -> (isize, isize, isize);
}

//endregion

//region Dimensions

#[macro_export]
macro_rules! define_dimension_3d_type_family {
    ($base_name:ident, $coordinate_type:ident) =>  {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[cfg_attr(feature = "alloc", derive(serde::Serialize, serde::Deserialize))]
        pub struct $base_name<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> {
            pub x: $crate::base_feagi_types::quantizable_types::spatial::SpatialDimensionAxis<T>,
            pub y: $crate::base_feagi_types::quantizable_types::spatial::SpatialDimensionAxis<T>,
            pub z: $crate::base_feagi_types::quantizable_types::spatial::SpatialDimensionAxis<T>,
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> $base_name<T> {
            #[inline(always)]
            pub(crate) const fn new_unchecked(x: T, y: T, z: T) -> Self {
                let x = $crate::base_feagi_types::quantizable_types::spatial::SpatialDimensionAxis::new_unchecked(x);
                let y = $crate::base_feagi_types::quantizable_types::spatial::SpatialDimensionAxis::new_unchecked(y);
                let z = $crate::base_feagi_types::quantizable_types::spatial::SpatialDimensionAxis::new_unchecked(z);
                Self { x, y, z }
            }

            #[inline(always)]
            pub fn new(x: T, y: T, z: T) -> Result<Self, $crate::FeagiStructuresError> {
                let x = $crate::base_feagi_types::quantizable_types::spatial::SpatialDimensionAxis::new(x)?;
                let y = $crate::base_feagi_types::quantizable_types::spatial::SpatialDimensionAxis::new(y)?;
                let z = $crate::base_feagi_types::quantizable_types::spatial::SpatialDimensionAxis::new(z)?;
                Ok(Self { x, y, z })
            }

            #[inline(always)]
            pub fn new_cube(n: T) -> Result<Self, $crate::FeagiStructuresError> {
                let x = $crate::base_feagi_types::quantizable_types::spatial::SpatialDimensionAxis::new(n)?;
                let y = x;
                let z = x;
                Ok(Self { x, y, z })
            }

            #[inline(always)]
            pub(crate) fn from_usize(usize_dims: $base_name<usize>) -> Self {
                $base_name::new_unchecked(
                    T::from_usize(usize_dims.x.to_usize()),
                    T::from_usize(usize_dims.y.to_usize()),
                    T::from_usize(usize_dims.z.to_usize()),
                )
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

            #[inline(always)]
            pub fn number_elements(&self) -> usize {
                (self.x * self.y * self.z).to_usize()
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::Add for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn add(self, rhs: Self) -> Self::Output {
                Self {
                    x: self.x + rhs.x,
                    y: self.y + rhs.y,
                    z: self.z + rhs.z,
                }
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::Sub for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn sub(self, rhs: Self) -> Self::Output {
                Self {
                    x: self.x - rhs.x,
                    y: self.y - rhs.y,
                    z: self.z - rhs.z,
                }
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::Mul for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn mul(self, rhs: Self) -> Self::Output {
                Self {
                    x: self.x * rhs.x,
                    y: self.y * rhs.y,
                    z: self.z * rhs.z,
                }
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::Div for $base_name<T> {
            type Output = Self;

            #[inline(always)]
            fn div(self, rhs: Self) -> Self::Output {
                Self {
                    x: self.x / rhs.x,
                    y: self.y / rhs.y,
                    z: self.z / rhs.z,
                }
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::AddAssign for $base_name<T> {
            #[inline(always)]
            fn add_assign(&mut self, rhs: Self) {
                self.x += rhs.x;
                self.y += rhs.y;
                self.z += rhs.z;
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::SubAssign for $base_name<T> {
            #[inline(always)]
            fn sub_assign(&mut self, rhs: Self) {
                self.x -= rhs.x;
                self.y -= rhs.y;
                self.z -= rhs.z;
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::MulAssign for $base_name<T> {
            #[inline(always)]
            fn mul_assign(&mut self, rhs: Self) {
                self.x *= rhs.x;
                self.y *= rhs.y;
                self.z *= rhs.z;
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> core::ops::DivAssign for $base_name<T> {
            #[inline(always)]
            fn div_assign(&mut self, rhs: Self) {
                self.x /= rhs.x;
                self.y /= rhs.y;
                self.z /= rhs.z;
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> Default for $base_name<T> {
            #[inline(always)]
            fn default() -> Self {
                Self::new_unchecked(T::ONE, T::ONE, T::ONE)
            }
        }

        #[cfg(feature = "alloc")]
        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType + core::fmt::Display> core::fmt::Display for $base_name<T> {
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

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> $crate::base_feagi_types::quantizable_types::FeagiBaseQuantizationType for $base_name<T> {
            const NUMBER_OF_BYTES: usize = T::NUMBER_OF_BYTES * 3;

            #[inline(always)]
            fn saturating_add(self, other: Self) -> Self {
                Self {
                    x: self.x.saturating_add(other.x),
                    y: self.y.saturating_add(other.y),
                    z: self.z.saturating_add(other.z),
                }
            }

            #[inline(always)]
            fn checked_add(self, other: Self) -> Option<Self> {
                Some(Self {
                    x: self.x.checked_add(other.x)?,
                    y: self.y.checked_add(other.y)?,
                    z: self.z.checked_add(other.z)?,
                })
            }

            #[inline(always)]
            fn saturating_sub(self, other: Self) -> Self {
                Self {
                    x: self.x.saturating_sub(other.x),
                    y: self.y.saturating_sub(other.y),
                    z: self.z.saturating_sub(other.z),
                }
            }

            #[inline(always)]
            fn checked_sub(self, other: Self) -> Option<Self> {
                Some(Self {
                    x: self.x.checked_sub(other.x)?,
                    y: self.y.checked_sub(other.y)?,
                    z: self.z.checked_sub(other.z)?,
                })
            }

            #[inline(always)]
            fn saturating_mul(self, other: Self) -> Self {
                Self {
                    x: self.x.saturating_mul(other.x),
                    y: self.y.saturating_mul(other.y),
                    z: self.z.saturating_mul(other.z),
                }
            }

            #[inline(always)]
            fn checked_mul(self, other: Self) -> Option<Self> {
                Some(Self {
                    x: self.x.checked_mul(other.x)?,
                    y: self.y.checked_mul(other.y)?,
                    z: self.z.checked_mul(other.z)?,
                })
            }

            #[inline(always)]
            fn checked_div(self, other: Self) -> Option<Self> {
                Some(Self {
                    x: self.x.checked_div(other.x)?,
                    y: self.y.checked_div(other.y)?,
                    z: self.z.checked_div(other.z)?,
                })
            }
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> $crate::base_feagi_types::quantizable_types::FeagiBaseMultiElementQuantizationType for $base_name<T> {
            const NUMBER_ELEMENTS: usize = 3;
            const ALL_ZEROS: Self = Self::new_unchecked(T::ONE, T::ONE, T::ONE);
            const ALL_ONES: Self = Self::new_unchecked(T::ONE, T::ONE, T::ONE);

            type ElementType = T;
        }

        impl<T: $crate::base_feagi_types::quantizable_types::QuantizableUIntType> $crate::base_feagi_types::quantizable_types::spatial::QuantizableUInt3DDimensionType
            for $base_name<T>
        {
            type CoordinateType = $coordinate_type<T>;

            #[inline(always)]
            fn from_tuple(tuple: (Self::ElementType, Self::ElementType, Self::ElementType)) -> Self {
                Self::new_unchecked(tuple.0, tuple.1, tuple.2)
            }

            #[inline(always)]
            fn to_tuple(self) -> (Self::ElementType, Self::ElementType, Self::ElementType) {
                (self.x.get(), self.y.get(), self.z.get())
            }

            #[inline(always)]
            fn fits_coordinate(&self, coordinate: &Self::CoordinateType) -> bool {
                self.does_fit(coordinate)
            }
        }
    };
}

pub trait QuantizableUInt3DDimensionType:
FeagiBaseMultiElementQuantizationType
{
    type CoordinateType: QuantizableUInt3DCoordinateType;

    fn from_tuple(tuple: (Self::ElementType, Self::ElementType, Self::ElementType)) -> Self;
    fn to_tuple(self) -> (Self::ElementType, Self::ElementType, Self::ElementType);
    fn fits_coordinate(&self, coordinate: &Self::CoordinateType) -> bool;
}

//endregion

//endregion




