use crate::common::{parse_optional_comma, parse_property_colon_member};
use crate::feagi_messages::request_response::common_structs::{FeagiRequestMessageMethod, FeagiRequestMessageU8};
use crate::feagi_messages::request_response::feagi_request_message::FeagiRequestMessage;
use crate::templates::Unparse;
use heck::{ToPascalCase, ToSnakeCase};
use proc_macro2::Span;
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::parse::{Parse, ParseStream};
use syn::{braced, LitStr, Token};

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

//region Feagi Request Message Category Specification Struct Names

/// Contains the names of the rust structs that a category itself produces.
/// Child message names come from [`FeagiRequestMessage`], which holds those messages.
#[derive(Clone)]
pub struct FeagiRequestMessageCategoryRustStructNames {
    pub base_name: syn::Ident,
}

impl FeagiRequestMessageCategoryRustStructNames {
    /// Build the name set from the category title.
    pub fn new(base_name: syn::Ident) -> Self {
        Self { base_name }
    }

    pub fn category_mappings_struct_name(&self) -> syn::Ident {
        format_ident!("{}FeagiRequestMessageCategoryMappings", self.base_name)
    }

    pub fn request_enum_name(&self) -> syn::Ident {
        format_ident!("{}Requests", self.base_name)
    }

    pub fn response_enum_name(&self) -> syn::Ident {
        format_ident!("{}Responses", self.base_name)
    }
}

//endregion

//region Feagi Request Message Category

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
    /// Struct names this category will generate. Child message names are queried
    /// from the messages this category holds, not from the category name set.
    pub fn calculate_feagi_category_struct_names(&self) -> FeagiRequestMessageCategoryRustStructNames {
        FeagiRequestMessageCategoryRustStructNames::new(format_ident!("{}", self.category_name.value().to_pascal_case()))
    }

    /// Create the specification data, feagi messages, and response structs for all internal categorized feagi messages
    pub fn generate_rust_specification_structs_for_all_messages(&self) -> proc_macro2::TokenStream {
        let mut output = proc_macro2::TokenStream::new();

        for read in &self.read {
            let read_name = format_ident!("{}", read.0.value());
            let request_message = read.1;
            let struct_names = request_message.calculate_feagi_message_struct_names(read_name);
            output.extend(request_message.generate_rust_specification_and_request_and_response_structs(struct_names));
        }

        for create in &self.create {
            let create_name = format_ident!("{}", create.0.value());
            let request_message = create.1;
            let struct_names = request_message.calculate_feagi_message_struct_names(create_name);
            output.extend(request_message.generate_rust_specification_and_request_and_response_structs(struct_names));
        }

        for edit in &self.edit {
            let edit_name = format_ident!("{}", edit.0.value());
            let request_message = edit.1;
            let struct_names = request_message.calculate_feagi_message_struct_names(edit_name);
            output.extend(request_message.generate_rust_specification_and_request_and_response_structs(struct_names));
        }

        for delete in &self.delete {
            let delete_name = format_ident!("{}", delete.0.value());
            let request_message = delete.1;
            let struct_names = request_message.calculate_feagi_message_struct_names(delete_name);
            output.extend(request_message.generate_rust_specification_and_request_and_response_structs(struct_names));
        }

        for patch in &self.patch {
            let patch_name = format_ident!("{}", patch.0.value());
            let request_message = patch.1;
            let struct_names = request_message.calculate_feagi_message_struct_names(patch_name);
            output.extend(request_message.generate_rust_specification_and_request_and_response_structs(struct_names));
        }

        output
    }

    /// Generate the rust enums that contain all the Requests with their Feagi Request Messages, and the Responses with their response datas.
    /// Variant existence and payload types come from the messages this category holds.
    pub fn generate_rust_request_response_enums_for_category(&self) -> proc_macro2::TokenStream {
        let category_names = self.calculate_feagi_category_struct_names();
        let request_enum_name = category_names.request_enum_name();
        let response_enum_name = category_names.response_enum_name();

        let mut request_enum_variants = proc_macro2::TokenStream::new();
        let mut response_enum_variants = proc_macro2::TokenStream::new();
        let mut intos = proc_macro2::TokenStream::new();

        Self::extend_request_response_enum_variants(
            &self.read,
            &request_enum_name,
            &response_enum_name,
            &mut request_enum_variants,
            &mut response_enum_variants,
            &mut intos,
        );
        Self::extend_request_response_enum_variants(
            &self.create,
            &request_enum_name,
            &response_enum_name,
            &mut request_enum_variants,
            &mut response_enum_variants,
            &mut intos,
        );
        Self::extend_request_response_enum_variants(
            &self.edit,
            &request_enum_name,
            &response_enum_name,
            &mut request_enum_variants,
            &mut response_enum_variants,
            &mut intos,
        );
        Self::extend_request_response_enum_variants(
            &self.delete,
            &request_enum_name,
            &response_enum_name,
            &mut request_enum_variants,
            &mut response_enum_variants,
            &mut intos,
        );
        Self::extend_request_response_enum_variants(
            &self.patch,
            &request_enum_name,
            &response_enum_name,
            &mut request_enum_variants,
            &mut response_enum_variants,
            &mut intos,
        );

        quote! {

            #[derive(Debug, Clone)]
            pub enum #request_enum_name {
                #request_enum_variants
            }

            #[derive(Debug, Clone)]
            pub enum #response_enum_name {
                #response_enum_variants
            }

            #intos
        }
    }

    fn extend_request_response_enum_variants<const METHOD: FeagiRequestMessageU8>(
        messages: &HashMap<LitStr, FeagiRequestMessage<METHOD>>,
        request_enum_name: &syn::Ident,
        response_enum_name: &syn::Ident,
        request_enum_variants: &mut proc_macro2::TokenStream,
        response_enum_variants: &mut proc_macro2::TokenStream,
        intos: &mut proc_macro2::TokenStream,
    ) {
        let method = FeagiRequestMessageMethod::from_u8(METHOD);
        for (title, message) in messages {
            let message_names = message.calculate_feagi_message_struct_names(format_ident!("{}", title.value()));
            let variant_name = format_ident!("{}{}", method.as_str(), message_names.base_name);
            let message_struct_name = message_names.feagi_message_response_struct_name();

            request_enum_variants.extend(quote! {
                #variant_name(#message_struct_name),
            });

            intos.extend(quote! {
                impl Into<#request_enum_name> for #message_struct_name {
                    fn into(self) -> #request_enum_name {
                        #request_enum_name::#variant_name(self)
                    }
                }
            });

            if message.is_response_struct_generated() {
                let response_name = message_names.feagi_message_response_specification_struct_name();
                response_enum_variants.extend(quote! {
                    #variant_name(#response_name),
                });

                intos.extend(quote! {
                    impl Into<#response_enum_name> for #response_name {
                        fn into(self) -> #response_enum_name {
                            #response_enum_name::#variant_name(self)
                        }
                    }
                });
            } else {
                response_enum_variants.extend(quote! {
                    #variant_name,
                });
            }
        }
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

//endregion
