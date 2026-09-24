use proc_macro::TokenStream;

mod common;
mod templates;
pub mod bit_struct_builder;

#[proc_macro]
pub fn template_request_category(input: TokenStream) -> TokenStream {
    templates::requests_and_responses::template_requests_category::template_requests_category(input)
}

#[proc_macro]
pub fn template_request(input: TokenStream) -> TokenStream {
    templates::requests_and_responses::template_requests::template_requests(input)
}