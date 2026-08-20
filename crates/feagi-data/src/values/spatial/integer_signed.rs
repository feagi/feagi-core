//region Internal macros

/// Shared behavior for enums that hide the quantization generic of spatial signed-integer
/// coordinates.
pub trait QuantizedSignedIntegerSpatialEnum:
    Copy + Clone + Send + Sync + core::fmt::Debug + core::cmp::PartialEq + core::cmp::Eq + core::hash::Hash + Sized + 'static
{
    type Shape<Q: crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>;

    fn get_level(&self) -> crate::values::quantizable::SignedIntegerQuantizationLevel;

    fn new_from_quantized<FromQ: crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(value: Self::Shape<FromQ>) -> Self;

    fn into_quantization_unchecked<NewQ: crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(self) -> Self::Shape<NewQ>;

    fn try_into_quantization<NewQ: crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(
        self,
    ) -> Result<Self::Shape<NewQ>, crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError>;

    fn into_quantization_clamped<NewQ: crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(self) -> Self::Shape<NewQ>;
}

/// Shared behavior for enums that hide the quantization generic of wrapped spatial signed-integer
/// coordinates.
pub trait WrappedQuantizedSignedIntegerSpatialEnum:
    Copy + Clone + Send + Sync + core::fmt::Debug + core::cmp::PartialEq + core::cmp::Eq + core::hash::Hash + Sized + 'static
{
    type WrappedShape<Q: crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>;

    fn get_level(&self) -> crate::values::quantizable::SignedIntegerQuantizationLevel;

    fn new_from_quantized<FromQ: crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(value: Self::WrappedShape<FromQ>) -> Self;

    fn into_quantization_unchecked<NewQ: crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(self) -> Self::WrappedShape<NewQ>;

    fn try_into_quantization<NewQ: crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(
        self,
    ) -> Result<Self::WrappedShape<NewQ>, crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError>;

    fn into_quantization_clamped<NewQ: crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(self) -> Self::WrappedShape<NewQ>;
}

fn signed_isize_fits_quant<Q: crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(value: isize) -> bool {
    match Q::LEVEL {
        crate::values::quantizable::SignedIntegerQuantizationLevel::I8 => (i8::MIN as isize) <= value && value <= (i8::MAX as isize),
        crate::values::quantizable::SignedIntegerQuantizationLevel::I16 => (i16::MIN as isize) <= value && value <= (i16::MAX as isize),
        crate::values::quantizable::SignedIntegerQuantizationLevel::I32 => (i32::MIN as isize) <= value && value <= (i32::MAX as isize),
        crate::values::quantizable::SignedIntegerQuantizationLevel::I64 => (i64::MIN as isize) <= value && value <= (i64::MAX as isize),
    }
}

fn clamp_isize_for_signed_quant<NewQ: crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(value: isize) -> isize {
    match NewQ::LEVEL {
        crate::values::quantizable::SignedIntegerQuantizationLevel::I8 => value.clamp(i8::MIN as isize, i8::MAX as isize),
        crate::values::quantizable::SignedIntegerQuantizationLevel::I16 => value.clamp(i16::MIN as isize, i16::MAX as isize),
        crate::values::quantizable::SignedIntegerQuantizationLevel::I32 => value.clamp(i32::MIN as isize, i32::MAX as isize),
        crate::values::quantizable::SignedIntegerQuantizationLevel::I64 => value.clamp(i64::MIN as isize, i64::MAX as isize),
    }
}

fn try_quant_from_isize<Q: crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(
    value: isize,
) -> Result<Q, crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
    if !signed_isize_fits_quant::<Q>(value) {
        return Err(
            crate::values::spatial::feagi_data_values_spatial_error::FeagiFailSpatialQuantizationOutOfRange::new(
                "A signed coordinate axis value does not fit in the target quantization",
            )
            .into(),
        );
    }
    Ok(Q::quant_from_isize(value))
}

macro_rules! create_signed_coordinate {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_name:ident,
        $num_dimensions:expr,
        $( ($index:tt, $field:ident) ),+ $(,)?
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[derive(serde::Serialize, serde::Deserialize)]
        $vis struct $struct_name<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait> {
            pub(crate) inner: [Q; $num_dimensions],
        }

        ::paste::paste! {
            impl<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait> $struct_name<Q> {
                pub fn new( $( $field: Q ),+ ) -> Self {
                    $struct_name {
                        inner: [ $( $field ),+ ]
                    }
                }

                pub fn try_new_from_isizes($( $field: isize ),+ ) -> Result<Self, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    $(let $field: Q = $crate::values::spatial::integer_signed::try_quant_from_isize($field)?; )+
                    Ok(Self::new($( $field ),+))
                }

                /// Constructs from isizes without checking that each value fits within the
                /// current quantization. Out of range values are silently truncated.
                pub fn new_from_isizes_unchecked($( $field: isize ),+ ) -> Self {
                    Self::new($( Q::quant_from_isize($field) ),+)
                }

                /// Converts this coordinate from its current quantization to another quantization (without checking if valid)
                pub fn to_quantization_unchecked<NewQ: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(self) -> $struct_name<NewQ> {
                    $struct_name::new($(
                        NewQ::quant_from_isize(
                            $crate::values::quantizable::SignedIntegerEnum::new_from_quantized(self.inner[$index]).to_isize()
                        )
                    ),+)
                }

                /// Tries to convert this coordinate from its current quantization to another quantization, returning an error if any axis value does not fit
                pub fn try_to_quantization<NewQ: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(self) -> Result<$struct_name<NewQ>, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    Ok($struct_name::new(
                        $(
                            $crate::values::quantizable::SignedIntegerEnum::new_from_quantized(self.inner[$index])
                                .try_into_quant::<NewQ>()
                                .map_err(|_| $crate::values::spatial::feagi_data_values_spatial_error::FeagiFailSpatialQuantizationOutOfRange::new("A signed coordinate axis value does not fit in the target quantization").into() )?
                        ),+
                    ))
                }

                /// Converts this coordinate from its current quantization to another quantization, clamping each axis value to fit
                pub fn to_quantization_clamped<NewQ: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(self) -> $struct_name<NewQ> {
                    $struct_name::new($(
                        NewQ::quant_from_isize(
                            $crate::values::spatial::integer_signed::clamp_isize_for_signed_quant::<NewQ>(
                                $crate::values::quantizable::SignedIntegerEnum::new_from_quantized(self.inner[$index]).to_isize()
                            )
                        )
                    ),+)
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
                I8($struct_name<i8>),
                I16($struct_name<i16>),
                I32($struct_name<i32>),
                I64($struct_name<i64>),
            }

            impl [<$struct_name Enum>] {
                pub fn new_from_quantized<FromQ: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(
                    value: $struct_name<FromQ>
                ) -> Self {
                    <Self as $crate::values::spatial::integer_signed::QuantizedSignedIntegerSpatialEnum>::new_from_quantized(value)
                }

                pub fn into_quantization_unchecked<NewQ: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(
                    self
                ) -> $struct_name<NewQ> {
                    <Self as $crate::values::spatial::integer_signed::QuantizedSignedIntegerSpatialEnum>::into_quantization_unchecked(self)
                }

                pub fn try_into_quantization<NewQ: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(
                    self
                ) -> Result<$struct_name<NewQ>, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    <Self as $crate::values::spatial::integer_signed::QuantizedSignedIntegerSpatialEnum>::try_into_quantization(self)
                }

                pub fn into_quantization_clamped<NewQ: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(
                    self
                ) -> $struct_name<NewQ> {
                    <Self as $crate::values::spatial::integer_signed::QuantizedSignedIntegerSpatialEnum>::into_quantization_clamped(self)
                }
            }

            impl $crate::values::spatial::integer_signed::QuantizedSignedIntegerSpatialEnum for [<$struct_name Enum>] {
                type Shape<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait> = $struct_name<Q>;

                fn get_level(&self) -> $crate::values::quantizable::SignedIntegerQuantizationLevel {
                    match self {
                        Self::I8(_) => $crate::values::quantizable::SignedIntegerQuantizationLevel::I8,
                        Self::I16(_) => $crate::values::quantizable::SignedIntegerQuantizationLevel::I16,
                        Self::I32(_) => $crate::values::quantizable::SignedIntegerQuantizationLevel::I32,
                        Self::I64(_) => $crate::values::quantizable::SignedIntegerQuantizationLevel::I64,
                    }
                }

                fn new_from_quantized<FromQ: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(
                    value: $struct_name<FromQ>
                ) -> Self {
                    match FromQ::LEVEL {
                        $crate::values::quantizable::SignedIntegerQuantizationLevel::I8 => {
                            Self::I8(value.to_quantization_unchecked())
                        }
                        $crate::values::quantizable::SignedIntegerQuantizationLevel::I16 => {
                            Self::I16(value.to_quantization_unchecked())
                        }
                        $crate::values::quantizable::SignedIntegerQuantizationLevel::I32 => {
                            Self::I32(value.to_quantization_unchecked())
                        }
                        $crate::values::quantizable::SignedIntegerQuantizationLevel::I64 => {
                            Self::I64(value.to_quantization_unchecked())
                        }
                    }
                }

                fn into_quantization_unchecked<NewQ: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(
                    self
                ) -> $struct_name<NewQ> {
                    match self {
                        Self::I8(value) => value.to_quantization_unchecked(),
                        Self::I16(value) => value.to_quantization_unchecked(),
                        Self::I32(value) => value.to_quantization_unchecked(),
                        Self::I64(value) => value.to_quantization_unchecked(),
                    }
                }

                fn try_into_quantization<NewQ: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(
                    self
                ) -> Result<$struct_name<NewQ>, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    match self {
                        Self::I8(value) => value.try_to_quantization(),
                        Self::I16(value) => value.try_to_quantization(),
                        Self::I32(value) => value.try_to_quantization(),
                        Self::I64(value) => value.try_to_quantization(),
                    }
                }

                fn into_quantization_clamped<NewQ: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(
                    self
                ) -> $struct_name<NewQ> {
                    match self {
                        Self::I8(value) => value.to_quantization_clamped(),
                        Self::I16(value) => value.to_quantization_clamped(),
                        Self::I32(value) => value.to_quantization_clamped(),
                        Self::I64(value) => value.to_quantization_clamped(),
                    }
                }
            }
        }
    };
}

#[macro_export]
macro_rules! create_wrapped_quantized_signed_integer_coordinate {
    (
        $(#[$meta:meta])*
        $vis:vis $wrapper_struct_name:ident,
        $signed_coord_to_wrap:ident,
        $( ($index:tt, $field_name:ident, $field_wrapped_quant_signed:ident) ),+ $(,)?
    ) => {
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[derive(serde::Serialize, serde::Deserialize)]
        #[serde(bound(
            serialize = "Q: serde::Serialize",
            deserialize = "Q: serde::Deserialize<'de>"
        ))]
        $vis struct $wrapper_struct_name<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>($signed_coord_to_wrap<Q>);

        ::paste::paste! {
            impl<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait> $wrapper_struct_name<Q> {
                pub fn new( $( $field_name: $field_wrapped_quant_signed<Q> ),+ ) -> Self {
                    $wrapper_struct_name (
                        $signed_coord_to_wrap::new(
                            $( *$field_name.as_ref() ),+
                        )
                    )
                }

                pub fn try_new_from_isizes($( $field_name: isize ),+ ) -> Result<Self, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    Ok($wrapper_struct_name(
                        $signed_coord_to_wrap::try_new_from_isizes($( $field_name ),+)?
                    ))
                }

                /// Constructs from isizes without checking that each value fits within the
                /// current quantization. Out of range values are silently truncated.
                pub fn new_from_isizes_unchecked($( $field_name: isize ),+ ) -> Self {
                    $wrapper_struct_name(
                        $signed_coord_to_wrap::new_from_isizes_unchecked($( $field_name ),+)
                    )
                }

                /// Converts this coordinate from its current quantization to another quantization (without checking if valid)
                pub fn to_quantization_unchecked<NewQ: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(self) -> $wrapper_struct_name<NewQ> {
                    $wrapper_struct_name(self.0.to_quantization_unchecked())
                }

                /// Tries to convert this coordinate from its current quantization to another quantization, returning an error if any axis value does not fit
                pub fn try_to_quantization<NewQ: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(self) -> Result<$wrapper_struct_name<NewQ>, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    Ok($wrapper_struct_name(self.0.try_to_quantization()?))
                }

                /// Converts this coordinate from its current quantization to another quantization, clamping each axis value to fit
                pub fn to_quantization_clamped<NewQ: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(self) -> $wrapper_struct_name<NewQ> {
                    $wrapper_struct_name(self.0.to_quantization_clamped())
                }

                $(
                    pub fn [<get_ $field_name>](&self) -> &$field_wrapped_quant_signed<Q> {
                        (&self.0.inner[$index]).into()
                    }
                )+
                $(
                    pub fn [<get_ $field_name _mut>](&mut self) -> &mut $field_wrapped_quant_signed<Q> {
                        (&mut self.0.inner[$index]).into()
                    }
                )+
            }

            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            $vis enum [<$wrapper_struct_name Enum>] {
                I8($wrapper_struct_name<i8>),
                I16($wrapper_struct_name<i16>),
                I32($wrapper_struct_name<i32>),
                I64($wrapper_struct_name<i64>),
            }

            impl [<$wrapper_struct_name Enum>] {
                pub fn new_from_quantized<FromQ: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(
                    value: $wrapper_struct_name<FromQ>
                ) -> Self {
                    <Self as $crate::values::spatial::integer_signed::WrappedQuantizedSignedIntegerSpatialEnum>::new_from_quantized(value)
                }

                pub fn into_quantization_unchecked<NewQ: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(
                    self
                ) -> $wrapper_struct_name<NewQ> {
                    <Self as $crate::values::spatial::integer_signed::WrappedQuantizedSignedIntegerSpatialEnum>::into_quantization_unchecked(self)
                }

                pub fn try_into_quantization<NewQ: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(
                    self
                ) -> Result<$wrapper_struct_name<NewQ>, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    <Self as $crate::values::spatial::integer_signed::WrappedQuantizedSignedIntegerSpatialEnum>::try_into_quantization(self)
                }

                pub fn into_quantization_clamped<NewQ: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(
                    self
                ) -> $wrapper_struct_name<NewQ> {
                    <Self as $crate::values::spatial::integer_signed::WrappedQuantizedSignedIntegerSpatialEnum>::into_quantization_clamped(self)
                }
            }

            impl $crate::values::spatial::integer_signed::WrappedQuantizedSignedIntegerSpatialEnum for [<$wrapper_struct_name Enum>] {
                type WrappedShape<Q: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait> = $wrapper_struct_name<Q>;

                fn get_level(&self) -> $crate::values::quantizable::SignedIntegerQuantizationLevel {
                    match self {
                        Self::I8(_) => $crate::values::quantizable::SignedIntegerQuantizationLevel::I8,
                        Self::I16(_) => $crate::values::quantizable::SignedIntegerQuantizationLevel::I16,
                        Self::I32(_) => $crate::values::quantizable::SignedIntegerQuantizationLevel::I32,
                        Self::I64(_) => $crate::values::quantizable::SignedIntegerQuantizationLevel::I64,
                    }
                }

                fn new_from_quantized<FromQ: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(
                    value: $wrapper_struct_name<FromQ>
                ) -> Self {
                    match FromQ::LEVEL {
                        $crate::values::quantizable::SignedIntegerQuantizationLevel::I8 => {
                            Self::I8(value.to_quantization_unchecked())
                        }
                        $crate::values::quantizable::SignedIntegerQuantizationLevel::I16 => {
                            Self::I16(value.to_quantization_unchecked())
                        }
                        $crate::values::quantizable::SignedIntegerQuantizationLevel::I32 => {
                            Self::I32(value.to_quantization_unchecked())
                        }
                        $crate::values::quantizable::SignedIntegerQuantizationLevel::I64 => {
                            Self::I64(value.to_quantization_unchecked())
                        }
                    }
                }

                fn into_quantization_unchecked<NewQ: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(
                    self
                ) -> $wrapper_struct_name<NewQ> {
                    match self {
                        Self::I8(value) => value.to_quantization_unchecked(),
                        Self::I16(value) => value.to_quantization_unchecked(),
                        Self::I32(value) => value.to_quantization_unchecked(),
                        Self::I64(value) => value.to_quantization_unchecked(),
                    }
                }

                fn try_into_quantization<NewQ: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(
                    self
                ) -> Result<$wrapper_struct_name<NewQ>, $crate::values::spatial::feagi_data_values_spatial_error::FeagiDataValuesSpatialError> {
                    match self {
                        Self::I8(value) => value.try_to_quantization(),
                        Self::I16(value) => value.try_to_quantization(),
                        Self::I32(value) => value.try_to_quantization(),
                        Self::I64(value) => value.try_to_quantization(),
                    }
                }

                fn into_quantization_clamped<NewQ: $crate::values::quantizable::QuantizedSignedIntegerUnwrappedTrait>(
                    self
                ) -> $wrapper_struct_name<NewQ> {
                    match self {
                        Self::I8(value) => value.to_quantization_clamped(),
                        Self::I16(value) => value.to_quantization_clamped(),
                        Self::I32(value) => value.to_quantization_clamped(),
                        Self::I64(value) => value.to_quantization_clamped(),
                    }
                }
            }
        }
    };
}

//endregion

create_signed_coordinate!(
    /// A signed (positive or negative) coordinate in 2D
    pub SignedCoordinate2D,
    2,
    (0, x), (1, y)
);

create_signed_coordinate!(
    /// A signed (positive or negative) coordinate in 3D
    pub SignedCoordinate3D,
    3,
    (0, x), (1, y), (2, z)
);

create_signed_coordinate!(
    /// A signed (positive or negative) coordinate in 4D
    pub SignedCoordinate4D,
    4,
    (0, x), (1, y), (2, z), (3, w)
);
