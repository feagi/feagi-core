//! Generates `IntoPicoservePath` implementations, one per path-tuple arity.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Index;

const MAX_PATH_ELEMENTS: usize = 8;

/// Build one `IntoPicoservePath` impl for every arity in `1..=MAX_PATH_ELEMENTS`
pub fn path_tuple_impl_generator() -> proc_macro::TokenStream {
    let mut output = TokenStream::new();

    for arity in 1..=MAX_PATH_ELEMENTS {
        output.extend(into_picoserve_path_impl(arity));
    }

    output.into()
}


/// One `IntoPicoservePath` impl for a tuple of `arity` elements
fn into_picoserve_path_impl(arity: usize) -> TokenStream {
    let element_types: Vec<_> = (0..arity).map(|slot| format_ident!("E{}", slot)).collect();
    let indices: Vec<Index> = (0..arity).map(Index::from).collect();

    // Trailing commas keep single-element tuples as tuples rather than groupings.
    let input_tuple = quote!((#(#element_types,)*));
    let output_tuple = quote!((#(<#element_types as IntoPicoserveSegment>::Output,)*));
    let converted_elements = quote!((#(self.#indices.into_picoserve_segment(),)*));

    quote! {
        impl<#(#element_types: IntoPicoserveSegment,)*> IntoPicoservePath for #input_tuple {
            type Output = #output_tuple;

            #[inline]
            fn into_picoserve_path(self) -> Self::Output {
                #converted_elements
            }
        }
    }
}
