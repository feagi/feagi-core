use crate::common::{parse_property_colon_member, unit_type, RustVisibility};
use crate::feagi_messages::request_response::common_structs::{FeagiRequestMessageMethod, FeagiRequestMessageU8};
use crate::feagi_messages::request_response::feagi_request_message_path::{FeagiRequestMessagePath, FeagiRequestMessagePathRustStructNames};
use crate::template_parsing::property_descriptors::PropertyDescriptors;
use crate::template_parsing::struct_template::StructTemplate;
use crate::templates::Unparse;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{braced, LitStr};

mod kw {
    syn::custom_keyword!(path);
    syn::custom_keyword!(path_item_descriptions);
    syn::custom_keyword!(description);
    syn::custom_keyword!(payload);
    syn::custom_keyword!(response);
}

//region Feagi Request Message Specification Struct Names

/// Contains all the names of the rust structs that would be generated from a FeagiMessage.
/// Whether a given named struct is actually emitted is decided by [`FeagiRequestMessage`],
/// which holds the path, payload, and response context.
#[derive(Clone)]
pub struct FeagiRequestMessageRustStructNames {
    pub base_name: syn::Ident,
    pub method: FeagiRequestMessageMethod,
}

impl FeagiRequestMessageRustStructNames {
    /// Build the name set from the message title and HTTP-equivalent method.
    pub fn new(base_name: syn::Ident, method: FeagiRequestMessageMethod) -> Self {
        Self { base_name, method }
    }

    pub fn feagi_message_response_struct_name(&self) -> syn::Ident {
        self.with_prefix_method_and_given_suffix("FeagiRequestMessage")
    }

    pub fn path_struct_names(&self) -> FeagiRequestMessagePathRustStructNames {
        FeagiRequestMessagePathRustStructNames::new(self.base_name.clone(), self.method.clone())
    }

    pub fn feagi_message_response_path_struct_name(&self) -> syn::Ident {
        self.path_struct_names().feagi_message_path_struct_name()
    }

    pub fn feagi_message_parameter_specification_struct_name(&self) -> syn::Ident {
        self.with_prefix_method_and_given_suffix("FeagiRequestMessageParameters")
    }

    pub fn feagi_message_queryable_specification_struct_name(&self) -> syn::Ident {
        self.with_prefix_method_and_given_suffix("FeagiRequestMessageQueryables")
    }

    pub fn feagi_message_payload_specification_struct_name(&self) -> syn::Ident {
        self.with_prefix_method_and_given_suffix("FeagiRequestMessagePayload")
    }

    pub fn feagi_message_response_specification_struct_name(&self) -> syn::Ident {
        self.with_prefix_method_and_given_suffix("FeagiRequestMessageResponse")
    }

    fn with_prefix_method_and_given_suffix(&self, suffix: &str) -> syn::Ident {
        format_ident!("{}{}{}", self.method.as_str(), self.base_name, suffix)
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
    pub custom_response_error: Option<syn::Ident>,
}

impl<const METHOD_TYPE_U8: FeagiRequestMessageU8> FeagiRequestMessage<METHOD_TYPE_U8> {
    /// Whether this message's path defines a parameter struct.
    pub fn is_parameter_struct_generated(&self) -> bool {
        self.path.is_parameter_struct_generated()
    }

    /// Whether this message's path defines a queryable struct.
    pub fn is_queryable_struct_generated(&self) -> bool {
        self.path.is_queryable_struct_generated()
    }

    /// Whether this message defines a payload struct.
    pub fn is_payload_struct_generated(&self) -> bool {
        !self.payload.is_empty()
    }

    /// Whether this message defines a response struct.
    pub fn is_response_struct_generated(&self) -> bool {
        !self.response.is_empty()
    }

    /// Struct names this message will generate. Existence of each named struct
    /// is queried from this message, not from the name set.
    pub fn calculate_feagi_message_struct_names(&self, name_base: syn::Ident) -> FeagiRequestMessageRustStructNames {
        let _ = self;
        FeagiRequestMessageRustStructNames::new(name_base, FeagiRequestMessageMethod::from_u8(METHOD_TYPE_U8))
    }

    /// Generate the Specification structs, the feagi request message struct, and the response struct (if valid)
    pub fn generate_rust_specification_and_request_and_response_structs(
        &self,
        feagi_message_specification_struct_names: FeagiRequestMessageRustStructNames,
    ) -> proc_macro2::TokenStream {
        let mut output: proc_macro2::TokenStream = proc_macro2::TokenStream::new();
        let specification_derives = vec![
            format_ident!("{}", "Clone"),
            format_ident!("{}", "Debug"),
            format_ident!("{}", "Serialize"),
            format_ident!("{}", "Deserialize"),
        ];

        let tokens_message_path = self.path.generate_feagi_message_path_struct_and_impls(
            &feagi_message_specification_struct_names.path_struct_names(),
            &feagi_message_specification_struct_names.feagi_message_response_struct_name(),
        ); // TODO add root route
        output.extend(tokens_message_path);

        let mut all_request_fields = StructTemplate::new_empty();

        let (parameter_type, parameter_message_field_constructor, parameter_value) = {
            // Struct is parsed from path itself
            if !self.is_parameter_struct_generated() {
                (unit_type(), proc_macro2::TokenStream::new(), quote!(()))
            } else {
                let mut struct_template = self.path.as_parameter_struct_template();
                struct_template.overwrite_descriptions_from_property_descriptors(&self.path_item_descriptions);

                let struct_name = feagi_message_specification_struct_names.feagi_message_parameter_specification_struct_name();

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
            if !self.is_queryable_struct_generated() {
                (unit_type(), proc_macro2::TokenStream::new(), quote!(()))
            } else {
                let mut struct_template = self.path.as_queryable_struct_template();
                struct_template.overwrite_descriptions_from_property_descriptors(&self.path_item_descriptions);

                let struct_name = feagi_message_specification_struct_names.feagi_message_queryable_specification_struct_name();

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
            if !self.is_payload_struct_generated() {
                (unit_type(), proc_macro2::TokenStream::new(), quote!(()))
            } else {
                let mut struct_template = self.payload.clone();
                struct_template.overwrite_descriptions_from_property_descriptors(&self.path_item_descriptions);

                let struct_name = feagi_message_specification_struct_names.feagi_message_payload_specification_struct_name();

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
            if !self.is_response_struct_generated() {
                unit_type()
            } else {
                let mut struct_template = self.response.clone();
                struct_template.overwrite_descriptions_from_property_descriptors(&self.path_item_descriptions);

                let struct_name = feagi_message_specification_struct_names.feagi_message_response_specification_struct_name();

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
        let consume_parameters = if !self.is_parameter_struct_generated() {
            quote!(let _ = parameters;)
        } else {
            quote!()
        };
        let consume_queryables = if !self.is_queryable_struct_generated() {
            quote!(let _ = queryables;)
        } else {
            quote!()
        };
        let consume_payload = if !self.is_payload_struct_generated() {
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
                type ResponseData = #response_type;
                type ResponseCustomError = ();
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
            custom_response_error: None, // TODO
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

        // TODO unparse error type

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
