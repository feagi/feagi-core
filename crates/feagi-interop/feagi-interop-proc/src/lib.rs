mod open_api_category;
mod path_tuple_generator;

use feagi_macro_components::feagi_messages::request_response::feagi_request_message_category::FeagiRequestMessageCategory;
use feagi_macro_components::template_parsing::feagi_messages::request_response::category_grouping::GroupedRequestResponseCategoryTemplate;
use feagi_macro_components::templates::TemplateRoot;
use proc_macro2::TokenStream;
use quote::quote;

#[proc_macro]
/// Builds `IntoPicoservePath` impls, one per path-tuple arity.
pub fn generate_into_picoserve_path_impls(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    path_tuple_generator::path_tuple_impl_generator()
}

//region OpenAPI

/// Expands one category template into a const `CategoryApi` expression.
///
/// Invoke it through the category template macro:
/// `template_design_server_system_requests!(from_template_make_open_api_category_data)`.
/// `CategoryApi`, `OperationApi`, `Schema`, and the other OpenAPI types must be in scope.
#[proc_macro]
pub fn from_template_make_open_api_category_data(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match TemplateRoot::<FeagiRequestMessageCategory>::parse_generator_input_macro(input.into()) {
        Err(error) => error.to_compile_error().into(),
        Ok(category) => open_api_category::category_api_tokens(&category).into(),
    }
}

/// Expands a grouped category template into `generate_open_api_str`.
///
/// Each category is expanded through the template macro named beside it.
/// Those macros, `from_template_make_open_api_category_data`, `CategoryApi`, and
/// `OpenApiDocument` must be in scope at the call site.
#[proc_macro]
pub fn from_template_make_open_api_json_generator_function(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match TemplateRoot::<GroupedRequestResponseCategoryTemplate>::parse_generator_input_macro(input.into()) {
        Err(error) => error.to_compile_error().into(),
        Ok(group) => open_api_function_tokens(&group).into(),
    }
}

/// Const function tokens for one grouped category template.
fn open_api_function_tokens(group: &GroupedRequestResponseCategoryTemplate) -> TokenStream {
    let calls = group.generator.categories().iter().map(|category| {
        let macro_name = category.template_macro_name();
        quote! { #macro_name!(from_template_make_open_api_category_data) }
    });
    quote! {
        pub const fn generate_open_api_str() -> &'static str {
            const CATEGORIES: &[CategoryApi] = &[
                #( #calls, )*
            ];
            const DOCUMENT: OpenApiDocument = OpenApiDocument::from_categories(CATEGORIES);
            const JSON_LEN: usize = DOCUMENT.json_len();
            const JSON_BYTES: [u8; JSON_LEN] = DOCUMENT.write_json();
            OpenApiDocument::json_str(&JSON_BYTES)
        }
    }
}

//endregion
