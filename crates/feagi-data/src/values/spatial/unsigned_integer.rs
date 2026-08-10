use std::ops::Deref;
use crate::values::quantizable::{FeagiDataValueQuantizationError, QuantizedElementBase, QuantizedUnsignedIntegerTrait, UnsignedIntegerQuantizationLevel, WrappedQuantizedUnsignedInteger, WrappedQuantizedUnsignedIntegerCount, WrappedQuantizedUnsignedIntegerIndex};
use crate::values::spatial::feagi_data_values_spatial_error::{
    FeagiDataValuesSpatialError, FeagiFailInvalidSpatialIndex,
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct UnsignedIntegerSpatial<Q: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize> {
    data: [Q; NUM_DIMS],
}

impl<Q: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize> UnsignedIntegerSpatial<Q, NUM_DIMS> {
    /// Create self from a correctly sized array in const context
    pub const fn new_from_array_const(array: [Q; NUM_DIMS]) -> Self {
        Self { data: array }
    }

    // NOTE: Not possible to have a new_from_usize_array_const since we would need to go through
    // the trait and rust stable doesnt support const functions yet.

    /// Create self from a correctly sized array
    pub fn new_from_array(array: [Q; NUM_DIMS]) -> Self {
        Self { data: array }
    }

    /// Create self from a correctly sized usize array. Does NOT validate
    /// valid quantization bounds!
    pub fn new_from_usize_array_unchecked(usize_array: [usize; NUM_DIMS]) -> Self {
        let mut data = [Q::QUANT_ZERO; NUM_DIMS];
        data.iter_mut()
            .zip(usize_array.iter())
            .for_each(|(q, u)| *q = Q::quant_from_usize_unchecked(*u));
        Self { data }
    }

    /// Create self from a correctly sized usize array, returning an error if any axis value does
    /// not fit in the current quantization.
    pub fn new_from_usize_array(
        usize_array: [usize; NUM_DIMS],
    ) -> Result<Self, FeagiDataValuesSpatialError> {
        let mut data = [Q::QUANT_ZERO; NUM_DIMS];
        for (q, &u) in data.iter_mut().zip(usize_array.iter()) {
            *q = Q::quant_try_from_usize(u).map_err(|e| e.into());
        }
        Ok(Self { data })
    }

    /// Create self from a correctly sized array of some other quant. Does NOT validate
    /// valid quantization bounds!
    pub fn new_from_quant_unchecked<FromQuant: QuantizedUnsignedIntegerTrait>(
        value_array: UnsignedIntegerSpatial<FromQuant, NUM_DIMS>,
    ) -> Self {
        let mut data = [Q::QUANT_ZERO; NUM_DIMS];
        data.iter_mut()
            .zip(value_array.as_slice().iter())
            .for_each(|(q, &v)| *q = Q::from_quantization_unchecked(v));
        Self { data }
    }

    /// Create self from a correctly sized array of some other quant, returning an error if any
    /// axis value does not fit in the current quantization.
    pub fn new_from_quant<FromQuant: QuantizedUnsignedIntegerTrait>(
        value_array: UnsignedIntegerSpatial<FromQuant, NUM_DIMS>,
    ) -> Result<Self, FeagiDataValueQuantizationError> {
        let mut data = [Q::QUANT_ZERO; NUM_DIMS];
        for (q, &v) in data.iter_mut().zip(value_array.as_slice().iter()) {
            *q = Q::try_from_quantization(v)?;
        }
        Ok(Self { data })
    }

    /// Create self from a correctly sized array of some other quant, clamping each axis value to
    /// fit in the current quantization.
    pub fn new_from_quant_clamped<FromQuant: QuantizedUnsignedIntegerTrait>(
        value_array: UnsignedIntegerSpatial<FromQuant, NUM_DIMS>,
    ) -> Self {
        let mut data = [Q::QUANT_ZERO; NUM_DIMS];
        data.iter_mut()
            .zip(value_array.as_slice().iter())
            .for_each(|(q, &v)| *q = Q::from_quantization_clamped(v));
        Self { data }
    }

    /// Returns a spatial array of all values zero
    pub fn new_zero() -> Self {
        Self { data: [0; NUM_DIMS] }
    }

    /// Converts this value from its current quantization to another quantization without
    /// checking if valid.
    pub fn to_quantization_unchecked<ToQuant: QuantizedUnsignedIntegerTrait>(
        self,
    ) -> UnsignedIntegerSpatial<ToQuant, NUM_DIMS> {
        let mut out = [ToQuant::QUANT_ZERO; NUM_DIMS];
        out.iter_mut()
            .zip(self.data.iter())
            .for_each(|(o, &s)| *o = ToQuant::from_quantization_unchecked(s));
        UnsignedIntegerSpatial { data: out }
    }

    /// Tries to convert this value from its current quantization to another quantization,
    /// returning an error if any axis value does not fit.
    pub fn try_to_quantization<ToQuant: QuantizedUnsignedIntegerTrait>(
        self,
    ) -> Result<UnsignedIntegerSpatial<ToQuant, NUM_DIMS>, FeagiDataValueQuantizationError> {
        let mut out = [ToQuant::QUANT_ZERO; NUM_DIMS];
        for (o, &s) in out.iter_mut().zip(self.data.iter()) {
            *o = s.try_to_quantization()?;
        }
        Ok(UnsignedIntegerSpatial { data: out })
    }

    /// Converts this value from its current quantization to another quantization, clamping
    /// each axis value to fit.
    pub fn to_quantization_clamped<ToQuant: QuantizedUnsignedIntegerTrait>(
        self,
    ) -> UnsignedIntegerSpatial<ToQuant, NUM_DIMS> {
        let mut out = [ToQuant::QUANT_ZERO; NUM_DIMS];
        out.iter_mut()
            .zip(self.data.iter())
            .for_each(|(o, &s)| *o = s.to_quantization_clamped());
        UnsignedIntegerSpatial { data: out }
    }

    /// Clamps each axis value for another quantization, but does not change the quantization
    /// itself.
    pub fn clamp_for_quantization<ClampFor: QuantizedUnsignedIntegerTrait>(self) -> Self {
        let mut data = self.data;
        data.iter_mut()
            .for_each(|q| *q = q.clamp_for_quantization::<ClampFor>());
        Self { data }
    }

    /// Clamps each axis value for a runtime-provided quantization level, but does not change the
    /// quantization itself.
    pub fn clamp_for_quantization_level_runtime(self, level: UnsignedIntegerQuantizationLevel) -> Self {
        let mut data = self.data;
        data.iter_mut()
            .for_each(|q| *q = q.clamp_for_quantization_level_runtime(level));
        Self { data }
    }

    /// Output to an [`UnsignedIntegerSpatialEnum`] to abstract away quantization.
    pub fn to_enum(self) -> UnsignedIntegerSpatialEnum<NUM_DIMS> {
        match Q::LEVEL {
            UnsignedIntegerQuantizationLevel::U8 => UnsignedIntegerSpatialEnum::U8(self),
            UnsignedIntegerQuantizationLevel::U16 => UnsignedIntegerSpatialEnum::U16(self),
            UnsignedIntegerQuantizationLevel::U32 => UnsignedIntegerSpatialEnum::U32(self),
            UnsignedIntegerQuantizationLevel::U64 => UnsignedIntegerSpatialEnum::U64(self),
        }
    }

    /// Get as a slice
    pub fn as_slice(&self) -> &[Q; NUM_DIMS] {
        &self.data
    }

    /// Get as a mutable slice
    pub fn as_slice_mut(&mut self) -> &mut [Q; NUM_DIMS] {
        &mut self.data
    }
}

/// Stores [`UnsignedIntegerSpatial`] as an enum to erase the generic type and to make transport
/// easier.
///
/// NOTE that due to how enums work in Rust, memory allocation will always be at u64 quant
/// levels!
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum UnsignedIntegerSpatialEnum<const NUM_DIMS: usize> {
    U8(UnsignedIntegerSpatial<u8, NUM_DIMS>),
    U16(UnsignedIntegerSpatial<u16, NUM_DIMS>),
    U32(UnsignedIntegerSpatial<u32, NUM_DIMS>),
    U64(UnsignedIntegerSpatial<u64, NUM_DIMS>),
}

impl<const NUM_DIMS: usize> UnsignedIntegerSpatialEnum<NUM_DIMS> {
    /// Get what level of quantization is contained
    pub fn get_level(&self) -> UnsignedIntegerQuantizationLevel {
        match self {
            Self::U8(_) => UnsignedIntegerQuantizationLevel::U8,
            Self::U16(_) => UnsignedIntegerQuantizationLevel::U16,
            Self::U32(_) => UnsignedIntegerQuantizationLevel::U32,
            Self::U64(_) => UnsignedIntegerQuantizationLevel::U64,
        }
    }

    /// Tries to convert to a spatial value of another quantization, returning an error if any axis
    /// value does not fit.
    pub fn try_into_quant<Quant: QuantizedUnsignedIntegerTrait>(
        self,
    ) -> Result<UnsignedIntegerSpatial<Quant, NUM_DIMS>, FeagiDataValueQuantizationError> {
        match self {
            Self::U8(value) => value.try_to_quantization(),
            Self::U16(value) => value.try_to_quantization(),
            Self::U32(value) => value.try_to_quantization(),
            Self::U64(value) => value.try_to_quantization(),
        }
    }

    /// Converts to a spatial value of another quantization without checking if valid.
    pub fn into_quant<Quant: QuantizedUnsignedIntegerTrait>(
        self,
    ) -> UnsignedIntegerSpatial<Quant, NUM_DIMS> {
        match self {
            Self::U8(value) => value.to_quantization_unchecked(),
            Self::U16(value) => value.to_quantization_unchecked(),
            Self::U32(value) => value.to_quantization_unchecked(),
            Self::U64(value) => value.to_quantization_unchecked(),
        }
    }

    /// Converts to a spatial value of another quantization, clamping each axis value to fit.
    pub fn into_quantization_clamped<Quant: QuantizedUnsignedIntegerTrait>(
        self,
    ) -> UnsignedIntegerSpatial<Quant, NUM_DIMS> {
        match self {
            Self::U8(value) => value.to_quantization_clamped(),
            Self::U16(value) => value.to_quantization_clamped(),
            Self::U32(value) => value.to_quantization_clamped(),
            Self::U64(value) => value.to_quantization_clamped(),
        }
    }
}


// NOTE: We cannot put an abritary number of types in the wrapper traits, that can only be done in macros!


/// Shared behaviour implemented by every strongly-typed wrapper around
/// [`UnsignedIntegerSpatial`]
pub trait WrappedUnsignedIntegerSpatial<Quant: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize>:
Copy
+ Clone
+ Send
+ Sync
+ core::fmt::Debug
+ core::cmp::PartialEq
+ core::cmp::Eq
+ core::hash::Hash
+ Sized
+ 'static
{
    fn new_from_array(value: UnsignedIntegerSpatial<Quant, NUM_DIMS>) -> Self;
    fn deref(self) -> UnsignedIntegerSpatial<Quant, NUM_DIMS>;

    fn as_slice(&self) -> &[Quant];

    /*
    fn new_from_quant_unchecked<FromQuant: QuantizedUnsignedIntegerTrait>(
        value: UnsignedIntegerSpatial<FromQuant, Self::NUM_DIMS>,
    ) -> Self;


    fn new_from_quant<FromQuant: QuantizedUnsignedIntegerTrait>(
        value: UnsignedIntegerSpatial<FromQuant, Self::NUM_DIMS>,
    ) -> Result<Self, FeagiDataValueQuantizationError>;


    fn new_from_quant_clamped<FromQuant: QuantizedUnsignedIntegerTrait>(
        value: UnsignedIntegerSpatial<FromQuant, Self::NUM_DIMS>,
    ) -> Self;

     */

    fn to_quantization_unchecked<ToQuant: QuantizedUnsignedIntegerTrait>(
        self,
    ) -> UnsignedIntegerSpatial<ToQuant, NUM_DIMS> {
        self.deref().to_quantization_unchecked()
    }

    fn try_to_quantization<ToQuant: QuantizedUnsignedIntegerTrait>(
        self,
    ) -> Result<UnsignedIntegerSpatial<ToQuant, NUM_DIMS>, FeagiDataValueQuantizationError> {
        self.deref().try_to_quantization()
    }

    fn to_quantization_clamped<ToQuant: QuantizedUnsignedIntegerTrait>(
        self,
    ) -> UnsignedIntegerSpatial<ToQuant, NUM_DIMS> {
        self.deref().to_quantization_clamped()
    }

    fn clamp_for_quantization<ClampFor: QuantizedUnsignedIntegerTrait>(self) -> Self
    {
        Self::new_from_array(self.deref().clamp_for_quantization::<ClampFor>())
    }

    fn clamp_for_quantization_level_runtime(self, level: UnsignedIntegerQuantizationLevel) -> Self
    {
        Self::new_from_array(self.deref().clamp_for_quantization_level_runtime(level))
    }
}


/// Shared behaviour implemented by every strongly-typed wrapper around
/// [`UnsignedIntegerSpatial`] used to store spatial data (not indexing).
pub trait WrappedUnsignedIntegerSpatialData<Quant: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize>: WrappedUnsignedIntegerSpatial<Quant, NUM_DIMS>
{
    /// Returns spatial data with all elements zero
    fn new_zero() -> Self {
        Self::new_from_array( UnsignedIntegerSpatial::new_zero() )
    }
}

/// Shared behaviour implemented by every strongly-typed wrapper around
/// [`UnsignedIntegerSpatial`] used to store spatial coordinates.
pub trait WrappedUnsignedIntegerSpatialCoordinate<Quant: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize>: WrappedUnsignedIntegerSpatial<Quant, NUM_DIMS> {
    /// Returns a coordinate with all axis zero
    fn new_zero() -> Self {
        Self::new_from_array( UnsignedIntegerSpatial::new_zero() )
    }
}

/// Shared behaviour implemented by every strongly-typed wrapper around
/// [`UnsignedIntegerSpatial`] used to store spatial dimensions.
pub trait WrappedUnsignedIntegerSpatialDimensions<Quant: QuantizedUnsignedIntegerTrait, const NUM_DIMS: usize>: WrappedUnsignedIntegerSpatial<Quant, NUM_DIMS>
{
    type LinearIndex: WrappedQuantizedUnsignedIntegerIndex<Quant>;
    type LinearCount: WrappedQuantizedUnsignedIntegerCount<Quant>;
    type Coordinate: WrappedUnsignedIntegerSpatialCoordinate<Quant, NUM_DIMS>;

    /// Total number of discrete coordinates contained within these dimensions (the product of
    /// every axis).
    fn number_contained_elements(&self) -> Self::LinearCount {

        let out = self.as_slice().iter().product();
        Self::LinearCount::new(out)
    }

    /// Does a given coordinate fit within these dimensions
    fn contains_coordinate(&self, coord: &Self::Coordinate) -> bool {
        coord
            .as_slice()
            .iter()
            .zip(self.as_slice().iter())
            .all(|(axis, size)| *axis < *size)
    }

    /// Does a given linear index fit within these dimensions. Calculates count each call!
    fn contains_linear_index(&self, linear_index: Self::LinearIndex) -> bool {
        linear_index < self.number_contained_elements()
    }

    /// Converts a coordinate to its linear index, incrementing along the first axis fastest
    /// (x -> y -> z -> ...).
    fn coordinate_to_linear_index(
        &self,
        coord: Self::Coordinate,
    ) -> Result<Self::LinearIndex, FeagiDataValuesSpatialError> {
        if !self.contains_coordinate(&coord) {
            return Err(
                FeagiFailInvalidSpatialIndex::new("Given coordinate is out of bounds of the given dimensions!").into(),
            );
        }
        Ok(self.coordinate_to_linear_index_unchecked(coord))
    }

    /// Converts a linear index back into a coordinate, the inverse of
    /// [`coordinate_to_linear_index`](Self::coordinate_to_linear_index).
    fn linear_index_to_coordinate(
        &self,
        linear_index: Self::LinearIndex,
    ) -> Result<UnsignedIntegerSpatial<Self::LinearCount, NUM_DIMS>, FeagiDataValuesSpatialError> {
        if !self.contains_linear_index(linear_index) {
            return Err(
                FeagiFailInvalidSpatialIndex::new("Given linear index is out of bounds of the given dimensions!").into(),
            );
        }
        Ok(self.linear_index_to_coordinate_unchecked(linear_index))
    }

    /// Converts a coordinate to its linear index without checking if the coordinate is valid.
    fn coordinate_to_linear_index_unchecked(
        &self,
        coord: Self::Coordinate,
    ) -> Self::LinearIndex {
        let mut linear_index = Quant::QUANT_ZERO;
        let mut stride = Quant::QUANT_ONE;
        for (axis, size) in coord.as_slice().iter().zip(self.as_slice().iter()) {
            linear_index = linear_index + (*axis.deref() * stride);
            stride = stride * *size.deref();
        }
        Self::LinearIndex::new(linear_index)
    }

    /// Converts a linear index back into a coordinate without checking.
    fn linear_index_to_coordinate_unchecked(
        &self,
        linear_index: Self::LinearIndex,
    ) -> Self::Coordinate {
        let mut coordinate = UnsignedIntegerSpatial::new_zero();
        let mut stride = Quant::QUANT_ONE;
        for (axis, size) in coordinate.iter_mut().zip(self.as_slice().iter()) {
            *axis = (linear_index / stride) % *size;
            stride = stride * *size;
        }
        UnsignedIntegerSpatial::new_from_array(coordinate)
    }
}

/// Shared behaviour for enums that hide the quantization generic of wrapped spatial values.
pub trait WrappedUnsignedIntegerSpatialEnum:
    Copy
    + Clone
    + Send
    + Sync
    + core::fmt::Debug
    + core::cmp::PartialEq
    + core::cmp::Eq
    + core::hash::Hash
    + Sized
    + 'static
{
    const NUM_DIMS: usize;
    type Shape<Q: QuantizedUnsignedIntegerTrait>;

    fn get_level(&self) -> UnsignedIntegerQuantizationLevel;

    fn new_from_quantized<FromQ: QuantizedUnsignedIntegerTrait>(value: Self::Shape<FromQ>) -> Self;

    fn into_quantization_unchecked<NewQ: QuantizedUnsignedIntegerTrait>(self) -> Self::Shape<NewQ>;

    fn try_into_quantization<NewQ: QuantizedUnsignedIntegerTrait>(
        self,
    ) -> Result<Self::Shape<NewQ>, FeagiDataValueQuantizationError>;

    fn into_quantization_clamped<NewQ: QuantizedUnsignedIntegerTrait>(self) -> Self::Shape<NewQ>;
}

/// Creates a wrapper for spatial unsigned-integer data (not indexing).
#[macro_export]
macro_rules! create_wrapped_unsigned_integer_spatial_data {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_name:ident,
        $num_dimensions:expr,
        $( ($index:tt, $field:ident, $wrapped_axis:ident) ),+ $(,)?
    ) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $vis struct $struct_name<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
            $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions>
        );

        ::paste::paste! {
            impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> $struct_name<Q> {
                pub const NUM_DIMS: usize = $num_dimensions;

                pub const fn const_new(
                    value: $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions>
                ) -> Self {
                    Self(value)
                }

                pub fn new($( $field: $wrapped_axis<Q> ),+ ) -> Self {
                    Self(
                        $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial::new_from_array([
                            $( *$field.as_ref() ),+
                        ])
                    )
                }

                pub fn new_from_spatial(
                    value: $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions>
                ) -> Self {
                    Self(value)
                }

                pub fn deref(
                    self
                ) -> $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions> {
                    self.0
                }

                pub fn new_from_array(array: [Q; $num_dimensions]) -> Self {
                    Self($crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial::new_from_array(array))
                }

                pub fn new_from_usize_array_unchecked(usize_array: [usize; $num_dimensions]) -> Self {
                    Self($crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial::new_from_usize_array_unchecked(usize_array))
                }

                pub fn new_from_usize_array(
                    usize_array: [usize; $num_dimensions],
                ) -> Result<Self, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                    Ok(Self($crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial::new_from_usize_array(usize_array)?))
                }

                $(
                    pub fn [<get_ $field>](&self) -> &$wrapped_axis<Q> {
                        (&self.0.as_slice()[$index]).into()
                    }

                    pub fn [<get_ $field _mut>](&mut self) -> &mut $wrapped_axis<Q> {
                        (&mut self.0.as_slice_mut()[$index]).into()
                    }
                )+
            }

            impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>
                $crate::values::spatial::unsigned_integer::WrappedUnsignedIntegerSpatialData<Q> for $struct_name<Q>
            {
                const NUM_DIMS: usize = $num_dimensions;

                fn new(
                    value: $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions>
                ) -> Self {
                    Self(value)
                }

                fn deref(
                    self
                ) -> $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions> {
                    self.0
                }
            }

            impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>
                From<$crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions>>
                for $struct_name<Q>
            {
                fn from(value: $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions>) -> Self {
                    Self(value)
                }
            }

            impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>
                AsRef<$crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions>>
                for $struct_name<Q>
            {
                fn as_ref(&self) -> &$crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions> {
                    &self.0
                }
            }

            impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>
                AsMut<$crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions>>
                for $struct_name<Q>
            {
                fn as_mut(&mut self) -> &mut $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions> {
                    &mut self.0
                }
            }

            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            $vis enum [<$struct_name Enum>] {
                U8($struct_name<u8>),
                U16($struct_name<u16>),
                U32($struct_name<u32>),
                U64($struct_name<u64>),
            }

            impl [<$struct_name Enum>] {
                pub fn new_from_quantized<FromQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    value: $struct_name<FromQ>
                ) -> Self {
                    <Self as $crate::values::spatial::unsigned_integer::WrappedUnsignedIntegerSpatialEnum>::new_from_quantized(value)
                }

                pub fn into_quantization_unchecked<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $struct_name<NewQ> {
                    <Self as $crate::values::spatial::unsigned_integer::WrappedUnsignedIntegerSpatialEnum>::into_quantization_unchecked(self)
                }

                pub fn try_into_quantization<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> Result<$struct_name<NewQ>, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                    <Self as $crate::values::spatial::unsigned_integer::WrappedUnsignedIntegerSpatialEnum>::try_into_quantization(self)
                }

                pub fn into_quantization_clamped<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $struct_name<NewQ> {
                    <Self as $crate::values::spatial::unsigned_integer::WrappedUnsignedIntegerSpatialEnum>::into_quantization_clamped(self)
                }
            }

            impl $crate::values::spatial::unsigned_integer::WrappedUnsignedIntegerSpatialEnum for [<$struct_name Enum>] {
                const NUM_DIMS: usize = $num_dimensions;
                type Shape<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> = $struct_name<Q>;

                fn get_level(&self) -> $crate::values::quantizable::UnsignedIntegerQuantizationLevel {
                    match self {
                        Self::U8(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U8,
                        Self::U16(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U16,
                        Self::U32(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U32,
                        Self::U64(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U64,
                    }
                }

                fn new_from_quantized<FromQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    value: $struct_name<FromQ>
                ) -> Self {
                    match FromQ::LEVEL {
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U8 => {
                            Self::U8($struct_name::<u8>::new_from_spatial(
                                $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial::<u8, $num_dimensions>::new_from_quant_unchecked(value.deref()),
                            ))
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U16 => {
                            Self::U16($struct_name::<u16>::new_from_spatial(
                                $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial::<u16, $num_dimensions>::new_from_quant_unchecked(value.deref()),
                            ))
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U32 => {
                            Self::U32($struct_name::<u32>::new_from_spatial(
                                $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial::<u32, $num_dimensions>::new_from_quant_unchecked(value.deref()),
                            ))
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U64 => {
                            Self::U64($struct_name::<u64>::new_from_spatial(
                                $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial::<u64, $num_dimensions>::new_from_quant_unchecked(value.deref()),
                            ))
                        }
                    }
                }

                fn into_quantization_unchecked<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $struct_name<NewQ> {
                    match self {
                        Self::U8(value) => $struct_name::<NewQ>::new_from_spatial(value.deref().to_quantization_unchecked()),
                        Self::U16(value) => $struct_name::<NewQ>::new_from_spatial(value.deref().to_quantization_unchecked()),
                        Self::U32(value) => $struct_name::<NewQ>::new_from_spatial(value.deref().to_quantization_unchecked()),
                        Self::U64(value) => $struct_name::<NewQ>::new_from_spatial(value.deref().to_quantization_unchecked()),
                    }
                }

                fn try_into_quantization<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> Result<$struct_name<NewQ>, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                    match self {
                        Self::U8(value) => Ok($struct_name::<NewQ>::new_from_spatial(value.deref().try_to_quantization()?)),
                        Self::U16(value) => Ok($struct_name::<NewQ>::new_from_spatial(value.deref().try_to_quantization()?)),
                        Self::U32(value) => Ok($struct_name::<NewQ>::new_from_spatial(value.deref().try_to_quantization()?)),
                        Self::U64(value) => Ok($struct_name::<NewQ>::new_from_spatial(value.deref().try_to_quantization()?)),
                    }
                }

                fn into_quantization_clamped<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $struct_name<NewQ> {
                    match self {
                        Self::U8(value) => $struct_name::<NewQ>::new_from_spatial(value.deref().to_quantization_clamped()),
                        Self::U16(value) => $struct_name::<NewQ>::new_from_spatial(value.deref().to_quantization_clamped()),
                        Self::U32(value) => $struct_name::<NewQ>::new_from_spatial(value.deref().to_quantization_clamped()),
                        Self::U64(value) => $struct_name::<NewQ>::new_from_spatial(value.deref().to_quantization_clamped()),
                    }
                }
            }
        }
    };
}

/// Creates a wrapper for spatial unsigned-integer coordinates.
#[macro_export]
macro_rules! create_wrapped_unsigned_integer_spatial_coordinate {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_name:ident,
        $num_dimensions:expr,
        $( ($index:tt, $field:ident, $wrapped_axis:ident) ),+ $(,)?
    ) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $vis struct $struct_name<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
            $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions>
        );

        ::paste::paste! {
            impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> $struct_name<Q> {
                pub const NUM_DIMS: usize = $num_dimensions;

                pub const fn const_new(
                    value: $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions>
                ) -> Self {
                    Self(value)
                }

                pub fn new($( $field: $wrapped_axis<Q> ),+ ) -> Self {
                    Self(
                        $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial::new_from_array([
                            $( *$field.as_ref() ),+
                        ])
                    )
                }

                pub fn new_from_spatial(
                    value: $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions>
                ) -> Self {
                    Self(value)
                }

                pub fn deref(
                    self
                ) -> $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions> {
                    self.0
                }

                pub fn as_spatial(&self) -> &$crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions> {
                    &self.0
                }

                pub fn new_from_array(array: [Q; $num_dimensions]) -> Self {
                    Self($crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial::new_from_array(array))
                }

                pub fn new_from_usize_array_unchecked(usize_array: [usize; $num_dimensions]) -> Self {
                    Self($crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial::new_from_usize_array_unchecked(usize_array))
                }

                pub fn new_from_usize_array(
                    usize_array: [usize; $num_dimensions],
                ) -> Result<Self, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                    Ok(Self($crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial::new_from_usize_array(usize_array)?))
                }

                $(
                    pub fn [<get_ $field>](&self) -> &$wrapped_axis<Q> {
                        (&self.0.as_slice()[$index]).into()
                    }

                    pub fn [<get_ $field _mut>](&mut self) -> &mut $wrapped_axis<Q> {
                        (&mut self.0.as_slice_mut()[$index]).into()
                    }
                )+
            }

            impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>
                $crate::values::spatial::unsigned_integer::WrappedUnsignedIntegerSpatialCoordinate for $struct_name<Q>
            {
                type Quant = Q;
                const NUM_DIMS: usize = $num_dimensions;

                fn new(
                    value: $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions>
                ) -> Self {
                    Self(value)
                }

                fn deref(
                    self
                ) -> $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions> {
                    self.0
                }
            }

            impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>
                From<$crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions>>
                for $struct_name<Q>
            {
                fn from(value: $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions>) -> Self {
                    Self(value)
                }
            }

            impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>
                AsRef<$crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions>>
                for $struct_name<Q>
            {
                fn as_ref(&self) -> &$crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions> {
                    &self.0
                }
            }

            impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>
                AsMut<$crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions>>
                for $struct_name<Q>
            {
                fn as_mut(&mut self) -> &mut $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions> {
                    &mut self.0
                }
            }

            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            $vis enum [<$struct_name Enum>] {
                U8($struct_name<u8>),
                U16($struct_name<u16>),
                U32($struct_name<u32>),
                U64($struct_name<u64>),
            }

            impl [<$struct_name Enum>] {
                pub fn new_from_quantized<FromQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    value: $struct_name<FromQ>
                ) -> Self {
                    <Self as $crate::values::spatial::unsigned_integer::WrappedUnsignedIntegerSpatialEnum>::new_from_quantized(value)
                }

                pub fn into_quantization_unchecked<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $struct_name<NewQ> {
                    <Self as $crate::values::spatial::unsigned_integer::WrappedUnsignedIntegerSpatialEnum>::into_quantization_unchecked(self)
                }

                pub fn try_into_quantization<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> Result<$struct_name<NewQ>, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                    <Self as $crate::values::spatial::unsigned_integer::WrappedUnsignedIntegerSpatialEnum>::try_into_quantization(self)
                }

                pub fn into_quantization_clamped<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $struct_name<NewQ> {
                    <Self as $crate::values::spatial::unsigned_integer::WrappedUnsignedIntegerSpatialEnum>::into_quantization_clamped(self)
                }
            }

            impl $crate::values::spatial::unsigned_integer::WrappedUnsignedIntegerSpatialEnum for [<$struct_name Enum>] {
                const NUM_DIMS: usize = $num_dimensions;
                type Shape<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> = $struct_name<Q>;

                fn get_level(&self) -> $crate::values::quantizable::UnsignedIntegerQuantizationLevel {
                    match self {
                        Self::U8(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U8,
                        Self::U16(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U16,
                        Self::U32(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U32,
                        Self::U64(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U64,
                    }
                }

                fn new_from_quantized<FromQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    value: $struct_name<FromQ>
                ) -> Self {
                    match FromQ::LEVEL {
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U8 => {
                            Self::U8($struct_name::<u8>::new_from_spatial(
                                $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial::<u8, $num_dimensions>::new_from_quant_unchecked(value.deref()),
                            ))
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U16 => {
                            Self::U16($struct_name::<u16>::new_from_spatial(
                                $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial::<u16, $num_dimensions>::new_from_quant_unchecked(value.deref()),
                            ))
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U32 => {
                            Self::U32($struct_name::<u32>::new_from_spatial(
                                $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial::<u32, $num_dimensions>::new_from_quant_unchecked(value.deref()),
                            ))
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U64 => {
                            Self::U64($struct_name::<u64>::new_from_spatial(
                                $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial::<u64, $num_dimensions>::new_from_quant_unchecked(value.deref()),
                            ))
                        }
                    }
                }

                fn into_quantization_unchecked<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $struct_name<NewQ> {
                    match self {
                        Self::U8(value) => $struct_name::<NewQ>::new_from_spatial(value.deref().to_quantization_unchecked()),
                        Self::U16(value) => $struct_name::<NewQ>::new_from_spatial(value.deref().to_quantization_unchecked()),
                        Self::U32(value) => $struct_name::<NewQ>::new_from_spatial(value.deref().to_quantization_unchecked()),
                        Self::U64(value) => $struct_name::<NewQ>::new_from_spatial(value.deref().to_quantization_unchecked()),
                    }
                }

                fn try_into_quantization<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> Result<$struct_name<NewQ>, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                    match self {
                        Self::U8(value) => Ok($struct_name::<NewQ>::new_from_spatial(value.deref().try_to_quantization()?)),
                        Self::U16(value) => Ok($struct_name::<NewQ>::new_from_spatial(value.deref().try_to_quantization()?)),
                        Self::U32(value) => Ok($struct_name::<NewQ>::new_from_spatial(value.deref().try_to_quantization()?)),
                        Self::U64(value) => Ok($struct_name::<NewQ>::new_from_spatial(value.deref().try_to_quantization()?)),
                    }
                }

                fn into_quantization_clamped<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $struct_name<NewQ> {
                    match self {
                        Self::U8(value) => $struct_name::<NewQ>::new_from_spatial(value.deref().to_quantization_clamped()),
                        Self::U16(value) => $struct_name::<NewQ>::new_from_spatial(value.deref().to_quantization_clamped()),
                        Self::U32(value) => $struct_name::<NewQ>::new_from_spatial(value.deref().to_quantization_clamped()),
                        Self::U64(value) => $struct_name::<NewQ>::new_from_spatial(value.deref().to_quantization_clamped()),
                    }
                }
            }
        }
    };
}

/// Creates a wrapper for spatial unsigned-integer dimensions.
#[macro_export]
macro_rules! create_wrapped_unsigned_integer_spatial_dimensions {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_name:ident,
        $coord_struct:ident,
        $linear_index:ident,
        $linear_count:ident,
        $num_dimensions:expr,
        $( ($index:tt, $field:ident, $wrapped_axis:ident) ),+ $(,)?
    ) => {
        $(#[$meta])*
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $vis struct $struct_name<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
            $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions>
        );

        ::paste::paste! {
            impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> $struct_name<Q> {
                pub const NUM_DIMS: usize = $num_dimensions;

                pub const fn const_new(
                    value: $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions>
                ) -> Self {
                    Self(value)
                }

                pub fn try_new($( $field: $wrapped_axis<Q> ),+ ) -> Result<Self, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    $(
                        if *$field.as_ref() == Q::QUANT_ZERO {
                            return Err($crate::values::spatial::feagi_data_values_spatial_error::FeagiFailDimensionsCannotBeZero::new("Dimension axis cannot have a length of zero!").into());
                        }
                    )+
                    Ok(Self(
                        $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial::new_from_array([
                            $( *$field.as_ref() ),+
                        ])
                    ))
                }

                pub fn try_new_from_usizes($( $field: usize ),+ ) -> Result<Self, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    $(let $field: Q = Q::quant_try_from_usize($field).map_err(|_| $crate::values::spatial::feagi_data_values_spatial_error::FeagiFailInvalidSpatialQuantization::new("Given usize does not fit current dimension quantization").into())?; )+
                    Self::try_new($( $field ),+)
                }

                pub fn new_unchecked($( $field: $wrapped_axis<Q> ),+ ) -> Self {
                    Self(
                        $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial::new_from_array([
                            $( *$field.as_ref() ),+
                        ])
                    )
                }

                pub fn new_from_usizes_unchecked($( $field: usize ),+ ) -> Self {
                    Self(
                        $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial::new_from_usize_array_unchecked([
                            $( $field ),+
                        ])
                    )
                }

                pub fn new_from_spatial(
                    value: $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions>
                ) -> Self {
                    Self(value)
                }

                pub fn deref(
                    self
                ) -> $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions> {
                    self.0
                }

                pub fn as_spatial(&self) -> &$crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions> {
                    &self.0
                }

                pub fn number_contained_elements(&self) -> $linear_count<Q> {
                    $linear_count::new(
                        <Self as $crate::values::spatial::unsigned_integer::WrappedUnsignedIntegerSpatialDimensions>::number_contained_elements(self)
                    )
                }

                pub fn contains_coordinate(&self, coord: &$coord_struct<Q>) -> bool {
                    <Self as $crate::values::spatial::unsigned_integer::WrappedUnsignedIntegerSpatialDimensions>::contains_coordinate(self, coord.as_spatial())
                }

                pub fn contains_linear_index(&self, linear_index: $linear_index<Q>) -> bool {
                    <Self as $crate::values::spatial::unsigned_integer::WrappedUnsignedIntegerSpatialDimensions>::contains_linear_index(self, *linear_index.as_ref())
                }

                pub fn coordinate_to_linear_index(
                    &self,
                    coord: $coord_struct<Q>
                ) -> Result<$linear_index<Q>, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    Ok($linear_index::new(
                        <Self as $crate::values::spatial::unsigned_integer::WrappedUnsignedIntegerSpatialDimensions>::coordinate_to_linear_index(self, coord.deref())?
                    ))
                }

                pub fn linear_index_to_coordinate(
                    &self,
                    linear_index: $linear_index<Q>
                ) -> Result<$coord_struct<Q>, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    Ok($coord_struct::new_from_spatial(
                        <Self as $crate::values::spatial::unsigned_integer::WrappedUnsignedIntegerSpatialDimensions>::linear_index_to_coordinate(self, *linear_index.as_ref())?
                    ))
                }

                pub fn coordinate_to_linear_index_unchecked(
                    &self,
                    coord: $coord_struct<Q>
                ) -> $linear_index<Q> {
                    $linear_index::new(
                        <Self as $crate::values::spatial::unsigned_integer::WrappedUnsignedIntegerSpatialDimensions>::coordinate_to_linear_index_unchecked(self, coord.deref())
                    )
                }

                pub fn linear_index_to_coordinate_unchecked(
                    &self,
                    linear_index: $linear_index<Q>
                ) -> $coord_struct<Q> {
                    $coord_struct::new_from_spatial(
                        <Self as $crate::values::spatial::unsigned_integer::WrappedUnsignedIntegerSpatialDimensions>::linear_index_to_coordinate_unchecked(self, *linear_index.as_ref())
                    )
                }

                pub fn iter_coordinates(&self) -> [<$struct_name CoordinateIter>]<Q> {
                    [<$struct_name CoordinateIter>] {
                        dimensions: *self.as_spatial().as_slice(),
                        current: [Q::QUANT_ZERO; $num_dimensions],
                        remaining: <Self as $crate::values::spatial::unsigned_integer::WrappedUnsignedIntegerSpatialDimensions>::number_contained_elements(self).quant_to_usize(),
                    }
                }

                $(
                    pub fn [<get_ $field>](&self) -> &$wrapped_axis<Q> {
                        (&self.0.as_slice()[$index]).into()
                    }

                    pub fn [<get_ $field _mut>](&mut self) -> &mut $wrapped_axis<Q> {
                        (&mut self.0.as_slice_mut()[$index]).into()
                    }
                )+
            }

            #[doc = concat!("Iterator over every coordinate contained within a [`", stringify!($struct_name), "`], incrementing along the first axis fastest.")]
            $vis struct [<$struct_name CoordinateIter>]<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> {
                dimensions: [Q; $num_dimensions],
                current: [Q; $num_dimensions],
                remaining: usize,
            }

            impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> Iterator for [<$struct_name CoordinateIter>]<Q> {
                type Item = $coord_struct<Q>;

                fn next(&mut self) -> Option<Self::Item> {
                    if self.remaining == 0 {
                        return None;
                    }
                    let coordinate = $coord_struct::new_from_spatial(
                        $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial::new_from_array(self.current)
                    );
                    self.remaining -= 1;
                    if self.remaining > 0 {
                        for (axis, size) in self.current.iter_mut().zip(self.dimensions.iter()) {
                            *axis = *axis + Q::QUANT_ONE;
                            if *axis < *size {
                                break;
                            }
                            *axis = Q::QUANT_ZERO;
                        }
                    }
                    Some(coordinate)
                }

                fn size_hint(&self) -> (usize, Option<usize>) {
                    (self.remaining, Some(self.remaining))
                }
            }

            impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> ExactSizeIterator for [<$struct_name CoordinateIter>]<Q> {}

            impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>
                $crate::values::spatial::unsigned_integer::WrappedUnsignedIntegerSpatialDimensions for $struct_name<Q>
            {
                type Quant = Q;
                const NUM_DIMS: usize = $num_dimensions;

                fn new(
                    value: $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions>
                ) -> Self {
                    Self(value)
                }

                fn deref(
                    self
                ) -> $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions> {
                    self.0
                }

                fn spatial_slice(&self) -> &[Q] {
                    self.0.as_slice()
                }
            }

            impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>
                From<$crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions>>
                for $struct_name<Q>
            {
                fn from(value: $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions>) -> Self {
                    Self(value)
                }
            }

            impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>
                AsRef<$crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions>>
                for $struct_name<Q>
            {
                fn as_ref(&self) -> &$crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions> {
                    &self.0
                }
            }

            impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>
                AsMut<$crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions>>
                for $struct_name<Q>
            {
                fn as_mut(&mut self) -> &mut $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial<Q, $num_dimensions> {
                    &mut self.0
                }
            }

            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            $vis enum [<$struct_name Enum>] {
                U8($struct_name<u8>),
                U16($struct_name<u16>),
                U32($struct_name<u32>),
                U64($struct_name<u64>),
            }

            impl [<$struct_name Enum>] {
                pub fn new_from_quantized<FromQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    value: $struct_name<FromQ>
                ) -> Self {
                    <Self as $crate::values::spatial::unsigned_integer::WrappedUnsignedIntegerSpatialEnum>::new_from_quantized(value)
                }

                pub fn into_quantization_unchecked<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $struct_name<NewQ> {
                    <Self as $crate::values::spatial::unsigned_integer::WrappedUnsignedIntegerSpatialEnum>::into_quantization_unchecked(self)
                }

                pub fn try_into_quantization<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> Result<$struct_name<NewQ>, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                    <Self as $crate::values::spatial::unsigned_integer::WrappedUnsignedIntegerSpatialEnum>::try_into_quantization(self)
                }

                pub fn into_quantization_clamped<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $struct_name<NewQ> {
                    <Self as $crate::values::spatial::unsigned_integer::WrappedUnsignedIntegerSpatialEnum>::into_quantization_clamped(self)
                }
            }

            impl $crate::values::spatial::unsigned_integer::WrappedUnsignedIntegerSpatialEnum for [<$struct_name Enum>] {
                const NUM_DIMS: usize = $num_dimensions;
                type Shape<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> = $struct_name<Q>;

                fn get_level(&self) -> $crate::values::quantizable::UnsignedIntegerQuantizationLevel {
                    match self {
                        Self::U8(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U8,
                        Self::U16(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U16,
                        Self::U32(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U32,
                        Self::U64(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U64,
                    }
                }

                fn new_from_quantized<FromQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    value: $struct_name<FromQ>
                ) -> Self {
                    match FromQ::LEVEL {
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U8 => {
                            Self::U8($struct_name::<u8>::new_from_spatial(
                                $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial::<u8, $num_dimensions>::new_from_quant_unchecked(value.deref()),
                            ))
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U16 => {
                            Self::U16($struct_name::<u16>::new_from_spatial(
                                $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial::<u16, $num_dimensions>::new_from_quant_unchecked(value.deref()),
                            ))
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U32 => {
                            Self::U32($struct_name::<u32>::new_from_spatial(
                                $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial::<u32, $num_dimensions>::new_from_quant_unchecked(value.deref()),
                            ))
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U64 => {
                            Self::U64($struct_name::<u64>::new_from_spatial(
                                $crate::values::spatial::unsigned_integer::UnsignedIntegerSpatial::<u64, $num_dimensions>::new_from_quant_unchecked(value.deref()),
                            ))
                        }
                    }
                }

                fn into_quantization_unchecked<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $struct_name<NewQ> {
                    match self {
                        Self::U8(value) => $struct_name::<NewQ>::new_from_spatial(value.deref().to_quantization_unchecked()),
                        Self::U16(value) => $struct_name::<NewQ>::new_from_spatial(value.deref().to_quantization_unchecked()),
                        Self::U32(value) => $struct_name::<NewQ>::new_from_spatial(value.deref().to_quantization_unchecked()),
                        Self::U64(value) => $struct_name::<NewQ>::new_from_spatial(value.deref().to_quantization_unchecked()),
                    }
                }

                fn try_into_quantization<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> Result<$struct_name<NewQ>, $crate::values::quantizable::FeagiDataValueQuantizationError> {
                    match self {
                        Self::U8(value) => Ok($struct_name::<NewQ>::new_from_spatial(value.deref().try_to_quantization()?)),
                        Self::U16(value) => Ok($struct_name::<NewQ>::new_from_spatial(value.deref().try_to_quantization()?)),
                        Self::U32(value) => Ok($struct_name::<NewQ>::new_from_spatial(value.deref().try_to_quantization()?)),
                        Self::U64(value) => Ok($struct_name::<NewQ>::new_from_spatial(value.deref().try_to_quantization()?)),
                    }
                }

                fn into_quantization_clamped<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $struct_name<NewQ> {
                    match self {
                        Self::U8(value) => $struct_name::<NewQ>::new_from_spatial(value.deref().to_quantization_clamped()),
                        Self::U16(value) => $struct_name::<NewQ>::new_from_spatial(value.deref().to_quantization_clamped()),
                        Self::U32(value) => $struct_name::<NewQ>::new_from_spatial(value.deref().to_quantization_clamped()),
                        Self::U64(value) => $struct_name::<NewQ>::new_from_spatial(value.deref().to_quantization_clamped()),
                    }
                }
            }
        }
    };
}
