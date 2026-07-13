// TODO check coordinate fit in dimension func in macro
// TODO right now the linear / coordinate index uses integer division, which is rather slow. We may want to use iterators instead
// TODO iter with coordinates within the dims

//region Internal macros

macro_rules! create_coordinate {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_name:ident,
        $num_dimensions:expr,
        $( ($index:tt, $field:ident) ),+ $(,)?
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $vis struct $struct_name<Q: $crate::values::quantizable::QuantizedIndexCountTrait> {
            pub inner: [Q; $num_dimensions],
        }

        ::paste::paste! {
            impl<Q: $crate::values::quantizable::QuantizedIndexCountTrait> $struct_name<Q> {
                pub fn new( $( $field: Q ),+ ) -> Self {
                    $struct_name {
                        inner: [ $( $field ),+ ]
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
        $vis struct $struct_name<Q: $crate::values::quantizable::QuantizedIndexCountTrait> {
            pub inner: [Q; $num_dimensions],
        }

        ::paste::paste! {
            impl<Q: $crate::values::quantizable::QuantizedIndexCountTrait> $struct_name<Q> {
                pub fn new( $( $field: Q ),+ ) -> Self {
                    $struct_name {
                        inner: [ $( $field ),+ ]
                    }
                }

                /// Total number of discrete coordinates contained within these dimensions
                /// (the product of every axis).
                pub fn number_contained_elements(&self) -> Q {
                    $( self.inner[$index] * )+ Q::QUANT_ONE // multiply last num by 1 to make use of last *
                }

                /// Converts a coordinate to its linear index, incrementing along the first axis
                /// fastest (x -> y -> z -> ...).
                pub fn coordinate_to_linear_index(&self, coord: $coord_impl<Q>) -> Q {
                    let mut linear_index = Q::QUANT_ZERO;
                    let mut stride = Q::QUANT_ONE;
                    for (axis, size) in coord.inner.iter().zip(self.inner.iter()) {
                        linear_index = linear_index + (*axis * stride);
                        stride = stride * *size;
                    }
                    linear_index
                }

                /// Converts a linear index back into a coordinate, the inverse of
                /// [`coordinate_to_linear_index`](Self::coordinate_to_linear_index).
                pub fn linear_to_coordinate_index(&self, linear_index: Q) -> $coord_impl<Q> {
                    let mut coordinate = [Q::QUANT_ZERO; $num_dimensions];
                    let mut stride = Q::QUANT_ONE;
                    for (axis, size) in coordinate.iter_mut().zip(self.inner.iter()) {
                        *axis = (linear_index / stride) % *size;
                        stride = stride * *size;
                    }
                    $coord_impl { inner: coordinate }
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
        $vis struct $wrapper_struct_name<Q: $crate::values::quantizable::QuantizedIndexCountTrait>($quant_index_coord_to_wrap<Q>);

        ::paste::paste! {
            impl<Q: $crate::values::quantizable::QuantizedIndexCountTrait> $wrapper_struct_name<Q> {
                pub fn new( $( $field_name: $field_wrapped_quant_index<Q> ),+ ) -> Self {
                    $wrapper_struct_name (
                        $quant_index_coord_to_wrap::new(
                            $( *$field_name.as_ref() ),+
                        )
                    )
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
        $vis struct $wrapper_struct_name<Q: $crate::values::quantizable::QuantizedIndexCountTrait>($quant_index_dim_to_wrap<Q>);

        ::paste::paste! {
            impl<Q: $crate::values::quantizable::QuantizedIndexCountTrait> $wrapper_struct_name<Q> {
                pub fn new( $( $field_name: $field_wrapped_quant_index<Q> ),+ ) -> Self {
                    $wrapper_struct_name (
                        $quant_index_dim_to_wrap::new(
                            $( *$field_name.as_ref() ),+
                        )
                    )
                }

                /// Total number of discrete coordinates contained within these dimensions
                /// (the product of every axis).
                pub fn number_contained_elements(&self) -> $wrapped_quant_index_linear<Q> {
                    $wrapped_quant_index_linear(self.0.number_contained_elements())
                }

                /// Converts a coordinate to its linear index, incrementing along the first axis
                /// fastest (x -> y -> z -> ...).
                pub fn coordinate_to_linear_index(&self, coord: $wrapped_quant_index_coord<Q>) -> $wrapped_quant_index_linear<Q> {
                    $wrapped_quant_index_linear(self.0.coordinate_to_linear_index(coord.0))
                }

                /// Converts a linear index back into a coordinate, the inverse of
                /// [`coordinate_to_linear_index`](Self::coordinate_to_linear_index).
                pub fn linear_to_coordinate_index(&self, linear_index: $wrapped_quant_index_linear<Q>) -> $wrapped_quant_index_coord<Q> {
                    $wrapped_quant_index_coord(self.0.linear_to_coordinate_index(*linear_index.as_ref()))
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
