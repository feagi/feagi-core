


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