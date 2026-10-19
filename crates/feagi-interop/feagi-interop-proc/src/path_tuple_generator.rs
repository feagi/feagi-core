//! Generates `FeagiMessagePathTupleConverter` implementations for mixed path tuples

use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use syn::Index;

/// Builds `FeagiMessagePathTupleConverter` impls for tuples of &str and generic parameters
pub fn path_tuple_impl_generator() -> proc_macro::TokenStream {
    let mut output = TokenStream::new();

    // How many path elements to allow. As the growth is rapid, do not put too high!
    const MAX_DEPTH: usize = 7;

    for number_path_elements in 0..MAX_DEPTH {
        for number_parameters in 0..number_path_elements {
            // Bit i set means slot i is a generic parameter. Clear means `&str`.
            let combination_count = 1_usize << number_path_elements;
            for mask in 0..combination_count {
                if mask.count_ones() as usize != number_parameters {
                    continue;
                }
                output.extend(built_path_tuple_impl(number_path_elements, mask));
            }
        }
    }

    output.into()
}

/// One `FeagiMessagePathTupleConverter` impl for a single placement of generic parameters.
fn built_path_tuple_impl(number_path_elements: usize, parameter_mask: usize) -> TokenStream {
    let mut parameter_types: Vec<Ident> = Vec::new();
    let mut input_elements: Vec<TokenStream> = Vec::with_capacity(number_path_elements);
    let mut output_picoserve_elements: Vec<TokenStream> = Vec::with_capacity(number_path_elements);
    let mut output_picoserve_values: Vec<TokenStream> = Vec::with_capacity(number_path_elements);

    for slot in 0..number_path_elements {
        let slot_is_parameter = (parameter_mask & (1_usize << slot)) != 0;
        if slot_is_parameter {
            let parameter_type = format_ident!("T{}", parameter_types.len() + 1);
            input_elements.push(quote!(#parameter_type));
            output_picoserve_elements.push(quote!(ParsePathSegment<#parameter_type>));
            output_picoserve_values.push(quote!(parse_path_segment::<#parameter_type>()));
            parameter_types.push(parameter_type);
        } else {
            let index = Index::from(slot);
            input_elements.push(quote!(&'a str));
            output_picoserve_elements.push(quote!(&'a str));
            output_picoserve_values.push(quote!(input.#index));
        }
    }

    // Trailing comma keeps a one-element tuple a tuple.
    let tuple_in_type = quote!((#(#input_elements,)*));
    let tuple_picoserve_type = quote!((#(#output_picoserve_elements,)*));
    let output_value = quote!((#(#output_picoserve_values,)*));

    // `parse_path_segment` requires `FromStr`.
    let generics = if parameter_types.is_empty() {
        quote!('a)
    } else {
        quote!('a, #(#parameter_types: FromStr),*)
    };

    // TODO other types

    quote! {
        impl<#generics> FeagiMessagePathTupleConverter<#tuple_in_type> {
            #[cfg(feature = "picoserve")]
            pub fn as_path_parameters(input: #tuple_in_type) -> #tuple_picoserve_type {
                use picoserve::routing::ParsePathSegment;
                use picoserve::routing::parse_path_segment;

                #output_value
            }
        }
    }
}
