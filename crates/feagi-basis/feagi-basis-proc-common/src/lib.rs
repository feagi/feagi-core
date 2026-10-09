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


//endregion

//endregion