use feagi_macro_components::feagi_messages::request_response::feagi_request_message_category::FeagiRequestMessageCategory;
use feagi_macro_components::template_parsing::feagi_messages::request_response::category_grouping::GroupedRequestResponseCategoryTemplate;
//region Templates

//region Feagi Request Messaging

/// Defines a category of request messages. Generates, for every message, its path, request
/// specifications, response, error, message struct, and descriptor. Then for the category its
/// configurables struct, request and response enums, and the category marker implementing
/// `FeagiRequestCategory`
#[proc_macro]
pub fn define_request_message_category(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match syn::parse::<FeagiRequestMessageCategory>(input) {
        Err(e) => e.to_compile_error().into(),
        Ok(category) => category.generate_rust_category().into(),
    }
}

//endregion

//region Feagi Request Messaging Categorization

/// Define all the categories of requests
#[proc_macro]
pub fn define_template_request_categories(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use feagi_macro_components::templates::TemplateRoot;
    TemplateRoot::<GroupedRequestResponseCategoryTemplate>::parse_template_and_generate_generator_input_macro(input.into()).into()
}

#[proc_macro]
pub fn from_template_define_generic_struct_of_category_fields(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use feagi_macro_components::templates::TemplateRoot;
    let res = TemplateRoot::<GroupedRequestResponseCategoryTemplate>::parse_generator_input_macro(input.into());
    match res {
        Err(e) => e.to_compile_error().into(),
        Ok(s) => s.generate_generic_with_field_name_categories_and_of_type().into(),
    }
}

//endregion

//endregion
