//! Combine multiple template_request_category's into a template_request

use proc_macro::TokenStream;
use syn::{parse_macro_input};
use crate::templates::requests_responses::basis_structs::TemplateRequest;

pub fn request_response_parser(input: TokenStream) -> TokenStream {
    let request = parse_macro_input!(input as TemplateRequest);

    // TODO validations

    TokenStream::from(request.expand_macro())
}