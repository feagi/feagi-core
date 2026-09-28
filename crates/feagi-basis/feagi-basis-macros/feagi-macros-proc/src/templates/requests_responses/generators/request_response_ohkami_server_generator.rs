
use quote::{format_ident, quote};
use crate::basis::{lit_str_to_ident, GeneratorFromTemplate};
use crate::templates::requests_responses::requests_responses_structs::TemplateRequestResponseCategory;

pub struct OhkamiServerGenerator;

impl GeneratorFromTemplate<TemplateRequestResponseCategory> for OhkamiServerGenerator {
    fn generate_code_from_template(template: TemplateRequestResponseCategory) -> proc_macro2::TokenStream {

        let mut output = proc_macro2::TokenStream::new();

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
            )






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
    }
}