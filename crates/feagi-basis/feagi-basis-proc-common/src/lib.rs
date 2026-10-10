use heck::ToPascalCase;
use quote::format_ident;
use feagi_macro_components::common::lit_str_to_ident;
use feagi_macro_components::template_parsing::feagi_request_messsages::FeagiMessageCategory;

//region Templates

//region Feagi Request Messaging

/// For a given category, parse a given Feagi Message Path definition with its Request
/// Parameters, Queryables, and Payload (if allowed), and its type.
#[proc_macro]
pub fn define_template_categorized_requests(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use feagi_macro_components::templates::TemplateRoot;
    TemplateRoot::<FeagiMessageCategory>::parse_template_and_generate_generator_input_macro(input.into()).into()
    
}

#[proc_macro]
pub fn from_template_make_request_message_structs(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use feagi_macro_components::templates::TemplateRoot;
    let res = TemplateRoot::<FeagiMessageCategory>::parse_generator_input_macro(input.into());
    match res {
        Err(e) => {
            e.to_compile_error().into()
        }
        Ok(s) => {
            s.generate_rust_structs_for_all_messages().into()
        }
    }
}

#[proc_macro]
pub fn from_template_make_request_message_categorized_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use feagi_macro_components::templates::TemplateRoot;
    let res = TemplateRoot::<FeagiMessageCategory>::parse_generator_input_macro(input.into());
    match res {
        Err(e) => {
            e.to_compile_error().into()
        }
        Ok(s) => {
            let names = s.create_categorized_rust_struct_names();
            let request_enum_name = format_ident!("{}Requests", s.category_name.value().to_pascal_case());
            let response_enum_name = format_ident!("{}Responses", s.category_name.value().to_pascal_case());
            
            names.generate_rust_request_response_enums_for_category(request_enum_name, response_enum_name).into()
        }
    }

}


//endregion

//endregion