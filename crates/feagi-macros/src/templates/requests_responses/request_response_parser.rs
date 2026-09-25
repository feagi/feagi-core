//! Combine multiple template_request_category's into a template_request

use proc_macro::TokenStream;
use syn::{parse_macro_input, Token};
use syn::parse::Parser;
use crate::templates::requests_responses::basis_structs::CompleteRequestResponsesTemplate;



pub fn request_response_parser(input: TokenStream) -> TokenStream {
    


    let request = parse_macro_input!(input as CompleteRequestResponsesTemplate);

    // TODO validations

    TokenStream::from(request.expand_macro())
}