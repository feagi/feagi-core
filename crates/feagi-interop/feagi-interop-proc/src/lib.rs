
use feagi_macro_components::template_parsing::feagi_messages::request_response::feagi_request_messsages::FeagiMessageCategory;
use heck::ToPascalCase;
use quote::{format_ident, quote};

//region Grouped Categorized Request Messages
// This is defined in the macro-components crate, we just have some specific uses here




#[proc_macro]
pub fn from_template_make_open_api_json_generator_function(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    use feagi_macro_components::templates::TemplateRoot;
    use feagi_macro_components::template_parsing::feagi_messages::request_response::category_grouping::GroupedRequestResponseCategoryTemplate;
    let res = TemplateRoot::<GroupedRequestResponseCategoryTemplate>::parse_generator_input_macro(input.into());
    match res {
        Err(e) => {
            e.to_compile_error().into()
        }
        Ok(s) => {

            // NOTE: assume the openapi generator struct has imported locally, as is all else.
            // Do not be concerned with pathing imports


            quote! {

                pub const fn generate_open_api_str() -> &'static str {

                    // Steps:
                    // iterate over s, this function should be able to reference the macros by name of each category , that within this function can unroll to become needed information.
                    // It may be required to have to write addiotonal macros that unroll those tempaltes to produce struct data that is needed here

                    // this function then should parse those structs, feeding them into the open_api struct
                    // the struct may need to be modified to work in a const context

                    // The open_api struct should then output the const str, which is emitted here


                }



            }


        }
    }


}



//endregion