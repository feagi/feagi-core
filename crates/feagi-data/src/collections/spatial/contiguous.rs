use crate::values::spatial::quantizable_index::{
    QuantizedIndexCoord2D, QuantizedIndexCoord3D, QuantizedIndexCoord4D, QuantizedIndexDimension2D,
    QuantizedIndexDimension3D, QuantizedIndexDimension4D,
};

macro_rules! create_spatial_quantized_contiguous_vector {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_name:ident,
        $coord_impl:ident,
        $dim_impl:ident,
        $num_dimensions:expr,
    ) => {
        $(#[$meta])*
        $vis struct $struct_name<QI: $crate::values::quantizable::QuantizedIndexCountTrait, V: Clone + Copy> {
            pub(crate) data: $crate::collections::linear::contiguous_data::QuantizedContiguousVector<QI, V>,
            pub(crate) dimensions: $dim_impl<QI>,
        }

        impl<QI: $crate::values::quantizable::QuantizedIndexCountTrait, V: Clone + Copy> Clone for $struct_name<QI, V> {
            fn clone(&self) -> Self {
                Self {
                    data: self.data.clone(),
                    dimensions: self.dimensions,
                }
            }
        }

        impl<QI: $crate::values::quantizable::QuantizedIndexCountTrait, V: Clone + Copy> $struct_name<QI, V>
        {
            /// Number of spatial axes this collection is addressed by.
            pub const NUM_DIMENSIONS: usize = $num_dimensions;

            /// Builds a collection sized to `dimensions`, with every element set to
            /// `filling_value`.
            pub fn new_uniform(dimensions: $dim_impl<QI>, filling_value: V) -> $struct_name<QI, V> {
                let number_values = dimensions.number_contained_elements();
                Self {
                    data: $crate::collections::linear::contiguous_data::QuantizedContiguousVector::new_uniform(number_values, filling_value),
                    dimensions,
                }
            }

            /// Wraps an existing linear vector, checking that its length matches the
            /// number of elements described by `dimensions`.
            pub fn try_from_vector(
                data: $crate::collections::linear::contiguous_data::QuantizedContiguousVector<QI, V>,
                dimensions: $dim_impl<QI>,
            ) -> Result<$struct_name<QI, V>, $crate::values::feagi_data_value_error::FeagiValueError> {
                use $crate::collections::linear::contiguous_data::QuantizedContiguousTrait;
                if data.len() != dimensions.number_contained_elements() {
                    return Err($crate::values::feagi_data_value_error::FeagiDimensionsErrKey::new(
                        "backing vector length does not match the given dimensions",
                    )
                    .into());
                }
                Ok(Self { data, dimensions })
            }

            /// Wraps an existing `Vec`, checking that its length matches the number
            /// of elements described by `dimensions`.
            pub fn try_from_vec(
                data: Vec<V>,
                dimensions: $dim_impl<QI>,
            ) -> Result<$struct_name<QI, V>, $crate::values::feagi_data_value_error::FeagiValueError> {
                Self::try_from_vector(
                    $crate::collections::linear::contiguous_data::QuantizedContiguousVector::from_vec(data),
                    dimensions,
                )
            }

            /// Consumes the wrapper, returning the backing linear vector.
            pub fn into_vector(
                self,
            ) -> $crate::collections::linear::contiguous_data::QuantizedContiguousVector<QI, V> {
                self.data
            }

            /// Consumes the wrapper, returning the backing `Vec`.
            pub fn into_vec(self) -> Vec<V> {
                self.data.into_vec()
            }

            /// The dimensions (per-axis extent) of this collection.
            pub fn dimensions(&self) -> &$dim_impl<QI> {
                &self.dimensions
            }

            /// Total number of elements, expressed in the quantized index/count type.
            pub fn len(&self) -> QI {
                use $crate::collections::linear::contiguous_data::QuantizedContiguousTrait;
                self.data.len()
            }

            /// Returns `true` if there are no elements.
            pub fn is_empty(&self) -> bool {
                use $crate::collections::linear::contiguous_data::QuantizedContiguousTrait;
                self.data.is_empty()
            }

            //region Coordinate <-> linear index

            /// Returns `true` if every axis of `coordinate` is within the bounds of
            /// this collection's dimensions.
            pub fn contains_coordinate(&self, coordinate: &$coord_impl<QI>) -> bool {
                for (axis, size) in coordinate.inner.iter().zip(self.dimensions.inner.iter()) {
                    if *axis >= *size {
                        return false;
                    }
                }
                true
            }

            /// Converts a coordinate into its linear index, with the first axis
            /// varying fastest. Does not bounds-check the coordinate.
            pub fn coordinate_to_linear_index(&self, coordinate: $coord_impl<QI>) -> QI {
                self.dimensions.coordinate_to_linear_index(coordinate)
            }

            /// Converts a linear index back into its coordinate, the inverse of
            /// [`coordinate_to_linear_index`](Self::coordinate_to_linear_index).
            pub fn linear_to_coordinate(&self, linear_index: QI) -> $coord_impl<QI> {
                self.dimensions.linear_to_coordinate_index(linear_index)
            }

            //endregion

            //region Linear index access

            /// Copies out the element at the given linear `index`, or `None` if out
            /// of bounds.
            pub fn get(&self, index: QI) -> Option<V> {
                use $crate::collections::linear::contiguous_data::QuantizedContiguousTrait;
                self.data.get(index)
            }

            /// Mutably borrows the element at the given linear `index`, or `None` if
            /// out of bounds.
            pub fn get_mut(&mut self, index: QI) -> Option<&mut V> {
                use $crate::collections::linear::contiguous_data::QuantizedContiguousMutTrait;
                self.data.get_mut(index)
            }

            /// Overwrites the element at the given linear `index`, returning the
            /// previous value if the index was in bounds.
            pub fn set(&mut self, index: QI, value: V) -> Option<V> {
                use $crate::collections::linear::contiguous_data::QuantizedContiguousMutTrait;
                self.data.set(index, value)
            }

            //endregion

            //region Coordinate access

            /// Copies out the element at `coordinate`, or `None` if the coordinate is
            /// out of bounds.
            pub fn get_at(&self, coordinate: $coord_impl<QI>) -> Option<V> {
                if !self.contains_coordinate(&coordinate) {
                    return None;
                }
                self.get(self.coordinate_to_linear_index(coordinate))
            }

            /// Mutably borrows the element at `coordinate`, or `None` if the
            /// coordinate is out of bounds.
            pub fn get_at_mut(&mut self, coordinate: $coord_impl<QI>) -> Option<&mut V> {
                if !self.contains_coordinate(&coordinate) {
                    return None;
                }
                let index = self.coordinate_to_linear_index(coordinate);
                self.get_mut(index)
            }

            /// Overwrites the element at `coordinate`, returning the previous value
            /// if the coordinate was in bounds.
            pub fn set_at(&mut self, coordinate: $coord_impl<QI>, value: V) -> Option<V> {
                if !self.contains_coordinate(&coordinate) {
                    return None;
                }
                let index = self.coordinate_to_linear_index(coordinate);
                self.set(index, value)
            }

            //endregion

            //region Slices

            /// Borrows the whole collection's backing storage as a regular shared
            /// slice, in linear (x-fastest) order.
            pub fn as_slice(&self) -> &[V] {
                use $crate::collections::linear::contiguous_data::QuantizedContiguousTrait;
                self.data.as_slice()
            }

            /// Mutably borrows the whole collection's backing storage as a regular
            /// slice, in linear (x-fastest) order.
            pub fn as_mut_slice(&mut self) -> &mut [V] {
                use $crate::collections::linear::contiguous_data::QuantizedContiguousMutTrait;
                self.data.as_mut_slice()
            }

            /// Borrows the whole collection as a read-only quantized slice view.
            pub fn as_quantized_slice(
                &self,
            ) -> $crate::collections::linear::contiguous_data::QuantizedContiguousSlice<'_, QI, V> {
                use $crate::collections::linear::contiguous_data::QuantizedContiguousTrait;
                self.data.as_quantized_slice()
            }

            //endregion

            //region Iterators

            /// Iterates over shared references to the elements, in linear order.
            pub fn iter(&self) -> core::slice::Iter<'_, V> {
                use $crate::collections::linear::contiguous_data::QuantizedContiguousTrait;
                self.data.iter()
            }

            /// Iterates over mutable references to the elements, in linear order.
            pub fn iter_mut(&mut self) -> core::slice::IterMut<'_, V> {
                use $crate::collections::linear::contiguous_data::QuantizedContiguousMutTrait;
                self.data.iter_mut()
            }

            /// Iterates over `(linear_index, &value)` pairs, in linear order.
            pub fn iter_with_index(&self) -> impl Iterator<Item = (QI, &V)> + '_ {
                use $crate::collections::linear::contiguous_data::QuantizedContiguousTrait;
                self.data
                    .iter()
                    .enumerate()
                    .map(|(i, v)| (QI::from_usize(i), v))
            }

            /// Iterates over `(linear_index, &mut value)` pairs, in linear order.
            pub fn iter_mut_with_index(&mut self) -> impl Iterator<Item = (QI, &mut V)> + '_ {
                use $crate::collections::linear::contiguous_data::QuantizedContiguousMutTrait;
                self.data
                    .iter_mut()
                    .enumerate()
                    .map(|(i, v)| (QI::from_usize(i), v))
            }

            /// Iterates over `(coordinate, &value)` pairs, in linear order.
            pub fn iter_with_coordinate(&self) -> impl Iterator<Item = ($coord_impl<QI>, &V)> + '_ {
                use $crate::collections::linear::contiguous_data::QuantizedContiguousTrait;
                let dimensions = self.dimensions;
                self.data.iter().enumerate().map(move |(i, v)| {
                    (dimensions.linear_to_coordinate_index(QI::from_usize(i)), v)
                })
            }

            /// Iterates over `(coordinate, &mut value)` pairs, in linear order.
            pub fn iter_mut_with_coordinate(
                &mut self,
            ) -> impl Iterator<Item = ($coord_impl<QI>, &mut V)> + '_ {
                use $crate::collections::linear::contiguous_data::QuantizedContiguousMutTrait;
                let dimensions = self.dimensions;
                self.data.iter_mut().enumerate().map(move |(i, v)| {
                    (dimensions.linear_to_coordinate_index(QI::from_usize(i)), v)
                })
            }

            /// Iterates over `(linear_index, coordinate, &value)` triples, in linear
            /// order.
            pub fn iter_with_index_and_coordinate(
                &self,
            ) -> impl Iterator<Item = (QI, $coord_impl<QI>, &V)> + '_ {
                use $crate::collections::linear::contiguous_data::QuantizedContiguousTrait;
                let dimensions = self.dimensions;
                self.data.iter().enumerate().map(move |(i, v)| {
                    let linear_index = QI::from_usize(i);
                    (linear_index, dimensions.linear_to_coordinate_index(linear_index), v)
                })
            }

            /// Iterates over `(linear_index, coordinate, &mut value)` triples, in
            /// linear order.
            pub fn iter_mut_with_index_and_coordinate(
                &mut self,
            ) -> impl Iterator<Item = (QI, $coord_impl<QI>, &mut V)> + '_ {
                use $crate::collections::linear::contiguous_data::QuantizedContiguousMutTrait;
                let dimensions = self.dimensions;
                self.data.iter_mut().enumerate().map(move |(i, v)| {
                    let linear_index = QI::from_usize(i);
                    (linear_index, dimensions.linear_to_coordinate_index(linear_index), v)
                })
            }

            //endregion
        }

        impl<QI: $crate::values::quantizable::QuantizedIndexCountTrait, V: Clone + Copy>
            core::ops::Index<$coord_impl<QI>> for $struct_name<QI, V>
        {
            type Output = V;
            fn index(&self, coordinate: $coord_impl<QI>) -> &V {
                assert!(
                    self.contains_coordinate(&coordinate),
                    "coordinate is out of bounds of the collection's dimensions",
                );
                &self.data[self.coordinate_to_linear_index(coordinate)]
            }
        }

        impl<QI: $crate::values::quantizable::QuantizedIndexCountTrait, V: Clone + Copy>
            core::ops::IndexMut<$coord_impl<QI>> for $struct_name<QI, V>
        {
            fn index_mut(&mut self, coordinate: $coord_impl<QI>) -> &mut V {
                assert!(
                    self.contains_coordinate(&coordinate),
                    "coordinate is out of bounds of the collection's dimensions",
                );
                let index = self.coordinate_to_linear_index(coordinate);
                &mut self.data[index]
            }
        }
    }
}

#[macro_export]
macro_rules! create_spatial_bitpacked_vector {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_name:ident,
        $coord_impl:ident,
        $dim_impl:ident,
        $num_dimensions:expr,
    ) => {
        $(#[$meta])*
        $vis struct $struct_name<QI: $crate::values::quantizable::QuantizedIndexCountTrait> {
            pub(crate) data: $crate::collections::linear::bitpacked::BitPackedVector,
            pub(crate) dimensions: $dim_impl<QI>
        }

        impl<QI: $crate::values::quantizable::QuantizedIndexCountTrait> $struct_name<QI>
        {
            pub fn new_uniform(dimensions: $dim_impl<QI>, initial_state: bool) -> $struct_name<QI> {
                let linear = dimensions.max_linear_index();

                let values: Vec<u8> = if initial_state
                { vec![255; linear.to_usize()] }
                else { vec![0; linear.to_usize()] };

                Self {
                    data: $crate::collections::linear::bitpacked::BitPackedVector::new_uniform(linear, initial_state),
                    dimensions
                }
            }

            // TODO with iter

            // TODO getters, setters, also with slices, with checks, iterators
        }
    }
}

// These are the base spatial contiguous vectors, mirroring the base coordinate /
// dimension types defined in `crate::values::spatial::quantizable_index`.

create_spatial_quantized_contiguous_vector!(
    /// A 2D dense collection of quantized values addressable by [`QuantizedIndexCoord2D`].
    pub SpatialContiguousVector2D,
    QuantizedIndexCoord2D,
    QuantizedIndexDimension2D,
    2,
);

create_spatial_quantized_contiguous_vector!(
    /// A 3D dense collection of quantized values addressable by [`QuantizedIndexCoord3D`].
    pub SpatialContiguousVector3D,
    QuantizedIndexCoord3D,
    QuantizedIndexDimension3D,
    3,
);

create_spatial_quantized_contiguous_vector!(
    /// A 4D dense collection of quantized values addressable by [`QuantizedIndexCoord4D`].
    pub SpatialContiguousVector4D,
    QuantizedIndexCoord4D,
    QuantizedIndexDimension4D,
    4,
);
