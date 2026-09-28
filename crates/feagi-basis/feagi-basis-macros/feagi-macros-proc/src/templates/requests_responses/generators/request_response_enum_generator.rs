use quote::{format_ident, quote};
use syn::{LitStr, Type};
use crate::basis::{append_to_lit_str, lit_str_to_ident, prepend_to_lit_str, GeneratorFromTemplate};
use crate::templates::requests_responses::requests_responses_structs::{RequestResponseContract, TemplateRequestResponseCategory};


pub struct EnumRequestResponseGenerator;

impl GeneratorFromTemplate<TemplateRequestResponseCategory> for EnumRequestResponseGenerator {
    fn generate_code_from_template(template: TemplateRequestResponseCategory) -> proc_macro2::TokenStream {

        struct EnumVariants {
            key_name: LitStr,
            request_struct: Option<syn::Ident>,
            response_struct: Option<syn::Ident>,
        };

        /// builds the structs and enum variants for the given request / responses (if relevant)
        fn process_template_types(category_name: &'static str,
                                  categories: &Vec<RequestResponseContract>,
                                  adding_stream: &mut proc_macro2::TokenStream,
                                  enum_vars: &mut Vec<EnumVariants>) {
            for category_request in categories {

                let request_struct: Option<syn::Ident>;
                let response_struct: Option<syn::Ident>;
                let struct_description = &category_request.description;

                let prefix_name = format!("{}{}", category_name, "Request");
                let combo_struct_name = prepend_to_lit_str(prefix_name.as_str(), &category_request.title);
                let combo_struct_name = lit_str_to_ident(&combo_struct_name).unwrap(); // TODO error handling
                let combo_path_request_struct = category_request.request
                    .make_combination_with(&category_request.path_parameters);
                if !combo_path_request_struct.is_empty() {
                    let combo_struct_tokens = combo_path_request_struct.generate_struct(&combo_struct_name);
                    adding_stream.extend(quote! {
                    #[doc = #struct_description]
                    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
                    #combo_struct_tokens
                });
                    request_struct = Some(combo_struct_name);
                } else {
                    request_struct = None;
                }

                if !category_request.response.is_empty() {
                    let prefix_name = format!("{}{}", category_name, "Response");
                    let struct_name = prepend_to_lit_str(prefix_name.as_str(), &category_request.title);
                    let struct_name = lit_str_to_ident(&struct_name).unwrap(); // TODO error handling
                    let struct_tokens = category_request.response.generate_struct(&struct_name);
                    adding_stream.extend(quote! {
                    #[doc = #struct_description]
                    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
                    #struct_tokens
                });
                    response_struct = Some(struct_name);
                } else {
                    response_struct = None;
                }

                enum_vars.push(
                    EnumVariants {
                        key_name: category_request.title.clone(),
                        request_struct,
                        response_struct,
                    }
                )

            };
        }


        let mut output = proc_macro2::TokenStream::new();
        let mut enum_variants: Vec<EnumVariants> = Vec::new();

        process_template_types("Read", &template.read, &mut output, &mut enum_variants);
        process_template_types("Create", &template.create, &mut output, &mut enum_variants);
        process_template_types("Edit", &template.edit, &mut output, &mut enum_variants);
        process_template_types("Delete", &template.delete, &mut output, &mut enum_variants);
        process_template_types("Patch", &template.patch, &mut output, &mut enum_variants);



        // Process into a request enum and response enum
        
        let request_enum_name = append_to_lit_str(
            &(&template.category_name), "RequestsEnum");
        let request_enum_name = lit_str_to_ident(&request_enum_name).unwrap(); // TODO error handling

        let response_enum_name = append_to_lit_str(
            &(&template.category_name), "ResponsesEnum");
        let response_enum_name = lit_str_to_ident(&response_enum_name).unwrap(); // TODO error handling
        
        let mut request_enum_tokens = proc_macro2::TokenStream::new();
        let mut response_enum_tokens = proc_macro2::TokenStream::new();

        for enum_variant in enum_variants {

            let variant = lit_str_to_ident(&enum_variant.key_name).unwrap(); // TODO error handling

            if let Some(request) = &enum_variant.request_struct {
                request_enum_tokens.extend(quote!{#variant(#request),
                })
            } else {
                request_enum_tokens.extend(quote!{#variant(),
                })
            }

            if let Some(response) = &enum_variant.response_struct {
                response_enum_tokens.extend(quote!{#variant(#response),})
            } else {
                response_enum_tokens.extend(quote!{#variant(),})
            }

        };

        let category_description = &template.category_description;

        output.extend(quote! {

            #[doc = #category_description]
            #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
            pub enum #request_enum_name {
                #request_enum_tokens
            }

            #[doc = #category_description]
            #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
            pub enum #response_enum_name {
                #response_enum_tokens
            }
        });

        output

    }
}