use crate::common::{lit_str_to_ident, parse_optional_comma, parse_property_colon_member, RustVisibility};
use crate::template_parsing::property_descriptors::PropertyDescriptors;
use crate::template_parsing::struct_template::StructTemplate;
use crate::templates::Unparse;
use proc_macro2::Span;
use quote::{format_ident, quote, TokenStreamExt};
use std::collections::HashMap;
use heck::{ToPascalCase, ToSnakeCase};
use syn::parse::{Parse, ParseStream};
use syn::{braced, LitStr, Token, Type};
use crate::template_parsing::message_path_element::MessagePathElement;

mod kw {

    syn::custom_keyword!(category_name);
    syn::custom_keyword!(base_path);
    syn::custom_keyword!(category_description);
    syn::custom_keyword!(feagi_error_type);
    syn::custom_keyword!(read);
    syn::custom_keyword!(create);
    syn::custom_keyword!(edit);
    syn::custom_keyword!(delete);
    syn::custom_keyword!(patch);

    syn::custom_keyword!(path);
    syn::custom_keyword!(path_item_descriptions);
    syn::custom_keyword!(description);
    syn::custom_keyword!(payload);
    syn::custom_keyword!(response);
}

//region Feagi Message Category

/// Holds all messages of a given category.  Formatted as the following:
/// category_name: "category_name",
/// base_path: "base_path",
/// category_description: "category_description",
/// (following sections are optional)
/// read: {
///     "MessageTitle": FeagiMessageWithoutPayload,
/// }
/// create: {
///     "MessageTitle": FeagiMessageWithPayload,
/// }
/// edit: {
///     "MessageTitle": FeagiMessageWithPayload,
/// }
/// delete: {
///     "MessageTitle": FeagiMessageWithPayload,
/// }
/// patch: {
///     "MessageTitle": FeagiMessageWithPayload,
/// }
pub struct FeagiMessageCategory {
    pub category_name: LitStr, // PascalCase
    /// Any additional path to this request that is appended to all defined
    pub base_path: LitStr,
    /// Comment Description of this category
    pub category_description: LitStr,
    pub read: HashMap<LitStr, FeagiRequestMessage<0>>, // TODO fix message type (why doesnt FeagiMessageType::Const work???)
    pub create: HashMap<LitStr, FeagiRequestMessage<1>>, // Name is PascalCase
    pub edit: HashMap<LitStr, FeagiRequestMessage<2>>,
    pub delete: HashMap<LitStr, FeagiRequestMessage<3>>,
    pub patch: HashMap<LitStr, FeagiRequestMessage<4>>,
}

impl FeagiMessageCategory {

    pub fn generate_rust_feagi_category_group(&self) -> proc_macro2::TokenStream {

        // We arent going to use struct template here because these use generics

        let mut properties = proc_macro2::TokenStream::new();

        // NOTE: Yes, this is repetitive, unfortunantely rustrover is stupid and will not accept
        // the below blocks to be wrapped in a function for some reason
        
        for (name, request) in &self.read {
            let message_struct_name = format_ident!("{}", name.value());
            let message_field_name = format_ident!("{}", name.value().to_snake_case());
            let message_path_name = FeagiRequestMessagePath::<0>::get_rust_struct_name(&message_struct_name);
            let description = &request.description;

            properties.extend(quote!{
                    ##[doc(#description)]
                    pub #message_field_name: (#message_path_name, T),
                });
        }

        for (name, request) in &self.create {
            let message_struct_name = format_ident!("{}", name.value());
            let message_field_name = format_ident!("{}", name.value().to_snake_case());
            let message_path_name = FeagiRequestMessagePath::<0>::get_rust_struct_name(&message_struct_name);
            let description = &request.description;

            properties.extend(quote!{
                    ##[doc(#description)]
                    pub #message_field_name: (#message_path_name, T),
                });
        }

        for (name, request) in &self.edit {
            let message_struct_name = format_ident!("{}", name.value());
            let message_field_name = format_ident!("{}", name.value().to_snake_case());
            let message_path_name = FeagiRequestMessagePath::<0>::get_rust_struct_name(&message_struct_name);
            let description = &request.description;

            properties.extend(quote!{
                    ##[doc(#description)]
                    pub #message_field_name: (#message_path_name, T),
                });
        }

        for (name, request) in &self.delete {
            let message_struct_name = format_ident!("{}", name.value());
            let message_field_name = format_ident!("{}", name.value().to_snake_case());
            let message_path_name = FeagiRequestMessagePath::<0>::get_rust_struct_name(&message_struct_name);
            let description = &request.description;

            properties.extend(quote!{
                    ##[doc(#description)]
                    pub #message_field_name: (#message_path_name, T),
                });
        }

        for (name, request) in &self.patch {
            let message_struct_name = format_ident!("{}", name.value());
            let message_field_name = format_ident!("{}", name.value().to_snake_case());
            let message_path_name = FeagiRequestMessagePath::<0>::get_rust_struct_name(&message_struct_name);
            let description = &request.description;

            properties.extend(quote!{
                    ##[doc(#description)]
                    pub #message_field_name: (#message_path_name, T),
                });
        };

        let struct_name = format_ident!("{}FeagiRequestMessageCategoryMappings", &self.category_name.value().to_pascal_case());
        let docs = &self.category_description;

        quote! {
            ##[doc(#docs)]
            pub struct #struct_name<T> {
                #properties
            }

            impl<T> FeagiRequestMessageCategoryMappings<T> for #struct_name<T> {}
        }

        // TODO implement clone depending on T implementing Clone

    }

    /// Create the specification data, feagi messages, and response structs for all internal categorized feagi messages
    pub fn generate_rust_structs_for_all_messages(&self) -> proc_macro2::TokenStream {
        let mut output = proc_macro2::TokenStream::new();

        for read in &self.read {
            let read_name = format_ident!("{}", read.0.value());
            let request_message = read.1;
            let struct_names = request_message.generate_feagi_message_struct_names(read_name);
            output.extend(request_message.generate_feagi_message_structs(struct_names));
        };

        for create in &self.create {
            let create_name = format_ident!("{}", create.0.value());
            let request_message = create.1;
            let struct_names = request_message.generate_feagi_message_struct_names(create_name);
            output.extend(request_message.generate_feagi_message_structs(struct_names));
        };

        for edit in &self.edit {
            let edit_name = format_ident!("{}", edit.0.value());
            let request_message = edit.1;
            let struct_names = request_message.generate_feagi_message_struct_names(edit_name);
            output.extend(request_message.generate_feagi_message_structs(struct_names));
        };

        for delete in &self.delete {
            let delete_name = format_ident!("{}", delete.0.value());
            let request_message = delete.1;
            let struct_names = request_message.generate_feagi_message_struct_names(delete_name);
            output.extend(request_message.generate_feagi_message_structs(struct_names));
        };

        for patch in &self.patch {
            let patch_name = format_ident!("{}", patch.0.value());
            let request_message = patch.1;
            let struct_names = request_message.generate_feagi_message_struct_names(patch_name);
            output.extend(request_message.generate_feagi_message_structs(struct_names));
        };

        output
    }

    /// Create the categorized names of the feagi request messages from all internal feagi request messages
    pub fn create_categorized_rust_struct_names(&self) -> FeagiMessageRustStructNamesCategorized {
        let mut output = FeagiMessageRustStructNamesCategorized::new_empty();

        for (message_name, message) in &self.read {
            let message_name = format_ident!("{}", message_name.value());
            let message_names = message.generate_feagi_message_struct_names(message_name);
            output.read.push(message_names);
        };

        for (message_name, message) in &self.create {
            let message_name = format_ident!("{}", message_name.value());
            let message_names = message.generate_feagi_message_struct_names(message_name);
            output.create.push(message_names);
        };

        for (message_name, message) in &self.edit {
            let message_name = format_ident!("{}", message_name.value());
            let message_names = message.generate_feagi_message_struct_names(message_name);
            output.edit.push(message_names);
        };

        for (message_name, message) in &self.delete {
            let message_name = format_ident!("{}", message_name.value());
            let message_names = message.generate_feagi_message_struct_names(message_name);
            output.delete.push(message_names);
        };

        for (message_name, message) in &self.patch {
            let message_name = format_ident!("{}", message_name.value());
            let message_names = message.generate_feagi_message_struct_names(message_name);
            output.patch.push(message_names);
        };
        output
    }
}

impl Parse for FeagiMessageCategory {
    /// Parse the category fields. Verb sections may be omitted, in docstring order.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        /// Parse `keyword` was already consumed. Read `: { "Title": Message, ... }`.
        fn parse_bucket<Message: Parse>(input: ParseStream, section: &str) -> syn::Result<HashMap<LitStr, Message>> {
            input.parse::<Token![:]>()?;
            let entries;
            braced!(entries in input);
            let mut messages = HashMap::new();
            while !entries.is_empty() {
                let title: LitStr = entries.parse()?;
                entries.parse::<Token![:]>()?;
                let message: Message = entries.parse()?;
                let title_span = title.span();
                if messages.insert(title, message).is_some() {
                    return Err(syn::Error::new(title_span, format!("duplicate message title in `{section}`")));
                }
                if entries.is_empty() {
                    break;
                }
                entries.parse::<Token![,]>()?;
            }
            parse_optional_comma(input)?;
            Ok(messages)
        }

        let category_name = parse_property_colon_member::<kw::category_name, LitStr>(input)?;
        let base_path = parse_property_colon_member::<kw::base_path, LitStr>(input)?;
        let category_description = parse_property_colon_member::<kw::category_description, LitStr>(input)?;

        let read = if input.peek(kw::read) {
            input.parse::<kw::read>()?;
            parse_bucket::<FeagiRequestMessage<0>>(input, "read")?
        } else {
            HashMap::new()
        };
        let create = if input.peek(kw::create) {
            input.parse::<kw::create>()?;
            parse_bucket::<FeagiRequestMessage<1>>(input, "create")?
        } else {
            HashMap::new()
        };
        let edit = if input.peek(kw::edit) {
            input.parse::<kw::edit>()?;
            parse_bucket::<FeagiRequestMessage<2>>(input, "edit")?
        } else {
            HashMap::new()
        };
        let delete = if input.peek(kw::delete) {
            input.parse::<kw::delete>()?;
            parse_bucket::<FeagiRequestMessage<3>>(input, "delete")?
        } else {
            HashMap::new()
        };
        let patch = if input.peek(kw::patch) {
            input.parse::<kw::patch>()?;
            parse_bucket::<FeagiRequestMessage<4>>(input, "patch")?
        } else {
            HashMap::new()
        };

        if !input.is_empty() {
            return Err(input.error("unexpected tokens in FEAGI message category"));
        }

        Ok(Self {
            category_name,
            base_path,
            category_description,
            read,
            create,
            edit,
            delete,
            patch,
        })
    }
}

impl Unparse for FeagiMessageCategory {
    /// Re-emit the category. Empty verb sections are left out.
    fn unparse(&self) -> proc_macro2::TokenStream {
        /// Append `section: { "Title": Message, ... },` when the map is not empty.
        fn extend_bucket<Message: Unparse>(stream: &mut proc_macro2::TokenStream, section: &str, messages: &HashMap<LitStr, Message>) {
            if messages.is_empty() {
                return;
            }
            let section = proc_macro2::Ident::new(section, Span::call_site());
            let mut entries = proc_macro2::TokenStream::new();
            for (title, message) in messages {
                let body = message.unparse();
                entries.extend(quote!(#title: #body,));
            }
            stream.extend(quote! {
                #section: {
                    #entries
                },
            });
        }

        let category_name = &self.category_name;
        let base_path = &self.base_path;
        let category_description = &self.category_description;
        let mut stream = quote! {
            category_name: #category_name,
            base_path: #base_path,
            category_description: #category_description,
        };
        extend_bucket(&mut stream, "read", &self.read);
        extend_bucket(&mut stream, "create", &self.create);
        extend_bucket(&mut stream, "edit", &self.edit);
        extend_bucket(&mut stream, "delete", &self.delete);
        extend_bucket(&mut stream, "patch", &self.patch);
        stream
    }
}

/// Contains the names of the structs across all feagi messages in a feagi message category
pub struct FeagiMessageRustStructNamesCategorized {
    pub read: Vec<FeagiRequestMessageRustStructNames>,
    pub create: Vec<FeagiRequestMessageRustStructNames>,
    pub edit: Vec<FeagiRequestMessageRustStructNames>,
    pub delete: Vec<FeagiRequestMessageRustStructNames>,
    pub patch: Vec<FeagiRequestMessageRustStructNames>,
}

impl FeagiMessageRustStructNamesCategorized {

    pub fn new_empty() -> Self {
        Self {
            read: Vec::new(),
            create: Vec::new(),
            edit: Vec::new(),
            delete: Vec::new(),
            patch: Vec::new(),
        }
    }

    pub fn get_by_method(&self, method: &FeagiRequestMessageMethod) -> &[FeagiRequestMessageRustStructNames] {
        match method {
            FeagiRequestMessageMethod::Read => {self.read.as_slice()}
            FeagiRequestMessageMethod::Create => {self.create.as_slice()}
            FeagiRequestMessageMethod::Edit => {self.edit.as_slice()}
            FeagiRequestMessageMethod::Delete => {self.delete.as_slice()}
            FeagiRequestMessageMethod::Patch => {self.patch.as_slice()}
        }
    }

    pub fn get_by_method_mut(&mut self, method: &FeagiRequestMessageMethod) -> &mut Vec<FeagiRequestMessageRustStructNames> {
        match method {
            FeagiRequestMessageMethod::Read => {&mut self.read}
            FeagiRequestMessageMethod::Create => {&mut self.create}
            FeagiRequestMessageMethod::Edit => {&mut self.edit}
            FeagiRequestMessageMethod::Delete => {&mut self.delete}
            FeagiRequestMessageMethod::Patch => {&mut self.patch}
        }
    }

    /// Generate the rust enums that contain all the Requests with their Feagi Request Messages, and the Responses with their response datas
    pub fn generate_rust_request_response_enums_for_category(&self, request_enum_name: syn::Ident, response_enum_name: syn::Ident) -> proc_macro2::TokenStream {
        let names = self.flatten_all_name_structs();
        
        let mut request_enum_variants = proc_macro2::TokenStream::new();
        let mut response_enum_variants = proc_macro2::TokenStream::new();
        let mut intos = proc_macro2::TokenStream::new();
        
        for name in &names {
            let base_name = name.method_base_title.clone();
            let message_name = name.message.clone();
            let maybe_response_name = name.response.clone();

            request_enum_variants.extend(quote! {
                #base_name(#message_name),
            });

            intos.extend( quote! {
                impl Into<#request_enum_name> for #message_name {
                    fn into(self) -> #request_enum_name {
                        #request_enum_name::#base_name(self)
                    }
                }
            });

            if let Some(response_name) = maybe_response_name {
                response_enum_variants.extend(quote! {
                    #base_name(#response_name),
                });

                intos.extend( quote! {
                impl Into<#response_enum_name> for #response_name {
                    fn into(self) -> #response_enum_name {
                        #response_enum_name::#base_name(self)
                    }
                }
            });

            } else {
                response_enum_variants.extend(quote! {
                    #base_name,
                });
            }
        };



        quote! {

            ##[derive(Debug, Clone)]
            pub enum #request_enum_name {
                #request_enum_variants
            }

            ##[derive(Debug, Clone)]
            pub enum #response_enum_name {
                #response_enum_variants
            }

        }

        


    }

    /// Gets the names of all message structs and the optional response names
    fn flatten_all_name_structs(&self) -> Vec<FeagiRequestMessageRustStructNames> {
        let mut all_names: Vec<FeagiRequestMessageRustStructNames> = Vec::new();
        for structs in &self.read {
            all_names.push(structs.clone());
        }
        for names in &self.create {
            all_names.push(names.clone());
        }
        for names in &self.edit {
            all_names.push(names.clone());
        }
        for names in &self.delete {
            all_names.push(names.clone());
        }
        for names in &self.patch {
            all_names.push(names.clone());
        }
        all_names
    }
}

//endregion

//region Feagi Message

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
    pub fn generate_feagi_message_struct_names(&self, name_base: syn::Ident) -> FeagiRequestMessageRustStructNames {
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
            method_base_title: named(method_str, ""),
            path: FeagiRequestMessagePath::<METHOD_TYPE_U8>::get_rust_struct_name(&name_base),
            parameter: present_name(&parameters, method_str, "FeagiMessageParameters"),
            queryable: present_name(&queryables, method_str, "FeagiMessageQueryables"),
            payload: present_name(&self.payload, method_str, "FeagiMessagePayload"),
            response: present_name(&self.response, method_str,"FeagiMessageResponse"),
            message: named("method_str", "FeagiMessage"),
        }
    }

    /// Generate the Specification structs, the feagi request message struct, and the response struct (if valid)
    pub fn generate_feagi_message_structs(&self, struct_names: FeagiRequestMessageRustStructNames) -> proc_macro2::TokenStream {
        let mut output: proc_macro2::TokenStream = proc_macro2::TokenStream::new();
        let FeagiRequestMessageRustStructNames {
            method_base_title,
            path: name_path,
            parameter: parameter_name,
            queryable: queryable_name,
            payload: payload_name,
            response: response_name,
            message: name_message,
        } = struct_names;
        let name_null = format_ident!("{}", "NullMessageSpecification");
        let specification_derives = vec![format_ident!("{}", "Clone"), format_ident!("{}", "Debug"), format_ident!("{}", "Serialize"), format_ident!("{}", "DeserializeOwned")];

        let tokens_message_path = self.path.generate_feagi_message_path_struct_and_impls(&name_path, &name_message);

        output.extend(tokens_message_path);

        let mut all_request_fields = StructTemplate::new_empty();

        fn generate_specification(
            mut struct_template: StructTemplate,
            path_item_descriptions: &PropertyDescriptors,
            struct_name: Option<&syn::Ident>,
            trait_suffix: &str,
            name_null: &syn::Ident,
            specification_derives: &Vec<syn::Ident>,
            all_request_fields: &mut StructTemplate,
            include_in_message: bool,
        ) -> (syn::Ident, proc_macro2::TokenStream, Vec<syn::Ident>) {
            struct_template.overwrite_descriptions_from_property_descriptors(path_item_descriptions);
            // `None` is the empty template, which reuses `NullMessageSpecification`.
            let (name, tokens) = if let Some(struct_name) = struct_name {
                let name = struct_name.clone();
                let mut tokens = struct_template.generate_rust_struct(
                    name.clone(), None, specification_derives, RustVisibility::Public
                );
                let trait_name = format_ident!("{}", trait_suffix);
                tokens.extend(quote! { impl #trait_name for #name });
                (name, tokens)
            } else {
                (name_null.clone(), proc_macro2::TokenStream::new())
            };
            if include_in_message {
                for (k, (t, d)) in struct_template.get_map() {
                    all_request_fields.insert_property(k.clone(), t.clone(), d.clone());
                }
            }
            let field_names: Vec<syn::Ident> = struct_template.get_map().keys().cloned().collect();
            (name, tokens, field_names)
        }

        fn moved_fields(owner: &syn::Ident, fields: &[syn::Ident]) -> proc_macro2::TokenStream {
            let mut tokens = proc_macro2::TokenStream::new();
            for field in fields {
                tokens.extend(quote! { #field: #owner.#field, });
            }
            tokens
        }

        let (name_parameters, tokens_parameters, parameter_fields) = generate_specification(
            self.path.as_parameter_struct_template(),
            &self.path_item_descriptions,
            parameter_name.as_ref(),
            "FeagiMessageParameters",
            &name_null,
            &specification_derives,
            &mut all_request_fields,
            true,
        );
        output.extend(tokens_parameters);

        let (name_queryables, tokens_queryables, queryable_fields) = generate_specification(
            self.path.as_queryable_struct_template(),
            &self.path_item_descriptions,
            queryable_name.as_ref(),
            "FeagiMessageQueryables",
            &name_null,
            &specification_derives,
            &mut all_request_fields,
            true,
        );
        output.extend(tokens_queryables);

        // NOTE: since we do validation that theres no payload when NO_PAYLOAD, we can always trust
        // that this will return empty / NULL in those cases
        let (name_payload, tokens_payload, payload_fields) = generate_specification(
            self.payload.clone(),
            &self.path_item_descriptions,
            payload_name.as_ref(),
            "FeagiMessagePayload",
            &name_null,
            &specification_derives,
            &mut all_request_fields,
            true,
        );
        output.extend(tokens_payload);

        let (name_response, tokens_response, _) = generate_specification(
            self.response.clone(),
            &self.path_item_descriptions,
            response_name.as_ref(),
            "FeagiMessageResponse",
            &name_null,
            &specification_derives,
            &mut all_request_fields,
            false,
        );
        output.extend(tokens_response);


        let feagi_message_derives = vec![format_ident!("{}", "Clone"), format_ident!("{}", "Debug")]; // doesn't need serialization
        let feagi_message_tokens = all_request_fields.generate_rust_struct(
            name_message.clone(), Some(self.description.clone()), &feagi_message_derives, RustVisibility::Public
        );

        output.extend(feagi_message_tokens);

        let parameters_binding = format_ident!("parameters");
        let queryables_binding = format_ident!("queryables");
        let payload_binding = format_ident!("payload");
        let self_binding = format_ident!("self");
        let from_parameters = moved_fields(&parameters_binding, &parameter_fields);
        let from_queryables = moved_fields(&queryables_binding, &queryable_fields);
        let from_payload = moved_fields(&payload_binding, &payload_fields);
        let to_parameters = moved_fields(&self_binding, &parameter_fields);
        let to_queryables = moved_fields(&self_binding, &queryable_fields);
        let to_payload = moved_fields(&self_binding, &payload_fields);
        // An empty specification is `NullMessageSpecification` and adds no fields.
        // Name the unused binding so the generated function does not warn.
        let consume_parameters = if parameter_fields.is_empty() { quote!(let _ = parameters;) } else { quote!() };
        let consume_queryables = if queryable_fields.is_empty() { quote!(let _ = queryables;) } else { quote!() };
        let consume_payload = if payload_fields.is_empty() { quote!(let _ = payload;) } else { quote!() };
        let consume_self = if parameter_fields.is_empty() && queryable_fields.is_empty() && payload_fields.is_empty() {
            quote!(let _ = self;)
        } else {
            quote!()
        };

        let mut new_arguments = proc_macro2::TokenStream::new();
        let mut new_members = proc_macro2::TokenStream::new();
        for (name, (field_type, _)) in all_request_fields.get_map() {
            new_arguments.extend(quote! { #name: #field_type, });
            new_members.extend(quote! { #name, });
        }

        output.extend(quote!{

            impl #name_message {
                pub fn new(#new_arguments) -> Self {
                    Self {
                        #new_members
                    }
                }
            }

            impl FeagiMessage for #name_message {
                type Path = #name_path;
                type Parameters = #name_parameters;
                type Queryables = #name_queryables;
                type Payload = #name_payload;

                fn from_message_data(parameters: Self::Parameters, queryables: Self::Queryables, payload: Self::Payload) -> Self {
                    #consume_parameters
                    #consume_queryables
                    #consume_payload
                    Self {
                        #from_parameters
                        #from_queryables
                        #from_payload
                    }
                }

                fn to_message_data(self) -> (Self::Parameters, Self::Queryables, Self::Payload) {
                    #consume_self
                    (
                        Self::Parameters { #to_parameters },
                        Self::Queryables { #to_queryables },
                        Self::Payload { #to_payload },
                    )
                }
            }

            impl FeagiMessageWithResponse for #name_message {
                type Response = #name_response;
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

        quote! {
            {
                path: #path,
                path_item_descriptions: #path_item_descriptions,
                description: #description,
                payload: #payload,
                response: #response,
            }
        }
    }
}

/// Contains all the names of the rust structs that would be generated from a FeagiMessage
#[derive(Clone)]
pub struct FeagiRequestMessageRustStructNames {
    /// The base title of all of these structs with the prefix of the method (Pascal)
    pub method_base_title: syn::Ident,
    pub path: syn::Ident,
    pub parameter: Option<syn::Ident>,
    pub queryable: Option<syn::Ident>,
    pub payload: Option<syn::Ident>,
    pub response: Option<syn::Ident>,
    pub message: syn::Ident,
}

//endregion

//region FeagiRequestMessagePath

/// Parses over a single string to get the pathing out, with the formatting:
/// "static_1/static_2/:parameter_1<ParameterType1>/static_3?query_1<QueryType1>&query_2<QueryType2>"
pub struct FeagiRequestMessagePath<const METHOD_U8: FeagiRequestMessageU8> {
    leaf: Vec<MessagePathElement>,
}

impl<const METHOD: FeagiRequestMessageU8> FeagiRequestMessagePath<METHOD> {

    pub fn get_rust_struct_name(name_base: &syn::Ident) -> syn::Ident {
        format_ident!("{}{}{}", FeagiRequestMessageMethod::from_u8(METHOD).as_str(), name_base, "FeagiMessagePath")
    }

    pub fn full_message_path(&self, category: &LitStr) -> LitStr {
        let path = self.as_lit_str();
        LitStr::new(&format!("{}/{}", category.value(), path.value()), Span::call_site())
    }

    // TODO full_message_path_with_method(&self, category: &LitStr) -> LitStr should append the method at the end

    pub fn as_lit_str(&self) -> LitStr {
        let string_out: String = self.leaf.iter().map(|l| l.to_string()).collect::<Vec<_>>().join("/");
        LitStr::new(&string_out, Span::call_site())
    }

    /// Outputs the parameters as a StructTemplate (No comments)
    pub fn as_parameter_struct_template(&self) -> StructTemplate {
        let mut output = StructTemplate::new_empty();
        for element in &self.leaf {
            match element {
                MessagePathElement::ParameterOfName(n, p) => {
                    let ident = lit_str_to_ident(n).unwrap();
                    output.insert_property(ident, p.clone(), None);
                }
                _ => {} // do nothing
            }
        };
        output
    }

    /// Outputs the queryable as a StructTemplate (No comments)
    pub fn as_queryable_struct_template(&self) -> StructTemplate {
        let mut output = StructTemplate::new_empty();
        for element in &self.leaf {
            match element {
                MessagePathElement::QueryableOfName(n, q) => {
                    let ident = lit_str_to_ident(n).unwrap();
                    output.insert_property(ident, q.clone(), None);
                }
                _ => {} // do nothing
            }
        };
        output
    }

    /// Generate the FeagiMessagePath impl struct rust code for this path
    pub fn generate_feagi_message_path_struct_and_impls(&self, full_struct_name: &syn::Ident, name_of_message_struct: &syn::Ident) -> proc_macro2::TokenStream {
        let mut output = proc_macro2::TokenStream::new();
        let elements: Vec<proc_macro2::TokenStream> = self.leaf.iter().map(MessagePathElement::as_rust_message_path_element_enum).collect();

        output.extend(quote! {

            ##[derive(Clone, Copy, Debug)]
            pub struct #full_struct_name;

            impl FeagiMessagePath for #full_struct_name {
                const PATH: &'static [MessagePathElement] = &[ #( #elements )* ];

                type Message = #name_of_message_struct;
            }
        });

        output

    }
}

impl<const METHOD: FeagiRequestMessageU8> Parse for FeagiRequestMessagePath<METHOD> {
    /// Parse `"static/:parameter<Type>/static?query<Type>&query<Type>"` into path elements.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path_str: LitStr = input.parse()?;
        let leaf = message_path_elements(&path_str)?;
        Ok(Self { leaf })
    }
}

impl<const METHOD: FeagiRequestMessageU8> Unparse for FeagiRequestMessagePath<METHOD> {
    fn unparse(&self) -> proc_macro2::TokenStream {
        let string_out = self.as_lit_str();
        quote! { #string_out }
    }
}

/// Split a path literal into static segments, parameters, and queries.
///
/// `/`, `?`, and `&` separate elements only outside `<...>`, so a type such as
/// `Vec<&str>` stays inside the element it belongs to. `>` in `->` is not a
/// closing angle bracket.
fn message_path_elements(path_str: &LitStr) -> syn::Result<Vec<MessagePathElement>> {
    let span = path_str.span();
    let raw = path_str.value();
    if raw.is_empty() {
        return Err(syn::Error::new(span, "FEAGI message path must not be empty"));
    }

    let mut leaf = Vec::new();
    let mut depth = 0usize;
    let mut prev: Option<char> = None;
    let mut in_query = false;
    let mut part_start = 0usize;

    for (index, ch) in raw.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' if prev != Some('-') => {
                if depth == 0 {
                    return Err(syn::Error::new(span, "FEAGI message path has '>' without a matching '<'"));
                }
                depth -= 1;
            }
            '/' if depth == 0 => {
                if in_query {
                    return Err(syn::Error::new(span, "FEAGI message path query string cannot contain '/'"));
                }
                push_path_part(&mut leaf, &raw[part_start..index], false, span)?;
                part_start = index + ch.len_utf8();
            }
            '?' if depth == 0 => {
                if in_query {
                    return Err(syn::Error::new(span, "FEAGI message path has more than one '?'"));
                }
                push_path_part(&mut leaf, &raw[part_start..index], false, span)?;
                in_query = true;
                part_start = index + ch.len_utf8();
            }
            '&' if depth == 0 => {
                if !in_query {
                    return Err(syn::Error::new(span, "FEAGI message path uses '&' only between query items"));
                }
                push_path_part(&mut leaf, &raw[part_start..index], true, span)?;
                part_start = index + ch.len_utf8();
            }
            _ => {}
        }
        prev = Some(ch);
    }

    if depth != 0 {
        return Err(syn::Error::new(span, "FEAGI message path has an unclosed '<'"));
    }
    push_path_part(&mut leaf, &raw[part_start..], in_query, span)?;
    Ok(leaf)
}

/// Append one `/` segment, or one query item when `in_query` is set.
fn push_path_part(leaf: &mut Vec<MessagePathElement>, part: &str, in_query: bool, span: Span) -> syn::Result<()> {
    if part.is_empty() {
        let message = if in_query {
            "FEAGI message path contains an empty query"
        } else {
            "FEAGI message path contains an empty segment"
        };
        return Err(syn::Error::new(span, message));
    }
    if in_query {
        leaf.push(parse_query(part, span)?);
    } else {
        leaf.push(parse_path_segment(part, span)?);
    }
    Ok(())
}

/// Parse `static_name` or `:parameter_name<Type>`.
fn parse_path_segment(segment: &str, span: Span) -> syn::Result<MessagePathElement> {
    if let Some(spec) = segment.strip_prefix(':') {
        let (name, parameter_type) = parse_name_and_type(spec, span)?;
        return Ok(MessagePathElement::ParameterOfName(LitStr::new(name, span), parameter_type));
    }

    if segment.chars().any(|ch| matches!(ch, '<' | '>' | '&' | ':') || ch.is_whitespace()) {
        return Err(syn::Error::new(
            span,
            format!("static path segment `{segment}` contains path syntax; parameters use `:name<Type>`"),
        ));
    }

    Ok(MessagePathElement::StaticPath(LitStr::new(segment, span)))
}

/// Parse `query_name<Type>`.
fn parse_query(spec: &str, span: Span) -> syn::Result<MessagePathElement> {
    if spec.starts_with(':') {
        return Err(syn::Error::new(
            span,
            format!("query `{spec}` must be `name<Type>` without a leading ':'"),
        ));
    }
    let (name, query_type) = parse_name_and_type(spec, span)?;
    Ok(MessagePathElement::QueryableOfName(LitStr::new(name, span), query_type))
}

/// Split `name<Type>` and parse both sides. `Type` may itself contain angle brackets.
fn parse_name_and_type(spec: &str, span: Span) -> syn::Result<(&str, Type)> {
    let (name, type_src) = split_name_and_type(spec, span)?;
    syn::parse_str::<syn::Ident>(name).map_err(|_| syn::Error::new(span, format!("`{name}` is not a valid path name")))?;
    let parsed_type =
        syn::parse_str::<Type>(type_src).map_err(|err| syn::Error::new(span, format!("invalid type `{type_src}` in FEAGI message path: {err}")))?;
    Ok((name, parsed_type))
}

/// Return the name and the source inside the outer `<...>` pair.
fn split_name_and_type(spec: &str, span: Span) -> syn::Result<(&str, &str)> {
    let Some(open) = spec.find('<') else {
        return Err(syn::Error::new(span, format!("`{spec}` is missing a <Type>")));
    };
    let name = &spec[..open];
    if name.is_empty() {
        return Err(syn::Error::new(span, format!("`{spec}` is missing a name before <Type>")));
    }

    let mut depth = 0usize;
    let mut close_at = None;
    let mut prev: Option<char> = None;
    for (offset, ch) in spec[open..].char_indices() {
        match ch {
            '<' => depth += 1,
            '>' if prev != Some('-') => {
                depth -= 1;
                if depth == 0 {
                    close_at = Some(open + offset);
                    break;
                }
            }
            _ => {}
        }
        prev = Some(ch);
    }

    let Some(close_at) = close_at else {
        return Err(syn::Error::new(span, format!("`{spec}` is missing a closing '>'")));
    };
    if close_at + 1 != spec.len() {
        return Err(syn::Error::new(span, format!("`{spec}` has tokens after the type")));
    }
    let type_src = &spec[open + 1..close_at];
    if type_src.is_empty() {
        return Err(syn::Error::new(span, format!("`{spec}` has an empty type")));
    }
    Ok((name, type_src))
}

//endregion

//region Request Method

/// Have to do this because of compiler limits
pub type FeagiRequestMessageU8 = u8;

/// Represents the method of the request (equivalent to HTTP Get / Put / etc)
#[repr(u8)]
pub enum FeagiRequestMessageMethod {
    Read = Self::READ, // GET
    Create = Self::CREATE, // PUT
    Edit = Self::EDIT, // POST
    Delete = Self::DELETE, // DELETE
    Patch = Self::PATCH // PATCH
}

impl FeagiRequestMessageMethod {
    pub const READ: u8 = 0;
    pub const CREATE: u8 = 1;
    pub const EDIT: u8 = 2;
    pub const DELETE: u8 = 3;
    pub const PATCH: u8 = 4;


    pub const fn had_no_payload(&self) -> bool {
        match self {
            FeagiRequestMessageMethod::Read => {true}
            _ => {false}
        }
    }

    pub const fn from_u8(u: u8) -> Self {
        match u {
            Self::READ => Self::Read,
            Self::CREATE => Self::Create,
            Self::EDIT => Self::Edit,
            Self::DELETE => Self::Delete,
            Self::PATCH => Self::Patch,
            _ => panic!("Invalid Method!")
        }
    }

    pub const fn to_u8(self) -> u8 {
        match self {
            FeagiRequestMessageMethod::Read => {Self::READ}
            FeagiRequestMessageMethod::Create => {Self::CREATE}
            FeagiRequestMessageMethod::Edit => {Self::EDIT}
            FeagiRequestMessageMethod::Delete => {Self::DELETE}
            FeagiRequestMessageMethod::Patch => {Self::PATCH}
        }
    }
    
    pub const fn as_str(&self) -> &'static str {
        match self {
            FeagiRequestMessageMethod::Read => {"Read"}
            FeagiRequestMessageMethod::Create => {"Crate"}
            FeagiRequestMessageMethod::Edit => {"Edit"}
            FeagiRequestMessageMethod::Delete => {"Delete"}
            FeagiRequestMessageMethod::Patch => {"Patch"}
        }
    }
}

//endregion