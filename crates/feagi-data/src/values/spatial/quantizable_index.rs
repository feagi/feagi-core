//region Internal macros
/// Shared behavior for enums that hide the quantization generic of spatial index
/// structures (coordinates or dimensions).
pub trait QuantizedIndexSpatialEnum:
    Copy + Clone + Send + Sync + core::fmt::Debug + core::cmp::PartialEq + core::cmp::Eq + core::hash::Hash + Sized + 'static
{
    type Shape<Q: crate::values::quantizable::QuantizedUnsignedIntegerTrait>;

    fn get_level(&self) -> crate::values::quantizable::UnsignedIntegerQuantizationLevel;

    fn new_from_quantized<FromQ: crate::values::quantizable::QuantizedUnsignedIntegerTrait>(value: Self::Shape<FromQ>) -> Self;

    fn into_quantization_unchecked<NewQ: crate::values::quantizable::QuantizedUnsignedIntegerTrait>(self) -> Self::Shape<NewQ>;

    fn try_into_quantization<NewQ: crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
        self,
    ) -> Result<Self::Shape<NewQ>, crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError>;

    fn into_quantization_clamped<NewQ: crate::values::quantizable::QuantizedUnsignedIntegerTrait>(self) -> Self::Shape<NewQ>;
}

/// Shared behavior for enums that hide the quantization generic of wrapped spatial index
/// structures (wrapped coordinates or wrapped dimensions).
pub trait WrappedQuantizedIndexSpatialEnum:
    Copy + Clone + Send + Sync + core::fmt::Debug + core::cmp::PartialEq + core::cmp::Eq + core::hash::Hash + Sized + 'static
{
    type WrappedShape<Q: crate::values::quantizable::QuantizedUnsignedIntegerTrait>;

    fn get_level(&self) -> crate::values::quantizable::UnsignedIntegerQuantizationLevel;

    fn new_from_quantized<FromQ: crate::values::quantizable::QuantizedUnsignedIntegerTrait>(value: Self::WrappedShape<FromQ>) -> Self;

    fn into_quantization_unchecked<NewQ: crate::values::quantizable::QuantizedUnsignedIntegerTrait>(self) -> Self::WrappedShape<NewQ>;

    fn try_into_quantization<NewQ: crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
        self,
    ) -> Result<Self::WrappedShape<NewQ>, crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError>;

    fn into_quantization_clamped<NewQ: crate::values::quantizable::QuantizedUnsignedIntegerTrait>(self) -> Self::WrappedShape<NewQ>;
}

macro_rules! create_coordinate {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_name:ident,
        $num_dimensions:expr,
        $( ($index:tt, $field:ident) ),+ $(,)?
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[derive(serde::Serialize, serde::Deserialize)]
        $vis struct $struct_name<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> {
            pub(crate) inner: [Q; $num_dimensions],
        }

        ::paste::paste! {
            impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> $struct_name<Q> {
                pub fn new( $( $field: Q ),+ ) -> Self {
                    $struct_name {
                        inner: [ $( $field ),+ ]
                    }
                }

                pub fn try_new_from_usizes($( $field: usize ),+ ) -> Result<Self, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    $(let $field: Q = Q::quant_try_from_usize($field)
                        .map_err(|_| $crate::values::spatial::feagi_data_values_spatial_error::FeagiFailInvalidSpatialQuantization::new("Given usize does not fit current coordinate quantization").into() )?; )+
                    Ok(Self::new($( $field),+))
                }

                /// Constructs from usizes without checking that each value fits within the
                /// current quantization. Out of range values are silently truncated.
                pub fn new_from_usizes_unchecked($( $field: usize ),+ ) -> Self {
                    Self::new($( Q::quant_from_usize($field) ),+)
                }

                /// Converts this coordinate from its current quantization to another quantization (without checking if valid)
                pub fn to_quantization_unchecked<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(self) -> $struct_name<NewQ> {
                    $struct_name::new($( self.inner[$index].to_quantization() ),+)
                }

                /// Tries to convert this coordinate from its current quantization to another quantization, returning an error if any axis value does not fit
                pub fn try_to_quantization<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(self) -> Result<$struct_name<NewQ>, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    Ok($struct_name::new(
                        $(
                            self.inner[$index].try_to_quantization().map_err(|_| $crate::values::spatial::feagi_data_values_spatial_error::FeagiFailSpatialQuantizationOutOfRange::new("A coordinate axis value does not fit in the target quantization").into() )?
                        ),+
                    ))
                }

                /// Converts this coordinate from its current quantization to another quantization, clamping each axis value to fit
                pub fn to_quantization_clamped<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(self) -> $struct_name<NewQ> {
                    $struct_name::new($( self.inner[$index].to_quantization_clamped() ),+)
                }

                $(
                    pub fn [<get_ $field>](&self) -> &Q {
                        &self.inner[$index]
                    }
                )+
                $(
                    pub fn [<get_ $field _mut>](&mut self) -> &mut Q {
                        &mut self.inner[$index]
                    }
                )+
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
                    <Self as $crate::values::spatial::quantizable_index::QuantizedIndexSpatialEnum>::new_from_quantized(value)
                }

                pub fn into_quantization_unchecked<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $struct_name<NewQ> {
                    <Self as $crate::values::spatial::quantizable_index::QuantizedIndexSpatialEnum>::into_quantization_unchecked(self)
                }

                pub fn try_into_quantization<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> Result<$struct_name<NewQ>, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    <Self as $crate::values::spatial::quantizable_index::QuantizedIndexSpatialEnum>::try_into_quantization(self)
                }

                pub fn into_quantization_clamped<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $struct_name<NewQ> {
                    <Self as $crate::values::spatial::quantizable_index::QuantizedIndexSpatialEnum>::into_quantization_clamped(self)
                }
            }

            impl $crate::values::spatial::quantizable_index::QuantizedIndexSpatialEnum for [<$struct_name Enum>] {
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
                            Self::U8(value.to_quantization_unchecked())
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U16 => {
                            Self::U16(value.to_quantization_unchecked())
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U32 => {
                            Self::U32(value.to_quantization_unchecked())
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U64
                        | $crate::values::quantizable::UnsignedIntegerQuantizationLevel::Usize => {
                            Self::U64(value.to_quantization_unchecked())
                        }
                    }
                }

                fn into_quantization_unchecked<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $struct_name<NewQ> {
                    match self {
                        Self::U8(value) => value.to_quantization_unchecked(),
                        Self::U16(value) => value.to_quantization_unchecked(),
                        Self::U32(value) => value.to_quantization_unchecked(),
                        Self::U64(value) => value.to_quantization_unchecked(),
                    }
                }

                fn try_into_quantization<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> Result<$struct_name<NewQ>, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    match self {
                        Self::U8(value) => value.try_to_quantization(),
                        Self::U16(value) => value.try_to_quantization(),
                        Self::U32(value) => value.try_to_quantization(),
                        Self::U64(value) => value.try_to_quantization(),
                    }
                }

                fn into_quantization_clamped<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $struct_name<NewQ> {
                    match self {
                        Self::U8(value) => value.to_quantization_clamped(),
                        Self::U16(value) => value.to_quantization_clamped(),
                        Self::U32(value) => value.to_quantization_clamped(),
                        Self::U64(value) => value.to_quantization_clamped(),
                    }
                }
            }
        }
    };
}

macro_rules! create_dimension {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_name:ident,
        $coord_impl:ident,
        $num_dimensions:expr,
        $( ($index:tt, $field:ident) ),+ $(,)?
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[derive(serde::Serialize, serde::Deserialize)]
        $vis struct $struct_name<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> {
            pub(crate) inner: [Q; $num_dimensions],
        }

        ::paste::paste! {
            impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> $struct_name<Q> {
                pub fn try_new( $( $field: Q ),+ ) -> Result<Self, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    $(
                    if $field == Q::QUANT_ZERO {
                        return Err($crate::values::spatial::feagi_data_values_spatial_error::FeagiFailDimensionsCannotBeZero::new("Dimension axis cannot have a length of zero!").into())
                    }
                    )+

                    Ok($struct_name {
                        inner: [ $( $field ),+ ]
                    })
                }

                pub fn try_new_from_usizes($( $field: usize ),+ ) -> Result<Self, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    $(let $field: Q = Q::quant_try_from_usize($field).map_err(|_| $crate::values::spatial::feagi_data_values_spatial_error::FeagiFailInvalidSpatialQuantization::new("Given usize does not fit current dimension quantization").into() )?; )+
                    Self::try_new($( $field),+)
                }

                /// Constructs without checking that each axis is non-zero. A dimension with a
                /// zero length axis contains no coordinates and will misbehave in index math.
                pub fn new_unchecked( $( $field: Q ),+ ) -> Self {
                    $struct_name {
                        inner: [ $( $field ),+ ]
                    }
                }

                /// Constructs from usizes without checking that each value fits within the
                /// current quantization or that each axis is non-zero.
                pub fn new_from_usizes_unchecked($( $field: usize ),+ ) -> Self {
                    Self::new_unchecked($( Q::quant_from_usize($field) ),+)
                }

                /// Converts this dimension from its current quantization to another quantization (without checking if valid)
                pub fn to_quantization_unchecked<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(self) -> $struct_name<NewQ> {
                    $struct_name::new_unchecked($( self.inner[$index].to_quantization() ),+)
                }

                /// Tries to convert this dimension from its current quantization to another quantization, returning an error if any axis value does not fit
                pub fn try_to_quantization<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(self) -> Result<$struct_name<NewQ>, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    Ok($struct_name::new_unchecked(
                        $(
                            self.inner[$index].try_to_quantization().map_err(|_| $crate::values::spatial::feagi_data_values_spatial_error::FeagiFailSpatialQuantizationOutOfRange::new("A dimension axis value does not fit in the target quantization").into() )?
                        ),+
                    ))
                }

                /// Converts this dimension from its current quantization to another quantization, clamping each axis value to fit
                pub fn to_quantization_clamped<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(self) -> $struct_name<NewQ> {
                    $struct_name::new_unchecked($( self.inner[$index].to_quantization_clamped() ),+)
                }

                /// Total number of discrete coordinates contained within these dimensions
                /// (the product of every axis).
                pub fn number_contained_elements(&self) -> Q {
                    $( self.inner[$index] * )+ Q::QUANT_ONE // multiply last num by 1 to make use of last *
                }

                /// Does a given coordinate fit within these dimensions
                pub fn contains_coordinate(&self, coord: &$coord_impl<Q>) -> bool {
                    $(
                    if coord.inner[$index] >= self.inner[$index]
                    {
                        return false;
                    }
                    )+
                    true
                }

                /// Does a given linear index fit within these dimensions
                pub fn contains_linear_index(&self, linear_index: Q) -> bool {
                    linear_index < self.number_contained_elements()
                }

                /// Converts a coordinate to its linear index, incrementing along the first axis
                /// fastest (x -> y -> z -> ...).
                pub fn coordinate_to_linear_index(&self, coord: $coord_impl<Q>) -> Result<Q, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError>  {

                    if !self.contains_coordinate(&coord) {
                        return Err(
                            $crate::values::spatial::feagi_data_values_spatial_error::FeagiFailInvalidSpatialIndex::new("Given given coordinate is out of bounds of the given dimensions!").into()
                        )
                    }
                    Ok(self.coordinate_to_linear_index_unchecked(coord))
                }

                /// Converts a linear index back into a coordinate, the inverse of
                /// [`coordinate_to_linear_index`](Self::coordinate_to_linear_index).
                pub fn linear_index_to_coordinate(&self, linear_index: Q) -> Result<$coord_impl<Q>, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {

                    if !self.contains_linear_index(linear_index)
                    {
                        return Err(
                            $crate::values::spatial::feagi_data_values_spatial_error::FeagiFailInvalidSpatialIndex::new("Given given linear index is out of bounds of the given dimensions!").into()
                        )
                    }
                    Ok(self.linear_index_to_coordinate_unchecked(linear_index))
                }

                /// Converts a coordinate to its linear index, incrementing along the first axis
                /// fastest (x -> y -> z -> ...) without any checking if the coordinate is valid
                pub fn coordinate_to_linear_index_unchecked(&self, coord: $coord_impl<Q>) -> Q {
                    let mut linear_index = Q::QUANT_ZERO;
                    let mut stride = Q::QUANT_ONE;
                    for (axis, size) in coord.inner.iter().zip(self.inner.iter()) {
                        linear_index = linear_index + (*axis * stride);
                        stride = stride * *size;
                    }
                    linear_index
                }

                /// Converts a linear index back into a coordinate without checking, the inverse of
                /// [`coordinate_to_linear_index_unchecked`](Self::coordinate_to_linear_index_unchecked).
                pub fn linear_index_to_coordinate_unchecked(&self, linear_index: Q) -> $coord_impl<Q> {
                    let mut coordinate = [Q::QUANT_ZERO; $num_dimensions];
                    let mut stride = Q::QUANT_ONE;
                    for (axis, size) in coordinate.iter_mut().zip(self.inner.iter()) {
                        *axis = (linear_index / stride) % *size;
                        stride = stride * *size;
                    }
                    $coord_impl { inner: coordinate }
                }

                /// Iterates over every coordinate contained within these dimensions, incrementing
                /// along the first axis (x) fastest, then the second (y), then the third (z), and
                /// so on. This matches the ordering used by
                /// [`coordinate_to_linear_index`](Self::coordinate_to_linear_index).
                pub fn iter_coordinates(&self) -> [<$struct_name CoordinateIter>]<Q> {
                    [<$struct_name CoordinateIter>] {
                        dimensions: self.inner,
                        current: [Q::QUANT_ZERO; $num_dimensions],
                        remaining: self.number_contained_elements().quant_to_usize(),
                    }
                }

                $(
                    pub fn [<get_ $field>](&self) -> &Q {
                        &self.inner[$index]
                    }
                )+
                $(
                    pub fn [<get_ $field _mut>](&mut self) -> &mut Q {
                        &mut self.inner[$index]
                    }
                )+
            }

            #[doc = concat!("Iterator over every coordinate contained within a [`", stringify!($struct_name), "`], incrementing along the first axis (x) fastest.")]
            $vis struct [<$struct_name CoordinateIter>]<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> {
                dimensions: [Q; $num_dimensions],
                current: [Q; $num_dimensions],
                remaining: usize,
            }

            impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> Iterator for [<$struct_name CoordinateIter>]<Q> {
                type Item = $coord_impl<Q>;

                fn next(&mut self) -> Option<Self::Item> {
                    if self.remaining == 0 {
                        return None;
                    }
                    let coordinate = $coord_impl { inner: self.current };
                    self.remaining -= 1;
                    if self.remaining > 0 {
                        // Increment odometer style, first axis (x) fastest.
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
                    <Self as $crate::values::spatial::quantizable_index::QuantizedIndexSpatialEnum>::new_from_quantized(value)
                }

                pub fn into_quantization_unchecked<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $struct_name<NewQ> {
                    <Self as $crate::values::spatial::quantizable_index::QuantizedIndexSpatialEnum>::into_quantization_unchecked(self)
                }

                pub fn try_into_quantization<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> Result<$struct_name<NewQ>, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    <Self as $crate::values::spatial::quantizable_index::QuantizedIndexSpatialEnum>::try_into_quantization(self)
                }

                pub fn into_quantization_clamped<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $struct_name<NewQ> {
                    <Self as $crate::values::spatial::quantizable_index::QuantizedIndexSpatialEnum>::into_quantization_clamped(self)
                }
            }

            impl $crate::values::spatial::quantizable_index::QuantizedIndexSpatialEnum for [<$struct_name Enum>] {
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
                            Self::U8(value.to_quantization_unchecked())
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U16 => {
                            Self::U16(value.to_quantization_unchecked())
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U32 => {
                            Self::U32(value.to_quantization_unchecked())
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U64
                        | $crate::values::quantizable::UnsignedIntegerQuantizationLevel::Usize => {
                            Self::U64(value.to_quantization_unchecked())
                        }
                    }
                }

                fn into_quantization_unchecked<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $struct_name<NewQ> {
                    match self {
                        Self::U8(value) => value.to_quantization_unchecked(),
                        Self::U16(value) => value.to_quantization_unchecked(),
                        Self::U32(value) => value.to_quantization_unchecked(),
                        Self::U64(value) => value.to_quantization_unchecked(),
                    }
                }

                fn try_into_quantization<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> Result<$struct_name<NewQ>, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    match self {
                        Self::U8(value) => value.try_to_quantization(),
                        Self::U16(value) => value.try_to_quantization(),
                        Self::U32(value) => value.try_to_quantization(),
                        Self::U64(value) => value.try_to_quantization(),
                    }
                }

                fn into_quantization_clamped<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $struct_name<NewQ> {
                    match self {
                        Self::U8(value) => value.to_quantization_clamped(),
                        Self::U16(value) => value.to_quantization_clamped(),
                        Self::U32(value) => value.to_quantization_clamped(),
                        Self::U64(value) => value.to_quantization_clamped(),
                    }
                }
            }
        }
    };
}

#[macro_export]
macro_rules! create_wrapped_quantized_index_coordinate {
    (
        $(#[$meta:meta])*
        $vis:vis $wrapper_struct_name:ident,
        $quant_index_coord_to_wrap:ident,
        $( ($index:tt, $field_name:ident, $field_wrapped_quant_index:ident) ),+ $(,)?
    ) => {
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[derive(serde::Serialize, serde::Deserialize)]
        #[serde(bound(
            serialize = "Q: serde::Serialize",
            deserialize = "Q: serde::Deserialize<'de>"
        ))]
        $vis struct $wrapper_struct_name<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>($quant_index_coord_to_wrap<Q>);

        ::paste::paste! {
            impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> $wrapper_struct_name<Q> {
                pub fn new( $( $field_name: $field_wrapped_quant_index<Q> ),+ ) -> Self {
                    $wrapper_struct_name (
                        $quant_index_coord_to_wrap::new(
                            $( *$field_name.as_ref() ),+
                        )
                    )
                }

                pub fn try_new_from_usizes($( $field_name: usize ),+ ) -> Result<Self, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    Ok($wrapper_struct_name(
                        $quant_index_coord_to_wrap::try_new_from_usizes($( $field_name ),+)?
                    ))
                }

                /// Constructs from usizes without checking that each value fits within the
                /// current quantization. Out of range values are silently truncated.
                pub fn new_from_usizes_unchecked($( $field_name: usize ),+ ) -> Self {
                    $wrapper_struct_name(
                        $quant_index_coord_to_wrap::new_from_usizes_unchecked($( $field_name ),+)
                    )
                }

                /// Converts this coordinate from its current quantization to another quantization (without checking if valid)
                pub fn to_quantization_unchecked<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(self) -> $wrapper_struct_name<NewQ> {
                    $wrapper_struct_name(self.0.to_quantization_unchecked())
                }

                /// Tries to convert this coordinate from its current quantization to another quantization, returning an error if any axis value does not fit
                pub fn try_to_quantization<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(self) -> Result<$wrapper_struct_name<NewQ>, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    Ok($wrapper_struct_name(self.0.try_to_quantization()?))
                }

                /// Converts this coordinate from its current quantization to another quantization, clamping each axis value to fit
                pub fn to_quantization_clamped<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(self) -> $wrapper_struct_name<NewQ> {
                    $wrapper_struct_name(self.0.to_quantization_clamped())
                }

                $(
                    pub fn [<get_ $field_name>](&self) -> &$field_wrapped_quant_index<Q> {
                        (&self.0.inner[$index]).into()
                    }
                )+
                $(
                    pub fn [<get_ $field_name _mut>](&mut self) -> &mut $field_wrapped_quant_index<Q> {
                        (&mut self.0.inner[$index]).into()
                    }
                )+
            }

            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            $vis enum [<$wrapper_struct_name Enum>] {
                U8($wrapper_struct_name<u8>),
                U16($wrapper_struct_name<u16>),
                U32($wrapper_struct_name<u32>),
                U64($wrapper_struct_name<u64>),
            }

            impl [<$wrapper_struct_name Enum>] {
                pub fn new_from_quantized<FromQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    value: $wrapper_struct_name<FromQ>
                ) -> Self {
                    <Self as $crate::values::spatial::quantizable_index::WrappedQuantizedIndexSpatialEnum>::new_from_quantized(value)
                }

                pub fn into_quantization_unchecked<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $wrapper_struct_name<NewQ> {
                    <Self as $crate::values::spatial::quantizable_index::WrappedQuantizedIndexSpatialEnum>::into_quantization_unchecked(self)
                }

                pub fn try_into_quantization<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> Result<$wrapper_struct_name<NewQ>, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    <Self as $crate::values::spatial::quantizable_index::WrappedQuantizedIndexSpatialEnum>::try_into_quantization(self)
                }

                pub fn into_quantization_clamped<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $wrapper_struct_name<NewQ> {
                    <Self as $crate::values::spatial::quantizable_index::WrappedQuantizedIndexSpatialEnum>::into_quantization_clamped(self)
                }
            }

            impl $crate::values::spatial::quantizable_index::WrappedQuantizedIndexSpatialEnum for [<$wrapper_struct_name Enum>] {
                type WrappedShape<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> = $wrapper_struct_name<Q>;

                fn get_level(&self) -> $crate::values::quantizable::UnsignedIntegerQuantizationLevel {
                    match self {
                        Self::U8(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U8,
                        Self::U16(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U16,
                        Self::U32(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U32,
                        Self::U64(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U64,
                    }
                }

                fn new_from_quantized<FromQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    value: $wrapper_struct_name<FromQ>
                ) -> Self {
                    match FromQ::LEVEL {
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U8 => {
                            Self::U8(value.to_quantization_unchecked())
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U16 => {
                            Self::U16(value.to_quantization_unchecked())
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U32 => {
                            Self::U32(value.to_quantization_unchecked())
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U64
                        | $crate::values::quantizable::UnsignedIntegerQuantizationLevel::Usize => {
                            Self::U64(value.to_quantization_unchecked())
                        }
                    }
                }

                fn into_quantization_unchecked<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $wrapper_struct_name<NewQ> {
                    match self {
                        Self::U8(value) => value.to_quantization_unchecked(),
                        Self::U16(value) => value.to_quantization_unchecked(),
                        Self::U32(value) => value.to_quantization_unchecked(),
                        Self::U64(value) => value.to_quantization_unchecked(),
                    }
                }

                fn try_into_quantization<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> Result<$wrapper_struct_name<NewQ>, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    match self {
                        Self::U8(value) => value.try_to_quantization(),
                        Self::U16(value) => value.try_to_quantization(),
                        Self::U32(value) => value.try_to_quantization(),
                        Self::U64(value) => value.try_to_quantization(),
                    }
                }

                fn into_quantization_clamped<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $wrapper_struct_name<NewQ> {
                    match self {
                        Self::U8(value) => value.to_quantization_clamped(),
                        Self::U16(value) => value.to_quantization_clamped(),
                        Self::U32(value) => value.to_quantization_clamped(),
                        Self::U64(value) => value.to_quantization_clamped(),
                    }
                }
            }
        }
    };
}

#[macro_export]
macro_rules! create_wrapped_quantized_index_dimension {
    (
        $(#[$meta:meta])*
        $vis:vis $wrapper_struct_name:ident,
        $quant_index_dim_to_wrap:ident,
        $wrapped_quant_index_coord:ident,
        $wrapped_quant_index_linear:ident,
        $( ($index:tt, $field_name:ident, $field_wrapped_quant_index:ident) ),+ $(,)?
    ) => {
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[derive(serde::Serialize, serde::Deserialize)]
        #[serde(bound(
            serialize = "Q: serde::Serialize",
            deserialize = "Q: serde::Deserialize<'de>"
        ))]
        $vis struct $wrapper_struct_name<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>($quant_index_dim_to_wrap<Q>);

        ::paste::paste! {
            impl<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> $wrapper_struct_name<Q> {
                pub fn try_new( $( $field_name: $field_wrapped_quant_index<Q> ),+ ) -> Result<Self, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    Ok($wrapper_struct_name(
                        $quant_index_dim_to_wrap::try_new(
                            $( *$field_name.as_ref() ),+
                        )?
                    ))
                }

                pub fn try_new_from_usizes($( $field_name: usize ),+ ) -> Result<Self, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    Ok($wrapper_struct_name(
                        $quant_index_dim_to_wrap::try_new_from_usizes($( $field_name ),+)?
                    ))
                }

                /// Constructs without checking that each axis is non-zero. A dimension with a
                /// zero length axis contains no coordinates and will misbehave in index math.
                pub fn new_unchecked( $( $field_name: $field_wrapped_quant_index<Q> ),+ ) -> Self {
                    $wrapper_struct_name(
                        $quant_index_dim_to_wrap::new_unchecked(
                            $( *$field_name.as_ref() ),+
                        )
                    )
                }

                /// Constructs from usizes without checking that each value fits within the
                /// current quantization or that each axis is non-zero.
                pub fn new_from_usizes_unchecked($( $field_name: usize ),+ ) -> Self {
                    $wrapper_struct_name(
                        $quant_index_dim_to_wrap::new_from_usizes_unchecked($( $field_name ),+)
                    )
                }

                /// Converts this dimension from its current quantization to another quantization (without checking if valid)
                pub fn to_quantization_unchecked<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(self) -> $wrapper_struct_name<NewQ> {
                    $wrapper_struct_name(self.0.to_quantization_unchecked())
                }

                /// Tries to convert this dimension from its current quantization to another quantization, returning an error if any axis value does not fit
                pub fn try_to_quantization<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(self) -> Result<$wrapper_struct_name<NewQ>, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    Ok($wrapper_struct_name(self.0.try_to_quantization()?))
                }

                /// Converts this dimension from its current quantization to another quantization, clamping each axis value to fit
                pub fn to_quantization_clamped<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(self) -> $wrapper_struct_name<NewQ> {
                    $wrapper_struct_name(self.0.to_quantization_clamped())
                }

                /// Total number of discrete coordinates contained within these dimensions
                /// (the product of every axis).
                pub fn number_contained_elements(&self) -> $wrapped_quant_index_linear<Q> {
                    $wrapped_quant_index_linear(self.0.number_contained_elements())
                }

                /// Does a given coordinate fit within these dimensions
                pub fn contains_coordinate(&self, coord: &$wrapped_quant_index_coord<Q>) -> bool {
                    self.0.contains_coordinate(&coord.0)
                }

                /// Does a given linear index fit within these dimensions
                pub fn contains_linear_index(&self, linear_index: $wrapped_quant_index_linear<Q>) -> bool {
                    self.0.contains_linear_index(*linear_index.as_ref())
                }

                /// Converts a coordinate to its linear index, incrementing along the first axis
                /// fastest (x -> y -> z -> ...).
                pub fn coordinate_to_linear_index(&self, coord: $wrapped_quant_index_coord<Q>) -> Result<$wrapped_quant_index_linear<Q>, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    Ok($wrapped_quant_index_linear(self.0.coordinate_to_linear_index(coord.0)?))
                }

                /// Converts a linear index back into a coordinate, the inverse of
                /// [`coordinate_to_linear_index`](Self::coordinate_to_linear_index).
                pub fn linear_index_to_coordinate(&self, linear_index: $wrapped_quant_index_linear<Q>) -> Result<$wrapped_quant_index_coord<Q>, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    Ok($wrapped_quant_index_coord(self.0.linear_index_to_coordinate(*linear_index.as_ref())?))
                }

                /// Converts a coordinate to its linear index, incrementing along the first axis
                /// fastest (x -> y -> z -> ...) without any checking if the coordinate is valid
                pub fn coordinate_to_linear_index_unchecked(&self, coord: $wrapped_quant_index_coord<Q>) -> $wrapped_quant_index_linear<Q> {
                    $wrapped_quant_index_linear(self.0.coordinate_to_linear_index_unchecked(coord.0))
                }

                /// Converts a linear index back into a coordinate without checking, the inverse of
                /// [`coordinate_to_linear_index_unchecked`](Self::coordinate_to_linear_index_unchecked).
                pub fn linear_index_to_coordinate_unchecked(&self, linear_index: $wrapped_quant_index_linear<Q>) -> $wrapped_quant_index_coord<Q> {
                    $wrapped_quant_index_coord(self.0.linear_index_to_coordinate_unchecked(*linear_index.as_ref()))
                }

                /// Iterates over every coordinate contained within these dimensions, incrementing
                /// along the first axis (x) fastest, then the second (y), then the third (z), and
                /// so on. This matches the ordering used by
                /// [`coordinate_to_linear_index`](Self::coordinate_to_linear_index).
                pub fn iter_coordinates(&self) -> impl ExactSizeIterator<Item = $wrapped_quant_index_coord<Q>> {
                    self.0.iter_coordinates().map(|coord| $wrapped_quant_index_coord(coord))
                }

                $(
                    pub fn [<get_ $field_name>](&self) -> &$field_wrapped_quant_index<Q> {
                        (&self.0.inner[$index]).into()
                    }
                )+
                $(
                    pub fn [<get_ $field_name _mut>](&mut self) -> &mut $field_wrapped_quant_index<Q> {
                        (&mut self.0.inner[$index]).into()
                    }
                )+
            }

            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            $vis enum [<$wrapper_struct_name Enum>] {
                U8($wrapper_struct_name<u8>),
                U16($wrapper_struct_name<u16>),
                U32($wrapper_struct_name<u32>),
                U64($wrapper_struct_name<u64>),
            }

            impl [<$wrapper_struct_name Enum>] {
                pub fn new_from_quantized<FromQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    value: $wrapper_struct_name<FromQ>
                ) -> Self {
                    <Self as $crate::values::spatial::quantizable_index::WrappedQuantizedIndexSpatialEnum>::new_from_quantized(value)
                }

                pub fn into_quantization_unchecked<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $wrapper_struct_name<NewQ> {
                    <Self as $crate::values::spatial::quantizable_index::WrappedQuantizedIndexSpatialEnum>::into_quantization_unchecked(self)
                }

                pub fn try_into_quantization<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> Result<$wrapper_struct_name<NewQ>, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    <Self as $crate::values::spatial::quantizable_index::WrappedQuantizedIndexSpatialEnum>::try_into_quantization(self)
                }

                pub fn into_quantization_clamped<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $wrapper_struct_name<NewQ> {
                    <Self as $crate::values::spatial::quantizable_index::WrappedQuantizedIndexSpatialEnum>::into_quantization_clamped(self)
                }
            }

            impl $crate::values::spatial::quantizable_index::WrappedQuantizedIndexSpatialEnum for [<$wrapper_struct_name Enum>] {
                type WrappedShape<Q: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> = $wrapper_struct_name<Q>;

                fn get_level(&self) -> $crate::values::quantizable::UnsignedIntegerQuantizationLevel {
                    match self {
                        Self::U8(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U8,
                        Self::U16(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U16,
                        Self::U32(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U32,
                        Self::U64(_) => $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U64,
                    }
                }

                fn new_from_quantized<FromQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    value: $wrapper_struct_name<FromQ>
                ) -> Self {
                    match FromQ::LEVEL {
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U8 => {
                            Self::U8(value.to_quantization_unchecked())
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U16 => {
                            Self::U16(value.to_quantization_unchecked())
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U32 => {
                            Self::U32(value.to_quantization_unchecked())
                        }
                        $crate::values::quantizable::UnsignedIntegerQuantizationLevel::U64
                        | $crate::values::quantizable::UnsignedIntegerQuantizationLevel::Usize => {
                            Self::U64(value.to_quantization_unchecked())
                        }
                    }
                }

                fn into_quantization_unchecked<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $wrapper_struct_name<NewQ> {
                    match self {
                        Self::U8(value) => value.to_quantization_unchecked(),
                        Self::U16(value) => value.to_quantization_unchecked(),
                        Self::U32(value) => value.to_quantization_unchecked(),
                        Self::U64(value) => value.to_quantization_unchecked(),
                    }
                }

                fn try_into_quantization<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> Result<$wrapper_struct_name<NewQ>, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    match self {
                        Self::U8(value) => value.try_to_quantization(),
                        Self::U16(value) => value.try_to_quantization(),
                        Self::U32(value) => value.try_to_quantization(),
                        Self::U64(value) => value.try_to_quantization(),
                    }
                }

                fn into_quantization_clamped<NewQ: $crate::values::quantizable::QuantizedUnsignedIntegerTrait>(
                    self
                ) -> $wrapper_struct_name<NewQ> {
                    match self {
                        Self::U8(value) => value.to_quantization_clamped(),
                        Self::U16(value) => value.to_quantization_clamped(),
                        Self::U32(value) => value.to_quantization_clamped(),
                        Self::U64(value) => value.to_quantization_clamped(),
                    }
                }
            }
        }
    };
}

//endregion

// These are the base spatial types everything will extend from

create_coordinate!(
    /// A 2D quantizable index coordinate
    pub QuantizedIndexCoord2D,
    2,
    (0, x), (1, y)
);

create_dimension!(
    /// A 2D quantizable index dimension
    pub QuantizedIndexDimension2D,
    QuantizedIndexCoord2D,
    2,
    (0, x), (1, y)
);

create_coordinate!(
    /// A 3D quantizable index coordinate
    pub QuantizedIndexCoord3D,
    3,
    (0, x), (1, y), (2, z)
);

create_dimension!(
    /// A 3D quantizable index dimension
    pub QuantizedIndexDimension3D,
    QuantizedIndexCoord3D,
    3,
    (0, x), (1, y), (2, z)
);

create_coordinate!(
    /// A 4D quantizable index coordinate
    pub QuantizedIndexCoord4D,
    4,
    (0, x), (1, y), (2, z), (3, w)
);

create_dimension!(
    /// A 4D quantizable index dimension
    pub QuantizedIndexDimension4D,
    QuantizedIndexCoord4D,
    4,
    (0, x), (1, y), (2, z), (3, w)
);
