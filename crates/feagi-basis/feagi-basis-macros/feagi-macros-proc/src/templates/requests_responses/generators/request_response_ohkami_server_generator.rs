
use quote::{format_ident, quote};
use syn::LitStr;
use crate::basis::{lit_str_to_ident, prepend_to_lit_str, GeneratorFromTemplate, StructParameters};
use crate::templates::requests_responses::requests_responses_structs::TemplateRequestResponseCategory;

pub struct OhkamiServerGenerator;

impl GeneratorFromTemplate<TemplateRequestResponseCategory> for OhkamiServerGenerator {
    fn generate_code_from_template(template: TemplateRequestResponseCategory) -> proc_macro2::TokenStream {


        let mut output = proc_macro2::TokenStream::new();

        fn process_parameters_to_rest_struct(parameters: &StructParameters,
                                             prefix_name: &'static str,
                                             title: &LitStr,
                                             category_desc: &LitStr,
                                             output: &mut proc_macro2::TokenStream
        ) -> bool {
            if parameters.is_empty() {
                return false
            }

            let struct_name = prepend_to_lit_str(prefix_name, title);
            let struct_name = lit_str_to_ident(&struct_name).unwrap(); // TODO error handling
            let struct_tokens = parameters.generate_priv_struct(&struct_name);

            output.extend(quote!{
                    #[doc = #category_desc]
                    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, openapi::Schema)]
                    #struct_tokens
            });

            return true
        }



        // NOTE: GET requests are different as they have no payload


        for get_category in template.read {
            let category_class: &'static str = "Read";
            let base_category_definition = &get_category.description;

            
            let path_parameters_present = process_parameters_to_rest_struct(
                &get_category.path_parameters,
                "GETPathParameter",
                &get_category.title,
                &get_category.description,
                &mut output
            );

            let request_present = process_parameters_to_rest_struct(
                &get_category.request,
                "GETRequest",
                &get_category.title,
                &get_category.description,
                &mut output
            );

            // Request path parameters / query -> request struct
            match (path_parameters_present, request_present) {
                (false, false) => {
                    // Nothing
                    // theres nothing to convert to or from!
                }
                (true, false) => {
                    // only path parameters

                    output.extend(quote!{

                        impl Into<>

                    })
                }
                (false, true) => {
                    // only query parameters
                }
                (true, true) => {
                    // both path and query parameters
                }
            }


            _ = process_parameters_to_rest_struct(
                &get_category.response,
                "GETResponse",
                &get_category.title,
                &get_category.description,
                &mut output
            );



        }




        /*
        let category_name = template.category_name;
        let base_path = template.base_path;

        for get_request in &template.read {
            // GET Request structs and impls

            let base_struct_title = &get_request.title.value();



            // Query Struct (end query parameters of URL. Private, used as a parsing inbetween
            let query_struct_title = format_ident!("Query{}", base_struct_title);
            let query_struct_tokens = get_request.path_parameters.generate_struct(query_struct_title);
            output.extend(
                quote! {
                    #[derive(Deserialize, openapi::Schema)]
                    #query_struct_tokens
                }
            );

            // Request Struct (GET doesnt use data payloads for requests, so this actually is broken
            // into the path parameters and the query parameters)
            let request_struct_title = format_ident!("Request{}", base_struct_title);
            let request_struct_tokens = get_request.request.generate_struct(query_struct_title);
            output.extend(
                quote! {

                    
                }
            );






            let response_struct_title = format_ident!("Response{}", base_struct_title);




            if !get_request.request.is_empty() {





                output.extend(quote! {

                    #[derive(openapi::Schema)]
                    struct #struct_title {

                    }
                })
            }


        }



        quote!{



            // Request path Struct

            // Request Struct

            // Response Struct



        }

         */
        output
    }
}