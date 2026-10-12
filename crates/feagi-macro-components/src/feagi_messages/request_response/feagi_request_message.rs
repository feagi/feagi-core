use quote::{format_ident, quote};
use syn::{braced, LitStr};
use syn::parse::{Parse, ParseStream};
use crate::common::{parse_property_colon_member, unit_type, RustVisibility};
use crate::feagi_messages::request_response::common_structs::{FeagiRequestMessageMethod, FeagiRequestMessageU8};
use crate::feagi_messages::request_response::feagi_request_message_path::FeagiRequestMessagePath;
use crate::template_parsing::property_descriptors::PropertyDescriptors;
use crate::template_parsing::struct_template::StructTemplate;
use crate::templates::Unparse;

mod kw {
    syn::custom_keyword!(path);
    syn::custom_keyword!(path_item_descriptions);
    syn::custom_keyword!(description);
    syn::custom_keyword!(payload);
    syn::custom_keyword!(response);
}

//region Feagi Request Message Specification Struct Names

/// Contains all the names of the rust structs that would be generated from a FeagiMessage
#[derive(Clone)]
pub struct FeagiRequestMessageRustStructNames {
    pub base_name: syn::Ident,
    pub method: FeagiRequestMessageMethod,
    pub is_parameter_struct_generated: bool,
    pub is_queryable_struct_generated: bool,
    pub is_payload_struct_generated: bool,
    pub is_response_struct_generated: bool,
}

impl FeagiRequestMessageRustStructNames {

    pub fn feagi_message_response_struct_name(&self) -> syn::Ident {
        self.with_prefix_method_and_given_suffix("FeagiRequestMessage")
    }
    
    pub fn feagi_message_response_path_struct_name(&self) -> syn::Ident {
        self.with_prefix_method_and_given_suffix("FeagiRequestMessagePath")
    }
    
    pub fn feagi_message_parameter_specification_struct_name(&self) -> Option<syn::Ident> {
        if self.is_parameter_struct_generated {
            Some(self.with_prefix_method_and_given_suffix("FeagiRequestMessageParameters"))
        } else { None }
    }

    pub fn feagi_message_queryable_specification_struct_name(&self) -> Option<syn::Ident> {
        if self.is_queryable_struct_generated {
            Some(self.with_prefix_method_and_given_suffix("FeagiRequestMessageQueryables"))
        } else { None }
    }

    pub fn feagi_message_payload_specification_struct_name(&self) -> Option<syn::Ident> {
        if self.is_payload_struct_generated {
            Some(self.with_prefix_method_and_given_suffix("FeagiRequestMessagePayload"))
        } else { None }
    }

    pub fn feagi_message_response_specification_struct_name(&self) -> Option<syn::Ident> {
        if self.is_response_struct_generated {
            Some(self.with_prefix_method_and_given_suffix("FeagiRequestMessageResponse"))
        } else { None }
    }
    
    fn with_prefix_method_and_given_suffix(&self, suffix: &str) -> syn::Ident {
        format_ident!("{}{}{}", self.method.as_str() , self.base_name, suffix)
    }
}

//endregion

//region Feagi Request Message

/// Parses over a FeagiMessage with a payload. Formatted as the following
/// "MessageTitle": {
///     path: `FeagiMessagePath`,
///     path_item_descriptions: {
///         property_name: "Description String",
///     },
///     description: "Description String",
///     payload: {
///         property_name: Type, "optional_comment",
///     },
///     response: {
///         property_name: Type, "optional_comment",
///     }
/// }
/// Note that the payload is skipped for Read Messages
pub struct FeagiRequestMessage<const METHOD_TYPE_U8: FeagiRequestMessageU8> {
    pub path: FeagiRequestMessagePath<METHOD_TYPE_U8>,
    pub path_item_descriptions: PropertyDescriptors,
    pub description: LitStr,
    pub payload: StructTemplate,
    pub response: StructTemplate,
}

impl<const METHOD_TYPE_U8: FeagiRequestMessageU8> FeagiRequestMessage<METHOD_TYPE_U8> {

    /// Struct names this message will generate.
    pub fn calculate_feagi_message_struct_names(&self, name_base: syn::Ident) -> FeagiRequestMessageRustStructNames {
        let named = |method_prefix: &str, suffix: &str| format_ident!("{}{}{}", method_prefix, name_base, suffix);
        let present_name = |template: &StructTemplate, method_prefix: &str, suffix: &str| {
            if template.is_empty() {
                None
            } else {
                Some(named(method_prefix, suffix))
            }
        };

        let parameters = self.path.as_parameter_struct_template();
        let queryables = self.path.as_queryable_struct_template();

        let method_str = FeagiRequestMessageMethod::from_u8(METHOD_TYPE_U8).as_str();

        FeagiRequestMessageRustStructNames {
            base_name: name_base,
            method: FeagiRequestMessageMethod::from_u8(METHOD_TYPE_U8),
            is_parameter_struct_generated: !parameters.is_empty(),
            is_queryable_struct_generated: !queryables.is_empty(),
            is_payload_struct_generated: !self.payload.is_empty(),
            is_response_struct_generated: !self.response.is_empty(),
        }
    }

    /// Generate the Specification structs, the feagi request message struct, and the response struct (if valid)
    pub fn generate_rust_specification_and_request_and_response_structs(&self, feagi_message_specification_struct_names: FeagiRequestMessageRustStructNames) -> proc_macro2::TokenStream {
        let mut output: proc_macro2::TokenStream = proc_macro2::TokenStream::new();
        let specification_derives = vec![
            format_ident!("{}", "Clone"),
            format_ident!("{}", "Debug"),
            format_ident!("{}", "Serialize"),
            format_ident!("{}", "Deserialize"),
        ];

        let tokens_message_path = self
            .path
            .generate_feagi_message_path_struct_and_impls(
                &feagi_message_specification_struct_names.feagi_message_response_path_struct_name(),
                &feagi_message_specification_struct_names.feagi_message_response_struct_name()); // TODO add root route
        output.extend(tokens_message_path);

        let mut all_request_fields = StructTemplate::new_empty();

        /// Names were already prefixed with the method in `generate_feagi_message_struct_names`.
        fn generated_name(name: &Option<syn::Ident>, role: &str) -> syn::Ident {
            name.clone().unwrap_or_else(|| panic!("non-empty {role} template has a struct name"))
        }

        let (parameter_type, parameter_message_field_constructor, parameter_value) = {
            // Struct is parsed from path itself
            let mut struct_template = self.path.as_parameter_struct_template();
            if struct_template.is_empty() {
                (unit_type(), proc_macro2::TokenStream::new(), quote!(()))
            } else {
                struct_template.overwrite_descriptions_from_property_descriptors(&self.path_item_descriptions);

                let struct_name = generated_name(&feagi_message_specification_struct_names.feagi_message_parameter_specification_struct_name(), "parameter");

                // Add fields to total field vector, field_name_tokens
                let mut field_name_tokens = proc_macro2::TokenStream::new();
                let mut deconstructor_tokens = proc_macro2::TokenStream::new();
                for (field_name, (field_type, maybe_field_docs)) in struct_template.get_map() {
                    all_request_fields.insert_property(field_name.clone(), field_type.clone(), maybe_field_docs.clone());
                    field_name_tokens.extend(quote! {
                        #field_name: parameters.#field_name,
                    });
                    deconstructor_tokens.extend(quote! {
                        #field_name: self.#field_name,
                    })
                }

                // Add Struct
                output.extend(struct_template.generate_rust_struct(struct_name.clone(), None, &specification_derives, RustVisibility::Public));

                // Add Trait impl
                output.extend(quote! {
                    impl FeagiMessageParameters for #struct_name {}
                });

                let value = quote! { Self::Parameters { #deconstructor_tokens } };
                (syn::parse_quote!(#struct_name), field_name_tokens, value)
            }
        };

        let (queryable_type, queryable_message_field_constructor, queryable_value) = {
            // Struct is parsed from path itself
            let mut struct_template = self.path.as_queryable_struct_template();
            if struct_template.is_empty() {
                (unit_type(), proc_macro2::TokenStream::new(), quote!(()))
            } else {
                struct_template.overwrite_descriptions_from_property_descriptors(&self.path_item_descriptions);

                let struct_name = generated_name(&feagi_message_specification_struct_names.feagi_message_queryable_specification_struct_name(), "queryable");

                // Add fields to total field vector, field_name_tokens
                let mut field_name_tokens = proc_macro2::TokenStream::new();
                let mut deconstructor_tokens = proc_macro2::TokenStream::new();
                for (field_name, (field_type, maybe_field_docs)) in struct_template.get_map() {
                    all_request_fields.insert_property(field_name.clone(), field_type.clone(), maybe_field_docs.clone());
                    field_name_tokens.extend(quote! {
                        #field_name: queryables.#field_name,
                    });
                    deconstructor_tokens.extend(quote! {
                        #field_name: self.#field_name,
                    })
                }

                // Add Struct
                output.extend(struct_template.generate_rust_struct(struct_name.clone(), None, &specification_derives, RustVisibility::Public));

                // Add Trait impl
                output.extend(quote! {
                    impl FeagiMessageQueryables for #struct_name {}
                });

                let value = quote! { Self::Queryables { #deconstructor_tokens } };
                (syn::parse_quote!(#struct_name), field_name_tokens, value)
            }
        };

        // NOTE: since we do validation that theres no payload with the READ METHOD, we can always trust
        // that this will return empty / NULL in those cases

        let (payload_type, payload_message_field_constructor, payload_value) = {
            // Struct is seperate
            let mut struct_template = self.payload.clone();
            if struct_template.is_empty() {
                (unit_type(), proc_macro2::TokenStream::new(), quote!(()))
            } else {
                struct_template.overwrite_descriptions_from_property_descriptors(&self.path_item_descriptions);

                let struct_name = generated_name(&feagi_message_specification_struct_names.feagi_message_payload_specification_struct_name(), "payload");

                // Add fields to total field vector, field_name_tokens
                let mut field_name_tokens = proc_macro2::TokenStream::new();
                let mut deconstructor_tokens = proc_macro2::TokenStream::new();
                for (field_name, (field_type, maybe_field_docs)) in struct_template.get_map() {
                    all_request_fields.insert_property(field_name.clone(), field_type.clone(), maybe_field_docs.clone());
                    field_name_tokens.extend(quote! {
                        #field_name: payload.#field_name,
                    });
                    deconstructor_tokens.extend(quote! {
                        #field_name: self.#field_name,
                    })
                }

                // Add Struct
                output.extend(struct_template.generate_rust_struct(struct_name.clone(), None, &specification_derives, RustVisibility::Public));

                // Add Trait impl
                output.extend(quote! {
                    impl FeagiMessagePayload for #struct_name {}
                });

                let value = quote! { Self::Payload { #deconstructor_tokens } };
                (syn::parse_quote!(#struct_name), field_name_tokens, value)
            }
        };

        let response_type = {
            // Struct is separate. Response fields stay off the request struct.
            let mut struct_template = self.response.clone();
            if struct_template.is_empty() {
                unit_type()
            } else {
                struct_template.overwrite_descriptions_from_property_descriptors(&self.path_item_descriptions);

                let struct_name = generated_name(&feagi_message_specification_struct_names.feagi_message_response_specification_struct_name(), "response");

                output.extend(struct_template.generate_rust_struct(struct_name.clone(), None, &specification_derives, RustVisibility::Public));

                output.extend(quote! {
                    impl FeagiMessageResponse for #struct_name {}
                });

                syn::parse_quote!(#struct_name)
            }
        };

        // Generate Feagi Message Struct Directly
        let feagi_message_name = feagi_message_specification_struct_names.feagi_message_response_struct_name().clone();
        let feagi_message_derives = vec![format_ident!("{}", "Clone"), format_ident!("{}", "Debug")]; // doesn't need serialization
        let feagi_message_tokens = all_request_fields.generate_rust_struct(
            feagi_message_name.clone(),
            Some(self.description.clone()),
            &feagi_message_derives,
            RustVisibility::Public,
        );
        output.extend(feagi_message_tokens);

        // Feagi Message impl
        let mut new_arguments = proc_macro2::TokenStream::new(); // arguments for new func
        let mut new_members = proc_macro2::TokenStream::new(); // parameters in new func (matches field names of struct)
        for (field_name, (field_type, _)) in all_request_fields.get_map() {
            new_arguments.extend(quote! { #field_name: #field_type, });
            new_members.extend(quote! { #field_name, });
        }

        output.extend(quote! {

            impl #feagi_message_name {
                pub fn new(#new_arguments) -> Self {
                    Self {
                        #new_members
                    }
                }
            }
        });

        // An empty specification is `()` and contributes no fields, so name the unused binding.
        let consume_parameters = if parameter_message_field_constructor.is_empty() {
            quote!(let _ = parameters;)
        } else {
            quote!()
        };
        let consume_queryables = if queryable_message_field_constructor.is_empty() {
            quote!(let _ = queryables;)
        } else {
            quote!()
        };
        let consume_payload = if payload_message_field_constructor.is_empty() {
            quote!(let _ = payload;)
        } else {
            quote!()
        };
        let consume_self = if all_request_fields.is_empty() {
            quote!(let _ = self;)
        } else {
            quote!()
        };

        let path_type = &feagi_message_specification_struct_names.feagi_message_response_path_struct_name();

        output.extend(quote! {

            impl FeagiMessage for #feagi_message_name {
                type Path = #path_type;
                type Parameters = #parameter_type;
                type Queryables = #queryable_type;
                type Payload = #payload_type;

                fn from_message_data(parameters: Self::Parameters, queryables: Self::Queryables, payload: Self::Payload) -> Self {
                    #consume_parameters
                    #consume_queryables
                    #consume_payload
                    Self {
                        #parameter_message_field_constructor
                        #queryable_message_field_constructor
                        #payload_message_field_constructor
                    }
                }

                fn to_message_data(self) -> (Self::Parameters, Self::Queryables, Self::Payload) {
                    #consume_self
                    (
                        #parameter_value,
                        #queryable_value,
                        #payload_value,
                    )
                }
            }

            impl FeagiRequestMessage for #feagi_message_name {
                type Response = #response_type;
            }

        });

        output
    }
}

impl<const METHOD_TYPE_U8: FeagiRequestMessageU8> Parse for FeagiRequestMessage<METHOD_TYPE_U8> {
    /// Parse the braced message body. `"MessageTitle":` belongs to the parent entry.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let members;
        braced!(members in input);

        let path = parse_property_colon_member::<kw::path, FeagiRequestMessagePath<METHOD_TYPE_U8>>(&members)?;
        let path_item_descriptions = parse_property_colon_member::<kw::path_item_descriptions, PropertyDescriptors>(&members)?;
        let description = parse_property_colon_member::<kw::description, LitStr>(&members)?;
        let payload;
        if !FeagiRequestMessageMethod::from_u8(METHOD_TYPE_U8).had_no_payload() {
            payload = parse_property_colon_member::<kw::payload, StructTemplate>(&members)?;
        } else {
            payload = StructTemplate::new_empty();
        }

        let response = parse_property_colon_member::<kw::response, StructTemplate>(&members)?;

        if !members.is_empty() {
            return Err(members.error("unexpected tokens in FEAGI message"));
        }

        Ok(Self {
            path,
            path_item_descriptions,
            description,
            payload,
            response,
        })
    }
}

impl<const METHOD_TYPE_U8: FeagiRequestMessageU8> Unparse for FeagiRequestMessage<METHOD_TYPE_U8> {
    /// Re-emit the braced message body so it can be parsed again.
    fn unparse(&self) -> proc_macro2::TokenStream {
        let path = self.path.unparse();
        let path_item_descriptions = self.path_item_descriptions.unparse();
        let description = &self.description;
        let payload = self.payload.unparse();
        let response = self.response.unparse();

        if !FeagiRequestMessageMethod::from_u8(METHOD_TYPE_U8).had_no_payload() {
            quote! {
                {
                    path: #path,
                    path_item_descriptions: #path_item_descriptions,
                    description: #description,
                    payload: #payload,
                    response: #response,
                }
            }
        } else {
            quote! {
                {
                    path: #path,
                    path_item_descriptions: #path_item_descriptions,
                    description: #description,
                    response: #response,
                }
            }
        }
    }
}

//endregion