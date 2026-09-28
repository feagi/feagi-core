use quote::{format_ident, quote};
use syn::{LitStr, Type};
use crate::basis::{append_to_lit_str, lit_str_to_ident, prepend_to_lit_str, GeneratorFromTemplate};
use crate::templates::requests_responses::requests_responses_structs::TemplateRequestResponseCategory;


pub struct EnumRequestResponseGenerator;

impl GeneratorFromTemplate<TemplateRequestResponseCategory> for EnumRequestResponseGenerator {
    fn generate_code_from_template(template: TemplateRequestResponseCategory) -> proc_macro2::TokenStream {

        struct EnumVariants {
            key_name: LitStr,
            request_struct: Option<syn::Ident>,
            response_struct: Option<syn::Ident>,
        };

        let mut output = proc_macro2::TokenStream::new();
        let mut enum_variants: Vec<EnumVariants> = Vec::new();

        for read_request in &template.read {

            let request_struct: Option<syn::Ident>;
            let response_struct: Option<syn::Ident>;

            let combo_struct_name = prepend_to_lit_str("ReadRequest", &read_request.title);
            let combo_struct_name = lit_str_to_ident(&combo_struct_name).unwrap(); // TODO error handling
            let combo_path_request_struct = read_request.request
                .make_combination_with(&read_request.path_parameters);
            if !combo_path_request_struct.is_empty() {
                let combo_struct_tokens = combo_path_request_struct.generate_struct(&combo_struct_name);
                output.extend(quote! {
                    #[derive(Debug, Clone, serde::Serialize)]
                    #combo_struct_tokens
                });
                request_struct = Some(combo_struct_name);
            } else {
                request_struct = None;
            }

            if !read_request.response.is_empty() {
                let struct_name = prepend_to_lit_str("ReadResponse", &read_request.title);
                let struct_name = lit_str_to_ident(&struct_name).unwrap(); // TODO error handling
                let struct_tokens = read_request.response.generate_struct(&struct_name);
                output.extend(quote! {
                    #[derive(Debug, Clone, serde::Serialize)]
                    #struct_tokens
                });
                response_struct = Some(struct_name);
            } else {
                response_struct = None;
            }

            enum_variants.push(
                EnumVariants {
                    key_name: read_request.title.clone(),
                    request_struct,
                    response_struct,
                }
            )

        };

        // TODO other types

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
        
        output.extend(quote! {
            
            pub enum #request_enum_name {
                #request_enum_tokens
            }
            
            pub enum #response_enum_name {
                #response_enum_tokens
            }
        });

        output

    }
}