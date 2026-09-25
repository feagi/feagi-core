use proc_macro::TokenStream;
use independent_builders::bit_struct_builder;


mod basis;
mod templates;
mod independent_builders;

#[cfg(feature = "request_response_macros")]
#[proc_macro]
pub fn template_request_category(input: TokenStream) -> TokenStream {
    templates::requests_responses::request_response_category_parser::request_response_category_parser(input)
}

#[cfg(feature = "request_response_macros")]
#[proc_macro]
pub fn template_request(input: TokenStream) -> TokenStream {
    templates::requests_responses::request_response_parser::request_response_parser(input)
}

/// Builds a bit field struct around a uint with one named bit field per flag.
///
/// # Example
///
/// ```ignore
/// use feagi_macros::bit_struct_builder;
///
/// bit_struct_builder! {
///     u8,
///     pub ExampleFlags,
///     #[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
///     /// Runtime visibility and enable flags.
///     {
///         enabled,
///         visible,
///     }
/// }
///
/// let mut flags = ExampleFlags::new(0);
/// flags.set_enabled(true);
/// assert!(flags.enabled());
/// flags.toggle_visible();
/// assert!(flags.visible());
/// assert_eq!(flags.bits(), 0b0000_0011);
/// ```
#[proc_macro]
pub fn bit_struct_builder(input: TokenStream) -> TokenStream {
    bit_struct_builder::bit_struct_builder(input)
}