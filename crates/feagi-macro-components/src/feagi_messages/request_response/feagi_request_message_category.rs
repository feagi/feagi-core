use crate::common::{parse_optional_comma, parse_property_colon_member};
use crate::feagi_messages::request_response::feagi_request_message_path::FeagiRequestMessagePath;
use crate::templates::Unparse;
use proc_macro2::Span;
use quote::{format_ident, quote};
use std::collections::HashMap;
use heck::{ToPascalCase, ToSnakeCase};
use syn::parse::{Parse, ParseStream};
use syn::{braced, LitStr, Token};
use crate::feagi_messages::request_response::common_structs::FeagiRequestMessageMethod;
use crate::feagi_messages::request_response::feagi_request_message::{FeagiRequestMessage, FeagiRequestMessageRustStructNames};

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
}

/// Contains the names of the structs across all feagi messages in a feagi message category
pub struct FeagiRequestMessageCategoryStructNames {
    pub read: Vec<FeagiRequestMessageRustStructNames>,
    pub create: Vec<FeagiRequestMessageRustStructNames>,
    pub edit: Vec<FeagiRequestMessageRustStructNames>,
    pub delete: Vec<FeagiRequestMessageRustStructNames>,
    pub patch: Vec<FeagiRequestMessageRustStructNames>,
}

impl FeagiRequestMessageCategoryStructNames {
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
            FeagiRequestMessageMethod::Read => self.read.as_slice(),
            FeagiRequestMessageMethod::Create => self.create.as_slice(),
            FeagiRequestMessageMethod::Edit => self.edit.as_slice(),
            FeagiRequestMessageMethod::Delete => self.delete.as_slice(),
            FeagiRequestMessageMethod::Patch => self.patch.as_slice(),
        }
    }

    pub fn get_by_method_mut(&mut self, method: &FeagiRequestMessageMethod) -> &mut Vec<FeagiRequestMessageRustStructNames> {
        match method {
            FeagiRequestMessageMethod::Read => &mut self.read,
            FeagiRequestMessageMethod::Create => &mut self.create,
            FeagiRequestMessageMethod::Edit => &mut self.edit,
            FeagiRequestMessageMethod::Delete => &mut self.delete,
            FeagiRequestMessageMethod::Patch => &mut self.patch,
        }
    }

    /// Generate the rust enums that contain all the Requests with their Feagi Request Messages, and the Responses with their response datas
    pub fn generate_rust_request_response_enums_for_category(
        &self,
        request_enum_name: syn::Ident,
        response_enum_name: syn::Ident,
    ) -> proc_macro2::TokenStream {
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

            intos.extend(quote! {
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

                intos.extend(quote! {
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
        }

        quote! {

            #[derive(Debug, Clone)]
            pub enum #request_enum_name {
                #request_enum_variants
            }

            #[derive(Debug, Clone)]
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
pub struct FeagiRequestMessageCategory {
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

impl FeagiRequestMessageCategory {
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

            properties.extend(quote! {
                #[doc = #description]
                pub #message_field_name: (#message_path_name, T),
            });
        }

        for (name, request) in &self.create {
            let message_struct_name = format_ident!("{}", name.value());
            let message_field_name = format_ident!("{}", name.value().to_snake_case());
            let message_path_name = FeagiRequestMessagePath::<0>::get_rust_struct_name(&message_struct_name);
            let description = &request.description;

            properties.extend(quote! {
                #[doc = #description]
                pub #message_field_name: (#message_path_name, T),
            });
        }

        for (name, request) in &self.edit {
            let message_struct_name = format_ident!("{}", name.value());
            let message_field_name = format_ident!("{}", name.value().to_snake_case());
            let message_path_name = FeagiRequestMessagePath::<0>::get_rust_struct_name(&message_struct_name);
            let description = &request.description;

            properties.extend(quote! {
                #[doc = #description]
                pub #message_field_name: (#message_path_name, T),
            });
        }

        for (name, request) in &self.delete {
            let message_struct_name = format_ident!("{}", name.value());
            let message_field_name = format_ident!("{}", name.value().to_snake_case());
            let message_path_name = FeagiRequestMessagePath::<0>::get_rust_struct_name(&message_struct_name);
            let description = &request.description;

            properties.extend(quote! {
                #[doc = #description]
                pub #message_field_name: (#message_path_name, T),
            });
        }

        for (name, request) in &self.patch {
            let message_struct_name = format_ident!("{}", name.value());
            let message_field_name = format_ident!("{}", name.value().to_snake_case());
            let message_path_name = FeagiRequestMessagePath::<0>::get_rust_struct_name(&message_struct_name);
            let description = &request.description;

            properties.extend(quote! {
                #[doc = #description]
                pub #message_field_name: (#message_path_name, T),
            });
        }

        let struct_name = format_ident!("{}FeagiRequestMessageCategoryMappings", &self.category_name.value().to_pascal_case());
        let docs = &self.category_description;

        quote! {
            #[doc = #docs]
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
            output.extend(request_message.generate_rust_specification_and_request_and_response_structs(struct_names));
        }

        for create in &self.create {
            let create_name = format_ident!("{}", create.0.value());
            let request_message = create.1;
            let struct_names = request_message.generate_feagi_message_struct_names(create_name);
            output.extend(request_message.generate_rust_specification_and_request_and_response_structs(struct_names));
        }

        for edit in &self.edit {
            let edit_name = format_ident!("{}", edit.0.value());
            let request_message = edit.1;
            let struct_names = request_message.generate_feagi_message_struct_names(edit_name);
            output.extend(request_message.generate_rust_specification_and_request_and_response_structs(struct_names));
        }

        for delete in &self.delete {
            let delete_name = format_ident!("{}", delete.0.value());
            let request_message = delete.1;
            let struct_names = request_message.generate_feagi_message_struct_names(delete_name);
            output.extend(request_message.generate_rust_specification_and_request_and_response_structs(struct_names));
        }

        for patch in &self.patch {
            let patch_name = format_ident!("{}", patch.0.value());
            let request_message = patch.1;
            let struct_names = request_message.generate_feagi_message_struct_names(patch_name);
            output.extend(request_message.generate_rust_specification_and_request_and_response_structs(struct_names));
        }

        output
    }

    /// Create the categorized names of the feagi request messages from all internal feagi request messages
    pub fn create_categorized_rust_struct_names(&self) -> FeagiRequestMessageCategoryStructNames {
        let mut output = FeagiRequestMessageCategoryStructNames::new_empty();

        for (message_name, message) in &self.read {
            let message_name = format_ident!("{}", message_name.value());
            let message_names = message.generate_feagi_message_struct_names(message_name);
            output.read.push(message_names);
        }

        for (message_name, message) in &self.create {
            let message_name = format_ident!("{}", message_name.value());
            let message_names = message.generate_feagi_message_struct_names(message_name);
            output.create.push(message_names);
        }

        for (message_name, message) in &self.edit {
            let message_name = format_ident!("{}", message_name.value());
            let message_names = message.generate_feagi_message_struct_names(message_name);
            output.edit.push(message_names);
        }

        for (message_name, message) in &self.delete {
            let message_name = format_ident!("{}", message_name.value());
            let message_names = message.generate_feagi_message_struct_names(message_name);
            output.delete.push(message_names);
        }

        for (message_name, message) in &self.patch {
            let message_name = format_ident!("{}", message_name.value());
            let message_names = message.generate_feagi_message_struct_names(message_name);
            output.patch.push(message_names);
        }
        output
    }
}

impl Parse for FeagiRequestMessageCategory {
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

impl Unparse for FeagiRequestMessageCategory {
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