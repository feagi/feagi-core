
// TODO check coordinate fit in dimension func in macro
// TODO add equals, hash

//region Internal macros

macro_rules! create_coordinate {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_name:ident,
        $num_dimensions:expr,
        $( ($index:tt, $field:ident) ),+ $(,)?
    ) => {
        $(#[$meta])*
        pub struct $struct_name<Q: crate::values::quantizable::QuantizedIndexCountTrait> {
            inner: [Q; $num_dimensions],
        }

        ::paste::paste! {
            impl<Q: crate::values::quantizable::QuantizedIndexCountTrait> $struct_name<Q> {
                $vis fn new( $( $field: Q ),+ ) -> Self {
                    $struct_name {
                        inner: [ $( $field ),+ ]
                    }
                }

                $(
                    $vis fn [<get_ $field>](&self) -> &Q {
                        &self.inner[$index]
                    }
                )+
                $(
                    $vis fn [<get_ $field _mut>](&mut self) -> &mut Q {
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
        $vis struct $struct_name<Q: crate::values::quantizable::QuantizedIndexCountTrait> {
            pub(crate) inner: [Q; $num_dimensions],
        }

        ::paste::paste! {
            impl<Q: crate::values::quantizable::QuantizedIndexCountTrait> $struct_name<Q> {
                pub fn new( $( $field: Q ),+ ) -> Self {
                    $struct_name {
                        inner: [ $( $field ),+ ]
                    }
                }

                /// Total number of discrete coordinates contained within these dimensions
                /// (the product of every axis).
                pub fn max_linear_index(&self) -> Q {
                    $( self.inner[$index] * )+ Q::QUANT_ONE
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


//endregion

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
