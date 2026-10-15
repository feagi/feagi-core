use feagi_macro_components::feagi_messages::request_response::feagi_request_message_category::FeagiRequestMessageCategory;
use feagi_macro_components::template_parsing::feagi_messages::request_response::category_grouping::GroupedRequestResponseCategoryTemplate;
//region Templates

//region Feagi Request Messaging

/// For a given category, parse a given Feagi Message Path definition with its Request
/// Parameters, Queryables, and Payload (if allowed), and its type.
#[proc_macro]
pub fn define_template_categorized_requests(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use feagi_macro_components::templates::TemplateRoot;
    TemplateRoot::<FeagiRequestMessageCategory>::parse_template_and_generate_generator_input_macro(input.into()).into()
}

#[proc_macro]
pub fn from_template_make_request_message_structs(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use feagi_macro_components::templates::TemplateRoot;
    let res = TemplateRoot::<FeagiRequestMessageCategory>::parse_generator_input_macro(input.into());
    match res {
        Err(e) => {
            e.to_compile_error().into()
        }
        Ok(s) => {
            s.generate_rust_specification_structs_for_all_messages().into()
        }
    }
}

#[proc_macro]
pub fn from_template_make_request_message_categorized_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use feagi_macro_components::templates::TemplateRoot;
    let res = TemplateRoot::<FeagiRequestMessageCategory>::parse_generator_input_macro(input.into());
    match res {
        Err(e) => {
            e.to_compile_error().into()
        }
        Ok(s) => {
            s.generate_rust_request_response_enums_for_category().into()
        }
    }
}

//endregion

//region Feagi Request Messaging Categorization

/// Define all the categories of requests
#[proc_macro]
pub fn define_template_request_categories(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use feagi_macro_components::template_parsing::feagi_messages::request_response::category_grouping::GroupedRequestResponseCategoryTemplate;
    use feagi_macro_components::templates::TemplateRoot;
    TemplateRoot::<GroupedRequestResponseCategoryTemplate>::parse_template_and_generate_generator_input_macro(input.into()).into()
}

#[proc_macro]
pub fn from_template_define_generic_struct_of_category_fields(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use feagi_macro_components::template_parsing::feagi_messages::request_response::category_grouping::GroupedRequestResponseCategoryTemplate;
    use feagi_macro_components::templates::TemplateRoot;
    let res = TemplateRoot::<GroupedRequestResponseCategoryTemplate>::parse_generator_input_macro(input.into());
    match res {
        Err(e) => {
            e.to_compile_error().into()
        }
        Ok(s) => {
            s.generate_generic_with_field_name_categories_and_of_type().into()
        }
    }
}


//endregion

//endregion