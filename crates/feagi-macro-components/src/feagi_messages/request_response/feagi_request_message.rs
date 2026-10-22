use crate::common::{basis_path, field_description_tokens, serde_data_derive_attributes, unit_type, RustVisibility};
use crate::feagi_messages::request_response::common_structs::{FeagiRequestMessageMethod, FeagiRequestMessageU8};
use crate::feagi_messages::request_response::feagi_request_message_category::FeagiRequestMessageCategoryContext;
use crate::feagi_messages::request_response::feagi_request_message_path::{FeagiRequestMessagePath, FeagiRequestMessagePathRustStructNames};
use crate::feagi_messages::request_response::message_error::MessageErrorTemplate;
use crate::template_parsing::property_descriptors::PropertyDescriptors;
use crate::template_parsing::struct_template::StructTemplate;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{braced, LitBool, LitStr, Token};

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

    /// Name of the variant this message has in its category's request and response enums
    pub fn category_enum_variant_name(&self) -> syn::Ident {
        format_ident!("{}{}", self.method.as_str(), self.base_name)
    }

    pub fn feagi_message_struct_name(&self) -> syn::Ident {
        self.with_prefix_method_and_given_suffix("FeagiRequestMessage")
    }

    pub fn feagi_message_descriptor_struct_name(&self) -> syn::Ident {
        self.with_prefix_method_and_given_suffix("FeagiRequestMessageDescriptor")
    }

    pub fn path_struct_names(&self) -> FeagiRequestMessagePathRustStructNames {
        FeagiRequestMessagePathRustStructNames::new(self.base_name.clone(), self.method.clone())
    }

    pub fn feagi_message_path_struct_name(&self) -> syn::Ident {
        self.path_struct_names().feagi_message_path_struct_name()
    }

    pub fn feagi_message_parameter_specification_struct_name(&self) -> syn::Ident {
        self.with_prefix_method_and_given_suffix("FeagiRequestMessageParameters")
    }

    pub fn feagi_message_payload_specification_struct_name(&self) -> syn::Ident {
        self.with_prefix_method_and_given_suffix("FeagiRequestMessagePayload")
    }

    pub fn feagi_message_response_specification_struct_name(&self) -> syn::Ident {
        self.with_prefix_method_and_given_suffix("FeagiRequestMessageResponse")
    }

    pub fn feagi_message_error_struct_name(&self) -> syn::Ident {
        self.with_prefix_method_and_given_suffix("FeagiRequestMessageError")
    }

    fn with_prefix_method_and_given_suffix(&self, suffix: &str) -> syn::Ident {
        format_ident!("{}{}{}", self.method.as_str(), self.base_name, suffix)
    }
}

//endregion

//region Feagi Request Message

/// Parses over a FeagiMessage. Formatted as the following, with the keys in any order
/// "MessageTitle": {
///     path: `FeagiMessagePath`,
///     path_item_descriptions: {          (optional)
///         property_name: "Description String",
///     },
///     description: "Description String",
///     alloc_only: true,                  (optional, defaults to false)
///     payload: {                         (optional, Read messages send it as the URL query in REST)
///         property_name: Type, "optional_comment",
///     },
///     response: {                        (optional)
///         property_name: Type, "optional_comment",
///     },
///     error: `MessageErrorTemplate`,     (optional)
/// }
pub struct FeagiRequestMessage<const METHOD_TYPE_U8: FeagiRequestMessageU8> {
    pub path: FeagiRequestMessagePath<METHOD_TYPE_U8>,
    pub path_item_descriptions: PropertyDescriptors,
    pub description: LitStr,
    /// The message uses types that need an allocator. It only exists when `feagi_basis` is built
    /// with its `alloc` feature. When false, the template author confirms the message works
    /// without an allocator
    pub alloc_only: bool,
    pub payload: StructTemplate,
    pub response: StructTemplate,
    pub error: MessageErrorTemplate,
}

impl<const METHOD_TYPE_U8: FeagiRequestMessageU8> FeagiRequestMessage<METHOD_TYPE_U8> {
    /// Whether this message's path defines a parameter struct.
    pub fn is_parameter_struct_generated(&self) -> bool {
        self.path.is_parameter_struct_generated()
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

    /// The type of the response payload, `()` when the message has no response fields
    pub fn response_payload_type(&self, names: &FeagiRequestMessageRustStructNames) -> syn::Type {
        if self.is_response_struct_generated() {
            let name = names.feagi_message_response_specification_struct_name();
            syn::parse_quote!(#name)
        } else {
            unit_type()
        }
    }

    /// Const `FeagiFieldDescription`s of the fields of `template`, documented by
    /// `path_item_descriptions` where given, otherwise by their own comment
    fn field_descriptions(&self, template: &StructTemplate) -> Vec<proc_macro2::TokenStream> {
        let mut template = template.clone();
        template.overwrite_descriptions_from_property_descriptors(&self.path_item_descriptions);
        template
            .get_map()
            .iter()
            .map(|(name, (field_type, description))| field_description_tokens(name, field_type, description.as_ref()))
            .collect()
    }

    /// The type of the path parameters, `()` when the path has none
    pub fn parameters_type(&self, names: &FeagiRequestMessageRustStructNames) -> syn::Type {
        if self.is_parameter_struct_generated() {
            let name = names.feagi_message_parameter_specification_struct_name();
            syn::parse_quote!(#name)
        } else {
            unit_type()
        }
    }

    /// The type of the request payload, `()` when the message has no payload fields
    pub fn payload_type(&self, names: &FeagiRequestMessageRustStructNames) -> syn::Type {
        if self.is_payload_struct_generated() {
            let name = names.feagi_message_payload_specification_struct_name();
            syn::parse_quote!(#name)
        } else {
            unit_type()
        }
    }

    /// The type of the response error
    pub fn response_error_type(&self, names: &FeagiRequestMessageRustStructNames) -> proc_macro2::TokenStream {
        self.error.error_type(&names.feagi_message_error_struct_name())
    }

    /// Generate the path struct, the specification structs, the error enum (if defined), the
    /// message struct, and the descriptor of this message
    pub fn generate_rust_message_items(&self, names: &FeagiRequestMessageRustStructNames, category: &FeagiRequestMessageCategoryContext) -> proc_macro2::TokenStream {
        let basis = basis_path();
        let request_module = quote!(#basis::feagi_message::request);
        let payloads = quote!(#request_module::feagi_request_response_payloads);
        let specification_derives = serde_data_derive_attributes();
        let mut output = proc_macro2::TokenStream::new();

        // Every field of the message, from all of its request specifications
        let mut all_request_fields = StructTemplate::new_empty();

        let mut generate_specification = |template: StructTemplate,
                                          struct_name: syn::Ident,
                                          trait_path: proc_macro2::TokenStream,
                                          argument: syn::Ident,
                                          output: &mut proc_macro2::TokenStream|
         -> Specification {
            if template.is_empty() {
                return Specification {
                    type_name: unit_type(),
                    compose_fields: quote!(),
                    decompose_value: quote!(()),
                    consume_argument: quote!(let _ = #argument;),
                };
            }

            let mut template = template;
            template.overwrite_descriptions_from_property_descriptors(&self.path_item_descriptions);
            let mut compose_fields = proc_macro2::TokenStream::new();
            let mut decompose_fields = proc_macro2::TokenStream::new();
            for (field_name, (field_type, field_docs)) in template.get_map() {
                all_request_fields.insert_property(field_name.clone(), field_type.clone(), field_docs.clone());
                compose_fields.extend(quote!(#field_name: #argument.#field_name,));
                decompose_fields.extend(quote!(#field_name: self.#field_name,));
            }

            output.extend(template.generate_rust_struct(struct_name.clone(), None, specification_derives.clone(), RustVisibility::Public));
            output.extend(quote! {
                impl #trait_path for #struct_name {}
            });

            Specification {
                type_name: syn::parse_quote!(#struct_name),
                compose_fields,
                decompose_value: quote!(#struct_name { #decompose_fields }),
                consume_argument: quote!(),
            }
        };

        let parameters = generate_specification(
            self.path.as_parameter_struct_template(),
            names.feagi_message_parameter_specification_struct_name(),
            quote!(#basis::feagi_message::feagi_message_path::parameters::FeagiMessagePathParameters),
            format_ident!("parameters"),
            &mut output,
        );
        let payload = generate_specification(
            self.payload.clone(),
            names.feagi_message_payload_specification_struct_name(),
            quote!(#payloads::FeagiRequestPayload),
            format_ident!("request_payload"),
            &mut output,
        );

        output.extend(self.path.generate_feagi_message_path_struct_and_impls(
            &names.path_struct_names(),
            &parameters.type_name,
            &self.path_item_descriptions,
            category,
        ));

        if self.is_response_struct_generated() {
            let mut template = self.response.clone();
            template.overwrite_descriptions_from_property_descriptors(&self.path_item_descriptions);
            let struct_name = names.feagi_message_response_specification_struct_name();
            output.extend(template.generate_rust_struct(struct_name.clone(), None, specification_derives.clone(), RustVisibility::Public));
            output.extend(quote! {
                impl #payloads::FeagiResponsePayload for #struct_name {}
            });
        }

        output.extend(self.error.generate_rust_error_enum(&names.feagi_message_error_struct_name(), &self.description));

        // Message struct
        let message_name = names.feagi_message_struct_name();
        output.extend(all_request_fields.generate_rust_struct(
            message_name.clone(),
            Some(self.description.clone()),
            quote!(#[derive(Clone, Debug)]),
            RustVisibility::Public,
        ));

        let new_arguments = all_request_fields.get_map().iter().map(|(name, (field_type, _))| quote!(#name: #field_type));
        let new_members = all_request_fields.get_map().keys();
        let consume_self = if all_request_fields.is_empty() { quote!(let _ = self;) } else { quote!() };

        let descriptor_name = names.feagi_message_descriptor_struct_name();
        let (parameters_type, payload_type) = (&parameters.type_name, &payload.type_name);
        let (consume_parameters, consume_payload) = (&parameters.consume_argument, &payload.consume_argument);
        let (compose_parameters, compose_payload) = (&parameters.compose_fields, &payload.compose_fields);
        let (decompose_parameters, decompose_payload) = (&parameters.decompose_value, &payload.decompose_value);

        output.extend(quote! {
            impl #message_name {
                pub fn new(#( #new_arguments, )*) -> Self {
                    Self { #( #new_members, )* }
                }
            }

            impl #request_module::feagi_request_message::FeagiRequestMessage for #message_name {
                type Descriptor = #descriptor_name;
                type DescriptorParameters = #parameters_type;
                type DescriptorRequestPayload = #payload_type;

                fn compose(parameters: Self::DescriptorParameters, request_payload: Self::DescriptorRequestPayload) -> Self {
                    #consume_parameters
                    #consume_payload
                    Self {
                        #compose_parameters
                        #compose_payload
                    }
                }

                fn decompose(self) -> (Self::DescriptorParameters, Self::DescriptorRequestPayload) {
                    #consume_self
                    (#decompose_parameters, #decompose_payload)
                }
            }
        });

        // Descriptor
        let description = &self.description;
        let path_name = names.feagi_message_path_struct_name();
        let response_payload_type = self.response_payload_type(names);
        let response_error_type = self.response_error_type(names);
        let request_payload_fields = self.field_descriptions(&self.payload);
        let response_payload_fields = self.field_descriptions(&self.response);
        let field_description = quote!(#basis::feagi_message::feagi_message_description::FeagiFieldDescription);
        let category_marker = &category.category_marker_name;
        let request_enum = &category.request_enum_name;
        let response_enum = &category.response_enum_name;
        let variant = names.category_enum_variant_name();
        let method_marker = format_ident!("{}", FeagiRequestMessageMethod::from_u8(METHOD_TYPE_U8).marker_type_name());

        output.extend(quote! {
            #[doc = #description]
            #[derive(Clone, Copy, Debug)]
            pub struct #descriptor_name;

            impl #request_module::feagi_request_descriptor::FeagiRequestMessageDescriptor for #descriptor_name {
                const MESSAGE_DESCRIPTION: &'static str = #description;
                type Message = #message_name;
                type MessagePath = #path_name;
                type Method = #request_module::feagi_request_descriptor::#method_marker;
                type Parameters = #parameters_type;
                type RequestPayload = #payload_type;
                type ResponsePayload = #response_payload_type;
                type ResponseError = #response_error_type;
                const REQUEST_PAYLOAD_FIELDS: &'static [#field_description] = &[ #( #request_payload_fields, )* ];
                const RESPONSE_PAYLOAD_FIELDS: &'static [#field_description] = &[ #( #response_payload_fields, )* ];
                type Category = #category_marker;

                fn into_category_request(message: Self::Message) -> #request_enum {
                    #request_enum::#variant(message)
                }

                fn into_category_response(response: ::core::result::Result<Self::ResponsePayload, Self::ResponseError>) -> #response_enum {
                    #response_enum::#variant(response)
                }

                fn try_from_category_response(
                    response: #response_enum,
                ) -> ::core::result::Result<::core::result::Result<Self::ResponsePayload, Self::ResponseError>, #response_enum> {
                    match response {
                        #response_enum::#variant(response) => Ok(response),
                        other => Err(other),
                    }
                }
            }
        });

        output
    }
}

/// How one request specification (parameters, payload) is wired into the message
struct Specification {
    /// The specification struct, or `()` when it has no fields
    type_name: syn::Type,
    /// `field: argument.field,` for each field, used by `compose`
    compose_fields: proc_macro2::TokenStream,
    /// Expression rebuilding the specification from `self`, used by `decompose`
    decompose_value: proc_macro2::TokenStream,
    /// Marks the `compose` argument as used when the specification is `()`
    consume_argument: proc_macro2::TokenStream,
}

mod kw {
    syn::custom_keyword!(path);
    syn::custom_keyword!(path_item_descriptions);
    syn::custom_keyword!(description);
    syn::custom_keyword!(alloc_only);
    syn::custom_keyword!(payload);
    syn::custom_keyword!(response);
    syn::custom_keyword!(error);
}

impl<const METHOD_TYPE_U8: FeagiRequestMessageU8> Parse for FeagiRequestMessage<METHOD_TYPE_U8> {
    /// Parse the braced message body. `"MessageTitle":` belongs to the parent entry.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        /// Errors if `slot` was already filled by an earlier key
        fn set_once<T>(slot: &mut Option<T>, value: T, key: &str, span: proc_macro2::Span) -> syn::Result<()> {
            if slot.is_some() {
                return Err(syn::Error::new(span, format!("duplicate `{key}` in FEAGI message")));
            }
            *slot = Some(value);
            Ok(())
        }

        let members;
        braced!(members in input);

        let mut path = None;
        let mut path_item_descriptions = None;
        let mut description = None;
        let mut alloc_only = None;
        let mut payload = None;
        let mut response = None;
        let mut error = None;

        while !members.is_empty() {
            let key: syn::Ident = members.fork().call(syn::ext::IdentExt::parse_any)?;
            let span = key.span();
            let lookahead = members.lookahead1();
            if lookahead.peek(kw::path) {
                members.parse::<kw::path>()?;
                members.parse::<Token![:]>()?;
                set_once(&mut path, members.parse()?, "path", span)?;
            } else if lookahead.peek(kw::path_item_descriptions) {
                members.parse::<kw::path_item_descriptions>()?;
                members.parse::<Token![:]>()?;
                set_once(&mut path_item_descriptions, members.parse()?, "path_item_descriptions", span)?;
            } else if lookahead.peek(kw::description) {
                members.parse::<kw::description>()?;
                members.parse::<Token![:]>()?;
                set_once(&mut description, members.parse()?, "description", span)?;
            } else if lookahead.peek(kw::alloc_only) {
                members.parse::<kw::alloc_only>()?;
                members.parse::<Token![:]>()?;
                let value: LitBool = members.parse()?;
                set_once(&mut alloc_only, value.value, "alloc_only", span)?;
            } else if lookahead.peek(kw::payload) {
                members.parse::<kw::payload>()?;
                members.parse::<Token![:]>()?;
                set_once(&mut payload, members.parse()?, "payload", span)?;
            } else if lookahead.peek(kw::response) {
                members.parse::<kw::response>()?;
                members.parse::<Token![:]>()?;
                set_once(&mut response, members.parse()?, "response", span)?;
            } else if lookahead.peek(kw::error) {
                members.parse::<kw::error>()?;
                members.parse::<Token![:]>()?;
                set_once(&mut error, members.parse()?, "error", span)?;
            } else {
                return Err(lookahead.error());
            }

            if members.is_empty() {
                break;
            }
            members.parse::<Token![,]>()?;
        }

        let Some(path) = path else {
            return Err(members.error("FEAGI message is missing `path`"));
        };
        let Some(description) = description else {
            return Err(members.error("FEAGI message is missing `description`"));
        };

        Ok(Self {
            path,
            path_item_descriptions: path_item_descriptions.unwrap_or_else(PropertyDescriptors::new_empty),
            description,
            alloc_only: alloc_only.unwrap_or(false),
            payload: payload.unwrap_or_else(StructTemplate::new_empty),
            response: response.unwrap_or_else(StructTemplate::new_empty),
            error: error.unwrap_or(MessageErrorTemplate::Default),
        })
    }
}

//endregion
