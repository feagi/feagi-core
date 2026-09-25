//! Parses endpoints, request/response structs, and other metadata and validates them for a given
//! category

use proc_macro::TokenStream;
use syn::{parse_macro_input};

use crate::basis::{parse_optional_comma, StructBuilderParameters};
use crate::templates::requests_responses::basis_structs::TemplateRequestCategory;

pub fn request_response_category_parser(input: TokenStream) -> TokenStream {
    let request_category = parse_macro_input!(input as TemplateRequestCategory);

    // TODO validations

    TokenStream::from(request_category.expand_macro())
}


