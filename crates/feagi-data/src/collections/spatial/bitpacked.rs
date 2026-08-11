macro_rules! create_spatial_bitpacked_vector {
    (
        $(#[$meta:meta])*
        $vis:vis $struct_name:ident,
        $coord_impl:ident,
        $dim_impl:ident,
        $num_dimensions:expr,
    ) => {
        $(#[$meta])*
        $vis struct $struct_name<QI: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> {
            pub data: $crate::collections::linear::bitpacked::BitPackedVector<QI>,
            pub dimensions: $dim_impl<QI>
        }

        impl<QI: $crate::values::quantizable::QuantizedUnsignedIntegerTrait> $struct_name<QI>
        {
            pub fn new_uniform(dimensions: $dim_impl<QI>, initial_state: bool) -> $struct_name<QI> {
                let linear = dimensions.number_contained_elements().deref();

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
