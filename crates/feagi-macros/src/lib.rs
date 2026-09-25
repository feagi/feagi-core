use independent_builders::bit_struct_builder;
use crate::basis::GeneratorFromTemplate;
use crate::templates::requests_responses::generators::request_response_ohkami_server_generator::OhkamiServerGenerator;
use crate::templates::requests_responses::requests_responses_structs::{TemplateRequestCategory};
use crate::templates::template_root::TemplateRoot;

mod basis;
mod templates;
mod independent_builders;

#[cfg(feature = "request_response_template_parsers")]
#[proc_macro]
pub fn template_request_category(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    TemplateRoot::<TemplateRequestCategory>::parse_template_and_generate_generator_input_macro(input)
}

#[cfg(feature = "request_response_ahkami_codegen")]
#[proc_macro]
pub fn generate_from_template_ohkami_rest_server(generator_input_macro: proc_macro::TokenStream) -> proc_macro::TokenStream {
    OhkamiServerGenerator::read_template_and_generate_code(generator_input_macro)
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
pub fn bit_struct_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    bit_struct_builder::bit_struct_builder(input)
}