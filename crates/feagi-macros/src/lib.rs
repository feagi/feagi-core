use proc_macro::TokenStream;

mod common;
mod templates;
mod bit_struct_builder;

#[cfg(feature = "request_response_macros")]
#[proc_macro]
pub fn template_request_category(input: TokenStream) -> TokenStream {
    templates::requests_and_responses::template_requests_category::template_requests_category(input)
}

#[cfg(feature = "request_response_macros")]
#[proc_macro]
pub fn template_request(input: TokenStream) -> TokenStream {
    templates::requests_and_responses::template_requests::template_requests(input)
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