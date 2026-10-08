use feagi_macro_components::templates::TemplateRoot;

/// For a given category, parse a given Feagi Message Path definition with its Request
/// Parameters, Queryables, and Payload (if allowed), and its type.
#[proc_macro]
pub fn regenerate_template_pathing_request_response(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use feagi_macro_components::templates::TemplateRoot;
    TemplateRoot::<>::parse_template_and_generate_generator_input_macro(input.into()).into()
    
}