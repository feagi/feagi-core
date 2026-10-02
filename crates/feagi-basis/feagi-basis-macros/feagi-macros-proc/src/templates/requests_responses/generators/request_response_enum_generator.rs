use quote::{format_ident, quote};
use syn::{LitStr, Type};
use heck::{ToSnakeCase, AsSnakeCase};
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
                let category_description = &category_request.description;

                let prefix_name = format!("{}{}", category_name, "Request");
                let combo_struct_name = prepend_to_lit_str(prefix_name.as_str(), &category_request.title);
                let combo_struct_name = lit_str_to_ident(&combo_struct_name).unwrap();
                let combo_path_request_struct = category_request.request
                    .make_combination_with(&category_request.path_parameters);
                if !combo_path_request_struct.is_empty() {
                    let combo_struct_tokens = combo_path_request_struct.generate_pub_struct(&combo_struct_name);
                    adding_stream.extend(quote! {
                    #[doc = #category_description]
                    #[derive(Debug, Clone, Default, ::serde::Serialize, ::serde::Deserialize)]
                    #combo_struct_tokens
                });
                    request_struct = Some(combo_struct_name);
                } else {
                    request_struct = None;
                }

                if !category_request.response.is_empty() {
                    let prefix_name = format!("{}{}", category_name, "Response");
                    let struct_name = prepend_to_lit_str(prefix_name.as_str(), &category_request.title);
                    let struct_name = lit_str_to_ident(&struct_name).unwrap();
                    let struct_tokens = category_request.response.generate_pub_struct(&struct_name);
                    adding_stream.extend(quote! {
                    #[doc = #category_description]
                    #[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
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
        let request_enum_name = lit_str_to_ident(&request_enum_name).unwrap();

        let response_enum_name = append_to_lit_str(
            &(&template.category_name), "ResponsesEnum");
        let response_enum_name = lit_str_to_ident(&response_enum_name).unwrap();

        let mut request_enum_tokens = proc_macro2::TokenStream::new();
        let mut response_enum_tokens = proc_macro2::TokenStream::new();

        // First variant overall, and the first variant whose tuple is empty.
        // `has_member` is true when that first variant carries a struct to default.
        let mut first_request_variant: Option<(syn::Ident, bool)> = None;
        let mut first_request_memberless: Option<syn::Ident> = None;
        let mut first_response_variant: Option<(syn::Ident, bool)> = None;
        let mut first_response_memberless: Option<syn::Ident> = None;

        for enum_variant in enum_variants {

            let variant = lit_str_to_ident(&enum_variant.key_name).unwrap(); // TODO error handling

            let request_has_member = enum_variant.request_struct.is_some();
            if first_request_variant.is_none() {
                first_request_variant = Some((variant.clone(), request_has_member));
            }
            if !request_has_member && first_request_memberless.is_none() {
                first_request_memberless = Some(variant.clone());
            }

            let response_has_member = enum_variant.response_struct.is_some();
            if first_response_variant.is_none() {
                first_response_variant = Some((variant.clone(), response_has_member));
            }
            if !response_has_member && first_response_memberless.is_none() {
                first_response_memberless = Some(variant.clone());
            }

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

        // thingbuf::recycling::DefaultRecycle requires Default. Prefer the first
        // variant with no members. If every variant has a member, use the first
        // variant and init that member with Default.
        output.extend(enum_default_impl(
            &request_enum_name,
            first_request_memberless.as_ref(),
            first_request_variant.as_ref().map(|(variant, has_member)| (variant, *has_member)),
        ));
        output.extend(enum_default_impl(
            &response_enum_name,
            first_response_memberless.as_ref(),
            first_response_variant.as_ref().map(|(variant, has_member)| (variant, *has_member)),
        ));

        // RequestResponseRecycle builds Result and Sender slots. The enum Default
        // impls supply the values those slots start from.


        // create the requester / responder creator function
        let category_name_snake = &template.category_name.value().to_snake_case();
        let req_res_func_name = format_ident!("{}_{}", "create_requester_responder", category_name_snake);
        let root_error_type = &template.feagi_error_type;
        let recycle = quote! {
            ::feagi_basis::thread_messaging::multi_request_channel::alloc_requester_processor::RequestResponseRecycle
        }; // TODO hacky

        output.extend(quote! {

            pub fn #req_res_func_name <const REQUEST_POOL_SIZE: usize, const ALLOW_BEYOND_POOL: bool>(request_queue_length: usize)
            -> (
                    PooledOneshotRequester<#request_enum_name, #response_enum_name, #recycle, #recycle, #root_error_type, REQUEST_POOL_SIZE, ALLOW_BEYOND_POOL>,
                    RequestResponder<#request_enum_name, #response_enum_name, #recycle, #recycle, #root_error_type>
            ) {
                create_requester_and_responder(request_queue_length, <#recycle>::new(), <#recycle>::new())
            }
        });


        output

    }
}

/// Builds `impl Default` for a generated request or response enum.
fn enum_default_impl(
    enum_name: &syn::Ident,
    first_memberless_variant: Option<&syn::Ident>,
    first_variant: Option<(&syn::Ident, bool)>,
) -> proc_macro2::TokenStream {
    let Some((variant, initialize_member)) = first_memberless_variant
        .map(|variant| (variant, false))
        .or_else(|| first_variant.map(|(variant, has_member)| (variant, has_member)))
    else {
        return proc_macro2::TokenStream::new();
    };

    let constructor = if initialize_member {
        quote! { Self::#variant(Default::default()) }
    } else {
        quote! { Self::#variant() }
    };

    quote! {
        impl Default for #enum_name {
            fn default() -> Self {
                #constructor
            }
        }
    }
}